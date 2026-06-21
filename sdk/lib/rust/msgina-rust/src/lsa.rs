use crate::prelude::*;
use crate::msgina_externs::*;
use crate::msgina_types::*;
use crate::resources::*;
use crate::msgina_consts::*;
use crate::strsafe::*;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{copy_nonoverlapping, null_mut};

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const MSV1_0_PACKAGE_NAME: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"MICROSOFT_AUTHENTICATION_PACKAGE_V1_0\0",
    )
};

static mut __wine_dbch_msgina: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"msgina\0\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_msgina as *mut __wine_debug_channel };

static mut User32TokenSourceName: [CHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [CHAR; 9]>(*b"User32  \0") };
#[no_mangle]
pub unsafe extern "C" fn ConnectToLsa(pgContext: *mut GINA_CONTEXT) -> NTSTATUS {
    let mut LogonProcessName: LSA_STRING = LSA_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut PackageName: LSA_STRING = LSA_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut SecurityMode: LSA_OPERATIONAL_MODE = 0;
    let mut Status: NTSTATUS = 0;

    if !(*pgContext).LsaHandle.is_null() {
        return STATUS_SUCCESS;
    }

    RtlInitAnsiString(
        core::ptr::addr_of_mut!(LogonProcessName) as PANSI_STRING,
        b"MSGINA\0".as_ptr() as *const i8,
    );

    Status = LsaRegisterLogonProcess(
        core::ptr::addr_of_mut!(LogonProcessName) as PANSI_STRING,
        core::ptr::addr_of_mut!((*pgContext).LsaHandle),
        core::ptr::addr_of_mut!(SecurityMode),
    );

    if !(Status >= 0) {
        if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32)) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/msgina/lsa.c\0".as_ptr() as *const i8,
                b"ConnectToLsa\0".as_ptr() as *const i8,
                32,
                b"LsaRegisterLogonProcess failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
        return Status;
    }

    RtlInitAnsiString(
        core::ptr::addr_of_mut!(PackageName) as PANSI_STRING,
        MSV1_0_PACKAGE_NAME.as_ptr() as *const i8,
    );

    Status = LsaLookupAuthenticationPackage(
        (*pgContext).LsaHandle,
        core::ptr::addr_of_mut!(PackageName) as PANSI_STRING,
        core::ptr::addr_of_mut!((*pgContext).AuthenticationPackage),
    );

    if !(Status >= 0) {
        if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32)) != 0 {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/msgina/lsa.c\0".as_ptr() as *const i8,
                b"ConnectToLsa\0".as_ptr() as *const i8,
                45,
                b"LsaLookupAuthenticationPackage failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn MyLogonUser(
    LsaHandle: HANDLE,
    AuthenticationPackage: ULONG,
    lpszUsername: LPWSTR,
    lpszDomain: LPWSTR,
    lpszPassword: LPWSTR,
    phToken: PHANDLE,
    SubStatus: PNTSTATUS,
) -> NTSTATUS {

    const SECURITY_LOCAL_SID_AUTHORITY: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 2],
    };
    const SECURITY_NT_AUTHORITY: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    const MSV1_0_INTERACTIVE_LOGON_SIZE: usize = size_of::<MSV1_0_INTERACTIVE_LOGON>();
    const ANYSIZE_ARRAY: isize = 1;

    let mut LogonSid: PSID = null_mut();
    let mut LocalSid: PSID = null_mut();
    let mut OriginName: LSA_STRING = LSA_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: null_mut(),
    };
    let mut DomainName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: null_mut(),
    };
    let mut UserName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: null_mut(),
    };
    let mut Password: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: null_mut(),
    };
    let mut AuthInfo: PMSV1_0_INTERACTIVE_LOGON = null_mut();
    let mut AuthInfoLength: ULONG;
    let mut Ptr: ULONG_PTR;
    let mut TokenSourceVar: TOKEN_SOURCE = TOKEN_SOURCE {
        SourceName: [0; 8],
        SourceIdentifier: LUID { LowPart: 0, HighPart: 0 },
    };
    let mut TokenGroupsVar: PTOKEN_GROUPS = null_mut();
    let mut ProfileBuffer: PMSV1_0_INTERACTIVE_PROFILE = null_mut();
    let mut ProfileBufferLength: ULONG = 0;
    let mut Luid: LUID = LUID { LowPart: 0, HighPart: 0 };
    let mut LogonId: LUID = LUID { LowPart: 0, HighPart: 0 };
    let mut TokenHandle: HANDLE = null_mut();
    // QUOTA_LIMITS contains a LARGE_INTEGER union (no named-field literal possible);
    // C zero-initializes the whole struct, so do the same.
    let mut QuotaLimits: QUOTA_LIMITS = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    *phToken = null_mut();

    // C goto-cleanup → run-once loop: `goto cleanup` became `break 'cleanup`, the
    // post-label cleanup code follows the loop. (faithful control-flow lowering)
    'cleanup: loop {
    RtlInitAnsiString(
        &mut OriginName as PANSI_STRING,
        b"MSGINA Logon\0".as_ptr() as PCSZ,
    );
    RtlInitUnicodeString(&mut DomainName, lpszDomain as PCWSTR);
    RtlInitUnicodeString(&mut UserName, lpszUsername as PCWSTR);
    RtlInitUnicodeString(&mut Password, lpszPassword as PCWSTR);

    AuthInfoLength = (MSV1_0_INTERACTIVE_LOGON_SIZE
        + DomainName.MaximumLength as usize
        + UserName.MaximumLength as usize
        + Password.MaximumLength as usize) as ULONG;

    AuthInfo = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (AuthInfoLength as usize).try_into().unwrap(),
    ) as PMSV1_0_INTERACTIVE_LOGON;
    if AuthInfo.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        break 'cleanup;
    }

    (*AuthInfo).MessageType = MsV1_0InteractiveLogon;
    Ptr = (AuthInfo as usize).wrapping_add(MSV1_0_INTERACTIVE_LOGON_SIZE) as ULONG_PTR;

    (*AuthInfo).LogonDomainName.Length = DomainName.Length;
    (*AuthInfo).LogonDomainName.MaximumLength = DomainName.MaximumLength;
    (*AuthInfo).LogonDomainName.Buffer = if DomainName.Buffer.is_null() {
        null_mut()
    } else {
        Ptr as PWCHAR
    };
    if DomainName.MaximumLength > 0 {
        copy_nonoverlapping::<u8>(
            DomainName.Buffer as *const u8,
            (*AuthInfo).LogonDomainName.Buffer as *mut u8,
            DomainName.MaximumLength as usize,
        );
        Ptr = (Ptr as usize).wrapping_add(DomainName.MaximumLength as usize) as ULONG_PTR;
    }

    (*AuthInfo).UserName.Length = UserName.Length;
    (*AuthInfo).UserName.MaximumLength = UserName.MaximumLength;
    (*AuthInfo).UserName.Buffer = Ptr as PWCHAR;
    if UserName.MaximumLength > 0 {
        copy_nonoverlapping::<u8>(
            UserName.Buffer as *const u8,
            (*AuthInfo).UserName.Buffer as *mut u8,
            UserName.MaximumLength as usize,
        );
    }
    Ptr = (Ptr as usize).wrapping_add(UserName.MaximumLength as usize) as ULONG_PTR;

    (*AuthInfo).Password.Length = Password.Length;
    (*AuthInfo).Password.MaximumLength = Password.MaximumLength;
    (*AuthInfo).Password.Buffer = Ptr as PWCHAR;
    if Password.MaximumLength > 0 {
        copy_nonoverlapping::<u8>(
            Password.Buffer as *const u8,
            (*AuthInfo).Password.Buffer as *mut u8,
            Password.MaximumLength as usize,
        );
    }

    AllocateLocallyUniqueId(&mut LogonId);
    Status = RtlAllocateAndInitializeSid(
        &SECURITY_NT_AUTHORITY as *const _ as PSID_IDENTIFIER_AUTHORITY,
        SECURITY_LOGON_IDS_RID_COUNT.try_into().unwrap(),
        SECURITY_LOGON_IDS_RID,
        LogonId.HighPart as ULONG,
        LogonId.LowPart as ULONG,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        &mut LogonSid,
    );
    if !NT_SUCCESS(Status) {
        break 'cleanup;
    }

    Status = RtlAllocateAndInitializeSid(
        &SECURITY_LOCAL_SID_AUTHORITY as *const _ as PSID_IDENTIFIER_AUTHORITY,
        1,
        SECURITY_LOCAL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        &mut LocalSid,
    );
    if !NT_SUCCESS(Status) {
        break 'cleanup;
    }

    TokenGroupsVar = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (size_of::<TOKEN_GROUPS>()
            + ((2 - ANYSIZE_ARRAY) as usize) * size_of::<SID_AND_ATTRIBUTES>()).try_into().unwrap(),
    ) as PTOKEN_GROUPS;
    if TokenGroupsVar.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        break 'cleanup;
    }

    (*TokenGroupsVar).GroupCount = 2;
    (*TokenGroupsVar).Groups[0].Sid = LogonSid;
    (*TokenGroupsVar).Groups[0].Attributes =
        SE_GROUP_MANDATORY | SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_LOGON_ID;
    (*TokenGroupsVar).Groups[1].Sid = LocalSid;
    (*TokenGroupsVar).Groups[1].Attributes =
        SE_GROUP_MANDATORY | SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT;

    RtlCopyMemory!(
        TokenSourceVar.SourceName.as_mut_ptr() as *mut u8,
        User32TokenSourceName.as_ptr() as *const i8,
        size_of::<[i8; 8]>(),
    );
    AllocateLocallyUniqueId(&mut TokenSourceVar.SourceIdentifier);

    Status = LsaLogonUser(
        LsaHandle,
        &mut OriginName,
        Interactive,
        AuthenticationPackage,
        AuthInfo as PVOID,
        AuthInfoLength,
        TokenGroupsVar,
        &mut TokenSourceVar,
        &mut ProfileBuffer as *mut _ as *mut PVOID,
        &mut ProfileBufferLength,
        &mut Luid,
        &mut TokenHandle,
        &mut QuotaLimits,
        SubStatus,
    );
    if !NT_SUCCESS(Status) {
        DbgPrint(
            b"LsaLogonUser failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        break 'cleanup;
    }

    if !ProfileBuffer.is_null() {
        DbgPrint(b"ProfileBuffer: %p\n\0".as_ptr() as *const i8, ProfileBuffer);
        DbgPrint(
            b"MessageType: %u\n\0".as_ptr() as *const i8,
            (*ProfileBuffer).MessageType,
        );
        DbgPrint(
            b"FullName: %p\n\0".as_ptr() as *const i8,
            (*ProfileBuffer).FullName.Buffer,
        );
        DbgPrint(
            b"FullName: %S\n\0".as_ptr() as *const i8,
            (*ProfileBuffer).FullName.Buffer,
        );
        DbgPrint(
            b"LogonServer: %p\n\0".as_ptr() as *const i8,
            (*ProfileBuffer).LogonServer.Buffer,
        );
        DbgPrint(
            b"LogonServer: %S\n\0".as_ptr() as *const i8,
            (*ProfileBuffer).LogonServer.Buffer,
        );
    }
    DbgPrint(
        b"Luid: 0x%08lx%08lx\n\0".as_ptr() as *const i8,
        Luid.HighPart,
        Luid.LowPart,
    );
    if !TokenHandle.is_null() {
        DbgPrint(b"TokenHandle: %p\n\0".as_ptr() as *const i8, TokenHandle);
    }

    *phToken = TokenHandle;

    break 'cleanup;
    } // 'cleanup loop (C cleanup: label)

    SecureZeroMemory(&mut Password as *mut _ as *mut c_void, size_of::<_UNICODE_STRING>() as usize);

    if !ProfileBuffer.is_null() {
        LsaFreeReturnBuffer(ProfileBuffer as PVOID);
    }
    if !NT_SUCCESS(Status) && !TokenHandle.is_null() {
        CloseHandle(TokenHandle);
    }
    if !TokenGroupsVar.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, TokenGroupsVar as PVOID);
    }
    if !LocalSid.is_null() {
        RtlFreeSid(LocalSid);
    }
    if !LogonSid.is_null() {
        RtlFreeSid(LogonSid);
    }
    if !AuthInfo.is_null() {
        SecureZeroMemory(
            (*AuthInfo).Password.Buffer as *mut c_void,
            (*AuthInfo).Password.MaximumLength as usize,
        );
        SecureZeroMemory(
            &mut (*AuthInfo).Password as *mut _ as *mut c_void,
            size_of::<_UNICODE_STRING>() as usize,
        );
        RtlFreeHeap(RtlGetProcessHeap(), 0, AuthInfo as PVOID);
    }

    Status
}
