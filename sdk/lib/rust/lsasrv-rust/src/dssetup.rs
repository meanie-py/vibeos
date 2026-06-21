//! dssetup.c -> Rust — Directory Service setup RPC (DsRoler interface).
//!
//! Faithful translation of dll/win32/lsasrv/dssetup.c (Eric Kohl). The single
//! RPC method `DsRolerGetPrimaryDomainInformation` reports this box as a
//! standalone workstation. Calls into the still-C Lsar* policy bodies
//! (LsarOpenPolicy/QueryInformationPolicy/Close) + the MIDL allocator, all of
//! which resolve at link. TRACE/WARN debug output (and the LsaNtStatusToWinError
//! it fed) is release-inert in the C, so it is omitted here.

use core::mem::{size_of, zeroed};
use core::ptr::{null_mut, write_volatile};
use lsasrv_sys::*;

// winerror.h
const ERROR_SUCCESS: u32 = 0;
const ERROR_OUTOFMEMORY: u32 = 14;
const ERROR_CALL_NOT_IMPLEMENTED: u32 = 120;
// ntsecapi.h
const POLICY_VIEW_LOCAL_INFORMATION: u32 = 1;
// POLICY_INFORMATION_CLASS::PolicyAccountDomainInformation
const POLICY_ACCOUNT_DOMAIN_INFORMATION: u32 =
    _POLICY_INFORMATION_CLASS_PolicyAccountDomainInformation as u32;
// DSROLE_PRIMARY_DOMAIN_INFO_LEVEL
const DS_ROLE_PRIMARY_DOMAIN_INFO_BASIC: u32 =
    _DSROLE_PRIMARY_DOMAIN_INFO_LEVEL_DsRolePrimaryDomainInfoBasic as u32;
const DS_ROLE_UPGRADE_STATUS: u32 = _DSROLE_PRIMARY_DOMAIN_INFO_LEVEL_DsRoleUpgradeStatus as u32;
const DS_ROLE_OPERATION_STATE: u32 = _DSROLE_PRIMARY_DOMAIN_INFO_LEVEL_DsRoleOperationState as u32;

#[inline]
fn nt_success(status: i32) -> bool {
    status >= 0
}

extern "system" {
    // Lsar* policy server bodies (still C in this module).
    fn LsarOpenPolicy(
        SystemName: PLSAPR_SERVER_NAME,
        ObjectAttributes: PLSAPR_OBJECT_ATTRIBUTES,
        DesiredAccess: u32,
        PolicyHandle: *mut LSAPR_HANDLE,
    ) -> i32;
    fn LsarQueryInformationPolicy(
        PolicyHandle: LSAPR_HANDLE,
        InformationClass: u32,
        PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
    ) -> i32;
    fn LsarClose(ObjectHandle: *mut LSAPR_HANDLE) -> i32;
    fn LsaIFree_LSAPR_POLICY_INFORMATION(
        InformationClass: u32,
        PolicyInformation: PLSAPR_POLICY_INFORMATION,
    );
    // MIDL allocator (lsasrv.c, __RPC_USER = stdcall; real symbol is the
    // uppercase MIDL_user_allocate that lsa_s.c references).
    fn MIDL_user_allocate(len: usize) -> *mut core::ffi::c_void;
    // rpcrt4 (RPC_ENTRY = stdcall).
    fn RpcServerRegisterIf(
        IfSpec: *mut core::ffi::c_void,
        MgrTypeUuid: *mut core::ffi::c_void,
        MgrEpv: *mut core::ffi::c_void,
    ) -> i32;
}

extern "C" {
    fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    // The dssetup RPC server interface spec — exported by the (still-C)
    // generated dssetup_s.c.
    static dssetup_v0_0_s_ifspec: *mut core::ffi::c_void;
}

#[no_mangle]
pub unsafe extern "C" fn DsSetupInit() {
    // WARN on failure is release-inert; faithful no-op-on-error control flow.
    let _status = RpcServerRegisterIf(dssetup_v0_0_s_ifspec, null_mut(), null_mut());
}

unsafe fn DsRolepGetBasicInfo(domain_info: *mut PDSROLER_PRIMARY_DOMAIN_INFORMATION) -> u32 {
    let mut object_attributes: _LSAPR_OBJECT_ATTRIBUTES = zeroed();
    let mut policy_handle: LSA_HANDLE = null_mut();

    let status = LsarOpenPolicy(
        null_mut(),
        &mut object_attributes as *mut _LSAPR_OBJECT_ATTRIBUTES,
        POLICY_VIEW_LOCAL_INFORMATION,
        &mut policy_handle as *mut LSA_HANDLE,
    );
    if !nt_success(status) {
        return ERROR_OUTOFMEMORY;
    }

    let mut policy_info: PLSAPR_POLICY_INFORMATION = null_mut();
    let status = LsarQueryInformationPolicy(
        policy_handle,
        POLICY_ACCOUNT_DOMAIN_INFORMATION,
        &mut policy_info,
    );
    LsarClose(&mut policy_handle as *mut LSA_HANDLE);
    if !nt_success(status) {
        return ERROR_OUTOFMEMORY;
    }

    let dom = &(*policy_info).PolicyAccountDomainInfo;
    let size = size_of::<_DSROLER_PRIMARY_DOMAIN_INFO_BASIC>()
        + dom.DomainName.Length as usize
        + size_of::<u16>();

    let buffer = MIDL_user_allocate(size) as *mut _DSROLER_PRIMARY_DOMAIN_INFO_BASIC;
    if buffer.is_null() {
        LsaIFree_LSAPR_POLICY_INFORMATION(POLICY_ACCOUNT_DOMAIN_INFORMATION, policy_info);
        return ERROR_OUTOFMEMORY;
    }

    (*buffer).MachineRole = _DSROLE_MACHINE_ROLE_DsRole_RoleStandaloneWorkstation;
    // DomainNameFlat = (LPWSTR)((LPBYTE)Buffer + sizeof(DSROLER_PRIMARY_DOMAIN_INFO_BASIC))
    let flat =
        (buffer as *mut u8).add(size_of::<_DSROLER_PRIMARY_DOMAIN_INFO_BASIC>()) as *mut u16;
    (*buffer).DomainNameFlat = flat;
    wcscpy(flat, dom.DomainName.Buffer);

    LsaIFree_LSAPR_POLICY_INFORMATION(POLICY_ACCOUNT_DOMAIN_INFORMATION, policy_info);

    write_volatile(domain_info, buffer as PDSROLER_PRIMARY_DOMAIN_INFORMATION);
    ERROR_SUCCESS
}

unsafe fn DsRolepGetUpdateStatus(domain_info: *mut PDSROLER_PRIMARY_DOMAIN_INFORMATION) -> u32 {
    let buffer = MIDL_user_allocate(size_of::<_DSROLE_UPGRADE_STATUS_INFO>())
        as *mut _DSROLE_UPGRADE_STATUS_INFO;
    if buffer.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    (*buffer).OperationState = 0;
    (*buffer).PreviousServerState = 0;

    write_volatile(domain_info, buffer as PDSROLER_PRIMARY_DOMAIN_INFORMATION);
    ERROR_SUCCESS
}

unsafe fn DsRolepGetOperationState(domain_info: *mut PDSROLER_PRIMARY_DOMAIN_INFORMATION) -> u32 {
    let buffer = MIDL_user_allocate(size_of::<_DSROLE_OPERATION_STATE_INFO>())
        as *mut _DSROLE_OPERATION_STATE_INFO;
    if buffer.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    (*buffer).OperationState = _DSROLE_OPERATION_STATE_DsRoleOperationIdle;

    write_volatile(domain_info, buffer as PDSROLER_PRIMARY_DOMAIN_INFORMATION);
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn DsRolerGetPrimaryDomainInformation(
    _hBinding: *mut core::ffi::c_void,
    InfoLevel: u32,
    DomainInfo: *mut PDSROLER_PRIMARY_DOMAIN_INFORMATION,
) -> u32 {
    match InfoLevel {
        DS_ROLE_PRIMARY_DOMAIN_INFO_BASIC => DsRolepGetBasicInfo(DomainInfo),
        DS_ROLE_UPGRADE_STATUS => DsRolepGetUpdateStatus(DomainInfo),
        DS_ROLE_OPERATION_STATE => DsRolepGetOperationState(DomainInfo),
        _ => ERROR_CALL_NOT_IMPLEMENTED,
    }
}
