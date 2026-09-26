use ntkrnl_sys::ddk::*;
use ntkrnl_sys::ddk::SeDefaultObjectMethod as SeDefaultObjectMethodImport;
use ntkrnl_sys::ddk::SeInitSystem as SeInitSystemImport;
use ntkrnl_sys::ddk::SeQuerySecurityAccessMask as SeQuerySecurityAccessMaskImport;
use ntkrnl_sys::ddk::SeSetSecurityAccessMask as SeSetSecurityAccessMaskImport;
use ntkrnl_sys::base::{STATUS_SUCCESS, NT_SUCCESS, TRUE, FALSE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::{PagedPool, NonPagedPool};
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn ZwCreateDirectoryObject(
        DirectoryHandle: *mut HANDLE,
        DesiredAccess: ACCESS_MASK,
        ObjectAttributes: *mut _OBJECT_ATTRIBUTES,
    ) -> NTSTATUS;
    fn ZwCreateEvent(
        EventHandle: *mut HANDLE,
        DesiredAccess: ACCESS_MASK,
        ObjectAttributes: *mut _OBJECT_ATTRIBUTES,
        EventType: ULONG,
        InitialState: BOOLEAN,
    ) -> NTSTATUS;
    fn ZwClose(Handle: HANDLE) -> NTSTATUS;
    fn SepInitializeTokenImplementation();

    static mut ExpInitializationPhase: ULONG;
    static mut SepSubjectContextLock: ERESOURCE;

    static SeCreateTokenPrivilege: _LUID;
    static SeAssignPrimaryTokenPrivilege: _LUID;
    static SeLockMemoryPrivilege: _LUID;
    static SeIncreaseQuotaPrivilege: _LUID;
    static SeUnsolicitedInputPrivilege: _LUID;
    static SeTcbPrivilege: _LUID;
    static SeSecurityPrivilege: _LUID;
    static SeTakeOwnershipPrivilege: _LUID;
    static SeLoadDriverPrivilege: _LUID;
    static SeCreatePagefilePrivilege: _LUID;
    static SeIncreaseBasePriorityPrivilege: _LUID;
    static SeSystemProfilePrivilege: _LUID;
    static SeSystemtimePrivilege: _LUID;
    static SeProfileSingleProcessPrivilege: _LUID;
    static SeCreatePermanentPrivilege: _LUID;
    static SeBackupPrivilege: _LUID;
    static SeRestorePrivilege: _LUID;
    static SeShutdownPrivilege: _LUID;
    static SeDebugPrivilege: _LUID;
    static SeAuditPrivilege: _LUID;
    static SeSystemEnvironmentPrivilege: _LUID;
    static SeChangeNotifyPrivilege: _LUID;
    static SeRemoteShutdownPrivilege: _LUID;
    static SeUndockPrivilege: _LUID;
    static SeSyncAgentPrivilege: _LUID;
    static SeEnableDelegationPrivilege: _LUID;
    static SeManageVolumePrivilege: _LUID;
    static SeImpersonatePrivilege: _LUID;
    static SeCreateGlobalPrivilege: _LUID;

    static mut SeNullSid: PSID;
    static mut SeWorldSid: PSID;
    static mut SeLocalSid: PSID;
    static mut SeCreatorOwnerSid: PSID;
    static mut SeCreatorGroupSid: PSID;
    static mut SeNtAuthoritySid: PSID;
    static mut SeDialupSid: PSID;
    static mut SeNetworkSid: PSID;
    static mut SeBatchSid: PSID;
    static mut SeInteractiveSid: PSID;
    static mut SeLocalSystemSid: PSID;
    static mut SeAliasAdminsSid: PSID;
    static mut SeAliasUsersSid: PSID;
    static mut SeAliasGuestsSid: PSID;
    static mut SeAliasPowerUsersSid: PSID;
    static mut SeAliasAccountOpsSid: PSID;
    static mut SeAliasSystemOpsSid: PSID;
    static mut SeAliasPrintOpsSid: PSID;
    static mut SeAliasBackupOpsSid: PSID;
    static mut SeAuthenticatedUsersSid: PSID;
    static mut SeRestrictedSid: PSID;
    static mut SeAnonymousLogonSid: PSID;
    static mut SeLocalServiceSid: PSID;
    static mut SeNetworkServiceSid: PSID;

    static SePublicDefaultSd: PSECURITY_DESCRIPTOR;
}
extern "C" {
    fn InitializeObjectAttributes(
        p: *mut _OBJECT_ATTRIBUTES,
        n: *mut _UNICODE_STRING,
        a: ULONG,
        r: HANDLE,
        s: PSECURITY_DESCRIPTOR,
    );
    fn ExInitializeResource(Resource: *mut ERESOURCE);
    fn PspClearCrossThreadFlag(Thread: PETHREAD, Flag: ULONG);
}

const CT_ACTIVE_IMPERSONATION_INFO_BIT: ULONG = 0x8;
// ReactOS uses 7 for ALL architectures (3 low bits), matching C #define.
const MAX_FAST_REFS: usize = 7;
const TAG_SE: ULONG = u32::from_le_bytes([b'S', b'e', b' ', b' ']);
const SECURITY_DESCRIPTOR_REVISION: ULONG = 1;
const ACL_REVISION: ULONG = 2;
const DIRECTORY_ALL_ACCESS: ACCESS_MASK = 0x000F000F;
const DIRECTORY_TRAVERSE: ACCESS_MASK = 0x2;
const DIRECTORY_QUERY: ACCESS_MASK = 0x1;
const READ_CONTROL: ACCESS_MASK = 0x00020000;
const WRITE_OWNER: ACCESS_MASK = 0x00080000;
const WRITE_DAC: ACCESS_MASK = 0x00040000;
const ACCESS_SYSTEM_SECURITY: ACCESS_MASK = 0x01000000;
const GENERIC_WRITE: ACCESS_MASK = 0x40000000;

const OWNER_SECURITY_INFORMATION: SECURITY_INFORMATION = 0x00000001;
const GROUP_SECURITY_INFORMATION: SECURITY_INFORMATION = 0x00000002;
const DACL_SECURITY_INFORMATION: SECURITY_INFORMATION = 0x00000004;
const SACL_SECURITY_INFORMATION: SECURITY_INFORMATION = 0x00000008;

const NotificationEvent: ULONG = 0;

const SE_MAX_AUDIT_PARAMETERS: ULONG = 32;

type SECURITY_OPERATION_CODE = u32;
const SetSecurityDescriptor: SECURITY_OPERATION_CODE = 0;
const QuerySecurityDescriptor: SECURITY_OPERATION_CODE = 1;
const DeleteSecurityDescriptor: SECURITY_OPERATION_CODE = 2;
const AssignSecurityDescriptor: SECURITY_OPERATION_CODE = 3;

type SE_ADT_PARAMETER_TYPE = u32;

#[repr(C)]
struct LUID {
    LowPart: ULONG,
    HighPart: LONG,
}

#[cfg(target_arch = "aarch64")]
unsafe fn _se_trace(msg: &[u8]) {
    extern "C" {
        static mut Arm64UartKernelBase: PVOID;
        fn Arm64UartPutChar(c: u8);
    }
    let u = Arm64UartKernelBase as *mut ULONG;
    if !u.is_null() {
        for &ch in msg {
            Arm64UartPutChar(ch as u8);
        }
    }
}

#[cfg(not(target_arch = "aarch64"))]
unsafe fn _se_trace(_msg: &[u8]) {}

#[no_mangle]
pub static mut SeAnonymousLogonToken: PTOKEN = core::ptr::null_mut();
#[no_mangle]
pub static mut SeAnonymousLogonTokenNoEveryone: PTOKEN = core::ptr::null_mut();
#[no_mangle]
pub static mut SeExports: PSE_EXPORTS = core::ptr::null_mut();
static mut SepExports: SE_EXPORTS = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut SidInTokenCalls: ULONG = 0;

// CODE_SEG("INIT")
unsafe fn SepInitExports() -> BOOLEAN {
    SepExports.SeCreateTokenPrivilege = SeCreateTokenPrivilege;
    SepExports.SeAssignPrimaryTokenPrivilege = SeAssignPrimaryTokenPrivilege;
    SepExports.SeLockMemoryPrivilege = SeLockMemoryPrivilege;
    SepExports.SeIncreaseQuotaPrivilege = SeIncreaseQuotaPrivilege;
    SepExports.SeUnsolicitedInputPrivilege = SeUnsolicitedInputPrivilege;
    SepExports.SeTcbPrivilege = SeTcbPrivilege;
    SepExports.SeSecurityPrivilege = SeSecurityPrivilege;
    SepExports.SeTakeOwnershipPrivilege = SeTakeOwnershipPrivilege;
    SepExports.SeLoadDriverPrivilege = SeLoadDriverPrivilege;
    SepExports.SeCreatePagefilePrivilege = SeCreatePagefilePrivilege;
    SepExports.SeIncreaseBasePriorityPrivilege = SeIncreaseBasePriorityPrivilege;
    SepExports.SeSystemProfilePrivilege = SeSystemProfilePrivilege;
    SepExports.SeSystemtimePrivilege = SeSystemtimePrivilege;
    SepExports.SeProfileSingleProcessPrivilege = SeProfileSingleProcessPrivilege;
    SepExports.SeCreatePermanentPrivilege = SeCreatePermanentPrivilege;
    SepExports.SeBackupPrivilege = SeBackupPrivilege;
    SepExports.SeRestorePrivilege = SeRestorePrivilege;
    SepExports.SeShutdownPrivilege = SeShutdownPrivilege;
    SepExports.SeDebugPrivilege = SeDebugPrivilege;
    SepExports.SeAuditPrivilege = SeAuditPrivilege;
    SepExports.SeSystemEnvironmentPrivilege = SeSystemEnvironmentPrivilege;
    SepExports.SeChangeNotifyPrivilege = SeChangeNotifyPrivilege;
    SepExports.SeRemoteShutdownPrivilege = SeRemoteShutdownPrivilege;

    SepExports.SeNullSid = SeNullSid;
    SepExports.SeWorldSid = SeWorldSid;
    SepExports.SeLocalSid = SeLocalSid;
    SepExports.SeCreatorOwnerSid = SeCreatorOwnerSid;
    SepExports.SeCreatorGroupSid = SeCreatorGroupSid;
    SepExports.SeNtAuthoritySid = SeNtAuthoritySid;
    SepExports.SeDialupSid = SeDialupSid;
    SepExports.SeNetworkSid = SeNetworkSid;
    SepExports.SeBatchSid = SeBatchSid;
    SepExports.SeInteractiveSid = SeInteractiveSid;
    SepExports.SeLocalSystemSid = SeLocalSystemSid;
    SepExports.SeAliasAdminsSid = SeAliasAdminsSid;
    SepExports.SeAliasUsersSid = SeAliasUsersSid;
    SepExports.SeAliasGuestsSid = SeAliasGuestsSid;
    SepExports.SeAliasPowerUsersSid = SeAliasPowerUsersSid;
    SepExports.SeAliasAccountOpsSid = SeAliasAccountOpsSid;
    SepExports.SeAliasSystemOpsSid = SeAliasSystemOpsSid;
    SepExports.SeAliasPrintOpsSid = SeAliasPrintOpsSid;
    SepExports.SeAliasBackupOpsSid = SeAliasBackupOpsSid;
    SepExports.SeAuthenticatedUsersSid = SeAuthenticatedUsersSid;
    SepExports.SeRestrictedSid = SeRestrictedSid;
    SepExports.SeAnonymousLogonSid = SeAnonymousLogonSid;
    SepExports.SeLocalServiceSid = SeLocalServiceSid;
    SepExports.SeNetworkServiceSid = SeNetworkServiceSid;

    SepExports.SeUndockPrivilege = SeUndockPrivilege;
    SepExports.SeSyncAgentPrivilege = SeSyncAgentPrivilege;
    SepExports.SeEnableDelegationPrivilege = SeEnableDelegationPrivilege;
    SepExports.SeManageVolumePrivilege = SeManageVolumePrivilege;
    SepExports.SeImpersonatePrivilege = SeImpersonatePrivilege;
    SepExports.SeCreateGlobalPrivilege = SeCreateGlobalPrivilege;

    SeExports = &raw mut SepExports as PSE_EXPORTS;
    1u8
}

// CODE_SEG("INIT")
unsafe fn SepInitializationPhase0() -> BOOLEAN {
    _se_trace(b"[SE] SepInitPhase0 enter\n");
    if ExLuidInitialization() == 0 {
        return 0u8;
    }
    _se_trace(b"[SE] after ExLuidInit\n");
    if SepInitSecurityIDs() == 0 {
        return 0u8;
    }
    _se_trace(b"[SE] after SepInitSecurityIDs\n");
    if SepInitDACLs() == 0 {
        return 0u8;
    }
    _se_trace(b"[SE] after SepInitDACLs\n");
    if SepInitSDs() == 0 {
        return 0u8;
    }
    _se_trace(b"[SE] after SepInitSDs\n");
    SepInitPrivileges();
    _se_trace(b"[SE] after SepInitPrivileges\n");
    if SepInitExports() == 0 {
        return 0u8;
    }
    _se_trace(b"[SE] after SepInitExports\n");

    ExInitializeResource(&raw mut SepSubjectContextLock);
    _se_trace(b"[SE] after ExInitializeResource\n");

    SepInitializeTokenImplementation();
    _se_trace(b"[SE] after SepInitTokenImpl\n");

    if SeRmInitPhase0() == 0 {
        return 0u8;
    }
    _se_trace(b"[SE] after SeRmInitPhase0\n");

    (*(PsGetCurrentThread() as *mut _ETHREAD)).ImpersonationInfo = core::ptr::null_mut();
    PspClearCrossThreadFlag(
        PsGetCurrentThread() as PETHREAD,
        CT_ACTIVE_IMPERSONATION_INFO_BIT,
    );
    _se_trace(b"[SE] after clear impersonation\n");

    // WORKAROUND: Token initialization is in a C helper (SepInitTokensShim in
    // kernel_c_helpers.c) to work around an LLVM i386 fat LTO bug. During LTO,
    // LLVM inlines SepInitializationPhase0 into SeInitSystem and incorrectly
    // inserts unreachable/ud2 after the call to `@ObInitializeFastReference@8`
    // that sets up the process token. This drops all subsequent code (anonymous
    // logon token creation, return TRUE).
    //
    // Re-probed 2026-05-07 by inlining the body into Rust. The bug is real and
    // NOT a captures(none) ghost — `test_no_unexpected_ud2` flagged it. None
    // of `opaque_ptr`, `compiler_fence(SeqCst)`, `core::hint::black_box`
    // defeat the miscompilation. The C trampoline genuinely has to stay
    // (it's the only way to put the call sequence outside the Rust LTO scope).
    extern "C" {
        fn SepInitTokensShim() -> BOOLEAN;
    }
    if SepInitTokensShim() == 0 {
        return 0u8;
    }
    _se_trace(b"[SE] after token init\n");

    1u8
}

// CODE_SEG("INIT")
unsafe fn SepInitializationPhase1() -> BOOLEAN {
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut name: _UNICODE_STRING = core::mem::zeroed();
    let mut security_handle: HANDLE = core::ptr::null_mut();
    let mut event_handle: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut security_descriptor: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut dacl: PACL;
    let dacl_length: ULONG;

    let token_value = (*(PsGetCurrentProcess() as *mut _EPROCESS)).Token.__bindgen_anon_1.Value;
    status = ObInsertObject(
        (token_value & !(MAX_FAST_REFS as ULONG_PTR)) as PVOID,
        core::ptr::null_mut(),
        0,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    RtlCreateSecurityDescriptor(
        &mut security_descriptor as *mut _SECURITY_DESCRIPTOR as PVOID,
        SECURITY_DESCRIPTOR_REVISION,
    );

    dacl_length = core::mem::size_of::<ACL>() as ULONG
        + 3 * core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
        + RtlLengthSid(SeLocalSystemSid)
        + RtlLengthSid(SeAliasAdminsSid)
        + RtlLengthSid(SeWorldSid);
    dacl = ExAllocatePoolWithTag(NonPagedPool, dacl_length as SIZE_T, TAG_SE) as PACL;
    if dacl.is_null() {
        return 0u8;
    }

    status = RtlCreateAcl(dacl, dacl_length, ACL_REVISION as ULONG);

    status = RtlAddAccessAllowedAce(dacl, ACL_REVISION as ULONG, DIRECTORY_ALL_ACCESS, SeLocalSystemSid);

    status = RtlAddAccessAllowedAce(
        dacl,
        ACL_REVISION,
        READ_CONTROL | DIRECTORY_TRAVERSE | DIRECTORY_QUERY,
        SeAliasAdminsSid,
    );

    status = RtlAddAccessAllowedAce(dacl, ACL_REVISION as ULONG, DIRECTORY_TRAVERSE, SeWorldSid);

    status = RtlSetDaclSecurityDescriptor(
        &mut security_descriptor as *mut _SECURITY_DESCRIPTOR as PVOID,
        1u8,
        dacl,
        0u8,
    );

    static mut SECURITY_PATH: [u16; 10] = [
        b'\\' as u16, b'S' as u16, b'e' as u16, b'c' as u16, b'u' as u16,
        b'r' as u16, b'i' as u16, b't' as u16, b'y' as u16, 0,
    ];
    RtlInitUnicodeString(&mut name, (*(&raw const SECURITY_PATH)).as_ptr());
    InitializeObjectAttributes(
        &mut object_attributes,
        &mut name,
        OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        &mut security_descriptor as *mut _SECURITY_DESCRIPTOR as PSECURITY_DESCRIPTOR,
    );

    status = ZwCreateDirectoryObject(
        &mut security_handle,
        DIRECTORY_ALL_ACCESS,
        &mut object_attributes,
    );

    ExFreePoolWithTag(dacl as PVOID, TAG_SE);

    static mut EVENT_NAME: [u16; 31] = [
        b'L' as u16, b'S' as u16, b'A' as u16, b'_' as u16, b'A' as u16,
        b'U' as u16, b'T' as u16, b'H' as u16, b'E' as u16, b'N' as u16,
        b'T' as u16, b'I' as u16, b'C' as u16, b'A' as u16, b'T' as u16,
        b'I' as u16, b'O' as u16, b'N' as u16, b'_' as u16, b'I' as u16,
        b'N' as u16, b'I' as u16, b'T' as u16, b'I' as u16, b'A' as u16,
        b'L' as u16, b'I' as u16, b'Z' as u16, b'E' as u16, b'D' as u16,
        0,
    ];
    RtlInitUnicodeString(&mut name, (*(&raw const EVENT_NAME)).as_ptr());
    InitializeObjectAttributes(
        &mut object_attributes,
        &mut name,
        OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
        security_handle,
        SePublicDefaultSd,
    );

    status = ZwCreateEvent(
        &mut event_handle,
        GENERIC_WRITE,
        &mut object_attributes,
        _EVENT_TYPE_NotificationEvent,
        0u8,
    );

    status = ZwClose(event_handle);

    status = ZwClose(security_handle);

    1u8
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn SeInitSystem() -> BOOLEAN {
    match ExpInitializationPhase {
        0 => SepInitializationPhase0(),
        1 => SepInitializationPhase1(),
        _ => {
            KeBugCheckEx(ntkrnl_sys::bugcodes::UNEXPECTED_INITIALIZATION_CALL, 0, ExpInitializationPhase as ULONG_PTR, 0, 0);
            0u8
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn SeDefaultObjectMethod(
    object: PVOID,
    operation_type: SECURITY_OPERATION_CODE,
    security_information: PSECURITY_INFORMATION,
    security_descriptor: PSECURITY_DESCRIPTOR,
    return_length: *mut ULONG,
    old_security_descriptor: *mut PSECURITY_DESCRIPTOR,
    pool_type: POOL_TYPE,
    generic_mapping: PGENERIC_MAPPING,
    _access_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let object = opaque_ptr(object);
    let security_descriptor = opaque_ptr(security_descriptor);
    let return_length = opaque_ptr(return_length);
    let old_security_descriptor = opaque_ptr(old_security_descriptor);
    let generic_mapping = opaque_ptr(generic_mapping);
    match operation_type {
        _SECURITY_OPERATION_CODE_SetSecurityDescriptor => ObSetSecurityDescriptorInfo(
            object,
            security_information,
            security_descriptor,
            old_security_descriptor,
            pool_type,
            generic_mapping,
        ),
        _SECURITY_OPERATION_CODE_QuerySecurityDescriptor => ObQuerySecurityDescriptorInfo(
            object,
            security_information,
            security_descriptor,
            return_length,
            old_security_descriptor,
        ),
        _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor => ObDeassignSecurity(old_security_descriptor),
        _SECURITY_OPERATION_CODE_AssignSecurityDescriptor => {
            ObAssignObjectSecurityDescriptor(object, security_descriptor, pool_type);
            STATUS_SUCCESS
        }
        _ => {
            KeBugCheckEx(ntkrnl_sys::bugcodes::SECURITY_SYSTEM, 0, STATUS_INVALID_PARAMETER as ULONG_PTR, 0, 0);
            STATUS_SUCCESS
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn SeQuerySecurityAccessMask(
    security_information: SECURITY_INFORMATION,
    desired_access: PACCESS_MASK,
) {
    *desired_access = 0;

    if (security_information
        & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION))
        != 0
    {
        *desired_access |= READ_CONTROL;
    }

    if (security_information & SACL_SECURITY_INFORMATION) != 0 {
        *desired_access |= ACCESS_SYSTEM_SECURITY;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SeSetSecurityAccessMask(
    security_information: SECURITY_INFORMATION,
    desired_access: PACCESS_MASK,
) {
    *desired_access = 0;

    if (security_information & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION)) != 0 {
        *desired_access |= WRITE_OWNER;
    }

    if (security_information & DACL_SECURITY_INFORMATION) != 0 {
        *desired_access |= WRITE_DAC;
    }

    if (security_information & SACL_SECURITY_INFORMATION) != 0 {
        *desired_access |= ACCESS_SYSTEM_SECURITY;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SeReportSecurityEvent(
    flags: ULONG,
    source_name: PUNICODE_STRING,
    user_sid: PSID,
    audit_parameters: PSE_ADT_PARAMETER_ARRAY,
) -> NTSTATUS {
    let source_name = opaque_ptr(source_name);
    let user_sid = opaque_ptr(user_sid);
    let mut subject_context: SECURITY_SUBJECT_CONTEXT = core::mem::zeroed();
    let mut effective_token: PTOKEN;
    let mut sid: PISID;
    let mut status: NTSTATUS;

    if (flags != 0)
        || source_name.is_null()
        || (*source_name).Buffer.is_null()
        || ((*source_name).Length == 0)
        || audit_parameters.is_null()
        || ((*audit_parameters).ParameterCount > SE_MAX_AUDIT_PARAMETERS - 4)
    {
        return STATUS_INVALID_PARAMETER;
    }

    status = RtlValidateUnicodeString(0, source_name);
    if status < 0 {
        return status;
    }

    if !user_sid.is_null() {
        if RtlValidSid(user_sid) == 0 {
            return STATUS_INVALID_PARAMETER;
        }
        sid = user_sid as PISID;
    } else {
        SeCaptureSubjectContext(&mut subject_context);

        effective_token = if !subject_context.ClientToken.is_null() {
            subject_context.ClientToken as PTOKEN
        } else {
            subject_context.PrimaryToken as PTOKEN
        };

        sid = (*(*effective_token).UserAndGroups).Sid as PISID;
    }

    if sid != user_sid as PISID {
        SeReleaseSubjectContext(&mut subject_context);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeSetAuditParameter(
    audit_parameters: PSE_ADT_PARAMETER_ARRAY,
    _type: SE_ADT_PARAMETER_TYPE,
    _index: ULONG,
    _data: PVOID,
) -> NTSTATUS {
    let _data = opaque_ptr(_data);
    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Security manager infrastructure
//  * COPYRIGHT:   Copyright Timo Kreuzer <timo.kreuzer@reactos.org>
//  *              Copyright Eric Kohl
//  *              Copyright Aleksey Bragin
//  *              Copyright Alex Ionescu <alex@relsoft.net>
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
// #define _SE_TRACE(msg) do { \
//     extern volatile PVOID Arm64UartKernelBase; \
//     volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase; \
//     if (_u) { const char *_m = (msg); while (*_m) *_u = (ULONG)*_m++; } \
// } while(0)
// #else
// #define _SE_TRACE(msg) ((void)0)
// #endif
//
//
// /* GLOBALS ********************************************************************/
//
// PTOKEN SeAnonymousLogonToken = NULL;
// PTOKEN SeAnonymousLogonTokenNoEveryone = NULL;
// PSE_EXPORTS SeExports = NULL;
// SE_EXPORTS SepExports;
// ULONG SidInTokenCalls = 0;
//
// extern ULONG ExpInitializationPhase;
// extern ERESOURCE SepSubjectContextLock;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /**
//  * @brief
//  * Initializes all the security exports upon initialization phase of
//  * the module.
//  *
//  * @return
//  * Returns TRUE.
//  */
// static
// CODE_SEG("INIT")
// BOOLEAN
// SepInitExports(VOID)
// {
//     SepExports.SeCreateTokenPrivilege = SeCreateTokenPrivilege;
//     SepExports.SeAssignPrimaryTokenPrivilege = SeAssignPrimaryTokenPrivilege;
//     SepExports.SeLockMemoryPrivilege = SeLockMemoryPrivilege;
//     SepExports.SeIncreaseQuotaPrivilege = SeIncreaseQuotaPrivilege;
//     SepExports.SeUnsolicitedInputPrivilege = SeUnsolicitedInputPrivilege;
//     SepExports.SeTcbPrivilege = SeTcbPrivilege;
//     SepExports.SeSecurityPrivilege = SeSecurityPrivilege;
//     SepExports.SeTakeOwnershipPrivilege = SeTakeOwnershipPrivilege;
//     SepExports.SeLoadDriverPrivilege = SeLoadDriverPrivilege;
//     SepExports.SeCreatePagefilePrivilege = SeCreatePagefilePrivilege;
//     SepExports.SeIncreaseBasePriorityPrivilege = SeIncreaseBasePriorityPrivilege;
//     SepExports.SeSystemProfilePrivilege = SeSystemProfilePrivilege;
//     SepExports.SeSystemtimePrivilege = SeSystemtimePrivilege;
//     SepExports.SeProfileSingleProcessPrivilege = SeProfileSingleProcessPrivilege;
//     SepExports.SeCreatePermanentPrivilege = SeCreatePermanentPrivilege;
//     SepExports.SeBackupPrivilege = SeBackupPrivilege;
//     SepExports.SeRestorePrivilege = SeRestorePrivilege;
//     SepExports.SeShutdownPrivilege = SeShutdownPrivilege;
//     SepExports.SeDebugPrivilege = SeDebugPrivilege;
//     SepExports.SeAuditPrivilege = SeAuditPrivilege;
//     SepExports.SeSystemEnvironmentPrivilege = SeSystemEnvironmentPrivilege;
//     SepExports.SeChangeNotifyPrivilege = SeChangeNotifyPrivilege;
//     SepExports.SeRemoteShutdownPrivilege = SeRemoteShutdownPrivilege;
//
//     SepExports.SeNullSid = SeNullSid;
//     SepExports.SeWorldSid = SeWorldSid;
//     SepExports.SeLocalSid = SeLocalSid;
//     SepExports.SeCreatorOwnerSid = SeCreatorOwnerSid;
//     SepExports.SeCreatorGroupSid = SeCreatorGroupSid;
//     SepExports.SeNtAuthoritySid = SeNtAuthoritySid;
//     SepExports.SeDialupSid = SeDialupSid;
//     SepExports.SeNetworkSid = SeNetworkSid;
//     SepExports.SeBatchSid = SeBatchSid;
//     SepExports.SeInteractiveSid = SeInteractiveSid;
//     SepExports.SeLocalSystemSid = SeLocalSystemSid;
//     SepExports.SeAliasAdminsSid = SeAliasAdminsSid;
//     SepExports.SeAliasUsersSid = SeAliasUsersSid;
//     SepExports.SeAliasGuestsSid = SeAliasGuestsSid;
//     SepExports.SeAliasPowerUsersSid = SeAliasPowerUsersSid;
//     SepExports.SeAliasAccountOpsSid = SeAliasAccountOpsSid;
//     SepExports.SeAliasSystemOpsSid = SeAliasSystemOpsSid;
//     SepExports.SeAliasPrintOpsSid = SeAliasPrintOpsSid;
//     SepExports.SeAliasBackupOpsSid = SeAliasBackupOpsSid;
//     SepExports.SeAuthenticatedUsersSid = SeAuthenticatedUsersSid;
//     SepExports.SeRestrictedSid = SeRestrictedSid;
//     SepExports.SeAnonymousLogonSid = SeAnonymousLogonSid;
//     SepExports.SeLocalServiceSid = SeLocalServiceSid;
//     SepExports.SeNetworkServiceSid = SeNetworkServiceSid;
//
//     SepExports.SeUndockPrivilege = SeUndockPrivilege;
//     SepExports.SeSyncAgentPrivilege = SeSyncAgentPrivilege;
//     SepExports.SeEnableDelegationPrivilege = SeEnableDelegationPrivilege;
//     SepExports.SeManageVolumePrivilege = SeManageVolumePrivilege;
//     SepExports.SeImpersonatePrivilege = SeImpersonatePrivilege;
//     SepExports.SeCreateGlobalPrivilege = SeCreateGlobalPrivilege;
//
//     SeExports = &SepExports;
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Handles the phase 0 procedure of the SRM initialization.
//  *
//  * @return
//  * Returns TRUE if the phase 0 initialization has succeeded and that
//  * we can proceed further with next initialization phase, FALSE
//  * otherwise.
//  */
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// SepInitializationPhase0(VOID)
// {
//     PAGED_CODE();
//
//     _SE_TRACE("[SE] SepInitPhase0 enter\n");
//     if (!ExLuidInitialization()) return FALSE;
//     _SE_TRACE("[SE] after ExLuidInit\n");
//     if (!SepInitSecurityIDs()) return FALSE;
//     _SE_TRACE("[SE] after SepInitSecurityIDs\n");
//     if (!SepInitDACLs()) return FALSE;
//     _SE_TRACE("[SE] after SepInitDACLs\n");
//     if (!SepInitSDs()) return FALSE;
//     _SE_TRACE("[SE] after SepInitSDs\n");
//     SepInitPrivileges();
//     _SE_TRACE("[SE] after SepInitPrivileges\n");
//     if (!SepInitExports()) return FALSE;
//     _SE_TRACE("[SE] after SepInitExports\n");
//
//     /* Initialize the subject context lock */
//     ExInitializeResource(&SepSubjectContextLock);
//     _SE_TRACE("[SE] after ExInitializeResource\n");
//
//     /* Initialize token objects */
//     SepInitializeTokenImplementation();
//     _SE_TRACE("[SE] after SepInitTokenImpl\n");
//
//     /* Initialize logon sessions */
//     if (!SeRmInitPhase0()) return FALSE;
//     _SE_TRACE("[SE] after SeRmInitPhase0\n");
//
//     /* Clear impersonation info for the idle thread */
//     PsGetCurrentThread()->ImpersonationInfo = NULL;
//     PspClearCrossThreadFlag(PsGetCurrentThread(),
//                             CT_ACTIVE_IMPERSONATION_INFO_BIT);
//     _SE_TRACE("[SE] after clear impersonation\n");
//
//     /* Initialize the boot token */
//     _SE_TRACE("[SE] before ObInitFastRef(NULL)\n");
//     ObInitializeFastReference(&PsGetCurrentProcess()->Token, NULL);
//     _SE_TRACE("[SE] after ObInitFastRef(NULL)\n");
//     {
//         PTOKEN _tok = SepCreateSystemProcessToken();
//         _SE_TRACE("[SE] after SepCreateSystemProcessToken\n");
//         ObInitializeFastReference(&PsGetCurrentProcess()->Token, _tok);
//     }
//     _SE_TRACE("[SE] after boot token\n");
//
//     /* Initialise the anonymous logon tokens */
//     SeAnonymousLogonToken = SepCreateSystemAnonymousLogonToken();
//     if (!SeAnonymousLogonToken)
//         return FALSE;
//     _SE_TRACE("[SE] after anon token\n");
//
//     SeAnonymousLogonTokenNoEveryone = SepCreateSystemAnonymousLogonTokenNoEveryone();
//     if (!SeAnonymousLogonTokenNoEveryone)
//         return FALSE;
//     _SE_TRACE("[SE] after anon token no everyone\n");
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Handles the phase 1 procedure of the SRM initialization.
//  *
//  * @return
//  * Returns TRUE if the phase 1 initialization has succeeded, FALSE
//  * otherwise.
//  */
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// SepInitializationPhase1(VOID)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING Name;
//     HANDLE SecurityHandle;
//     HANDLE EventHandle;
//     NTSTATUS Status;
//     SECURITY_DESCRIPTOR SecurityDescriptor;
//     PACL Dacl;
//     ULONG DaclLength;
//
//     PAGED_CODE();
//
//     /* Insert the system token into the tree */
//     Status = ObInsertObject((PVOID)(PsGetCurrentProcess()->Token.Value &
//                                     ~MAX_FAST_REFS),
//                             NULL,
//                             0,
//                             0,
//                             NULL,
//                             NULL);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Create a security descriptor for the directory */
//     RtlCreateSecurityDescriptor(&SecurityDescriptor, SECURITY_DESCRIPTOR_REVISION);
//
//     /* Setup the ACL */
//     DaclLength = sizeof(ACL) + 3 * sizeof(ACCESS_ALLOWED_ACE) +
//                  RtlLengthSid(SeLocalSystemSid) +
//                  RtlLengthSid(SeAliasAdminsSid) +
//                  RtlLengthSid(SeWorldSid);
//     Dacl = ExAllocatePoolWithTag(NonPagedPool, DaclLength, TAG_SE);
//     if (Dacl == NULL)
//     {
//         return FALSE;
//     }
//
//     Status = RtlCreateAcl(Dacl, DaclLength, ACL_REVISION as ULONG);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Grant full access to SYSTEM */
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     DIRECTORY_ALL_ACCESS,
//                                     SeLocalSystemSid);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Allow admins to traverse and query */
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     READ_CONTROL | DIRECTORY_TRAVERSE | DIRECTORY_QUERY,
//                                     SeAliasAdminsSid);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Allow anyone to traverse */
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     DIRECTORY_TRAVERSE,
//                                     SeWorldSid);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* And link ACL and SD */
//     Status = RtlSetDaclSecurityDescriptor(&SecurityDescriptor, TRUE, Dacl, FALSE);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Create '\Security' directory */
//     RtlInitUnicodeString(&Name, L"\\Security");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Name,
//                                OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
//                                0,
//                                &SecurityDescriptor);
//
//     Status = ZwCreateDirectoryObject(&SecurityHandle,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Free the DACL */
//     ExFreePoolWithTag(Dacl, TAG_SE);
//
//     /* Create 'LSA_AUTHENTICATION_INITIALIZED' event */
//     RtlInitUnicodeString(&Name, L"LSA_AUTHENTICATION_INITIALIZED");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Name,
//                                OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
//                                SecurityHandle,
//                                SePublicDefaultSd);
//
//     Status = ZwCreateEvent(&EventHandle,
//                            GENERIC_WRITE,
//                            &ObjectAttributes,
//                            NotificationEvent,
//                            FALSE);
//     ASSERT(NT_SUCCESS(Status));
//
//     Status = ZwClose(EventHandle);
//     ASSERT(NT_SUCCESS(Status));
//
//     Status = ZwClose(SecurityHandle);
//     ASSERT(NT_SUCCESS(Status));
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Main security manager initialization function.
//  *
//  * @return
//  * Returns a boolean value according to the phase initialization
//  * routine that handles it. If TRUE, the routine deems the initialization
//  * phase as complete, FALSE otherwise.
//  */
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// SeInitSystem(VOID)
// {
//     /* Check the initialization phase */
//     switch (ExpInitializationPhase)
//     {
//         case 0:
//
//             /* Do Phase 0 */
//             return SepInitializationPhase0();
//
//         case 1:
//
//             /* Do Phase 1 */
//             return SepInitializationPhase1();
//
//         default:
//
//             /* Don't know any other phase! Bugcheck! */
//             KeBugCheckEx(UNEXPECTED_INITIALIZATION_CALL,
//                          0,
//                          ExpInitializationPhase,
//                          0,
//                          0);
//             return FALSE;
//     }
// }
//
// /**
//  * @brief
//  * Internal function that is responsible for querying, deleting, assigning and
//  * setting a security descriptor for an object in the NT kernel. It is the default
//  * security method for objects regarding the security context of objects.
//  *
//  * @param[in] Object
//  * The object that has the default security method, which the function has been
//  * called upon.
//  *
//  * @param[in] OperationType
//  * Operation type to perform to that object.
//  *
//  * @param[in] SecurityInformation
//  * Auxiliary security information of the object.
//  *
//  * @param[in,out] SecurityDescriptor
//  * A security descriptor. This SD is used accordingly to the operation type
//  * requested by the caller.
//  *
//  * @param[in,out] ReturnLength
//  * The length size of the queried security descriptor, in bytes.
//  *
//  * @param[in,out] OldSecurityDescriptor
//  * The old SD that belonged to the object, in case we're either deleting
//  * or replacing it.
//  *
//  * @param[in] PoolType
//  * Pool type allocation for the security descriptor.
//  *
//  * @param[in] GenericMapping
//  * The generic mapping of access rights masks for the object.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the specific operation tasked has been
//  * completed. Otherwise a failure NTSTATUS code is returned.
//  */
// NTSTATUS
// NTAPI
// SeDefaultObjectMethod(
//     _In_ PVOID Object,
//     _In_ SECURITY_OPERATION_CODE OperationType,
//     _In_ PSECURITY_INFORMATION SecurityInformation,
//     _Inout_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _Inout_opt_ PULONG ReturnLength,
//     _Inout_ PSECURITY_DESCRIPTOR *OldSecurityDescriptor,
//     _In_ POOL_TYPE PoolType,
//     _In_ PGENERIC_MAPPING GenericMapping)
// {
//     PAGED_CODE();
//
//     /* Select the operation type */
//     switch (OperationType)
//     {
//             /* Setting a new descriptor */
//         case SetSecurityDescriptor:
//
//             /* Sanity check */
//             ASSERT((PoolType == PagedPool) || (PoolType == NonPagedPool));
//
//             /* Set the information */
//             return ObSetSecurityDescriptorInfo(Object,
//                                                SecurityInformation,
//                                                SecurityDescriptor,
//                                                OldSecurityDescriptor,
//                                                PoolType,
//                                                GenericMapping);
//
//         case QuerySecurityDescriptor:
//
//             /* Query the information */
//             return ObQuerySecurityDescriptorInfo(Object,
//                                                  SecurityInformation,
//                                                  SecurityDescriptor,
//                                                  ReturnLength,
//                                                  OldSecurityDescriptor);
//
//         case DeleteSecurityDescriptor:
//
//             /* De-assign it */
//             return ObDeassignSecurity(OldSecurityDescriptor);
//
//         case AssignSecurityDescriptor:
//
//             /* Assign it */
//             ObAssignObjectSecurityDescriptor(Object, SecurityDescriptor, PoolType);
//             return STATUS_SUCCESS;
//
//         default:
//
//             /* Bug check */
//             KeBugCheckEx(SECURITY_SYSTEM, 0, STATUS_INVALID_PARAMETER, 0, 0);
//     }
//
//     /* Should never reach here */
//     ASSERT(FALSE);
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Queries the access mask from a security information context.
//  *
//  * @param[in] SecurityInformation
//  * The security information context where the access mask is to be
//  * gathered.
//  *
//  * @param[out] DesiredAccess
//  * The queried access mask right.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeQuerySecurityAccessMask(
//     _In_ SECURITY_INFORMATION SecurityInformation,
//     _Out_ PACCESS_MASK DesiredAccess)
// {
//     *DesiredAccess = 0;
//
//     if (SecurityInformation & (OWNER_SECURITY_INFORMATION |
//                                GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION))
//     {
//         *DesiredAccess |= READ_CONTROL;
//     }
//
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//     {
//         *DesiredAccess |= ACCESS_SYSTEM_SECURITY;
//     }
// }
//
// /**
//  * @brief
//  * Sets the access mask for a security information context.
//  *
//  * @param[in] SecurityInformation
//  * The security information context to apply a new access right.
//  *
//  * @param[out] DesiredAccess
//  * The returned access mask right.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeSetSecurityAccessMask(
//     _In_ SECURITY_INFORMATION SecurityInformation,
//     _Out_ PACCESS_MASK DesiredAccess)
// {
//     *DesiredAccess = 0;
//
//     if (SecurityInformation & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION))
//     {
//         *DesiredAccess |= WRITE_OWNER;
//     }
//
//     if (SecurityInformation & DACL_SECURITY_INFORMATION)
//     {
//         *DesiredAccess |= WRITE_DAC;
//     }
//
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//     {
//         *DesiredAccess |= ACCESS_SYSTEM_SECURITY;
//     }
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Report a security event to the security manager.
//  *
//  * @param[in] Flags
//  * Flags that influence how the event should be reported.
//  *
//  * @param[in] SourceName
//  * A Unicode string that represents the source name of the event.
//  *
//  * @param[in] UserSid
//  * The SID that represents a user that initiated the reporting.
//  *
//  * @param[in] AuditParameters
//  * An array of parameters for auditing purposes. This is used
//  * for reporting the event which the security manager will take
//  * care subsequently of doing eventual security auditing.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the security event has been reported.
//  * STATUS_INVALID_PARAMETER is returned if one of the parameters
//  * do not satisfy the requirements expected by the function.
//  */
// NTSTATUS
// NTAPI
// SeReportSecurityEvent(
//     _In_ ULONG Flags,
//     _In_ PUNICODE_STRING SourceName,
//     _In_opt_ PSID UserSid,
//     _In_ PSE_ADT_PARAMETER_ARRAY AuditParameters)
// {
//     SECURITY_SUBJECT_CONTEXT SubjectContext;
//     PTOKEN EffectiveToken;
//     PISID Sid;
//     NTSTATUS Status;
//
//     /* Validate parameters */
//     if ((Flags != 0) ||
//         (SourceName == NULL) ||
//         (SourceName->Buffer == NULL) ||
//         (SourceName->Length == 0) ||
//         (AuditParameters == NULL) ||
//         (AuditParameters->ParameterCount > SE_MAX_AUDIT_PARAMETERS - 4))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Validate the source name */
//     Status = RtlValidateUnicodeString(0, SourceName);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Check if we have a user SID */
//     if (UserSid != NULL)
//     {
//         /* Validate it */
//         if (!RtlValidSid(UserSid))
//         {
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         /* Use the user SID */
//         Sid = UserSid;
//     }
//     else
//     {
//         /* No user SID, capture the security subject context */
//         SeCaptureSubjectContext(&SubjectContext);
//
//         /* Extract the effective token */
//         EffectiveToken = SubjectContext.ClientToken ?
//             SubjectContext.ClientToken : SubjectContext.PrimaryToken;
//
//         /* Use the user-and-groups SID */
//         Sid = EffectiveToken->UserAndGroups->Sid;
//     }
//
//     UNIMPLEMENTED;
//
//     /* Check if we captured the subject context */
//     if (Sid != UserSid)
//     {
//         /* Release it */
//         SeReleaseSubjectContext(&SubjectContext);
//     }
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Sets an array of audit parameters for later security auditing use.
//  *
//  * @param[in,out] AuditParameters
//  * An array of audit parameters to be set.
//  *
//  * @param[in] Type
//  * The type of audit parameters to be set.
//  *
//  * @param[in] Index
//  * Index number that represents an instance of an audit parameters.
//  * Such index must be within the maximum range of audit parameters.
//  *
//  * @param[in] Data
//  * An arbitrary buffer data that is bounds to what kind of audit parameter
//  * type must be set.
//  *
//  * @return
//  * To be added...
//  */
// _Const_
// NTSTATUS
// NTAPI
// SeSetAuditParameter(
//     _Inout_ PSE_ADT_PARAMETER_ARRAY AuditParameters,
//     _In_ SE_ADT_PARAMETER_TYPE Type,
//     _In_range_(<, SE_MAX_AUDIT_PARAMETERS) ULONG Index,
//     _In_reads_(_Inexpressible_("depends on SE_ADT_PARAMETER_TYPE")) PVOID Data)
// {
//     UNIMPLEMENTED;
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
