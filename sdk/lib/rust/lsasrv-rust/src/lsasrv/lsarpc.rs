use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::prelude::CreateEventW;

// DsSetupInit is internal lsasrv (C: `VOID DsSetupInit(VOID)` — cdecl, NOT
// WINAPI), defined in dssetup.rs as `extern "C"`. Declaring it extern "system"
// (stdcall, _DsSetupInit@0) here is an ABI/decoration mismatch vs the cdecl
// definition (_DsSetupInit) → i386 call diverges. Must be extern "C".
extern "C" {
    fn DsSetupInit();
}
extern "system" {
    fn GetLastError() -> u32;
    fn I_RpcMapWin32Status(Status: i32) -> NTSTATUS;
    fn RpcServerListen(MinimumCallThreads: u32, MaxCalls: u32, DontWait: u32) -> i32;
    fn RpcServerRegisterIf( IfSpec: *mut core::ffi::c_void, MgrTypeUuid: *mut core::ffi::c_void, MgrEpv: *mut core::ffi::c_void, ) -> i32;
    fn RpcServerUseProtseqEpW( Protseq: PWSTR, MaxCalls: u32, Endpoint: PWSTR, SecurityDescriptor: *mut core::ffi::c_void, ) -> i32;
    fn SetEvent(hEvent: *mut core::ffi::c_void) -> i32;
    static lsarpc_v0_0_s_ifspec: *mut core::ffi::c_void;
}

// ---- cross-module LSA helpers (sibling Rust translations; #[no_mangle]) ----
use crate::lsasrv::database::{
    LsapCloseDbObject, LsapCreateDbObject, LsapDeleteDbObject, LsapDeleteObjectAttribute,
    LsapGetObjectAttribute, LsapOpenDbObject, LsapSetObjectAttribute, LsapValidateDbObject,
};
use crate::lsasrv::security::{LsapCreateAccountSd, LsapCreateSecretSd};
use crate::lsasrv::lookup::{LsapGetRelativeIdFromSid, LsapLookupNames, LsapLookupSids};
use crate::lsasrv::privileges::{
    LsapLookupAccountRightName, LsapLookupAccountRightValue, LsarpEnumeratePrivileges,
    LsarpLookupPrivilegeDisplayName, LsarpLookupPrivilegeName, LsarpLookupPrivilegeValue,
};
use crate::lsasrv::notify::LsapNotifyPolicyChange;
use crate::lsasrv::registry::{
    LsapRegCloseKey, LsapRegEnumerateSubKey, LsapRegOpenKey, LsapRegQueryKeyInfo, LsapRegQueryValue,
};
use crate::lsasrv::policy::{


    LsarQueryAccountDomain, LsarQueryAuditEvents, LsarQueryAuditFull, LsarQueryAuditLog,
    LsarQueryDefaultQuota, LsarQueryDnsDomain, LsarQueryDnsDomainInt, LsarQueryLocalAccountDomain,
    LsarQueryModification, LsarQueryPdAccount, LsarQueryPrimaryDomain, LsarQueryReplicaSource,
    LsarQueryServerRole, LsarSetAccountDomain, LsarSetAuditEvents, LsarSetAuditFull, LsarSetAuditLog,
    LsarSetDefaultQuota, LsarSetDnsDomain, LsarSetDnsDomainInt, LsarSetLocalAccountDomain,
    LsarSetModification, LsarSetPrimaryDomain, LsarSetReplicaSource, LsarSetServerRole,
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


// ---- Win32 / RPC-runtime imports (WINAPI/__stdcall) ----
extern "system" {
    // advapi32: ConvertSidToStringSid → ...W (UNICODE macro); GetLengthSid.
    #[link_name = "ConvertSidToStringSidW"]
    fn ConvertSidToStringSid(Sid: PSID, StringSid: *mut LPWSTR) -> BOOL;
    fn GetLengthSid(pSid: PSID) -> DWORD;
    // kernel32
    fn LocalFree(hMem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    // rpcrt4 (RPC_STATUS = long)
    fn RpcImpersonateClient(BindingHandle: handle_t) -> i32;
    fn RpcRevertToSelf() -> i32;
}

// ---- file-scope preamble (translated) ----
static mut PolicyHandleTableLock: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };

pub static mut LsapPolicyMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: POLICY_READ,
    GenericWrite: POLICY_WRITE,
    GenericExecute: POLICY_EXECUTE,
    GenericAll: POLICY_ALL_ACCESS,
};

pub static mut LsapAccountMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: ACCOUNT_READ,
    GenericWrite: ACCOUNT_WRITE,
    GenericExecute: ACCOUNT_EXECUTE,
    GenericAll: ACCOUNT_ALL_ACCESS,
};

pub static mut LsapSecretMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: SECRET_READ,
    GenericWrite: SECRET_WRITE,
    GenericExecute: SECRET_EXECUTE,
    GenericAll: SECRET_ALL_ACCESS,
};
#[no_mangle]
pub unsafe extern "C" fn LsarStartRpcServer() -> NTSTATUS {

    const RPC_S_OK: i32 = 0;
    const RPC_C_PROTSEQ_MAX_REQS_DEFAULT: u32 = 10;

    let mut Status: i32;
    let dwError: u32;
    let hEvent: *mut core::ffi::c_void;

    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(PolicyHandleTableLock));

    Status = RpcServerUseProtseqEpW(
        w!("ncacn_np").as_ptr() as PWSTR,
        RPC_C_PROTSEQ_MAX_REQS_DEFAULT,
        w!("\\pipe\\lsarpc").as_ptr() as PWSTR,
        core::ptr::null_mut(),
    );
    if Status != RPC_S_OK {
        return I_RpcMapWin32Status(Status);
    }

    Status = RpcServerRegisterIf(
        lsarpc_v0_0_s_ifspec,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Status != RPC_S_OK {
        return I_RpcMapWin32Status(Status);
    }

    DsSetupInit();

    Status = RpcServerListen(1, 20, 1);
    if Status != RPC_S_OK {
        return I_RpcMapWin32Status(Status);
    }

    hEvent = CreateEventW(
        core::ptr::null_mut(),
        1,
        0,
        w!("LSA_RPC_SERVER_ACTIVE").as_ptr() as PWSTR,
    );
    if hEvent.is_null() {
        dwError = GetLastError();
        let _ = dwError;
        return STATUS_UNSUCCESSFUL;
    }

    SetEvent(hEvent);

    STATUS_SUCCESS
}
// __RPC_USER (= __stdcall on i386) context-handle rundown routine → extern "system".
#[no_mangle]
pub unsafe extern "system" fn LSAPR_HANDLE_rundown(_hHandle: LSAPR_HANDLE) {
}

#[no_mangle]
pub unsafe extern "system" fn LsarClose(
    ObjectHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    let mut DbObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(b"LsarClose(%p)\n\0".as_ptr() as *const i8, ObjectHandle);

    Status = LsapValidateDbObject(
        *ObjectHandle as PLSA_DB_OBJECT,
        LsaDbIgnoreObject,
        0,
        &mut DbObject,
    );
    if Status == STATUS_SUCCESS {
        Status = LsapCloseDbObject(DbObject);
        *ObjectHandle = core::ptr::null_mut();
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarDelete(
    ObjectHandle: LSAPR_HANDLE,
) -> NTSTATUS {
    let mut ObjectHandle = ObjectHandle;
    LsarDeleteObject(&mut ObjectHandle)
}

#[no_mangle]
pub unsafe extern "system" fn LsarEnumeratePrivileges(
    PolicyHandle: LSAPR_HANDLE,
    EnumerationContext: *mut DWORD,
    EnumerationBuffer: PLSAPR_PRIVILEGE_ENUM_BUFFER,
    PreferedMaximumLength: DWORD,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let Status: NTSTATUS;

    TRACE!(
        b"LsarEnumeratePrivileges(%p %p %p %lu)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        EnumerationContext,
        EnumerationBuffer,
        PreferedMaximumLength,
    );

    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        POLICY_VIEW_LOCAL_INFORMATION,
        core::ptr::addr_of_mut!(PolicyObject),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if EnumerationContext.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    LsarpEnumeratePrivileges(EnumerationContext, EnumerationBuffer, PreferedMaximumLength)
}

#[no_mangle]
pub unsafe extern "system" fn LsarQuerySecurityObject(
    ObjectHandle: LSAPR_HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: *mut PLSAPR_SR_SECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut DbObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let RelativeSd: PISECURITY_DESCRIPTOR_RELATIVE;
    let mut ResultSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let SdData: PLSAPR_SR_SECURITY_DESCRIPTOR;
    let mut DesiredAccess: ACCESS_MASK = 0;
    let mut RelativeSdSize: ULONG = 0;
    let mut ResultSdSize: ULONG = 0;
    let mut Status: NTSTATUS;

    if SecurityDescriptor.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *SecurityDescriptor = core::ptr::null_mut();

    if (SecurityInformation & OWNER_SECURITY_INFORMATION) != 0
        || (SecurityInformation & GROUP_SECURITY_INFORMATION) != 0
        || (SecurityInformation & DACL_SECURITY_INFORMATION) != 0
    {
        DesiredAccess |= READ_CONTROL;
    }

    if (SecurityInformation & SACL_SECURITY_INFORMATION) != 0 {
        DesiredAccess |= ACCESS_SYSTEM_SECURITY;
    }

    Status = LsapValidateDbObject(
        ObjectHandle as PLSA_DB_OBJECT,
        LsaDbIgnoreObject,
        DesiredAccess,
        &mut DbObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LsapGetObjectAttribute(
        DbObject,
        w!("SecDesc").as_ptr() as PWSTR,
        core::ptr::null_mut(),
        &mut RelativeSdSize,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RelativeSd = MIDL_user_allocate(RelativeSdSize as usize) as PISECURITY_DESCRIPTOR_RELATIVE;
    if RelativeSd.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Status = LsapGetObjectAttribute(
            DbObject,
            w!("SecDesc").as_ptr() as PWSTR,
            RelativeSd as *mut core::ffi::c_void,
            &mut RelativeSdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if (SecurityInformation & OWNER_SECURITY_INFORMATION) == 0 {
            (*RelativeSd).Owner = 0;
        }

        if (SecurityInformation & GROUP_SECURITY_INFORMATION) == 0 {
            (*RelativeSd).Group = 0;
        }

        if (SecurityInformation & DACL_SECURITY_INFORMATION) == 0 {
            (*RelativeSd).Control &= !(SE_DACL_PRESENT as u16);
        }

        if (SecurityInformation & SACL_SECURITY_INFORMATION) == 0 {
            (*RelativeSd).Control &= !(SE_SACL_PRESENT as u16);
        }

        Status = RtlMakeSelfRelativeSD(
            RelativeSd as PSECURITY_DESCRIPTOR,
            core::ptr::null_mut(),
            &mut ResultSdSize,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        ResultSd = MIDL_user_allocate(ResultSdSize as usize) as PSECURITY_DESCRIPTOR;
        if ResultSd.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlMakeSelfRelativeSD(
            RelativeSd as PSECURITY_DESCRIPTOR,
            ResultSd,
            &mut ResultSdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        SdData = MIDL_user_allocate(core::mem::size_of::<LSAPR_SR_SECURITY_DESCRIPTOR>())
            as PLSAPR_SR_SECURITY_DESCRIPTOR;
        if SdData.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*SdData).Length = ResultSdSize;
        (*SdData).SecurityDescriptor = ResultSd as PBYTE;

        *SecurityDescriptor = SdData;
    }

    if !NT_SUCCESS(Status) {
        if !ResultSd.is_null() {
            MIDL_user_free(ResultSd as *mut core::ffi::c_void);
        }
    }

    if !RelativeSd.is_null() {
        MIDL_user_free(RelativeSd as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarSetSecurityObject(
    ObjectHandle: LSAPR_HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PLSAPR_SR_SECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut DbObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut DesiredAccess: ACCESS_MASK = 0;
    let mut RelativeSd: PSECURITY_DESCRIPTOR;
    let mut RelativeSdSize: ULONG = 0;
    let mut TokenHandle: HANDLE = core::ptr::null_mut();
    let Mapping: PGENERIC_MAPPING;
    let mut Status: NTSTATUS;

    if SecurityDescriptor.is_null()
        || (*SecurityDescriptor).SecurityDescriptor.is_null()
        || RtlValidSecurityDescriptor(
            (*SecurityDescriptor).SecurityDescriptor as PSECURITY_DESCRIPTOR,
        ) == 0
    {
        return ERROR_INVALID_PARAMETER as NTSTATUS;
    }

    if SecurityInformation == 0
        || (SecurityInformation
            & !(OWNER_SECURITY_INFORMATION
                | GROUP_SECURITY_INFORMATION
                | DACL_SECURITY_INFORMATION
                | SACL_SECURITY_INFORMATION))
            != 0
    {
        return ERROR_INVALID_PARAMETER as NTSTATUS;
    }

    if SecurityInformation & SACL_SECURITY_INFORMATION != 0 {
        DesiredAccess |= ACCESS_SYSTEM_SECURITY;
    }

    if SecurityInformation & DACL_SECURITY_INFORMATION != 0 {
        DesiredAccess |= WRITE_DAC;
    }

    if SecurityInformation & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION) != 0 {
        DesiredAccess |= WRITE_OWNER;
    }

    if SecurityInformation & OWNER_SECURITY_INFORMATION != 0
        && (*(SecurityDescriptor as PISECURITY_DESCRIPTOR)).Owner.is_null()
    {
        return ERROR_INVALID_PARAMETER as NTSTATUS;
    }

    if SecurityInformation & GROUP_SECURITY_INFORMATION != 0
        && (*(SecurityDescriptor as PISECURITY_DESCRIPTOR)).Group.is_null()
    {
        return ERROR_INVALID_PARAMETER as NTSTATUS;
    }

    Status = LsapValidateDbObject(
        ObjectHandle as PLSA_DB_OBJECT,
        LsaDbIgnoreObject,
        DesiredAccess,
        &mut DbObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    match (*DbObject).ObjectType {
        LsaDbPolicyObject => {
            Mapping = core::ptr::addr_of_mut!(LsapPolicyMapping);
        }
        LsaDbAccountObject => {
            Mapping = core::ptr::addr_of_mut!(LsapAccountMapping);
        }
        LsaDbSecretObject => {
            Mapping = core::ptr::addr_of_mut!(LsapSecretMapping);
        }
        _ => {
            return STATUS_INVALID_HANDLE;
        }
    }

    Status = LsapGetObjectAttribute(
        DbObject,
        w!("SecDesc").as_ptr() as PWSTR,
        core::ptr::null_mut(),
        &mut RelativeSdSize,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RelativeSd = RtlAllocateHeap(RtlGetProcessHeap(), 0, RelativeSdSize as SIZE_T);
    if RelativeSd.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Status = LsapGetObjectAttribute(
            DbObject,
            w!("SecDesc").as_ptr() as PWSTR,
            RelativeSd,
            &mut RelativeSdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if SecurityInformation & OWNER_SECURITY_INFORMATION != 0 {
            Status = I_RpcMapWin32Status(RpcImpersonateClient(core::ptr::null_mut()));
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = NtOpenThreadToken(
                (-2isize) as HANDLE,
                TOKEN_QUERY,
                1u8,
                &mut TokenHandle,
            );
            RpcRevertToSelf();
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        Status = RtlSetSecurityObject(
            SecurityInformation,
            (*SecurityDescriptor).SecurityDescriptor as PSECURITY_DESCRIPTOR,
            &mut RelativeSd,
            Mapping,
            TokenHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = LsapSetObjectAttribute(
            DbObject,
            w!("SecDesc").as_ptr() as PWSTR,
            RelativeSd,
            RtlLengthSecurityDescriptor(RelativeSd),
        );
    }

    if !TokenHandle.is_null() {
        NtClose(TokenHandle);
    }

    if !RelativeSd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarChangePassword(
    _IDL_handle: handle_t,
    _String1: PRPC_UNICODE_STRING,
    _String2: PRPC_UNICODE_STRING,
    _String3: PRPC_UNICODE_STRING,
    _String4: PRPC_UNICODE_STRING,
    _String5: PRPC_UNICODE_STRING,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarOpenPolicy(
    _SystemName: LPWSTR,
    _ObjectAttributes: PLSAPR_OBJECT_ATTRIBUTES,
    DesiredAccess: ACCESS_MASK,
    PolicyHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let Status: NTSTATUS;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(PolicyHandleTableLock));

    Status = LsapOpenDbObject(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        w!("Policy").as_ptr() as PWSTR,
        LsaDbPolicyObject,
        DesiredAccess,
        0u8,
        core::ptr::addr_of_mut!(PolicyObject),
    );

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(PolicyHandleTableLock));

    if NT_SUCCESS(Status) {
        *PolicyHandle = PolicyObject as LSAPR_HANDLE;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarQueryInformationPolicy(
    PolicyHandle: LSAPR_HANDLE,
    InformationClass: POLICY_INFORMATION_CLASS,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let DesiredAccess: ACCESS_MASK;
    let Status: NTSTATUS;

    match InformationClass {
        POLICY_INFORMATION_CLASS_PolicyAuditLogInformation
        | POLICY_INFORMATION_CLASS_PolicyAuditEventsInformation
        | POLICY_INFORMATION_CLASS_PolicyAuditFullQueryInformation => {
            DesiredAccess = POLICY_VIEW_AUDIT_INFORMATION;
        }

        POLICY_INFORMATION_CLASS_PolicyPrimaryDomainInformation
        | POLICY_INFORMATION_CLASS_PolicyAccountDomainInformation
        | POLICY_INFORMATION_CLASS_PolicyLsaServerRoleInformation
        | POLICY_INFORMATION_CLASS_PolicyReplicaSourceInformation
        | POLICY_INFORMATION_CLASS_PolicyDefaultQuotaInformation
        | POLICY_INFORMATION_CLASS_PolicyModificationInformation
        | POLICY_INFORMATION_CLASS_PolicyDnsDomainInformation
        | POLICY_INFORMATION_CLASS_PolicyDnsDomainInformationInt
        | POLICY_INFORMATION_CLASS_PolicyLocalAccountDomainInformation => {
            DesiredAccess = POLICY_VIEW_LOCAL_INFORMATION;
        }

        POLICY_INFORMATION_CLASS_PolicyPdAccountInformation => {
            DesiredAccess = POLICY_GET_PRIVATE_INFORMATION;
        }

        _ => {
            return STATUS_INVALID_PARAMETER;
        }
    }

    let validate_status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        DesiredAccess,
        &mut PolicyObject,
    );
    if !NT_SUCCESS(validate_status) {
        return validate_status;
    }

    match InformationClass {
        POLICY_INFORMATION_CLASS_PolicyAuditLogInformation => {
            Status = LsarQueryAuditLog(PolicyObject, PolicyInformation);
        }

        POLICY_INFORMATION_CLASS_PolicyAuditEventsInformation => {
            Status = LsarQueryAuditEvents(PolicyObject, PolicyInformation);
        }

        POLICY_INFORMATION_CLASS_PolicyPrimaryDomainInformation => {
            Status = LsarQueryPrimaryDomain(PolicyObject, PolicyInformation);
        }

        POLICY_INFORMATION_CLASS_PolicyPdAccountInformation => {
            Status = LsarQueryPdAccount(PolicyObject, PolicyInformation);
        }

        POLICY_INFORMATION_CLASS_PolicyAccountDomainInformation => {
            Status = LsarQueryAccountDomain(PolicyObject, PolicyInformation);
        }

        POLICY_INFORMATION_CLASS_PolicyLsaServerRoleInformation => {
            Status = LsarQueryServerRole(PolicyObject, PolicyInformation);
        }

        POLICY_INFORMATION_CLASS_PolicyReplicaSourceInformation => {
            Status = LsarQueryReplicaSource(PolicyObject, PolicyInformation);
        }

        POLICY_INFORMATION_CLASS_PolicyDefaultQuotaInformation => {
            Status = LsarQueryDefaultQuota(PolicyObject, PolicyInformation);
        }

        POLICY_INFORMATION_CLASS_PolicyModificationInformation => {
            Status = LsarQueryModification(PolicyObject, PolicyInformation);
        }

        POLICY_INFORMATION_CLASS_PolicyAuditFullQueryInformation => {
            Status = LsarQueryAuditFull(PolicyObject, PolicyInformation);
        }

        POLICY_INFORMATION_CLASS_PolicyDnsDomainInformation => {
            Status = LsarQueryDnsDomain(PolicyObject, PolicyInformation);
        }

        POLICY_INFORMATION_CLASS_PolicyDnsDomainInformationInt => {
            Status = LsarQueryDnsDomainInt(PolicyObject, PolicyInformation);
        }

        POLICY_INFORMATION_CLASS_PolicyLocalAccountDomainInformation => {
            Status = LsarQueryLocalAccountDomain(PolicyObject, PolicyInformation);
        }

        _ => {
            Status = STATUS_INVALID_PARAMETER;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarSetInformationPolicy(
    PolicyHandle: LSAPR_HANDLE,
    InformationClass: POLICY_INFORMATION_CLASS,
    PolicyInformation: PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let DesiredAccess: ACCESS_MASK;
    let mut Status: NTSTATUS;

    match InformationClass {
        POLICY_INFORMATION_CLASS_PolicyAuditLogInformation
        | POLICY_INFORMATION_CLASS_PolicyAuditFullSetInformation => {
            DesiredAccess = POLICY_AUDIT_LOG_ADMIN;
        }
        POLICY_INFORMATION_CLASS_PolicyAuditEventsInformation => {
            DesiredAccess = POLICY_SET_AUDIT_REQUIREMENTS;
        }
        POLICY_INFORMATION_CLASS_PolicyPrimaryDomainInformation
        | POLICY_INFORMATION_CLASS_PolicyAccountDomainInformation
        | POLICY_INFORMATION_CLASS_PolicyDnsDomainInformation
        | POLICY_INFORMATION_CLASS_PolicyDnsDomainInformationInt
        | POLICY_INFORMATION_CLASS_PolicyLocalAccountDomainInformation => {
            DesiredAccess = POLICY_TRUST_ADMIN;
        }
        POLICY_INFORMATION_CLASS_PolicyLsaServerRoleInformation
        | POLICY_INFORMATION_CLASS_PolicyReplicaSourceInformation => {
            DesiredAccess = POLICY_SERVER_ADMIN;
        }
        POLICY_INFORMATION_CLASS_PolicyDefaultQuotaInformation => {
            DesiredAccess = POLICY_SET_DEFAULT_QUOTA_LIMITS;
        }
        _ => {
            return STATUS_INVALID_PARAMETER;
        }
    }

    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        DesiredAccess,
        &mut PolicyObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    match InformationClass {
        POLICY_INFORMATION_CLASS_PolicyAuditLogInformation => {
            Status = LsarSetAuditLog(
                PolicyObject,
                PolicyInformation as PPOLICY_AUDIT_LOG_INFO,
            );
        }
        POLICY_INFORMATION_CLASS_PolicyAuditEventsInformation => {
            Status = LsarSetAuditEvents(
                PolicyObject,
                PolicyInformation as PLSAPR_POLICY_AUDIT_EVENTS_INFO,
            );
            if NT_SUCCESS(Status) {
                LsapNotifyPolicyChange(POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyAuditEventsInformation);
            }
        }
        POLICY_INFORMATION_CLASS_PolicyPrimaryDomainInformation => {
            Status = LsarSetPrimaryDomain(
                PolicyObject,
                PolicyInformation as PLSAPR_POLICY_PRIMARY_DOM_INFO,
            );
            if NT_SUCCESS(Status) {
                LsapNotifyPolicyChange(POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyDnsDomainInformation);
            }
        }
        POLICY_INFORMATION_CLASS_PolicyAccountDomainInformation => {
            Status = LsarSetAccountDomain(
                PolicyObject,
                PolicyInformation as PLSAPR_POLICY_ACCOUNT_DOM_INFO,
            );
            if NT_SUCCESS(Status) {
                LsapNotifyPolicyChange(POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyAccountDomainInformation);
            }
        }
        POLICY_INFORMATION_CLASS_PolicyLsaServerRoleInformation => {
            Status = LsarSetServerRole(
                PolicyObject,
                PolicyInformation as PPOLICY_LSA_SERVER_ROLE_INFO,
            );
            if NT_SUCCESS(Status) {
                LsapNotifyPolicyChange(POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyServerRoleInformation);
            }
        }
        POLICY_INFORMATION_CLASS_PolicyReplicaSourceInformation => {
            Status = LsarSetReplicaSource(
                PolicyObject,
                PolicyInformation as PPOLICY_LSA_REPLICA_SRCE_INFO,
            );
        }
        POLICY_INFORMATION_CLASS_PolicyDefaultQuotaInformation => {
            Status = LsarSetDefaultQuota(
                PolicyObject,
                PolicyInformation as PPOLICY_DEFAULT_QUOTA_INFO,
            );
        }
        POLICY_INFORMATION_CLASS_PolicyModificationInformation => {
            Status = LsarSetModification(
                PolicyObject,
                PolicyInformation as PPOLICY_MODIFICATION_INFO,
            );
        }
        POLICY_INFORMATION_CLASS_PolicyAuditFullSetInformation => {
            Status = LsarSetAuditFull(
                PolicyObject,
                PolicyInformation as PPOLICY_AUDIT_FULL_QUERY_INFO,
            );
        }
        POLICY_INFORMATION_CLASS_PolicyDnsDomainInformation => {
            Status = LsarSetDnsDomain(
                PolicyObject,
                PolicyInformation as PLSAPR_POLICY_DNS_DOMAIN_INFO,
            );
            if NT_SUCCESS(Status) {
                LsapNotifyPolicyChange(POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyDnsDomainInformation);
            }
        }
        POLICY_INFORMATION_CLASS_PolicyDnsDomainInformationInt => {
            Status = LsarSetDnsDomainInt(
                PolicyObject,
                PolicyInformation as PLSAPR_POLICY_DNS_DOMAIN_INFO,
            );
        }
        POLICY_INFORMATION_CLASS_PolicyLocalAccountDomainInformation => {
            Status = LsarSetLocalAccountDomain(
                PolicyObject,
                PolicyInformation as PLSAPR_POLICY_ACCOUNT_DOM_INFO,
            );
        }
        _ => {
            Status = STATUS_INVALID_PARAMETER;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarClearAuditLog(
    _ObjectHandle: LSAPR_HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn LsarpCreateAccount(
    PolicyObject: PLSA_DB_OBJECT,
    AccountSid: PRPC_SID,
    DesiredAccess: ACCESS_MASK,
    AccountObject: *mut PLSA_DB_OBJECT,
) -> NTSTATUS {
    let mut SidString: LPWSTR = core::ptr::null_mut();
    let mut AccountSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut AccountSdSize: ULONG = 0;
    let mut Status: NTSTATUS;

    if ConvertSidToStringSid(AccountSid as PSID, &mut SidString) == 0 {
        ERR!("ConvertSidToStringSid failed\n");
        return STATUS_INVALID_PARAMETER;
    }

    'done: {
        Status = LsapCreateAccountSd(&mut AccountSd, &mut AccountSdSize);
        if !NT_SUCCESS(Status) {
            ERR!("LsapCreateAccountSd returned 0x%08lx\n", Status);
            break 'done;
        }

        Status = LsapCreateDbObject(
            PolicyObject,
            crate::wide!("Accounts").as_ptr() as PWSTR,
            SidString,
            LsaDbAccountObject,
            DesiredAccess,
            (*PolicyObject).Trusted,
            AccountObject,
        );
        if !NT_SUCCESS(Status) {
            ERR!("LsapCreateDbObject failed (Status 0x%08lx)\n", Status);
            break 'done;
        }

        Status = LsapSetObjectAttribute(
            *AccountObject,
            crate::wide!("Sid").as_ptr() as PWSTR,
            AccountSid as PVOID,
            GetLengthSid(AccountSid as PSID),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = LsapSetObjectAttribute(
            *AccountObject,
            crate::wide!("SecDesc").as_ptr() as PWSTR,
            AccountSd,
            AccountSdSize,
        );
    }

    if !SidString.is_null() {
        LocalFree(SidString as PVOID);
    }

    if !AccountSd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AccountSd);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarCreateAccount(
    PolicyHandle: LSAPR_HANDLE,
    AccountSid: PRPC_SID,
    DesiredAccess: ACCESS_MASK,
    AccountHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut AccountObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if RtlValidSid(AccountSid as PSID) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        POLICY_CREATE_ACCOUNT,
        &mut PolicyObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LsarpCreateAccount(PolicyObject, AccountSid, DesiredAccess, &mut AccountObject);
    if NT_SUCCESS(Status) {
        *AccountHandle = AccountObject as LSAPR_HANDLE;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarEnumerateAccounts(
    PolicyHandle: LSAPR_HANDLE,
    EnumerationContext: *mut DWORD,
    EnumerationBuffer: PLSAPR_ACCOUNT_ENUM_BUFFER,
    PreferedMaximumLength: DWORD,
) -> NTSTATUS {
    let mut EnumBuffer: LSAPR_ACCOUNT_ENUM_BUFFER = LSAPR_ACCOUNT_ENUM_BUFFER {
        EntriesRead: 0,
        Information: core::ptr::null_mut(),
    };
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let AccountKeyBuffer: PWSTR;
    let mut AccountsKeyHandle: HANDLE = core::ptr::null_mut();
    let mut AccountKeyHandle: HANDLE = core::ptr::null_mut();
    let mut SidKeyHandle: HANDLE = core::ptr::null_mut();
    let mut AccountKeyBufferSize: ULONG = 0;
    let mut EnumIndex: ULONG;
    let mut EnumCount: ULONG;
    let mut RequiredLength: ULONG;
    let mut DataLength: ULONG;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    if EnumerationContext.is_null() || EnumerationBuffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    (*EnumerationBuffer).EntriesRead = 0;
    (*EnumerationBuffer).Information = core::ptr::null_mut();

    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        POLICY_VIEW_LOCAL_INFORMATION,
        &mut PolicyObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LsapRegOpenKey(
        (*PolicyObject).KeyHandle,
        w!("Accounts").as_ptr() as PWSTR,
        KEY_READ,
        &mut AccountsKeyHandle,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LsapRegQueryKeyInfo(
        AccountsKeyHandle,
        core::ptr::null_mut(),
        &mut AccountKeyBufferSize,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    AccountKeyBufferSize += core::mem::size_of::<WCHAR>() as ULONG;
    AccountKeyBuffer =
        RtlAllocateHeap(RtlGetProcessHeap(), 0, AccountKeyBufferSize as SIZE_T) as PWSTR;
    if AccountKeyBuffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    EnumIndex = *EnumerationContext;
    EnumCount = 0;
    RequiredLength = 0;

    loop {
        Status = LsapRegEnumerateSubKey(
            AccountsKeyHandle,
            EnumIndex,
            AccountKeyBufferSize,
            AccountKeyBuffer,
        );
        if !NT_SUCCESS(Status) {
            break;
        }

        Status = LsapRegOpenKey(
            AccountsKeyHandle,
            AccountKeyBuffer,
            KEY_READ,
            &mut AccountKeyHandle,
        );
        if NT_SUCCESS(Status) {
            Status = LsapRegOpenKey(
                AccountKeyHandle,
                w!("Sid").as_ptr() as PWSTR,
                KEY_READ,
                &mut SidKeyHandle,
            );
            if NT_SUCCESS(Status) {
                DataLength = 0;
                Status = LsapRegQueryValue(
                    SidKeyHandle,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &mut DataLength,
                );
                if NT_SUCCESS(Status) {
                    if (RequiredLength
                        + DataLength
                        + core::mem::size_of::<LSAPR_ACCOUNT_INFORMATION>() as ULONG)
                        > PreferedMaximumLength
                    {
                        break;
                    }

                    RequiredLength +=
                        DataLength + core::mem::size_of::<LSAPR_ACCOUNT_INFORMATION>() as ULONG;
                    EnumCount += 1;
                }

                LsapRegCloseKey(SidKeyHandle);
            }

            LsapRegCloseKey(AccountKeyHandle);
        }

        EnumIndex += 1;
    }

    EnumBuffer.EntriesRead = EnumCount;
    EnumBuffer.Information = midl_user_allocate(
        EnumCount as usize * core::mem::size_of::<LSAPR_ACCOUNT_INFORMATION>(),
    ) as *mut LSAPR_ACCOUNT_INFORMATION;

    'done: {
        if EnumBuffer.Information.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        EnumIndex = *EnumerationContext;
        i = 0;
        while i < EnumCount {
            Status = LsapRegEnumerateSubKey(
                AccountsKeyHandle,
                EnumIndex,
                AccountKeyBufferSize,
                AccountKeyBuffer,
            );
            if !NT_SUCCESS(Status) {
                break;
            }

            Status = LsapRegOpenKey(
                AccountsKeyHandle,
                AccountKeyBuffer,
                KEY_READ,
                &mut AccountKeyHandle,
            );
            if NT_SUCCESS(Status) {
                Status = LsapRegOpenKey(
                    AccountKeyHandle,
                    w!("Sid").as_ptr() as PWSTR,
                    KEY_READ,
                    &mut SidKeyHandle,
                );
                if NT_SUCCESS(Status) {
                    DataLength = 0;
                    Status = LsapRegQueryValue(
                        SidKeyHandle,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        &mut DataLength,
                    );
                    if NT_SUCCESS(Status) {
                        (*EnumBuffer.Information.add(i as usize)).Sid =
                            midl_user_allocate(DataLength as usize) as PLSAPR_SID;
                        if (*EnumBuffer.Information.add(i as usize)).Sid.is_null() {
                            LsapRegCloseKey(AccountKeyHandle);
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                            break 'done;
                        }

                        Status = LsapRegQueryValue(
                            SidKeyHandle,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            (*EnumBuffer.Information.add(i as usize)).Sid as PVOID,
                            &mut DataLength,
                        );
                    }

                    LsapRegCloseKey(SidKeyHandle);
                }

                LsapRegCloseKey(AccountKeyHandle);

                if !NT_SUCCESS(Status) {
                    break 'done;
                }
            }

            i += 1;
            EnumIndex += 1;
        }

        if NT_SUCCESS(Status) {
            *EnumerationContext += EnumCount;
            (*EnumerationBuffer).EntriesRead = EnumBuffer.EntriesRead;
            (*EnumerationBuffer).Information = EnumBuffer.Information;
        }
    }

    if !NT_SUCCESS(Status) {
        if !EnumBuffer.Information.is_null() {
            i = 0;
            while i < EnumBuffer.EntriesRead {
                if !(*EnumBuffer.Information.add(i as usize)).Sid.is_null() {
                    midl_user_free((*EnumBuffer.Information.add(i as usize)).Sid as PVOID);
                }
                i += 1;
            }

            midl_user_free(EnumBuffer.Information as PVOID);
        }
    }

    if !AccountKeyBuffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AccountKeyBuffer as PVOID);
    }

    if !AccountsKeyHandle.is_null() {
        LsapRegCloseKey(AccountsKeyHandle);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarCreateTrustedDomain(
    _PolicyHandle: LSAPR_HANDLE,
    _TrustedDomainInformation: PLSAPR_TRUST_INFORMATION,
    _DesiredAccess: ACCESS_MASK,
    _TrustedDomainHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    if LsapProductType != NtProductLanManNt {
        return STATUS_DIRECTORY_SERVICE_REQUIRED;
    }

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarEnumerateTrustedDomains(
    _PolicyHandle: LSAPR_HANDLE,
    _EnumerationContext: *mut DWORD,
    EnumerationBuffer: PLSAPR_TRUSTED_ENUM_BUFFER,
    _PreferedMaximumLength: DWORD,
) -> NTSTATUS {
    (*EnumerationBuffer).EntriesRead = 0;
    (*EnumerationBuffer).Information = core::ptr::null_mut();
    STATUS_NO_MORE_ENTRIES
}

#[no_mangle]
pub unsafe extern "system" fn LsarLookupNames(
    _PolicyHandle: LSAPR_HANDLE,
    Count: DWORD,
    Names: PRPC_UNICODE_STRING,
    ReferencedDomains: *mut PLSAPR_REFERENCED_DOMAIN_LIST,
    TranslatedSids: PLSAPR_TRANSLATED_SIDS,
    LookupLevel: LSAP_LOOKUP_LEVEL,
    MappedCount: *mut DWORD,
) -> NTSTATUS {
    let mut TranslatedSidsEx2: LSAPR_TRANSLATED_SIDS_EX2 = core::mem::zeroed();
    let mut i: ULONG;
    let Status: NTSTATUS;

    TRACE!(
        b"LsarLookupNames(%p %lu %p %p %p %d %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        Count,
        Names,
        ReferencedDomains,
        TranslatedSids,
        LookupLevel,
        MappedCount,
    );

    (*TranslatedSids).Entries = 0;
    (*TranslatedSids).Sids = core::ptr::null_mut();
    *ReferencedDomains = core::ptr::null_mut();

    if Count == 0 {
        return STATUS_NONE_MAPPED;
    }

    TranslatedSidsEx2.Entries = 0;
    TranslatedSidsEx2.Sids = core::ptr::null_mut();

    Status = LsapLookupNames(
        Count,
        Names,
        ReferencedDomains,
        &mut TranslatedSidsEx2,
        LookupLevel,
        MappedCount,
        0,
        0,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*TranslatedSids).Entries = TranslatedSidsEx2.Entries;
    (*TranslatedSids).Sids = MIDL_user_allocate(
        (*TranslatedSids).Entries as usize * core::mem::size_of::<LSA_TRANSLATED_SID>(),
    ) as *mut LSA_TRANSLATED_SID;
    if (*TranslatedSids).Sids.is_null() {
        MIDL_user_free(TranslatedSidsEx2.Sids as *mut _);
        MIDL_user_free(*ReferencedDomains as *mut _);
        *ReferencedDomains = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    i = 0;
    while i < TranslatedSidsEx2.Entries {
        (*(*TranslatedSids).Sids.add(i as usize)).Use =
            (*TranslatedSidsEx2.Sids.add(i as usize)).Use;
        (*(*TranslatedSids).Sids.add(i as usize)).RelativeId =
            LsapGetRelativeIdFromSid((*TranslatedSidsEx2.Sids.add(i as usize)).Sid as PSID);
        (*(*TranslatedSids).Sids.add(i as usize)).DomainIndex =
            (*TranslatedSidsEx2.Sids.add(i as usize)).DomainIndex;
        i += 1;
    }

    MIDL_user_free(TranslatedSidsEx2.Sids as *mut _);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LsarLookupSids(
    _PolicyHandle: LSAPR_HANDLE,
    SidEnumBuffer: PLSAPR_SID_ENUM_BUFFER,
    ReferencedDomains: *mut PLSAPR_REFERENCED_DOMAIN_LIST,
    TranslatedNames: PLSAPR_TRANSLATED_NAMES,
    LookupLevel: LSAP_LOOKUP_LEVEL,
    MappedCount: *mut DWORD,
) -> NTSTATUS {
    let mut TranslatedNamesEx: LSAPR_TRANSLATED_NAMES_EX = core::mem::zeroed();
    let mut i: ULONG;
    let Status: NTSTATUS;

    TRACE!(
        b"LsarLookupSids(%p %p %p %p %d %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        SidEnumBuffer,
        ReferencedDomains,
        TranslatedNames,
        LookupLevel,
        MappedCount,
    );

    (*TranslatedNames).Entries = (*SidEnumBuffer).Entries;
    (*TranslatedNames).Names = core::ptr::null_mut();
    *ReferencedDomains = core::ptr::null_mut();

    TranslatedNamesEx.Entries = (*SidEnumBuffer).Entries;
    TranslatedNamesEx.Names = core::ptr::null_mut();

    Status = LsapLookupSids(
        SidEnumBuffer,
        ReferencedDomains,
        &mut TranslatedNamesEx,
        LookupLevel,
        MappedCount,
        0,
        0,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*TranslatedNames).Entries = (*SidEnumBuffer).Entries;
    (*TranslatedNames).Names = MIDL_user_allocate(
        (*SidEnumBuffer).Entries as usize * core::mem::size_of::<LSAPR_TRANSLATED_NAME>(),
    ) as *mut LSAPR_TRANSLATED_NAME;
    if (*TranslatedNames).Names.is_null() {
        MIDL_user_free(TranslatedNamesEx.Names as *mut _);
        MIDL_user_free(*ReferencedDomains as *mut _);
        *ReferencedDomains = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    i = 0;
    while i < TranslatedNamesEx.Entries {
        (*(*TranslatedNames).Names.offset(i as isize)).Use =
            (*TranslatedNamesEx.Names.offset(i as isize)).Use;
        (*(*TranslatedNames).Names.offset(i as isize)).Name.Length =
            (*TranslatedNamesEx.Names.offset(i as isize)).Name.Length;
        (*(*TranslatedNames).Names.offset(i as isize)).Name.MaximumLength =
            (*TranslatedNamesEx.Names.offset(i as isize)).Name.MaximumLength;
        (*(*TranslatedNames).Names.offset(i as isize)).Name.Buffer =
            (*TranslatedNamesEx.Names.offset(i as isize)).Name.Buffer;
        (*(*TranslatedNames).Names.offset(i as isize)).DomainIndex =
            (*TranslatedNamesEx.Names.offset(i as isize)).DomainIndex;
        i += 1;
    }

    MIDL_user_free(TranslatedNamesEx.Names as *mut _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarCreateSecret(
    PolicyHandle: LSAPR_HANDLE,
    SecretName: PRPC_UNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    SecretHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut SecretObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let mut SecretSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SecretSdSize: ULONG = 0;
    let mut Status: NTSTATUS;

    'done: loop {
        Status = LsapValidateDbObject(
            PolicyHandle as PLSA_DB_OBJECT,
            LsaDbPolicyObject,
            POLICY_CREATE_SECRET,
            &mut PolicyObject,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        Status = NtQuerySystemTime(&mut Time);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = LsapCreateSecretSd(&mut SecretSd, &mut SecretSdSize);
        if !NT_SUCCESS(Status) {
            return Status;
        }

        Status = LsapCreateDbObject(
            PolicyObject,
            crate::wide!("Secrets").as_ptr() as PWSTR,
            (*SecretName).Buffer,
            LsaDbSecretObject,
            DesiredAccess,
            (*PolicyObject).Trusted,
            &mut SecretObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = LsapSetObjectAttribute(
            SecretObject,
            crate::wide!("CurrentTime").as_ptr() as PWSTR,
            &mut Time as *mut _ as PVOID,
            core::mem::size_of::<LARGE_INTEGER>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = LsapSetObjectAttribute(
            SecretObject,
            crate::wide!("OldTime").as_ptr() as PWSTR,
            &mut Time as *mut _ as PVOID,
            core::mem::size_of::<LARGE_INTEGER>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = LsapSetObjectAttribute(
            SecretObject,
            crate::wide!("SecDesc").as_ptr() as PWSTR,
            SecretSd,
            SecretSdSize,
        );

        break 'done;
    }

    if !SecretSd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, SecretSd);
    }

    if !NT_SUCCESS(Status) {
        if !SecretObject.is_null() {
            LsapCloseDbObject(SecretObject);
        }
    } else {
        *SecretHandle = SecretObject as LSAPR_HANDLE;
    }

    STATUS_SUCCESS
}

unsafe fn LsarpOpenAccount(
    PolicyObject: PLSA_DB_OBJECT,
    AccountSid: PRPC_SID,
    DesiredAccess: ACCESS_MASK,
    AccountObject: *mut PLSA_DB_OBJECT,
) -> NTSTATUS {
    let mut SidString: LPWSTR = core::ptr::null_mut();
    let Status: NTSTATUS;

    if ConvertSidToStringSid(AccountSid as PSID, &mut SidString) == 0 {
        ERR!("ConvertSidToStringSid failed\n");
        return STATUS_INVALID_PARAMETER;
    }

    Status = LsapOpenDbObject(
        PolicyObject,
        w!("Accounts").as_ptr() as PWSTR,
        SidString,
        LsaDbAccountObject,
        DesiredAccess,
        (*PolicyObject).Trusted,
        AccountObject,
    );
    if !NT_SUCCESS(Status) {
        ERR!(
            "LsapOpenDbObject(Accounts/%S) failed (Status 0x%08lx)\n",
            SidString,
            Status,
        );
    }

    if !SidString.is_null() {
        LocalFree(SidString as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarOpenAccount(
    PolicyHandle: LSAPR_HANDLE,
    AccountSid: PRPC_SID,
    DesiredAccess: ACCESS_MASK,
    AccountHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let Status: NTSTATUS;

    if RtlValidSid(AccountSid as PSID) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        0,
        &mut PolicyObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    LsarpOpenAccount(
        PolicyObject,
        AccountSid,
        DesiredAccess,
        AccountHandle as *mut PLSA_DB_OBJECT,
    )
}

#[no_mangle]
pub unsafe extern "system" fn LsarEnumeratePrivilegesAccount(
    AccountHandle: LSAPR_HANDLE,
    Privileges: *mut PLSAPR_PRIVILEGE_SET,
) -> NTSTATUS {
    let mut AccountObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut PrivilegeSetSize: ULONG = 0;
    let PrivilegeSet: PLSAPR_PRIVILEGE_SET;
    let mut Status: NTSTATUS;

    *Privileges = core::ptr::null_mut();

    Status = LsapValidateDbObject(
        AccountHandle as PLSA_DB_OBJECT,
        LsaDbAccountObject,
        ACCOUNT_VIEW,
        &mut AccountObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LsapGetObjectAttribute(
        AccountObject,
        w!("Privilgs").as_ptr() as PWSTR,
        core::ptr::null_mut(),
        &mut PrivilegeSetSize,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    PrivilegeSet = MIDL_user_allocate(PrivilegeSetSize as usize) as PLSAPR_PRIVILEGE_SET;
    if PrivilegeSet.is_null() {
        return STATUS_NO_MEMORY;
    }

    Status = LsapGetObjectAttribute(
        AccountObject,
        w!("Privilgs").as_ptr() as PWSTR,
        PrivilegeSet as *mut core::ffi::c_void,
        &mut PrivilegeSetSize,
    );
    if !NT_SUCCESS(Status) {
        MIDL_user_free(PrivilegeSet as *mut core::ffi::c_void);
        return Status;
    }

    *Privileges = PrivilegeSet;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LsarAddPrivilegesToAccount(
    AccountHandle: LSAPR_HANDLE,
    Privileges: PLSAPR_PRIVILEGE_SET,
) -> NTSTATUS {
    let mut AccountObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut CurrentPrivileges: PPRIVILEGE_SET = core::ptr::null_mut();
    let mut NewPrivileges: PPRIVILEGE_SET = core::ptr::null_mut();
    let mut PrivilegeSetSize: ULONG = 0;
    let mut PrivilegeCount: ULONG;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut bFound: BOOL;
    let mut Status: NTSTATUS;

    Status = LsapValidateDbObject(
        AccountHandle as PLSA_DB_OBJECT,
        LsaDbAccountObject,
        ACCOUNT_ADJUST_PRIVILEGES,
        &mut AccountObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LsapGetObjectAttribute(
        AccountObject,
        w!("Privilgs").as_ptr() as PWSTR,
        core::ptr::null_mut(),
        &mut PrivilegeSetSize,
    );
    if !NT_SUCCESS(Status) || PrivilegeSetSize == 0 {
        PrivilegeSetSize = (core::mem::size_of::<PRIVILEGE_SET>() as ULONG)
            + ((*Privileges).PrivilegeCount - 1)
                * (core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG);
        Status = LsapSetObjectAttribute(
            AccountObject,
            w!("Privilgs").as_ptr() as PWSTR,
            Privileges as *mut core::ffi::c_void,
            PrivilegeSetSize,
        );
    } else {
        CurrentPrivileges = MIDL_user_allocate(PrivilegeSetSize as usize) as PPRIVILEGE_SET;
        if CurrentPrivileges.is_null() {
            return STATUS_NO_MEMORY;
        }

        'done: {
            Status = LsapGetObjectAttribute(
                AccountObject,
                w!("Privilgs").as_ptr() as PWSTR,
                CurrentPrivileges as *mut core::ffi::c_void,
                &mut PrivilegeSetSize,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            PrivilegeCount = (*CurrentPrivileges).PrivilegeCount;

            i = 0;
            while i < (*Privileges).PrivilegeCount {
                bFound = 0;
                j = 0;
                while j < (*CurrentPrivileges).PrivilegeCount {
                    if RtlEqualLuid_inline(
                        core::ptr::addr_of_mut!((*(&raw mut (*Privileges).Privilege[0]).add(i as usize)).Luid) as *mut _LUID,
                        &mut (*(&raw mut (*CurrentPrivileges).Privilege[0]).add(i as usize)).Luid,
                    ) {
                        bFound = 1;
                        break;
                    }
                    j += 1;
                }

                if bFound == 0 {
                    PrivilegeCount += 1;
                }
                i += 1;
            }

            PrivilegeSetSize = (core::mem::size_of::<PRIVILEGE_SET>() as ULONG)
                + (PrivilegeCount - 1)
                    * (core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG);
            NewPrivileges = MIDL_user_allocate(PrivilegeSetSize as usize) as PPRIVILEGE_SET;
            if NewPrivileges.is_null() {
                Status = STATUS_NO_MEMORY;
                break 'done;
            }

            (*NewPrivileges).PrivilegeCount = PrivilegeCount;
            (*NewPrivileges).Control = 0;

            RtlCopyLuidAndAttributesArray(
                (*CurrentPrivileges).PrivilegeCount,
                &mut (*CurrentPrivileges).Privilege[0] as PLUID_AND_ATTRIBUTES,
                &mut (*NewPrivileges).Privilege[0] as PLUID_AND_ATTRIBUTES,
            );

            PrivilegeCount = (*CurrentPrivileges).PrivilegeCount;
            i = 0;
            while i < (*Privileges).PrivilegeCount {
                bFound = 0;
                j = 0;
                while j < (*CurrentPrivileges).PrivilegeCount {
                    if RtlEqualLuid_inline(
                        core::ptr::addr_of_mut!((*(&raw mut (*Privileges).Privilege[0]).add(i as usize)).Luid) as *mut _LUID,
                        &mut (*(&raw mut (*CurrentPrivileges).Privilege[0]).add(i as usize)).Luid,
                    ) {
                        (*(&raw mut (*NewPrivileges).Privilege[0]).add(j as usize)).Attributes =
                            (*(&raw mut (*Privileges).Privilege[0]).add(i as usize)).Attributes;

                        bFound = 1;
                        break;
                    }
                    j += 1;
                }

                if bFound == 0 {
                    RtlCopyLuidAndAttributesArray(
                        1,
                        &mut (*(&raw mut (*Privileges).Privilege[0]).add(i as usize)) as *mut _
                            as PLUID_AND_ATTRIBUTES,
                        &mut *(&raw mut (*NewPrivileges).Privilege[0])
                            .add(PrivilegeCount as usize)
                            as PLUID_AND_ATTRIBUTES,
                    );
                    PrivilegeCount += 1;
                }
                i += 1;
            }

            Status = LsapSetObjectAttribute(
                AccountObject,
                w!("Privilgs").as_ptr() as PWSTR,
                NewPrivileges as *mut core::ffi::c_void,
                PrivilegeSetSize,
            );
        }
    }

    if !CurrentPrivileges.is_null() {
        MIDL_user_free(CurrentPrivileges as *mut core::ffi::c_void);
    }

    if !NewPrivileges.is_null() {
        MIDL_user_free(NewPrivileges as *mut core::ffi::c_void);
    }

    Status
}

unsafe fn RtlEqualLuid_inline(L1: PLUID, L2: PLUID) -> bool {
    (*L1).LowPart == (*L2).LowPart && (*L1).HighPart == (*L2).HighPart
}

#[no_mangle]
pub unsafe extern "system" fn LsarRemovePrivilegesFromAccount(
    AccountHandle: LSAPR_HANDLE,
    AllPrivileges: BOOLEAN,
    Privileges: PLSAPR_PRIVILEGE_SET,
) -> NTSTATUS {
    let mut AccountObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut CurrentPrivileges: PPRIVILEGE_SET = core::ptr::null_mut();
    let mut NewPrivileges: PPRIVILEGE_SET = core::ptr::null_mut();
    let mut PrivilegeSetSize: ULONG = 0;
    let mut PrivilegeCount: ULONG;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut k: ULONG;
    let mut bFound: BOOL;
    let mut Status: NTSTATUS;

    if ((AllPrivileges == 0) && (Privileges.is_null()))
        || ((AllPrivileges != 0) && (!Privileges.is_null()))
    {
        return STATUS_INVALID_PARAMETER;
    }

    Status = LsapValidateDbObject(
        AccountHandle as PLSA_DB_OBJECT,
        LsaDbAccountObject,
        ACCOUNT_ADJUST_PRIVILEGES,
        &mut AccountObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: {
        if AllPrivileges != 0 {
            Status = LsapDeleteObjectAttribute(
                AccountObject,
                crate::wide!("Privilgs").as_ptr() as PWSTR,
            );
            if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                Status = STATUS_SUCCESS;
            }
        } else {
            Status = LsapGetObjectAttribute(
                AccountObject,
                crate::wide!("Privilgs").as_ptr() as PWSTR,
                core::ptr::null_mut(),
                &mut PrivilegeSetSize,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            if PrivilegeSetSize == 0 {
                Status = STATUS_SUCCESS;
                break 'done;
            }

            CurrentPrivileges = MIDL_user_allocate(PrivilegeSetSize as usize) as PPRIVILEGE_SET;
            if CurrentPrivileges.is_null() {
                return STATUS_NO_MEMORY;
            }

            Status = LsapGetObjectAttribute(
                AccountObject,
                crate::wide!("Privilgs").as_ptr() as PWSTR,
                CurrentPrivileges as *mut _,
                &mut PrivilegeSetSize,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            PrivilegeCount = (*CurrentPrivileges).PrivilegeCount;

            i = 0;
            while i < (*CurrentPrivileges).PrivilegeCount {
                j = 0;
                while j < (*Privileges).PrivilegeCount {
                    let lhs = &(*CurrentPrivileges).Privilege.as_ptr().add(i as usize).as_ref().unwrap().Luid;
                    let rhs = &(*Privileges).Privilege.as_ptr().add(j as usize).as_ref().unwrap().Luid;
                    if lhs.LowPart == rhs.LowPart && lhs.HighPart == rhs.HighPart {
                        if PrivilegeCount > 0 {
                            PrivilegeCount -= 1;
                        }
                    }
                    j += 1;
                }
                i += 1;
            }

            if PrivilegeCount == 0 {
                Status = LsapDeleteObjectAttribute(
                    AccountObject,
                    crate::wide!("Privilgs").as_ptr() as PWSTR,
                );
                if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                    Status = STATUS_SUCCESS;
                }
            } else {
                PrivilegeSetSize = (core::mem::size_of::<PRIVILEGE_SET>() as ULONG)
                    + (PrivilegeCount - 1) * (core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG);
                NewPrivileges = MIDL_user_allocate(PrivilegeSetSize as usize) as PPRIVILEGE_SET;
                if NewPrivileges.is_null() {
                    Status = STATUS_NO_MEMORY;
                    break 'done;
                }

                (*NewPrivileges).PrivilegeCount = PrivilegeCount;
                (*NewPrivileges).Control = 0;

                i = 0;
                k = 0;
                while i < (*CurrentPrivileges).PrivilegeCount {
                    bFound = 0;
                    j = 0;
                    while j < (*Privileges).PrivilegeCount {
                        let lhs = &(*CurrentPrivileges).Privilege.as_ptr().add(i as usize).as_ref().unwrap().Luid;
                        let rhs = &(*Privileges).Privilege.as_ptr().add(j as usize).as_ref().unwrap().Luid;
                        if lhs.LowPart == rhs.LowPart && lhs.HighPart == rhs.HighPart {
                            bFound = 1;
                        }
                        j += 1;
                    }

                    if bFound == 0 {
                        RtlCopyLuidAndAttributesArray(
                            1,
                            (*CurrentPrivileges).Privilege.as_mut_ptr().add(i as usize),
                            (*NewPrivileges).Privilege.as_mut_ptr().add(k as usize),
                        );
                        k += 1;
                    }
                    i += 1;
                }

                Status = LsapSetObjectAttribute(
                    AccountObject,
                    crate::wide!("Privilgs").as_ptr() as PWSTR,
                    NewPrivileges as *mut _,
                    PrivilegeSetSize,
                );
            }
        }
    }

    if !CurrentPrivileges.is_null() {
        MIDL_user_free(CurrentPrivileges as *mut _);
    }

    if !NewPrivileges.is_null() {
        MIDL_user_free(NewPrivileges as *mut _);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarGetQuotasForAccount(
    AccountHandle: LSAPR_HANDLE,
    QuotaLimits: PQUOTA_LIMITS,
) -> NTSTATUS {
    let mut AccountObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut Status: NTSTATUS;

    Status = LsapValidateDbObject(
        AccountHandle as PLSA_DB_OBJECT,
        LsaDbAccountObject,
        ACCOUNT_VIEW,
        &mut AccountObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LsapGetObjectAttribute(
        AccountObject,
        w!("DefQuota").as_ptr() as PWSTR,
        QuotaLimits as *mut core::ffi::c_void,
        &mut Size,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarSetQuotasForAccount(
    AccountHandle: LSAPR_HANDLE,
    QuotaLimits: PQUOTA_LIMITS,
) -> NTSTATUS {
    let mut AccountObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut InternalQuotaLimits: QUOTA_LIMITS = core::mem::zeroed();
    let mut Size: ULONG;
    let mut Status: NTSTATUS;

    Status = LsapValidateDbObject(
        AccountHandle as PLSA_DB_OBJECT,
        LsaDbAccountObject,
        ACCOUNT_ADJUST_QUOTAS,
        &mut AccountObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Size = core::mem::size_of::<QUOTA_LIMITS>() as ULONG;
    Status = LsapGetObjectAttribute(
        AccountObject,
        ['D' as u16, 'e' as u16, 'f' as u16, 'Q' as u16, 'u' as u16, 'o' as u16, 't' as u16, 'a' as u16, 0].as_ptr() as PWSTR,
        &mut InternalQuotaLimits as *mut _ as *mut core::ffi::c_void,
        &mut Size,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*QuotaLimits).PagedPoolLimit != 0 {
        InternalQuotaLimits.PagedPoolLimit = (*QuotaLimits).PagedPoolLimit;
    }

    if (*QuotaLimits).NonPagedPoolLimit != 0 {
        InternalQuotaLimits.NonPagedPoolLimit = (*QuotaLimits).NonPagedPoolLimit;
    }

    if (*QuotaLimits).MinimumWorkingSetSize != 0 {
        InternalQuotaLimits.MinimumWorkingSetSize = (*QuotaLimits).MinimumWorkingSetSize;
    }

    if (*QuotaLimits).MaximumWorkingSetSize != 0 {
        InternalQuotaLimits.MaximumWorkingSetSize = (*QuotaLimits).MaximumWorkingSetSize;
    }

    if (*QuotaLimits).PagefileLimit != 0 {
        InternalQuotaLimits.PagefileLimit = (*QuotaLimits).PagefileLimit;
    }

    Status = LsapSetObjectAttribute(
        AccountObject,
        ['D' as u16, 'e' as u16, 'f' as u16, 'Q' as u16, 'u' as u16, 'o' as u16, 't' as u16, 'a' as u16, 0].as_ptr() as PWSTR,
        &mut InternalQuotaLimits as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<QUOTA_LIMITS>() as ULONG,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarGetSystemAccessAccount(
    AccountHandle: LSAPR_HANDLE,
    SystemAccess: *mut ACCESS_MASK,
) -> NTSTATUS {
    let mut AccountObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut Size: ULONG = core::mem::size_of::<ACCESS_MASK>() as ULONG;
    let mut Status: NTSTATUS;

    Status = LsapValidateDbObject(
        AccountHandle as PLSA_DB_OBJECT,
        LsaDbAccountObject,
        ACCOUNT_VIEW,
        &mut AccountObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LsapGetObjectAttribute(
        AccountObject,
        w!("ActSysAc").as_ptr() as PWSTR,
        SystemAccess as *mut core::ffi::c_void,
        &mut Size,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarSetSystemAccessAccount(
    AccountHandle: LSAPR_HANDLE,
    SystemAccess: ACCESS_MASK,
) -> NTSTATUS {
    let mut AccountObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = LsapValidateDbObject(
        AccountHandle as PLSA_DB_OBJECT,
        LsaDbAccountObject,
        ACCOUNT_ADJUST_SYSTEM_ACCESS,
        &mut AccountObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let mut SystemAccess = SystemAccess;
    Status = LsapSetObjectAttribute(
        AccountObject,
        w!("ActSysAc").as_ptr() as PWSTR,
        core::ptr::addr_of_mut!(SystemAccess) as *mut core::ffi::c_void,
        core::mem::size_of::<ACCESS_MASK>() as u32,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarOpenTrustedDomain(
    _PolicyHandle: LSAPR_HANDLE,
    _TrustedDomainSid: PRPC_SID,
    _DesiredAccess: ACCESS_MASK,
    _TrustedDomainHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    if LsapProductType != NtProductLanManNt {
        return STATUS_DIRECTORY_SERVICE_REQUIRED;
    }

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarQueryInfoTrustedDomain(
    _TrustedDomainHandle: LSAPR_HANDLE,
    _InformationClass: TRUSTED_INFORMATION_CLASS,
    _TrustedDomainInformation: *mut PLSAPR_TRUSTED_DOMAIN_INFO,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarSetInformationTrustedDomain(
    _TrustedDomainHandle: LSAPR_HANDLE,
    _InformationClass: TRUSTED_INFORMATION_CLASS,
    _TrustedDomainInformation: PLSAPR_TRUSTED_DOMAIN_INFO,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarOpenSecret(
    PolicyHandle: LSAPR_HANDLE,
    SecretName: PRPC_UNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    SecretHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut SecretObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        0,
        &mut PolicyObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LsapOpenDbObject(
        PolicyObject,
        w!("Secrets").as_ptr() as PWSTR,
        (*SecretName).Buffer,
        LsaDbSecretObject,
        DesiredAccess,
        (*PolicyObject).Trusted,
        &mut SecretObject,
    );
    if !NT_SUCCESS(Status) {
        // goto done
    }

    if !NT_SUCCESS(Status) {
        if !SecretObject.is_null() {
            LsapCloseDbObject(SecretObject);
        }
    } else {
        *SecretHandle = SecretObject as LSAPR_HANDLE;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarSetSecret(
    SecretHandle: LSAPR_HANDLE,
    EncryptedCurrentValue: PLSAPR_CR_CIPHER_VALUE,
    EncryptedOldValue: PLSAPR_CR_CIPHER_VALUE,
) -> NTSTATUS {
    let mut SecretObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut CurrentValue: PBYTE = core::ptr::null_mut();
    let mut OldValue: PBYTE = core::ptr::null_mut();
    let mut CurrentValueLength: ULONG = 0;
    let mut OldValueLength: ULONG = 0;
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = LsapValidateDbObject(
        SecretHandle as PLSA_DB_OBJECT,
        LsaDbSecretObject,
        SECRET_SET_VALUE as ACCESS_MASK,
        &mut SecretObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: {
        if !EncryptedCurrentValue.is_null() {
            CurrentValue = (*EncryptedCurrentValue).Buffer;
            CurrentValueLength = (*EncryptedCurrentValue).MaximumLength;
        }

        Status = LsapSetObjectAttribute(
            SecretObject,
            crate::wide!("CurrentValue").as_ptr() as PWSTR,
            CurrentValue as *mut core::ffi::c_void,
            CurrentValueLength,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = NtQuerySystemTime(&mut Time);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = LsapSetObjectAttribute(
            SecretObject,
            crate::wide!("CurrentTime").as_ptr() as PWSTR,
            core::ptr::addr_of_mut!(Time) as *mut core::ffi::c_void,
            core::mem::size_of::<LARGE_INTEGER>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if !EncryptedOldValue.is_null() {
            OldValue = (*EncryptedOldValue).Buffer;
            OldValueLength = (*EncryptedOldValue).MaximumLength;
        }

        Status = LsapSetObjectAttribute(
            SecretObject,
            crate::wide!("OldValue").as_ptr() as PWSTR,
            OldValue as *mut core::ffi::c_void,
            OldValueLength,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = LsapSetObjectAttribute(
            SecretObject,
            crate::wide!("OldTime").as_ptr() as PWSTR,
            core::ptr::addr_of_mut!(Time) as *mut core::ffi::c_void,
            core::mem::size_of::<LARGE_INTEGER>() as ULONG,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarQuerySecret(
    SecretHandle: LSAPR_HANDLE,
    EncryptedCurrentValue: *mut PLSAPR_CR_CIPHER_VALUE,
    CurrentValueSetTime: PLARGE_INTEGER,
    EncryptedOldValue: *mut PLSAPR_CR_CIPHER_VALUE,
    OldValueSetTime: PLARGE_INTEGER,
) -> NTSTATUS {
    let mut SecretObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut EncCurrentValue: PLSAPR_CR_CIPHER_VALUE = core::ptr::null_mut();
    let mut EncOldValue: PLSAPR_CR_CIPHER_VALUE = core::ptr::null_mut();
    let mut CurrentValue: PBYTE = core::ptr::null_mut();
    let mut OldValue: PBYTE = core::ptr::null_mut();
    let mut CurrentValueLength: ULONG;
    let mut OldValueLength: ULONG;
    let mut BufferSize: ULONG;
    let mut Status: NTSTATUS;

    Status = LsapValidateDbObject(
        SecretHandle as PLSA_DB_OBJECT,
        LsaDbSecretObject,
        SECRET_QUERY_VALUE,
        &mut SecretObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: loop {
        if !EncryptedCurrentValue.is_null() {
            CurrentValueLength = 0;

            Status = LsapGetObjectAttribute(
                SecretObject,
                crate::wide!("CurrentValue").as_ptr() as PWSTR,
                core::ptr::null_mut(),
                &mut CurrentValueLength,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            CurrentValue = midl_user_allocate(CurrentValueLength as usize) as PBYTE;
            if CurrentValue.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            Status = LsapGetObjectAttribute(
                SecretObject,
                crate::wide!("CurrentValue").as_ptr() as PWSTR,
                CurrentValue as PVOID,
                &mut CurrentValueLength,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            EncCurrentValue =
                midl_user_allocate(core::mem::size_of::<LSAPR_CR_CIPHER_VALUE>()) as PLSAPR_CR_CIPHER_VALUE;
            if EncCurrentValue.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            (*EncCurrentValue).Length =
                (CurrentValueLength - core::mem::size_of::<u16>() as ULONG) as ULONG;
            (*EncCurrentValue).MaximumLength = CurrentValueLength as ULONG;
            (*EncCurrentValue).Buffer = CurrentValue;
        }

        if !CurrentValueSetTime.is_null() {
            BufferSize = core::mem::size_of::<LARGE_INTEGER>() as ULONG;

            Status = LsapGetObjectAttribute(
                SecretObject,
                crate::wide!("CurrentTime").as_ptr() as PWSTR,
                CurrentValueSetTime as PBYTE as PVOID,
                &mut BufferSize,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if !EncryptedOldValue.is_null() {
            OldValueLength = 0;

            Status = LsapGetObjectAttribute(
                SecretObject,
                crate::wide!("OldValue").as_ptr() as PWSTR,
                core::ptr::null_mut(),
                &mut OldValueLength,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            OldValue = midl_user_allocate(OldValueLength as usize) as PBYTE;
            if OldValue.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            Status = LsapGetObjectAttribute(
                SecretObject,
                crate::wide!("OldValue").as_ptr() as PWSTR,
                OldValue as PVOID,
                &mut OldValueLength,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            EncOldValue = midl_user_allocate(
                core::mem::size_of::<LSAPR_CR_CIPHER_VALUE>() + OldValueLength as usize,
            ) as PLSAPR_CR_CIPHER_VALUE;
            if EncOldValue.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            (*EncOldValue).Length =
                (OldValueLength - core::mem::size_of::<u16>() as ULONG) as ULONG;
            (*EncOldValue).MaximumLength = OldValueLength as ULONG;
            (*EncOldValue).Buffer = OldValue;
        }

        if !OldValueSetTime.is_null() {
            BufferSize = core::mem::size_of::<LARGE_INTEGER>() as ULONG;

            Status = LsapGetObjectAttribute(
                SecretObject,
                crate::wide!("OldTime").as_ptr() as PWSTR,
                OldValueSetTime as PBYTE as PVOID,
                &mut BufferSize,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        break 'done;
    }

    if NT_SUCCESS(Status) {
        if !EncryptedCurrentValue.is_null() {
            *EncryptedCurrentValue = EncCurrentValue;
        }

        if !EncryptedOldValue.is_null() {
            *EncryptedOldValue = EncOldValue;
        }
    } else {
        if !EncryptedCurrentValue.is_null() {
            *EncryptedCurrentValue = core::ptr::null_mut();
        }

        if !EncryptedOldValue.is_null() {
            *EncryptedOldValue = core::ptr::null_mut();
        }

        if !EncCurrentValue.is_null() {
            midl_user_free(EncCurrentValue as *mut core::ffi::c_void);
        }

        if !EncOldValue.is_null() {
            midl_user_free(EncOldValue as *mut core::ffi::c_void);
        }

        if !CurrentValue.is_null() {
            midl_user_free(CurrentValue as *mut core::ffi::c_void);
        }

        if !OldValue.is_null() {
            midl_user_free(OldValue as *mut core::ffi::c_void);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarLookupPrivilegeValue(
    PolicyHandle: LSAPR_HANDLE,
    Name: PRPC_UNICODE_STRING,
    Value: PLUID,
) -> NTSTATUS {
    let pValue: PLUID;
    let Status: NTSTATUS;

    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        POLICY_LOOKUP_NAMES,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    pValue = LsarpLookupPrivilegeValue(Name);
    if pValue.is_null() {
        return STATUS_NO_SUCH_PRIVILEGE;
    }

    RtlCopyLuid(Value, pValue);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LsarLookupPrivilegeName(
    PolicyHandle: LSAPR_HANDLE,
    Value: PLUID,
    Name: *mut *mut RPC_UNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        POLICY_LOOKUP_NAMES,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LsarpLookupPrivilegeName(Value, Name);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarLookupPrivilegeDisplayName(
    PolicyHandle: LSAPR_HANDLE,
    Name: PRPC_UNICODE_STRING,
    ClientLanguage: USHORT,
    ClientSystemDefaultLanguage: USHORT,
    DisplayName: *mut PRPC_UNICODE_STRING,
    LanguageReturned: *mut USHORT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        POLICY_LOOKUP_NAMES,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LsarpLookupPrivilegeDisplayName(
        Name,
        ClientLanguage,
        ClientSystemDefaultLanguage,
        DisplayName,
        LanguageReturned,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarDeleteObject(
    ObjectHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    let mut DbObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if ObjectHandle.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = LsapValidateDbObject(
        *ObjectHandle as PLSA_DB_OBJECT,
        LsaDbIgnoreObject,
        DELETE as ACCESS_MASK,
        &mut DbObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*DbObject).ObjectType == LsaDbPolicyObject {
        return STATUS_INVALID_PARAMETER;
    }

    Status = LsapDeleteDbObject(DbObject);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    *ObjectHandle = core::ptr::null_mut();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LsarEnumerateAccountsWithUserRight(
    PolicyHandle: LSAPR_HANDLE,
    UserRight: PRPC_UNICODE_STRING,
    EnumerationBuffer: PLSAPR_ACCOUNT_ENUM_BUFFER,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut AccountRight: ACCESS_MASK = 0;
    let mut Luid: PLUID = core::ptr::null_mut();
    let mut AccountKeyBufferSize: ULONG = 0;
    let AccountKeyBuffer: PWSTR;
    let mut AccountsKeyHandle: HANDLE = core::ptr::null_mut();
    let mut AccountKeyHandle: HANDLE = core::ptr::null_mut();
    let mut AttributeKeyHandle: HANDLE = core::ptr::null_mut();
    let mut SystemAccess: ACCESS_MASK;
    let mut PrivilegeSet: PPRIVILEGE_SET;
    let EnumBuffer: PLSAPR_ACCOUNT_INFORMATION;
    let ReturnBuffer: PLSAPR_ACCOUNT_INFORMATION;
    let mut SubKeyCount: ULONG = 0;
    let mut EnumIndex: ULONG;
    let mut EnumCount: ULONG = 0;
    let mut Size: ULONG;
    let mut i: ULONG;
    let mut Found: BOOL;
    let mut Status: NTSTATUS;

    'done: loop {
        /* Validate the privilege and account right names */
        if UserRight != core::ptr::null_mut() {
            Luid = LsarpLookupPrivilegeValue(UserRight);
            if Luid == core::ptr::null_mut() {
                AccountRight = LsapLookupAccountRightValue(UserRight as PUNICODE_STRING);
                if AccountRight == 0 {
                    return STATUS_NO_SUCH_PRIVILEGE;
                }
            }
        }

        if EnumerationBuffer == core::ptr::null_mut() {
            return STATUS_INVALID_PARAMETER;
        }

        (*EnumerationBuffer).EntriesRead = 0;
        (*EnumerationBuffer).Information = core::ptr::null_mut();

        /* Validate the PolicyHandle */
        Status = LsapValidateDbObject(
            PolicyHandle as PLSA_DB_OBJECT,
            LsaDbPolicyObject,
            POLICY_LOOKUP_NAMES | POLICY_VIEW_LOCAL_INFORMATION,
            &mut PolicyObject,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        Status = LsapRegOpenKey(
            (*PolicyObject).KeyHandle,
            crate::wide!("Accounts").as_ptr() as PWSTR,
            KEY_READ,
            &mut AccountsKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        Status = LsapRegQueryKeyInfo(
            AccountsKeyHandle,
            &mut SubKeyCount,
            &mut AccountKeyBufferSize,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        AccountKeyBufferSize += core::mem::size_of::<u16>() as ULONG;
        AccountKeyBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, AccountKeyBufferSize as SIZE_T) as PWSTR;
        if AccountKeyBuffer == core::ptr::null_mut() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        EnumBuffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((SubKeyCount as usize) * core::mem::size_of::<LSAPR_ACCOUNT_INFORMATION>()) as SIZE_T,
        ) as PLSAPR_ACCOUNT_INFORMATION;
        if EnumBuffer == core::ptr::null_mut() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        EnumCount = 0;
        EnumIndex = 0;
        loop {
            Found = 0;

            Status = LsapRegEnumerateSubKey(
                AccountsKeyHandle,
                EnumIndex,
                AccountKeyBufferSize,
                AccountKeyBuffer,
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_NO_MORE_ENTRIES {
                    Status = STATUS_SUCCESS;
                }
                break;
            }

            Status = LsapRegOpenKey(
                AccountsKeyHandle,
                AccountKeyBuffer,
                KEY_READ,
                &mut AccountKeyHandle,
            );
            if NT_SUCCESS(Status) {
                if Luid != core::ptr::null_mut() || AccountRight != 0 {
                    let attr_name: &'static [u16] = if Luid != core::ptr::null_mut() {
                        crate::wide!("Privilgs")
                    } else {
                        crate::wide!("ActSysAc")
                    };
                    Status = LsapRegOpenKey(
                        AccountKeyHandle,
                        attr_name.as_ptr() as PWSTR,
                        KEY_READ,
                        &mut AttributeKeyHandle,
                    );
                    if NT_SUCCESS(Status) {
                        if Luid != core::ptr::null_mut() {
                            Size = 0;
                            LsapRegQueryValue(
                                AttributeKeyHandle,
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                &mut Size,
                            );
                            if Size != 0 {
                                PrivilegeSet = RtlAllocateHeap(RtlGetProcessHeap(), 0, Size as SIZE_T) as PPRIVILEGE_SET;
                                if PrivilegeSet != core::ptr::null_mut() {
                                    if LsapRegQueryValue(
                                        AttributeKeyHandle,
                                        core::ptr::null_mut(),
                                        core::ptr::null_mut(),
                                        PrivilegeSet as *mut _,
                                        &mut Size,
                                    ) == STATUS_SUCCESS
                                    {
                                        i = 0;
                                        while i < (*PrivilegeSet).PrivilegeCount {
                                            let priv_luid = core::ptr::addr_of!(
                                                (*((*PrivilegeSet).Privilege.as_ptr().add(i as usize))).Luid
                                            );
                                            if (*priv_luid).LowPart == (*Luid).LowPart
                                                && (*priv_luid).HighPart == (*Luid).HighPart
                                            {
                                                Found = 1;
                                                break;
                                            }
                                            i += 1;
                                        }
                                    }

                                    RtlFreeHeap(RtlGetProcessHeap(), 0, PrivilegeSet as *mut _);
                                }
                            }
                        } else if AccountRight != 0 {
                            SystemAccess = 0;
                            Size = core::mem::size_of::<ACCESS_MASK>() as ULONG;
                            LsapRegQueryValue(
                                AttributeKeyHandle,
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                &mut SystemAccess as *mut _ as *mut _,
                                &mut Size,
                            );
                            if SystemAccess & AccountRight != 0 {
                                Found = 1;
                            }
                        }

                        LsapRegCloseKey(AttributeKeyHandle);
                    }
                } else {
                    /* enumerate all accounts */
                    Found = 1;
                }

                if Found == 1 {
                    Status = LsapRegOpenKey(
                        AccountKeyHandle,
                        crate::wide!("Sid").as_ptr() as PWSTR,
                        KEY_READ,
                        &mut AttributeKeyHandle,
                    );
                    if NT_SUCCESS(Status) {
                        Size = 0;
                        LsapRegQueryValue(
                            AttributeKeyHandle,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            &mut Size,
                        );
                        if Size != 0 {
                            (*EnumBuffer.add(EnumCount as usize)).Sid = midl_user_allocate(Size as usize) as PRPC_SID;
                            if (*EnumBuffer.add(EnumCount as usize)).Sid != core::ptr::null_mut() {
                                Status = LsapRegQueryValue(
                                    AttributeKeyHandle,
                                    core::ptr::null_mut(),
                                    core::ptr::null_mut(),
                                    (*EnumBuffer.add(EnumCount as usize)).Sid as *mut _,
                                    &mut Size,
                                );
                                if NT_SUCCESS(Status) {
                                    EnumCount += 1;
                                } else {
                                    midl_user_free((*EnumBuffer.add(EnumCount as usize)).Sid as *mut _);
                                    (*EnumBuffer.add(EnumCount as usize)).Sid = core::ptr::null_mut();
                                }
                            }
                        }

                        LsapRegCloseKey(AttributeKeyHandle);
                    }
                }

                LsapRegCloseKey(AccountKeyHandle);
            }

            EnumIndex += 1;
        }

        if NT_SUCCESS(Status) && EnumCount != 0 {
            ReturnBuffer = midl_user_allocate(
                (EnumCount as usize) * core::mem::size_of::<LSAPR_ACCOUNT_INFORMATION>(),
            ) as PLSAPR_ACCOUNT_INFORMATION;
            if ReturnBuffer == core::ptr::null_mut() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            core::ptr::copy_nonoverlapping(
                EnumBuffer,
                ReturnBuffer,
                EnumCount as usize,
            );

            (*EnumerationBuffer).EntriesRead = EnumCount;
            (*EnumerationBuffer).Information = ReturnBuffer;
        }

        break 'done;
    }

    if EnumBuffer != core::ptr::null_mut() {
        if Status != STATUS_SUCCESS {
            i = 0;
            while i < EnumCount {
                if (*EnumBuffer.add(i as usize)).Sid != core::ptr::null_mut() {
                    midl_user_free((*EnumBuffer.add(i as usize)).Sid as *mut _);
                }
                i += 1;
            }
        }

        RtlFreeHeap(RtlGetProcessHeap(), 0, EnumBuffer as *mut _);
    }

    if AccountKeyBuffer != core::ptr::null_mut() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AccountKeyBuffer as *mut _);
    }

    if Status == STATUS_SUCCESS && EnumCount == 0 {
        Status = STATUS_NO_MORE_ENTRIES;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarEnumerateAccountRights(
    PolicyHandle: LSAPR_HANDLE,
    AccountSid: PRPC_SID,
    UserRights: PLSAPR_USER_RIGHT_SET,
) -> NTSTATUS {
    let mut AccountHandle: LSAPR_HANDLE = core::ptr::null_mut();
    let mut PrivilegeSet: PLSAPR_PRIVILEGE_SET = core::ptr::null_mut();
    let mut RightsBuffer: PRPC_UNICODE_STRING = core::ptr::null_mut();
    let mut PrivilegeString: PRPC_UNICODE_STRING;
    let mut SystemAccess: ACCESS_MASK = 0;
    let mut RightsCount: ULONG = 0;
    let mut Index: ULONG;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    Status = LsarOpenAccount(
        PolicyHandle,
        AccountSid,
        ACCOUNT_VIEW,
        &mut AccountHandle,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: loop {
        Status = LsarEnumeratePrivilegesAccount(AccountHandle, &mut PrivilegeSet);
        if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
            break 'done;
        }

        Status = LsarGetSystemAccessAccount(AccountHandle, &mut SystemAccess);
        if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
            break 'done;
        }

        RightsCount = (*PrivilegeSet).PrivilegeCount;

        i = 0;
        while i < (core::mem::size_of::<ACCESS_MASK>() as ULONG) * 8 {
            if SystemAccess & (1u32 << i) != 0 {
                RightsCount += 1;
            }
            i += 1;
        }

        if RightsCount == 0 {
            (*UserRights).Entries = 0;
            (*UserRights).UserRights = core::ptr::null_mut();
            Status = STATUS_SUCCESS;
            break 'done;
        }

        RightsBuffer = MIDL_user_allocate(
            RightsCount as usize * core::mem::size_of::<RPC_UNICODE_STRING>(),
        ) as PRPC_UNICODE_STRING;
        if RightsBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Index = 0;
        if !PrivilegeSet.is_null() {
            i = 0;
            while i < (*PrivilegeSet).PrivilegeCount {
                PrivilegeString = core::ptr::null_mut();
                Status = LsarLookupPrivilegeName(
                    PolicyHandle,
                    core::ptr::addr_of_mut!((*(&raw mut (*PrivilegeSet).Privilege[0]).add(i as usize)).Luid) as PLUID,
                    &mut PrivilegeString,
                );
                if !NT_SUCCESS(Status) {
                    break 'done;
                }

                (*RightsBuffer.add(Index as usize)).Length = (*PrivilegeString).Length;
                (*RightsBuffer.add(Index as usize)).MaximumLength =
                    (*PrivilegeString).MaximumLength;
                (*RightsBuffer.add(Index as usize)).Buffer = (*PrivilegeString).Buffer;

                MIDL_user_free(PrivilegeString as *mut core::ffi::c_void);
                Index += 1;
                i += 1;
            }
        }

        i = 0;
        while i < (core::mem::size_of::<ACCESS_MASK>() as ULONG) * 8 {
            if SystemAccess & (1u32 << i) != 0 {
                PrivilegeString = core::ptr::null_mut();
                Status = LsapLookupAccountRightName(1u32 << i, &mut PrivilegeString);
                if !NT_SUCCESS(Status) {
                    break 'done;
                }

                (*RightsBuffer.add(Index as usize)).Length = (*PrivilegeString).Length;
                (*RightsBuffer.add(Index as usize)).MaximumLength =
                    (*PrivilegeString).MaximumLength;
                (*RightsBuffer.add(Index as usize)).Buffer = (*PrivilegeString).Buffer;

                MIDL_user_free(PrivilegeString as *mut core::ffi::c_void);
                Index += 1;
            }
            i += 1;
        }

        (*UserRights).Entries = RightsCount;
        (*UserRights).UserRights = RightsBuffer;

        break 'done;
    }

    if !NT_SUCCESS(Status) {
        if !RightsBuffer.is_null() {
            Index = 0;
            while Index < RightsCount {
                if !(*RightsBuffer.add(Index as usize)).Buffer.is_null() {
                    MIDL_user_free(
                        (*RightsBuffer.add(Index as usize)).Buffer as *mut core::ffi::c_void,
                    );
                }
                Index += 1;
            }

            MIDL_user_free(RightsBuffer as *mut core::ffi::c_void);
        }
    }

    if !PrivilegeSet.is_null() {
        MIDL_user_free(PrivilegeSet as *mut core::ffi::c_void);
    }

    LsarClose(&mut AccountHandle);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarAddAccountRights(
    PolicyHandle: LSAPR_HANDLE,
    AccountSid: PRPC_SID,
    UserRights: PLSAPR_USER_RIGHT_SET,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut AccountObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut ulNewPrivileges: ULONG = 0;
    let mut ulNewRights: ULONG = 0;
    let mut SystemAccess: ACCESS_MASK = 0;
    let mut Size: ULONG;
    let mut Value: ULONG;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut PrivilegeSet: PPRIVILEGE_SET = core::ptr::null_mut();
    let PrivilegeSetBufferSize: ULONG;
    let PrivilegeCount: ULONG;
    let mut bFound: BOOLEAN;
    let mut pLuid: PLUID;
    let mut Status: NTSTATUS;

    /* Validate the AccountSid */
    if RtlValidSid(AccountSid as PSID) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    /* Validate the UserRights */
    if UserRights.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    /* Validate the privilege and account right names */
    i = 0;
    while i < (*UserRights).Entries {
        if !LsarpLookupPrivilegeValue(core::ptr::addr_of_mut!((*(*UserRights).UserRights.offset(i as isize)))).is_null() {
            ulNewPrivileges += 1;
        } else {
            if LsapLookupAccountRightValue(core::ptr::addr_of_mut!((*(*UserRights).UserRights.offset(i as isize))) as PUNICODE_STRING) == 0 {
                return STATUS_NO_SUCH_PRIVILEGE;
            }
            ulNewRights += 1;
        }
        i += 1;
    }

    /* Validate the PolicyHandle */
    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        POLICY_LOOKUP_NAMES,
        &mut PolicyObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: loop {
        /* Open the account */
        Status = LsarpOpenAccount(PolicyObject, AccountSid, 0, &mut AccountObject);
        if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
            break 'done;
        } else if Status == STATUS_OBJECT_NAME_NOT_FOUND {
            /* Create a new account if it does not yet exist */
            Status = LsarpCreateAccount(PolicyObject, AccountSid, 0, &mut AccountObject);
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if ulNewPrivileges > 0 {
            Size = 0;

            /* Get the size of the Privilgs attribute */
            Status = LsapGetObjectAttribute(
                AccountObject,
                crate::wide!("Privilgs").as_ptr() as PWSTR,
                core::ptr::null_mut(),
                &mut Size,
            );
            if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
                break 'done;
            }

            /* Calculate the required privilege set buffer size */
            if Size == 0 {
                PrivilegeSetBufferSize = core::mem::size_of::<PRIVILEGE_SET>() as ULONG
                    + (ulNewPrivileges - 1) * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG;
            } else {
                PrivilegeSetBufferSize =
                    Size + ulNewPrivileges * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG;
            }

            /* Allocate the privilege set buffer */
            PrivilegeSet = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                PrivilegeSetBufferSize as SIZE_T,
            ) as PPRIVILEGE_SET;
            if PrivilegeSet.is_null() {
                return STATUS_NO_MEMORY;
            }

            /* Get the privilege set */
            if Size != 0 {
                Status = LsapGetObjectAttribute(
                    AccountObject,
                    crate::wide!("Privilgs").as_ptr() as PWSTR,
                    PrivilegeSet as PVOID,
                    &mut Size,
                );
                if !NT_SUCCESS(Status) {
                    break 'done;
                }
            }

            PrivilegeCount = (*PrivilegeSet).PrivilegeCount;

            i = 0;
            while i < (*UserRights).Entries {
                pLuid = LsarpLookupPrivilegeValue(core::ptr::addr_of_mut!((*(*UserRights).UserRights.offset(i as isize))));
                if pLuid.is_null() {
                    i += 1;
                    continue;
                }

                bFound = 0u8;
                j = 0;
                while j < (*PrivilegeSet).PrivilegeCount {
                    let lpLuid = core::ptr::addr_of!((*(*PrivilegeSet).Privilege.as_ptr().offset(j as isize)).Luid);
                    if (*lpLuid).LowPart == (*pLuid).LowPart && (*lpLuid).HighPart == (*pLuid).HighPart {
                        bFound = 1u8;
                        break;
                    }
                    j += 1;
                }

                if bFound == 0u8 {
                    /* Copy the new privilege */
                    let dest = core::ptr::addr_of_mut!((*(*PrivilegeSet).Privilege.as_mut_ptr().offset((*PrivilegeSet).PrivilegeCount as isize)));
                    core::ptr::copy_nonoverlapping(
                        pLuid as *const u8,
                        dest as *mut u8,
                        core::mem::size_of::<LUID>(),
                    );
                    (*PrivilegeSet).PrivilegeCount += 1;
                }
                i += 1;
            }

            /* Store the extended privilege set */
            if PrivilegeCount != (*PrivilegeSet).PrivilegeCount {
                Size = core::mem::size_of::<PRIVILEGE_SET>() as ULONG
                    + ((*PrivilegeSet).PrivilegeCount - 1) * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG;

                Status = LsapSetObjectAttribute(
                    AccountObject,
                    crate::wide!("Privilgs").as_ptr() as PWSTR,
                    PrivilegeSet as PVOID,
                    Size,
                );
                if !NT_SUCCESS(Status) {
                    break 'done;
                }
            }
        }

        if ulNewRights > 0 {
            Size = core::mem::size_of::<ACCESS_MASK>() as ULONG;

            /* Get the system access flags, if the attribute exists */
            Status = LsapGetObjectAttribute(
                AccountObject,
                crate::wide!("ActSysAc").as_ptr() as PWSTR,
                &mut SystemAccess as *mut ACCESS_MASK as PVOID,
                &mut Size,
            );
            if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
                break 'done;
            }

            /* Set the new access rights */
            i = 0;
            while i < (*UserRights).Entries {
                Value = LsapLookupAccountRightValue(core::ptr::addr_of_mut!((*(*UserRights).UserRights.offset(i as isize))) as PUNICODE_STRING);
                if Value != 0 {
                    SystemAccess |= Value;
                }
                i += 1;
            }

            /* Set the system access flags */
            Status = LsapSetObjectAttribute(
                AccountObject,
                crate::wide!("ActSysAc").as_ptr() as PWSTR,
                &mut SystemAccess as *mut ACCESS_MASK as PVOID,
                core::mem::size_of::<ACCESS_MASK>() as ULONG,
            );
        }

        break 'done;
    }

    if !PrivilegeSet.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, PrivilegeSet as PVOID);
    }

    if !AccountObject.is_null() {
        LsapCloseDbObject(AccountObject);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarRemoveAccountRights(
    PolicyHandle: LSAPR_HANDLE,
    AccountSid: PRPC_SID,
    AllRights: BOOLEAN,
    UserRights: PLSAPR_USER_RIGHT_SET,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut AccountObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut PrivilegesToRemove: ULONG = 0;
    let mut RightsToRemove: ULONG = 0;
    let mut SystemAccess: ACCESS_MASK = 0;
    let mut Size: ULONG;
    let mut Value: ULONG;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut Index: ULONG;
    let mut PrivilegeSet: PPRIVILEGE_SET = core::ptr::null_mut();
    let PrivilegeCount: ULONG;
    let mut pLuid: PLUID;
    let mut Status: NTSTATUS;

    if RtlValidSid(AccountSid as PSID) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if UserRights.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    i = 0;
    while i < (*UserRights).Entries {
        if !LsarpLookupPrivilegeValue(core::ptr::addr_of_mut!(
            *(*UserRights).UserRights.offset(i as isize)
        ))
        .is_null()
        {
            PrivilegesToRemove += 1;
        } else {
            if LsapLookupAccountRightValue(core::ptr::addr_of_mut!(
                *(*UserRights).UserRights.offset(i as isize)
            ) as PUNICODE_STRING) == 0
            {
                return STATUS_NO_SUCH_PRIVILEGE;
            }
            RightsToRemove += 1;
        }
        i += 1;
    }

    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbObjectType_LsaDbPolicyObject,
        POLICY_LOOKUP_NAMES,
        &mut PolicyObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: {
        Status = LsarpOpenAccount(PolicyObject, AccountSid, 0, &mut AccountObject);
        if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
            break 'done;
        }

        if AllRights == 0u8 {
            Size = 0;
            Status = LsapGetObjectAttribute(
                AccountObject,
                crate::wide!("Privilgs").as_ptr() as PWSTR,
                core::ptr::null_mut(),
                &mut Size,
            );
            if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
                break 'done;
            }

            if (Size != 0) && (PrivilegesToRemove != 0) {
                PrivilegeSet = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, Size as SIZE_T)
                    as PPRIVILEGE_SET;
                if PrivilegeSet.is_null() {
                    return STATUS_NO_MEMORY;
                }

                Status = LsapGetObjectAttribute(
                    AccountObject,
                    crate::wide!("Privilgs").as_ptr() as PWSTR,
                    PrivilegeSet as PVOID,
                    &mut Size,
                );
                if !NT_SUCCESS(Status) {
                    break 'done;
                }

                PrivilegeCount = (*PrivilegeSet).PrivilegeCount;

                i = 0;
                while i < (*UserRights).Entries {
                    pLuid = LsarpLookupPrivilegeValue(core::ptr::addr_of_mut!(
                        *(*UserRights).UserRights.offset(i as isize)
                    ));
                    if pLuid.is_null() {
                        i += 1;
                        continue;
                    }

                    Index = u32::MAX;
                    j = 0;
                    while j < (*PrivilegeSet).PrivilegeCount {
                        let pl =
                            core::ptr::addr_of_mut!((*(*PrivilegeSet).Privilege.as_mut_ptr().offset(j as isize)).Luid);
                        if (*pl).HighPart == (*pLuid).HighPart && (*pl).LowPart == (*pLuid).LowPart {
                            Index = j;
                            break;
                        }
                        j += 1;
                    }

                    if Index != u32::MAX {
                        if ((*PrivilegeSet).PrivilegeCount > 1)
                            && (Index < (*PrivilegeSet).PrivilegeCount - 1)
                        {
                            core::ptr::copy(
                                (*PrivilegeSet).Privilege.as_ptr().offset((Index + 1) as isize)
                                    as *const u8,
                                (*PrivilegeSet).Privilege.as_mut_ptr().offset(Index as isize)
                                    as *mut u8,
                                (Index
                                    .wrapping_sub((*PrivilegeSet).PrivilegeCount)
                                    .wrapping_sub(1) as usize)
                                    .wrapping_mul(core::mem::size_of::<LUID>()),
                            );
                        }

                        core::ptr::write_bytes(
                            (*PrivilegeSet)
                                .Privilege
                                .as_mut_ptr()
                                .offset(((*PrivilegeSet).PrivilegeCount - 1) as isize)
                                as *mut u8,
                            0,
                            core::mem::size_of::<LUID>(),
                        );

                        (*PrivilegeSet).PrivilegeCount -= 1;
                    }

                    i += 1;
                }

                if PrivilegeCount != (*PrivilegeSet).PrivilegeCount {
                    Size = (core::mem::size_of::<PRIVILEGE_SET>()
                        + ((*PrivilegeSet).PrivilegeCount as usize - 1)
                            * core::mem::size_of::<LUID_AND_ATTRIBUTES>())
                        as ULONG;

                    Status = LsapSetObjectAttribute(
                        AccountObject,
                        crate::wide!("Privilgs").as_ptr() as PWSTR,
                        PrivilegeSet as PVOID,
                        Size,
                    );
                    if !NT_SUCCESS(Status) {
                        break 'done;
                    }
                }
            }

            Size = 0;
            Status = LsapGetObjectAttribute(
                AccountObject,
                crate::wide!("ActSysAc").as_ptr() as PWSTR,
                &mut SystemAccess as *mut ACCESS_MASK as PVOID,
                &mut Size,
            );
            if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
                break 'done;
            }

            if (Size != 0) && (RightsToRemove != 0) {
                i = 0;
                while i < (*UserRights).Entries {
                    Value = LsapLookupAccountRightValue(core::ptr::addr_of_mut!(
                        *(*UserRights).UserRights.offset(i as isize)
                    ) as PUNICODE_STRING);
                    if Value != 0 {
                        SystemAccess &= !Value;
                    }
                    i += 1;
                }

                Status = LsapSetObjectAttribute(
                    AccountObject,
                    crate::wide!("ActSysAc").as_ptr() as PWSTR,
                    &mut SystemAccess as *mut ACCESS_MASK as PVOID,
                    core::mem::size_of::<ACCESS_MASK>() as ULONG,
                );
            }
        }
    }

    if !PrivilegeSet.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, PrivilegeSet as PVOID);
    }

    if !AccountObject.is_null() {
        LsapCloseDbObject(AccountObject);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarQueryTrustedDomainInfo(
    _PolicyHandle: LSAPR_HANDLE,
    _TrustedDomainSid: PRPC_SID,
    _InformationClass: TRUSTED_INFORMATION_CLASS,
    _TrustedDomainInformation: *mut PLSAPR_TRUSTED_DOMAIN_INFO,
) -> NTSTATUS {
    if LsapProductType != NtProductLanManNt {
        return STATUS_DIRECTORY_SERVICE_REQUIRED;
    }

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarSetTrustedDomainInfo(
    _PolicyHandle: LSAPR_HANDLE,
    _TrustedDomainSid: PRPC_SID,
    _InformationClass: TRUSTED_INFORMATION_CLASS,
    _TrustedDomainInformation: PLSAPR_TRUSTED_DOMAIN_INFO,
) -> NTSTATUS {
    if LsapProductType != NtProductLanManNt {
        return STATUS_DIRECTORY_SERVICE_REQUIRED;
    }

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarDeleteTrustedDomain(
    _PolicyHandle: LSAPR_HANDLE,
    _TrustedDomainSid: PRPC_SID,
) -> NTSTATUS {
    if LsapProductType != NtProductLanManNt {
        return STATUS_DIRECTORY_SERVICE_REQUIRED;
    }

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarStorePrivateData(
    PolicyHandle: LSAPR_HANDLE,
    KeyName: PRPC_UNICODE_STRING,
    EncryptedData: PLSAPR_CR_CIPHER_VALUE,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut SecretsObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut SecretObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let Value: PBYTE;
    let ValueLength: ULONG;
    let mut Status: NTSTATUS;

    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        POLICY_CREATE_SECRET,
        &mut PolicyObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: {
        Status = LsapOpenDbObject(
            PolicyObject,
            core::ptr::null_mut(),
            crate::wstr!("Secrets").as_ptr() as PWSTR,
            LsaDbIgnoreObject,
            0,
            (*PolicyObject).Trusted,
            &mut SecretsObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if EncryptedData.is_null() {
            Status = LsapOpenDbObject(
                SecretsObject,
                core::ptr::null_mut(),
                (*KeyName).Buffer,
                LsaDbSecretObject,
                0,
                (*PolicyObject).Trusted,
                &mut SecretObject,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = LsapDeleteDbObject(SecretObject);
            if NT_SUCCESS(Status) {
                SecretObject = core::ptr::null_mut();
            }
        } else {
            Status = LsapCreateDbObject(
                SecretsObject,
                core::ptr::null_mut(),
                (*KeyName).Buffer,
                LsaDbSecretObject,
                0,
                (*PolicyObject).Trusted,
                &mut SecretObject,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Value = (*EncryptedData).Buffer;
            ValueLength = (*EncryptedData).MaximumLength;

            Status = NtQuerySystemTime(&mut Time);
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = LsapSetObjectAttribute(
                SecretObject,
                crate::wstr!("CurrentValue").as_ptr() as PWSTR,
                Value as *mut _,
                ValueLength,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = LsapSetObjectAttribute(
                SecretObject,
                crate::wstr!("CurrentTime").as_ptr() as PWSTR,
                &mut Time as *mut _ as *mut _,
                core::mem::size_of::<LARGE_INTEGER>() as ULONG,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = NtQuerySystemTime(&mut Time);
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = LsapSetObjectAttribute(
                SecretObject,
                crate::wstr!("OldValue").as_ptr() as PWSTR,
                core::ptr::null_mut(),
                0,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = LsapSetObjectAttribute(
                SecretObject,
                crate::wstr!("OldTime").as_ptr() as PWSTR,
                &mut Time as *mut _ as *mut _,
                core::mem::size_of::<LARGE_INTEGER>() as ULONG,
            );
            if !NT_SUCCESS(Status) {
            }
        }
    }

    if !SecretObject.is_null() {
        LsapCloseDbObject(SecretObject);
    }

    if !SecretsObject.is_null() {
        LsapCloseDbObject(SecretsObject);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarRetrievePrivateData(
    PolicyHandle: LSAPR_HANDLE,
    KeyName: PRPC_UNICODE_STRING,
    EncryptedData: *mut PLSAPR_CR_CIPHER_VALUE,
) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut SecretObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut EncCurrentValue: PLSAPR_CR_CIPHER_VALUE = core::ptr::null_mut();
    let mut CurrentValueLength: ULONG = 0;
    let CurrentValue: PBYTE;
    let mut Status: NTSTATUS;

    TRACE!(
        b"LsarRetrievePrivateData(%p %wZ %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        KeyName,
        EncryptedData,
    );

    Status = LsapValidateDbObject(
        PolicyHandle as PLSA_DB_OBJECT,
        LsaDbPolicyObject,
        POLICY_GET_PRIVATE_INFORMATION,
        &mut PolicyObject,
    );
    if !NT_SUCCESS(Status) {
        ERR!(
            "LsapValidateDbObject returned 0x%08lx\n",
            Status,
        );
        return Status;
    }

    'done: {
        Status = LsapOpenDbObject(
            PolicyObject,
            crate::wide!("Secrets").as_ptr() as PWSTR,
            (*KeyName).Buffer,
            LsaDbSecretObject,
            0,
            (*PolicyObject).Trusted,
            &mut SecretObject,
        );
        if !NT_SUCCESS(Status) {
            ERR!(
                "LsapOpenDbObject failed (Status 0x%08lx)\n",
                Status,
            );
            break 'done;
        }

        Status = LsapGetObjectAttribute(
            SecretObject,
            crate::wide!("CurrentValue").as_ptr() as PWSTR,
            core::ptr::null_mut(),
            &mut CurrentValueLength,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        CurrentValue = midl_user_allocate(CurrentValueLength as usize) as PBYTE;
        if CurrentValue.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = LsapGetObjectAttribute(
            SecretObject,
            crate::wide!("CurrentValue").as_ptr() as PWSTR,
            CurrentValue as *mut _,
            &mut CurrentValueLength,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        EncCurrentValue = midl_user_allocate(
            core::mem::size_of::<LSAPR_CR_CIPHER_VALUE>() + CurrentValueLength as usize,
        ) as PLSAPR_CR_CIPHER_VALUE;
        if EncCurrentValue.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*EncCurrentValue).Length =
            (CurrentValueLength as usize - core::mem::size_of::<u16>()) as ULONG;
        (*EncCurrentValue).MaximumLength = CurrentValueLength as ULONG;
        (*EncCurrentValue).Buffer = EncCurrentValue.add(1) as PBYTE;
        RtlCopyMemory(
            (*EncCurrentValue).Buffer as *mut _,
            CurrentValue as *const _,
            CurrentValueLength as usize,
        );
    }

    if NT_SUCCESS(Status) {
        if !EncryptedData.is_null() {
            *EncryptedData = EncCurrentValue;
        }
    } else {
        if !EncryptedData.is_null() {
            *EncryptedData = core::ptr::null_mut();
        }

        if !EncCurrentValue.is_null() {
            midl_user_free(EncCurrentValue as *mut _);
        }
    }

    if !SecretObject.is_null() {
        LsapCloseDbObject(SecretObject);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarOpenPolicy2(
    SystemName: LPWSTR,
    ObjectAttributes: PLSAPR_OBJECT_ATTRIBUTES,
    DesiredAccess: ACCESS_MASK,
    PolicyHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    LsarOpenPolicy(SystemName, ObjectAttributes, DesiredAccess, PolicyHandle)
}

#[no_mangle]
pub unsafe extern "system" fn LsarGetUserName(
    _SystemName: LPWSTR,
    _UserName: *mut PRPC_UNICODE_STRING,
    _DomainName: *mut PRPC_UNICODE_STRING,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarQueryInformationPolicy2(
    PolicyHandle: LSAPR_HANDLE,
    InformationClass: POLICY_INFORMATION_CLASS,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    LsarQueryInformationPolicy(PolicyHandle, InformationClass, PolicyInformation)
}

#[no_mangle]
pub unsafe extern "system" fn LsarSetInformationPolicy2(
    PolicyHandle: LSAPR_HANDLE,
    InformationClass: POLICY_INFORMATION_CLASS,
    PolicyInformation: PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    LsarSetInformationPolicy(PolicyHandle, InformationClass, PolicyInformation)
}

#[no_mangle]
pub unsafe extern "system" fn LsarQueryTrustedDomainInfoByName(
    _PolicyHandle: LSAPR_HANDLE,
    _TrustedDomainName: PRPC_UNICODE_STRING,
    _InformationClass: POLICY_INFORMATION_CLASS,
    _PolicyInformation: *mut PLSAPR_TRUSTED_DOMAIN_INFO,
) -> NTSTATUS {
    if LsapProductType != NtProductLanManNt {
        return STATUS_DIRECTORY_SERVICE_REQUIRED;
    }

    STATUS_OBJECT_NAME_NOT_FOUND
}

#[no_mangle]
pub unsafe extern "system" fn LsarSetTrustedDomainInfoByName(
    _PolicyHandle: LSAPR_HANDLE,
    _TrustedDomainName: PRPC_UNICODE_STRING,
    _InformationClass: POLICY_INFORMATION_CLASS,
    _PolicyInformation: PLSAPR_TRUSTED_DOMAIN_INFO,
) -> NTSTATUS {
    if LsapProductType != NtProductLanManNt {
        return STATUS_DIRECTORY_SERVICE_REQUIRED;
    }

    STATUS_OBJECT_NAME_NOT_FOUND
}

#[no_mangle]
pub unsafe extern "system" fn LsarEnumerateTrustedDomainsEx(
    _PolicyHandle: LSAPR_HANDLE,
    _EnumerationContext: *mut DWORD,
    EnumerationBuffer: PLSAPR_TRUSTED_ENUM_BUFFER_EX,
    _PreferedMaximumLength: DWORD,
) -> NTSTATUS {
    if LsapProductType != NtProductLanManNt {
        return STATUS_DIRECTORY_SERVICE_REQUIRED;
    }

    (*EnumerationBuffer).EntriesRead = 0;
    (*EnumerationBuffer).EnumerationBuffer = core::ptr::null_mut();
    STATUS_NO_MORE_ENTRIES
}

#[no_mangle]
pub unsafe extern "system" fn LsarCreateTrustedDomainEx(
    _PolicyHandle: LSAPR_HANDLE,
    _TrustedDomainInformation: PLSAPR_TRUSTED_DOMAIN_INFORMATION_EX,
    _AuthentificationInformation: PLSAPR_TRUSTED_DOMAIN_AUTH_INFORMATION,
    _DesiredAccess: ACCESS_MASK,
    _TrustedDomainHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    if LsapProductType != NtProductLanManNt {
        return STATUS_DIRECTORY_SERVICE_REQUIRED;
    }

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarSetPolicyReplicationHandle(
    _PolicyHandle: PLSAPR_HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarQueryDomainInformationPolicy(
    _PolicyHandle: LSAPR_HANDLE,
    _InformationClass: POLICY_INFORMATION_CLASS,
    _PolicyInformation: *mut PLSAPR_POLICY_DOMAIN_INFORMATION,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarSetDomainInformationPolicy(
    _PolicyHandle: LSAPR_HANDLE,
    _InformationClass: POLICY_INFORMATION_CLASS,
    _PolicyInformation: PLSAPR_POLICY_DOMAIN_INFORMATION,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarOpenTrustedDomainByName(
    _PolicyHandle: LSAPR_HANDLE,
    _TrustedDomainName: PRPC_UNICODE_STRING,
    _DesiredAccess: ACCESS_MASK,
    _TrustedDomainHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    if LsapProductType != NtProductLanManNt {
        return STATUS_DIRECTORY_SERVICE_REQUIRED;
    }

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarTestCall(_hBinding: handle_t) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarLookupSids2(
    _PolicyHandle: LSAPR_HANDLE,
    SidEnumBuffer: PLSAPR_SID_ENUM_BUFFER,
    ReferencedDomains: *mut PLSAPR_REFERENCED_DOMAIN_LIST,
    TranslatedNames: PLSAPR_TRANSLATED_NAMES_EX,
    LookupLevel: LSAP_LOOKUP_LEVEL,
    MappedCount: *mut DWORD,
    LookupOptions: DWORD,
    ClientRevision: DWORD,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsarLookupSids2(%p %p %p %p %d %p %lu %lu)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        SidEnumBuffer,
        ReferencedDomains,
        TranslatedNames,
        LookupLevel,
        MappedCount,
        LookupOptions,
        ClientRevision,
    );

    (*TranslatedNames).Entries = (*SidEnumBuffer).Entries;
    (*TranslatedNames).Names = core::ptr::null_mut();
    *ReferencedDomains = core::ptr::null_mut();

    Status = LsapLookupSids(
        SidEnumBuffer,
        ReferencedDomains,
        TranslatedNames,
        LookupLevel,
        MappedCount,
        LookupOptions,
        ClientRevision,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarLookupNames2(
    _PolicyHandle: LSAPR_HANDLE,
    Count: DWORD,
    Names: PRPC_UNICODE_STRING,
    ReferencedDomains: *mut PLSAPR_REFERENCED_DOMAIN_LIST,
    TranslatedSids: PLSAPR_TRANSLATED_SIDS_EX,
    LookupLevel: LSAP_LOOKUP_LEVEL,
    MappedCount: *mut DWORD,
    LookupOptions: DWORD,
    ClientRevision: DWORD,
) -> NTSTATUS {
    let mut TranslatedSidsEx2: LSAPR_TRANSLATED_SIDS_EX2 = core::mem::zeroed();
    let mut i: ULONG;
    let Status: NTSTATUS;

    TRACE!(
        b"LsarLookupNames2(%p %lu %p %p %p %d %p %lu %lu)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        Count,
        Names,
        ReferencedDomains,
        TranslatedSids,
        LookupLevel,
        MappedCount,
        LookupOptions,
        ClientRevision,
    );

    (*TranslatedSids).Entries = 0;
    (*TranslatedSids).Sids = core::ptr::null_mut();
    *ReferencedDomains = core::ptr::null_mut();

    if Count == 0 {
        return STATUS_NONE_MAPPED;
    }

    TranslatedSidsEx2.Entries = 0;
    TranslatedSidsEx2.Sids = core::ptr::null_mut();

    Status = LsapLookupNames(
        Count,
        Names,
        ReferencedDomains,
        &mut TranslatedSidsEx2,
        LookupLevel,
        MappedCount,
        LookupOptions,
        ClientRevision,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*TranslatedSids).Entries = TranslatedSidsEx2.Entries;
    (*TranslatedSids).Sids = MIDL_user_allocate(
        (*TranslatedSids).Entries as usize * core::mem::size_of::<LSA_TRANSLATED_SID>(),
    ) as PLSAPR_TRANSLATED_SID_EX;
    if (*TranslatedSids).Sids.is_null() {
        MIDL_user_free(TranslatedSidsEx2.Sids as *mut core::ffi::c_void);
        MIDL_user_free(*ReferencedDomains as *mut core::ffi::c_void);
        *ReferencedDomains = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    i = 0;
    while i < TranslatedSidsEx2.Entries {
        let src = &*TranslatedSidsEx2.Sids.offset(i as isize);
        let dst = &mut *(*TranslatedSids).Sids.offset(i as isize);
        dst.Use = src.Use;
        dst.RelativeId = LsapGetRelativeIdFromSid(src.Sid as PSID);
        dst.DomainIndex = src.DomainIndex;
        dst.Flags = src.Flags;
        i += 1;
    }

    MIDL_user_free(TranslatedSidsEx2.Sids as *mut core::ffi::c_void);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LsarCreateTrustedDomainEx2(
    _PolicyHandle: LSAPR_HANDLE,
    _TrustedDomainInformation: PLSAPR_TRUSTED_DOMAIN_INFORMATION_EX,
    _AuthentificationInformation: PLSAPR_TRUSTED_DOMAIN_AUTH_INFORMATION_INTERNAL,
    _DesiredAccess: ACCESS_MASK,
    _TrustedDomainHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    if LsapProductType != NtProductLanManNt {
        return STATUS_DIRECTORY_SERVICE_REQUIRED;
    }

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CredrWrite(_SystemName: PLSAPR_SERVER_NAME) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CredrRead(_SystemName: PLSAPR_SERVER_NAME) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CredrEnumerate(
    _SystemName: PLSAPR_SERVER_NAME,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CredrWriteDomainCredentials(
    _SystemName: PLSAPR_SERVER_NAME,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CredrReadDomainCredentials(
    _SystemName: PLSAPR_SERVER_NAME,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CredrDelete(
    _SystemName: PLSAPR_SERVER_NAME,
    _TargetName: LPWSTR,
    _Type: DWORD,
    _Flags: DWORD,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CredrGetTargetInfo(
    _SystemName: PLSAPR_SERVER_NAME,
    _TargetName: LPWSTR,
    _Flags: DWORD,
    _TargetInformation: *mut CREDPR_TARGET_INFORMATION,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CredrProfileLoaded(
    _SystemName: PLSAPR_SERVER_NAME,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarLookupNames3(
    _PolicyHandle: LSAPR_HANDLE,
    Count: DWORD,
    Names: PRPC_UNICODE_STRING,
    ReferencedDomains: *mut PLSAPR_REFERENCED_DOMAIN_LIST,
    TranslatedSids: PLSAPR_TRANSLATED_SIDS_EX2,
    LookupLevel: LSAP_LOOKUP_LEVEL,
    MappedCount: *mut DWORD,
    LookupOptions: DWORD,
    ClientRevision: DWORD,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsarLookupNames3(%p %lu %p %p %p %d %p %lu %lu)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        Count,
        Names,
        ReferencedDomains,
        TranslatedSids,
        LookupLevel,
        MappedCount,
        LookupOptions,
        ClientRevision,
    );

    (*TranslatedSids).Entries = 0;
    (*TranslatedSids).Sids = core::ptr::null_mut();
    *ReferencedDomains = core::ptr::null_mut();

    if Count == 0 {
        return STATUS_NONE_MAPPED;
    }

    Status = LsapLookupNames(
        Count,
        Names,
        ReferencedDomains,
        TranslatedSids,
        LookupLevel,
        MappedCount,
        LookupOptions,
        ClientRevision,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CredrGetSessionTypes(
    _SystemName: PLSAPR_SERVER_NAME,
    _MaximumPersistCount: DWORD,
    _MaximumPersist: *mut DWORD,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn LsarRegisterAuditEvent(_hBinding: handle_t) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn LsarGenAuditEvent(_hBinding: handle_t) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarUnregisterAuditEvent(_hBinding: handle_t) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarQueryForestTrustInformation(
    _PolicyHandle: LSAPR_HANDLE,
    _TrustedDomainName: PLSA_UNICODE_STRING,
    _HighestRecordType: LSA_FOREST_TRUST_RECORD_TYPE,
    _ForestTrustInfo: *mut PLSA_FOREST_TRUST_INFORMATION,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarSetForestTrustInformation(
    _PolicyHandle: LSAPR_HANDLE,
    _TrustedDomainName: PLSA_UNICODE_STRING,
    _HighestRecordType: LSA_FOREST_TRUST_RECORD_TYPE,
    _ForestTrustInfo: PLSA_FOREST_TRUST_INFORMATION,
    _CheckOnly: BOOLEAN,
    _CollisionInfo: *mut PLSA_FOREST_TRUST_COLLISION_INFORMATION,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CredrRename(
    _SystemName: PLSAPR_SERVER_NAME,
    _OldTargetName: LPWSTR,
    _NewTargetName: LPWSTR,
    _Type: DWORD,
    _Flags: DWORD,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarLookupSids3(
    _PolicyHandle: LSAPR_HANDLE,
    SidEnumBuffer: PLSAPR_SID_ENUM_BUFFER,
    ReferencedDomains: *mut PLSAPR_REFERENCED_DOMAIN_LIST,
    TranslatedNames: PLSAPR_TRANSLATED_NAMES_EX,
    LookupLevel: LSAP_LOOKUP_LEVEL,
    MappedCount: *mut DWORD,
    LookupOptions: DWORD,
    ClientRevision: DWORD,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsarLookupSids3(%p %p %p %p %d %p %lu %lu)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        SidEnumBuffer,
        ReferencedDomains,
        TranslatedNames,
        LookupLevel,
        MappedCount,
        LookupOptions,
        ClientRevision,
    );

    (*TranslatedNames).Entries = (*SidEnumBuffer).Entries;
    (*TranslatedNames).Names = core::ptr::null_mut();
    *ReferencedDomains = core::ptr::null_mut();

    Status = LsapLookupSids(
        SidEnumBuffer,
        ReferencedDomains,
        TranslatedNames,
        LookupLevel,
        MappedCount,
        LookupOptions,
        ClientRevision,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarLookupNames4(
    _RpcHandle: handle_t,
    Count: DWORD,
    Names: PRPC_UNICODE_STRING,
    ReferencedDomains: *mut PLSAPR_REFERENCED_DOMAIN_LIST,
    TranslatedSids: PLSAPR_TRANSLATED_SIDS_EX2,
    LookupLevel: LSAP_LOOKUP_LEVEL,
    MappedCount: *mut DWORD,
    LookupOptions: DWORD,
    ClientRevision: DWORD,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsarLookupNames4(%p %lu %p %p %p %d %p %lu %lu)\n\0".as_ptr() as *const i8,
        RpcHandle,
        Count,
        Names,
        ReferencedDomains,
        TranslatedSids,
        LookupLevel,
        MappedCount,
        LookupOptions,
        ClientRevision,
    );

    (*TranslatedSids).Entries = 0;
    (*TranslatedSids).Sids = core::ptr::null_mut();
    *ReferencedDomains = core::ptr::null_mut();

    if Count == 0 {
        return STATUS_NONE_MAPPED;
    }

    Status = LsapLookupNames(
        Count,
        Names,
        ReferencedDomains,
        TranslatedSids,
        LookupLevel,
        MappedCount,
        LookupOptions,
        ClientRevision,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsarOpenPolicySce(
    _SystemName: LPWSTR,
    _ObjectAttributes: PLSAPR_OBJECT_ATTRIBUTES,
    _DesiredAccess: ACCESS_MASK,
    _PolicyHandle: *mut LSAPR_HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarAdtRegisterSecurityEventSource(
    _hBinding: handle_t,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarAdtUnregisterSecurityEventSource(
    _hBinding: handle_t,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsarAdtReportSecurityEvent(
    _hBinding: handle_t,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Local Security Authority (LSA) Server
//  * FILE:            reactos/dll/win32/lsasrv/lsarpc.h
//  * PURPOSE:         RPC interface functions
//  *
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// #include "lsasrv.h"
//
// /* GLOBALS *****************************************************************/
//
// static RTL_CRITICAL_SECTION PolicyHandleTableLock;
//
// static
// GENERIC_MAPPING
// LsapPolicyMapping = {POLICY_READ,
//                      POLICY_WRITE,
//                      POLICY_EXECUTE,
//                      POLICY_ALL_ACCESS};
//
// static
// GENERIC_MAPPING
// LsapAccountMapping = {ACCOUNT_READ,
//                       ACCOUNT_WRITE,
//                       ACCOUNT_EXECUTE,
//                       ACCOUNT_ALL_ACCESS};
//
// static
// GENERIC_MAPPING
// LsapSecretMapping = {SECRET_READ,
//                      SECRET_WRITE,
//                      SECRET_EXECUTE,
//                      SECRET_ALL_ACCESS};
//
// /* FUNCTIONS ***************************************************************/
//
// NTSTATUS
// LsarStartRpcServer(VOID)
// {
//     RPC_STATUS Status;
//     DWORD dwError;
//     HANDLE hEvent;
//
//     RtlInitializeCriticalSection(&PolicyHandleTableLock);
//
//     TRACE("LsarStartRpcServer() called\n");
//
//     Status = RpcServerUseProtseqEpW(L"ncacn_np",
//                                     RPC_C_PROTSEQ_MAX_REQS_DEFAULT,
//                                     L"\\pipe\\lsarpc",
//                                     NULL);
//     if (Status != RPC_S_OK)
//     {
//         WARN("RpcServerUseProtseqEpW() failed (Status %lx)\n", Status);
//         return I_RpcMapWin32Status(Status);
//     }
//
//     Status = RpcServerRegisterIf(lsarpc_v0_0_s_ifspec,
//                                  NULL,
//                                  NULL);
//     if (Status != RPC_S_OK)
//     {
//         WARN("RpcServerRegisterIf() failed (Status %lx)\n", Status);
//         return I_RpcMapWin32Status(Status);
//     }
//
//     DsSetupInit();
//
//     Status = RpcServerListen(1, 20, TRUE);
//     if (Status != RPC_S_OK)
//     {
//         WARN("RpcServerListen() failed (Status %lx)\n", Status);
//         return I_RpcMapWin32Status(Status);
//     }
//
//     /* Notify the service manager */
//     TRACE("Creating notification event!\n");
//     hEvent = CreateEventW(NULL,
//                           TRUE,
//                           FALSE,
//                           L"LSA_RPC_SERVER_ACTIVE");
//     if (hEvent == NULL)
//     {
//         dwError = GetLastError();
//         TRACE("Failed to create or open the notification event (Error %lu)\n", dwError);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     TRACE("Set notification event!\n");
//     SetEvent(hEvent);
//
//     /* NOTE: Do not close the event handle, as it must remain alive! */
//
//     TRACE("LsarStartRpcServer() done\n");
//     return STATUS_SUCCESS;
// }
//
//
// void
// __RPC_USER
// LSAPR_HANDLE_rundown(
//     LSAPR_HANDLE hHandle)
// {
// }
//
//
// /* Function 0 */
// NTSTATUS
// WINAPI
// LsarClose(
//     LSAPR_HANDLE *ObjectHandle)
// {
//     PLSA_DB_OBJECT DbObject;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("LsarClose(%p)\n", ObjectHandle);
//
// //    RtlEnterCriticalSection(&PolicyHandleTableLock);
//
//     Status = LsapValidateDbObject(*ObjectHandle,
//                                   LsaDbIgnoreObject,
//                                   0,
//                                   &DbObject);
//     if (Status == STATUS_SUCCESS)
//     {
//         Status = LsapCloseDbObject(DbObject);
//         *ObjectHandle = NULL;
//     }
//
// //    RtlLeaveCriticalSection(&PolicyHandleTableLock);
//
//     return Status;
// }
//
//
// /* Function 1 */
// NTSTATUS
// WINAPI
// LsarDelete(
//     LSAPR_HANDLE ObjectHandle)
// {
//     TRACE("LsarDelete(%p)\n", ObjectHandle);
//
//     return LsarDeleteObject(&ObjectHandle);
// }
//
//
// /* Function 2 */
// NTSTATUS
// WINAPI
// LsarEnumeratePrivileges(
//     LSAPR_HANDLE PolicyHandle,
//     DWORD *EnumerationContext,
//     PLSAPR_PRIVILEGE_ENUM_BUFFER EnumerationBuffer,
//     DWORD PreferedMaximumLength)
// {
//     PLSA_DB_OBJECT PolicyObject;
//     NTSTATUS Status;
//
//     TRACE("LsarEnumeratePrivileges(%p %p %p %lu)\n",
//           PolicyHandle, EnumerationContext, EnumerationBuffer,
//           PreferedMaximumLength);
//
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   POLICY_VIEW_LOCAL_INFORMATION,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (EnumerationContext == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     return LsarpEnumeratePrivileges(EnumerationContext,
//                                     EnumerationBuffer,
//                                     PreferedMaximumLength);
// }
//
//
// /* Function 3 */
// NTSTATUS
// WINAPI
// LsarQuerySecurityObject(
//     LSAPR_HANDLE ObjectHandle,
//     SECURITY_INFORMATION SecurityInformation,
//     PLSAPR_SR_SECURITY_DESCRIPTOR *SecurityDescriptor)
// {
//     PLSA_DB_OBJECT DbObject = NULL;
//     PISECURITY_DESCRIPTOR_RELATIVE RelativeSd = NULL;
//     PSECURITY_DESCRIPTOR ResultSd = NULL;
//     PLSAPR_SR_SECURITY_DESCRIPTOR SdData = NULL;
//     ACCESS_MASK DesiredAccess = 0;
//     ULONG RelativeSdSize = 0;
//     ULONG ResultSdSize = 0;
//     NTSTATUS Status;
//
//     TRACE("LsarQuerySecurityObject(%p %lx %p)\n",
//           ObjectHandle, SecurityInformation, SecurityDescriptor);
//
//     if (SecurityDescriptor == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     *SecurityDescriptor = NULL;
//
//     if ((SecurityInformation & OWNER_SECURITY_INFORMATION) ||
//         (SecurityInformation & GROUP_SECURITY_INFORMATION) ||
//         (SecurityInformation & DACL_SECURITY_INFORMATION))
//         DesiredAccess |= READ_CONTROL;
//
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//         DesiredAccess |= ACCESS_SYSTEM_SECURITY;
//
//     /* Validate the ObjectHandle */
//     Status = LsapValidateDbObject(ObjectHandle,
//                                   LsaDbIgnoreObject,
//                                   DesiredAccess,
//                                   &DbObject);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Get the size of the SD */
//     Status = LsapGetObjectAttribute(DbObject,
//                                     L"SecDesc",
//                                     NULL,
//                                     &RelativeSdSize);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Allocate a buffer for the SD */
//     RelativeSd = MIDL_user_allocate(RelativeSdSize);
//     if (RelativeSd == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the SD */
//     Status = LsapGetObjectAttribute(DbObject,
//                                     L"SecDesc",
//                                     RelativeSd,
//                                     &RelativeSdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Invalidate the SD information that was not requested */
//     if (!(SecurityInformation & OWNER_SECURITY_INFORMATION))
//         RelativeSd->Owner = 0;
//
//     if (!(SecurityInformation & GROUP_SECURITY_INFORMATION))
//         RelativeSd->Group = 0;
//
//     if (!(SecurityInformation & DACL_SECURITY_INFORMATION))
//         RelativeSd->Control &= ~SE_DACL_PRESENT;
//
//     if (!(SecurityInformation & SACL_SECURITY_INFORMATION))
//         RelativeSd->Control &= ~SE_SACL_PRESENT;
//
//     /* Calculate the required SD size */
//     Status = RtlMakeSelfRelativeSD(RelativeSd,
//                                    NULL,
//                                    &ResultSdSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     /* Allocate a buffer for the new SD */
//     ResultSd = MIDL_user_allocate(ResultSdSize);
//     if (ResultSd == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* Build the new SD */
//     Status = RtlMakeSelfRelativeSD(RelativeSd,
//                                    ResultSd,
//                                    &ResultSdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Allocate the SD data buffer */
//     SdData = MIDL_user_allocate(sizeof(LSAPR_SR_SECURITY_DESCRIPTOR));
//     if (SdData == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* Fill the SD data buffer and return it to the caller */
//     SdData->Length = ResultSdSize;
//     SdData->SecurityDescriptor = (PBYTE)ResultSd;
//
//     *SecurityDescriptor = SdData;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (ResultSd != NULL)
//             MIDL_user_free(ResultSd);
//     }
//
//     if (RelativeSd != NULL)
//         MIDL_user_free(RelativeSd);
//
//     return Status;
// }
//
//
// /* Function 4 */
// NTSTATUS
// WINAPI
// LsarSetSecurityObject(
//     LSAPR_HANDLE ObjectHandle,
//     SECURITY_INFORMATION SecurityInformation,
//     PLSAPR_SR_SECURITY_DESCRIPTOR SecurityDescriptor)
// {
//     PLSA_DB_OBJECT DbObject = NULL;
//     ACCESS_MASK DesiredAccess = 0;
//     PSECURITY_DESCRIPTOR RelativeSd = NULL;
//     ULONG RelativeSdSize = 0;
//     HANDLE TokenHandle = NULL;
//     PGENERIC_MAPPING Mapping;
//     NTSTATUS Status;
//
//     TRACE("LsarSetSecurityObject(%p %lx %p)\n",
//           ObjectHandle, SecurityInformation, SecurityDescriptor);
//
//     if ((SecurityDescriptor == NULL) ||
//         (SecurityDescriptor->SecurityDescriptor == NULL) ||
//         !RtlValidSecurityDescriptor((PSECURITY_DESCRIPTOR)SecurityDescriptor->SecurityDescriptor))
//         return ERROR_INVALID_PARAMETER;
//
//     if (SecurityInformation == 0 ||
//         SecurityInformation & ~(OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION
//         | DACL_SECURITY_INFORMATION | SACL_SECURITY_INFORMATION))
//         return ERROR_INVALID_PARAMETER;
//
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//         DesiredAccess |= ACCESS_SYSTEM_SECURITY;
//
//     if (SecurityInformation & DACL_SECURITY_INFORMATION)
//         DesiredAccess |= WRITE_DAC;
//
//     if (SecurityInformation & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION))
//         DesiredAccess |= WRITE_OWNER;
//
//     if ((SecurityInformation & OWNER_SECURITY_INFORMATION) &&
//         (((PISECURITY_DESCRIPTOR)SecurityDescriptor)->Owner == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     if ((SecurityInformation & GROUP_SECURITY_INFORMATION) &&
//         (((PISECURITY_DESCRIPTOR)SecurityDescriptor)->Group == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     /* Validate the ObjectHandle */
//     Status = LsapValidateDbObject(ObjectHandle,
//                                   LsaDbIgnoreObject,
//                                   DesiredAccess,
//                                   &DbObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Get the mapping for the object type */
//     switch (DbObject->ObjectType)
//     {
//         case LsaDbPolicyObject:
//             Mapping = &LsapPolicyMapping;
//             break;
//
//         case LsaDbAccountObject:
//             Mapping = &LsapAccountMapping;
//             break;
//
// //        case LsaDbDomainObject:
// //            Mapping = &LsapDomainMapping;
// //            break;
//
//         case LsaDbSecretObject:
//             Mapping = &LsapSecretMapping;
//             break;
//
//         default:
//             return STATUS_INVALID_HANDLE;
//     }
//
//     /* Get the size of the SD */
//     Status = LsapGetObjectAttribute(DbObject,
//                                     L"SecDesc",
//                                     NULL,
//                                     &RelativeSdSize);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Allocate a buffer for the SD */
//     RelativeSd = RtlAllocateHeap(RtlGetProcessHeap(), 0, RelativeSdSize);
//     if (RelativeSd == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the SD */
//     Status = LsapGetObjectAttribute(DbObject,
//                                     L"SecDesc",
//                                     RelativeSd,
//                                     &RelativeSdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the clients token if we try to set the owner */
//     if (SecurityInformation & OWNER_SECURITY_INFORMATION)
//     {
//         Status = I_RpcMapWin32Status(RpcImpersonateClient(NULL));
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("RpcImpersonateClient returns 0x%08lx\n", Status);
//             goto done;
//         }
//
//         Status = NtOpenThreadToken(NtCurrentThread(),
//                                    TOKEN_QUERY,
//                                    TRUE,
//                                    &TokenHandle);
//         RpcRevertToSelf();
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("NtOpenThreadToken returns 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     /* Build the new security descriptor */
//     Status = RtlSetSecurityObject(SecurityInformation,
//                                   (PSECURITY_DESCRIPTOR)SecurityDescriptor->SecurityDescriptor,
//                                   &RelativeSd,
//                                   Mapping,
//                                   TokenHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("RtlSetSecurityObject failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Set the modified SD */
//     Status = LsapSetObjectAttribute(DbObject,
//                                     L"SecDesc",
//                                     RelativeSd,
//                                     RtlLengthSecurityDescriptor(RelativeSd));
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//     }
//
// done:
//     if (TokenHandle != NULL)
//         NtClose(TokenHandle);
//
//     if (RelativeSd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
//
//     return Status;
// }
//
//
// /* Function 5 */
// NTSTATUS
// WINAPI
// LsarChangePassword(
//     handle_t IDL_handle,
//     PRPC_UNICODE_STRING String1,
//     PRPC_UNICODE_STRING String2,
//     PRPC_UNICODE_STRING String3,
//     PRPC_UNICODE_STRING String4,
//     PRPC_UNICODE_STRING String5)
// {
//     /* Deprecated */
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 6 */
// NTSTATUS
// WINAPI
// LsarOpenPolicy(
//     LPWSTR SystemName,
//     PLSAPR_OBJECT_ATTRIBUTES ObjectAttributes,
//     ACCESS_MASK DesiredAccess,
//     LSAPR_HANDLE *PolicyHandle)
// {
//     PLSA_DB_OBJECT PolicyObject;
//     NTSTATUS Status;
//
//     TRACE("LsarOpenPolicy(%S %p %lx %p)\n",
//           SystemName, ObjectAttributes, DesiredAccess, PolicyHandle);
//
//     RtlEnterCriticalSection(&PolicyHandleTableLock);
//
//     Status = LsapOpenDbObject(NULL,
//                               NULL,
//                               L"Policy",
//                               LsaDbPolicyObject,
//                               DesiredAccess,
//                               FALSE,
//                               &PolicyObject);
//
//     RtlLeaveCriticalSection(&PolicyHandleTableLock);
//
//     if (NT_SUCCESS(Status))
//         *PolicyHandle = (LSAPR_HANDLE)PolicyObject;
//
//     TRACE("LsarOpenPolicy done!\n");
//
//     return Status;
// }
//
//
// /* Function 7 */
// NTSTATUS
// WINAPI
// LsarQueryInformationPolicy(
//     LSAPR_HANDLE PolicyHandle,
//     POLICY_INFORMATION_CLASS InformationClass,
//     PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     PLSA_DB_OBJECT PolicyObject;
//     ACCESS_MASK DesiredAccess = 0;
//     NTSTATUS Status;
//
//     TRACE("LsarQueryInformationPolicy(%p,0x%08x,%p)\n",
//           PolicyHandle, InformationClass, PolicyInformation);
//
//     if (PolicyInformation)
//     {
//         TRACE("*PolicyInformation %p\n", *PolicyInformation);
//     }
//
//     switch (InformationClass)
//     {
//         case PolicyAuditLogInformation:
//         case PolicyAuditEventsInformation:
//         case PolicyAuditFullQueryInformation:
//             DesiredAccess = POLICY_VIEW_AUDIT_INFORMATION;
//             break;
//
//         case PolicyPrimaryDomainInformation:
//         case PolicyAccountDomainInformation:
//         case PolicyLsaServerRoleInformation:
//         case PolicyReplicaSourceInformation:
//         case PolicyDefaultQuotaInformation:
//         case PolicyModificationInformation:
//         case PolicyDnsDomainInformation:
//         case PolicyDnsDomainInformationInt:
//         case PolicyLocalAccountDomainInformation:
//             DesiredAccess = POLICY_VIEW_LOCAL_INFORMATION;
//             break;
//
//         case PolicyPdAccountInformation:
//             DesiredAccess = POLICY_GET_PRIVATE_INFORMATION;
//             break;
//
//         default:
//             ERR("Invalid InformationClass!\n");
//             return STATUS_INVALID_PARAMETER;
//     }
//
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   DesiredAccess,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     switch (InformationClass)
//     {
//         case PolicyAuditLogInformation:      /* 1 */
//             Status = LsarQueryAuditLog(PolicyObject,
//                                        PolicyInformation);
//             break;
//
//         case PolicyAuditEventsInformation:   /* 2 */
//             Status = LsarQueryAuditEvents(PolicyObject,
//                                           PolicyInformation);
//             break;
//
//         case PolicyPrimaryDomainInformation: /* 3 */
//             Status = LsarQueryPrimaryDomain(PolicyObject,
//                                             PolicyInformation);
//             break;
//
//         case PolicyPdAccountInformation:     /* 4 */
//             Status = LsarQueryPdAccount(PolicyObject,
//                                         PolicyInformation);
//             break;
//
//         case PolicyAccountDomainInformation: /* 5 */
//             Status = LsarQueryAccountDomain(PolicyObject,
//                                             PolicyInformation);
//             break;
//
//         case PolicyLsaServerRoleInformation: /* 6 */
//             Status = LsarQueryServerRole(PolicyObject,
//                                          PolicyInformation);
//             break;
//
//         case PolicyReplicaSourceInformation: /* 7 */
//             Status = LsarQueryReplicaSource(PolicyObject,
//                                             PolicyInformation);
//             break;
//
//         case PolicyDefaultQuotaInformation:  /* 8 */
//             Status = LsarQueryDefaultQuota(PolicyObject,
//                                            PolicyInformation);
//             break;
//
//         case PolicyModificationInformation:  /* 9 */
//             Status = LsarQueryModification(PolicyObject,
//                                            PolicyInformation);
//             break;
//
//         case PolicyAuditFullQueryInformation: /* 11 (0xB) */
//             Status = LsarQueryAuditFull(PolicyObject,
//                                         PolicyInformation);
//             break;
//
//         case PolicyDnsDomainInformation:      /* 12 (0xC) */
//             Status = LsarQueryDnsDomain(PolicyObject,
//                                         PolicyInformation);
//             break;
//
//         case PolicyDnsDomainInformationInt:   /* 13 (0xD) */
//             Status = LsarQueryDnsDomainInt(PolicyObject,
//                                            PolicyInformation);
//             break;
//
//         case PolicyLocalAccountDomainInformation: /* 14 (0xE) */
//             Status = LsarQueryLocalAccountDomain(PolicyObject,
//                                                  PolicyInformation);
//             break;
//
//         default:
//             ERR("Invalid InformationClass!\n");
//             Status = STATUS_INVALID_PARAMETER;
//     }
//
//     return Status;
// }
//
//
// /* Function 8 */
// NTSTATUS
// WINAPI
// LsarSetInformationPolicy(
//     LSAPR_HANDLE PolicyHandle,
//     POLICY_INFORMATION_CLASS InformationClass,
//     PLSAPR_POLICY_INFORMATION PolicyInformation)
// {
//     PLSA_DB_OBJECT PolicyObject;
//     ACCESS_MASK DesiredAccess = 0;
//     NTSTATUS Status;
//
//     TRACE("LsarSetInformationPolicy(%p,0x%08x,%p)\n",
//           PolicyHandle, InformationClass, PolicyInformation);
//
//     if (PolicyInformation)
//     {
//         TRACE("*PolicyInformation %p\n", *PolicyInformation);
//     }
//
//     switch (InformationClass)
//     {
//         case PolicyAuditLogInformation:
//         case PolicyAuditFullSetInformation:
//             DesiredAccess = POLICY_AUDIT_LOG_ADMIN;
//             break;
//
//         case PolicyAuditEventsInformation:
//             DesiredAccess = POLICY_SET_AUDIT_REQUIREMENTS;
//             break;
//
//         case PolicyPrimaryDomainInformation:
//         case PolicyAccountDomainInformation:
//         case PolicyDnsDomainInformation:
//         case PolicyDnsDomainInformationInt:
//         case PolicyLocalAccountDomainInformation:
//             DesiredAccess = POLICY_TRUST_ADMIN;
//             break;
//
//         case PolicyLsaServerRoleInformation:
//         case PolicyReplicaSourceInformation:
//             DesiredAccess = POLICY_SERVER_ADMIN;
//             break;
//
//         case PolicyDefaultQuotaInformation:
//             DesiredAccess = POLICY_SET_DEFAULT_QUOTA_LIMITS;
//             break;
//
//         default:
//             ERR("Invalid InformationClass!\n");
//             return STATUS_INVALID_PARAMETER;
//     }
//
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   DesiredAccess,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     switch (InformationClass)
//     {
//         case PolicyAuditLogInformation:      /* 1 */
//             Status = LsarSetAuditLog(PolicyObject,
//                                      (PPOLICY_AUDIT_LOG_INFO)PolicyInformation);
//             break;
//
//         case PolicyAuditEventsInformation:   /* 2 */
//             Status = LsarSetAuditEvents(PolicyObject,
//                                         (PLSAPR_POLICY_AUDIT_EVENTS_INFO)PolicyInformation);
//             if (NT_SUCCESS(Status))
//                 LsapNotifyPolicyChange(PolicyNotifyAuditEventsInformation);
//             break;
//
//         case PolicyPrimaryDomainInformation: /* 3 */
//             Status = LsarSetPrimaryDomain(PolicyObject,
//                                           (PLSAPR_POLICY_PRIMARY_DOM_INFO)PolicyInformation);
//             if (NT_SUCCESS(Status))
//                 LsapNotifyPolicyChange(PolicyNotifyDnsDomainInformation);
//             break;
//
//         case PolicyAccountDomainInformation: /* 5 */
//             Status = LsarSetAccountDomain(PolicyObject,
//                                           (PLSAPR_POLICY_ACCOUNT_DOM_INFO)PolicyInformation);
//             if (NT_SUCCESS(Status))
//                 LsapNotifyPolicyChange(PolicyNotifyAccountDomainInformation);
//             break;
//
//         case PolicyLsaServerRoleInformation: /* 6 */
//             Status = LsarSetServerRole(PolicyObject,
//                                        (PPOLICY_LSA_SERVER_ROLE_INFO)PolicyInformation);
//             if (NT_SUCCESS(Status))
//                 LsapNotifyPolicyChange(PolicyNotifyServerRoleInformation);
//             break;
//
//         case PolicyReplicaSourceInformation: /* 7 */
//             Status = LsarSetReplicaSource(PolicyObject,
//                                           (PPOLICY_LSA_REPLICA_SRCE_INFO)PolicyInformation);
//             break;
//
//         case PolicyDefaultQuotaInformation:  /* 8 */
//             Status = LsarSetDefaultQuota(PolicyObject,
//                                          (PPOLICY_DEFAULT_QUOTA_INFO)PolicyInformation);
//             break;
//
//         case PolicyModificationInformation:  /* 9 */
//             Status = LsarSetModification(PolicyObject,
//                                          (PPOLICY_MODIFICATION_INFO)PolicyInformation);
//             break;
//
//         case PolicyAuditFullSetInformation:  /* 10 (0xA) */
//             Status = LsarSetAuditFull(PolicyObject,
//                                       (PPOLICY_AUDIT_FULL_QUERY_INFO)PolicyInformation);
//             break;
//
//         case PolicyDnsDomainInformation:      /* 12 (0xC) */
//             Status = LsarSetDnsDomain(PolicyObject,
//                                       (PLSAPR_POLICY_DNS_DOMAIN_INFO)PolicyInformation);
//             if (NT_SUCCESS(Status))
//                 LsapNotifyPolicyChange(PolicyNotifyDnsDomainInformation);
//             break;
//
//         case PolicyDnsDomainInformationInt:   /* 13 (0xD) */
//             Status = LsarSetDnsDomainInt(PolicyObject,
//                                          (PLSAPR_POLICY_DNS_DOMAIN_INFO)PolicyInformation);
//             break;
//
//         case PolicyLocalAccountDomainInformation: /* 14 (0xE) */
//             Status = LsarSetLocalAccountDomain(PolicyObject,
//                                                (PLSAPR_POLICY_ACCOUNT_DOM_INFO)PolicyInformation);
//             break;
//
//         default:
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//     }
//
//     return Status;
// }
//
//
// /* Function 9 */
// NTSTATUS
// WINAPI
// LsarClearAuditLog(
//     LSAPR_HANDLE ObjectHandle)
// {
//     /* Deprecated */
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// NTSTATUS
// LsarpCreateAccount(
//     PLSA_DB_OBJECT PolicyObject,
//     PRPC_SID AccountSid,
//     ACCESS_MASK DesiredAccess,
//     PLSA_DB_OBJECT *AccountObject)
// {
//     LPWSTR SidString = NULL;
//     PSECURITY_DESCRIPTOR AccountSd = NULL;
//     ULONG AccountSdSize;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Create SID string */
//     if (!ConvertSidToStringSid((PSID)AccountSid,
//                                &SidString))
//     {
//         ERR("ConvertSidToStringSid failed\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Create a security descriptor for the account */
//     Status = LsapCreateAccountSd(&AccountSd,
//                                  &AccountSdSize);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapCreateAccountSd returned 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Create the Account object */
//     Status = LsapCreateDbObject(PolicyObject,
//                                 L"Accounts",
//                                 SidString,
//                                 LsaDbAccountObject,
//                                 DesiredAccess,
//                                 PolicyObject->Trusted,
//                                 AccountObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapCreateDbObject failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Set the Sid attribute */
//     Status = LsapSetObjectAttribute(*AccountObject,
//                                     L"Sid",
//                                     (PVOID)AccountSid,
//                                     GetLengthSid(AccountSid));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the SecDesc attribute */
//     Status = LsapSetObjectAttribute(*AccountObject,
//                                     L"SecDesc",
//                                     AccountSd,
//                                     AccountSdSize);
//
// done:
//     if (SidString != NULL)
//         LocalFree(SidString);
//
//     if (AccountSd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AccountSd);
//
//     return Status;
// }
//
//
// /* Function 10 */
// NTSTATUS
// WINAPI
// LsarCreateAccount(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_SID AccountSid,
//     ACCESS_MASK DesiredAccess,
//     LSAPR_HANDLE *AccountHandle)
// {
//     PLSA_DB_OBJECT PolicyObject;
//     PLSA_DB_OBJECT AccountObject = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("LsarCreateAccount(%p %p %lx %p)\n",
//           PolicyHandle, AccountSid, DesiredAccess, AccountHandle);
//
//     /* Validate the AccountSid */
//     if (!RtlValidSid(AccountSid))
//         return STATUS_INVALID_PARAMETER;
//
//     /* Validate the PolicyHandle */
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   POLICY_CREATE_ACCOUNT,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//
//     Status = LsarpCreateAccount(PolicyObject,
//                                 AccountSid,
//                                 DesiredAccess,
//                                 &AccountObject);
//     if (NT_SUCCESS(Status))
//     {
//         *AccountHandle = (LSAPR_HANDLE)AccountObject;
//     }
//
//     return Status;
// }
//
//
// /* Function 11 */
// NTSTATUS
// WINAPI
// LsarEnumerateAccounts(
//     LSAPR_HANDLE PolicyHandle,
//     DWORD *EnumerationContext,
//     PLSAPR_ACCOUNT_ENUM_BUFFER EnumerationBuffer,
//     DWORD PreferedMaximumLength)
// {
//     LSAPR_ACCOUNT_ENUM_BUFFER EnumBuffer = {0, NULL};
//     PLSA_DB_OBJECT PolicyObject = NULL;
//     PWSTR AccountKeyBuffer = NULL;
//     HANDLE AccountsKeyHandle = NULL;
//     HANDLE AccountKeyHandle;
//     HANDLE SidKeyHandle;
//     ULONG AccountKeyBufferSize;
//     ULONG EnumIndex;
//     ULONG EnumCount;
//     ULONG RequiredLength;
//     ULONG DataLength;
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("LsarEnumerateAccount(%p %p %p %lu)\n",
//           PolicyHandle, EnumerationContext, EnumerationBuffer, PreferedMaximumLength);
//
//     if (EnumerationContext == NULL ||
//         EnumerationBuffer == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     EnumerationBuffer->EntriesRead = 0;
//     EnumerationBuffer->Information = NULL;
//
//     /* Validate the PolicyHandle */
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   POLICY_VIEW_LOCAL_INFORMATION,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     Status = LsapRegOpenKey(PolicyObject->KeyHandle,
//                             L"Accounts",
//                             KEY_READ,
//                             &AccountsKeyHandle);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = LsapRegQueryKeyInfo(AccountsKeyHandle,
//                                  NULL,
//                                  &AccountKeyBufferSize,
//                                  NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapRegQueryKeyInfo returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     AccountKeyBufferSize += sizeof(WCHAR);
//     AccountKeyBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, AccountKeyBufferSize);
//     if (AccountKeyBuffer == NULL)
//     {
//         return STATUS_NO_MEMORY;
//     }
//
//     EnumIndex = *EnumerationContext;
//     EnumCount = 0;
//     RequiredLength = 0;
//
//     while (TRUE)
//     {
//         Status = LsapRegEnumerateSubKey(AccountsKeyHandle,
//                                         EnumIndex,
//                                         AccountKeyBufferSize,
//                                         AccountKeyBuffer);
//         if (!NT_SUCCESS(Status))
//             break;
//
//         TRACE("EnumIndex: %lu\n", EnumIndex);
//         TRACE("Account key name: %S\n", AccountKeyBuffer);
//
//         Status = LsapRegOpenKey(AccountsKeyHandle,
//                                 AccountKeyBuffer,
//                                 KEY_READ,
//                                 &AccountKeyHandle);
//         TRACE("LsapRegOpenKey returned %08lX\n", Status);
//         if (NT_SUCCESS(Status))
//         {
//             Status = LsapRegOpenKey(AccountKeyHandle,
//                                     L"Sid",
//                                     KEY_READ,
//                                     &SidKeyHandle);
//             TRACE("LsapRegOpenKey returned %08lX\n", Status);
//             if (NT_SUCCESS(Status))
//             {
//                 DataLength = 0;
//                 Status = LsapRegQueryValue(SidKeyHandle,
//                                            NULL,
//                                            NULL,
//                                            NULL,
//                                            &DataLength);
//                 TRACE("LsapRegQueryValue returned %08lX\n", Status);
//                 if (NT_SUCCESS(Status))
//                 {
//                     TRACE("Data length: %lu\n", DataLength);
//
//                     if ((RequiredLength + DataLength + sizeof(LSAPR_ACCOUNT_INFORMATION)) > PreferedMaximumLength)
//                         break;
//
//                     RequiredLength += (DataLength + sizeof(LSAPR_ACCOUNT_INFORMATION));
//                     EnumCount++;
//                 }
//
//                 LsapRegCloseKey(SidKeyHandle);
//             }
//
//             LsapRegCloseKey(AccountKeyHandle);
//         }
//
//         EnumIndex++;
//     }
//
//     TRACE("EnumCount: %lu\n", EnumCount);
//     TRACE("RequiredLength: %lu\n", RequiredLength);
//
//     EnumBuffer.EntriesRead = EnumCount;
//     EnumBuffer.Information = midl_user_allocate(EnumCount * sizeof(LSAPR_ACCOUNT_INFORMATION));
//     if (EnumBuffer.Information == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     EnumIndex = *EnumerationContext;
//     for (i = 0; i < EnumCount; i++, EnumIndex++)
//     {
//         Status = LsapRegEnumerateSubKey(AccountsKeyHandle,
//                                         EnumIndex,
//                                         AccountKeyBufferSize,
//                                         AccountKeyBuffer);
//         if (!NT_SUCCESS(Status))
//             break;
//
//         TRACE("EnumIndex: %lu\n", EnumIndex);
//         TRACE("Account key name: %S\n", AccountKeyBuffer);
//
//         Status = LsapRegOpenKey(AccountsKeyHandle,
//                                 AccountKeyBuffer,
//                                 KEY_READ,
//                                 &AccountKeyHandle);
//         TRACE("LsapRegOpenKey returned %08lX\n", Status);
//         if (NT_SUCCESS(Status))
//         {
//             Status = LsapRegOpenKey(AccountKeyHandle,
//                                     L"Sid",
//                                     KEY_READ,
//                                     &SidKeyHandle);
//             TRACE("LsapRegOpenKey returned %08lX\n", Status);
//             if (NT_SUCCESS(Status))
//             {
//                 DataLength = 0;
//                 Status = LsapRegQueryValue(SidKeyHandle,
//                                            NULL,
//                                            NULL,
//                                            NULL,
//                                            &DataLength);
//                 TRACE("LsapRegQueryValue returned %08lX\n", Status);
//                 if (NT_SUCCESS(Status))
//                 {
//                     EnumBuffer.Information[i].Sid = midl_user_allocate(DataLength);
//                     if (EnumBuffer.Information[i].Sid == NULL)
//                     {
//                         LsapRegCloseKey(AccountKeyHandle);
//                         Status = STATUS_INSUFFICIENT_RESOURCES;
//                         goto done;
//                     }
//
//                     Status = LsapRegQueryValue(SidKeyHandle,
//                                                NULL,
//                                                NULL,
//                                                EnumBuffer.Information[i].Sid,
//                                                &DataLength);
//                     TRACE("SampRegQueryValue returned %08lX\n", Status);
//                 }
//
//                 LsapRegCloseKey(SidKeyHandle);
//             }
//
//             LsapRegCloseKey(AccountKeyHandle);
//
//             if (!NT_SUCCESS(Status))
//                 goto done;
//         }
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         *EnumerationContext += EnumCount;
//         EnumerationBuffer->EntriesRead = EnumBuffer.EntriesRead;
//         EnumerationBuffer->Information = EnumBuffer.Information;
//     }
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (EnumBuffer.Information)
//         {
//             for (i = 0; i < EnumBuffer.EntriesRead; i++)
//             {
//                 if (EnumBuffer.Information[i].Sid != NULL)
//                     midl_user_free(EnumBuffer.Information[i].Sid);
//             }
//
//             midl_user_free(EnumBuffer.Information);
//         }
//     }
//
//     if (AccountKeyBuffer != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AccountKeyBuffer);
//
//     if (AccountsKeyHandle != NULL)
//         LsapRegCloseKey(AccountsKeyHandle);
//
//     return Status;
// }
//
//
// /* Function 12 */
// NTSTATUS
// WINAPI
// LsarCreateTrustedDomain(
//     LSAPR_HANDLE PolicyHandle,
//     PLSAPR_TRUST_INFORMATION TrustedDomainInformation,
//     ACCESS_MASK DesiredAccess,
//     LSAPR_HANDLE *TrustedDomainHandle)
// {
//     /* Fail, if we are not a domain controller */
//     if (LsapProductType != NtProductLanManNt)
//         return STATUS_DIRECTORY_SERVICE_REQUIRED;
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 13 */
// NTSTATUS
// WINAPI
// LsarEnumerateTrustedDomains(
//     LSAPR_HANDLE PolicyHandle,
//     DWORD *EnumerationContext,
//     PLSAPR_TRUSTED_ENUM_BUFFER EnumerationBuffer,
//     DWORD PreferedMaximumLength)
// {
//     /* FIXME: We are not running an AD yet */
//     EnumerationBuffer->EntriesRead = 0;
//     EnumerationBuffer->Information = NULL;
//     return STATUS_NO_MORE_ENTRIES;
// }
//
//
// /* Function 14 */
// NTSTATUS
// WINAPI
// LsarLookupNames(
//     LSAPR_HANDLE PolicyHandle,
//     DWORD Count,
//     PRPC_UNICODE_STRING Names,
//     PLSAPR_REFERENCED_DOMAIN_LIST *ReferencedDomains,
//     PLSAPR_TRANSLATED_SIDS TranslatedSids,
//     LSAP_LOOKUP_LEVEL LookupLevel,
//     DWORD *MappedCount)
// {
//     LSAPR_TRANSLATED_SIDS_EX2 TranslatedSidsEx2;
//     ULONG i;
//     NTSTATUS Status;
//
//     TRACE("LsarLookupNames(%p %lu %p %p %p %d %p)\n",
//           PolicyHandle, Count, Names, ReferencedDomains, TranslatedSids,
//           LookupLevel, MappedCount);
//
//     TranslatedSids->Entries = 0;
//     TranslatedSids->Sids = NULL;
//     *ReferencedDomains = NULL;
//
//     if (Count == 0)
//         return STATUS_NONE_MAPPED;
//
//     TranslatedSidsEx2.Entries = 0;
//     TranslatedSidsEx2.Sids = NULL;
//
//     Status = LsapLookupNames(Count,
//                              Names,
//                              ReferencedDomains,
//                              &TranslatedSidsEx2,
//                              LookupLevel,
//                              MappedCount,
//                              0,
//                              0);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     TranslatedSids->Entries = TranslatedSidsEx2.Entries;
//     TranslatedSids->Sids = MIDL_user_allocate(TranslatedSids->Entries * sizeof(LSA_TRANSLATED_SID));
//     if (TranslatedSids->Sids == NULL)
//     {
//         MIDL_user_free(TranslatedSidsEx2.Sids);
//         MIDL_user_free(*ReferencedDomains);
//         *ReferencedDomains = NULL;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     for (i = 0; i < TranslatedSidsEx2.Entries; i++)
//     {
//         TranslatedSids->Sids[i].Use = TranslatedSidsEx2.Sids[i].Use;
//         TranslatedSids->Sids[i].RelativeId = LsapGetRelativeIdFromSid(TranslatedSidsEx2.Sids[i].Sid);
//         TranslatedSids->Sids[i].DomainIndex = TranslatedSidsEx2.Sids[i].DomainIndex;
//     }
//
//     MIDL_user_free(TranslatedSidsEx2.Sids);
//
//     return STATUS_SUCCESS;
// }
//
//
// /* Function 15 */
// NTSTATUS
// WINAPI
// LsarLookupSids(
//     LSAPR_HANDLE PolicyHandle,
//     PLSAPR_SID_ENUM_BUFFER SidEnumBuffer,
//     PLSAPR_REFERENCED_DOMAIN_LIST *ReferencedDomains,
//     PLSAPR_TRANSLATED_NAMES TranslatedNames,
//     LSAP_LOOKUP_LEVEL LookupLevel,
//     DWORD *MappedCount)
// {
//     LSAPR_TRANSLATED_NAMES_EX TranslatedNamesEx;
//     ULONG i;
//     NTSTATUS Status;
//
//     TRACE("LsarLookupSids(%p %p %p %p %d %p)\n",
//           PolicyHandle, SidEnumBuffer, ReferencedDomains, TranslatedNames,
//           LookupLevel, MappedCount);
//
//     /* FIXME: Fail, if there is an invalid SID in the SidEnumBuffer */
//
//     TranslatedNames->Entries = SidEnumBuffer->Entries;
//     TranslatedNames->Names = NULL;
//     *ReferencedDomains = NULL;
//
//     TranslatedNamesEx.Entries = SidEnumBuffer->Entries;
//     TranslatedNamesEx.Names = NULL;
//
//     Status = LsapLookupSids(SidEnumBuffer,
//                             ReferencedDomains,
//                             &TranslatedNamesEx,
//                             LookupLevel,
//                             MappedCount,
//                             0,
//                             0);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     TranslatedNames->Entries = SidEnumBuffer->Entries;
//     TranslatedNames->Names = MIDL_user_allocate(SidEnumBuffer->Entries * sizeof(LSAPR_TRANSLATED_NAME));
//     if (TranslatedNames->Names == NULL)
//     {
//         MIDL_user_free(TranslatedNamesEx.Names);
//         MIDL_user_free(*ReferencedDomains);
//         *ReferencedDomains = NULL;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     for (i = 0; i < TranslatedNamesEx.Entries; i++)
//     {
//         TranslatedNames->Names[i].Use = TranslatedNamesEx.Names[i].Use;
//         TranslatedNames->Names[i].Name.Length = TranslatedNamesEx.Names[i].Name.Length;
//         TranslatedNames->Names[i].Name.MaximumLength = TranslatedNamesEx.Names[i].Name.MaximumLength;
//         TranslatedNames->Names[i].Name.Buffer = TranslatedNamesEx.Names[i].Name.Buffer;
//         TranslatedNames->Names[i].DomainIndex = TranslatedNamesEx.Names[i].DomainIndex;
//     }
//
//     MIDL_user_free(TranslatedNamesEx.Names);
//
//     return Status;
// }
//
//
// /* Function 16 */
// NTSTATUS
// WINAPI
// LsarCreateSecret(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_UNICODE_STRING SecretName,
//     ACCESS_MASK DesiredAccess,
//     LSAPR_HANDLE *SecretHandle)
// {
//     PLSA_DB_OBJECT PolicyObject;
//     PLSA_DB_OBJECT SecretObject = NULL;
//     LARGE_INTEGER Time;
//     PSECURITY_DESCRIPTOR SecretSd = NULL;
//     ULONG SecretSdSize;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("LsarCreateSecret(%p %wZ %lx %p)\n",
//           PolicyHandle, SecretName, DesiredAccess, SecretHandle);
//
//     /* Validate the PolicyHandle */
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   POLICY_CREATE_SECRET,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Get the current time */
//     Status = NtQuerySystemTime(&Time);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtQuerySystemTime failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Create a security descriptor for the secret */
//     Status = LsapCreateSecretSd(&SecretSd,
//                                 &SecretSdSize);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapCreateAccountSd returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Create the Secret object */
//     Status = LsapCreateDbObject(PolicyObject,
//                                 L"Secrets",
//                                 SecretName->Buffer,
//                                 LsaDbSecretObject,
//                                 DesiredAccess,
//                                 PolicyObject->Trusted,
//                                 &SecretObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapCreateDbObject failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Set the CurrentTime attribute */
//     Status = LsapSetObjectAttribute(SecretObject,
//                                     L"CurrentTime",
//                                     (PVOID)&Time,
//                                     sizeof(LARGE_INTEGER));
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapSetObjectAttribute (CurrentTime) failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Set the OldTime attribute */
//     Status = LsapSetObjectAttribute(SecretObject,
//                                     L"OldTime",
//                                     (PVOID)&Time,
//                                     sizeof(LARGE_INTEGER));
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapSetObjectAttribute (OldTime) failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Set the SecDesc attribute */
//     Status = LsapSetObjectAttribute(SecretObject,
//                                     L"SecDesc",
//                                     SecretSd,
//                                     SecretSdSize);
//
// done:
//     if (SecretSd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, SecretSd);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (SecretObject != NULL)
//             LsapCloseDbObject(SecretObject);
//     }
//     else
//     {
//         *SecretHandle = (LSAPR_HANDLE)SecretObject;
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// NTSTATUS
// LsarpOpenAccount(
//     IN PLSA_DB_OBJECT PolicyObject,
//     IN PRPC_SID AccountSid,
//     IN ACCESS_MASK DesiredAccess,
//     OUT PLSA_DB_OBJECT *AccountObject)
// {
//     LPWSTR SidString = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Create SID string */
//     if (!ConvertSidToStringSid((PSID)AccountSid,
//                                &SidString))
//     {
//         ERR("ConvertSidToStringSid failed\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Create the Account object */
//     Status = LsapOpenDbObject(PolicyObject,
//                               L"Accounts",
//                               SidString,
//                               LsaDbAccountObject,
//                               DesiredAccess,
//                               PolicyObject->Trusted,
//                               AccountObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapOpenDbObject(Accounts/%S) failed (Status 0x%08lx)\n", SidString, Status);
//     }
//
//     if (SidString != NULL)
//         LocalFree(SidString);
//
//     return Status;
// }
//
//
// /* Function 17 */
// NTSTATUS
// WINAPI
// LsarOpenAccount(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_SID AccountSid,
//     ACCESS_MASK DesiredAccess,
//     LSAPR_HANDLE *AccountHandle)
// {
//     PLSA_DB_OBJECT PolicyObject;
//     NTSTATUS Status;
//
//     TRACE("LsarOpenAccount(%p %p %lx %p)\n",
//           PolicyHandle, AccountSid, DesiredAccess, AccountHandle);
//
//     /* Validate the AccountSid */
//     if (!RtlValidSid(AccountSid))
//         return STATUS_INVALID_PARAMETER;
//
//     /* Validate the PolicyHandle */
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   0,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//
//     /* Open the Account object */
//     return LsarpOpenAccount(PolicyObject,
//                             AccountSid,
//                             DesiredAccess,
//                             (PLSA_DB_OBJECT *)AccountHandle);
// }
//
//
// /* Function 18 */
// NTSTATUS
// WINAPI
// LsarEnumeratePrivilegesAccount(
//     LSAPR_HANDLE AccountHandle,
//     PLSAPR_PRIVILEGE_SET *Privileges)
// {
//     PLSA_DB_OBJECT AccountObject;
//     ULONG PrivilegeSetSize = 0;
//     PLSAPR_PRIVILEGE_SET PrivilegeSet = NULL;
//     NTSTATUS Status;
//
//     TRACE("LsarEnumeratePrivilegesAccount(%p %p)\n",
//           AccountHandle, Privileges);
//
//     *Privileges = NULL;
//
//     /* Validate the AccountHandle */
//     Status = LsapValidateDbObject(AccountHandle,
//                                   LsaDbAccountObject,
//                                   ACCOUNT_VIEW,
//                                   &AccountObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Get the size of the privilege set */
//     Status = LsapGetObjectAttribute(AccountObject,
//                                     L"Privilgs",
//                                     NULL,
//                                     &PrivilegeSetSize);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Allocate a buffer for the privilege set */
//     PrivilegeSet = MIDL_user_allocate(PrivilegeSetSize);
//     if (PrivilegeSet == NULL)
//         return STATUS_NO_MEMORY;
//
//     /* Get the privilege set */
//     Status = LsapGetObjectAttribute(AccountObject,
//                                     L"Privilgs",
//                                     PrivilegeSet,
//                                     &PrivilegeSetSize);
//     if (!NT_SUCCESS(Status))
//     {
//         MIDL_user_free(PrivilegeSet);
//         return Status;
//     }
//
//     /* Return a pointer to the privilege set */
//     *Privileges = PrivilegeSet;
//
//     return STATUS_SUCCESS;
// }
//
//
// /* Function 19 */
// NTSTATUS
// WINAPI
// LsarAddPrivilegesToAccount(
//     LSAPR_HANDLE AccountHandle,
//     PLSAPR_PRIVILEGE_SET Privileges)
// {
//     PLSA_DB_OBJECT AccountObject;
//     PPRIVILEGE_SET CurrentPrivileges = NULL;
//     PPRIVILEGE_SET NewPrivileges = NULL;
//     ULONG PrivilegeSetSize = 0;
//     ULONG PrivilegeCount;
//     ULONG i, j;
//     BOOL bFound;
//     NTSTATUS Status;
//
//     TRACE("LsarAddPrivilegesToAccount(%p %p)\n",
//           AccountHandle, Privileges);
//
//     /* Validate the AccountHandle */
//     Status = LsapValidateDbObject(AccountHandle,
//                                   LsaDbAccountObject,
//                                   ACCOUNT_ADJUST_PRIVILEGES,
//                                   &AccountObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Get the size of the Privilgs attribute */
//     Status = LsapGetObjectAttribute(AccountObject,
//                                     L"Privilgs",
//                                     NULL,
//                                     &PrivilegeSetSize);
//     if (!NT_SUCCESS(Status) || PrivilegeSetSize == 0)
//     {
//         /* The Privilgs attribute does not exist */
//
//         PrivilegeSetSize = sizeof(PRIVILEGE_SET) +
//                            (Privileges->PrivilegeCount - 1) * sizeof(LUID_AND_ATTRIBUTES);
//         Status = LsapSetObjectAttribute(AccountObject,
//                                         L"Privilgs",
//                                         Privileges,
//                                         PrivilegeSetSize);
//     }
//     else
//     {
//         /* The Privilgs attribute exists */
//
//         /* Allocate memory for the stored privilege set */
//         CurrentPrivileges = MIDL_user_allocate(PrivilegeSetSize);
//         if (CurrentPrivileges == NULL)
//             return STATUS_NO_MEMORY;
//
//         /* Get the current privilege set */
//         Status = LsapGetObjectAttribute(AccountObject,
//                                         L"Privilgs",
//                                         CurrentPrivileges,
//                                         &PrivilegeSetSize);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("LsapGetObjectAttribute() failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         PrivilegeCount = CurrentPrivileges->PrivilegeCount;
//         TRACE("Current privilege count: %lu\n", PrivilegeCount);
//
//         /* Calculate the number of privileges in the combined privilege set */
//         for (i = 0; i < Privileges->PrivilegeCount; i++)
//         {
//             bFound = FALSE;
//             for (j = 0; j < CurrentPrivileges->PrivilegeCount; j++)
//             {
//                 if (RtlEqualLuid(&(Privileges->Privilege[i].Luid),
//                                  &(CurrentPrivileges->Privilege[i].Luid)))
//                 {
//                     bFound = TRUE;
//                     break;
//                 }
//             }
//
//             if (bFound == FALSE)
//             {
//                 TRACE("Found new privilege\n");
//                 PrivilegeCount++;
//             }
//         }
//         TRACE("New privilege count: %lu\n", PrivilegeCount);
//
//         /* Calculate the size of the new privilege set and allocate it */
//         PrivilegeSetSize = sizeof(PRIVILEGE_SET) +
//                            (PrivilegeCount - 1) * sizeof(LUID_AND_ATTRIBUTES);
//         NewPrivileges = MIDL_user_allocate(PrivilegeSetSize);
//         if (NewPrivileges == NULL)
//         {
//             Status = STATUS_NO_MEMORY;
//             goto done;
//         }
//
//         /* Initialize the new privilege set */
//         NewPrivileges->PrivilegeCount = PrivilegeCount;
//         NewPrivileges->Control = 0;
//
//         /* Copy all privileges from the current privilege set */
//         RtlCopyLuidAndAttributesArray(CurrentPrivileges->PrivilegeCount,
//                                       &(CurrentPrivileges->Privilege[0]),
//                                       &(NewPrivileges->Privilege[0]));
//
//         /* Add new privileges to the new privilege set */
//         PrivilegeCount = CurrentPrivileges->PrivilegeCount;
//         for (i = 0; i < Privileges->PrivilegeCount; i++)
//         {
//             bFound = FALSE;
//             for (j = 0; j < CurrentPrivileges->PrivilegeCount; j++)
//             {
//                 if (RtlEqualLuid(&(Privileges->Privilege[i].Luid),
//                                  &(CurrentPrivileges->Privilege[i].Luid)))
//                 {
//                     /* Overwrite attributes if a matching privilege was found */
//                     NewPrivileges->Privilege[j].Attributes = Privileges->Privilege[i].Attributes;
//
//                     bFound = TRUE;
//                     break;
//                 }
//             }
//
//             if (bFound == FALSE)
//             {
//                 /* Copy the new privilege */
//                 RtlCopyLuidAndAttributesArray(1,
//                                               (PLUID_AND_ATTRIBUTES)&(Privileges->Privilege[i]),
//                                               &(NewPrivileges->Privilege[PrivilegeCount]));
//                 PrivilegeCount++;
//             }
//         }
//
//         /* Set the new privilege set */
//         Status = LsapSetObjectAttribute(AccountObject,
//                                         L"Privilgs",
//                                         NewPrivileges,
//                                         PrivilegeSetSize);
//     }
//
// done:
//     if (CurrentPrivileges != NULL)
//         MIDL_user_free(CurrentPrivileges);
//
//     if (NewPrivileges != NULL)
//         MIDL_user_free(NewPrivileges);
//
//     return Status;
// }
//
//
// /* Function 20 */
// NTSTATUS
// WINAPI
// LsarRemovePrivilegesFromAccount(
//     LSAPR_HANDLE AccountHandle,
//     BOOLEAN AllPrivileges,
//     PLSAPR_PRIVILEGE_SET Privileges)
// {
//     PLSA_DB_OBJECT AccountObject;
//     PPRIVILEGE_SET CurrentPrivileges = NULL;
//     PPRIVILEGE_SET NewPrivileges = NULL;
//     ULONG PrivilegeSetSize = 0;
//     ULONG PrivilegeCount;
//     ULONG i, j, k;
//     BOOL bFound;
//     NTSTATUS Status;
//
//     TRACE("LsarRemovePrivilegesFromAccount(%p %u %p)\n",
//           AccountHandle, AllPrivileges, Privileges);
//
//     /* */
//     if (((AllPrivileges == FALSE) && (Privileges == NULL)) ||
//         ((AllPrivileges != FALSE) && (Privileges != NULL)))
//             return STATUS_INVALID_PARAMETER;
//
//     /* Validate the AccountHandle */
//     Status = LsapValidateDbObject(AccountHandle,
//                                   LsaDbAccountObject,
//                                   ACCOUNT_ADJUST_PRIVILEGES,
//                                   &AccountObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     if (AllPrivileges != FALSE)
//     {
//         /* Delete the Privilgs attribute */
//         Status = LsapDeleteObjectAttribute(AccountObject,
//                                            L"Privilgs");
//         if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//             Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Get the size of the Privilgs attribute */
//         Status = LsapGetObjectAttribute(AccountObject,
//                                         L"Privilgs",
//                                         NULL,
//                                         &PrivilegeSetSize);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         /* Succeed, if there is no privilege set to remove privileges from */
//         if (PrivilegeSetSize == 0)
//         {
//             Status = STATUS_SUCCESS;
//             goto done;
//         }
//
//         /* Allocate memory for the stored privilege set */
//         CurrentPrivileges = MIDL_user_allocate(PrivilegeSetSize);
//         if (CurrentPrivileges == NULL)
//             return STATUS_NO_MEMORY;
//
//         /* Get the current privilege set */
//         Status = LsapGetObjectAttribute(AccountObject,
//                                         L"Privilgs",
//                                         CurrentPrivileges,
//                                         &PrivilegeSetSize);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("LsapGetObjectAttribute() failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         PrivilegeCount = CurrentPrivileges->PrivilegeCount;
//         TRACE("Current privilege count: %lu\n", PrivilegeCount);
//
//         /* Calculate the number of privileges in the new privilege set */
//         for (i = 0; i < CurrentPrivileges->PrivilegeCount; i++)
//         {
//             for (j = 0; j < Privileges->PrivilegeCount; j++)
//             {
//                 if (RtlEqualLuid(&(CurrentPrivileges->Privilege[i].Luid),
//                                  &(Privileges->Privilege[j].Luid)))
//                 {
//                     if (PrivilegeCount > 0)
//                         PrivilegeCount--;
//                 }
//             }
//         }
//         TRACE("New privilege count: %lu\n", PrivilegeCount);
//
//         if (PrivilegeCount == 0)
//         {
//             /* Delete the Privilgs attribute */
//             Status = LsapDeleteObjectAttribute(AccountObject,
//                                                L"Privilgs");
//             if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//                 Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* Calculate the size of the new privilege set and allocate it */
//             PrivilegeSetSize = sizeof(PRIVILEGE_SET) +
//                                (PrivilegeCount - 1) * sizeof(LUID_AND_ATTRIBUTES);
//             NewPrivileges = MIDL_user_allocate(PrivilegeSetSize);
//             if (NewPrivileges == NULL)
//             {
//                 Status = STATUS_NO_MEMORY;
//                 goto done;
//             }
//
//             /* Initialize the new privilege set */
//             NewPrivileges->PrivilegeCount = PrivilegeCount;
//             NewPrivileges->Control = 0;
//
//             /* Copy the privileges which are not to be removed */
//             for (i = 0, k = 0; i < CurrentPrivileges->PrivilegeCount; i++)
//             {
//                 bFound = FALSE;
//                 for (j = 0; j < Privileges->PrivilegeCount; j++)
//                 {
//                     if (RtlEqualLuid(&(CurrentPrivileges->Privilege[i].Luid),
//                                      &(Privileges->Privilege[j].Luid)))
//                         bFound = TRUE;
//                 }
//
//                 if (bFound == FALSE)
//                 {
//                     /* Copy the privilege */
//                     RtlCopyLuidAndAttributesArray(1,
//                                                   &(CurrentPrivileges->Privilege[i]),
//                                                   &(NewPrivileges->Privilege[k]));
//                     k++;
//                 }
//             }
//
//             /* Set the new privilege set */
//             Status = LsapSetObjectAttribute(AccountObject,
//                                             L"Privilgs",
//                                             NewPrivileges,
//                                             PrivilegeSetSize);
//         }
//     }
//
// done:
//     if (CurrentPrivileges != NULL)
//         MIDL_user_free(CurrentPrivileges);
//
//     if (NewPrivileges != NULL)
//         MIDL_user_free(NewPrivileges);
//
//     return Status;
// }
//
//
// /* Function 21 */
// NTSTATUS
// WINAPI
// LsarGetQuotasForAccount(
//     LSAPR_HANDLE AccountHandle,
//     PQUOTA_LIMITS QuotaLimits)
// {
//     PLSA_DB_OBJECT AccountObject;
//     ULONG Size;
//     NTSTATUS Status;
//
//     TRACE("LsarGetQuotasForAccount(%p %p)\n",
//           AccountHandle, QuotaLimits);
//
//     /* Validate the account handle */
//     Status = LsapValidateDbObject(AccountHandle,
//                                   LsaDbAccountObject,
//                                   ACCOUNT_VIEW,
//                                   &AccountObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Invalid handle (Status %lx)\n", Status);
//         return Status;
//     }
//
//     /* Get the quota attribute */
//     Status = LsapGetObjectAttribute(AccountObject,
//                                     L"DefQuota",
//                                     QuotaLimits,
//                                     &Size);
//
//     return Status;
// }
//
//
// /* Function 22 */
// NTSTATUS
// WINAPI
// LsarSetQuotasForAccount(
//     LSAPR_HANDLE AccountHandle,
//     PQUOTA_LIMITS QuotaLimits)
// {
//     PLSA_DB_OBJECT AccountObject;
//     QUOTA_LIMITS InternalQuotaLimits;
//     ULONG Size;
//     NTSTATUS Status;
//
//     TRACE("LsarSetQuotasForAccount(%p %p)\n",
//           AccountHandle, QuotaLimits);
//
//     /* Validate the account handle */
//     Status = LsapValidateDbObject(AccountHandle,
//                                   LsaDbAccountObject,
//                                   ACCOUNT_ADJUST_QUOTAS,
//                                   &AccountObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Invalid handle (Status %lx)\n", Status);
//         return Status;
//     }
//
//     /* Get the quota limits attribute */
//     Size = sizeof(QUOTA_LIMITS);
//     Status = LsapGetObjectAttribute(AccountObject,
//                                     L"DefQuota",
//                                     &InternalQuotaLimits,
//                                     &Size);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("LsapGetObjectAttribute() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Update the quota limits */
//     if (QuotaLimits->PagedPoolLimit != 0)
//         InternalQuotaLimits.PagedPoolLimit = QuotaLimits->PagedPoolLimit;
//
//     if (QuotaLimits->NonPagedPoolLimit != 0)
//         InternalQuotaLimits.NonPagedPoolLimit = QuotaLimits->NonPagedPoolLimit;
//
//     if (QuotaLimits->MinimumWorkingSetSize != 0)
//         InternalQuotaLimits.MinimumWorkingSetSize = QuotaLimits->MinimumWorkingSetSize;
//
//     if (QuotaLimits->MaximumWorkingSetSize != 0)
//         InternalQuotaLimits.MaximumWorkingSetSize = QuotaLimits->MaximumWorkingSetSize;
//
//     if (QuotaLimits->PagefileLimit != 0)
//         InternalQuotaLimits.PagefileLimit = QuotaLimits->PagefileLimit;
//
//     /* Set the quota limits attribute */
//     Status = LsapSetObjectAttribute(AccountObject,
//                                     L"DefQuota",
//                                     &InternalQuotaLimits,
//                                     sizeof(QUOTA_LIMITS));
//
//     return Status;
// }
//
//
// /* Function 23 */
// NTSTATUS
// WINAPI
// LsarGetSystemAccessAccount(
//     LSAPR_HANDLE AccountHandle,
//     ACCESS_MASK *SystemAccess)
// {
//     PLSA_DB_OBJECT AccountObject;
//     ULONG Size = sizeof(ACCESS_MASK);
//     NTSTATUS Status;
//
//     TRACE("LsarGetSystemAccessAccount(%p %p)\n",
//           AccountHandle, SystemAccess);
//
//     /* Validate the account handle */
//     Status = LsapValidateDbObject(AccountHandle,
//                                   LsaDbAccountObject,
//                                   ACCOUNT_VIEW,
//                                   &AccountObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Invalid handle (Status %lx)\n", Status);
//         return Status;
//     }
//
//     /* Get the system access flags */
//     Status = LsapGetObjectAttribute(AccountObject,
//                                     L"ActSysAc",
//                                     SystemAccess,
//                                     &Size);
//
//     return Status;
// }
//
//
// /* Function 24 */
// NTSTATUS
// WINAPI
// LsarSetSystemAccessAccount(
//     LSAPR_HANDLE AccountHandle,
//     ACCESS_MASK SystemAccess)
// {
//     PLSA_DB_OBJECT AccountObject;
//     NTSTATUS Status;
//
//     TRACE("LsarSetSystemAccessAccount(%p %lx)\n",
//           AccountHandle, SystemAccess);
//
//     /* Validate the account handle */
//     Status = LsapValidateDbObject(AccountHandle,
//                                   LsaDbAccountObject,
//                                   ACCOUNT_ADJUST_SYSTEM_ACCESS,
//                                   &AccountObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Invalid handle (Status %lx)\n", Status);
//         return Status;
//     }
//
//     /* Set the system access flags */
//     Status = LsapSetObjectAttribute(AccountObject,
//                                     L"ActSysAc",
//                                     &SystemAccess,
//                                     sizeof(ACCESS_MASK));
//
//     return Status;
// }
//
//
// /* Function 25 */
// NTSTATUS
// WINAPI
// LsarOpenTrustedDomain(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_SID TrustedDomainSid,
//     ACCESS_MASK DesiredAccess,
//     LSAPR_HANDLE *TrustedDomainHandle)
// {
//     /* Fail, if we are not a domain controller */
//     if (LsapProductType != NtProductLanManNt)
//         return STATUS_DIRECTORY_SERVICE_REQUIRED;
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 26 */
// NTSTATUS
// WINAPI
// LsarQueryInfoTrustedDomain(
//     LSAPR_HANDLE TrustedDomainHandle,
//     TRUSTED_INFORMATION_CLASS InformationClass,
//     PLSAPR_TRUSTED_DOMAIN_INFO *TrustedDomainInformation)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 27 */
// NTSTATUS
// WINAPI
// LsarSetInformationTrustedDomain(
//     LSAPR_HANDLE TrustedDomainHandle,
//     TRUSTED_INFORMATION_CLASS InformationClass,
//     PLSAPR_TRUSTED_DOMAIN_INFO TrustedDomainInformation)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 28 */
// NTSTATUS
// WINAPI
// LsarOpenSecret(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_UNICODE_STRING SecretName,
//     ACCESS_MASK DesiredAccess,
//     LSAPR_HANDLE *SecretHandle)
// {
//     PLSA_DB_OBJECT PolicyObject;
//     PLSA_DB_OBJECT SecretObject = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("LsarOpenSecret(%p %wZ %lx %p)\n",
//           PolicyHandle, SecretName, DesiredAccess, SecretHandle);
//
//     /* Validate the PolicyHandle */
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   0,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Create the secret object */
//     Status = LsapOpenDbObject(PolicyObject,
//                               L"Secrets",
//                               SecretName->Buffer,
//                               LsaDbSecretObject,
//                               DesiredAccess,
//                               PolicyObject->Trusted,
//                               &SecretObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapOpenDbObject failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (SecretObject != NULL)
//             LsapCloseDbObject(SecretObject);
//     }
//     else
//     {
//         *SecretHandle = (LSAPR_HANDLE)SecretObject;
//     }
//
//     return Status;
// }
//
//
// /* Function 29 */
// NTSTATUS
// WINAPI
// LsarSetSecret(
//     LSAPR_HANDLE SecretHandle,
//     PLSAPR_CR_CIPHER_VALUE EncryptedCurrentValue,
//     PLSAPR_CR_CIPHER_VALUE EncryptedOldValue)
// {
//     PLSA_DB_OBJECT SecretObject;
//     PBYTE CurrentValue = NULL;
//     PBYTE OldValue = NULL;
//     ULONG CurrentValueLength = 0;
//     ULONG OldValueLength = 0;
//     LARGE_INTEGER Time;
//     NTSTATUS Status;
//
//     TRACE("LsarSetSecret(%p %p %p)\n", SecretHandle,
//           EncryptedCurrentValue, EncryptedOldValue);
//
//     /* Validate the SecretHandle */
//     Status = LsapValidateDbObject(SecretHandle,
//                                   LsaDbSecretObject,
//                                   SECRET_SET_VALUE,
//                                   &SecretObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     if (EncryptedCurrentValue != NULL)
//     {
//         /* FIXME: Decrypt the current value */
//         CurrentValue = EncryptedCurrentValue->Buffer;
//         CurrentValueLength = EncryptedCurrentValue->MaximumLength;
//     }
//
//     /* Set the current value */
//     Status = LsapSetObjectAttribute(SecretObject,
//                                     L"CurrentValue",
//                                     CurrentValue,
//                                     CurrentValueLength);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Get the current time */
//     Status = NtQuerySystemTime(&Time);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtQuerySystemTime failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Set the current time */
//     Status = LsapSetObjectAttribute(SecretObject,
//                                     L"CurrentTime",
//                                     &Time,
//                                     sizeof(LARGE_INTEGER));
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     if (EncryptedOldValue != NULL)
//     {
//         /* FIXME: Decrypt the old value */
//         OldValue = EncryptedOldValue->Buffer;
//         OldValueLength = EncryptedOldValue->MaximumLength;
//     }
//
//     /* Set the old value */
//     Status = LsapSetObjectAttribute(SecretObject,
//                                     L"OldValue",
//                                     OldValue,
//                                     OldValueLength);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Set the old time */
//     Status = LsapSetObjectAttribute(SecretObject,
//                                     L"OldTime",
//                                     &Time,
//                                     sizeof(LARGE_INTEGER));
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//     }
//
// done:
//     return Status;
// }
//
//
// /* Function 30 */
// NTSTATUS
// WINAPI
// LsarQuerySecret(
//     LSAPR_HANDLE SecretHandle,
//     PLSAPR_CR_CIPHER_VALUE *EncryptedCurrentValue,
//     PLARGE_INTEGER CurrentValueSetTime,
//     PLSAPR_CR_CIPHER_VALUE *EncryptedOldValue,
//     PLARGE_INTEGER OldValueSetTime)
// {
//     PLSA_DB_OBJECT SecretObject;
//     PLSAPR_CR_CIPHER_VALUE EncCurrentValue = NULL;
//     PLSAPR_CR_CIPHER_VALUE EncOldValue = NULL;
//     PBYTE CurrentValue = NULL;
//     PBYTE OldValue = NULL;
//     ULONG CurrentValueLength = 0;
//     ULONG OldValueLength = 0;
//     ULONG BufferSize;
//     NTSTATUS Status;
//
//     TRACE("LsarQuerySecret(%p %p %p %p %p)\n", SecretHandle,
//           EncryptedCurrentValue, CurrentValueSetTime,
//           EncryptedOldValue, OldValueSetTime);
//
//     /* Validate the SecretHandle */
//     Status = LsapValidateDbObject(SecretHandle,
//                                   LsaDbSecretObject,
//                                   SECRET_QUERY_VALUE,
//                                   &SecretObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     if (EncryptedCurrentValue != NULL)
//     {
//         CurrentValueLength = 0;
//
//         /* Get the size of the current value */
//         Status = LsapGetObjectAttribute(SecretObject,
//                                         L"CurrentValue",
//                                         NULL,
//                                         &CurrentValueLength);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         /* Allocate a buffer for the current value */
//         CurrentValue = midl_user_allocate(CurrentValueLength);
//         if (CurrentValue == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         /* Get the current value */
//         Status = LsapGetObjectAttribute(SecretObject,
//                                         L"CurrentValue",
//                                         CurrentValue,
//                                         &CurrentValueLength);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         /* Allocate a buffer for the encrypted current value */
//         EncCurrentValue = midl_user_allocate(sizeof(LSAPR_CR_CIPHER_VALUE));
//         if (EncCurrentValue == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         /* FIXME: Encrypt the current value */
//         EncCurrentValue->Length = (USHORT)(CurrentValueLength - sizeof(WCHAR));
//         EncCurrentValue->MaximumLength = (USHORT)CurrentValueLength;
//         EncCurrentValue->Buffer = (PBYTE)CurrentValue;
//     }
//
//     if (CurrentValueSetTime != NULL)
//     {
//         BufferSize = sizeof(LARGE_INTEGER);
//
//         /* Get the current value time */
//         Status = LsapGetObjectAttribute(SecretObject,
//                                         L"CurrentTime",
//                                         (PBYTE)CurrentValueSetTime,
//                                         &BufferSize);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (EncryptedOldValue != NULL)
//     {
//         OldValueLength = 0;
//
//         /* Get the size of the old value */
//         Status = LsapGetObjectAttribute(SecretObject,
//                                         L"OldValue",
//                                         NULL,
//                                         &OldValueLength);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         /* Allocate a buffer for the old value */
//         OldValue = midl_user_allocate(OldValueLength);
//         if (OldValue == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         /* Get the old value */
//         Status = LsapGetObjectAttribute(SecretObject,
//                                         L"OldValue",
//                                         OldValue,
//                                         &OldValueLength);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         /* Allocate a buffer for the encrypted old value */
//         EncOldValue = midl_user_allocate(sizeof(LSAPR_CR_CIPHER_VALUE) + OldValueLength);
//         if (EncOldValue == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         /* FIXME: Encrypt the old value */
//         EncOldValue->Length = (USHORT)(OldValueLength - sizeof(WCHAR));
//         EncOldValue->MaximumLength = (USHORT)OldValueLength;
//         EncOldValue->Buffer = (PBYTE)OldValue;
//     }
//
//     if (OldValueSetTime != NULL)
//     {
//         BufferSize = sizeof(LARGE_INTEGER);
//
//         /* Get the old value time */
//         Status = LsapGetObjectAttribute(SecretObject,
//                                         L"OldTime",
//                                         (PBYTE)OldValueSetTime,
//                                         &BufferSize);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//
// done:
//     if (NT_SUCCESS(Status))
//     {
//         if (EncryptedCurrentValue != NULL)
//             *EncryptedCurrentValue = EncCurrentValue;
//
//         if (EncryptedOldValue != NULL)
//             *EncryptedOldValue = EncOldValue;
//     }
//     else
//     {
//         if (EncryptedCurrentValue != NULL)
//             *EncryptedCurrentValue = NULL;
//
//         if (EncryptedOldValue != NULL)
//             *EncryptedOldValue = NULL;
//
//         if (EncCurrentValue != NULL)
//             midl_user_free(EncCurrentValue);
//
//         if (EncOldValue != NULL)
//             midl_user_free(EncOldValue);
//
//         if (CurrentValue != NULL)
//             midl_user_free(CurrentValue);
//
//         if (OldValue != NULL)
//             midl_user_free(OldValue);
//     }
//
//     TRACE("LsarQuerySecret done (Status 0x%08lx)\n", Status);
//
//     return Status;
// }
//
//
// /* Function 31 */
// NTSTATUS
// WINAPI
// LsarLookupPrivilegeValue(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_UNICODE_STRING Name,
//     PLUID Value)
// {
//     PLUID pValue;
//     NTSTATUS Status;
//
//     TRACE("LsarLookupPrivilegeValue(%p, %wZ, %p)\n",
//           PolicyHandle, Name, Value);
//
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   POLICY_LOOKUP_NAMES,
//                                   NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Invalid handle (Status %lx)\n", Status);
//         return Status;
//     }
//
//     TRACE("Privilege: %wZ\n", Name);
//
//     pValue = LsarpLookupPrivilegeValue(Name);
//     if (pValue == NULL)
//         return STATUS_NO_SUCH_PRIVILEGE;
//
//     RtlCopyLuid(Value, pValue);
//
//     return STATUS_SUCCESS;
// }
//
//
// /* Function 32 */
// NTSTATUS
// WINAPI
// LsarLookupPrivilegeName(
//     LSAPR_HANDLE PolicyHandle,
//     PLUID Value,
//     PRPC_UNICODE_STRING *Name)
// {
//     NTSTATUS Status;
//
//     TRACE("LsarLookupPrivilegeName(%p, %p, %p)\n",
//           PolicyHandle, Value, Name);
//
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   POLICY_LOOKUP_NAMES,
//                                   NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Invalid handle\n");
//         return Status;
//     }
//
//     Status = LsarpLookupPrivilegeName(Value,
//                                       Name);
//
//     return Status;
// }
//
//
// /* Function 33 */
// NTSTATUS
// WINAPI
// LsarLookupPrivilegeDisplayName(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_UNICODE_STRING Name,
//     USHORT ClientLanguage,
//     USHORT ClientSystemDefaultLanguage,
//     PRPC_UNICODE_STRING *DisplayName,
//     USHORT *LanguageReturned)
// {
//     NTSTATUS Status;
//
//     TRACE("LsarLookupPrivilegeDisplayName(%p, %p, %u, %u, %p, %p)\n",
//           PolicyHandle, Name, ClientLanguage, ClientSystemDefaultLanguage, DisplayName, LanguageReturned);
//
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   POLICY_LOOKUP_NAMES,
//                                   NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Invalid handle\n");
//         return Status;
//     }
//
//     Status = LsarpLookupPrivilegeDisplayName(Name,
//                                              ClientLanguage,
//                                              ClientSystemDefaultLanguage,
//                                              DisplayName,
//                                              LanguageReturned);
//
//     return Status;
// }
//
//
// /* Function 34 */
// NTSTATUS
// WINAPI
// LsarDeleteObject(
//     LSAPR_HANDLE *ObjectHandle)
// {
//     PLSA_DB_OBJECT DbObject;
//     NTSTATUS Status;
//
//     TRACE("LsarDeleteObject(%p)\n", ObjectHandle);
//
//     if (ObjectHandle == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Validate the ObjectHandle */
//     Status = LsapValidateDbObject(*ObjectHandle,
//                                   LsaDbIgnoreObject,
//                                   DELETE,
//                                   &DbObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* You cannot delete the policy object */
//     if (DbObject->ObjectType == LsaDbPolicyObject)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Delete the database object */
//     Status = LsapDeleteDbObject(DbObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapDeleteDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Invalidate the object handle */
//     *ObjectHandle = NULL;
//
//     return STATUS_SUCCESS;
// }
//
//
// /* Function 35 */
// NTSTATUS
// WINAPI
// LsarEnumerateAccountsWithUserRight(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_UNICODE_STRING UserRight,
//     PLSAPR_ACCOUNT_ENUM_BUFFER EnumerationBuffer)
// {
//     PLSA_DB_OBJECT PolicyObject;
//     ACCESS_MASK AccountRight = 0;
//     PLUID Luid = NULL;
//     ULONG AccountKeyBufferSize;
//     PWSTR AccountKeyBuffer = NULL;
//     HKEY AccountsKeyHandle = NULL;
//     HKEY AccountKeyHandle = NULL;
//     HKEY AttributeKeyHandle;
//     ACCESS_MASK SystemAccess;
//     PPRIVILEGE_SET PrivilegeSet;
//     PLSAPR_ACCOUNT_INFORMATION EnumBuffer = NULL, ReturnBuffer;
//     ULONG SubKeyCount = 0;
//     ULONG EnumIndex, EnumCount;
//     ULONG Size, i;
//     BOOL Found;
//     NTSTATUS Status;
//
//     TRACE("LsarEnumerateAccountsWithUserRights(%p %wZ %p)\n",
//           PolicyHandle, UserRight, EnumerationBuffer);
//
//     /* Validate the privilege and account right names */
//     if (UserRight != NULL)
//     {
//         Luid = LsarpLookupPrivilegeValue(UserRight);
//         if (Luid == NULL)
//         {
//             AccountRight = LsapLookupAccountRightValue(UserRight);
//             if (AccountRight == 0)
//                 return STATUS_NO_SUCH_PRIVILEGE;
//         }
//     }
//
//     if (EnumerationBuffer == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     EnumerationBuffer->EntriesRead = 0;
//     EnumerationBuffer->Information = NULL;
//
//     /* Validate the PolicyHandle */
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   POLICY_LOOKUP_NAMES | POLICY_VIEW_LOCAL_INFORMATION,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     Status = LsapRegOpenKey(PolicyObject->KeyHandle,
//                             L"Accounts",
//                             KEY_READ,
//                             &AccountsKeyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapRegOpenKey returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     Status = LsapRegQueryKeyInfo(AccountsKeyHandle,
//                                  &SubKeyCount,
//                                  &AccountKeyBufferSize,
//                                  NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapRegOpenKey returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     AccountKeyBufferSize += sizeof(WCHAR);
//     AccountKeyBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, AccountKeyBufferSize);
//     if (AccountKeyBuffer == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     EnumBuffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  SubKeyCount * sizeof(LSAPR_ACCOUNT_INFORMATION));
//     if (EnumBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     EnumCount = 0;
//     EnumIndex = 0;
//     while (TRUE)
//     {
//         Found = FALSE;
//
//         Status = LsapRegEnumerateSubKey(AccountsKeyHandle,
//                                         EnumIndex,
//                                         AccountKeyBufferSize,
//                                         AccountKeyBuffer);
//         if (!NT_SUCCESS(Status))
//         {
//             if (Status == STATUS_NO_MORE_ENTRIES)
//                 Status = STATUS_SUCCESS;
//             break;
//         }
//
//         TRACE("EnumIndex: %lu\n", EnumIndex);
//         TRACE("Account key name: %S\n", AccountKeyBuffer);
//
//         Status = LsapRegOpenKey(AccountsKeyHandle,
//                                 AccountKeyBuffer,
//                                 KEY_READ,
//                                 &AccountKeyHandle);
//         if (NT_SUCCESS(Status))
//         {
//             if (Luid != NULL || AccountRight != 0)
//             {
//                 Status = LsapRegOpenKey(AccountKeyHandle,
//                                         (Luid != NULL) ? L"Privilgs" : L"ActSysAc",
//                                         KEY_READ,
//                                         &AttributeKeyHandle);
//                 if (NT_SUCCESS(Status))
//                 {
//                     if (Luid != NULL)
//                     {
//                         Size = 0;
//                         LsapRegQueryValue(AttributeKeyHandle,
//                                           NULL,
//                                           NULL,
//                                           NULL,
//                                           &Size);
//                         if (Size != 0)
//                         {
//                             PrivilegeSet = RtlAllocateHeap(RtlGetProcessHeap(), 0, Size);
//                             if (PrivilegeSet)
//                             {
//                                 if (LsapRegQueryValue(AttributeKeyHandle,
//                                                       NULL,
//                                                       NULL,
//                                                       PrivilegeSet,
//                                                       &Size) == STATUS_SUCCESS)
//                                 {
//                                     for (i = 0; i < PrivilegeSet->PrivilegeCount; i++)
//                                     {
//                                         if (RtlEqualLuid(&(PrivilegeSet->Privilege[i].Luid), Luid))
//                                         {
//                                             TRACE("%S got the privilege!\n", AccountKeyBuffer);
//                                             Found = TRUE;
//                                             break;
//                                         }
//                                     }
//                                 }
//
//                                 RtlFreeHeap(RtlGetProcessHeap(), 0, PrivilegeSet);
//                             }
//                         }
//                     }
//                     else if (AccountRight != 0)
//                     {
//                         SystemAccess = 0;
//                         Size = sizeof(ACCESS_MASK);
//                         LsapRegQueryValue(AttributeKeyHandle,
//                                           NULL,
//                                           NULL,
//                                           &SystemAccess,
//                                           &Size);
//                         if (SystemAccess & AccountRight)
//                         {
//                             TRACE("%S got the account right!\n", AccountKeyBuffer);
//                             Found = TRUE;
//                         }
//                     }
//
//                     LsapRegCloseKey(AttributeKeyHandle);
//                 }
//             }
//             else
//             {
//                 /* enumerate all accounts */
//                 Found = TRUE;
//             }
//
//             if (Found == TRUE)
//             {
//                 TRACE("Add account: %S\n", AccountKeyBuffer);
//
//                 Status = LsapRegOpenKey(AccountKeyHandle,
//                                         L"Sid",
//                                         KEY_READ,
//                                         &AttributeKeyHandle);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Size = 0;
//                     LsapRegQueryValue(AttributeKeyHandle,
//                                       NULL,
//                                       NULL,
//                                       NULL,
//                                       &Size);
//                     if (Size != 0)
//                     {
//                         EnumBuffer[EnumCount].Sid = midl_user_allocate(Size);
//                         if (EnumBuffer[EnumCount].Sid != NULL)
//                         {
//                             Status = LsapRegQueryValue(AttributeKeyHandle,
//                                                        NULL,
//                                                        NULL,
//                                                        EnumBuffer[EnumCount].Sid,
//                                                        &Size);
//                             if (NT_SUCCESS(Status))
//                             {
//                                 EnumCount++;
//                             }
//                             else
//                             {
//                                 TRACE("SampRegQueryValue returned %08lX\n", Status);
//                                 midl_user_free(EnumBuffer[EnumCount].Sid);
//                                 EnumBuffer[EnumCount].Sid = NULL;
//                             }
//                         }
//                     }
//
//                     LsapRegCloseKey(AttributeKeyHandle);
//                 }
//             }
//
//             LsapRegCloseKey(AccountKeyHandle);
//         }
//
//         EnumIndex++;
//     }
//
//     TRACE("EnumCount: %lu\n", EnumCount);
//
//     if (NT_SUCCESS(Status) && EnumCount != 0)
//     {
//         ReturnBuffer = midl_user_allocate(EnumCount * sizeof(LSAPR_ACCOUNT_INFORMATION));
//         if (ReturnBuffer == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         RtlCopyMemory(ReturnBuffer,
//                       EnumBuffer,
//                       EnumCount * sizeof(LSAPR_ACCOUNT_INFORMATION));
//
//         EnumerationBuffer->EntriesRead = EnumCount;
//         EnumerationBuffer->Information = ReturnBuffer;
//     }
//
// done:
//     if (EnumBuffer != NULL)
//     {
//         if (Status != STATUS_SUCCESS)
//         {
//             for (i = 0; i < EnumCount; i++)
//             {
//                 if (EnumBuffer[i].Sid != NULL)
//                     midl_user_free(EnumBuffer[i].Sid);
//             }
//         }
//
//         RtlFreeHeap(RtlGetProcessHeap(), 0, EnumBuffer);
//     }
//
//     if (AccountKeyBuffer != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AccountKeyBuffer);
//
//     if (Status == STATUS_SUCCESS && EnumCount == 0)
//         Status = STATUS_NO_MORE_ENTRIES;
//
//     return Status;
// }
//
//
// /* Function 36 */
// NTSTATUS
// WINAPI
// LsarEnumerateAccountRights(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_SID AccountSid,
//     PLSAPR_USER_RIGHT_SET UserRights)
// {
//     LSAPR_HANDLE AccountHandle;
//     PLSAPR_PRIVILEGE_SET PrivilegeSet = NULL;
//     PRPC_UNICODE_STRING RightsBuffer = NULL;
//     PRPC_UNICODE_STRING PrivilegeString;
//     ACCESS_MASK SystemAccess = 0;
//     ULONG RightsCount = 0;
//     ULONG Index;
//     ULONG i;
//     NTSTATUS Status;
//
//     TRACE("LsarEnumerateAccountRights(%p %p %p)\n",
//           PolicyHandle, AccountSid, UserRights);
//
//     /* Open the account */
//     Status = LsarOpenAccount(PolicyHandle,
//                              AccountSid,
//                              ACCOUNT_VIEW,
//                              &AccountHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN("LsarOpenAccount returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Enumerate the privileges */
//     Status = LsarEnumeratePrivilegesAccount(AccountHandle,
//                                             &PrivilegeSet);
//     if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         WARN("LsarEnumeratePrivilegesAccount returned 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get account rights */
//     Status = LsarGetSystemAccessAccount(AccountHandle,
//                                         &SystemAccess);
//     if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         WARN("LsarGetSystemAccessAccount returned 0x%08lx\n", Status);
//         goto done;
//     }
//
//     RightsCount = PrivilegeSet->PrivilegeCount;
//
//     /* Count account rights */
//     for (i = 0; i < sizeof(ACCESS_MASK) * 8; i++)
//     {
//         if (SystemAccess & (1 << i))
//             RightsCount++;
//     }
//
//     /* We are done if there are no rights to be enumerated */
//     if (RightsCount == 0)
//     {
//         UserRights->Entries = 0;
//         UserRights->UserRights = NULL;
//         Status = STATUS_SUCCESS;
//         goto done;
//     }
//
//     /* Allocate a buffer for the account rights */
//     RightsBuffer = MIDL_user_allocate(RightsCount * sizeof(RPC_UNICODE_STRING));
//     if (RightsBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* Copy the privileges into the buffer */
//     Index = 0;
//     if (PrivilegeSet)
//     {
//         for (i = 0; i < PrivilegeSet->PrivilegeCount; i++)
//         {
//             PrivilegeString = NULL;
//             Status = LsarLookupPrivilegeName(PolicyHandle,
//                                              (PLUID)&PrivilegeSet->Privilege[i].Luid,
//                                              &PrivilegeString);
//             if (!NT_SUCCESS(Status))
//             {
//                 WARN("LsarLookupPrivilegeName returned 0x%08lx\n", Status);
//                 goto done;
//             }
//
//             RightsBuffer[Index].Length = PrivilegeString->Length;
//             RightsBuffer[Index].MaximumLength = PrivilegeString->MaximumLength;
//             RightsBuffer[Index].Buffer = PrivilegeString->Buffer;
//
//             MIDL_user_free(PrivilegeString);
//             Index++;
//         }
//     }
//
//     /* Copy account rights into the buffer */
//     for (i = 0; i < sizeof(ACCESS_MASK) * 8; i++)
//     {
//         if (SystemAccess & (1 << i))
//         {
//             Status = LsapLookupAccountRightName(1 << i,
//                                                 &PrivilegeString);
//             if (!NT_SUCCESS(Status))
//             {
//                 WARN("LsarLookupAccountRightName returned 0x%08lx\n", Status);
//                 goto done;
//             }
//
//             RightsBuffer[Index].Length = PrivilegeString->Length;
//             RightsBuffer[Index].MaximumLength = PrivilegeString->MaximumLength;
//             RightsBuffer[Index].Buffer = PrivilegeString->Buffer;
//
//             MIDL_user_free(PrivilegeString);
//             Index++;
//         }
//     }
//
//     UserRights->Entries = RightsCount;
//     UserRights->UserRights = (PRPC_UNICODE_STRING)RightsBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (RightsBuffer != NULL)
//         {
//             for (Index = 0; Index < RightsCount; Index++)
//             {
//                 if (RightsBuffer[Index].Buffer != NULL)
//                     MIDL_user_free(RightsBuffer[Index].Buffer);
//             }
//
//             MIDL_user_free(RightsBuffer);
//         }
//     }
//
//     if (PrivilegeSet != NULL)
//         MIDL_user_free(PrivilegeSet);
//
//     LsarClose(&AccountHandle);
//
//     return Status;
// }
//
//
// /* Function 37 */
// NTSTATUS
// WINAPI
// LsarAddAccountRights(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_SID AccountSid,
//     PLSAPR_USER_RIGHT_SET UserRights)
// {
//     PLSA_DB_OBJECT PolicyObject;
//     PLSA_DB_OBJECT AccountObject = NULL;
//     ULONG ulNewPrivileges = 0, ulNewRights = 0;
//     ACCESS_MASK SystemAccess = 0;
//     ULONG Size, Value, i, j;
//     PPRIVILEGE_SET PrivilegeSet = NULL;
//     ULONG PrivilegeSetBufferSize = 0;
//     ULONG PrivilegeCount;
//     BOOLEAN bFound;
//     PLUID pLuid;
//     NTSTATUS Status;
//
//     TRACE("LsarAddAccountRights(%p %p %p)\n",
//           PolicyHandle, AccountSid, UserRights);
//
//     /* Validate the AccountSid */
//     if (!RtlValidSid(AccountSid))
//         return STATUS_INVALID_PARAMETER;
//
//     /* Validate the UserRights */
//     if (UserRights == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Validate the privilege and account right names */
//     for (i = 0; i < UserRights->Entries; i++)
//     {
//         if (LsarpLookupPrivilegeValue(&UserRights->UserRights[i]) != NULL)
//         {
//             ulNewPrivileges++;
//         }
//         else
//         {
//             if (LsapLookupAccountRightValue(&UserRights->UserRights[i]) == 0)
//                 return STATUS_NO_SUCH_PRIVILEGE;
//
//             ulNewRights++;
//         }
//     }
//
//     TRACE("ulNewPrivileges: %lu\n", ulNewPrivileges);
//     TRACE("ulNewRights: %lu\n", ulNewRights);
//
//     /* Validate the PolicyHandle */
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   POLICY_LOOKUP_NAMES,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Open the account */
//     Status = LsarpOpenAccount(PolicyObject,
//                               AccountSid,
//                               0,
//                               &AccountObject);
//     if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         WARN("LsarpOpenAccount returned 0x%08lx\n", Status);
//         goto done;
//     }
//     else if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         /* Create a new account if it does not yet exist */
//         Status = LsarpCreateAccount(PolicyObject,
//                                     AccountSid,
//                                     0,
//                                     &AccountObject);
//         if (!NT_SUCCESS(Status))
//         {
//             WARN("LsarpCreateAccount returned 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     if (ulNewPrivileges > 0)
//     {
//         Size = 0;
//
//         /* Get the size of the Privilgs attribute */
//         Status = LsapGetObjectAttribute(AccountObject,
//                                         L"Privilgs",
//                                         NULL,
//                                         &Size);
//         if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//             goto done;
//
//         /* Calculate the required privilege set buffer size */
//         if (Size == 0)
//             PrivilegeSetBufferSize = sizeof(PRIVILEGE_SET) +
//                                      (ulNewPrivileges - 1) * sizeof(LUID_AND_ATTRIBUTES);
//         else
//             PrivilegeSetBufferSize = Size +
//                                      ulNewPrivileges * sizeof(LUID_AND_ATTRIBUTES);
//
//         /* Allocate the privilege set buffer */
//         PrivilegeSet = RtlAllocateHeap(RtlGetProcessHeap(),
//                                        HEAP_ZERO_MEMORY,
//                                        PrivilegeSetBufferSize);
//         if (PrivilegeSet == NULL)
//             return STATUS_NO_MEMORY;
//
//         /* Get the privilege set */
//         if (Size != 0)
//         {
//             Status = LsapGetObjectAttribute(AccountObject,
//                                             L"Privilgs",
//                                             PrivilegeSet,
//                                             &Size);
//             if (!NT_SUCCESS(Status))
//             {
//                 WARN("LsapGetObjectAttribute() failed (Status 0x%08lx)\n", Status);
//                 goto done;
//             }
//         }
//
//         PrivilegeCount = PrivilegeSet->PrivilegeCount;
//         TRACE("Privilege count: %lu\n", PrivilegeCount);
//
//         for (i = 0; i < UserRights->Entries; i++)
//         {
//             pLuid = LsarpLookupPrivilegeValue(&UserRights->UserRights[i]);
//             if (pLuid == NULL)
//                 continue;
//
//             bFound = FALSE;
//             for (j = 0; j < PrivilegeSet->PrivilegeCount; j++)
//             {
//                 if (RtlEqualLuid(&(PrivilegeSet->Privilege[j].Luid), pLuid))
//                 {
//                     bFound = TRUE;
//                     break;
//                 }
//             }
//
//             if (bFound == FALSE)
//             {
//                 /* Copy the new privilege */
//                 RtlCopyMemory(&(PrivilegeSet->Privilege[PrivilegeSet->PrivilegeCount]),
//                               pLuid,
//                               sizeof(LUID));
//                 PrivilegeSet->PrivilegeCount++;
//             }
//         }
//
//         /* Store the extended privilege set */
//         if (PrivilegeCount != PrivilegeSet->PrivilegeCount)
//         {
//             Size = sizeof(PRIVILEGE_SET) +
//                    (PrivilegeSet->PrivilegeCount - 1) * sizeof(LUID_AND_ATTRIBUTES);
//
//             Status = LsapSetObjectAttribute(AccountObject,
//                                             L"Privilgs",
//                                             PrivilegeSet,
//                                             Size);
//             if (!NT_SUCCESS(Status))
//             {
//                 WARN("LsapSetObjectAttribute() failed (Status 0x%08lx)\n", Status);
//                 goto done;
//             }
//         }
//     }
//
//     if (ulNewRights > 0)
//     {
//         Size = sizeof(ACCESS_MASK);
//
//         /* Get the system access flags, if the attribute exists */
//         Status = LsapGetObjectAttribute(AccountObject,
//                                         L"ActSysAc",
//                                         &SystemAccess,
//                                         &Size);
//         if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//             goto done;
//
//         /* Set the new access rights */
//         for (i = 0; i < UserRights->Entries; i++)
//         {
//             Value = LsapLookupAccountRightValue(&UserRights->UserRights[i]);
//             if (Value != 0)
//                 SystemAccess |= Value;
//         }
//
//         /* Set the system access flags */
//         Status = LsapSetObjectAttribute(AccountObject,
//                                         L"ActSysAc",
//                                         &SystemAccess,
//                                         sizeof(ACCESS_MASK));
//     }
//
// done:
//     if (PrivilegeSet != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, PrivilegeSet);
//
//     if (AccountObject != NULL)
//         LsapCloseDbObject(AccountObject);
//
//     return Status;
// }
//
//
// /* Function 38 */
// NTSTATUS
// WINAPI
// LsarRemoveAccountRights(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_SID AccountSid,
//     BOOLEAN AllRights,
//     PLSAPR_USER_RIGHT_SET UserRights)
// {
//     PLSA_DB_OBJECT PolicyObject;
//     PLSA_DB_OBJECT AccountObject = NULL;
//     ULONG PrivilegesToRemove = 0, RightsToRemove = 0;
//     ACCESS_MASK SystemAccess = 0;
//     ULONG Size, Value, i, j, Index;
//     PPRIVILEGE_SET PrivilegeSet = NULL;
//     ULONG PrivilegeCount;
//     PLUID pLuid;
//     NTSTATUS Status;
//
//     TRACE("LsarRemoveAccountRights(%p %p %lu %p)\n",
//           PolicyHandle, AccountSid, AllRights, UserRights);
//
//     /* Validate the AccountSid */
//     if (!RtlValidSid(AccountSid))
//         return STATUS_INVALID_PARAMETER;
//
//     /* Validate the UserRights */
//     if (UserRights == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Validate the privilege and account right names */
//     for (i = 0; i < UserRights->Entries; i++)
//     {
//         if (LsarpLookupPrivilegeValue(&UserRights->UserRights[i]) != NULL)
//         {
//             PrivilegesToRemove++;
//         }
//         else
//         {
//             if (LsapLookupAccountRightValue(&UserRights->UserRights[i]) == 0)
//                 return STATUS_NO_SUCH_PRIVILEGE;
//
//             RightsToRemove++;
//         }
//     }
//
//     /* Validate the PolicyHandle */
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   POLICY_LOOKUP_NAMES,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Open the account */
//     Status = LsarpOpenAccount(PolicyObject,
//                               AccountSid,
//                               0,
//                               &AccountObject);
//     if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         ERR("LsarpOpenAccount returned 0x%08lx\n", Status);
//         goto done;
//     }
//
//     if (AllRights == FALSE)
//     {
//         /* Get the size of the Privilgs attribute */
//         Size = 0;
//         Status = LsapGetObjectAttribute(AccountObject,
//                                         L"Privilgs",
//                                         NULL,
//                                         &Size);
//         if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//             goto done;
//
//         if ((Size != 0) && (PrivilegesToRemove != 0))
//         {
//             /* Allocate the privilege set buffer */
//             PrivilegeSet = RtlAllocateHeap(RtlGetProcessHeap(),
//                                            HEAP_ZERO_MEMORY,
//                                            Size);
//             if (PrivilegeSet == NULL)
//                 return STATUS_NO_MEMORY;
//
//             /* Get the privilege set */
//             Status = LsapGetObjectAttribute(AccountObject,
//                                             L"Privilgs",
//                                             PrivilegeSet,
//                                             &Size);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("LsapGetObjectAttribute() failed (Status 0x%08lx)\n", Status);
//                 goto done;
//             }
//
//             PrivilegeCount = PrivilegeSet->PrivilegeCount;
//
//             for (i = 0; i < UserRights->Entries; i++)
//             {
//                 pLuid = LsarpLookupPrivilegeValue(&UserRights->UserRights[i]);
//                 if (pLuid == NULL)
//                     continue;
//
//                 Index = -1;
//                 for (j = 0; j < PrivilegeSet->PrivilegeCount; j++)
//                 {
//                     if (RtlEqualLuid(&(PrivilegeSet->Privilege[j].Luid), pLuid))
//                     {
//                         Index = j;
//                         break;
//                     }
//                 }
//
//                 if (Index != -1)
//                 {
//                     /* Remove the privilege */
//                     if ((PrivilegeSet->PrivilegeCount > 1) &&
//                         (Index < PrivilegeSet->PrivilegeCount - 1))
//                         RtlMoveMemory(&(PrivilegeSet->Privilege[Index]),
//                                       &(PrivilegeSet->Privilege[Index + 1]),
//                                       (Index - PrivilegeSet->PrivilegeCount - 1) * sizeof(LUID));
//
//                     /* Wipe the last entry */
//                     RtlZeroMemory(&(PrivilegeSet->Privilege[PrivilegeSet->PrivilegeCount - 1]),
//                                   sizeof(LUID));
//
//                     PrivilegeSet->PrivilegeCount--;
//                 }
//             }
//
//             /* Store the extended privilege set */
//             if (PrivilegeCount != PrivilegeSet->PrivilegeCount)
//             {
//                 Size = sizeof(PRIVILEGE_SET) +
//                        (PrivilegeSet->PrivilegeCount - 1) * sizeof(LUID_AND_ATTRIBUTES);
//
//                 Status = LsapSetObjectAttribute(AccountObject,
//                                                 L"Privilgs",
//                                                 PrivilegeSet,
//                                                 Size);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     ERR("LsapSetObjectAttribute() failed (Status 0x%08lx)\n", Status);
//                     goto done;
//                 }
//             }
//         }
//
//         /* Get the system access flags, if the attribute exists */
//         Size = 0;
//         Status = LsapGetObjectAttribute(AccountObject,
//                                         L"ActSysAc",
//                                         &SystemAccess,
//                                         &Size);
//         if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//             goto done;
//
//         if ((Size != 0) && (RightsToRemove != 0))
//         {
//             ERR("Rights: 0x%lx\n", SystemAccess);
//
//             /* Set the new access rights */
//             for (i = 0; i < UserRights->Entries; i++)
//             {
//                 Value = LsapLookupAccountRightValue(&UserRights->UserRights[i]);
//                 if (Value != 0)
//                     SystemAccess &= ~Value;
//             }
//             ERR("New Rights: 0x%lx\n", SystemAccess);
//
//             /* Set the system access flags */
//             Status = LsapSetObjectAttribute(AccountObject,
//                                             L"ActSysAc",
//                                             &SystemAccess,
//                                             sizeof(ACCESS_MASK));
//         }
//     }
//     else
//     {
//     }
//
// done:
//     if (PrivilegeSet != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, PrivilegeSet);
//
//     if (AccountObject != NULL)
//         LsapCloseDbObject(AccountObject);
//
//     return Status;
// }
//
//
// /* Function 39 */
// NTSTATUS
// WINAPI
// LsarQueryTrustedDomainInfo(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_SID TrustedDomainSid,
//     TRUSTED_INFORMATION_CLASS InformationClass,
//     PLSAPR_TRUSTED_DOMAIN_INFO *TrustedDomainInformation)
// {
//     /* Fail, if we are not a domain controller */
//     if (LsapProductType != NtProductLanManNt)
//         return STATUS_DIRECTORY_SERVICE_REQUIRED;
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 40 */
// NTSTATUS
// WINAPI
// LsarSetTrustedDomainInfo(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_SID TrustedDomainSid,
//     TRUSTED_INFORMATION_CLASS InformationClass,
//     PLSAPR_TRUSTED_DOMAIN_INFO TrustedDomainInformation)
// {
//     /* Fail, if we are not a domain controller */
//     if (LsapProductType != NtProductLanManNt)
//         return STATUS_DIRECTORY_SERVICE_REQUIRED;
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 41 */
// NTSTATUS
// WINAPI
// LsarDeleteTrustedDomain(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_SID TrustedDomainSid)
// {
//     /* Fail, if we are not a domain controller */
//     if (LsapProductType != NtProductLanManNt)
//         return STATUS_DIRECTORY_SERVICE_REQUIRED;
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 42 */
// NTSTATUS
// WINAPI
// LsarStorePrivateData(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_UNICODE_STRING KeyName,
//     PLSAPR_CR_CIPHER_VALUE EncryptedData)
// {
//     PLSA_DB_OBJECT PolicyObject = NULL;
//     PLSA_DB_OBJECT SecretsObject = NULL;
//     PLSA_DB_OBJECT SecretObject = NULL;
//     LARGE_INTEGER Time;
//     PBYTE Value = NULL;
//     ULONG ValueLength = 0;
//     NTSTATUS Status;
//
//     TRACE("LsarStorePrivateData(%p %p %p)\n",
//           PolicyHandle, KeyName, EncryptedData);
//
//     /* Validate the SecretHandle */
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   POLICY_CREATE_SECRET,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Open the 'Secrets' object */
//     Status = LsapOpenDbObject(PolicyObject,
//                               NULL,
//                               L"Secrets",
//                               LsaDbIgnoreObject,
//                               0,
//                               PolicyObject->Trusted,
//                               &SecretsObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapOpenDbObject failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     if (EncryptedData == NULL)
//     {
//         /* Open the Secret object */
//         Status = LsapOpenDbObject(SecretsObject,
//                                   NULL,
//                                   KeyName->Buffer,
//                                   LsaDbSecretObject,
//                                   0,
//                                   PolicyObject->Trusted,
//                                   &SecretObject);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("LsapOpenDbObject failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         /* Delete the secret */
//         Status = LsapDeleteDbObject(SecretObject);
//         if (NT_SUCCESS(Status))
//             SecretObject = NULL;
//     }
//     else
//     {
//         /* Create the Secret object */
//         Status = LsapCreateDbObject(SecretsObject,
//                                     NULL,
//                                     KeyName->Buffer,
//                                     LsaDbSecretObject,
//                                     0,
//                                     PolicyObject->Trusted,
//                                     &SecretObject);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("LsapCreateDbObject failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         /* FIXME: Decrypt data */
//         Value = EncryptedData->Buffer;
//         ValueLength = EncryptedData->MaximumLength;
//
//         /* Get the current time */
//         Status = NtQuerySystemTime(&Time);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("NtQuerySystemTime failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         /* Set the current value */
//         Status = LsapSetObjectAttribute(SecretObject,
//                                         L"CurrentValue",
//                                         Value,
//                                         ValueLength);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("LsapSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         /* Set the current time */
//         Status = LsapSetObjectAttribute(SecretObject,
//                                         L"CurrentTime",
//                                         &Time,
//                                         sizeof(LARGE_INTEGER));
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("LsapSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         /* Get the current time */
//         Status = NtQuerySystemTime(&Time);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("NtQuerySystemTime failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         /* Set the old value */
//         Status = LsapSetObjectAttribute(SecretObject,
//                                         L"OldValue",
//                                         NULL,
//                                         0);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("LsapSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         /* Set the old time */
//         Status = LsapSetObjectAttribute(SecretObject,
//                                         L"OldTime",
//                                         &Time,
//                                         sizeof(LARGE_INTEGER));
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("LsapSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//         }
//     }
//
// done:
//     if (SecretObject != NULL)
//         LsapCloseDbObject(SecretObject);
//
//     if (SecretsObject != NULL)
//         LsapCloseDbObject(SecretsObject);
//
//     return Status;
// }
//
//
// /* Function 43 */
// NTSTATUS
// WINAPI
// LsarRetrievePrivateData(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_UNICODE_STRING KeyName,
//     PLSAPR_CR_CIPHER_VALUE *EncryptedData)
// {
//     /* TODO: This should just call LsarOpenSecret(SECRET_QUERY_VALUE)+LsarQuerySecret? */
//
//     PLSA_DB_OBJECT PolicyObject = NULL;
//     PLSA_DB_OBJECT SecretObject = NULL;
//     PLSAPR_CR_CIPHER_VALUE EncCurrentValue = NULL;
//     ULONG CurrentValueLength = 0;
//     PBYTE CurrentValue = NULL;
//     NTSTATUS Status;
//
//     TRACE("LsarRetrievePrivateData(%p %wZ %p)\n",
//           PolicyHandle, KeyName, EncryptedData);
//
//     /* Validate the SecretHandle */
//     Status = LsapValidateDbObject(PolicyHandle,
//                                   LsaDbPolicyObject,
//                                   POLICY_GET_PRIVATE_INFORMATION,
//                                   &PolicyObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapValidateDbObject returned 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Open the secret object */
//     Status = LsapOpenDbObject(PolicyObject,
//                               L"Secrets",
//                               KeyName->Buffer,
//                               LsaDbSecretObject,
//                               0,
//                               PolicyObject->Trusted,
//                               &SecretObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapOpenDbObject failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Get the size of the current value */
//     Status = LsapGetObjectAttribute(SecretObject,
//                                     L"CurrentValue",
//                                     NULL,
//                                     &CurrentValueLength);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Allocate a buffer for the current value */
//     CurrentValue = midl_user_allocate(CurrentValueLength);
//     if (CurrentValue == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* Get the current value */
//     Status = LsapGetObjectAttribute(SecretObject,
//                                     L"CurrentValue",
//                                     CurrentValue,
//                                     &CurrentValueLength);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Allocate a buffer for the encrypted current value */
//     EncCurrentValue = midl_user_allocate(sizeof(LSAPR_CR_CIPHER_VALUE) + CurrentValueLength);
//     if (EncCurrentValue == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* FIXME: Encrypt the current value */
//     EncCurrentValue->Length = (USHORT)(CurrentValueLength - sizeof(WCHAR));
//     EncCurrentValue->MaximumLength = (USHORT)CurrentValueLength;
//     EncCurrentValue->Buffer = (PBYTE)(EncCurrentValue + 1);
//     RtlCopyMemory(EncCurrentValue->Buffer,
//                   CurrentValue,
//                   CurrentValueLength);
//
// done:
//     if (NT_SUCCESS(Status))
//     {
//         if (EncryptedData != NULL)
//             *EncryptedData = EncCurrentValue;
//     }
//     else
//     {
//         if (EncryptedData != NULL)
//             *EncryptedData = NULL;
//
//         if (EncCurrentValue != NULL)
//             midl_user_free(EncCurrentValue);
//     }
//
//     if (SecretObject != NULL)
//         LsapCloseDbObject(SecretObject);
//
//     return Status;
// }
//
//
// /* Function 44 */
// NTSTATUS
// WINAPI
// LsarOpenPolicy2(
//     LPWSTR SystemName,
//     PLSAPR_OBJECT_ATTRIBUTES ObjectAttributes,
//     ACCESS_MASK DesiredAccess,
//     LSAPR_HANDLE *PolicyHandle)
// {
//     return LsarOpenPolicy(SystemName,
//                           ObjectAttributes,
//                           DesiredAccess,
//                           PolicyHandle);
// }
//
//
// /* Function 45 */
// NTSTATUS
// WINAPI
// LsarGetUserName(
//     LPWSTR SystemName,
//     PRPC_UNICODE_STRING *UserName,
//     PRPC_UNICODE_STRING *DomainName)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 46 */
// NTSTATUS
// WINAPI
// LsarQueryInformationPolicy2(
//     LSAPR_HANDLE PolicyHandle,
//     POLICY_INFORMATION_CLASS InformationClass,
//     PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     return LsarQueryInformationPolicy(PolicyHandle,
//                                       InformationClass,
//                                       PolicyInformation);
// }
//
//
// /* Function 47 */
// NTSTATUS
// WINAPI
// LsarSetInformationPolicy2(
//     LSAPR_HANDLE PolicyHandle,
//     POLICY_INFORMATION_CLASS InformationClass,
//     PLSAPR_POLICY_INFORMATION PolicyInformation)
// {
//     return LsarSetInformationPolicy(PolicyHandle,
//                                     InformationClass,
//                                     PolicyInformation);
// }
//
//
// /* Function 48 */
// NTSTATUS
// WINAPI
// LsarQueryTrustedDomainInfoByName(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_UNICODE_STRING TrustedDomainName,
//     POLICY_INFORMATION_CLASS InformationClass,
//     PLSAPR_TRUSTED_DOMAIN_INFO *PolicyInformation)
// {
//     /* Fail, if we are not a domain controller */
//     if (LsapProductType != NtProductLanManNt)
//         return STATUS_DIRECTORY_SERVICE_REQUIRED;
//
//     /* FIXME: We are not running an AD yet */
//     return STATUS_OBJECT_NAME_NOT_FOUND;
// }
//
//
// /* Function 49 */
// NTSTATUS
// WINAPI
// LsarSetTrustedDomainInfoByName(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_UNICODE_STRING TrustedDomainName,
//     POLICY_INFORMATION_CLASS InformationClass,
//     PLSAPR_TRUSTED_DOMAIN_INFO PolicyInformation)
// {
//     /* Fail, if we are not a domain controller */
//     if (LsapProductType != NtProductLanManNt)
//         return STATUS_DIRECTORY_SERVICE_REQUIRED;
//
//     /* FIXME: We are not running an AD yet */
//     return STATUS_OBJECT_NAME_NOT_FOUND;
// }
//
//
// /* Function 50 */
// NTSTATUS
// WINAPI
// LsarEnumerateTrustedDomainsEx(
//     LSAPR_HANDLE PolicyHandle,
//     DWORD *EnumerationContext,
//     PLSAPR_TRUSTED_ENUM_BUFFER_EX EnumerationBuffer,
//     DWORD PreferedMaximumLength)
// {
//     /* Fail, if we are not a domain controller */
//     if (LsapProductType != NtProductLanManNt)
//         return STATUS_DIRECTORY_SERVICE_REQUIRED;
//
//     /* FIXME: We are not running an AD yet */
//     EnumerationBuffer->EntriesRead = 0;
//     EnumerationBuffer->EnumerationBuffer = NULL;
//     return STATUS_NO_MORE_ENTRIES;
// }
//
//
// /* Function 51 */
// NTSTATUS
// WINAPI
// LsarCreateTrustedDomainEx(
//     LSAPR_HANDLE PolicyHandle,
//     PLSAPR_TRUSTED_DOMAIN_INFORMATION_EX TrustedDomainInformation,
//     PLSAPR_TRUSTED_DOMAIN_AUTH_INFORMATION AuthentificationInformation,
//     ACCESS_MASK DesiredAccess,
//     LSAPR_HANDLE *TrustedDomainHandle)
// {
//     /* Fail, if we are not a domain controller */
//     if (LsapProductType != NtProductLanManNt)
//         return STATUS_DIRECTORY_SERVICE_REQUIRED;
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 52 */
// NTSTATUS
// WINAPI
// LsarSetPolicyReplicationHandle(
//     PLSAPR_HANDLE PolicyHandle)
// {
//     /* Deprecated */
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 53 */
// NTSTATUS
// WINAPI
// LsarQueryDomainInformationPolicy(
//     LSAPR_HANDLE PolicyHandle,
//     POLICY_INFORMATION_CLASS InformationClass,
//     PLSAPR_POLICY_DOMAIN_INFORMATION *PolicyInformation)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 54 */
// NTSTATUS
// WINAPI
// LsarSetDomainInformationPolicy(
//     LSAPR_HANDLE PolicyHandle,
//     POLICY_INFORMATION_CLASS InformationClass,
//     PLSAPR_POLICY_DOMAIN_INFORMATION PolicyInformation)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 55 */
// NTSTATUS
// WINAPI
// LsarOpenTrustedDomainByName(
//     LSAPR_HANDLE PolicyHandle,
//     PRPC_UNICODE_STRING TrustedDomainName,
//     ACCESS_MASK DesiredAccess,
//     LSAPR_HANDLE *TrustedDomainHandle)
// {
//     /* Fail, if we are not a domain controller */
//     if (LsapProductType != NtProductLanManNt)
//         return STATUS_DIRECTORY_SERVICE_REQUIRED;
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 56 */
// NTSTATUS
// WINAPI
// LsarTestCall(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 57 */
// NTSTATUS
// WINAPI
// LsarLookupSids2(
//     LSAPR_HANDLE PolicyHandle,
//     PLSAPR_SID_ENUM_BUFFER SidEnumBuffer,
//     PLSAPR_REFERENCED_DOMAIN_LIST *ReferencedDomains,
//     PLSAPR_TRANSLATED_NAMES_EX TranslatedNames,
//     LSAP_LOOKUP_LEVEL LookupLevel,
//     DWORD *MappedCount,
//     DWORD LookupOptions,
//     DWORD ClientRevision)
// {
//     NTSTATUS Status;
//
//     TRACE("LsarLookupSids2(%p %p %p %p %d %p %lu %lu)\n",
//           PolicyHandle, SidEnumBuffer, ReferencedDomains, TranslatedNames,
//           LookupLevel, MappedCount, LookupOptions, ClientRevision);
//
//     TranslatedNames->Entries = SidEnumBuffer->Entries;
//     TranslatedNames->Names = NULL;
//     *ReferencedDomains = NULL;
//
//     /* FIXME: Fail, if there is an invalid SID in the SidEnumBuffer */
//
//     Status = LsapLookupSids(SidEnumBuffer,
//                             ReferencedDomains,
//                             TranslatedNames,
//                             LookupLevel,
//                             MappedCount,
//                             LookupOptions,
//                             ClientRevision);
//
//     return Status;
// }
//
//
// /* Function 58 */
// NTSTATUS
// WINAPI
// LsarLookupNames2(
//     LSAPR_HANDLE PolicyHandle,
//     DWORD Count,
//     PRPC_UNICODE_STRING Names,
//     PLSAPR_REFERENCED_DOMAIN_LIST *ReferencedDomains,
//     PLSAPR_TRANSLATED_SIDS_EX TranslatedSids,
//     LSAP_LOOKUP_LEVEL LookupLevel,
//     DWORD *MappedCount,
//     DWORD LookupOptions,
//     DWORD ClientRevision)
// {
//     LSAPR_TRANSLATED_SIDS_EX2 TranslatedSidsEx2;
//     ULONG i;
//     NTSTATUS Status;
//
//     TRACE("LsarLookupNames2(%p %lu %p %p %p %d %p %lu %lu)\n",
//           PolicyHandle, Count, Names, ReferencedDomains, TranslatedSids,
//           LookupLevel, MappedCount, LookupOptions, ClientRevision);
//
//     TranslatedSids->Entries = 0;
//     TranslatedSids->Sids = NULL;
//     *ReferencedDomains = NULL;
//
//     if (Count == 0)
//         return STATUS_NONE_MAPPED;
//
//     TranslatedSidsEx2.Entries = 0;
//     TranslatedSidsEx2.Sids = NULL;
//
//     Status = LsapLookupNames(Count,
//                              Names,
//                              ReferencedDomains,
//                              &TranslatedSidsEx2,
//                              LookupLevel,
//                              MappedCount,
//                              LookupOptions,
//                              ClientRevision);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     TranslatedSids->Entries = TranslatedSidsEx2.Entries;
//     TranslatedSids->Sids = MIDL_user_allocate(TranslatedSids->Entries * sizeof(LSA_TRANSLATED_SID));
//     if (TranslatedSids->Sids == NULL)
//     {
//         MIDL_user_free(TranslatedSidsEx2.Sids);
//         MIDL_user_free(*ReferencedDomains);
//         *ReferencedDomains = NULL;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     for (i = 0; i < TranslatedSidsEx2.Entries; i++)
//     {
//         TranslatedSids->Sids[i].Use = TranslatedSidsEx2.Sids[i].Use;
//         TranslatedSids->Sids[i].RelativeId = LsapGetRelativeIdFromSid(TranslatedSidsEx2.Sids[i].Sid);
//         TranslatedSids->Sids[i].DomainIndex = TranslatedSidsEx2.Sids[i].DomainIndex;
//         TranslatedSids->Sids[i].Flags = TranslatedSidsEx2.Sids[i].Flags;
//     }
//
//     MIDL_user_free(TranslatedSidsEx2.Sids);
//
//     return STATUS_SUCCESS;
// }
//
//
// /* Function 59 */
// NTSTATUS
// WINAPI
// LsarCreateTrustedDomainEx2(
//     LSAPR_HANDLE PolicyHandle,
//     PLSAPR_TRUSTED_DOMAIN_INFORMATION_EX TrustedDomainInformation,
//     PLSAPR_TRUSTED_DOMAIN_AUTH_INFORMATION_INTERNAL AuthentificationInformation,
//     ACCESS_MASK DesiredAccess,
//     LSAPR_HANDLE *TrustedDomainHandle)
// {
//     /* Fail, if we are not a domain controller */
//     if (LsapProductType != NtProductLanManNt)
//         return STATUS_DIRECTORY_SERVICE_REQUIRED;
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 60 */
// NTSTATUS
// WINAPI
// CredrWrite(
//     PLSAPR_SERVER_NAME SystemName)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 61 */
// NTSTATUS
// WINAPI
// CredrRead(
//     PLSAPR_SERVER_NAME SystemName)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 62 */
// NTSTATUS
// WINAPI
// CredrEnumerate(
//     PLSAPR_SERVER_NAME SystemName)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 63 */
// NTSTATUS
// WINAPI
// CredrWriteDomainCredentials(
//     PLSAPR_SERVER_NAME SystemName)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 64 */
// NTSTATUS
// WINAPI
// CredrReadDomainCredentials(
//     PLSAPR_SERVER_NAME SystemName)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 65 */
// NTSTATUS
// WINAPI
// CredrDelete(
//     PLSAPR_SERVER_NAME SystemName,
//     LPWSTR TargetName,
//     DWORD Type,
//     DWORD Flags)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 66 */
// NTSTATUS
// WINAPI
// CredrGetTargetInfo(
//     PLSAPR_SERVER_NAME SystemName,
//     LPWSTR TargetName,
//     DWORD Flags,
//     CREDPR_TARGET_INFORMATION *TargetInformation)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 67 */
// NTSTATUS
// WINAPI
// CredrProfileLoaded(
//     PLSAPR_SERVER_NAME SystemName)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 68 */
// NTSTATUS
// WINAPI
// LsarLookupNames3(
//     LSAPR_HANDLE PolicyHandle,
//     DWORD Count,
//     PRPC_UNICODE_STRING Names,
//     PLSAPR_REFERENCED_DOMAIN_LIST *ReferencedDomains,
//     PLSAPR_TRANSLATED_SIDS_EX2 TranslatedSids,
//     LSAP_LOOKUP_LEVEL LookupLevel,
//     DWORD *MappedCount,
//     DWORD LookupOptions,
//     DWORD ClientRevision)
// {
//     NTSTATUS Status;
//
//     TRACE("LsarLookupNames3(%p %lu %p %p %p %d %p %lu %lu)\n",
//           PolicyHandle, Count, Names, ReferencedDomains, TranslatedSids,
//           LookupLevel, MappedCount, LookupOptions, ClientRevision);
//
//     TranslatedSids->Entries = 0;
//     TranslatedSids->Sids = NULL;
//     *ReferencedDomains = NULL;
//
//     if (Count == 0)
//         return STATUS_NONE_MAPPED;
//
//     Status = LsapLookupNames(Count,
//                              Names,
//                              ReferencedDomains,
//                              TranslatedSids,
//                              LookupLevel,
//                              MappedCount,
//                              LookupOptions,
//                              ClientRevision);
//
//     return Status;
// }
//
//
// /* Function 69 */
// NTSTATUS
// WINAPI
// CredrGetSessionTypes(
//     PLSAPR_SERVER_NAME SystemName,
//     DWORD MaximumPersistCount,
//     DWORD *MaximumPersist)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 70 */
// NTSTATUS
// WINAPI
// LsarRegisterAuditEvent(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 71 */
// NTSTATUS
// WINAPI
// LsarGenAuditEvent(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 72 */
// NTSTATUS
// WINAPI
// LsarUnregisterAuditEvent(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 73 */
// NTSTATUS
// WINAPI
// LsarQueryForestTrustInformation(
//     LSAPR_HANDLE PolicyHandle,
//     PLSA_UNICODE_STRING TrustedDomainName,
//     LSA_FOREST_TRUST_RECORD_TYPE HighestRecordType,
//     PLSA_FOREST_TRUST_INFORMATION *ForestTrustInfo)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 74 */
// NTSTATUS
// WINAPI
// LsarSetForestTrustInformation(
//     LSAPR_HANDLE PolicyHandle,
//     PLSA_UNICODE_STRING TrustedDomainName,
//     LSA_FOREST_TRUST_RECORD_TYPE HighestRecordType,
//     PLSA_FOREST_TRUST_INFORMATION ForestTrustInfo,
//     BOOLEAN CheckOnly,
//     PLSA_FOREST_TRUST_COLLISION_INFORMATION *CollisionInfo)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 75 */
// NTSTATUS
// WINAPI
// CredrRename(
//     PLSAPR_SERVER_NAME SystemName,
//     LPWSTR OldTargetName,
//     LPWSTR NewTargetName,
//     DWORD Type,
//     DWORD Flags)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 76 */
// NTSTATUS
// WINAPI
// LsarLookupSids3(
//     LSAPR_HANDLE PolicyHandle,
//     PLSAPR_SID_ENUM_BUFFER SidEnumBuffer,
//     PLSAPR_REFERENCED_DOMAIN_LIST *ReferencedDomains,
//     PLSAPR_TRANSLATED_NAMES_EX TranslatedNames,
//     LSAP_LOOKUP_LEVEL LookupLevel,
//     DWORD *MappedCount,
//     DWORD LookupOptions,
//     DWORD ClientRevision)
// {
//     NTSTATUS Status;
//
//     TRACE("LsarLookupSids3(%p %p %p %p %d %p %lu %lu)\n",
//           PolicyHandle, SidEnumBuffer, ReferencedDomains, TranslatedNames,
//           LookupLevel, MappedCount, LookupOptions, ClientRevision);
//
//     TranslatedNames->Entries = SidEnumBuffer->Entries;
//     TranslatedNames->Names = NULL;
//     *ReferencedDomains = NULL;
//
//     /* FIXME: Fail, if there is an invalid SID in the SidEnumBuffer */
//
//     Status = LsapLookupSids(SidEnumBuffer,
//                             ReferencedDomains,
//                             TranslatedNames,
//                             LookupLevel,
//                             MappedCount,
//                             LookupOptions,
//                             ClientRevision);
//
//     return Status;
// }
//
//
// /* Function 77 */
// NTSTATUS
// WINAPI
// LsarLookupNames4(
//     handle_t RpcHandle,
//     DWORD Count,
//     PRPC_UNICODE_STRING Names,
//     PLSAPR_REFERENCED_DOMAIN_LIST *ReferencedDomains,
//     PLSAPR_TRANSLATED_SIDS_EX2 TranslatedSids,
//     LSAP_LOOKUP_LEVEL LookupLevel,
//     DWORD *MappedCount,
//     DWORD LookupOptions,
//     DWORD ClientRevision)
// {
//     NTSTATUS Status;
//
//     TRACE("LsarLookupNames4(%p %lu %p %p %p %d %p %lu %lu)\n",
//           RpcHandle, Count, Names, ReferencedDomains, TranslatedSids,
//           LookupLevel, MappedCount, LookupOptions, ClientRevision);
//
//     TranslatedSids->Entries = 0;
//     TranslatedSids->Sids = NULL;
//     *ReferencedDomains = NULL;
//
//     if (Count == 0)
//         return STATUS_NONE_MAPPED;
//
//     Status = LsapLookupNames(Count,
//                              Names,
//                              ReferencedDomains,
//                              TranslatedSids,
//                              LookupLevel,
//                              MappedCount,
//                              LookupOptions,
//                              ClientRevision);
//
//     return Status;
// }
//
//
// /* Function 78 */
// NTSTATUS
// WINAPI
// LsarOpenPolicySce(
//     LPWSTR SystemName,
//     PLSAPR_OBJECT_ATTRIBUTES ObjectAttributes,
//     ACCESS_MASK DesiredAccess,
//     LSAPR_HANDLE *PolicyHandle)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 79 */
// NTSTATUS
// WINAPI
// LsarAdtRegisterSecurityEventSource(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 80 */
// NTSTATUS
// WINAPI
// LsarAdtUnregisterSecurityEventSource(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 81 */
// NTSTATUS
// WINAPI
// LsarAdtReportSecurityEvent(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
