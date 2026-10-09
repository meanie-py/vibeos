use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{_EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, PHANDLE, PLSA_OPERATIONAL_MODE, PLSA_STRING, PLUID, PNTSTATUS, PPORT_MESSAGE, PQUOTA_LIMITS, PTOKEN_GROUPS, PTOKEN_SOURCE, RtlInitUnicodeString, SECURITY_QUALITY_OF_SERVICE, ZwClose, ZwConnectPort, ZwCreateEvent, ZwFreeVirtualMemory, ZwOpenEvent, ZwRequestWaitReplyPort, ZwWaitForSingleObject,
    HANDLE, CSHORT, _UNICODE_STRING, PUNICODE_STRING, SECURITY_LOGON_TYPE,
    _SECURITY_IMPERSONATION_LEVEL_SecurityIdentification as SecurityIdentification,
    _SECURITY_IMPERSONATION_LEVEL_SecurityAnonymous as SecurityAnonymous};
// LSA LPC message substrate (subsys/lsass/lsass.h) — not in ntkrnl-sys.
use lsass_sys::{
    LSA_API_MSG, LSA_CONNECTION_INFO,
    LSA_LOGON_USER_MSG, LSA_CALL_AUTHENTICATION_PACKAGE_MSG,
    LSA_DEREGISTER_LOGON_PROCESS_MSG, LSA_LOOKUP_AUTHENTICATION_PACKAGE_MSG,
    LSASS_MAX_PACKAGE_NAME_LENGTH, LSASS_MAX_LOGON_PROCESS_NAME_LENGTH,
    // NO/YES/CHECK are LSA_TRUSTED_CALLER *enum variants* (NO=0, YES=1, CHECK=2),
    // NOT macros — the grinder fabricated them as `const CHECK: u8 = 1` (wrong
    // value AND wrong type). Import the real u32 variant constants.
    _LSA_TRUSTED_CALLER_NO as NO, _LSA_TRUSTED_CALLER_CHECK as CHECK,
    _LSA_API_NUMBER_LSASS_REQUEST_CALL_AUTHENTICATION_PACKAGE as LSASS_REQUEST_CALL_AUTHENTICATION_PACKAGE,
    _LSA_API_NUMBER_LSASS_REQUEST_DEREGISTER_LOGON_PROCESS as LSASS_REQUEST_DEREGISTER_LOGON_PROCESS,
    _LSA_API_NUMBER_LSASS_REQUEST_LOGON_USER as LSASS_REQUEST_LOGON_USER,
    _LSA_API_NUMBER_LSASS_REQUEST_LOOKUP_AUTHENTICATION_PACKAGE as LSASS_REQUEST_LOOKUP_AUTHENTICATION_PACKAGE,
};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}

// LSA_PORT_MESSAGE_SIZE is `sizeof(LSA_API_MSG)` (a macro bindgen can't eval).
const LSA_PORT_MESSAGE_SIZE: u32 = core::mem::size_of::<LSA_API_MSG>() as u32;

extern "C" {
    fn strncpy( dest: *mut ::core::ffi::c_char, src: *const ::core::ffi::c_char, n: ::core::ffi::c_ulong, ) -> *mut ::core::ffi::c_char;
}
extern "C" {
    fn DbgPrint(format: *const i8, ...) -> i32;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: bool = true;

const MEM_RELEASE: u32 = 0x8000;

// LogonProcessNameBuffer / PackageName are CHAR[] (i8); the NUL terminator must be i8.
const ANSI_NULL: i8 = 0;
const FALSE: u8 = 0;
// CHECK/NO and LSASS_MAX_LOGON_PROCESS_NAME_LENGTH now come from lsass-sys (see imports).
const STATUS_NAME_TOO_LONG: i32 = 0xC0000106u32 as i32;

#[no_mangle]
pub unsafe extern "system" fn LsaCallAuthenticationPackage(
    LsaHandle: HANDLE,
    AuthenticationPackage: ULONG,
    ProtocolSubmitBuffer: PVOID,
    SubmitBufferLength: ULONG,
    ProtocolReturnBuffer: *mut PVOID,
    ReturnBufferLength: PULONG,
    ProtocolStatus: PNTSTATUS,
) -> NTSTATUS {
    let mut ApiMessage: LSA_API_MSG = core::mem::zeroed();
    let Status: NTSTATUS;
    if DbgPrint(
        b"(%s:%d) LsaCallAuthenticationPackage()\n\0" as *const u8 as *const i8,
        b"sdk/lib/lsalib/lsa.c\0" as *const u8
            as *const ::core::ffi::c_char,
        41 as ::core::ffi::c_int,
    ) != 0 {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"sdk/lib/lsalib/lsa.c\0" as *const u8
                as *const ::core::ffi::c_char,
            41 as ::core::ffi::c_int,
        );
    }
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.ApiNumber = LSASS_REQUEST_CALL_AUTHENTICATION_PACKAGE;
    ApiMessage.h.u1.s1.DataLength = (core::mem::size_of::<ULONG>()
        + core::mem::size_of::<NTSTATUS>()
        + core::mem::size_of::<LSA_CALL_AUTHENTICATION_PACKAGE_MSG>())
        as CSHORT;
    ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE as CSHORT;
    ApiMessage.h.u2.ZeroInit = 0 as ULONG;
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
        .CallAuthenticationPackage
        .__bindgen_anon_1
        .Request
        .AuthenticationPackage = AuthenticationPackage;
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
        .CallAuthenticationPackage
        .__bindgen_anon_1
        .Request
        .ProtocolSubmitBuffer = ProtocolSubmitBuffer;
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
        .CallAuthenticationPackage
        .__bindgen_anon_1
        .Request
        .SubmitBufferLength = SubmitBufferLength;
    Status = ZwRequestWaitReplyPort(
        LsaHandle,
        &raw mut ApiMessage as PPORT_MESSAGE,
        &raw mut ApiMessage as PPORT_MESSAGE,
    );
    if !(Status >= 0) {
        if DbgPrint(
            b"(%s:%d) ZwRequestWaitReplyPort() failed (Status 0x%08lx)\n\0" as *const u8
                as *const i8,
            b"sdk/lib/lsalib/lsa.c\0" as *const u8
                as *const ::core::ffi::c_char,
            57 as ::core::ffi::c_int,
            Status,
        ) != 0 {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                b"sdk/lib/lsalib/lsa.c\0" as *const u8
                    as *const ::core::ffi::c_char,
                57 as ::core::ffi::c_int,
            );
        }
        return Status;
    }
    if !(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status >= 0) {
        if DbgPrint(
            b"(%s:%d) ZwRequestWaitReplyPort() failed (ApiMessage.__bindgen_anon_1.Status 0x%08lx)\n\0"
                as *const u8 as *const i8,
            b"sdk/lib/lsalib/lsa.c\0" as *const u8
                as *const ::core::ffi::c_char,
            63 as ::core::ffi::c_int,
            ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status,
        ) != 0 {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                b"sdk/lib/lsalib/lsa.c\0" as *const u8
                    as *const ::core::ffi::c_char,
                63 as ::core::ffi::c_int,
            );
        }
        return ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status;
    }
    *ProtocolReturnBuffer = ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
        .CallAuthenticationPackage
        .__bindgen_anon_1
        .Reply
        .ProtocolReturnBuffer;
    *ReturnBufferLength = ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
        .CallAuthenticationPackage
        .__bindgen_anon_1
        .Reply
        .ReturnBufferLength;
    *ProtocolStatus = ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
        .CallAuthenticationPackage
        .__bindgen_anon_1
        .Reply
        .ProtocolStatus as ::core::ffi::c_long;
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn LsaDeregisterLogonProcess(LsaHandle: HANDLE) -> NTSTATUS {
    let mut ApiMessage: LSA_API_MSG = core::mem::zeroed();
    let Status: NTSTATUS;

    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.ApiNumber = LSASS_REQUEST_DEREGISTER_LOGON_PROCESS;
    ApiMessage.h.u1.s1.DataLength = core::mem::size_of::<ULONG>()
        .wrapping_add(core::mem::size_of::<NTSTATUS>())
        .wrapping_add(core::mem::size_of::<LSA_DEREGISTER_LOGON_PROCESS_MSG>()) as CSHORT ;
    ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE as CSHORT;
    ApiMessage.h.u2.ZeroInit = 0 as ULONG;

    Status = ZwRequestWaitReplyPort(
        LsaHandle,
        &raw mut ApiMessage as PPORT_MESSAGE,
        &raw mut ApiMessage as PPORT_MESSAGE,
    );

    if !(Status >= 0) {
        if DbgPrint(
            b"(%s:%d) ZwRequestWaitReplyPort() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            b"sdk/lib/lsalib/lsa.c\0".as_ptr() as *const i8,
            97 as i32,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"sdk/lib/lsalib/lsa.c\0".as_ptr() as *const i8,
                97 as i32,
            );
        }
        return Status;
    }

    if !(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status >= 0) {
        if DbgPrint(
            b"(%s:%d) ZwRequestWaitReplyPort() failed (ApiMessage.__bindgen_anon_1.Status 0x%08lx)\n\0".as_ptr() as *const i8,
            b"sdk/lib/lsalib/lsa.c\0".as_ptr() as *const i8,
            103 as i32,
            ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"sdk/lib/lsalib/lsa.c\0".as_ptr() as *const i8,
                103 as i32,
            );
        }
        return ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status;
    }

    ZwClose(LsaHandle);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn LsaFreeReturnBuffer(Buffer: *mut core::ffi::c_void) -> i32 {
    // ZwFreeVirtualMemory takes the base pointer by reference (&Buffer in C); the
    // param itself is a local copy, so shadow it as mutable to take its address.
    let mut Buffer = Buffer;
    let mut Size: ntkrnl_sys::ddk::SIZE_T = 0; // u32 on i386, u64 on 64-bit
    return ZwFreeVirtualMemory(
        -1isize as *mut core::ffi::c_void ,
        core::ptr::addr_of_mut!(Buffer),
        core::ptr::addr_of_mut!(Size),
        MEM_RELEASE as u32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn LsaLookupAuthenticationPackage(
    LsaHandle: HANDLE,
    PackageName: PLSA_STRING,
    AuthenticationPackage: PULONG,
) -> NTSTATUS {
    let mut ApiMessage: LSA_API_MSG = core::mem::zeroed();
    let Status: NTSTATUS;

    if (*PackageName).Length as u32 > LSASS_MAX_PACKAGE_NAME_LENGTH {
        return STATUS_NAME_TOO_LONG;
    }

    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.ApiNumber = LSASS_REQUEST_LOOKUP_AUTHENTICATION_PACKAGE;
    ApiMessage.h.u1.s1.DataLength = (core::mem::size_of::<ULONG>()
        .wrapping_add(core::mem::size_of::<NTSTATUS>())
        .wrapping_add(core::mem::size_of::<LSA_LOOKUP_AUTHENTICATION_PACKAGE_MSG>()))
        as CSHORT;
    ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE as CSHORT;
    ApiMessage.h.u2.ZeroInit = 0 as ULONG;

    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
        .LookupAuthenticationPackage
        .__bindgen_anon_1
        .Request
        .PackageNameLength = (*PackageName).Length as ULONG;
    strncpy(
        &raw mut ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
            .LookupAuthenticationPackage
            .__bindgen_anon_1
            .Request
            .PackageName as *mut i8,
        (*PackageName).Buffer as *const i8,
        ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
            .LookupAuthenticationPackage
            .__bindgen_anon_1
            .Request
            .PackageNameLength as ::core::ffi::c_ulong,
    );
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
        .LookupAuthenticationPackage
        .__bindgen_anon_1
        .Request
        .PackageName[ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
        .LookupAuthenticationPackage
        .__bindgen_anon_1
        .Request
        .PackageNameLength as usize] = ANSI_NULL;

    Status = ZwRequestWaitReplyPort(
        LsaHandle,
        &raw mut ApiMessage as PPORT_MESSAGE,
        &raw mut ApiMessage as PPORT_MESSAGE,
    );

    if !(Status >= 0) {
        return Status;
    }

    if !(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status >= 0) {
        return ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status;
    }

    *AuthenticationPackage = ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LookupAuthenticationPackage.__bindgen_anon_1.Reply.Package;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaLogonUser(
    LsaHandle: HANDLE,
    OriginName: PLSA_STRING,
    LogonType: SECURITY_LOGON_TYPE,
    AuthenticationPackage: ULONG,
    AuthenticationInformation: PVOID,
    AuthenticationInformationLength: ULONG,
    LocalGroups: PTOKEN_GROUPS,
    SourceContext: PTOKEN_SOURCE,
    ProfileBuffer: *mut PVOID,
    ProfileBufferLength: PULONG,
    LogonId: PLUID,
    Token: PHANDLE,
    Quotas: PQUOTA_LIMITS,
    SubStatus: PNTSTATUS,
) -> NTSTATUS {
    let mut ApiMessage: LSA_API_MSG = unsafe { core::mem::zeroed() };
    let Status: NTSTATUS;

    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.ApiNumber = LSASS_REQUEST_LOGON_USER;
    ApiMessage.h.u1.s1.DataLength = (core::mem::size_of::<ULONG>()
        + core::mem::size_of::<NTSTATUS>()
        + core::mem::size_of::<LSA_LOGON_USER_MSG>()) as i16;
    ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE as i16;
    ApiMessage.h.u2.ZeroInit = 0;

    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Request.OriginName = core::mem::transmute(*OriginName);
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Request.LogonType = LogonType;
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Request.AuthenticationPackage = AuthenticationPackage;
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Request.AuthenticationInformation = AuthenticationInformation;
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Request.AuthenticationInformationLength = AuthenticationInformationLength;
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Request.LocalGroups = core::mem::transmute(LocalGroups);
    if !LocalGroups.is_null() {
        ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Request.LocalGroupsCount = (*LocalGroups).GroupCount;
    } else {
        ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Request.LocalGroupsCount = 0;
    }
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Request.SourceContext = core::mem::transmute(*SourceContext);

    Status = ZwRequestWaitReplyPort(
        LsaHandle,
        &raw mut ApiMessage as PPORT_MESSAGE,
        &raw mut ApiMessage as PPORT_MESSAGE,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    *SubStatus = ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Reply.SubStatus;

    if !NT_SUCCESS(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status) {
        return ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status;
    }

    *ProfileBuffer = ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Reply.ProfileBuffer;
    *ProfileBufferLength = ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Reply.ProfileBufferLength;
    *LogonId = core::mem::transmute(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Reply.LogonId);
    *Token = ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Reply.Token;
    *Quotas = core::mem::transmute(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser.__bindgen_anon_1.Reply.Quotas);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn LsaRegisterLogonProcess(
    LogonProcessName: PLSA_STRING,
    LsaHandle: PHANDLE,
    OperationalMode: PLSA_OPERATIONAL_MODE,
) -> NTSTATUS {
    let mut SecurityQos: SECURITY_QUALITY_OF_SERVICE = SECURITY_QUALITY_OF_SERVICE {
        Length: 0,
        ImpersonationLevel: SecurityAnonymous,
        ContextTrackingMode: 0,
        EffectiveOnly: 0,
    };
    let mut ConnectInfo: LSA_CONNECTION_INFO = LSA_CONNECTION_INFO {
        Status: 0,
        OperationalMode: 0,
        Length: 0,
        LogonProcessNameBuffer: [0; 128],
        CreateContext: 0,
        TrustedCaller: NO,
    };
    let mut ConnectInfoLength: u32 = ::core::mem::size_of::<LSA_CONNECTION_INFO>() as u32;
    let mut PortName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ::core::ptr::null_mut::<u16>(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ObjectName: ::core::ptr::null_mut::<_UNICODE_STRING>(),
        Attributes: 0,
        SecurityDescriptor: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        SecurityQualityOfService: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut EventName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ::core::ptr::null_mut::<u16>(),
    };
    let mut EventHandle: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut Status: NTSTATUS;

    if (*LogonProcessName).Length as i32 > LSASS_MAX_LOGON_PROCESS_NAME_LENGTH as i32 {
        return STATUS_NAME_TOO_LONG;
    }

    RtlInitUnicodeString(
        &raw mut EventName,
        w!("\\SECURITY\\LSA_AUTHENTICATION_INITIALIZED").as_ptr(),
    );

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut::<core::ffi::c_void>() as HANDLE;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as u32 | OBJ_PERMANENT as u32 ;
    ObjectAttributes.ObjectName = &raw mut EventName as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>() as *mut core::ffi::c_void;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>() as *mut core::ffi::c_void;

    Status = ZwOpenEvent(
        &raw mut EventHandle,
        SYNCHRONIZE as u32,
        &raw mut ObjectAttributes,
    );

    if !NT_SUCCESS(Status) {
        Status = ZwCreateEvent(
            &raw mut EventHandle,
            SYNCHRONIZE as u32,
            &raw mut ObjectAttributes,
            _EVENT_TYPE_NotificationEvent,
            FALSE,
        );
        if !NT_SUCCESS(Status) {
            if DbgPrint(
                b"(%s:%d) ZwCreateEvent failed (Status 0x%08lx)\n\0" as *const u8 as *const i8,
                b"sdk/lib/lsalib/lsa.c\0" as *const u8 as *const i8,
                290i32,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"sdk/lib/lsalib/lsa.c\0" as *const u8 as *const i8,
                    290i32,
                );
            }
            return Status;
        }
    }

    Status = ZwWaitForSingleObject(
        EventHandle,
        1u8,
        core::ptr::null_mut(),
    );
    ZwClose(EventHandle);

    if !NT_SUCCESS(Status) {
        if DbgPrint(
            b"(%s:%d) ZwWaitForSingleObject failed (Status 0x%08lx)\n\0" as *const u8 as *const i8,
            b"sdk/lib/lsalib/lsa.c\0" as *const u8 as *const i8,
            301i32,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                b"sdk/lib/lsalib/lsa.c\0" as *const u8 as *const i8,
                301i32,
            );
        }
        return Status;
    }

    RtlInitUnicodeString(
        &raw mut PortName,
        w!("\\LsaAuthenticationPort").as_ptr(),
    );

    SecurityQos.Length = core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as u32;
    SecurityQos.ImpersonationLevel = SecurityIdentification;
    SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING as u8;
    SecurityQos.EffectiveOnly = 1u8;

    strncpy(
        &raw mut ConnectInfo.LogonProcessNameBuffer as *mut i8,
        (*LogonProcessName).Buffer as *const i8,
        (*LogonProcessName).Length as ::core::ffi::c_ulong,
    );
    ConnectInfo.Length = (*LogonProcessName).Length as u32;
    ConnectInfo.LogonProcessNameBuffer[ConnectInfo.Length as usize] = ANSI_NULL;
    ConnectInfo.CreateContext = 1; // BOOL (c_int)
    ConnectInfo.TrustedCaller = CHECK;

    Status = ZwConnectPort(
        LsaHandle,
        &raw mut PortName,
        &raw mut SecurityQos,
        core::ptr::null_mut(), // ClientView: PPORT_VIEW
        core::ptr::null_mut(), // ServerView: PREMOTE_PORT_VIEW
        core::ptr::null_mut::<u32>(),
        &raw mut ConnectInfo as *mut core::ffi::c_void,
        &raw mut ConnectInfoLength,
    );

    if !NT_SUCCESS(Status) {
        if DbgPrint(
            b"(%s:%d) ZwConnectPort failed (Status 0x%08lx)\n\0" as *const u8 as *const i8,
            b"sdk/lib/lsalib/lsa.c\0" as *const u8 as *const i8,
            332i32,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                b"sdk/lib/lsalib/lsa.c\0" as *const u8 as *const i8,
                332i32,
            );
        }
        return Status;
    }

    *OperationalMode = ConnectInfo.OperationalMode as u32;

    if !NT_SUCCESS(ConnectInfo.Status) {
        if DbgPrint(
            b"(%s:%d) ConnectInfo.__bindgen_anon_1.Status: 0x%08lx\n\0" as *const u8 as *const i8,
            b"sdk/lib/lsalib/lsa.c\0" as *const u8 as *const i8,
            341i32,
            ConnectInfo.Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                b"sdk/lib/lsalib/lsa.c\0" as *const u8 as *const i8,
                341i32,
            );
        }
    }

    ConnectInfo.Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            lib/lsalib/lsa.c
//  * PURPOSE:         Client-side LSA functions
//  * UPDATE HISTORY:
//  *                  Created 05/08/00
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ndk/exfuncs.h>
// #include <ndk/lpctypes.h>
// #include <ndk/lpcfuncs.h>
// #include <ndk/mmfuncs.h>
// #include <ndk/rtlfuncs.h>
// #include <ndk/obfuncs.h>
// #include <lsass/lsass.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LsaCallAuthenticationPackage(IN HANDLE LsaHandle,
//                              IN ULONG AuthenticationPackage,
//                              IN PVOID ProtocolSubmitBuffer,
//                              IN ULONG SubmitBufferLength,
//                              OUT PVOID *ProtocolReturnBuffer,
//                              OUT PULONG ReturnBufferLength,
//                              OUT PNTSTATUS ProtocolStatus)
// {
//     LSA_API_MSG ApiMessage;
//     NTSTATUS Status;
//
//     DPRINT1("LsaCallAuthenticationPackage()\n");
//
//     ApiMessage.ApiNumber = LSASS_REQUEST_CALL_AUTHENTICATION_PACKAGE;
//     ApiMessage.h.u1.s1.DataLength = LSA_PORT_DATA_SIZE(ApiMessage.CallAuthenticationPackage);
//     ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE;
//     ApiMessage.h.u2.ZeroInit = 0;
//
//     ApiMessage.CallAuthenticationPackage.Request.AuthenticationPackage = AuthenticationPackage;
//     ApiMessage.CallAuthenticationPackage.Request.ProtocolSubmitBuffer = ProtocolSubmitBuffer;
//     ApiMessage.CallAuthenticationPackage.Request.SubmitBufferLength = SubmitBufferLength;
//
//     Status = ZwRequestWaitReplyPort(LsaHandle,
//                                     (PPORT_MESSAGE)&ApiMessage,
//                                     (PPORT_MESSAGE)&ApiMessage);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwRequestWaitReplyPort() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         DPRINT1("ZwRequestWaitReplyPort() failed (ApiMessage.Status 0x%08lx)\n", ApiMessage.Status);
//         return ApiMessage.Status;
//     }
//
//     *ProtocolReturnBuffer = ApiMessage.CallAuthenticationPackage.Reply.ProtocolReturnBuffer;
//     *ReturnBufferLength = ApiMessage.CallAuthenticationPackage.Reply.ReturnBufferLength;
//     *ProtocolStatus = ApiMessage.CallAuthenticationPackage.Reply.ProtocolStatus;
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
// LsaDeregisterLogonProcess(IN HANDLE LsaHandle)
// {
//     LSA_API_MSG ApiMessage;
//     NTSTATUS Status;
//
//     DPRINT("LsaDeregisterLogonProcess()\n");
//
//     ApiMessage.ApiNumber = LSASS_REQUEST_DEREGISTER_LOGON_PROCESS;
//     ApiMessage.h.u1.s1.DataLength = LSA_PORT_DATA_SIZE(ApiMessage.DeregisterLogonProcess);
//     ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE;
//     ApiMessage.h.u2.ZeroInit = 0;
//
//     Status = ZwRequestWaitReplyPort(LsaHandle,
//                                     (PPORT_MESSAGE)&ApiMessage,
//                                     (PPORT_MESSAGE)&ApiMessage);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwRequestWaitReplyPort() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         DPRINT1("ZwRequestWaitReplyPort() failed (ApiMessage.Status 0x%08lx)\n", ApiMessage.Status);
//         return ApiMessage.Status;
//     }
//
//     ZwClose(LsaHandle);
//
//     DPRINT("LsaDeregisterLogonProcess() done (Status 0x%08lx)\n", Status);
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
// LsaFreeReturnBuffer(IN PVOID Buffer)
// {
//     SIZE_T Size = 0;
//
//     return ZwFreeVirtualMemory(NtCurrentProcess(),
//                                &Buffer,
//                                &Size,
//                                MEM_RELEASE);
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LsaLookupAuthenticationPackage(IN HANDLE LsaHandle,
//                                IN PLSA_STRING PackageName,
//                                OUT PULONG AuthenticationPackage)
// {
//     LSA_API_MSG ApiMessage;
//     NTSTATUS Status;
//
//     /* Check the package name length */
//     if (PackageName->Length > LSASS_MAX_PACKAGE_NAME_LENGTH)
//     {
//         return STATUS_NAME_TOO_LONG;
//     }
//
//     ApiMessage.ApiNumber = LSASS_REQUEST_LOOKUP_AUTHENTICATION_PACKAGE;
//     ApiMessage.h.u1.s1.DataLength = LSA_PORT_DATA_SIZE(ApiMessage.LookupAuthenticationPackage);
//     ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE;
//     ApiMessage.h.u2.ZeroInit = 0;
//
//     ApiMessage.LookupAuthenticationPackage.Request.PackageNameLength = PackageName->Length;
//     strncpy(ApiMessage.LookupAuthenticationPackage.Request.PackageName,
//             PackageName->Buffer,
//             ApiMessage.LookupAuthenticationPackage.Request.PackageNameLength);
//     ApiMessage.LookupAuthenticationPackage.Request.PackageName[ApiMessage.LookupAuthenticationPackage.Request.PackageNameLength] = ANSI_NULL;
//
//     Status = ZwRequestWaitReplyPort(LsaHandle,
//                                     (PPORT_MESSAGE)&ApiMessage,
//                                     (PPORT_MESSAGE)&ApiMessage);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         return ApiMessage.Status;
//     }
//
//     *AuthenticationPackage = ApiMessage.LookupAuthenticationPackage.Reply.Package;
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
// LsaLogonUser(IN HANDLE LsaHandle,
//              IN PLSA_STRING OriginName,
//              IN SECURITY_LOGON_TYPE LogonType,
//              IN ULONG AuthenticationPackage,
//              IN PVOID AuthenticationInformation,
//              IN ULONG AuthenticationInformationLength,
//              IN PTOKEN_GROUPS LocalGroups OPTIONAL,
//              IN PTOKEN_SOURCE SourceContext,
//              OUT PVOID *ProfileBuffer,
//              OUT PULONG ProfileBufferLength,
//              OUT PLUID LogonId,
//              OUT PHANDLE Token,
//              OUT PQUOTA_LIMITS Quotas,
//              OUT PNTSTATUS SubStatus)
// {
//     LSA_API_MSG ApiMessage;
//     NTSTATUS Status;
//
//     ApiMessage.ApiNumber = LSASS_REQUEST_LOGON_USER;
//     ApiMessage.h.u1.s1.DataLength = LSA_PORT_DATA_SIZE(ApiMessage.LogonUser);
//     ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE;
//     ApiMessage.h.u2.ZeroInit = 0;
//
//     ApiMessage.LogonUser.Request.OriginName = *OriginName;
//     ApiMessage.LogonUser.Request.LogonType = LogonType;
//     ApiMessage.LogonUser.Request.AuthenticationPackage = AuthenticationPackage;
//     ApiMessage.LogonUser.Request.AuthenticationInformation = AuthenticationInformation;
//     ApiMessage.LogonUser.Request.AuthenticationInformationLength = AuthenticationInformationLength;
//     ApiMessage.LogonUser.Request.LocalGroups = LocalGroups;
//     if (LocalGroups != NULL)
//         ApiMessage.LogonUser.Request.LocalGroupsCount = LocalGroups->GroupCount;
//     else
//         ApiMessage.LogonUser.Request.LocalGroupsCount = 0;
//     ApiMessage.LogonUser.Request.SourceContext = *SourceContext;
//
//     Status = ZwRequestWaitReplyPort(LsaHandle,
//                                     (PPORT_MESSAGE)&ApiMessage,
//                                     (PPORT_MESSAGE)&ApiMessage);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     *SubStatus = ApiMessage.LogonUser.Reply.SubStatus;
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         return ApiMessage.Status;
//     }
//
//     *ProfileBuffer = ApiMessage.LogonUser.Reply.ProfileBuffer;
//     *ProfileBufferLength = ApiMessage.LogonUser.Reply.ProfileBufferLength;
//     *LogonId = ApiMessage.LogonUser.Reply.LogonId;
//     *Token = ApiMessage.LogonUser.Reply.Token;
//     *Quotas = ApiMessage.LogonUser.Reply.Quotas;
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
// LsaRegisterLogonProcess(IN PLSA_STRING LogonProcessName,
//                         OUT PHANDLE LsaHandle,
//                         OUT PLSA_OPERATIONAL_MODE OperationalMode)
// {
//     SECURITY_QUALITY_OF_SERVICE SecurityQos;
//     LSA_CONNECTION_INFO ConnectInfo;
//     ULONG ConnectInfoLength = sizeof(ConnectInfo);
//     UNICODE_STRING PortName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING EventName;
//     HANDLE EventHandle;
//     NTSTATUS Status;
//
//     DPRINT("LsaRegisterLogonProcess()\n");
//
//     /* Check the logon process name length */
//     if (LogonProcessName->Length > LSASS_MAX_LOGON_PROCESS_NAME_LENGTH)
//         return STATUS_NAME_TOO_LONG;
//
//     /* Wait for the LSA authentication thread */
//     RtlInitUnicodeString(&EventName,
//                          L"\\SECURITY\\LSA_AUTHENTICATION_INITIALIZED");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &EventName,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                NULL,
//                                NULL);
//     Status = ZwOpenEvent(&EventHandle,
//                          SYNCHRONIZE,
//                          &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ZwOpenEvent failed (Status 0x%08lx)\n", Status);
//
//         Status = ZwCreateEvent(&EventHandle,
//                                SYNCHRONIZE,
//                                &ObjectAttributes,
//                                NotificationEvent,
//                                FALSE);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("ZwCreateEvent failed (Status 0x%08lx)\n", Status);
//             return Status;
//         }
//     }
//
//     Status = ZwWaitForSingleObject(EventHandle,
//                                    TRUE,
//                                    NULL);
//     ZwClose(EventHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwWaitForSingleObject failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Establish the connection */
//     RtlInitUnicodeString(&PortName,
//                          L"\\LsaAuthenticationPort");
//
//     SecurityQos.Length              = sizeof(SecurityQos);
//     SecurityQos.ImpersonationLevel  = SecurityIdentification;
//     SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING;
//     SecurityQos.EffectiveOnly       = TRUE;
//
//     strncpy(ConnectInfo.LogonProcessNameBuffer,
//             LogonProcessName->Buffer,
//             LogonProcessName->Length);
//     ConnectInfo.Length = LogonProcessName->Length;
//     ConnectInfo.LogonProcessNameBuffer[ConnectInfo.Length] = ANSI_NULL;
//     ConnectInfo.CreateContext = TRUE;
//     ConnectInfo.TrustedCaller = CHECK;
//
//     Status = ZwConnectPort(LsaHandle,
//                            &PortName,
//                            &SecurityQos,
//                            NULL,
//                            NULL,
//                            NULL,
//                            &ConnectInfo,
//                            &ConnectInfoLength);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwConnectPort failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     DPRINT("ConnectInfo.OperationalMode: 0x%08lx\n", ConnectInfo.OperationalMode);
//     *OperationalMode = ConnectInfo.OperationalMode;
//
//     if (!NT_SUCCESS(ConnectInfo.Status))
//     {
//         DPRINT1("ConnectInfo.Status: 0x%08lx\n", ConnectInfo.Status);
//     }
//
//     return ConnectInfo.Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
