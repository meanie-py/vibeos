use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NOT_IMPLEMENTED, NT_SUCCESS, TRUE, FALSE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::{PagedPool, NonPagedPool};
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

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


#[cfg(target_arch = "aarch64")]
extern "C" {
    static mut Arm64UartKernelBase: PVOID;
    fn Arm64UartPutChar(c: u8);
}

#[cfg(target_arch = "aarch64")]
unsafe fn _TOKI_TRACE(msg: &[u8]) {
    let u = Arm64UartKernelBase as *mut ULONG;
    if !u.is_null() {
        for &byte in msg {
            Arm64UartPutChar(byte as u8);
        }
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn _TOKI_HEX(prefix: &[u8], val: ULONG_PTR) {
    let u = Arm64UartKernelBase as *mut ULONG;
    if !u.is_null() {
        for &byte in prefix {
            Arm64UartPutChar(byte as u8);
        }
        let hex = b"0123456789ABCDEF";
        Arm64UartPutChar(b'0' as u8);
        Arm64UartPutChar(b'x' as u8);
        for i in (0..16).rev() {
            let shift = i * 4;
            let nibble = ((val >> shift) & 0xF) as usize;
            Arm64UartPutChar(hex[nibble] as u8);
        }
        Arm64UartPutChar(b'\n' as u8);
    }
}

#[cfg(not(target_arch = "aarch64"))]
unsafe fn _TOKI_TRACE(_msg: &[u8]) {}

#[cfg(not(target_arch = "aarch64"))]
unsafe fn _TOKI_HEX(_prefix: &[u8], _val: ULONG_PTR) {}

extern "system" {
    fn SepRegQueryHelper(
        path: *mut u16,
        name: *mut u16,
        reg_type: ULONG,
        size: ULONG,
        data: *mut ULONG,
    ) -> NTSTATUS;
    fn SepSidInToken(token: PTOKEN, sid: PSID) -> BOOLEAN;
    fn SepSidInTokenEx(
        token: PTOKEN,
        a: PVOID,
        sid: PSID,
        b: BOOLEAN,
        c: BOOLEAN,
    ) -> BOOLEAN;
    fn SepCreateImpersonationTokenDacl(
        thread_token: PTOKEN,
        primary_token: PTOKEN,
        dacl: *mut PACL,
    ) -> NTSTATUS;
    fn SepDuplicateToken(
        token: PTOKEN,
        oa: *mut _OBJECT_ATTRIBUTES,
        effective_only: BOOLEAN,
        token_type: TOKEN_TYPE,
        level: SECURITY_IMPERSONATION_LEVEL,
        mode: KPROCESSOR_MODE,
        new_token: *mut PTOKEN,
    ) -> NTSTATUS;
    fn SepCreateToken(
        handle: *mut HANDLE,
        mode: KPROCESSOR_MODE,
        flags: ULONG,
        oa: *mut _OBJECT_ATTRIBUTES,
        token_type: TOKEN_TYPE,
        level: SECURITY_IMPERSONATION_LEVEL,
        auth_id: *const LUID,
        expiration: *const LARGE_INTEGER,
        user_sid: *const _SID_AND_ATTRIBUTES,
        group_count: ULONG,
        groups: *const _SID_AND_ATTRIBUTES,
        groups_length: ULONG,
        priv_count: ULONG,
        privs: *const _LUID_AND_ATTRIBUTES,
        owner: PSID,
        primary_group: PSID,
        dacl: PACL,
        source: *const _TOKEN_SOURCE,
        system_token: BOOLEAN,
    ) -> NTSTATUS;
    fn SepRmRemoveLogonSessionFromToken(token: PTOKEN) -> NTSTATUS;
    fn PsDisableImpersonation(
        thread: PETHREAD,
        state: *mut SE_IMPERSONATION_STATE,
    ) -> BOOLEAN;
    fn PsRestoreImpersonation(thread: PETHREAD, state: *mut SE_IMPERSONATION_STATE);
    fn PsReferenceImpersonationToken(
        thread: PETHREAD,
        copy_on_open: *mut BOOLEAN,
        effective_only: *mut BOOLEAN,
        level: *mut SECURITY_IMPERSONATION_LEVEL,
    ) -> PTOKEN;
    fn PsDereferenceImpersonationToken(token: PTOKEN);
    fn PsRevertToSelf();
    static mut SeAnonymousLogonToken: PTOKEN;
    static mut SeAnonymousLogonTokenNoEveryone: PTOKEN;
    static mut SeAnonymousLogonSid: PSID;
    static mut SeWorldSid: PSID;
    static mut SeAuthenticatedUsersSid: PSID;
    static mut SeLocalSystemSid: PSID;
    static mut SeAliasAdminsSid: PSID;
    static mut SeLocalSid: PSID;
    static mut SeSystemDefaultDacl: PACL;
    static mut SeSystemAnonymousLogonDacl: PACL;
    static mut PsThreadType: POBJECT_TYPE;
    static SeTcbPrivilege: LUID;
    static SeCreateTokenPrivilege: LUID;
    static SeTakeOwnershipPrivilege: LUID;
    static SeCreatePagefilePrivilege: LUID;
    static SeLockMemoryPrivilege: LUID;
    static SeAssignPrimaryTokenPrivilege: LUID;
    static SeIncreaseQuotaPrivilege: LUID;
    static SeIncreaseBasePriorityPrivilege: LUID;
    static SeCreatePermanentPrivilege: LUID;
    static SeDebugPrivilege: LUID;
    static SeAuditPrivilege: LUID;
    static SeSecurityPrivilege: LUID;
    static SeSystemEnvironmentPrivilege: LUID;
    static SeChangeNotifyPrivilege: LUID;
    static SeBackupPrivilege: LUID;
    static SeRestorePrivilege: LUID;
    static SeShutdownPrivilege: LUID;
    static SeLoadDriverPrivilege: LUID;
    static SeProfileSingleProcessPrivilege: LUID;
    static SeSystemtimePrivilege: LUID;
    static SeUndockPrivilege: LUID;
    static SeManageVolumePrivilege: LUID;
    static SeImpersonatePrivilege: LUID;
    static SeCreateGlobalPrivilege: LUID;
    fn RtlEqualLuid(a: *const LUID, b: *const LUID) -> BOOLEAN;
}
extern "C" {
    fn SepAcquireTokenLockExclusive(token: PTOKEN);
    fn SepAcquireTokenLockShared(token: PTOKEN);
    fn SepReleaseTokenLock(token: PTOKEN);
}
extern "C" {
    fn SepGetOwnerFromDescriptor(sd: PSECURITY_DESCRIPTOR) -> PSID;
    fn SepRmDereferenceLogonSession(auth_id: *const LUID) -> u32;
    fn SeLengthSid(sid: PSID) -> ULONG;
    fn ObpValidateAttributes(attrs: ULONG, mode: KPROCESSOR_MODE) -> ULONG;
}

const SYSTEM_LUID: LUID = _LUID {
    LowPart: 0x3e7,
    HighPart: 0,
};
const ANONYMOUS_LOGON_LUID: LUID = _LUID {
    LowPart: 0x3e6,
    HighPart: 0,
};

const TAG_SE_TOKEN_LOCK: ULONG = 0x6c546553;
const TAG_TOKEN_DYNAMIC: ULONG = 0x64546553;
const TAG_ACL: ULONG = 0x63416553;

const REG_DWORD: ULONG = 4;

const TOKEN_READ: ACCESS_MASK = 0x20008;
const TOKEN_WRITE: ACCESS_MASK = 0x200e0;
const TOKEN_EXECUTE: ACCESS_MASK = 0x20000;
const TOKEN_ALL_ACCESS: ACCESS_MASK = 0xf00ff;
const TOKEN_QUERY: ACCESS_MASK = 0x8;
const TOKEN_IMPERSONATE: ACCESS_MASK = 0x4;
const THREAD_QUERY_INFORMATION: ACCESS_MASK = 0x40;
const THREAD_IMPERSONATE: ACCESS_MASK = 0x100;
// 0x1F03FF = STANDARD_RIGHTS_REQUIRED|SYNCHRONIZE|0x3FF. xdk/pstypes.h picks this
// under `#if (NTDDI_VERSION >= NTDDI_VISTA) ... #else`, and this tree builds
// -D_WIN32_WINNT=0x502 (Server 2003), so the PRE-Vista branch is the live one. The
// Vista value 0x1FFFFF asks for 0xFC00 of thread rights the object's GENERIC_MAPPING
// does not grant, which an open would refuse rather than silently narrow.
const THREAD_ALL_ACCESS: ACCESS_MASK = 0x1F03FF;
const SECURITY_DESCRIPTOR_REVISION: ULONG = 1;

const TOKEN_HAS_TRAVERSE_PRIVILEGE: ULONG = 0x1;
const TOKEN_HAS_BACKUP_PRIVILEGE: ULONG = 0x2;
const TOKEN_HAS_RESTORE_PRIVILEGE: ULONG = 0x4;
const TOKEN_HAS_IMPERSONATE_PRIVILEGE: ULONG = 0x80;
const TOKEN_IS_RESTRICTED: ULONG = 0x10;
const TOKEN_SESSION_NOT_REFERENCED: ULONG = 0x20;
const TOKEN_SANDBOX_INERT: ULONG = 0x40;
const TOKEN_HAS_ADMIN_GROUP: ULONG = 0x80;
const SE_BACKUP_PRIVILEGES_CHECKED: ULONG = 0x100;

const SE_GROUP_ENABLED: ULONG = 0x4;
const SE_GROUP_MANDATORY: ULONG = 0x1;
const SE_GROUP_ENABLED_BY_DEFAULT: ULONG = 0x2;
const SE_GROUP_OWNER: ULONG = 0x8;

const SE_PRIVILEGE_ENABLED_BY_DEFAULT: ULONG = 0x1;
const SE_PRIVILEGE_ENABLED: ULONG = 0x2;

const SE_CHANGE_NOTIFY_PRIVILEGE: ULONG = 23;
const SE_BACKUP_PRIVILEGE: ULONG = 17;
const SE_RESTORE_PRIVILEGE: ULONG = 18;
const SE_IMPERSONATE_PRIVILEGE: ULONG = 29;

const TokenPrimary: TOKEN_TYPE = 1;
const TokenImpersonation: TOKEN_TYPE = 2;

const SecurityAnonymous: SECURITY_IMPERSONATION_LEVEL = 0;
const SecurityIdentification: SECURITY_IMPERSONATION_LEVEL = 1;
const SecurityImpersonation: SECURITY_IMPERSONATION_LEVEL = 2;

#[no_mangle]
pub static mut SeTokenObjectType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub static mut SeSystemTokenSource: _TOKEN_SOURCE = _TOKEN_SOURCE {
    SourceName: [0x2a, 0x53, 0x59, 0x53, 0x54, 0x45, 0x4d, 0x2a],
    SourceIdentifier: _LUID {
        LowPart: 0,
        HighPart: 0,
    },
};

#[no_mangle]
pub static mut SeSystemAuthenticationId: LUID = SYSTEM_LUID;

#[no_mangle]
pub static mut SeAnonymousAuthenticationId: LUID = ANONYMOUS_LOGON_LUID;

static mut SepTokenMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: TOKEN_READ,
    GenericWrite: TOKEN_WRITE,
    GenericExecute: TOKEN_EXECUTE,
    GenericAll: TOKEN_ALL_ACCESS,
};

#[no_mangle]
pub unsafe extern "C" fn SepCreateTokenLock(token: PTOKEN) -> NTSTATUS {
    let mut token = opaque_ptr(token);
    (*token).TokenLock = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<ERESOURCE>() as SIZE_T,
        TAG_SE_TOKEN_LOCK,
    ) as *mut ERESOURCE;
    if (*token).TokenLock.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    ExInitializeResourceLite((*token).TokenLock);
    STATUS_SUCCESS
}

unsafe fn SepDeleteTokenLock(token: PTOKEN) {
    ExDeleteResourceLite((*token).TokenLock);
    ExFreePoolWithTag((*token).TokenLock as PVOID, TAG_SE_TOKEN_LOCK);
}

unsafe fn SepCompareSidAndAttributesFromTokens(
    sid_array_token1: PSID_AND_ATTRIBUTES,
    count_sid_array1: ULONG,
    sid_array_token2: PSID_AND_ATTRIBUTES,
    count_sid_array2: ULONG,
) -> BOOLEAN {
    if count_sid_array1 != count_sid_array2 {
        return 0u8;
    }

    for first_count in 0..count_sid_array1 {
        let mut found = 0u8;
        for second_count in 0..count_sid_array2 {
            let first_sid_array =
                &*sid_array_token1.offset(first_count as isize);
            let second_sid_array =
                &*sid_array_token2.offset(second_count as isize);

            if RtlEqualSid(first_sid_array.Sid, second_sid_array.Sid) != 0
                && first_sid_array.Attributes == second_sid_array.Attributes
            {
                found = 1u8;
                break;
            }
        }

        if found == 0 {
            return 0u8;
        }
    }

    1u8
}

unsafe fn SepComparePrivilegeAndAttributesFromTokens(
    priv_array_token1: PLUID_AND_ATTRIBUTES,
    count_priv_array1: ULONG,
    priv_array_token2: PLUID_AND_ATTRIBUTES,
    count_priv_array2: ULONG,
) -> BOOLEAN {
    if count_priv_array1 != count_priv_array2 {
        return 0u8;
    }

    for first_count in 0..count_priv_array1 {
        let mut found = 0u8;
        for second_count in 0..count_priv_array2 {
            let first_priv_array =
                &*priv_array_token1.offset(first_count as isize);
            let second_priv_array =
                &*priv_array_token2.offset(second_count as isize);

            if RtlEqualLuid(
                &first_priv_array.Luid as *const LUID,
                &second_priv_array.Luid as *const LUID,
            ) != 0
                && first_priv_array.Attributes == second_priv_array.Attributes
            {
                found = 1u8;
                break;
            }
        }

        if found == 0 {
            return 0u8;
        }
    }

    1u8
}

unsafe fn SepCompareTokens(
    first_token: PTOKEN,
    second_token: PTOKEN,
    equal: *mut BOOLEAN,
) -> NTSTATUS {
    let mut is_equal = 0u8;

    SepAcquireTokenLockShared(first_token);
    SepAcquireTokenLockShared(second_token);

    if SepCompareSidAndAttributesFromTokens(
        (*first_token).UserAndGroups,
        (*first_token).UserAndGroupCount,
        (*second_token).UserAndGroups,
        (*second_token).UserAndGroupCount,
    ) == 0
    {
        SepReleaseTokenLock(second_token);
        SepReleaseTokenLock(first_token);
        *equal = is_equal;
        return STATUS_SUCCESS;
    }

    let restricted = SeTokenIsRestricted(first_token as PVOID);
    if restricted != SeTokenIsRestricted(second_token as PVOID) {
        SepReleaseTokenLock(second_token);
        SepReleaseTokenLock(first_token);
        *equal = is_equal;
        return STATUS_SUCCESS;
    }

    if restricted != 0 {
        if SepCompareSidAndAttributesFromTokens(
            (*first_token).RestrictedSids,
            (*first_token).RestrictedSidCount,
            (*second_token).RestrictedSids,
            (*second_token).RestrictedSidCount,
        ) == 0
        {
            SepReleaseTokenLock(second_token);
            SepReleaseTokenLock(first_token);
            *equal = is_equal;
            return STATUS_SUCCESS;
        }
    }

    if SepComparePrivilegeAndAttributesFromTokens(
        (*first_token).Privileges,
        (*first_token).PrivilegeCount,
        (*second_token).Privileges,
        (*second_token).PrivilegeCount,
    ) == 0
    {
        SepReleaseTokenLock(second_token);
        SepReleaseTokenLock(first_token);
        *equal = is_equal;
        return STATUS_SUCCESS;
    }

    is_equal = 1u8;

    SepReleaseTokenLock(second_token);
    SepReleaseTokenLock(first_token);

    *equal = is_equal;
    STATUS_SUCCESS
}

unsafe fn SepImpersonateAnonymousToken(
    thread: PETHREAD,
    previous_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let mut include_everyone_value_data: ULONG = 0;

    let status = SepRegQueryHelper(
        b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0L\0s\0a\0\0\0"
            .as_ptr() as *mut u16,
        w!("EveryoneIncludesAnonymous").as_ptr() as *mut u16,
        REG_DWORD,
        core::mem::size_of::<ULONG>() as ULONG,
        &mut include_everyone_value_data,
    );
    if status < 0 {
        return status;
    }

    let token_to_impersonate = if include_everyone_value_data == 0 {
        SeAnonymousLogonTokenNoEveryone
    } else {
        SeAnonymousLogonToken
    };

    let status = ObReferenceObjectByPointer(
        token_to_impersonate as PVOID,
        TOKEN_IMPERSONATE,
        SeTokenObjectType as *mut _OBJECT_TYPE,
        previous_mode,
    );
    if status < 0 {
        return status;
    }

    let process_token =
        PsReferencePrimaryToken(PsGetCurrentProcess() as *mut _EPROCESS);
    if process_token.is_null() {
        ObDereferenceObject(token_to_impersonate as PVOID);
        return STATUS_UNSUCCESSFUL;
    }

    if SeTokenIsRestricted(process_token) != 0 {
        PsDereferencePrimaryToken(process_token);
        ObDereferenceObject(token_to_impersonate as PVOID);
        return STATUS_ACCESS_DENIED;
    }

    ObFastDereferenceObject(
        core::ptr::addr_of_mut!(
            (*(PsGetCurrentProcess() as *mut _EPROCESS)).Token
        ),
        process_token as PVOID,
    );
    let status = PsImpersonateClient(
        thread,
        token_to_impersonate as PVOID,
        1u8,
        0u8,
        SecurityImpersonation,
    );
    if status < 0 {
        ObDereferenceObject(token_to_impersonate as PVOID);
        return status;
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn SepUpdateSinglePrivilegeFlagToken(token: PTOKEN, index: ULONG) {
    let mut token = opaque_ptr(token);
    let token_flag: ULONG;

    if (*(*token).Privileges.offset(index as isize)).Luid.HighPart != 0 {
        return;
    }

    let low_part =
        (*(*token).Privileges.offset(index as isize)).Luid.LowPart;
    if low_part == SE_CHANGE_NOTIFY_PRIVILEGE {
        token_flag = TOKEN_HAS_TRAVERSE_PRIVILEGE;
    } else if low_part == SE_BACKUP_PRIVILEGE {
        token_flag = TOKEN_HAS_BACKUP_PRIVILEGE;
    } else if low_part == SE_RESTORE_PRIVILEGE {
        token_flag = TOKEN_HAS_RESTORE_PRIVILEGE;
    } else if low_part == SE_IMPERSONATE_PRIVILEGE {
        token_flag = TOKEN_HAS_IMPERSONATE_PRIVILEGE;
    } else {
        return;
    }

    if ((*(*token).Privileges.offset(index as isize)).Attributes
        & SE_PRIVILEGE_ENABLED)
        != 0
    {
        (*token).TokenFlags |= token_flag;
    } else {
        (*token).TokenFlags &= !token_flag;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SepTokenIsOwner(
    _token: PACCESS_TOKEN,
    security_descriptor: PSECURITY_DESCRIPTOR,
    token_locked: BOOLEAN,
) -> BOOLEAN {
    let security_descriptor = opaque_ptr(security_descriptor);
    let token = _token as PTOKEN;
    let sid = SepGetOwnerFromDescriptor(security_descriptor);

    // Guard: C code has ASSERT(Sid != NULL) which is no-op in release.
    // With DUMMY_PSEH, a bad SID pointer causes unhandled page fault.
    if sid.is_null() {
        return 0;
    }
    // Check if SID pointer is accessible — avoids page fault with DUMMY_PSEH
    if MmIsAddressValid(sid as PVOID) == 0 {
        return 0;
    }

    if token_locked == 0 {
        SepAcquireTokenLockShared(token);
    }

    let mut result = SepSidInToken(token, sid);
    if result != 0 && ((*token).TokenFlags & TOKEN_IS_RESTRICTED) != 0 {
        result = SepSidInTokenEx(
            token,
            core::ptr::null_mut(),
            sid,
            0u8,
            1u8,
        );
    }

    if token_locked == 0 {
        SepReleaseTokenLock(token);
    }

    result
}

#[no_mangle]
pub unsafe extern "C" fn SepUpdatePrivilegeFlagsToken(token: PTOKEN) {
    let mut token = opaque_ptr(token);
    for i in 0..(*token).PrivilegeCount {
        SepUpdateSinglePrivilegeFlagToken(token, i);
    }
}

#[no_mangle]
pub unsafe extern "C" fn SepRemovePrivilegeToken(token: PTOKEN, index: ULONG) {
    let mut token = opaque_ptr(token);
    let move_count = (*token).PrivilegeCount - index - 1;
    if move_count != 0 {
        core::ptr::copy(
            (*token).Privileges.offset((index + 1) as isize) as *const u8,
            (*token).Privileges.offset(index as isize) as *mut u8,
            (move_count as usize) * core::mem::size_of::<_LUID_AND_ATTRIBUTES>(),
        );
    }
    (*token).PrivilegeCount -= 1;
}

#[no_mangle]
pub unsafe extern "C" fn SepRemoveUserGroupToken(token: PTOKEN, index: ULONG) {
    let mut token = opaque_ptr(token);
    let move_count = (*token).UserAndGroupCount - index - 1;
    if move_count != 0 {
        core::ptr::copy(
            (*token).UserAndGroups.offset((index + 1) as isize) as *const u8,
            (*token).UserAndGroups.offset(index as isize) as *mut u8,
            (move_count as usize) * core::mem::size_of::<_SID_AND_ATTRIBUTES>(),
        );
    }
    (*token).UserAndGroupCount -= 1;
}

#[no_mangle]
pub unsafe extern "C" fn SepComputeAvailableDynamicSpace(
    dynamic_charged: ULONG,
    primary_group: PSID,
    default_dacl: PACL,
) -> ULONG {
    let primary_group = opaque_ptr(primary_group);
    let default_dacl = opaque_ptr(default_dacl);
    let mut dynamic_available = dynamic_charged - RtlLengthSid(primary_group);
    if !default_dacl.is_null() {
        dynamic_available -= (*default_dacl).AclSize as ULONG;
    }
    dynamic_available
}

#[no_mangle]
pub unsafe extern "C" fn SepRebuildDynamicPartOfToken(
    access_token: PTOKEN,
    new_dynamic_part_size: ULONG,
) -> NTSTATUS {
    let access_token = opaque_ptr(access_token);
    let mut current_dynamic_length =
        (*access_token).DynamicAvailable + RtlLengthSid((*access_token).PrimaryGroup);
    if !(*access_token).DefaultDacl.is_null() {
        current_dynamic_length += (*(*access_token).DefaultDacl).AclSize as ULONG;
    }

    if current_dynamic_length < new_dynamic_part_size {
        let new_dynamic_part = ExAllocatePoolWithTag(
            PagedPool,
            new_dynamic_part_size as SIZE_T,
            TAG_TOKEN_DYNAMIC,
        );
        if new_dynamic_part.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        let previous_dynamic_part = (*access_token).DynamicPart;
        core::ptr::copy_nonoverlapping(
            previous_dynamic_part as *const u8,
            new_dynamic_part as *mut u8,
            current_dynamic_length as usize,
        );

        (*access_token).DynamicAvailable +=
            new_dynamic_part_size - current_dynamic_length;
        (*access_token).DynamicPart = new_dynamic_part as *mut ULONG;

        (*access_token).PrimaryGroup = ((new_dynamic_part as ULONG_PTR)
            + (((*access_token).PrimaryGroup as ULONG_PTR)
                - (previous_dynamic_part as ULONG_PTR)))
            as PSID;
        if !(*access_token).DefaultDacl.is_null() {
            (*access_token).DefaultDacl = ((new_dynamic_part as ULONG_PTR)
                + (((*access_token).DefaultDacl as ULONG_PTR)
                    - (previous_dynamic_part as ULONG_PTR)))
                as PACL;
        }

        ExFreePoolWithTag(previous_dynamic_part as PVOID, TAG_TOKEN_DYNAMIC);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SepFreeProxyData(_proxy_data: PVOID) {}
#[no_mangle]
pub unsafe extern "system" fn SepCopyProxyData(_dest: *mut PVOID, _src: PVOID) -> NTSTATUS {
    let _dest = opaque_ptr(_dest);
    let _src = opaque_ptr(_src);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SeExchangePrimaryToken(
    process: PEPROCESS,
    new_access_token: PACCESS_TOKEN,
    old_access_token: *mut PACCESS_TOKEN,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    let old_access_token = opaque_ptr(old_access_token);
    let new_token = new_access_token as PTOKEN;

    if (*new_token).TokenType != TokenPrimary {
        return STATUS_BAD_TOKEN_TYPE;
    }

    if (*new_token).TokenInUse != 0 {
        let old_token = PsReferencePrimaryToken(process as *mut _EPROCESS);
        if old_token == new_token as PVOID {
            *old_access_token = old_token as PACCESS_TOKEN;
            return STATUS_SUCCESS;
        }

        let mut is_equal = 0u8;
        let status = SepCompareTokens(old_token as PTOKEN, new_token, &mut is_equal);
        if status < 0 {
            PsDereferencePrimaryToken(old_token);
            *old_access_token = core::ptr::null_mut();
            return status;
        }

        if is_equal == 0 {
            PsDereferencePrimaryToken(old_token);
            *old_access_token = core::ptr::null_mut();
            return STATUS_TOKEN_ALREADY_IN_USE;
        }

        *old_access_token = old_token as PACCESS_TOKEN;
        return STATUS_SUCCESS;
    }

    SepAcquireTokenLockExclusive(new_token);
    (*new_token).TokenInUse = 1u8;
    (*new_token).SessionId = MmGetSessionId(process as *mut _EPROCESS);
    SepReleaseTokenLock(new_token);

    ObReferenceObject(new_token as PVOID);

    let old_token = ObFastReplaceObject(
        core::ptr::addr_of_mut!((*process).Token),
        new_token as PVOID,
    ) as PTOKEN;

    SepAcquireTokenLockExclusive(old_token);
    (*old_token).TokenInUse = 0u8;
    SepReleaseTokenLock(old_token);

    *old_access_token = old_token as PACCESS_TOKEN;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeDeassignPrimaryToken(process: PEPROCESS) {
    let process = opaque_ptr(process);
    let old_token = ObFastReplaceObject(
        core::ptr::addr_of_mut!((*process).Token),
        core::ptr::null_mut(),
    ) as PTOKEN;

    (*old_token).TokenInUse = 0u8;
    ObDereferenceObject(old_token as PVOID);
}

#[no_mangle]
pub unsafe extern "C" fn RtlLengthSidAndAttributes(count: ULONG, src: PSID_AND_ATTRIBUTES) -> ULONG {
    let mut u_length = count * core::mem::size_of::<_SID_AND_ATTRIBUTES>() as ULONG;
    for i in 0..count {
        u_length += RtlLengthSid((*src.offset(i as isize)).Sid);
    }
    u_length
}

#[no_mangle]
pub unsafe extern "C" fn SepFindPrimaryGroupAndDefaultOwner(
    token: PTOKEN,
    primary_group: PSID,
    default_owner: PSID,
    primary_group_index: *mut ULONG,
    default_owner_index: *mut ULONG,
) -> NTSTATUS {
    let mut token = opaque_ptr(token);
    let primary_group = opaque_ptr(primary_group);
    let default_owner = opaque_ptr(default_owner);
    let primary_group_index = opaque_ptr(primary_group_index);
    let default_owner_index = opaque_ptr(default_owner_index);
    if primary_group_index.is_null() && default_owner_index.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let mut pgi_search = !primary_group_index.is_null();
    let mut doi_search = !default_owner_index.is_null();

    if pgi_search {
        *primary_group_index = (*token).UserAndGroupCount;
    }

    if doi_search {
        if !default_owner.is_null() {
            if RtlEqualSid((*(*token).UserAndGroups.offset(0)).Sid, default_owner) != 0 {
                *default_owner_index = 0;
                doi_search = false;
            } else {
                *default_owner_index = (*token).UserAndGroupCount;
            }
        } else {
            *default_owner_index = 0;
            doi_search = false;
        }
    }

    for i in 0..(*token).UserAndGroupCount {
        if !pgi_search && !doi_search {
            break;
        }

        if doi_search
            && !default_owner.is_null()
            && RtlEqualSid((*(*token).UserAndGroups.offset(i as isize)).Sid, default_owner) != 0
            && ((*(*token).UserAndGroups.offset(i as isize)).Attributes & SE_GROUP_OWNER) != 0
        {
            *default_owner_index = i;
            doi_search = false;
        }

        if pgi_search && RtlEqualSid((*(*token).UserAndGroups.offset(i as isize)).Sid, primary_group) != 0 {
            *primary_group_index = i;
            pgi_search = false;
        }
    }

    if !default_owner_index.is_null() && *default_owner_index == (*token).UserAndGroupCount {
        return STATUS_INVALID_OWNER;
    }

    if !primary_group_index.is_null() && *primary_group_index == (*token).UserAndGroupCount {
        return STATUS_INVALID_PRIMARY_GROUP;
    }

    STATUS_SUCCESS
}

unsafe fn SepOpenThreadToken(
    thread: PETHREAD,
    thread_handle: HANDLE,
    thread_token: PTOKEN,
    desired_access: ACCESS_MASK,
    handle_attributes: ULONG,
    effective_only: BOOLEAN,
    copy_on_open: BOOLEAN,
    impersonation_level: SECURITY_IMPERSONATION_LEVEL,
    previous_mode: KPROCESSOR_MODE,
    opened_token_handle: PHANDLE,
) -> NTSTATUS {
    *opened_token_handle = core::ptr::null_mut();

    if copy_on_open == 0 {
        let mut token_handle: HANDLE = core::ptr::null_mut();
        let status = ObOpenObjectByPointer(
            thread_token as PVOID,
            handle_attributes,
            core::ptr::null_mut(),
            desired_access,
            SeTokenObjectType as *mut _OBJECT_TYPE,
            previous_mode,
            &mut token_handle,
        );
        if status < 0 {
            return status;
        }
        *opened_token_handle = token_handle;
        return STATUS_SUCCESS;
    }

    let mut thread2: PETHREAD = core::ptr::null_mut();
    let status = ObReferenceObjectByHandle(
        thread_handle,
        THREAD_ALL_ACCESS,
        PsThreadType,
        0i8,
        &mut thread2 as *mut PETHREAD as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    if thread != thread2 {
        ObDereferenceObject(thread2 as PVOID);
        return STATUS_OBJECT_TYPE_MISMATCH;
    }

    let primary_token = PsReferencePrimaryToken((*thread2).ThreadsProcess);
    if primary_token.is_null() {
        ObDereferenceObject(thread2 as PVOID);
        return STATUS_NO_TOKEN;
    }

    let mut dacl: PACL = core::ptr::null_mut();
    let status = SepCreateImpersonationTokenDacl(thread_token, primary_token as PTOKEN, &mut dacl);
    ObFastDereferenceObject(
        core::ptr::addr_of_mut!((*(*thread2).ThreadsProcess).Token),
        primary_token as PVOID,
    );
    if status < 0 {
        ObDereferenceObject(thread2 as PVOID);
        return status;
    }

    let mut security_descriptor: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let status = RtlCreateSecurityDescriptor(
        &mut security_descriptor as *mut _SECURITY_DESCRIPTOR as PSECURITY_DESCRIPTOR,
        SECURITY_DESCRIPTOR_REVISION,
    );
    if status < 0 {
        ExFreePoolWithTag(dacl as PVOID, TAG_ACL);
        ObDereferenceObject(thread2 as PVOID);
        return status;
    }

    let status = RtlSetDaclSecurityDescriptor(
        &mut security_descriptor as *mut _SECURITY_DESCRIPTOR as PSECURITY_DESCRIPTOR,
        1u8,
        dacl,
        0u8,
    );
    if status < 0 {
        ExFreePoolWithTag(dacl as PVOID, TAG_ACL);
        ObDereferenceObject(thread2 as PVOID);
        return status;
    }

    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    object_attributes.RootDirectory = core::ptr::null_mut();
    object_attributes.ObjectName = core::ptr::null_mut();
    object_attributes.Attributes = handle_attributes;
    object_attributes.SecurityDescriptor = &mut security_descriptor as *mut _SECURITY_DESCRIPTOR as PVOID;
    object_attributes.SecurityQualityOfService = core::ptr::null_mut();

    let mut new_token: PTOKEN = core::ptr::null_mut();
    let status = SepDuplicateToken(
        thread_token,
        &mut object_attributes,
        effective_only,
        TokenImpersonation,
        impersonation_level,
        0i8,
        &mut new_token,
    );
    if status < 0 {
        ExFreePoolWithTag(dacl as PVOID, TAG_ACL);
        ObDereferenceObject(thread2 as PVOID);
        return status;
    }

    ObReferenceObject(new_token as PVOID);
    let mut token_handle: HANDLE = core::ptr::null_mut();
    let status = ObInsertObject(
        new_token as PVOID,
        core::ptr::null_mut(),
        desired_access,
        0,
        core::ptr::null_mut(),
        &mut token_handle,
    );
    if status < 0 {
        ExFreePoolWithTag(dacl as PVOID, TAG_ACL);
        ObDereferenceObject(new_token as PVOID);
        ObDereferenceObject(thread2 as PVOID);
        return status;
    }

    ExFreePoolWithTag(dacl as PVOID, TAG_ACL);

    let status = PsImpersonateClient(
        thread,
        new_token as PVOID,
        0u8,
        effective_only,
        impersonation_level,
    );
    if status < 0 {
        ObCloseHandle(token_handle, previous_mode);
        ObDereferenceObject(new_token as PVOID);
        ObDereferenceObject(thread2 as PVOID);
        return status;
    }

    *opened_token_handle = token_handle;
    ObDereferenceObject(new_token as PVOID);
    ObDereferenceObject(thread2 as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn SeSubProcessToken(
    parent_token: PTOKEN,
    token: *mut PTOKEN,
    in_use: BOOLEAN,
    session_id: ULONG,
) -> NTSTATUS {
    let parent_token = opaque_ptr(parent_token);
    let mut token = opaque_ptr(token);
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;

    let mut new_token: PTOKEN = core::ptr::null_mut();
    let status = SepDuplicateToken(
        parent_token,
        &mut object_attributes,
        0u8,
        TokenPrimary,
        (*parent_token).ImpersonationLevel,
        0i8,
        &mut new_token,
    );
    if status >= 0 {
        let status = ObInsertObject(
            new_token as PVOID,
            core::ptr::null_mut(),
            0,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if status >= 0 {
            (*new_token).SessionId = session_id;
            (*new_token).TokenInUse = in_use;
            *token = new_token;
        }
    }
    status
}

#[no_mangle]
pub unsafe extern "system" fn SeIsTokenChild(token: PTOKEN, is_child: *mut BOOLEAN) -> NTSTATUS {
    let mut token = opaque_ptr(token);
    let is_child = opaque_ptr(is_child);
    *is_child = 0u8;

    let process_token = PsReferencePrimaryToken(PsGetCurrentProcess() as *mut _EPROCESS);
    if process_token.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    let process_token_id = (*(process_token as PTOKEN)).TokenId;

    ObFastDereferenceObject(
        core::ptr::addr_of_mut!((*(PsGetCurrentProcess() as *mut _EPROCESS)).Token),
        process_token as PVOID,
    );

    let caller_parent_id = (*token).ParentTokenId;

    if RtlEqualLuid(
        &caller_parent_id as *const LUID,
        &process_token_id as *const LUID,
    ) != 0
    {
        *is_child = 1u8;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeIsTokenSibling(
    token: PTOKEN,
    is_sibling: *mut BOOLEAN,
) -> NTSTATUS {
    let mut token = opaque_ptr(token);
    let is_sibling = opaque_ptr(is_sibling);
    *is_sibling = 0u8;

    let process_token = PsReferencePrimaryToken(PsGetCurrentProcess() as *mut _EPROCESS);
    if process_token.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    let process_parent_id = (*(process_token as PTOKEN)).ParentTokenId;
    let process_auth_id = (*(process_token as PTOKEN)).AuthenticationId;

    ObFastDereferenceObject(
        core::ptr::addr_of_mut!((*(PsGetCurrentProcess() as *mut _EPROCESS)).Token),
        process_token as PVOID,
    );

    let caller_parent_id = (*token).ParentTokenId;
    let caller_auth_id = (*token).AuthenticationId;

    if RtlEqualLuid(
        &caller_parent_id as *const LUID,
        &process_parent_id as *const LUID,
    ) != 0
        && RtlEqualLuid(
            &caller_auth_id as *const LUID,
            &process_auth_id as *const LUID,
        ) != 0
    {
        *is_sibling = 1u8;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeCopyClientToken(
    token: PACCESS_TOKEN,
    level: SECURITY_IMPERSONATION_LEVEL,
    previous_mode: KPROCESSOR_MODE,
    new_token: *mut PACCESS_TOKEN,
) -> NTSTATUS {
    let new_token = opaque_ptr(new_token);
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;

    SepDuplicateToken(
        token as PTOKEN,
        &mut object_attributes,
        0u8,
        TokenImpersonation,
        level,
        previous_mode,
        new_token as *mut PTOKEN,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SeTokenIsInert(token: PTOKEN) -> BOOLEAN {
    let mut token = opaque_ptr(token);
    (((*token).TokenFlags & TOKEN_SANDBOX_INERT) != 0) as BOOLEAN
}

unsafe extern "system" fn SepDeleteToken(object_body: PVOID) {
    let access_token = object_body as PTOKEN;

    if !(*access_token).LogonSession.is_null() {
        let status = SepRmRemoveLogonSessionFromToken(access_token);
        if status < 0 {
            return;
        }
    }

    if ((*access_token).TokenFlags & TOKEN_SESSION_NOT_REFERENCED) == 0 {
        SepRmDereferenceLogonSession(&(*access_token).AuthenticationId as *const LUID);
    }

    if !(*access_token).TokenLock.is_null() {
        SepDeleteTokenLock(access_token);
    }

    if !(*access_token).DynamicPart.is_null() {
        ExFreePoolWithTag((*access_token).DynamicPart as PVOID, TAG_TOKEN_DYNAMIC);
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn SepInitializeTokenImplementation() {
    let mut name: _UNICODE_STRING = core::mem::zeroed();
    let mut object_type_initializer: _OBJECT_TYPE_INITIALIZER = core::mem::zeroed();

    core::ptr::write_bytes(&mut object_type_initializer as *mut _OBJECT_TYPE_INITIALIZER as *mut u8, 0, core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>());
    RtlInitUnicodeString(&mut name, w!("Token").as_ptr() as *const u16);
    object_type_initializer.Length = core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>() as u16;
    object_type_initializer.InvalidAttributes = OBJ_OPENLINK;
    object_type_initializer.SecurityRequired = 1u8;
    object_type_initializer.DefaultPagedPoolCharge = core::mem::size_of::<_TOKEN>() as ULONG;
    object_type_initializer.GenericMapping = SepTokenMapping;
    object_type_initializer.PoolType = PagedPool;
    object_type_initializer.ValidAccessMask = TOKEN_ALL_ACCESS;
    object_type_initializer.UseDefaultObject = 1u8;
    object_type_initializer.DeleteProcedure = core::mem::transmute(SepDeleteToken as unsafe extern "system" fn(PVOID));

    _TOKI_TRACE(b"[TOKI] before ObCreateObjectType(Token)\n");
    _TOKI_HEX(b"[TOKI] OTI.Length=", object_type_initializer.Length as ULONG_PTR);
    _TOKI_HEX(b"[TOKI] OTI.PoolType=", object_type_initializer.PoolType as ULONG_PTR);
    _TOKI_HEX(b"[TOKI] OTI.UseDefaultObject=", object_type_initializer.UseDefaultObject as ULONG_PTR);
    let _st = ObCreateObjectType(&mut name, &mut object_type_initializer, core::ptr::null_mut(), core::ptr::addr_of_mut!(SeTokenObjectType) as *mut *mut _OBJECT_TYPE);
    _TOKI_HEX(b"[TOKI] ObCreateObjectType returned=", _st as ULONG_PTR);
    _TOKI_HEX(b"[TOKI] SeTokenObjectType as *mut _OBJECT_TYPE=", SeTokenObjectType as ULONG_PTR);
}

#[no_mangle]
pub unsafe extern "system" fn SeAssignPrimaryToken(process: PEPROCESS, token: PTOKEN) {
    let process = opaque_ptr(process);
    let mut token = opaque_ptr(token);
    if !(*process).Token.__bindgen_anon_1.Object.is_null() {
        SeDeassignPrimaryToken(process);
    }

    ObReferenceObject(token as PVOID);
    (*token).TokenInUse = 1u8;
    ObInitializeFastReference(
        core::ptr::addr_of_mut!((*process).Token),
        token as PVOID,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SeGetTokenControlInformation(
    _token: PACCESS_TOKEN,
    token_control: *mut _TOKEN_CONTROL,
) {
    let token_control = opaque_ptr(token_control);
    let token = _token as PTOKEN;

    (*token_control).AuthenticationId = (*token).AuthenticationId;
    (*token_control).TokenId = (*token).TokenId;
    (*token_control).TokenSource = (*token).TokenSource;

    SepAcquireTokenLockShared(token);
    (*token_control).ModifiedId = (*token).ModifiedId;
    SepReleaseTokenLock(token);
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn SepCreateSystemProcessToken() -> PTOKEN {
    let mut expiration: _LARGE_INTEGER = core::mem::zeroed();
    expiration.QuadPart = -1i64;

    let group_attributes = SE_GROUP_ENABLED | SE_GROUP_MANDATORY | SE_GROUP_ENABLED_BY_DEFAULT;
    let owner_attributes = SE_GROUP_ENABLED | SE_GROUP_OWNER | SE_GROUP_ENABLED_BY_DEFAULT;

    let user_sid = _SID_AND_ATTRIBUTES {
        Sid: SeLocalSystemSid,
        Attributes: 0,
    };

    let primary_group = SeLocalSystemSid;
    let owner = SeAliasAdminsSid;

    let mut groups = [
        _SID_AND_ATTRIBUTES {
            Sid: SeAliasAdminsSid,
            Attributes: owner_attributes,
        },
        _SID_AND_ATTRIBUTES {
            Sid: SeWorldSid,
            Attributes: group_attributes,
        },
        _SID_AND_ATTRIBUTES {
            Sid: SeAuthenticatedUsersSid,
            Attributes: group_attributes,
        },
        _SID_AND_ATTRIBUTES {
            Sid: SeLocalSid,
            Attributes: SE_GROUP_ENABLED,
        },
    ];
    let groups_length = core::mem::size_of::<_SID_AND_ATTRIBUTES>() as ULONG
        + SeLengthSid(groups[0].Sid)
        + SeLengthSid(groups[1].Sid)
        + SeLengthSid(groups[2].Sid)
        + SeLengthSid(groups[3].Sid);

    let mut privileges = [
        _LUID_AND_ATTRIBUTES {
            Luid: SeTcbPrivilege,
            Attributes: SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeCreateTokenPrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeTakeOwnershipPrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeCreatePagefilePrivilege,
            Attributes: SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeLockMemoryPrivilege,
            Attributes: SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeAssignPrimaryTokenPrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeIncreaseQuotaPrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeIncreaseBasePriorityPrivilege,
            Attributes: SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeCreatePermanentPrivilege,
            Attributes: SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeDebugPrivilege,
            Attributes: SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeAuditPrivilege,
            Attributes: SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeSecurityPrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeSystemEnvironmentPrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeChangeNotifyPrivilege,
            Attributes: SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeBackupPrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeRestorePrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeShutdownPrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeLoadDriverPrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeProfileSingleProcessPrivilege,
            Attributes: SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeSystemtimePrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeUndockPrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeManageVolumePrivilege,
            Attributes: 0,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeImpersonatePrivilege,
            Attributes: SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED,
        },
        _LUID_AND_ATTRIBUTES {
            Luid: SeCreateGlobalPrivilege,
            Attributes: SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED,
        },
    ];

    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;

    let mut token: PTOKEN = core::ptr::null_mut();
    let _status = SepCreateToken(
        &mut token as *mut PTOKEN as *mut HANDLE,
        0i8,
        0,
        &mut object_attributes,
        TokenPrimary,
        SecurityAnonymous,
        &raw const SeSystemAuthenticationId as *const LUID,
        &expiration as *const _LARGE_INTEGER as *const LARGE_INTEGER,
        &user_sid as *const _SID_AND_ATTRIBUTES,
        4,
        groups.as_ptr(),
        groups_length,
        24,
        privileges.as_ptr(),
        owner,
        primary_group,
        SeSystemDefaultDacl,
        &raw const SeSystemTokenSource as *const _TOKEN_SOURCE,
        1u8,
    );

    token
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn SepCreateSystemAnonymousLogonToken() -> PTOKEN {
    let mut expiration: _LARGE_INTEGER = core::mem::zeroed();
    expiration.QuadPart = -1i64;

    let user_sid = _SID_AND_ATTRIBUTES {
        Sid: SeAnonymousLogonSid,
        Attributes: 0,
    };

    let primary_group = SeAnonymousLogonSid;

    let mut groups = [_SID_AND_ATTRIBUTES {
        Sid: SeWorldSid,
        Attributes: SE_GROUP_ENABLED | SE_GROUP_MANDATORY | SE_GROUP_ENABLED_BY_DEFAULT,
    }];
    let groups_length =
        core::mem::size_of::<_SID_AND_ATTRIBUTES>() as ULONG + SeLengthSid(groups[0].Sid);

    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;

    let mut token: PTOKEN = core::ptr::null_mut();
    let _status = SepCreateToken(
        &mut token as *mut PTOKEN as *mut HANDLE,
        0i8,
        0,
        &mut object_attributes,
        TokenPrimary,
        SecurityAnonymous,
        &raw const SeAnonymousAuthenticationId as *const LUID,
        &expiration as *const _LARGE_INTEGER as *const LARGE_INTEGER,
        &user_sid as *const _SID_AND_ATTRIBUTES,
        1,
        groups.as_ptr(),
        groups_length,
        0,
        core::ptr::null(),
        core::ptr::null_mut(),
        primary_group,
        SeSystemAnonymousLogonDacl,
        &raw const SeSystemTokenSource as *const _TOKEN_SOURCE,
        1u8,
    );

    token
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn SepCreateSystemAnonymousLogonTokenNoEveryone() -> PTOKEN {
    let mut expiration: _LARGE_INTEGER = core::mem::zeroed();
    expiration.QuadPart = -1i64;

    let user_sid = _SID_AND_ATTRIBUTES {
        Sid: SeAnonymousLogonSid,
        Attributes: 0,
    };

    let primary_group = SeAnonymousLogonSid;

    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;

    let mut token: PTOKEN = core::ptr::null_mut();
    let _status = SepCreateToken(
        &mut token as *mut PTOKEN as *mut HANDLE,
        0i8,
        0,
        &mut object_attributes,
        TokenPrimary,
        SecurityAnonymous,
        &raw const SeAnonymousAuthenticationId as *const LUID,
        &expiration as *const _LARGE_INTEGER as *const LARGE_INTEGER,
        &user_sid as *const _SID_AND_ATTRIBUTES,
        0,
        core::ptr::null(),
        0,
        0,
        core::ptr::null(),
        core::ptr::null_mut(),
        primary_group,
        SeSystemAnonymousLogonDacl,
        &raw const SeSystemTokenSource as *const _TOKEN_SOURCE,
        1u8,
    );

    token
}

#[no_mangle]
pub unsafe extern "system" fn SeQuerySessionIdToken(
    token: PACCESS_TOKEN,
    p_session_id: *mut ULONG,
) -> NTSTATUS {
    let p_session_id = opaque_ptr(p_session_id);
    SepAcquireTokenLockShared(token as PTOKEN);
    *p_session_id = (*(token as PTOKEN)).SessionId;
    SepReleaseTokenLock(token as PTOKEN);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeQueryAuthenticationIdToken(
    token: PACCESS_TOKEN,
    logon_id: PLUID,
) -> NTSTATUS {
    let logon_id = opaque_ptr(logon_id);
    *logon_id = (*(token as PTOKEN)).AuthenticationId;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeTokenImpersonationLevel(
    token: PACCESS_TOKEN,
) -> SECURITY_IMPERSONATION_LEVEL {
    (*(token as PTOKEN)).ImpersonationLevel
}

#[no_mangle]
pub unsafe extern "system" fn SeTokenType(token: PACCESS_TOKEN) -> TOKEN_TYPE {
    (*(token as PTOKEN)).TokenType
}

#[no_mangle]
pub unsafe extern "system" fn SeTokenIsAdmin(token: PACCESS_TOKEN) -> BOOLEAN {
    (((*(token as PTOKEN)).TokenFlags & TOKEN_HAS_ADMIN_GROUP) != 0) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn SeTokenIsRestricted(token: PACCESS_TOKEN) -> BOOLEAN {
    (((*(token as PTOKEN)).TokenFlags & TOKEN_IS_RESTRICTED) != 0) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn SeTokenIsWriteRestricted(token: PACCESS_TOKEN) -> BOOLEAN {
    (((*(token as PTOKEN)).TokenFlags & SE_BACKUP_PRIVILEGES_CHECKED) != 0) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn SeTokenCanImpersonate(
    process_token: PTOKEN,
    token_to_impersonate: PTOKEN,
    impersonation_level: SECURITY_IMPERSONATION_LEVEL,
) -> BOOLEAN {
    let process_token = opaque_ptr(process_token);
    let token_to_impersonate = opaque_ptr(token_to_impersonate);
    if impersonation_level < SecurityIdentification {
        return 1u8;
    }

    SepAcquireTokenLockShared(process_token);
    SepAcquireTokenLockShared(token_to_impersonate);

    if RtlEqualLuid(
        &(*token_to_impersonate).AuthenticationId as *const LUID,
        &raw const SeAnonymousAuthenticationId as *const LUID,
    ) != 0
    {
        SepReleaseTokenLock(token_to_impersonate);
        SepReleaseTokenLock(process_token);
        return 1u8;
    }

    if ((*process_token).TokenFlags & TOKEN_HAS_IMPERSONATE_PRIVILEGE) != 0 {
        SepReleaseTokenLock(token_to_impersonate);
        SepReleaseTokenLock(process_token);
        return 1u8;
    }

    if impersonation_level > (*token_to_impersonate).ImpersonationLevel {
        SepReleaseTokenLock(token_to_impersonate);
        SepReleaseTokenLock(process_token);
        return 0u8;
    }

    if RtlEqualLuid(
        &(*process_token).AuthenticationId as *const LUID,
        &(*token_to_impersonate).OriginatingLogonSession as *const LUID,
    ) == 0
    {
        if RtlEqualSid(
            (*(*process_token).UserAndGroups.offset(0)).Sid,
            (*(*token_to_impersonate).UserAndGroups.offset(0)).Sid,
        ) == 0
        {
            SepReleaseTokenLock(token_to_impersonate);
            SepReleaseTokenLock(process_token);
            return 0u8;
        }

        if SeTokenIsRestricted(process_token as PVOID) != SeTokenIsRestricted(token_to_impersonate as PVOID) {
            SepReleaseTokenLock(token_to_impersonate);
            SepReleaseTokenLock(process_token);
            return 0u8;
        }
    }

    SepReleaseTokenLock(token_to_impersonate);
    SepReleaseTokenLock(process_token);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenThreadTokenEx(
    thread_handle: HANDLE,
    desired_access: ACCESS_MASK,
    open_as_self: BOOLEAN,
    handle_attributes: ULONG,
    token_handle: PHANDLE,
) -> NTSTATUS {
    let thread_handle = opaque_ptr(thread_handle);
    let token_handle = opaque_ptr(token_handle);
    let previous_mode = ExGetPreviousMode();

    if previous_mode != 0i8 {
        if let Err(s) = probe_write_handle(token_handle as _, previous_mode) {
            return s;
        }
    }

    let handle_attributes = ObpValidateAttributes(handle_attributes, previous_mode);

    let mut thread: PETHREAD = core::ptr::null_mut();
    let mut status = ObReferenceObjectByHandle(
        thread_handle,
        THREAD_QUERY_INFORMATION,
        PsThreadType,
        previous_mode,
        &mut thread as *mut PETHREAD as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    let mut copy_on_open: BOOLEAN = 0;
    let mut effective_only: BOOLEAN = 0;
    let mut impersonation_level: SECURITY_IMPERSONATION_LEVEL = 0;
    let token = PsReferenceImpersonationToken(
        thread,
        &mut copy_on_open,
        &mut effective_only,
        &mut impersonation_level,
    );
    if token.is_null() {
        ObDereferenceObject(thread as PVOID);
        return STATUS_NO_TOKEN;
    }

    if impersonation_level == SecurityAnonymous {
        PsDereferenceImpersonationToken(token);
        ObDereferenceObject(thread as PVOID);
        return STATUS_CANT_OPEN_ANONYMOUS;
    }

    let mut restore_impersonation = 0u8;
    let mut impersonation_state: SE_IMPERSONATION_STATE = core::mem::zeroed();
    if open_as_self != 0 {
        restore_impersonation = PsDisableImpersonation(
            PsGetCurrentThread() as *mut _ETHREAD,
            &mut impersonation_state,
        );
    }

    let mut h_token: HANDLE = core::ptr::null_mut();
    status = SepOpenThreadToken(
        thread,
        thread_handle,
        token,
        desired_access,
        handle_attributes,
        effective_only,
        copy_on_open,
        impersonation_level,
        previous_mode,
        &mut h_token,
    );

    if restore_impersonation != 0 {
        PsRestoreImpersonation(
            PsGetCurrentThread() as *mut _ETHREAD,
            &mut impersonation_state,
        );
    }

    ObDereferenceObject(token as PVOID);
    ObDereferenceObject(thread as PVOID);

    if status < 0 {
        return status;
    }

    if let Err(s) = probe_write_user(token_handle, h_token, previous_mode) {
        return s;
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenThreadToken(
    thread_handle: HANDLE,
    desired_access: ACCESS_MASK,
    open_as_self: BOOLEAN,
    token_handle: PHANDLE,
) -> NTSTATUS {
    let thread_handle = opaque_ptr(thread_handle);
    let token_handle = opaque_ptr(token_handle);
    NtOpenThreadTokenEx(thread_handle, desired_access, open_as_self, 0, token_handle)
}

#[no_mangle]
pub unsafe extern "system" fn NtCompareTokens(
    first_token_handle: HANDLE,
    second_token_handle: HANDLE,
    equal: *mut BOOLEAN,
) -> NTSTATUS {
    let first_token_handle = opaque_ptr(first_token_handle);
    let second_token_handle = opaque_ptr(second_token_handle);
    let equal = opaque_ptr(equal);
    let previous_mode = ExGetPreviousMode();

    if previous_mode != 0i8 {
        if let Err(s) = probe_write_boolean(equal as _, previous_mode) {
            return s;
        }
    }

    let mut first_token: PTOKEN = core::ptr::null_mut();
    let mut status = ObReferenceObjectByHandle(
        first_token_handle, TOKEN_QUERY,
        SeTokenObjectType as *mut _OBJECT_TYPE,
        previous_mode,
        &mut first_token as *mut PTOKEN as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    let mut second_token: PTOKEN = core::ptr::null_mut();
    status = ObReferenceObjectByHandle(
        second_token_handle,
        TOKEN_QUERY,
        SeTokenObjectType as *mut _OBJECT_TYPE,
        previous_mode,
        &mut second_token as *mut PTOKEN as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        ObDereferenceObject(first_token as PVOID);
        return status;
    }

    let mut is_equal = 0u8;
    if first_token != second_token {
        status = SepCompareTokens(first_token, second_token, &mut is_equal);
    } else {
        is_equal = 1u8;
    }

    ObDereferenceObject(second_token as PVOID);
    ObDereferenceObject(first_token as PVOID);

    if status >= 0 {
        if let Err(s) = probe_write_user(equal, is_equal, previous_mode) {
            return s;
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtImpersonateAnonymousToken(
    thread_handle: HANDLE,
) -> NTSTATUS {
    let thread_handle = opaque_ptr(thread_handle);
    let previous_mode = ExGetPreviousMode();

    let mut thread: PETHREAD = core::ptr::null_mut();
    let status = ObReferenceObjectByHandle(
        thread_handle,
        THREAD_IMPERSONATE,
        PsThreadType,
        previous_mode,
        &mut thread as *mut PETHREAD as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    let status = SepImpersonateAnonymousToken(thread, previous_mode);

    ObDereferenceObject(thread as PVOID);
    status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Security access token implementation base support routines
//  * COPYRIGHT:   Copyright David Welch <welch@cwcom.net>
//  *              Copyright 2021-2023 George Bișoc <george.bisoc@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* DEBUG: UART trace for crash diagnosis */
// #ifdef _M_ARM64
// #define _TOKI_TRACE(msg) do { \
//     extern volatile PVOID Arm64UartKernelBase; \
//     volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase; \
//     if (_u) { const char *_m = (msg); while (*_m) *_u = (ULONG)*_m++; } \
// } while(0)
// #define _TOKI_HEX(prefix, val) do { \
//     extern volatile PVOID Arm64UartKernelBase; \
//     volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase; \
//     if (_u) { \
//         const char *_m = (prefix); while (*_m) *_u = (ULONG)*_m++; \
//         ULONG_PTR _v = (ULONG_PTR)(val); \
//         const char _hex[] = "0123456789ABCDEF"; \
//         *_u = (ULONG)'0'; *_u = (ULONG)'x'; \
//         for (int _i = 60; _i >= 0; _i -= 4) *_u = (ULONG)_hex[(_v >> _i) & 0xF]; \
//         *_u = (ULONG)'\n'; \
//     } \
// } while(0)
// #else
// #define _TOKI_TRACE(msg) ((void)0)
// #define _TOKI_HEX(prefix, val) ((void)0)
// #endif
//
//
// /* GLOBALS ********************************************************************/
//
// POBJECT_TYPE SeTokenObjectType as *mut _OBJECT_TYPE = NULL;
//
// TOKEN_SOURCE SeSystemTokenSource = {"*SYSTEM*", {0}};
// LUID SeSystemAuthenticationId = SYSTEM_LUID;
// LUID SeAnonymousAuthenticationId = ANONYMOUS_LOGON_LUID;
//
// static GENERIC_MAPPING SepTokenMapping = {
//     TOKEN_READ,
//     TOKEN_WRITE,
//     TOKEN_EXECUTE,
//     TOKEN_ALL_ACCESS
// };
//
// /* PRIVATE FUNCTIONS *****************************************************************/
//
// /**
//  * @brief
//  * Creates a lock for the token.
//  *
//  * @param[in,out] Token
//  * A token which lock has to be created.
//  *
//  * @return
//  * STATUS_SUCCESS if the pool allocation and resource initialisation have
//  * completed successfully, otherwise STATUS_INSUFFICIENT_RESOURCES on a
//  * pool allocation failure.
//  */
// NTSTATUS
// SepCreateTokenLock(
//     _Inout_ PTOKEN Token)
// {
//     PAGED_CODE();
//
//     Token->TokenLock = ExAllocatePoolWithTag(NonPagedPool,
//                                              sizeof(ERESOURCE),
//                                              TAG_SE_TOKEN_LOCK);
//     if (Token->TokenLock == NULL)
//     {
//         DPRINT1("SepCreateTokenLock(): Failed to allocate memory!\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     ExInitializeResourceLite(Token->TokenLock);
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Deletes a lock of a token.
//  *
//  * @param[in,out] Token
//  * A token which contains the lock.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// SepDeleteTokenLock(
//     _Inout_ PTOKEN Token)
// {
//     PAGED_CODE();
//
//     ExDeleteResourceLite(Token->TokenLock);
//     ExFreePoolWithTag(Token->TokenLock, TAG_SE_TOKEN_LOCK);
// }
//
// /**
//  * @brief
//  * Compares the elements of SID arrays provided by tokens.
//  * The elements that are being compared for equality are
//  * the SIDs and their attributes.
//  *
//  * @param[in] SidArrayToken1
//  * SID array from the first token.
//  *
//  * @param[in] CountSidArray1
//  * SID count array from the first token.
//  *
//  * @param[in] SidArrayToken2
//  * SID array from the second token.
//  *
//  * @param[in] CountSidArray2
//  * SID count array from the second token.
//  *
//  * @return
//  * Returns TRUE if the elements match from either arrays,
//  * FALSE otherwise.
//  */
// static
// BOOLEAN
// SepCompareSidAndAttributesFromTokens(
//     _In_ PSID_AND_ATTRIBUTES SidArrayToken1,
//     _In_ ULONG CountSidArray1,
//     _In_ PSID_AND_ATTRIBUTES SidArrayToken2,
//     _In_ ULONG CountSidArray2)
// {
//     ULONG FirstCount, SecondCount;
//     PSID_AND_ATTRIBUTES FirstSidArray, SecondSidArray;
//     PAGED_CODE();
//
//     /* Bail out if index counters provided are not equal */
//     if (CountSidArray1 != CountSidArray2)
//     {
//         DPRINT("SepCompareSidAndAttributesFromTokens(): Index counters are not the same!\n");
//         return FALSE;
//     }
//
//     /* Loop over the SID arrays and compare them */
//     for (FirstCount = 0; FirstCount < CountSidArray1; FirstCount++)
//     {
//         for (SecondCount = 0; SecondCount < CountSidArray2; SecondCount++)
//         {
//             FirstSidArray = &SidArrayToken1[FirstCount];
//             SecondSidArray = &SidArrayToken2[SecondCount];
//
//             if (RtlEqualSid(FirstSidArray->Sid, SecondSidArray->Sid) &&
//                 FirstSidArray->Attributes == SecondSidArray->Attributes)
//             {
//                 break;
//             }
//         }
//
//         /* We've exhausted the array of the second token without finding this one */
//         if (SecondCount == CountSidArray2)
//         {
//             DPRINT("SepCompareSidAndAttributesFromTokens(): No matching elements could be found in either token!\n");
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Compares the elements of privilege arrays provided by tokens.
//  * The elements that are being compared for equality are
//  * the privileges and their attributes.
//  *
//  * @param[in] PrivArrayToken1
//  * Privilege array from the first token.
//  *
//  * @param[in] CountPrivArray1
//  * Privilege count array from the first token.
//  *
//  * @param[in] PrivArrayToken2
//  * Privilege array from the second token.
//  *
//  * @param[in] CountPrivArray2
//  * Privilege count array from the second token.
//  *
//  * @return
//  * Returns TRUE if the elements match from either arrays,
//  * FALSE otherwise.
//  */
// static
// BOOLEAN
// SepComparePrivilegeAndAttributesFromTokens(
//     _In_ PLUID_AND_ATTRIBUTES PrivArrayToken1,
//     _In_ ULONG CountPrivArray1,
//     _In_ PLUID_AND_ATTRIBUTES PrivArrayToken2,
//     _In_ ULONG CountPrivArray2)
// {
//     ULONG FirstCount, SecondCount;
//     PLUID_AND_ATTRIBUTES FirstPrivArray, SecondPrivArray;
//     PAGED_CODE();
//
//     /* Bail out if index counters provided are not equal */
//     if (CountPrivArray1 != CountPrivArray2)
//     {
//         DPRINT("SepComparePrivilegeAndAttributesFromTokens(): Index counters are not the same!\n");
//         return FALSE;
//     }
//
//     /* Loop over the privilege arrays and compare them */
//     for (FirstCount = 0; FirstCount < CountPrivArray1; FirstCount++)
//     {
//         for (SecondCount = 0; SecondCount < CountPrivArray2; SecondCount++)
//         {
//             FirstPrivArray = &PrivArrayToken1[FirstCount];
//             SecondPrivArray = &PrivArrayToken2[SecondCount];
//
//             if (RtlEqualLuid(&FirstPrivArray->Luid, &SecondPrivArray->Luid) &&
//                 FirstPrivArray->Attributes == SecondPrivArray->Attributes)
//             {
//                 break;
//             }
//         }
//
//         /* We've exhausted the array of the second token without finding this one */
//         if (SecondCount == CountPrivArray2)
//         {
//             DPRINT("SepComparePrivilegeAndAttributesFromTokens(): No matching elements could be found in either token!\n");
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Compares tokens if they're equal based on all the following properties. If all
//  * of the said conditions are met then the tokens are deemed as equal.
//  *
//  * - Every SID that is present in either token is also present in the other one.
//  * - Both or none of the tokens are restricted.
//  * - If both tokens are restricted, every SID that is restricted in either token is
//  *   also restricted in the other one.
//  * - Every privilege present in either token is also present in the other one.
//  *
//  * @param[in] FirstToken
//  * The first token.
//  *
//  * @param[in] SecondToken
//  * The second token.
//  *
//  * @param[out] Equal
//  * The retrieved value which determines if the tokens are
//  * equal or not.
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  */
// static
// NTSTATUS
// SepCompareTokens(
//     _In_ PTOKEN FirstToken,
//     _In_ PTOKEN SecondToken,
//     _Out_ PBOOLEAN Equal)
// {
//     BOOLEAN Restricted, IsEqual = FALSE;
//     PAGED_CODE();
//
//     ASSERT(FirstToken != SecondToken);
//
//     /* Lock the tokens */
//     SepAcquireTokenLockShared(FirstToken);
//     SepAcquireTokenLockShared(SecondToken);
//
//     /* Check if every SID that is present in either token is also present in the other one */
//     if (!SepCompareSidAndAttributesFromTokens(FirstToken->UserAndGroups,
//                                               FirstToken->UserAndGroupCount,
//                                               SecondToken->UserAndGroups,
//                                               SecondToken->UserAndGroupCount))
//     {
//         goto Quit;
//     }
//
//     /* Is one token restricted but the other isn't? */
//     Restricted = SeTokenIsRestricted(FirstToken);
//     if (Restricted != SeTokenIsRestricted(SecondToken))
//     {
//         /* If that's the case then bail out */
//         goto Quit;
//     }
//
//     /*
//      * If both tokens are restricted check if every SID
//      * that is restricted in either token is also restricted
//      * in the other one.
//      */
//     if (Restricted)
//     {
//         if (!SepCompareSidAndAttributesFromTokens(FirstToken->RestrictedSids,
//                                                   FirstToken->RestrictedSidCount,
//                                                   SecondToken->RestrictedSids,
//                                                   SecondToken->RestrictedSidCount))
//         {
//             goto Quit;
//         }
//     }
//
//     /* Check if every privilege present in either token is also present in the other one */
//     if (!SepComparePrivilegeAndAttributesFromTokens(FirstToken->Privileges,
//                                                     FirstToken->PrivilegeCount,
//                                                     SecondToken->Privileges,
//                                                     SecondToken->PrivilegeCount))
//     {
//         goto Quit;
//     }
//
//     /* If we're here then the tokens are equal */
//     IsEqual = TRUE;
//     DPRINT("SepCompareTokens(): Tokens are equal!\n");
//
// Quit:
//     /* Unlock the tokens */
//     SepReleaseTokenLock(SecondToken);
//     SepReleaseTokenLock(FirstToken);
//
//     *Equal = IsEqual;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Private function that impersonates the system's anonymous logon token.
//  * The major bulk of the impersonation procedure is done here.
//  *
//  * @param[in] Thread
//  * The executive thread object that is to impersonate the client.
//  *
//  * @param[in] PreviousMode
//  * The access processor mode, indicating if the call is executed
//  * in kernel or user mode.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the impersonation has succeeded.
//  * STATUS_UNSUCCESSFUL is returned if the primary token couldn't be
//  * obtained from the current process to perform additional tasks.
//  * STATUS_ACCESS_DENIED is returned if the process' primary token is
//  * restricted, which for this matter we cannot impersonate onto a
//  * restricted process. Otherwise a failure NTSTATUS code is returned.
//  */
// static
// NTSTATUS
// SepImpersonateAnonymousToken(
//     _In_ PETHREAD Thread,
//     _In_ KPROCESSOR_MODE PreviousMode)
// {
//     NTSTATUS Status;
//     PTOKEN TokenToImpersonate, ProcessToken;
//     ULONG IncludeEveryoneValueData;
//     PAGED_CODE();
//
//     /*
//      * We must check first which kind of token
//      * shall we assign for the thread to impersonate,
//      * the one with Everyone Group SID or the other
//      * without. Invoke the registry helper to
//      * return the data value for us.
//      */
//     Status = SepRegQueryHelper(L"\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\Lsa",
//                                L"EveryoneIncludesAnonymous",
//                                REG_DWORD,
//                                sizeof(IncludeEveryoneValueData),
//                                &IncludeEveryoneValueData);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SepRegQueryHelper(): Failed to query the registry value (Status 0x%lx)\n", Status);
//         return Status;
//     }
//
//     if (IncludeEveryoneValueData == 0)
//     {
//         DPRINT("SepImpersonateAnonymousToken(): Assigning the token not including the Everyone Group SID...\n");
//         TokenToImpersonate = SeAnonymousLogonTokenNoEveryone;
//     }
//     else
//     {
//         DPRINT("SepImpersonateAnonymousToken(): Assigning the token including the Everyone Group SID...\n");
//         TokenToImpersonate = SeAnonymousLogonToken;
//     }
//
//     /*
//      * Tell the object manager that we're going to use this token
//      * object now by incrementing the reference count.
//     */
//     Status = ObReferenceObjectByPointer(TokenToImpersonate,
//                                         TOKEN_IMPERSONATE,
//                                         SeTokenObjectType as *mut _OBJECT_TYPE,
//                                         PreviousMode);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SepImpersonateAnonymousToken(): Couldn't be able to use the token, bail out...\n");
//         return Status;
//     }
//
//     /*
//      * Reference the primary token of the current process that the anonymous
//      * logon token impersonation procedure is being performed. We'll be going
//      * to use the process' token to figure out if the process is actually
//      * restricted or not.
//      */
//     ProcessToken = PsReferencePrimaryToken(PsGetCurrentProcess());
//     if (!ProcessToken)
//     {
//         DPRINT1("SepImpersonateAnonymousToken(): Couldn't be able to get the process' primary token, bail out...\n");
//         ObDereferenceObject(TokenToImpersonate);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     /* Now, is the token from the current process restricted? */
//     if (SeTokenIsRestricted(ProcessToken))
//     {
//         DPRINT1("SepImpersonateAnonymousToken(): The process is restricted, can't do anything. Bail out...\n");
//         PsDereferencePrimaryToken(ProcessToken);
//         ObDereferenceObject(TokenToImpersonate);
//         return STATUS_ACCESS_DENIED;
//     }
//
//     /*
//      * Finally it's time to impersonate! But first, fast dereference the
//      * process' primary token as we no longer need it.
//      */
//     ObFastDereferenceObject(&PsGetCurrentProcess()->Token, ProcessToken);
//     Status = PsImpersonateClient(Thread, TokenToImpersonate, TRUE, FALSE, SecurityImpersonation);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SepImpersonateAnonymousToken(): Failed to impersonate, bail out...\n");
//         ObDereferenceObject(TokenToImpersonate);
//         return Status;
//     }
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Updates the token's flags based upon the privilege that the token
//  * has been granted. The flag can either be taken out or given to the token
//  * if the attributes of the specified privilege is enabled or not.
//  *
//  * @param[in,out] Token
//  * The token where the flags are to be changed.
//  *
//  * @param[in] Index
//  * The index count which represents the total sum of privileges. The count in question
//  * MUST NOT exceed the expected privileges count of the token.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// SepUpdateSinglePrivilegeFlagToken(
//     _Inout_ PTOKEN Token,
//     _In_ ULONG Index)
// {
//     ULONG TokenFlag;
//     ASSERT(Index < Token->PrivilegeCount);
//
//     /* The high part of all values we are interested in is 0 */
//     if (Token->Privileges[Index].Luid.HighPart != 0)
//     {
//         return;
//     }
//
//     /* Check for certain privileges to update flags */
//     if (Token->Privileges[Index].Luid.LowPart == SE_CHANGE_NOTIFY_PRIVILEGE)
//     {
//         TokenFlag = TOKEN_HAS_TRAVERSE_PRIVILEGE;
//     }
//     else if (Token->Privileges[Index].Luid.LowPart == SE_BACKUP_PRIVILEGE)
//     {
//         TokenFlag = TOKEN_HAS_BACKUP_PRIVILEGE;
//     }
//     else if (Token->Privileges[Index].Luid.LowPart == SE_RESTORE_PRIVILEGE)
//     {
//         TokenFlag = TOKEN_HAS_RESTORE_PRIVILEGE;
//     }
//     else if (Token->Privileges[Index].Luid.LowPart == SE_IMPERSONATE_PRIVILEGE)
//     {
//         TokenFlag = TOKEN_HAS_IMPERSONATE_PRIVILEGE;
//     }
//     else
//     {
//         /* Nothing to do */
//         return;
//     }
//
//     /* Check if the specified privilege is enabled */
//     if (Token->Privileges[Index].Attributes & SE_PRIVILEGE_ENABLED)
//     {
//         /* It is enabled, so set the flag */
//         Token->TokenFlags |= TokenFlag;
//     }
//     else
//     {
//         /* Is is disabled, so remove the flag */
//         Token->TokenFlags &= ~TokenFlag;
//     }
// }
//
// /**
//  * @brief
//  * Checks if a token belongs to the main user, being the owner.
//  *
//  * @param[in] _Token
//  * A valid token object.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor where the owner is to be found.
//  *
//  * @param[in] TokenLocked
//  * If set to TRUE, the token has been already locked and there's
//  * no need to lock it again. Otherwise the function will acquire
//  * the lock.
//  *
//  * @return
//  * Returns TRUE if the token belongs to a owner, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SepTokenIsOwner(
//     _In_ PACCESS_TOKEN _Token,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_ BOOLEAN TokenLocked)
// {
//     PSID Sid;
//     BOOLEAN Result;
//     PTOKEN Token = _Token;
//
//     /* Get the owner SID */
//     Sid = SepGetOwnerFromDescriptor(SecurityDescriptor);
//     ASSERT(Sid != NULL);
//
//     /* Lock the token if needed */
//     if (!TokenLocked) SepAcquireTokenLockShared(Token);
//
//     /* Check if the owner SID is found, handling restricted case as well */
//     Result = SepSidInToken(Token, Sid);
//     if ((Result) && (Token->TokenFlags & TOKEN_IS_RESTRICTED))
//     {
//         Result = SepSidInTokenEx(Token, NULL, Sid, FALSE, TRUE);
//     }
//
//     /* Release the lock if we had acquired it */
//     if (!TokenLocked) SepReleaseTokenLock(Token);
//
//     /* Return the result */
//     return Result;
// }
//
// /**
//  * @brief
//  * Updates the token's flags based upon the privilege that the token
//  * has been granted. The function uses the private helper, SepUpdateSinglePrivilegeFlagToken,
//  * in order to update the flags of a token.
//  *
//  * @param[in,out] Token
//  * The token where the flags are to be changed.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// SepUpdatePrivilegeFlagsToken(
//     _Inout_ PTOKEN Token)
// {
//     ULONG i;
//
//     /* Loop all privileges */
//     for (i = 0; i < Token->PrivilegeCount; i++)
//     {
//         /* Updates the flags for this privilege */
//         SepUpdateSinglePrivilegeFlagToken(Token, i);
//     }
// }
//
// /**
//  * @brief
//  * Removes a privilege from the token.
//  *
//  * @param[in,out] Token
//  * The token where the privilege is to be removed.
//  *
//  * @param[in] Index
//  * The index count which represents the number position of the privilege
//  * we want to remove.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// SepRemovePrivilegeToken(
//     _Inout_ PTOKEN Token,
//     _In_ ULONG Index)
// {
//     ULONG MoveCount;
//     ASSERT(Index < Token->PrivilegeCount);
//
//     /* Calculate the number of trailing privileges */
//     MoveCount = Token->PrivilegeCount - Index - 1;
//     if (MoveCount != 0)
//     {
//         /* Move them one location ahead */
//         RtlMoveMemory(&Token->Privileges[Index],
//                       &Token->Privileges[Index + 1],
//                       MoveCount * sizeof(LUID_AND_ATTRIBUTES));
//     }
//
//     /* Update privilege count */
//     Token->PrivilegeCount--;
// }
//
// /**
//  * @brief
//  * Removes a group from the token.
//  *
//  * @param[in,out] Token
//  * The token where the group is to be removed.
//  *
//  * @param[in] Index
//  * The index count which represents the number position of the group
//  * we want to remove.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// SepRemoveUserGroupToken(
//     _Inout_ PTOKEN Token,
//     _In_ ULONG Index)
// {
//     ULONG MoveCount;
//     ASSERT(Index < Token->UserAndGroupCount);
//
//     /* Calculate the number of trailing groups */
//     MoveCount = Token->UserAndGroupCount - Index - 1;
//     if (MoveCount != 0)
//     {
//         /* Time to remove the group by moving one location ahead */
//         RtlMoveMemory(&Token->UserAndGroups[Index],
//                       &Token->UserAndGroups[Index + 1],
//                       MoveCount * sizeof(SID_AND_ATTRIBUTES));
//     }
//
//     /* Remove one group count */
//     Token->UserAndGroupCount--;
// }
//
// /**
//  * @brief
//  * Computes the exact available dynamic area of an access
//  * token whilst querying token statistics.
//  *
//  * @param[in] DynamicCharged
//  * The current charged dynamic area of an access token.
//  * This must not be 0!
//  *
//  * @param[in] PrimaryGroup
//  * A pointer to a primary group SID.
//  *
//  * @param[in] DefaultDacl
//  * If provided, this pointer points to a default DACL of an
//  * access token.
//  *
//  * @return
//  * Returns the calculated available dynamic area.
//  */
// ULONG
// SepComputeAvailableDynamicSpace(
//     _In_ ULONG DynamicCharged,
//     _In_ PSID PrimaryGroup,
//     _In_opt_ PACL DefaultDacl)
// {
//     ULONG DynamicAvailable;
//
//     PAGED_CODE();
//
//     /* A token's dynamic area is always charged */
//     ASSERT(DynamicCharged != 0);
//
//     /*
//      * Take into account the default DACL if
//      * the token has one. Otherwise the occupied
//      * space is just the present primary group.
//      */
//     DynamicAvailable = DynamicCharged - RtlLengthSid(PrimaryGroup);
//     if (DefaultDacl)
//     {
//         DynamicAvailable -= DefaultDacl->AclSize;
//     }
//
//     return DynamicAvailable;
// }
//
// /**
//  * @brief
//  * Re-builds the dynamic part area of an access token
//  * during an a default DACL or primary group replacement
//  * within the said token if the said dynamic area can't
//  * hold the new security content.
//  *
//  * @param[in] AccessToken
//  * A pointer to an access token where its dynamic part
//  * is to be re-built and expanded based upon the new
//  * dynamic part size provided by the caller. Dynamic
//  * part expansion is not always guaranteed. See Remarks
//  * for further information.
//  *
//  * @param[in] NewDynamicPartSize
//  * The new dynamic part size.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the function has completed its
//  * operations successfully. STATUS_INSUFFICIENT_RESOURCES
//  * is returned if the new dynamic part could not be allocated.
//  *
//  * @remarks
//  * STATUS_SUCCESS does not indicate if the function has re-built
//  * the dynamic part of a token. If the current dynamic area size
//  * suffices the new dynamic area length provided by the caller
//  * then the dynamic area can hold the new security content buffer
//  * so dynamic part expansion is not necessary.
//  */
// NTSTATUS
// SepRebuildDynamicPartOfToken(
//     _Inout_ PTOKEN AccessToken,
//     _In_ ULONG NewDynamicPartSize)
// {
//     PVOID NewDynamicPart;
//     PVOID PreviousDynamicPart;
//     ULONG CurrentDynamicLength;
//
//     PAGED_CODE();
//
//     /* Sanity checks */
//     ASSERT(AccessToken);
//     ASSERT(NewDynamicPartSize != 0);
//
//     /*
//      * Compute the exact length of the available
//      * dynamic part of the access token.
//      */
//     CurrentDynamicLength = AccessToken->DynamicAvailable + RtlLengthSid(AccessToken->PrimaryGroup);
//     if (AccessToken->DefaultDacl)
//     {
//         CurrentDynamicLength += AccessToken->DefaultDacl->AclSize;
//     }
//
//     /*
//      * Figure out if the current dynamic part is too small
//      * to fit new contents inside the said dynamic part.
//      * Rebuild the dynamic area and expand it if necessary.
//      */
//     if (CurrentDynamicLength < NewDynamicPartSize)
//     {
//         NewDynamicPart = ExAllocatePoolWithTag(PagedPool,
//                                                NewDynamicPartSize,
//                                                TAG_TOKEN_DYNAMIC);
//         if (NewDynamicPart == NULL)
//         {
//             DPRINT1("SepRebuildDynamicPartOfToken(): Insufficient resources to allocate new dynamic part!\n");
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Copy the existing dynamic part */
//         PreviousDynamicPart = AccessToken->DynamicPart;
//         RtlCopyMemory(NewDynamicPart, PreviousDynamicPart, CurrentDynamicLength);
//
//         /* Update the available dynamic area and assign new dynamic */
//         AccessToken->DynamicAvailable += NewDynamicPartSize - CurrentDynamicLength;
//         AccessToken->DynamicPart = NewDynamicPart;
//
//         /* Move the contents (primary group and default DACL) addresses as well */
//         AccessToken->PrimaryGroup = (PSID)((ULONG_PTR)AccessToken->DynamicPart +
//                                     ((ULONG_PTR)AccessToken->PrimaryGroup - (ULONG_PTR)PreviousDynamicPart));
//         if (AccessToken->DefaultDacl != NULL)
//         {
//             AccessToken->DefaultDacl = (PACL)((ULONG_PTR)AccessToken->DynamicPart +
//                                        ((ULONG_PTR)AccessToken->DefaultDacl - (ULONG_PTR)PreviousDynamicPart));
//         }
//
//         /* And discard the previous dynamic part */
//         DPRINT("SepRebuildDynamicPartOfToken(): The dynamic part has been re-built with success!\n");
//         ExFreePoolWithTag(PreviousDynamicPart, TAG_TOKEN_DYNAMIC);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Frees (de-allocates) the proxy data memory block of a token.
//  *
//  * @param[in,out] ProxyData
//  * The proxy data to be freed.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SepFreeProxyData(
//     _Inout_ PVOID ProxyData)
// {
//     UNIMPLEMENTED;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Copies the proxy data from the source into the destination of a token.
//  *
//  * @param[out] Dest
//  * The destination path where the proxy data is to be copied to.
//  *
//  * @param[in] Src
//  * The source path where the proxy data is be copied from.
//  *
//  * @return
//  * To be added...
//  */
// NTSTATUS
// NTAPI
// SepCopyProxyData(
//     _Out_ PVOID* Dest,
//     _In_ PVOID Src)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /**
//  * @brief
//  * Replaces the old access token of a process (pointed by the EPROCESS kernel structure) with a
//  * new access token. The new access token must be a primary token for use.
//  *
//  * @param[in] Process
//  * The process instance where its access token is about to be replaced.
//  *
//  * @param[in] NewAccessToken
//  * The new token that it's going to replace the old one.
//  *
//  * @param[out] OldAccessToken
//  * The returned old token that's been replaced, which the caller can do anything.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the exchange operation between tokens has completed successfully.
//  * STATUS_BAD_TOKEN_TYPE is returned if the new token is not a primary one so that we cannot
//  * exchange it with the old one from the process. STATUS_TOKEN_ALREADY_IN_USE is returned if
//  * both tokens aren't equal which means one of them has different properties (groups, privileges, etc.)
//  * and as such one of them is currently in use. A failure NTSTATUS code is returned otherwise.
//  */
// NTSTATUS
// NTAPI
// SeExchangePrimaryToken(
//     _In_ PEPROCESS Process,
//     _In_ PACCESS_TOKEN NewAccessToken,
//     _Out_ PACCESS_TOKEN* OldAccessToken)
// {
//     PTOKEN OldToken;
//     PTOKEN NewToken = (PTOKEN)NewAccessToken;
//
//     PAGED_CODE();
//
//     if (NewToken->TokenType != TokenPrimary)
//         return STATUS_BAD_TOKEN_TYPE;
//
//     if (NewToken->TokenInUse)
//     {
//         BOOLEAN IsEqual;
//         NTSTATUS Status;
//
//         /* Maybe we're trying to set the same token */
//         OldToken = PsReferencePrimaryToken(Process);
//         if (OldToken == NewToken)
//         {
//             /* So it's a nop. */
//             *OldAccessToken = OldToken;
//             return STATUS_SUCCESS;
//         }
//
//         Status = SepCompareTokens(OldToken, NewToken, &IsEqual);
//         if (!NT_SUCCESS(Status))
//         {
//             PsDereferencePrimaryToken(OldToken);
//             *OldAccessToken = NULL;
//             return Status;
//         }
//
//         if (!IsEqual)
//         {
//             PsDereferencePrimaryToken(OldToken);
//             *OldAccessToken = NULL;
//             return STATUS_TOKEN_ALREADY_IN_USE;
//         }
//         /* Silently return STATUS_SUCCESS but do not set the new token,
//          * as it's already in use elsewhere. */
//         *OldAccessToken = OldToken;
//         return STATUS_SUCCESS;
//     }
//
//     /* Lock the new token */
//     SepAcquireTokenLockExclusive(NewToken);
//
//     /* Mark new token in use */
//     NewToken->TokenInUse = TRUE;
//
//     /* Set the session ID for the new token */
//     NewToken->SessionId = MmGetSessionId(Process);
//
//     /* Unlock the new token */
//     SepReleaseTokenLock(NewToken);
//
//     /* Reference the new token */
//     ObReferenceObject(NewToken);
//
//     /* Replace the old with the new */
//     OldToken = ObFastReplaceObject(&Process->Token, NewToken);
//
//     /* Lock the old token */
//     SepAcquireTokenLockExclusive(OldToken);
//
//     /* Mark the old token as free */
//     OldToken->TokenInUse = FALSE;
//
//     /* Unlock the old token */
//     SepReleaseTokenLock(OldToken);
//
//     *OldAccessToken = (PACCESS_TOKEN)OldToken;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Removes the primary token of a process.
//  *
//  * @param[in,out] Process
//  * The process instance with the access token to be removed.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeDeassignPrimaryToken(
//     _Inout_ PEPROCESS Process)
// {
//     PTOKEN OldToken;
//
//     /* Remove the Token */
//     OldToken = ObFastReplaceObject(&Process->Token, NULL);
//
//     /* Mark the Old Token as free */
//     OldToken->TokenInUse = FALSE;
//
//     /* Dereference the Token */
//     ObDereferenceObject(OldToken);
// }
//
// /**
//  * @brief
//  * Computes the length size of a SID.
//  *
//  * @param[in] Count
//  * Total count of entries that have SIDs in them (that being PSID_AND_ATTRIBUTES in this context).
//  *
//  * @param[in] Src
//  * Source that points to the attributes and SID entry structure.
//  *
//  * @return
//  * Returns the total length of a SID size.
//  */
// ULONG
// RtlLengthSidAndAttributes(
//     _In_ ULONG Count,
//     _In_ PSID_AND_ATTRIBUTES Src)
// {
//     ULONG i;
//     ULONG uLength;
//
//     PAGED_CODE();
//
//     uLength = Count * sizeof(SID_AND_ATTRIBUTES);
//     for (i = 0; i < Count; i++)
//         uLength += RtlLengthSid(Src[i].Sid);
//
//     return uLength;
// }
//
// /**
//  * @brief
//  * Finds the primary group and default owner entity based on the submitted primary group instance
//  * and an access token.
//  *
//  * @param[in] Token
//  * Access token to begin the search query of primary group and default owner.
//  *
//  * @param[in] PrimaryGroup
//  * A primary group SID to be used for search query, determining if user & groups of a token
//  * and the submitted primary group do match.
//  *
//  * @param[in] DefaultOwner
//  * The default owner. If specified, it's used to determine if the token belongs to the actual user,
//  * that is, being the owner himself.
//  *
//  * @param[out] PrimaryGroupIndex
//  * Returns the primary group index.
//  *
//  * @param[out] DefaultOwnerIndex
//  * Returns the default owner index.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the find query operation has completed successfully and that at least one
//  * search result is requested by the caller. STATUS_INVALID_PARAMETER is returned if the caller hasn't requested
//  * any search result. STATUS_INVALID_OWNER is returned if the specified default user owner does not match with the other
//  * user from the token. STATUS_INVALID_PRIMARY_GROUP is returned if the specified default primary group does not match with the
//  * other group from the token.
//  */
// NTSTATUS
// SepFindPrimaryGroupAndDefaultOwner(
//     _In_ PTOKEN Token,
//     _In_ PSID PrimaryGroup,
//     _In_opt_ PSID DefaultOwner,
//     _Out_opt_ PULONG PrimaryGroupIndex,
//     _Out_opt_ PULONG DefaultOwnerIndex)
// {
//     ULONG i;
//
//     /* We should return at least a search result */
//     if (!PrimaryGroupIndex && !DefaultOwnerIndex)
//         return STATUS_INVALID_PARAMETER;
//
//     if (PrimaryGroupIndex)
//     {
//         /* Initialize with an invalid index */
//         // Token->PrimaryGroup = NULL;
//         *PrimaryGroupIndex = Token->UserAndGroupCount;
//     }
//
//     if (DefaultOwnerIndex)
//     {
//         if (DefaultOwner)
//         {
//             /* An owner is specified: check whether this is actually the user */
//             if (RtlEqualSid(Token->UserAndGroups[0].Sid, DefaultOwner))
//             {
//                 /*
//                  * It's the user (first element in array): set it
//                  * as the owner and stop the search for it.
//                  */
//                 *DefaultOwnerIndex = 0;
//                 DefaultOwnerIndex = NULL;
//             }
//             else
//             {
//                 /* An owner is specified: initialize with an invalid index */
//                 *DefaultOwnerIndex = Token->UserAndGroupCount;
//             }
//         }
//         else
//         {
//             /*
//              * No owner specified: set the user (first element in array)
//              * as the owner and stop the search for it.
//              */
//             *DefaultOwnerIndex = 0;
//             DefaultOwnerIndex = NULL;
//         }
//     }
//
//     /* Validate and set the primary group and default owner indices */
//     for (i = 0; i < Token->UserAndGroupCount; i++)
//     {
//         /* Stop the search if we have found what we searched for */
//         if (!PrimaryGroupIndex && !DefaultOwnerIndex)
//             break;
//
//         if (DefaultOwnerIndex && DefaultOwner &&
//             RtlEqualSid(Token->UserAndGroups[i].Sid, DefaultOwner) &&
//             (Token->UserAndGroups[i].Attributes & SE_GROUP_OWNER))
//         {
//             /* Owner is found, stop the search for it */
//             *DefaultOwnerIndex = i;
//             DefaultOwnerIndex = NULL;
//         }
//
//         if (PrimaryGroupIndex &&
//             RtlEqualSid(Token->UserAndGroups[i].Sid, PrimaryGroup))
//         {
//             /* Primary group is found, stop the search for it */
//             // Token->PrimaryGroup = Token->UserAndGroups[i].Sid;
//             *PrimaryGroupIndex = i;
//             PrimaryGroupIndex = NULL;
//         }
//     }
//
//     if (DefaultOwnerIndex)
//     {
//         if (*DefaultOwnerIndex == Token->UserAndGroupCount)
//             return STATUS_INVALID_OWNER;
//     }
//
//     if (PrimaryGroupIndex)
//     {
//         if (*PrimaryGroupIndex == Token->UserAndGroupCount)
//         // if (Token->PrimaryGroup == NULL)
//             return STATUS_INVALID_PRIMARY_GROUP;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Internal private function that returns an opened handle
//  * of an access token associated with a thread.
//  *
//  * @param[in] Thread
//  * A pointer to a Executive thread. This parameter is used to
//  * validate that the newly obtained thread in this function
//  * hasn't diverged. This could potentially lead to a scenario
//  * that we might get an access token from a different token
//  * which is not what we want. The validation is performed
//  * if the token has to copied and can't be opened directly.
//  *
//  * @param[in] ThreadHandle
//  * A handle to a thread, of which an access token is to be opened
//  * and given from that thread.
//  *
//  * @param[in] ThreadToken
//  * A pointer to an access token associated with the specific thread.
//  * The function assumes that the token is an impersonation one
//  * prior the calling of this function.
//  *
//  * @param[in] DesiredAccess
//  * The desired access rights for the access token.
//  *
//  * @param[in] HandleAttributes
//  * Handle attributes of which they are used for the newly creation
//  * of the opened thread token. The function assumes that they have
//  * been validated prior the calling of this function.
//  *
//  * @param[in] EffectiveOnly
//  * If set to TRUE, the function will copy a new access token with
//  * privileges and groups that are effectively enabled. Any disabled
//  * privilege or group is removed from the copied token. Otherwise
//  * if set to FALSE, the function retains all the enabled and disabled
//  * privielges and groups.
//  *
//  * @param[in] CopyOnOpen
//  * If set to TRUE, it tells the function that the access token cannot
//  * be directly opened due to the security impersonation info of the
//  * associated thread being enforced. In this case the function will
//  * make a copy of the said token by duplicating it. Otherwise if set
//  * to FALSE, the function will just open the access token directly.
//  *
//  * @param[in] ImpersonationLevel
//  * The security impersonation level, at which it is allowed to
//  * access the token.
//  *
//  * @param[in] PreviousMode
//  * The processor request level mode.
//  *
//  * @param[out] OpenedTokenHandle
//  * A pointer to an opened access token handle associated with the
//  * specific thread, returned to the caller. Initially this parameter
//  * is set to NULL and if the function fails to open the thread's token,
//  * it will stay NULL.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the function has successfully opened the thread's
//  * token. STATUS_OBJECT_TYPE_MISMATCH is returned if the obtained thread object
//  * no longer matches with the other thread that has been obtained previously.
//  * STATUS_NO_TOKEN is returned if the associated thread's process has no
//  * primary access token. A failure NTSTATUS code is returned otherwise.
//  */
// static
// NTSTATUS
// SepOpenThreadToken(
//     _In_ PETHREAD Thread,
//     _In_ HANDLE ThreadHandle,
//     _In_ PTOKEN ThreadToken,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ ULONG HandleAttributes,
//     _In_ BOOLEAN EffectiveOnly,
//     _In_ BOOLEAN CopyOnOpen,
//     _In_ SECURITY_IMPERSONATION_LEVEL ImpersonationLevel,
//     _In_ KPROCESSOR_MODE PreviousMode,
//     _Out_ PHANDLE OpenedTokenHandle)
// {
//     NTSTATUS Status;
//     HANDLE TokenHandle;
//     PETHREAD Thread2;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PTOKEN NewToken, PrimaryToken;
//     SECURITY_DESCRIPTOR SecurityDescriptor;
//     PACL Dacl;
//
//     PAGED_CODE();
//
//     /* Assume no opened token handle at first */
//     *OpenedTokenHandle = NULL;
//
//     /* Check if we have to do a copy of the token on open or not */
//     if (!CopyOnOpen)
//     {
//         /* Just open the thread's token directly */
//         Status = ObOpenObjectByPointer(ThreadToken,
//                                        HandleAttributes,
//                                        NULL,
//                                        DesiredAccess,
//                                        SeTokenObjectType as *mut _OBJECT_TYPE,
//                                        PreviousMode,
//                                        &TokenHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to open the thread's token object (Status 0x%lx)\n", Status);
//             return Status;
//         }
//
//         /* Give it to caller */
//         *OpenedTokenHandle = TokenHandle;
//         return STATUS_SUCCESS;
//     }
//
//     /*
//      * The caller asks to do a copy of that token whilst it's opened.
//      * Obtain a thread object again but this time we have to obtain
//      * it in our side, kernel mode, and request all the access needed
//      * to do a copy of the token because the original thread only has
//      * query access needed for access token validation.
//      */
//     Status = ObReferenceObjectByHandle(ThreadHandle,
//                                        THREAD_ALL_ACCESS,
//                                        PsThreadType,
//                                        KernelMode,
//                                        (PVOID*)&Thread2,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to reference the object thread (Status 0x%lx)\n", Status);
//         return Status;
//     }
//
//     /* Check that one of the threads hasn't diverged */
//     if (Thread != Thread2)
//     {
//         DPRINT1("One of the threads aren't the same (original thread 0x%p, thread 0x%p)\n", Thread, Thread2);
//         ObDereferenceObject(Thread2);
//         return STATUS_OBJECT_TYPE_MISMATCH;
//     }
//
//     /* Reference the primary token of the process' thread */
//     PrimaryToken = PsReferencePrimaryToken(Thread2->ThreadsProcess);
//     if (!PrimaryToken)
//     {
//         DPRINT1("Failed to reference the primary token of thread\n");
//         ObDereferenceObject(Thread2);
//         return STATUS_NO_TOKEN;
//     }
//
//     /* Create an impersonation DACL from the tokens we got */
//     Status = SepCreateImpersonationTokenDacl(ThreadToken, PrimaryToken, &Dacl);
//     ObFastDereferenceObject(&Thread2->ThreadsProcess->Token, PrimaryToken);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create an impersonation token DACL (Status 0x%lx)\n", Status);
//         ObDereferenceObject(Thread2);
//         return Status;
//     }
//
//     /* Create a security descriptor with the DACL we got */
//     Status = RtlCreateSecurityDescriptor(&SecurityDescriptor,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create a security descriptor (Status 0x%lx)\n", Status);
//         ExFreePoolWithTag(Dacl, TAG_ACL);
//         ObDereferenceObject(Thread2);
//         return Status;
//     }
//
//     /* Attach the DACL to that security descriptor */
//     Status = RtlSetDaclSecurityDescriptor(&SecurityDescriptor,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to set the DACL to the security descriptor (Status 0x%lx)\n", Status);
//         ExFreePoolWithTag(Dacl, TAG_ACL);
//         ObDereferenceObject(Thread2);
//         return Status;
//     }
//
//     /*
//      * Initialize the object attributes for the token we
//      * are going to duplicate.
//      */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                HandleAttributes,
//                                NULL,
//                                &SecurityDescriptor);
//
//     /* Duplicate (copy) it now */
//     Status = SepDuplicateToken(ThreadToken,
//                                &ObjectAttributes,
//                                EffectiveOnly,
//                                TokenImpersonation,
//                                ImpersonationLevel,
//                                KernelMode,
//                                &NewToken);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to duplicate the token (Status 0x%lx)\n", Status);
//         ExFreePoolWithTag(Dacl, TAG_ACL);
//         ObDereferenceObject(Thread2);
//         return Status;
//     }
//
//     /* Insert that copied token into the handle now */
//     ObReferenceObject(NewToken);
//     Status = ObInsertObject(NewToken,
//                             NULL,
//                             DesiredAccess,
//                             0,
//                             NULL,
//                             &TokenHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to insert the token object (Status 0x%lx)\n", Status);
//         ExFreePoolWithTag(Dacl, TAG_ACL);
//         ObDereferenceObject(NewToken);
//         ObDereferenceObject(Thread2);
//         return Status;
//     }
//
//     /* We're almost done, free the DACL if we got one */
//     ExFreePoolWithTag(Dacl, TAG_ACL);
//
//     /* Impersonate the client finally */
//     Status = PsImpersonateClient(Thread, NewToken, FALSE, EffectiveOnly, ImpersonationLevel);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to impersonate the client (Status 0x%lx)\n", Status);
//         ObCloseHandle(TokenHandle, PreviousMode);
//         ObDereferenceObject(NewToken);
//         ObDereferenceObject(Thread2);
//         return Status;
//     }
//
//     /* Give the newly opened token handle to caller */
//     *OpenedTokenHandle = TokenHandle;
//     ObDereferenceObject(NewToken);
//     ObDereferenceObject(Thread2);
//     return Status;
// }
//
// /**
//  * @brief
//  * Subtracts a token in exchange of duplicating a new one.
//  *
//  * @param[in] ParentToken
//  * The parent access token for duplication.
//  *
//  * @param[out] Token
//  * The new duplicated token.
//  *
//  * @param[in] InUse
//  * Set this to TRUE if the token is about to be used immediately after the call execution
//  * of this function, FALSE otherwise.
//  *
//  * @param[in] SessionId
//  * Session ID for the token to be assigned.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if token subtracting and duplication have completed successfully.
//  * A failure NTSTATUS code is returned otherwise.
//  */
// NTSTATUS
// NTAPI
// SeSubProcessToken(
//     _In_ PTOKEN ParentToken,
//     _Out_ PTOKEN *Token,
//     _In_ BOOLEAN InUse,
//     _In_ ULONG SessionId)
// {
//     PTOKEN NewToken;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//
//     /* Initialize the attributes and duplicate it */
//     InitializeObjectAttributes(&ObjectAttributes, NULL, 0, NULL, NULL);
//     Status = SepDuplicateToken(ParentToken,
//                                &ObjectAttributes,
//                                FALSE,
//                                TokenPrimary,
//                                ParentToken->ImpersonationLevel,
//                                KernelMode,
//                                &NewToken);
//     if (NT_SUCCESS(Status))
//     {
//         /* Insert it */
//         Status = ObInsertObject(NewToken,
//                                 NULL,
//                                 0,
//                                 0,
//                                 NULL,
//                                 NULL);
//         if (NT_SUCCESS(Status))
//         {
//             /* Set the session ID */
//             NewToken->SessionId = SessionId;
//             NewToken->TokenInUse = InUse;
//
//             /* Return the token */
//             *Token = NewToken;
//         }
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /**
//  * @brief
//  * Checks if the token is a child of the other token
//  * of the current process that the calling thread is invoking this function.
//  *
//  * @param[in] Token
//  * An access token to determine if it's a child or not.
//  *
//  * @param[out] IsChild
//  * The returned boolean result.
//  *
//  * @return
//  * Returns STATUS_SUCCESS when the function finishes its operation. STATUS_UNSUCCESSFUL is
//  * returned if primary token of the current calling process couldn't be referenced otherwise.
//  */
// NTSTATUS
// NTAPI
// SeIsTokenChild(
//     _In_ PTOKEN Token,
//     _Out_ PBOOLEAN IsChild)
// {
//     PTOKEN ProcessToken;
//     LUID ProcessTokenId, CallerParentId;
//
//     /* Assume failure */
//     *IsChild = FALSE;
//
//     /* Reference the process token */
//     ProcessToken = PsReferencePrimaryToken(PsGetCurrentProcess());
//     if (!ProcessToken)
//         return STATUS_UNSUCCESSFUL;
//
//     /* Get its token ID */
//     ProcessTokenId = ProcessToken->TokenId;
//
//     /* Dereference the token */
//     ObFastDereferenceObject(&PsGetCurrentProcess()->Token, ProcessToken);
//
//     /* Get our parent token ID */
//     CallerParentId = Token->ParentTokenId;
//
//     /* Compare the token IDs */
//     if (RtlEqualLuid(&CallerParentId, &ProcessTokenId))
//         *IsChild = TRUE;
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Checks if the token is a sibling of the other token of
//  * the current process that the calling thread is invoking this function.
//  *
//  * @param[in] Token
//  * An access token to determine if it's a sibling or not.
//  *
//  * @param[out] IsSibling
//  * The returned boolean result.
//  *
//  * @return
//  * Returns STATUS_SUCCESS when the function finishes its operation. STATUS_UNSUCCESSFUL is
//  * returned if primary token of the current calling process couldn't be referenced otherwise.
//  */
// NTSTATUS
// NTAPI
// SeIsTokenSibling(
//     _In_ PTOKEN Token,
//     _Out_ PBOOLEAN IsSibling)
// {
//     PTOKEN ProcessToken;
//     LUID ProcessParentId, ProcessAuthId;
//     LUID CallerParentId, CallerAuthId;
//
//     /* Assume failure */
//     *IsSibling = FALSE;
//
//     /* Reference the process token */
//     ProcessToken = PsReferencePrimaryToken(PsGetCurrentProcess());
//     if (!ProcessToken)
//         return STATUS_UNSUCCESSFUL;
//
//     /* Get its parent and authentication IDs */
//     ProcessParentId = ProcessToken->ParentTokenId;
//     ProcessAuthId = ProcessToken->AuthenticationId;
//
//     /* Dereference the token */
//     ObFastDereferenceObject(&PsGetCurrentProcess()->Token, ProcessToken);
//
//     /* Get our parent and authentication IDs */
//     CallerParentId = Token->ParentTokenId;
//     CallerAuthId = Token->AuthenticationId;
//
//     /* Compare the token IDs */
//     if (RtlEqualLuid(&CallerParentId, &ProcessParentId) &&
//         RtlEqualLuid(&CallerAuthId, &ProcessAuthId))
//     {
//         *IsSibling = TRUE;
//     }
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Copies an existing access token (technically duplicating a new one).
//  *
//  * @param[in] Token
//  * Token to copy.
//  *
//  * @param[in] Level
//  * Impersonation security level to assign to the newly copied token.
//  *
//  * @param[in] PreviousMode
//  * Processor request level mode.
//  *
//  * @param[out] NewToken
//  * The newly copied token.
//  *
//  * @return
//  * Returns STATUS_SUCCESS when token copying has finished successfully. A failure
//  * NTSTATUS code is returned otherwise.
//  */
// NTSTATUS
// NTAPI
// SeCopyClientToken(
//     _In_ PACCESS_TOKEN Token,
//     _In_ SECURITY_IMPERSONATION_LEVEL Level,
//     _In_ KPROCESSOR_MODE PreviousMode,
//     _Out_ PACCESS_TOKEN* NewToken)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//
//     PAGED_CODE();
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                0,
//                                NULL,
//                                NULL);
//
//     Status = SepDuplicateToken(Token,
//                                &ObjectAttributes,
//                                FALSE,
//                                TokenImpersonation,
//                                Level,
//                                PreviousMode,
//                                (PTOKEN*)NewToken);
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Determines if a token is a sandbox inert token or not,
//  * based upon the token flags.
//  *
//  * @param[in] Token
//  * A valid access token to determine if such token is inert.
//  *
//  * @return
//  * Returns TRUE if the token is inert, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SeTokenIsInert(
//     _In_ PTOKEN Token)
// {
//     PAGED_CODE();
//
//     return (((PTOKEN)Token)->TokenFlags & TOKEN_SANDBOX_INERT) != 0;
// }
//
// /**
//  * @brief
//  * Internal function that deals with access token object destruction and deletion.
//  * The function is used solely by the object manager mechanism that handles the life
//  * management of a token object.
//  *
//  * @param[in] ObjectBody
//  * The object body that represents an access token object.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SepDeleteToken(
//     _In_ PVOID ObjectBody)
// {
//     NTSTATUS Status;
//     PTOKEN AccessToken = (PTOKEN)ObjectBody;
//
//     DPRINT("SepDeleteToken()\n");
//
//     /* Remove the referenced logon session from token */
//     if (AccessToken->LogonSession)
//     {
//         Status = SepRmRemoveLogonSessionFromToken(AccessToken);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Something seriously went wrong */
//             DPRINT1("SepDeleteToken(): Failed to remove the logon session from token (Status: 0x%lx)\n", Status);
//             return;
//         }
//     }
//
//     /* Dereference the logon session */
//     if ((AccessToken->TokenFlags & TOKEN_SESSION_NOT_REFERENCED) == 0)
//         SepRmDereferenceLogonSession(&AccessToken->AuthenticationId);
//
//     /* Delete the token lock */
//     if (AccessToken->TokenLock)
//         SepDeleteTokenLock(AccessToken);
//
//     /* Delete the dynamic information area */
//     if (AccessToken->DynamicPart)
//         ExFreePoolWithTag(AccessToken->DynamicPart, TAG_TOKEN_DYNAMIC);
// }
//
// /**
//  * @brief
//  * Internal function that initializes critical kernel data for access
//  * token implementation in SRM.
//  *
//  * @return
//  * Nothing.
//  */
// CODE_SEG("INIT")
// VOID
// NTAPI
// SepInitializeTokenImplementation(VOID)
// {
//     UNICODE_STRING Name;
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//
//     DPRINT("Creating Token Object Type\n");
//
//     /* Initialize the Token type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     RtlInitUnicodeString(&Name, L"Token");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     ObjectTypeInitializer.SecurityRequired = TRUE;
//     ObjectTypeInitializer.DefaultPagedPoolCharge = sizeof(TOKEN);
//     ObjectTypeInitializer.GenericMapping = SepTokenMapping;
//     ObjectTypeInitializer.PoolType = PagedPool;
//     ObjectTypeInitializer.ValidAccessMask = TOKEN_ALL_ACCESS;
//     ObjectTypeInitializer.UseDefaultObject = TRUE;
//     ObjectTypeInitializer.DeleteProcedure = SepDeleteToken;
//     {
//         NTSTATUS _st;
//         _TOKI_TRACE("[TOKI] before ObCreateObjectType(Token)\n");
//         _TOKI_HEX("[TOKI] OTI.Length=", ObjectTypeInitializer.Length);
//         _TOKI_HEX("[TOKI] OTI.PoolType=", ObjectTypeInitializer.PoolType);
//         _TOKI_HEX("[TOKI] OTI.UseDefaultObject=", ObjectTypeInitializer.UseDefaultObject);
//         _st = ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &SeTokenObjectType as *mut _OBJECT_TYPE);
//         _TOKI_HEX("[TOKI] ObCreateObjectType returned=", _st);
//         _TOKI_HEX("[TOKI] SeTokenObjectType as *mut _OBJECT_TYPE=", SeTokenObjectType as *mut _OBJECT_TYPE);
//     }
// }
//
// /**
//  * @brief
//  * Assigns a primary access token to a given process.
//  *
//  * @param[in] Process
//  * Process where the token is about to be assigned.
//  *
//  * @param[in] Token
//  * The token to be assigned.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeAssignPrimaryToken(
//     _In_ PEPROCESS Process,
//     _In_ PTOKEN Token)
// {
//     PAGED_CODE();
//
//     /* Sanity checks */
//     ASSERT(Token->TokenType == TokenPrimary);
//     ASSERT(!Token->TokenInUse);
//
//     /* Clean any previous token */
//     if (Process->Token.Object) SeDeassignPrimaryToken(Process);
//
//     /* Set the new token */
//     ObReferenceObject(Token);
//     Token->TokenInUse = TRUE;
//     ObInitializeFastReference(&Process->Token, Token);
// }
//
// /**
//  * @brief
//  * Retrieves token control information.
//  *
//  * @param[in] _Token
//  * A valid token object.
//  *
//  * @param[out] SecurityDescriptor
//  * The returned token control information.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeGetTokenControlInformation(
//     _In_ PACCESS_TOKEN _Token,
//     _Out_ PTOKEN_CONTROL TokenControl)
// {
//     PTOKEN Token = _Token;
//     PAGED_CODE();
//
//     /* Capture the main fields */
//     TokenControl->AuthenticationId = Token->AuthenticationId;
//     TokenControl->TokenId = Token->TokenId;
//     TokenControl->TokenSource = Token->TokenSource;
//
//     /* Lock the token */
//     SepAcquireTokenLockShared(Token);
//
//     /* Capture the modified ID */
//     TokenControl->ModifiedId = Token->ModifiedId;
//
//     /* Unlock it */
//     SepReleaseTokenLock(Token);
// }
//
// /**
//  * @brief
//  * Creates the system process token.
//  *
//  * @return
//  * Returns the system process token if the operations have
//  * completed successfully.
//  */
// CODE_SEG("INIT")
// PTOKEN
// NTAPI
// SepCreateSystemProcessToken(VOID)
// {
//     ULONG GroupAttributes, OwnerAttributes;
//     LARGE_INTEGER Expiration;
//     SID_AND_ATTRIBUTES UserSid;
//     ULONG GroupsLength;
//     PSID PrimaryGroup;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PSID Owner;
//     PTOKEN Token;
//     NTSTATUS Status;
//
//     /* Don't ever expire */
//     Expiration.QuadPart = -1;
//
//     /* All groups mandatory and enabled */
//     GroupAttributes = SE_GROUP_ENABLED | SE_GROUP_MANDATORY | SE_GROUP_ENABLED_BY_DEFAULT;
//     OwnerAttributes = SE_GROUP_ENABLED | SE_GROUP_OWNER | SE_GROUP_ENABLED_BY_DEFAULT;
//
//     /* User is Local System */
//     UserSid.Sid = SeLocalSystemSid;
//     UserSid.Attributes = 0;
//
//     /* Primary group is Local System */
//     PrimaryGroup = SeLocalSystemSid;
//
//     /* Owner is Administrators */
//     Owner = SeAliasAdminsSid;
//
//     /* Groups are Administrators, World, and Authenticated Users */
//     SID_AND_ATTRIBUTES Groups[] =
//     {
//         {SeAliasAdminsSid, OwnerAttributes},
//         {SeWorldSid, GroupAttributes},
//         {SeAuthenticatedUsersSid, GroupAttributes},
//         {SeLocalSid, SE_GROUP_ENABLED} // HACK: Temporarily add the local group. See CORE-18250.
//     };
//     GroupsLength = sizeof(SID_AND_ATTRIBUTES) +
//                    SeLengthSid(Groups[0].Sid) +
//                    SeLengthSid(Groups[1].Sid) +
//                    SeLengthSid(Groups[2].Sid) +
//                    SeLengthSid(Groups[3].Sid); // HACK
//     ASSERT(GroupsLength <= (sizeof(Groups) * sizeof(ULONG)));
//
//     /* Setup the privileges */
//     LUID_AND_ATTRIBUTES Privileges[] =
//     {
//         {SeTcbPrivilege, SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED},
//         {SeCreateTokenPrivilege, 0},
//         {SeTakeOwnershipPrivilege, 0},
//         {SeCreatePagefilePrivilege, SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED},
//         {SeLockMemoryPrivilege, SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED},
//         {SeAssignPrimaryTokenPrivilege, 0},
//         {SeIncreaseQuotaPrivilege, 0},
//         {SeIncreaseBasePriorityPrivilege, SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED},
//         {SeCreatePermanentPrivilege, SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED},
//         {SeDebugPrivilege, SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED},
//         {SeAuditPrivilege, SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED},
//         {SeSecurityPrivilege, 0},
//         {SeSystemEnvironmentPrivilege, 0},
//         {SeChangeNotifyPrivilege, SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED},
//         {SeBackupPrivilege, 0},
//         {SeRestorePrivilege, 0},
//         {SeShutdownPrivilege, 0},
//         {SeLoadDriverPrivilege, 0},
//         {SeProfileSingleProcessPrivilege, SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED},
//         {SeSystemtimePrivilege, 0},
//         {SeUndockPrivilege, 0},
//         {SeManageVolumePrivilege, 0},
//         {SeImpersonatePrivilege, SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED},
//         {SeCreateGlobalPrivilege, SE_PRIVILEGE_ENABLED_BY_DEFAULT | SE_PRIVILEGE_ENABLED},
//     };
//
//     /* Setup the object attributes */
//     InitializeObjectAttributes(&ObjectAttributes, NULL, 0, NULL, NULL);
//     ASSERT(SeSystemDefaultDacl != NULL);
//
//     /* Create the token */
//     Status = SepCreateToken((PHANDLE)&Token,
//                             KernelMode,
//                             0,
//                             &ObjectAttributes,
//                             TokenPrimary,
//                             SecurityAnonymous,
//                             &SeSystemAuthenticationId,
//                             &Expiration,
//                             &UserSid,
//                             RTL_NUMBER_OF(Groups),
//                             Groups,
//                             GroupsLength,
//                             RTL_NUMBER_OF(Privileges),
//                             Privileges,
//                             Owner,
//                             PrimaryGroup,
//                             SeSystemDefaultDacl,
//                             &SeSystemTokenSource,
//                             TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//
//     /* Return the token */
//     return Token;
// }
//
// /**
//  * @brief
//  * Creates the anonymous logon token for the system. The difference between this
//  * token and the other one is the inclusion of everyone SID group (being SeWorldSid).
//  * The other token lacks such group.
//  *
//  * @return
//  * Returns the system's anonymous logon token if the operations have
//  * completed successfully.
//  */
// CODE_SEG("INIT")
// PTOKEN
// SepCreateSystemAnonymousLogonToken(VOID)
// {
//     SID_AND_ATTRIBUTES UserSid;
//     PSID PrimaryGroup;
//     PTOKEN Token;
//     ULONG GroupsLength;
//     LARGE_INTEGER Expiration;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//
//     /* The token never expires */
//     Expiration.QuadPart = -1;
//
//     /* The user is the anonymous logon */
//     UserSid.Sid = SeAnonymousLogonSid;
//     UserSid.Attributes = 0;
//
//     /* The primary group is also the anonymous logon */
//     PrimaryGroup = SeAnonymousLogonSid;
//
//     /* The only group for the token is the World */
//     SID_AND_ATTRIBUTES Groups[] =
//     {
//         {SeWorldSid, SE_GROUP_ENABLED | SE_GROUP_MANDATORY | SE_GROUP_ENABLED_BY_DEFAULT}
//     };
//     GroupsLength = sizeof(SID_AND_ATTRIBUTES) +
//                    SeLengthSid(Groups[0].Sid);
//     ASSERT(GroupsLength <= (sizeof(Groups) * sizeof(ULONG)));
//
//     /* Initialise the object attributes for the token */
//     InitializeObjectAttributes(&ObjectAttributes, NULL, 0, NULL, NULL);
//     ASSERT(SeSystemAnonymousLogonDacl != NULL);
//
//     /* Create token */
//     Status = SepCreateToken((PHANDLE)&Token,
//                             KernelMode,
//                             0,
//                             &ObjectAttributes,
//                             TokenPrimary,
//                             SecurityAnonymous,
//                             &SeAnonymousAuthenticationId,
//                             &Expiration,
//                             &UserSid,
//                             RTL_NUMBER_OF(Groups),
//                             Groups,
//                             GroupsLength,
//                             0,
//                             NULL,
//                             NULL,
//                             PrimaryGroup,
//                             SeSystemAnonymousLogonDacl,
//                             &SeSystemTokenSource,
//                             TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//
//     /* Return the anonymous logon token */
//     return Token;
// }
//
// /**
//  * @brief
//  * Creates the anonymous logon token for the system. This kind of token
//  * doesn't include the everyone SID group (being SeWorldSid).
//  *
//  * @return
//  * Returns the system's anonymous logon token if the operations have
//  * completed successfully.
//  */
// CODE_SEG("INIT")
// PTOKEN
// SepCreateSystemAnonymousLogonTokenNoEveryone(VOID)
// {
//     SID_AND_ATTRIBUTES UserSid;
//     PSID PrimaryGroup;
//     PTOKEN Token;
//     LARGE_INTEGER Expiration;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//
//     /* The token never expires */
//     Expiration.QuadPart = -1;
//
//     /* The user is the anonymous logon */
//     UserSid.Sid = SeAnonymousLogonSid;
//     UserSid.Attributes = 0;
//
//     /* The primary group is also the anonymous logon */
//     PrimaryGroup = SeAnonymousLogonSid;
//
//     /* Initialise the object attributes for the token */
//     InitializeObjectAttributes(&ObjectAttributes, NULL, 0, NULL, NULL);
//     ASSERT(SeSystemAnonymousLogonDacl != NULL);
//
//     /* Create token */
//     Status = SepCreateToken((PHANDLE)&Token,
//                             KernelMode,
//                             0,
//                             &ObjectAttributes,
//                             TokenPrimary,
//                             SecurityAnonymous,
//                             &SeAnonymousAuthenticationId,
//                             &Expiration,
//                             &UserSid,
//                             0,
//                             NULL,
//                             0,
//                             0,
//                             NULL,
//                             NULL,
//                             PrimaryGroup,
//                             SeSystemAnonymousLogonDacl,
//                             &SeSystemTokenSource,
//                             TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//
//     /* Return the anonymous (not including everyone) logon token */
//     return Token;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /**
//  * @brief
//  * Queries the session ID of an access token.
//  *
//  * @param[in] Token
//  * A valid access token where the session ID has to be gathered.
//  *
//  * @param[out] pSessionId
//  * The returned pointer to a session ID to the caller.
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  */
// NTSTATUS
// NTAPI
// SeQuerySessionIdToken(
//     _In_ PACCESS_TOKEN Token,
//     _Out_ PULONG pSessionId)
// {
//     PAGED_CODE();
//
//     /* Lock the token */
//     SepAcquireTokenLockShared(Token);
//
//     *pSessionId = ((PTOKEN)Token)->SessionId;
//
//     /* Unlock the token */
//     SepReleaseTokenLock(Token);
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Queries the authentication ID of an access token.
//  *
//  * @param[in] Token
//  * A valid access token where the authentication ID has to be gathered.
//  *
//  * @param[out] pSessionId
//  * The returned pointer to an authentication ID to the caller.
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  */
// NTSTATUS
// NTAPI
// SeQueryAuthenticationIdToken(
//     _In_ PACCESS_TOKEN Token,
//     _Out_ PLUID LogonId)
// {
//     PAGED_CODE();
//
//     *LogonId = ((PTOKEN)Token)->AuthenticationId;
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Gathers the security impersonation level of an access token.
//  *
//  * @param[in] Token
//  * A valid access token where the impersonation level has to be gathered.
//  *
//  * @return
//  * Returns the security impersonation level from a valid token.
//  */
// SECURITY_IMPERSONATION_LEVEL
// NTAPI
// SeTokenImpersonationLevel(
//     _In_ PACCESS_TOKEN Token)
// {
//     PAGED_CODE();
//
//     return ((PTOKEN)Token)->ImpersonationLevel;
// }
//
// /**
//  * @brief
//  * Gathers the token type of an access token. A token ca be either
//  * a primary token or impersonation token.
//  *
//  * @param[in] Token
//  * A valid access token where the token type has to be gathered.
//  *
//  * @return
//  * Returns the token type from a valid token.
//  */
// TOKEN_TYPE
// NTAPI
// SeTokenType(
//     _In_ PACCESS_TOKEN Token)
// {
//     PAGED_CODE();
//
//     return ((PTOKEN)Token)->TokenType;
// }
//
// /**
//  * @brief
//  * Determines if a token is either an admin token or not. Such
//  * condition is checked based upon TOKEN_HAS_ADMIN_GROUP flag,
//  * which means if the respective access token belongs to an
//  * administrator group or not.
//  *
//  * @param[in] Token
//  * A valid access token to determine if such token is admin or not.
//  *
//  * @return
//  * Returns TRUE if the token is an admin one, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SeTokenIsAdmin(
//     _In_ PACCESS_TOKEN Token)
// {
//     PAGED_CODE();
//
//     // NOTE: Win7+ instead really checks the list of groups in the token
//     // (since TOKEN_HAS_ADMIN_GROUP == TOKEN_WRITE_RESTRICTED ...)
//     return (((PTOKEN)Token)->TokenFlags & TOKEN_HAS_ADMIN_GROUP) != 0;
// }
//
// /**
//  * @brief
//  * Determines if a token is restricted or not, based upon the token
//  * flags.
//  *
//  * @param[in] Token
//  * A valid access token to determine if such token is restricted.
//  *
//  * @return
//  * Returns TRUE if the token is restricted, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SeTokenIsRestricted(
//     _In_ PACCESS_TOKEN Token)
// {
//     PAGED_CODE();
//
//     return (((PTOKEN)Token)->TokenFlags & TOKEN_IS_RESTRICTED) != 0;
// }
//
// /**
//  * @brief
//  * Determines if a token is write restricted, that is, nobody can write anything
//  * to it.
//  *
//  * @param[in] Token
//  * A valid access token to determine if such token is write restricted.
//  *
//  * @return
//  * Returns TRUE if the token is write restricted, FALSE otherwise.
//  *
//  * @remarks
//  * First introduced in NT 5.1 SP2 x86 (5.1.2600.2622), absent in NT 5.2,
//  * then finally re-introduced in Vista+.
//  */
// BOOLEAN
// NTAPI
// SeTokenIsWriteRestricted(
//     _In_ PACCESS_TOKEN Token)
// {
//     PAGED_CODE();
//
//     // NOTE: NT 5.1 SP2 x86 checks the SE_BACKUP_PRIVILEGES_CHECKED flag
//     // while Vista+ checks the TOKEN_WRITE_RESTRICTED flag as one expects.
//     return (((PTOKEN)Token)->TokenFlags & SE_BACKUP_PRIVILEGES_CHECKED) != 0;
// }
//
// /**
//  * @brief
//  * Determines whether the server is allowed to impersonate on behalf
//  * of a client or not. For further details, see Remarks.
//  *
//  * @param[in] ProcessToken
//  * A pointer to the primary access token of the server process
//  * that requests impersonation of the client target.
//  *
//  * @param[in] TokenToImpersonate
//  * A pointer to an access token that represents a client that is to
//  * be impersonated.
//  *
//  * @param[in] ImpersonationLevel
//  * The requested impersonation level.
//  *
//  * @return
//  * Returns TRUE if the conditions checked are met for token impersonation,
//  * FALSE otherwise.
//  *
//  * @remarks
//  * The server has to meet the following criteria in order to impersonate
//  * a client, that is:
//  *
//  * - The server must not impersonate a client beyond the level that
//  *   the client imposed on itself.
//  *
//  * - The server must be authenticated on the same logon session of
//  *   the target client.
//  *
//  * - IF NOT then the server's user ID has to match to that of the
//  *   target client.
//  *
//  * - The server must not be restricted in order to impersonate a
//  *   client that is not restricted.
//  *
//  * If the associated access token that represents the security properties
//  * of the server is granted the SeImpersonatePrivilege privilege the server
//  * is given immediate impersonation, regardless of the conditions above.
//  * If the client in question is associated with an anonymous token then
//  * the server is given immediate impersonation. Or if the server simply
//  * doesn't ask for impersonation but instead it wants to get the security
//  * identification of a client, the server is given immediate impersonation.
//  */
// BOOLEAN
// NTAPI
// SeTokenCanImpersonate(
//     _In_ PTOKEN ProcessToken,
//     _In_ PTOKEN TokenToImpersonate,
//     _In_ SECURITY_IMPERSONATION_LEVEL ImpersonationLevel)
// {
//     BOOLEAN CanImpersonate;
//     PAGED_CODE();
//
//     /*
//      * The server may want to obtain identification details of a client
//      * instead of impersonating so just give the server a pass.
//      */
//     if (ImpersonationLevel < SecurityIdentification)
//     {
//         DPRINT("The server doesn't ask for impersonation\n");
//         return TRUE;
//     }
//
//     /* Time to lock our tokens */
//     SepAcquireTokenLockShared(ProcessToken);
//     SepAcquireTokenLockShared(TokenToImpersonate);
//
//     /*
//      * As the name implies, an anonymous token has invisible security
//      * identification details. By the general rule these tokens do not
//      * pose a danger in terms of power escalation so give the server a pass.
//      */
//     if (RtlEqualLuid(&TokenToImpersonate->AuthenticationId,
//                      &SeAnonymousAuthenticationId))
//     {
//         DPRINT("The token to impersonate has an anonymous authentication ID, allow impersonation either way\n");
//         CanImpersonate = TRUE;
//         goto Quit;
//     }
//
//     /* Allow impersonation for the process server if it's granted the impersonation privilege */
//     if ((ProcessToken->TokenFlags & TOKEN_HAS_IMPERSONATE_PRIVILEGE) != 0)
//     {
//         DPRINT("The process is granted the impersonation privilege, allow impersonation\n");
//         CanImpersonate = TRUE;
//         goto Quit;
//     }
//
//     /*
//      * Deny impersonation for the server if it wants to impersonate a client
//      * beyond what the impersonation level originally permits.
//      */
//     if (ImpersonationLevel > TokenToImpersonate->ImpersonationLevel)
//     {
//         DPRINT1("Cannot impersonate a client above the permitted impersonation level!\n");
//         CanImpersonate = FALSE;
//         goto Quit;
//     }
//
//     /* Is the server authenticated on the same client originating session? */
//     if (!RtlEqualLuid(&ProcessToken->AuthenticationId,
//                       &TokenToImpersonate->OriginatingLogonSession))
//     {
//         /* It's not, check that at least both the server and client are the same user */
//         if (!RtlEqualSid(ProcessToken->UserAndGroups[0].Sid,
//                          TokenToImpersonate->UserAndGroups[0].Sid))
//         {
//             DPRINT1("Server and client aren't the same user!\n");
//             CanImpersonate = FALSE;
//             goto Quit;
//         }
//
//         /*
//          * Make sure the tokens haven't diverged in terms of restrictions
//          * that is one token is restricted but the other one isn't. If that
//          * would have been the case then the server would have impersonated
//          * a less restricted client thus potentially triggering an elevation,
//          * which is not what we want.
//          */
//         if (SeTokenIsRestricted(ProcessToken) !=
//             SeTokenIsRestricted(TokenToImpersonate))
//         {
//             DPRINT1("Attempting to impersonate a less restricted client token, bail out!\n");
//             CanImpersonate = FALSE;
//             goto Quit;
//         }
//     }
//
//     /* If we've reached that far then we can impersonate! */
//     DPRINT("We can impersonate\n");
//     CanImpersonate = TRUE;
//
// Quit:
//     SepReleaseTokenLock(TokenToImpersonate);
//     SepReleaseTokenLock(ProcessToken);
//     return CanImpersonate;
// }
//
// /* SYSTEM CALLS ***************************************************************/
//
// /**
//  * @brief
//  * Opens a token that is tied to a thread handle.
//  *
//  * @param[out] ThreadHandle
//  * Thread handle where the token is about to be opened.
//  *
//  * @param[in] DesiredAccess
//  * The request access right for the token.
//  *
//  * @param[in] OpenAsSelf
//  * If set to TRUE, the access check will be made with the security context
//  * of the process of the calling thread (opening as self). Otherwise the access
//  * check will be made with the security context of the calling thread instead.
//  *
//  * @param[in] HandleAttributes
//  * Handle attributes for the opened thread token handle.
//  *
//  * @param[out] TokenHandle
//  * The opened token handle returned to the caller for use.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the function has successfully opened the thread
//  * token. STATUS_CANT_OPEN_ANONYMOUS is returned if a token has SecurityAnonymous
//  * as impersonation level and we cannot open it. A failure NTSTATUS code is returned
//  * otherwise.
//  */
// NTSTATUS
// NTAPI
// NtOpenThreadTokenEx(
//     _In_ HANDLE ThreadHandle,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ BOOLEAN OpenAsSelf,
//     _In_ ULONG HandleAttributes,
//     _Out_ PHANDLE TokenHandle)
// {
//     PETHREAD Thread;
//     HANDLE hToken;
//     PTOKEN Token;
//     BOOLEAN CopyOnOpen, EffectiveOnly;
//     SECURITY_IMPERSONATION_LEVEL ImpersonationLevel;
//     SE_IMPERSONATION_STATE ImpersonationState;
//     KPROCESSOR_MODE PreviousMode;
//     NTSTATUS Status;
//     BOOLEAN RestoreImpersonation = FALSE;
//
//     PAGED_CODE();
//
//     PreviousMode = ExGetPreviousMode();
//
//     /* Ensure that we can give the handle to the caller */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForWriteHandle(TokenHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Validate object attributes */
//     HandleAttributes = ObpValidateAttributes(HandleAttributes, PreviousMode);
//
//     /*
//      * At first open the thread token for information access and verify
//      * that the token associated with the thread is valid.
//      */
//     Status = ObReferenceObjectByHandle(ThreadHandle, THREAD_QUERY_INFORMATION,
//                                        PsThreadType, PreviousMode, (PVOID*)&Thread,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to reference the object thread (Status 0x%lx)\n", Status);
//         return Status;
//     }
//
//     /* Reference the token from the thread */
//     Token = PsReferenceImpersonationToken(Thread, &CopyOnOpen, &EffectiveOnly,
//                                           &ImpersonationLevel);
//     if (Token == NULL)
//     {
//         DPRINT("Failed to reference the thread's impersonation token, thread has no token\n");
//         ObDereferenceObject(Thread);
//         return STATUS_NO_TOKEN;
//     }
//
//     /* Ensure the token has no anonymous security */
//     if (ImpersonationLevel == SecurityAnonymous)
//     {
//         DPRINT1("The thread token has anonymous security, can't open it\n");
//         PsDereferenceImpersonationToken(Token);
//         ObDereferenceObject(Thread);
//         return STATUS_CANT_OPEN_ANONYMOUS;
//     }
//
//     /* Revert to self if OpenAsSelf is specified */
//     if (OpenAsSelf)
//     {
//         RestoreImpersonation = PsDisableImpersonation(PsGetCurrentThread(),
//                                                       &ImpersonationState);
//     }
//
//     /* Call the private function to do the job */
//     Status = SepOpenThreadToken(Thread,
//                                 ThreadHandle,
//                                 Token,
//                                 DesiredAccess,
//                                 HandleAttributes,
//                                 EffectiveOnly,
//                                 CopyOnOpen,
//                                 ImpersonationLevel,
//                                 PreviousMode,
//                                 &hToken);
//
//     /* Restore the impersonation back if needed */
//     if (RestoreImpersonation)
//     {
//         PsRestoreImpersonation(PsGetCurrentThread(), &ImpersonationState);
//     }
//
//     /* Dereference the access token and the associated thread */
//     ObDereferenceObject(Token);
//     ObDereferenceObject(Thread);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open the thread's token (Status 0x%lx)\n", Status);
//         return Status;
//     }
//
//     /* Give the opened token handle to the caller */
//     _SEH2_TRY
//     {
//         *TokenHandle = hToken;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Opens a token that is tied to a thread handle.
//  *
//  * @param[out] ThreadHandle
//  * Thread handle where the token is about to be opened.
//  *
//  * @param[in] DesiredAccess
//  * The request access right for the token.
//  *
//  * @param[in] OpenAsSelf
//  * If set to TRUE, the access check will be made with the security context
//  * of the process of the calling thread (opening as self). Otherwise the access
//  * check will be made with the security context of the calling thread instead.
//  *
//  * @param[out] TokenHandle
//  * The opened token handle returned to the caller for use.
//  *
//  * @return
//  * See NtOpenThreadTokenEx.
//  */
// NTSTATUS
// NTAPI
// NtOpenThreadToken(
//     _In_ HANDLE ThreadHandle,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ BOOLEAN OpenAsSelf,
//     _Out_ PHANDLE TokenHandle)
// {
//     return NtOpenThreadTokenEx(ThreadHandle, DesiredAccess, OpenAsSelf, 0,
//                                TokenHandle);
// }
//
// /**
//  * @brief
//  * Compares tokens if they're equal or not.
//  *
//  * @param[in] FirstToken
//  * The first token.
//  *
//  * @param[in] SecondToken
//  * The second token.
//  *
//  * @param[out] Equal
//  * The retrieved value which determines if the tokens are
//  * equal or not.
//  *
//  * @return
//  * Returns STATUS_SUCCESS, otherwise it returns a failure NTSTATUS code.
//  */
// NTSTATUS
// NTAPI
// NtCompareTokens(
//     _In_ HANDLE FirstTokenHandle,
//     _In_ HANDLE SecondTokenHandle,
//     _Out_ PBOOLEAN Equal)
// {
//     KPROCESSOR_MODE PreviousMode;
//     PTOKEN FirstToken, SecondToken;
//     BOOLEAN IsEqual;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     PreviousMode = ExGetPreviousMode();
//
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForWriteBoolean(Equal);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     Status = ObReferenceObjectByHandle(FirstTokenHandle,
//                                        TOKEN_QUERY,
//                                        SeTokenObjectType as *mut _OBJECT_TYPE,
//                                        PreviousMode,
//                                        (PVOID*)&FirstToken,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ObReferenceObjectByHandle() failed (Status 0x%lx)\n", Status);
//         return Status;
//     }
//
//     Status = ObReferenceObjectByHandle(SecondTokenHandle,
//                                        TOKEN_QUERY,
//                                        SeTokenObjectType as *mut _OBJECT_TYPE,
//                                        PreviousMode,
//                                        (PVOID*)&SecondToken,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ObReferenceObjectByHandle() failed (Status 0x%lx)\n", Status);
//         ObDereferenceObject(FirstToken);
//         return Status;
//     }
//
//     if (FirstToken != SecondToken)
//     {
//         Status = SepCompareTokens(FirstToken,
//                                   SecondToken,
//                                   &IsEqual);
//     }
//     else
//     {
//         IsEqual = TRUE;
//     }
//
//     ObDereferenceObject(SecondToken);
//     ObDereferenceObject(FirstToken);
//
//     if (NT_SUCCESS(Status))
//     {
//         _SEH2_TRY
//         {
//             *Equal = IsEqual;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Allows the calling thread to impersonate the system's anonymous
//  * logon token.
//  *
//  * @param[in] ThreadHandle
//  * A handle to the thread to start the procedure of logon token
//  * impersonation. The thread must have the THREAD_IMPERSONATE
//  * access right.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the thread has successfully impersonated the
//  * anonymous logon token, otherwise a failure NTSTATUS code is returned.
//  *
//  * @remarks
//  * By default the system gives the opportunity to the caller to impersonate
//  * the anonymous logon token without including the Everyone Group SID.
//  * In cases where the caller wants to impersonate the token including such
//  * group, the EveryoneIncludesAnonymous registry value setting has to be set
//  * to 1, from HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Lsa registry
//  * path. The calling thread must invoke PsRevertToSelf when impersonation
//  * is no longer needed or RevertToSelf if the calling execution is done
//  * in user mode.
//  */
// NTSTATUS
// NTAPI
// NtImpersonateAnonymousToken(
//     _In_ HANDLE ThreadHandle)
// {
//     PETHREAD Thread;
//     KPROCESSOR_MODE PreviousMode;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     PreviousMode = ExGetPreviousMode();
//
//     /* Obtain the thread object from the handle */
//     Status = ObReferenceObjectByHandle(ThreadHandle,
//                                        THREAD_IMPERSONATE,
//                                        PsThreadType,
//                                        PreviousMode,
//                                        (PVOID*)&Thread,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("NtImpersonateAnonymousToken(): Failed to reference the object (Status 0x%lx)\n", Status);
//         return Status;
//     }
//
//     /* Call the private routine to impersonate the token */
//     Status = SepImpersonateAnonymousToken(Thread, PreviousMode);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("NtImpersonateAnonymousToken(): Failed to impersonate the token (Status 0x%lx)\n", Status);
//     }
//
//     ObDereferenceObject(Thread);
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
