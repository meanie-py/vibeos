use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, NT_SUCCESS, TRUE, FALSE, POOL_RAISE_IF_ALLOCATION_FAILURE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::PagedPool;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

unsafe fn RtlEqualMemory(dst: PVOID, src: PVOID, len: usize) -> BOOLEAN {
    if RtlCompareMemory(dst, src, len as SIZE_T) == len as SIZE_T { 1 } else { 0 }
}

const SE_MAXIMUM_GROUP_LIMIT: u32 = 0x1000;

#[no_mangle]
pub static mut SeNullSidAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 0],
};
#[no_mangle]
pub static mut SeWorldSidAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 1],
};
#[no_mangle]
pub static mut SeLocalSidAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 2],
};
#[no_mangle]
pub static mut SeCreatorSidAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 3],
};
#[no_mangle]
pub static mut SeNtSidAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 5],
};

#[no_mangle]
pub static mut SeNullSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeWorldSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeLocalSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeCreatorOwnerSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeCreatorGroupSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeCreatorOwnerServerSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeCreatorGroupServerSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeNtAuthoritySid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeDialupSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeNetworkSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeBatchSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeInteractiveSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeServiceSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SePrincipalSelfSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeLocalSystemSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeAuthenticatedUserSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeRestrictedCodeSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeAliasAdminsSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeAliasUsersSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeAliasGuestsSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeAliasPowerUsersSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeAliasAccountOpsSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeAliasSystemOpsSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeAliasPrintOpsSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeAliasBackupOpsSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeAuthenticatedUsersSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeRestrictedSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeAnonymousLogonSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeLocalServiceSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut SeNetworkServiceSid: PSID = core::ptr::null_mut();

#[repr(C)]
struct SID_VALIDATE {
    SubAuthorityCount: UCHAR,
    ProbeSid: PISID,
}

unsafe fn FreeInitializedSids() {
    if !SeNullSid.is_null() { ExFreePoolWithTag(SeNullSid as PVOID, TAG_SID); }
    if !SeWorldSid.is_null() { ExFreePoolWithTag(SeWorldSid as PVOID, TAG_SID); }
    if !SeLocalSid.is_null() { ExFreePoolWithTag(SeLocalSid as PVOID, TAG_SID); }
    if !SeCreatorOwnerSid.is_null() { ExFreePoolWithTag(SeCreatorOwnerSid as PVOID, TAG_SID); }
    if !SeCreatorGroupSid.is_null() { ExFreePoolWithTag(SeCreatorGroupSid as PVOID, TAG_SID); }
    if !SeCreatorOwnerServerSid.is_null() { ExFreePoolWithTag(SeCreatorOwnerServerSid as PVOID, TAG_SID); }
    if !SeCreatorGroupServerSid.is_null() { ExFreePoolWithTag(SeCreatorGroupServerSid as PVOID, TAG_SID); }
    if !SeNtAuthoritySid.is_null() { ExFreePoolWithTag(SeNtAuthoritySid as PVOID, TAG_SID); }
    if !SeDialupSid.is_null() { ExFreePoolWithTag(SeDialupSid as PVOID, TAG_SID); }
    if !SeNetworkSid.is_null() { ExFreePoolWithTag(SeNetworkSid as PVOID, TAG_SID); }
    if !SeBatchSid.is_null() { ExFreePoolWithTag(SeBatchSid as PVOID, TAG_SID); }
    if !SeInteractiveSid.is_null() { ExFreePoolWithTag(SeInteractiveSid as PVOID, TAG_SID); }
    if !SeServiceSid.is_null() { ExFreePoolWithTag(SeServiceSid as PVOID, TAG_SID); }
    if !SePrincipalSelfSid.is_null() { ExFreePoolWithTag(SePrincipalSelfSid as PVOID, TAG_SID); }
    if !SeLocalSystemSid.is_null() { ExFreePoolWithTag(SeLocalSystemSid as PVOID, TAG_SID); }
    if !SeAuthenticatedUserSid.is_null() { ExFreePoolWithTag(SeAuthenticatedUserSid as PVOID, TAG_SID); }
    if !SeRestrictedCodeSid.is_null() { ExFreePoolWithTag(SeRestrictedCodeSid as PVOID, TAG_SID); }
    if !SeAliasAdminsSid.is_null() { ExFreePoolWithTag(SeAliasAdminsSid as PVOID, TAG_SID); }
    if !SeAliasUsersSid.is_null() { ExFreePoolWithTag(SeAliasUsersSid as PVOID, TAG_SID); }
    if !SeAliasGuestsSid.is_null() { ExFreePoolWithTag(SeAliasGuestsSid as PVOID, TAG_SID); }
    if !SeAliasPowerUsersSid.is_null() { ExFreePoolWithTag(SeAliasPowerUsersSid as PVOID, TAG_SID); }
    if !SeAliasAccountOpsSid.is_null() { ExFreePoolWithTag(SeAliasAccountOpsSid as PVOID, TAG_SID); }
    if !SeAliasSystemOpsSid.is_null() { ExFreePoolWithTag(SeAliasSystemOpsSid as PVOID, TAG_SID); }
    if !SeAliasPrintOpsSid.is_null() { ExFreePoolWithTag(SeAliasPrintOpsSid as PVOID, TAG_SID); }
    if !SeAliasBackupOpsSid.is_null() { ExFreePoolWithTag(SeAliasBackupOpsSid as PVOID, TAG_SID); }
    if !SeAuthenticatedUsersSid.is_null() { ExFreePoolWithTag(SeAuthenticatedUsersSid as PVOID, TAG_SID); }
    if !SeRestrictedSid.is_null() { ExFreePoolWithTag(SeRestrictedSid as PVOID, TAG_SID); }
    if !SeAnonymousLogonSid.is_null() { ExFreePoolWithTag(SeAnonymousLogonSid as PVOID, TAG_SID); }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn SepInitSecurityIDs() -> BOOLEAN {
    let SidLength0: ULONG;
    let SidLength1: ULONG;
    let SidLength2: ULONG;
    let mut SubAuthority: *mut ULONG;

    SidLength0 = RtlLengthRequiredSid(0);
    SidLength1 = RtlLengthRequiredSid(1);
    SidLength2 = RtlLengthRequiredSid(2);

    SeNullSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeWorldSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeLocalSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeCreatorOwnerSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeCreatorGroupSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeCreatorOwnerServerSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeCreatorGroupServerSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeNtAuthoritySid = ExAllocatePoolWithTag(PagedPool, SidLength0 as SIZE_T, TAG_SID) as PSID;
    SeDialupSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeNetworkSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeBatchSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeInteractiveSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeServiceSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SePrincipalSelfSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeLocalSystemSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeAuthenticatedUserSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeRestrictedCodeSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeAliasAdminsSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID) as PSID;
    SeAliasUsersSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID) as PSID;
    SeAliasGuestsSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID) as PSID;
    SeAliasPowerUsersSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID) as PSID;
    SeAliasAccountOpsSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID) as PSID;
    SeAliasSystemOpsSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID) as PSID;
    SeAliasPrintOpsSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID) as PSID;
    SeAliasBackupOpsSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID) as PSID;
    SeAuthenticatedUsersSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeRestrictedSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeAnonymousLogonSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeLocalServiceSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;
    SeNetworkServiceSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID) as PSID;

    if SeNullSid.is_null() || SeWorldSid.is_null() ||
        SeLocalSid.is_null() || SeCreatorOwnerSid.is_null() ||
        SeCreatorGroupSid.is_null() || SeCreatorOwnerServerSid.is_null() ||
        SeCreatorGroupServerSid.is_null() || SeNtAuthoritySid.is_null() ||
        SeDialupSid.is_null() || SeNetworkSid.is_null() || SeBatchSid.is_null() ||
        SeInteractiveSid.is_null() || SeServiceSid.is_null() ||
        SePrincipalSelfSid.is_null() || SeLocalSystemSid.is_null() ||
        SeAuthenticatedUserSid.is_null() || SeRestrictedCodeSid.is_null() ||
        SeAliasAdminsSid.is_null() || SeAliasUsersSid.is_null() ||
        SeAliasGuestsSid.is_null() || SeAliasPowerUsersSid.is_null() ||
        SeAliasAccountOpsSid.is_null() || SeAliasSystemOpsSid.is_null() ||
        SeAliasPrintOpsSid.is_null() || SeAliasBackupOpsSid.is_null() ||
        SeAuthenticatedUsersSid.is_null() || SeRestrictedSid.is_null() ||
        SeAnonymousLogonSid.is_null() || SeLocalServiceSid.is_null() ||
        SeNetworkServiceSid.is_null()
    {
        FreeInitializedSids();
        return 0u8;
    }

    RtlInitializeSid(SeNullSid, core::ptr::addr_of_mut!(SeNullSidAuthority), 1);
    RtlInitializeSid(SeWorldSid, core::ptr::addr_of_mut!(SeWorldSidAuthority), 1);
    RtlInitializeSid(SeLocalSid, core::ptr::addr_of_mut!(SeLocalSidAuthority), 1);
    RtlInitializeSid(SeCreatorOwnerSid, core::ptr::addr_of_mut!(SeCreatorSidAuthority), 1);
    RtlInitializeSid(SeCreatorGroupSid, core::ptr::addr_of_mut!(SeCreatorSidAuthority), 1);
    RtlInitializeSid(SeCreatorOwnerServerSid, core::ptr::addr_of_mut!(SeCreatorSidAuthority), 1);
    RtlInitializeSid(SeCreatorGroupServerSid, core::ptr::addr_of_mut!(SeCreatorSidAuthority), 1);
    RtlInitializeSid(SeNtAuthoritySid, core::ptr::addr_of_mut!(SeNtSidAuthority), 0);
    RtlInitializeSid(SeDialupSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SeNetworkSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SeBatchSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SeInteractiveSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SeServiceSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SePrincipalSelfSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SeLocalSystemSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SeAuthenticatedUserSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SeRestrictedCodeSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SeAliasAdminsSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 2);
    RtlInitializeSid(SeAliasUsersSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 2);
    RtlInitializeSid(SeAliasGuestsSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 2);
    RtlInitializeSid(SeAliasPowerUsersSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 2);
    RtlInitializeSid(SeAliasAccountOpsSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 2);
    RtlInitializeSid(SeAliasSystemOpsSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 2);
    RtlInitializeSid(SeAliasPrintOpsSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 2);
    RtlInitializeSid(SeAliasBackupOpsSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 2);
    RtlInitializeSid(SeAuthenticatedUsersSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SeRestrictedSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SeAnonymousLogonSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SeLocalServiceSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);
    RtlInitializeSid(SeNetworkServiceSid, core::ptr::addr_of_mut!(SeNtSidAuthority), 1);

    SubAuthority = RtlSubAuthoritySid(SeNullSid, 0);
    *SubAuthority = SECURITY_NULL_RID;
    SubAuthority = RtlSubAuthoritySid(SeWorldSid, 0);
    *SubAuthority = SECURITY_WORLD_RID;
    SubAuthority = RtlSubAuthoritySid(SeLocalSid, 0);
    *SubAuthority = SECURITY_LOCAL_RID;
    SubAuthority = RtlSubAuthoritySid(SeCreatorOwnerSid, 0);
    *SubAuthority = SECURITY_CREATOR_OWNER_RID;
    SubAuthority = RtlSubAuthoritySid(SeCreatorGroupSid, 0);
    *SubAuthority = SECURITY_CREATOR_GROUP_RID;
    SubAuthority = RtlSubAuthoritySid(SeCreatorOwnerServerSid, 0);
    *SubAuthority = SECURITY_CREATOR_OWNER_SERVER_RID;
    SubAuthority = RtlSubAuthoritySid(SeCreatorGroupServerSid, 0);
    *SubAuthority = SECURITY_CREATOR_GROUP_SERVER_RID;
    SubAuthority = RtlSubAuthoritySid(SeDialupSid, 0);
    *SubAuthority = SECURITY_DIALUP_RID;
    SubAuthority = RtlSubAuthoritySid(SeNetworkSid, 0);
    *SubAuthority = SECURITY_NETWORK_RID;
    SubAuthority = RtlSubAuthoritySid(SeBatchSid, 0);
    *SubAuthority = SECURITY_BATCH_RID;
    SubAuthority = RtlSubAuthoritySid(SeInteractiveSid, 0);
    *SubAuthority = SECURITY_INTERACTIVE_RID;
    SubAuthority = RtlSubAuthoritySid(SeServiceSid, 0);
    *SubAuthority = SECURITY_SERVICE_RID;
    SubAuthority = RtlSubAuthoritySid(SePrincipalSelfSid, 0);
    *SubAuthority = SECURITY_PRINCIPAL_SELF_RID;
    SubAuthority = RtlSubAuthoritySid(SeLocalSystemSid, 0);
    *SubAuthority = SECURITY_LOCAL_SYSTEM_RID;
    SubAuthority = RtlSubAuthoritySid(SeAuthenticatedUserSid, 0);
    *SubAuthority = SECURITY_AUTHENTICATED_USER_RID;
    SubAuthority = RtlSubAuthoritySid(SeRestrictedCodeSid, 0);
    *SubAuthority = SECURITY_RESTRICTED_CODE_RID;
    SubAuthority = RtlSubAuthoritySid(SeAliasAdminsSid, 0);
    *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
    SubAuthority = RtlSubAuthoritySid(SeAliasAdminsSid, 1);
    *SubAuthority = DOMAIN_ALIAS_RID_ADMINS;
    SubAuthority = RtlSubAuthoritySid(SeAliasUsersSid, 0);
    *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
    SubAuthority = RtlSubAuthoritySid(SeAliasUsersSid, 1);
    *SubAuthority = DOMAIN_ALIAS_RID_USERS;
    SubAuthority = RtlSubAuthoritySid(SeAliasGuestsSid, 0);
    *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
    SubAuthority = RtlSubAuthoritySid(SeAliasGuestsSid, 1);
    *SubAuthority = DOMAIN_ALIAS_RID_GUESTS;
    SubAuthority = RtlSubAuthoritySid(SeAliasPowerUsersSid, 0);
    *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
    SubAuthority = RtlSubAuthoritySid(SeAliasPowerUsersSid, 1);
    *SubAuthority = DOMAIN_ALIAS_RID_POWER_USERS;
    SubAuthority = RtlSubAuthoritySid(SeAliasAccountOpsSid, 0);
    *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
    SubAuthority = RtlSubAuthoritySid(SeAliasAccountOpsSid, 1);
    *SubAuthority = DOMAIN_ALIAS_RID_ACCOUNT_OPS;
    SubAuthority = RtlSubAuthoritySid(SeAliasSystemOpsSid, 0);
    *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
    SubAuthority = RtlSubAuthoritySid(SeAliasSystemOpsSid, 1);
    *SubAuthority = DOMAIN_ALIAS_RID_SYSTEM_OPS;
    SubAuthority = RtlSubAuthoritySid(SeAliasPrintOpsSid, 0);
    *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
    SubAuthority = RtlSubAuthoritySid(SeAliasPrintOpsSid, 1);
    *SubAuthority = DOMAIN_ALIAS_RID_PRINT_OPS;
    SubAuthority = RtlSubAuthoritySid(SeAliasBackupOpsSid, 0);
    *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
    SubAuthority = RtlSubAuthoritySid(SeAliasBackupOpsSid, 1);
    *SubAuthority = DOMAIN_ALIAS_RID_BACKUP_OPS;
    SubAuthority = RtlSubAuthoritySid(SeAuthenticatedUsersSid, 0);
    *SubAuthority = SECURITY_AUTHENTICATED_USER_RID;
    SubAuthority = RtlSubAuthoritySid(SeRestrictedSid, 0);
    *SubAuthority = SECURITY_RESTRICTED_CODE_RID;
    SubAuthority = RtlSubAuthoritySid(SeAnonymousLogonSid, 0);
    *SubAuthority = SECURITY_ANONYMOUS_LOGON_RID;
    SubAuthority = RtlSubAuthoritySid(SeLocalServiceSid, 0);
    *SubAuthority = SECURITY_LOCAL_SERVICE_RID;
    SubAuthority = RtlSubAuthoritySid(SeNetworkServiceSid, 0);
    *SubAuthority = SECURITY_NETWORK_SERVICE_RID;

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn SepCaptureSid(
    InputSid: PSID,
    AccessMode: KPROCESSOR_MODE,
    PoolType: POOL_TYPE,
    CaptureIfKernel: BOOLEAN,
    CapturedSid: *mut PSID,
) -> NTSTATUS {
    let InputSid = opaque_ptr(InputSid);
    let CapturedSid = opaque_ptr(CapturedSid);
    let mut SidSize: ULONG = 0;
    let mut NewSid: PISID;
    let Sid = InputSid as PISID;

    if AccessMode != 0i8 {
        if let Err(s) = probe_user_buffer(Sid as _, core::mem::offset_of!(SID, SubAuthority), core::mem::size_of::<UCHAR>(), AccessMode) {
            return s;
        }
        // The count is read from user memory, so read it fault-tolerantly. The C
        // dereferences it inside its _SEH2_TRY.
        let mut SubAuthorityCount: UCHAR = 0;
        if let Err(s) = safe_copy_from_user(
            core::ptr::addr_of_mut!(SubAuthorityCount) as PVOID,
            core::ptr::addr_of!((*Sid).SubAuthorityCount) as _,
            core::mem::size_of::<UCHAR>(),
            core::mem::size_of::<UCHAR>(),
            AccessMode,
        ) {
            return s;
        }
        SidSize = RtlLengthRequiredSid(SubAuthorityCount as ULONG);
        if let Err(s) = probe_user_buffer(Sid as _, SidSize as usize, core::mem::size_of::<UCHAR>(), AccessMode) {
            return s;
        }

        NewSid = ExAllocatePoolWithTag(PoolType, SidSize as SIZE_T, TAG_SID) as PISID;
        if NewSid.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        let _guard = PoolAllocGuard::new(NewSid as PVOID, TAG_SID);
        if let Err(s) = safe_copy_from_user(NewSid as PVOID, Sid as _, SidSize as usize, core::mem::size_of::<UCHAR>(), AccessMode) {
            return s;
        }
        // The caller can change the count between the read above and this copy, and
        // the copy is only SidSize bytes. Pin it to the count the size came from, or
        // every later reader of the captured SID walks past the allocation. (The C
        // re-reads the count from user memory and has the same window.)
        (*NewSid).SubAuthorityCount = SubAuthorityCount;
        // CapturedSid is always a kernel-mode pointer (caller's output param) — direct write, no probe
        *CapturedSid = NewSid as PSID;
        _guard.disarm();
    } else if CaptureIfKernel == 0u8 {
        *CapturedSid = InputSid;
    } else {
        SidSize = RtlLengthRequiredSid((*Sid).SubAuthorityCount as ULONG);

        NewSid = ExAllocatePoolWithTag(PoolType, SidSize as SIZE_T, TAG_SID) as PISID;
        if NewSid.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::copy_nonoverlapping(Sid as *const u8, NewSid as *mut u8, SidSize as usize);
        *CapturedSid = NewSid as PSID;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SepReleaseSid(
    CapturedSid: PSID,
    AccessMode: KPROCESSOR_MODE,
    CaptureIfKernel: BOOLEAN,
) {
    let CapturedSid = opaque_ptr(CapturedSid);
    if !CapturedSid.is_null() &&
        (AccessMode != 0i8 || (AccessMode == 0i8 && CaptureIfKernel != 0u8))
    {
        ExFreePoolWithTag(CapturedSid as PVOID, TAG_SID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn SepSidInTokenEx(
    _Token: PACCESS_TOKEN,
    PrincipalSelfSid: PSID,
    _Sid: PSID,
    Deny: BOOLEAN,
    Restricted: BOOLEAN,
) -> BOOLEAN {
    let PrincipalSelfSid = opaque_ptr(PrincipalSelfSid);
    let _Sid = opaque_ptr(_Sid);
    let mut SidIndex: ULONG;
    let Token = _Token as PTOKEN;
    let mut TokenSid: PISID;
    let mut Sid = _Sid as PISID;
    let mut SidAndAttributes: PSID_AND_ATTRIBUTES;
    let SidCount: ULONG;
    let SidLength: ULONG;
    let SidMetadata: USHORT;

    if !PrincipalSelfSid.is_null() && RtlEqualSid(SePrincipalSelfSid, Sid as PSID) != 0 {
        Sid = PrincipalSelfSid as PISID;
    }

    if Restricted != 0u8 {
        SidAndAttributes = (*Token).RestrictedSids;
        SidCount = (*Token).RestrictedSidCount;
    } else {
        SidAndAttributes = (*Token).UserAndGroups;
        SidCount = (*Token).UserAndGroupCount;
    }

    SidLength = (core::mem::offset_of!(SID, SubAuthority) + ((*Sid).SubAuthorityCount as usize * core::mem::size_of::<ULONG>())) as ULONG;
    SidMetadata = *(core::ptr::addr_of!((*Sid).Revision) as *const USHORT);

    SidIndex = 0;
    while SidIndex < SidCount {
        TokenSid = (*SidAndAttributes).Sid as PISID;

        if *(core::ptr::addr_of!((*TokenSid).Revision) as *const USHORT) == SidMetadata {
            if RtlEqualMemory(Sid as PVOID, TokenSid as PVOID, SidLength as usize) != 0 {
                if (Restricted == 0u8 && SidIndex == 0 && ((*SidAndAttributes).Attributes & SE_GROUP_USE_FOR_DENY_ONLY) == 0) ||
                    ((*SidAndAttributes).Attributes & SE_GROUP_ENABLED) != 0 ||
                    (Deny != 0u8 && ((*SidAndAttributes).Attributes & SE_GROUP_USE_FOR_DENY_ONLY) != 0)
                {
                    return 1u8;
                } else {
                    return 0u8;
                }
            }
        }

        SidAndAttributes = SidAndAttributes.offset(1);
        SidIndex += 1;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn SepSidInToken(
    _Token: PACCESS_TOKEN,
    Sid: PSID,
) -> BOOLEAN {
    let Sid = opaque_ptr(Sid);
    SepSidInTokenEx(_Token, core::ptr::null_mut(), Sid, 0u8, 0u8)
}

#[no_mangle]
pub unsafe extern "system" fn SepGetSidFromAce(
    Ace: PACE,
) -> PSID {
    let mut Flags: *mut ULONG;
    let mut GuidSize: ULONG = 0;
    let mut Sid: PSID = core::ptr::null_mut();

    match (*Ace).Header.AceType {
        ACCESS_DENIED_ACE_TYPE | ACCESS_ALLOWED_ACE_TYPE | SYSTEM_AUDIT_ACE_TYPE | SYSTEM_ALARM_ACE_TYPE => {
            Sid = core::ptr::addr_of!((*(Ace as PKNOWN_ACE)).SidStart) as PSID;
        }
        ACCESS_DENIED_OBJECT_ACE_TYPE | ACCESS_ALLOWED_OBJECT_ACE_TYPE => {
            Flags = core::ptr::addr_of_mut!((*(Ace as PKNOWN_OBJECT_ACE)).Flags);
            if (*Flags & ACE_OBJECT_TYPE_PRESENT) != 0 {
                GuidSize += core::mem::size_of::<GUID>() as ULONG;
            }
            if (*Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT) != 0 {
                GuidSize += core::mem::size_of::<GUID>() as ULONG;
            }
            Sid = (core::ptr::addr_of!((*(Ace as PKNOWN_OBJECT_ACE)).SidStart) as usize + GuidSize as usize) as PSID;
        }
        _ => {}
    }

    Sid
}

#[no_mangle]
pub unsafe extern "system" fn SeCaptureSidAndAttributesArray(
    SrcSidAndAttributes: PSID_AND_ATTRIBUTES,
    AttributeCount: ULONG,
    PreviousMode: KPROCESSOR_MODE,
    AllocatedMem: PVOID,
    AllocatedLength: ULONG,
    PoolType: POOL_TYPE,
    CaptureIfKernel: BOOLEAN,
    CapturedSidAndAttributes: *mut PSID_AND_ATTRIBUTES,
    ResultLength: *mut ULONG,
) -> NTSTATUS {
    let AllocatedMem = opaque_ptr(AllocatedMem);
    let CapturedSidAndAttributes = opaque_ptr(CapturedSidAndAttributes);
    let ResultLength = opaque_ptr(ResultLength);
    let mut ArraySize: ULONG;
    let mut RequiredLength: ULONG;
    let mut SidLength: ULONG;
    let mut i: ULONG;
    let mut TempArrayValidate: ULONG;
    let mut TempLengthValidate: ULONG;
    let mut SidAndAttributes: PSID_AND_ATTRIBUTES;
    let mut ValidateArray: *mut SID_VALIDATE = core::ptr::null_mut();
    let mut CurrentDest: *mut UCHAR;
    let mut Sid: PISID;
    let mut Status: NTSTATUS;

    *CapturedSidAndAttributes = core::ptr::null_mut();
    *ResultLength = 0;

    if AttributeCount == 0 {
        return STATUS_SUCCESS;
    }

    if AttributeCount > SE_MAXIMUM_GROUP_LIMIT {
        return STATUS_INVALID_PARAMETER;
    }

    if PreviousMode == 0i8 && CaptureIfKernel == 0u8 {
        *CapturedSidAndAttributes = SrcSidAndAttributes;
        return STATUS_SUCCESS;
    }

    ArraySize = AttributeCount * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG;
    RequiredLength = ((ArraySize + core::mem::size_of::<ULONG>() as ULONG - 1) / core::mem::size_of::<ULONG>() as ULONG) * core::mem::size_of::<ULONG>() as ULONG;

    if PreviousMode != 0i8 {
        // Every read of SrcSidAndAttributes and of the SIDs it points at is a read of
        // user memory, which the C does inside _SEH2_TRY. Here each one goes through
        // the fault-tolerant copy and an Err lands on the C's Cleanup label.
        if let Err(s) = probe_user_buffer(SrcSidAndAttributes as _, ArraySize as usize, core::mem::size_of::<ULONG>(), PreviousMode) {
            return s;
        }

        TempArrayValidate = AttributeCount * core::mem::size_of::<SID_VALIDATE>() as ULONG;
        TempLengthValidate = ((TempArrayValidate + core::mem::size_of::<ULONG>() as ULONG - 1) / core::mem::size_of::<ULONG>() as ULONG) * core::mem::size_of::<ULONG>() as ULONG;

        // The C passes POOL_RAISE_IF_ALLOCATION_FAILURE, which is a raise as well.
        ValidateArray = ExAllocatePoolWithTag(PoolType, TempLengthValidate as SIZE_T, TAG_SID_VALIDATE) as *mut SID_VALIDATE;
        if ValidateArray.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        // Freed on every exit, as the C's Cleanup does.
        let _validate_guard = PoolAllocGuard::new(ValidateArray as PVOID, TAG_SID_VALIDATE);

        let read_entry = |i: ULONG| -> Result<SID_AND_ATTRIBUTES, NTSTATUS> {
            let mut entry: SID_AND_ATTRIBUTES = core::mem::zeroed();
            safe_copy_from_user(
                core::ptr::addr_of_mut!(entry) as PVOID,
                SrcSidAndAttributes.offset(i as isize) as _,
                core::mem::size_of::<SID_AND_ATTRIBUTES>(),
                1,
                PreviousMode,
            )?;
            Ok(entry)
        };

        i = 0;
        while i < AttributeCount {
            Sid = match read_entry(i) {
                Ok(entry) => entry.Sid as PISID,
                Err(s) => return s,
            };
            if let Err(s) = probe_user_buffer(Sid as _, core::mem::size_of::<SID>(), core::mem::size_of::<ULONG>(), PreviousMode) {
                return s;
            }

            let mut SubAuthorityCount: UCHAR = 0;
            if let Err(s) = safe_copy_from_user(
                core::ptr::addr_of_mut!(SubAuthorityCount) as PVOID,
                core::ptr::addr_of!((*Sid).SubAuthorityCount) as _,
                core::mem::size_of::<UCHAR>(),
                1,
                PreviousMode,
            ) {
                return s;
            }
            (*ValidateArray.offset(i as isize)).SubAuthorityCount = SubAuthorityCount;
            (*ValidateArray.offset(i as isize)).ProbeSid = Sid;

            SidLength = RtlLengthRequiredSid(SubAuthorityCount as ULONG);
            if let Err(s) = probe_user_buffer(Sid as _, SidLength as usize, core::mem::size_of::<ULONG>(), PreviousMode) {
                return s;
            }

            RequiredLength += ((SidLength + core::mem::size_of::<ULONG>() as ULONG - 1) / core::mem::size_of::<ULONG>() as ULONG) * core::mem::size_of::<ULONG>() as ULONG;
            i += 1;
        }

        *ResultLength = RequiredLength;

        if AllocatedMem.is_null() {
            SidAndAttributes = ExAllocatePoolWithTag(PoolType, RequiredLength as SIZE_T, TAG_SID_AND_ATTRIBUTES) as PSID_AND_ATTRIBUTES;
            if SidAndAttributes.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
        } else if AllocatedLength >= RequiredLength {
            SidAndAttributes = AllocatedMem as PSID_AND_ATTRIBUTES;
        } else {
            return STATUS_BUFFER_TOO_SMALL;
        }

        CurrentDest = SidAndAttributes as *mut UCHAR;
        CurrentDest = CurrentDest.offset((((ArraySize + core::mem::size_of::<ULONG>() as ULONG - 1) / core::mem::size_of::<ULONG>() as ULONG) * core::mem::size_of::<ULONG>() as ULONG) as isize);

        Status = STATUS_SUCCESS;
        i = 0;
        while i < AttributeCount {
            SidLength = RtlLengthRequiredSid((*ValidateArray.offset(i as isize)).SubAuthorityCount as ULONG);

            match read_entry(i) {
                Ok(entry) => (*SidAndAttributes.offset(i as isize)).Attributes = entry.Attributes,
                Err(s) => {
                    Status = s;
                    break;
                }
            }
            (*SidAndAttributes.offset(i as isize)).Sid = CurrentDest as PSID;
            if let Err(s) = safe_copy_from_user(CurrentDest as PVOID, (*ValidateArray.offset(i as isize)).ProbeSid as _, SidLength as usize, core::mem::size_of::<ULONG>(), PreviousMode) {
                Status = s;
                break;
            }

            Sid = (*SidAndAttributes.offset(i as isize)).Sid as PISID;

            if (*ValidateArray.offset(i as isize)).SubAuthorityCount != (*Sid).SubAuthorityCount {
                Status = STATUS_INVALID_SID;
                break;
            }

            if SidLength != RtlLengthSid(Sid as PSID) {
                Status = STATUS_INVALID_SID;
                break;
            }

            if RtlValidSid(Sid as PSID) == 0 {
                Status = STATUS_INVALID_SID;
                break;
            }

            CurrentDest = CurrentDest.offset((((SidLength + core::mem::size_of::<ULONG>() as ULONG - 1) / core::mem::size_of::<ULONG>() as ULONG) * core::mem::size_of::<ULONG>() as ULONG) as isize);
            i += 1;
        }

        // C Cleanup: a failed capture frees the array it allocated and returns NULL.
        // (The previous translation freed it through a guard and still returned it.)
        if NT_SUCCESS(Status) {
            *CapturedSidAndAttributes = SidAndAttributes;
        } else if SidAndAttributes != AllocatedMem as PSID_AND_ATTRIBUTES {
            ExFreePoolWithTag(SidAndAttributes as PVOID, TAG_SID_AND_ATTRIBUTES);
        }
    } else {
        i = 0;
        while i < AttributeCount {
            Sid = (*SrcSidAndAttributes.offset(i as isize)).Sid as PISID;
            SidLength = RtlLengthRequiredSid((*Sid).SubAuthorityCount as ULONG);

            RequiredLength += ((SidLength + core::mem::size_of::<ULONG>() as ULONG - 1) / core::mem::size_of::<ULONG>() as ULONG) * core::mem::size_of::<ULONG>() as ULONG;
            i += 1;
        }

        Status = STATUS_SUCCESS;
        *ResultLength = RequiredLength;

        if AllocatedMem.is_null() {
            SidAndAttributes = ExAllocatePoolWithTag(PoolType, RequiredLength as SIZE_T, TAG_SID_AND_ATTRIBUTES) as PSID_AND_ATTRIBUTES;
            if SidAndAttributes.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                SidAndAttributes = core::ptr::null_mut();
            }
        } else if AllocatedLength >= RequiredLength {
            SidAndAttributes = AllocatedMem as PSID_AND_ATTRIBUTES;
        } else {
            Status = STATUS_BUFFER_TOO_SMALL;
            SidAndAttributes = core::ptr::null_mut();
        }

        if !SidAndAttributes.is_null() {
            let _sid_guard = PoolAllocGuard::new(
                if SidAndAttributes != AllocatedMem as PSID_AND_ATTRIBUTES { SidAndAttributes as PVOID } else { core::ptr::null_mut() },
                TAG_SID_AND_ATTRIBUTES,
            );
            *CapturedSidAndAttributes = SidAndAttributes;

            CurrentDest = SidAndAttributes as *mut UCHAR;
            CurrentDest = CurrentDest.offset((((ArraySize + core::mem::size_of::<ULONG>() as ULONG - 1) / core::mem::size_of::<ULONG>() as ULONG) * core::mem::size_of::<ULONG>() as ULONG) as isize);

            i = 0;
            while i < AttributeCount {
                Sid = (*SrcSidAndAttributes.offset(i as isize)).Sid as PISID;
                SidLength = RtlLengthRequiredSid((*Sid).SubAuthorityCount as ULONG);

                (*SidAndAttributes.offset(i as isize)).Attributes = (*SrcSidAndAttributes.offset(i as isize)).Attributes;
                (*SidAndAttributes.offset(i as isize)).Sid = CurrentDest as PSID;
                core::ptr::copy_nonoverlapping((*SrcSidAndAttributes.offset(i as isize)).Sid as *const u8, CurrentDest, SidLength as usize);

                CurrentDest = CurrentDest.offset((((SidLength + core::mem::size_of::<ULONG>() as ULONG - 1) / core::mem::size_of::<ULONG>() as ULONG) * core::mem::size_of::<ULONG>() as ULONG) as isize);
                i += 1;
            }

            if NT_SUCCESS(Status) {
                _sid_guard.disarm();
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SeReleaseSidAndAttributesArray(
    CapturedSidAndAttributes: PSID_AND_ATTRIBUTES,
    AccessMode: KPROCESSOR_MODE,
    CaptureIfKernel: BOOLEAN,
) {
    if !CapturedSidAndAttributes.is_null() &&
        (AccessMode != 0i8 || CaptureIfKernel != 0u8)
    {
        ExFreePoolWithTag(CapturedSidAndAttributes as PVOID, TAG_SID_AND_ATTRIBUTES);
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Security Identifier (SID) implementation support and handling
//  * COPYRIGHT:   Copyright David Welch <welch@cwcom.net>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// #define SE_MAXIMUM_GROUP_LIMIT 0x1000
//
// SID_IDENTIFIER_AUTHORITY SeNullSidAuthority = {SECURITY_NULL_SID_AUTHORITY};
// SID_IDENTIFIER_AUTHORITY SeWorldSidAuthority = {SECURITY_WORLD_SID_AUTHORITY};
// SID_IDENTIFIER_AUTHORITY SeLocalSidAuthority = {SECURITY_LOCAL_SID_AUTHORITY};
// SID_IDENTIFIER_AUTHORITY SeCreatorSidAuthority = {SECURITY_CREATOR_SID_AUTHORITY};
// SID_IDENTIFIER_AUTHORITY SeNtSidAuthority = {SECURITY_NT_AUTHORITY};
//
// PSID SeNullSid = NULL;
// PSID SeWorldSid = NULL;
// PSID SeLocalSid = NULL;
// PSID SeCreatorOwnerSid = NULL;
// PSID SeCreatorGroupSid = NULL;
// PSID SeCreatorOwnerServerSid = NULL;
// PSID SeCreatorGroupServerSid = NULL;
// PSID SeNtAuthoritySid = NULL;
// PSID SeDialupSid = NULL;
// PSID SeNetworkSid = NULL;
// PSID SeBatchSid = NULL;
// PSID SeInteractiveSid = NULL;
// PSID SeServiceSid = NULL;
// PSID SePrincipalSelfSid = NULL;
// PSID SeLocalSystemSid = NULL;
// PSID SeAuthenticatedUserSid = NULL;
// PSID SeRestrictedCodeSid = NULL;
// PSID SeAliasAdminsSid = NULL;
// PSID SeAliasUsersSid = NULL;
// PSID SeAliasGuestsSid = NULL;
// PSID SeAliasPowerUsersSid = NULL;
// PSID SeAliasAccountOpsSid = NULL;
// PSID SeAliasSystemOpsSid = NULL;
// PSID SeAliasPrintOpsSid = NULL;
// PSID SeAliasBackupOpsSid = NULL;
// PSID SeAuthenticatedUsersSid = NULL;
// PSID SeRestrictedSid = NULL;
// PSID SeAnonymousLogonSid = NULL;
// PSID SeLocalServiceSid = NULL;
// PSID SeNetworkServiceSid = NULL;
//
// typedef struct _SID_VALIDATE
// {
//     UCHAR SubAuthorityCount;
//     PISID ProbeSid;
// } SID_VALIDATE, *PSID_VALIDATE;
//
// /* FUNCTIONS ******************************************************************/
//
// /**
//  * @brief
//  * Frees all the known initialized SIDs in the system from the memory.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// FreeInitializedSids(VOID)
// {
//     if (SeNullSid) ExFreePoolWithTag(SeNullSid, TAG_SID);
//     if (SeWorldSid) ExFreePoolWithTag(SeWorldSid, TAG_SID);
//     if (SeLocalSid) ExFreePoolWithTag(SeLocalSid, TAG_SID);
//     if (SeCreatorOwnerSid) ExFreePoolWithTag(SeCreatorOwnerSid, TAG_SID);
//     if (SeCreatorGroupSid) ExFreePoolWithTag(SeCreatorGroupSid, TAG_SID);
//     if (SeCreatorOwnerServerSid) ExFreePoolWithTag(SeCreatorOwnerServerSid, TAG_SID);
//     if (SeCreatorGroupServerSid) ExFreePoolWithTag(SeCreatorGroupServerSid, TAG_SID);
//     if (SeNtAuthoritySid) ExFreePoolWithTag(SeNtAuthoritySid, TAG_SID);
//     if (SeDialupSid) ExFreePoolWithTag(SeDialupSid, TAG_SID);
//     if (SeNetworkSid) ExFreePoolWithTag(SeNetworkSid, TAG_SID);
//     if (SeBatchSid) ExFreePoolWithTag(SeBatchSid, TAG_SID);
//     if (SeInteractiveSid) ExFreePoolWithTag(SeInteractiveSid, TAG_SID);
//     if (SeServiceSid) ExFreePoolWithTag(SeServiceSid, TAG_SID);
//     if (SePrincipalSelfSid) ExFreePoolWithTag(SePrincipalSelfSid, TAG_SID);
//     if (SeLocalSystemSid) ExFreePoolWithTag(SeLocalSystemSid, TAG_SID);
//     if (SeAuthenticatedUserSid) ExFreePoolWithTag(SeAuthenticatedUserSid, TAG_SID);
//     if (SeRestrictedCodeSid) ExFreePoolWithTag(SeRestrictedCodeSid, TAG_SID);
//     if (SeAliasAdminsSid) ExFreePoolWithTag(SeAliasAdminsSid, TAG_SID);
//     if (SeAliasUsersSid) ExFreePoolWithTag(SeAliasUsersSid, TAG_SID);
//     if (SeAliasGuestsSid) ExFreePoolWithTag(SeAliasGuestsSid, TAG_SID);
//     if (SeAliasPowerUsersSid) ExFreePoolWithTag(SeAliasPowerUsersSid, TAG_SID);
//     if (SeAliasAccountOpsSid) ExFreePoolWithTag(SeAliasAccountOpsSid, TAG_SID);
//     if (SeAliasSystemOpsSid) ExFreePoolWithTag(SeAliasSystemOpsSid, TAG_SID);
//     if (SeAliasPrintOpsSid) ExFreePoolWithTag(SeAliasPrintOpsSid, TAG_SID);
//     if (SeAliasBackupOpsSid) ExFreePoolWithTag(SeAliasBackupOpsSid, TAG_SID);
//     if (SeAuthenticatedUsersSid) ExFreePoolWithTag(SeAuthenticatedUsersSid, TAG_SID);
//     if (SeRestrictedSid) ExFreePoolWithTag(SeRestrictedSid, TAG_SID);
//     if (SeAnonymousLogonSid) ExFreePoolWithTag(SeAnonymousLogonSid, TAG_SID);
// }
//
// /**
//  * @brief
//  * Initializes all the SIDs known in the system.
//  *
//  * @return
//  * Returns TRUE if all the SIDs have been initialized,
//  * FALSE otherwise.
//  */
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// SepInitSecurityIDs(VOID)
// {
//     ULONG SidLength0;
//     ULONG SidLength1;
//     ULONG SidLength2;
//     PULONG SubAuthority;
//
//     SidLength0 = RtlLengthRequiredSid(0);
//     SidLength1 = RtlLengthRequiredSid(1);
//     SidLength2 = RtlLengthRequiredSid(2);
//
//     /* create NullSid */
//     SeNullSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeWorldSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeLocalSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeCreatorOwnerSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeCreatorGroupSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeCreatorOwnerServerSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeCreatorGroupServerSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeNtAuthoritySid = ExAllocatePoolWithTag(PagedPool, SidLength0 as SIZE_T, TAG_SID);
//     SeDialupSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeNetworkSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeBatchSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeInteractiveSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeServiceSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SePrincipalSelfSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeLocalSystemSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeAuthenticatedUserSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeRestrictedCodeSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeAliasAdminsSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID);
//     SeAliasUsersSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID);
//     SeAliasGuestsSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID);
//     SeAliasPowerUsersSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID);
//     SeAliasAccountOpsSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID);
//     SeAliasSystemOpsSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID);
//     SeAliasPrintOpsSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID);
//     SeAliasBackupOpsSid = ExAllocatePoolWithTag(PagedPool, SidLength2 as SIZE_T, TAG_SID);
//     SeAuthenticatedUsersSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeRestrictedSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeAnonymousLogonSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeLocalServiceSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//     SeNetworkServiceSid = ExAllocatePoolWithTag(PagedPool, SidLength1 as SIZE_T, TAG_SID);
//
//     if (SeNullSid == NULL || SeWorldSid == NULL ||
//         SeLocalSid == NULL || SeCreatorOwnerSid == NULL ||
//         SeCreatorGroupSid == NULL || SeCreatorOwnerServerSid == NULL ||
//         SeCreatorGroupServerSid == NULL || SeNtAuthoritySid == NULL ||
//         SeDialupSid == NULL || SeNetworkSid == NULL || SeBatchSid == NULL ||
//         SeInteractiveSid == NULL || SeServiceSid == NULL ||
//         SePrincipalSelfSid == NULL || SeLocalSystemSid == NULL ||
//         SeAuthenticatedUserSid == NULL || SeRestrictedCodeSid == NULL ||
//         SeAliasAdminsSid == NULL || SeAliasUsersSid == NULL ||
//         SeAliasGuestsSid == NULL || SeAliasPowerUsersSid == NULL ||
//         SeAliasAccountOpsSid == NULL || SeAliasSystemOpsSid == NULL ||
//         SeAliasPrintOpsSid == NULL || SeAliasBackupOpsSid == NULL ||
//         SeAuthenticatedUsersSid == NULL || SeRestrictedSid == NULL ||
//         SeAnonymousLogonSid == NULL || SeLocalServiceSid == NULL ||
//         SeNetworkServiceSid == NULL)
//     {
//         FreeInitializedSids();
//         return FALSE;
//     }
//
//     RtlInitializeSid(SeNullSid, &SeNullSidAuthority, 1);
//     RtlInitializeSid(SeWorldSid, &SeWorldSidAuthority, 1);
//     RtlInitializeSid(SeLocalSid, &SeLocalSidAuthority, 1);
//     RtlInitializeSid(SeCreatorOwnerSid, &SeCreatorSidAuthority, 1);
//     RtlInitializeSid(SeCreatorGroupSid, &SeCreatorSidAuthority, 1);
//     RtlInitializeSid(SeCreatorOwnerServerSid, &SeCreatorSidAuthority, 1);
//     RtlInitializeSid(SeCreatorGroupServerSid, &SeCreatorSidAuthority, 1);
//     RtlInitializeSid(SeNtAuthoritySid, &SeNtSidAuthority, 0);
//     RtlInitializeSid(SeDialupSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SeNetworkSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SeBatchSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SeInteractiveSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SeServiceSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SePrincipalSelfSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SeLocalSystemSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SeAuthenticatedUserSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SeRestrictedCodeSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SeAliasAdminsSid, &SeNtSidAuthority, 2);
//     RtlInitializeSid(SeAliasUsersSid, &SeNtSidAuthority, 2);
//     RtlInitializeSid(SeAliasGuestsSid, &SeNtSidAuthority, 2);
//     RtlInitializeSid(SeAliasPowerUsersSid, &SeNtSidAuthority, 2);
//     RtlInitializeSid(SeAliasAccountOpsSid, &SeNtSidAuthority, 2);
//     RtlInitializeSid(SeAliasSystemOpsSid, &SeNtSidAuthority, 2);
//     RtlInitializeSid(SeAliasPrintOpsSid, &SeNtSidAuthority, 2);
//     RtlInitializeSid(SeAliasBackupOpsSid, &SeNtSidAuthority, 2);
//     RtlInitializeSid(SeAuthenticatedUsersSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SeRestrictedSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SeAnonymousLogonSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SeLocalServiceSid, &SeNtSidAuthority, 1);
//     RtlInitializeSid(SeNetworkServiceSid, &SeNtSidAuthority, 1);
//
//     SubAuthority = RtlSubAuthoritySid(SeNullSid, 0);
//     *SubAuthority = SECURITY_NULL_RID;
//     SubAuthority = RtlSubAuthoritySid(SeWorldSid, 0);
//     *SubAuthority = SECURITY_WORLD_RID;
//     SubAuthority = RtlSubAuthoritySid(SeLocalSid, 0);
//     *SubAuthority = SECURITY_LOCAL_RID;
//     SubAuthority = RtlSubAuthoritySid(SeCreatorOwnerSid, 0);
//     *SubAuthority = SECURITY_CREATOR_OWNER_RID;
//     SubAuthority = RtlSubAuthoritySid(SeCreatorGroupSid, 0);
//     *SubAuthority = SECURITY_CREATOR_GROUP_RID;
//     SubAuthority = RtlSubAuthoritySid(SeCreatorOwnerServerSid, 0);
//     *SubAuthority = SECURITY_CREATOR_OWNER_SERVER_RID;
//     SubAuthority = RtlSubAuthoritySid(SeCreatorGroupServerSid, 0);
//     *SubAuthority = SECURITY_CREATOR_GROUP_SERVER_RID;
//     SubAuthority = RtlSubAuthoritySid(SeDialupSid, 0);
//     *SubAuthority = SECURITY_DIALUP_RID;
//     SubAuthority = RtlSubAuthoritySid(SeNetworkSid, 0);
//     *SubAuthority = SECURITY_NETWORK_RID;
//     SubAuthority = RtlSubAuthoritySid(SeBatchSid, 0);
//     *SubAuthority = SECURITY_BATCH_RID;
//     SubAuthority = RtlSubAuthoritySid(SeInteractiveSid, 0);
//     *SubAuthority = SECURITY_INTERACTIVE_RID;
//     SubAuthority = RtlSubAuthoritySid(SeServiceSid, 0);
//     *SubAuthority = SECURITY_SERVICE_RID;
//     SubAuthority = RtlSubAuthoritySid(SePrincipalSelfSid, 0);
//     *SubAuthority = SECURITY_PRINCIPAL_SELF_RID;
//     SubAuthority = RtlSubAuthoritySid(SeLocalSystemSid, 0);
//     *SubAuthority = SECURITY_LOCAL_SYSTEM_RID;
//     SubAuthority = RtlSubAuthoritySid(SeAuthenticatedUserSid, 0);
//     *SubAuthority = SECURITY_AUTHENTICATED_USER_RID;
//     SubAuthority = RtlSubAuthoritySid(SeRestrictedCodeSid, 0);
//     *SubAuthority = SECURITY_RESTRICTED_CODE_RID;
//     SubAuthority = RtlSubAuthoritySid(SeAliasAdminsSid, 0);
//     *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthority = RtlSubAuthoritySid(SeAliasAdminsSid, 1);
//     *SubAuthority = DOMAIN_ALIAS_RID_ADMINS;
//     SubAuthority = RtlSubAuthoritySid(SeAliasUsersSid, 0);
//     *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthority = RtlSubAuthoritySid(SeAliasUsersSid, 1);
//     *SubAuthority = DOMAIN_ALIAS_RID_USERS;
//     SubAuthority = RtlSubAuthoritySid(SeAliasGuestsSid, 0);
//     *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthority = RtlSubAuthoritySid(SeAliasGuestsSid, 1);
//     *SubAuthority = DOMAIN_ALIAS_RID_GUESTS;
//     SubAuthority = RtlSubAuthoritySid(SeAliasPowerUsersSid, 0);
//     *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthority = RtlSubAuthoritySid(SeAliasPowerUsersSid, 1);
//     *SubAuthority = DOMAIN_ALIAS_RID_POWER_USERS;
//     SubAuthority = RtlSubAuthoritySid(SeAliasAccountOpsSid, 0);
//     *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthority = RtlSubAuthoritySid(SeAliasAccountOpsSid, 1);
//     *SubAuthority = DOMAIN_ALIAS_RID_ACCOUNT_OPS;
//     SubAuthority = RtlSubAuthoritySid(SeAliasSystemOpsSid, 0);
//     *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthority = RtlSubAuthoritySid(SeAliasSystemOpsSid, 1);
//     *SubAuthority = DOMAIN_ALIAS_RID_SYSTEM_OPS;
//     SubAuthority = RtlSubAuthoritySid(SeAliasPrintOpsSid, 0);
//     *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthority = RtlSubAuthoritySid(SeAliasPrintOpsSid, 1);
//     *SubAuthority = DOMAIN_ALIAS_RID_PRINT_OPS;
//     SubAuthority = RtlSubAuthoritySid(SeAliasBackupOpsSid, 0);
//     *SubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthority = RtlSubAuthoritySid(SeAliasBackupOpsSid, 1);
//     *SubAuthority = DOMAIN_ALIAS_RID_BACKUP_OPS;
//     SubAuthority = RtlSubAuthoritySid(SeAuthenticatedUsersSid, 0);
//     *SubAuthority = SECURITY_AUTHENTICATED_USER_RID;
//     SubAuthority = RtlSubAuthoritySid(SeRestrictedSid, 0);
//     *SubAuthority = SECURITY_RESTRICTED_CODE_RID;
//     SubAuthority = RtlSubAuthoritySid(SeAnonymousLogonSid, 0);
//     *SubAuthority = SECURITY_ANONYMOUS_LOGON_RID;
//     SubAuthority = RtlSubAuthoritySid(SeLocalServiceSid, 0);
//     *SubAuthority = SECURITY_LOCAL_SERVICE_RID;
//     SubAuthority = RtlSubAuthoritySid(SeNetworkServiceSid, 0);
//     *SubAuthority = SECURITY_NETWORK_SERVICE_RID;
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Captures a SID.
//  *
//  * @param[in] InputSid
//  * A valid security identifier to be captured.
//  *
//  * @param[in] AccessMode
//  * Processor level access mode.
//  *
//  * @param[in] PoolType
//  * Pool memory type for the captured SID to assign upon
//  * allocation.
//  *
//  * @param[in] CaptureIfKernel
//  * If set to TRUE, the capturing is done within the kernel.
//  * Otherwise the capturing is done in a kernel mode driver.
//  *
//  * @param[out] CapturedSid
//  * The captured security identifier, returned to the caller.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the SID was captured. STATUS_INSUFFICIENT_RESOURCES
//  * is returned if memory pool allocation for the captured SID has failed.
//  */
// NTSTATUS
// NTAPI
// SepCaptureSid(
//     _In_ PSID InputSid,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _In_ POOL_TYPE PoolType,
//     _In_ BOOLEAN CaptureIfKernel,
//     _Out_ PSID *CapturedSid)
// {
//     ULONG SidSize = 0;
//     PISID NewSid, Sid = (PISID)InputSid;
//
//     PAGED_CODE();
//
//     if (AccessMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForRead(Sid, FIELD_OFFSET(SID, SubAuthority), sizeof(UCHAR));
//             SidSize = RtlLengthRequiredSid(Sid->SubAuthorityCount);
//             ProbeForRead(Sid, SidSize, sizeof(UCHAR));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//
//         /* Allocate a SID and copy it */
//         NewSid = ExAllocatePoolWithTag(PoolType, SidSize as SIZE_T, TAG_SID);
//         if (!NewSid)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         _SEH2_TRY
//         {
//             RtlCopyMemory(NewSid, Sid, SidSize);
//
//             *CapturedSid = NewSid;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Free the SID and return the exception code */
//             ExFreePoolWithTag(NewSid, TAG_SID);
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else if (!CaptureIfKernel)
//     {
//         *CapturedSid = InputSid;
//     }
//     else
//     {
//         SidSize = RtlLengthRequiredSid(Sid->SubAuthorityCount);
//
//         /* Allocate a SID and copy it */
//         NewSid = ExAllocatePoolWithTag(PoolType, SidSize as SIZE_T, TAG_SID);
//         if (NewSid == NULL)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         RtlCopyMemory(NewSid, Sid, SidSize);
//
//         *CapturedSid = NewSid;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Releases a captured SID.
//  *
//  * @param[in] CapturedSid
//  * The captured SID to be released.
//  *
//  * @param[in] AccessMode
//  * Processor level access mode.
//  *
//  * @param[in] CaptureIfKernel
//  * If set to TRUE, the releasing is done within the kernel.
//  * Otherwise the releasing is done in a kernel mode driver.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SepReleaseSid(
//     _In_ PSID CapturedSid,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _In_ BOOLEAN CaptureIfKernel)
// {
//     PAGED_CODE();
//
//     if (CapturedSid != NULL &&
//         (AccessMode != KernelMode ||
//          (AccessMode == KernelMode && CaptureIfKernel)))
//     {
//         ExFreePoolWithTag(CapturedSid, TAG_SID);
//     }
// }
//
// /**
//  * @brief
//  * Checks if a SID is present in a token.
//  *
//  * @param[in] _Token
//  * A valid token object.
//  *
//  * @param[in] PrincipalSelfSid
//  * A principal self SID.
//  *
//  * @param[in] _Sid
//  * A regular SID.
//  *
//  * @param[in] Deny
//  * If set to TRUE, the caller expected that a SID in a token
//  * must be a deny-only SID, that is, access checks are performed
//  * only for deny-only ACEs of the said SID.
//  *
//  * @param[in] Restricted
//  * If set to TRUE, the caller expects that a SID in a token is
//  * restricted (by the general definition, a token is restricted).
//  *
//  * @return
//  * Returns TRUE if the specified SID in the call is present in the token,
//  * FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SepSidInTokenEx(
//     _In_ PACCESS_TOKEN _Token,
//     _In_ PSID PrincipalSelfSid,
//     _In_ PSID _Sid,
//     _In_ BOOLEAN Deny,
//     _In_ BOOLEAN Restricted)
// {
//     ULONG SidIndex;
//     PTOKEN Token = (PTOKEN)_Token;
//     PISID TokenSid, Sid = (PISID)_Sid;
//     PSID_AND_ATTRIBUTES SidAndAttributes;
//     ULONG SidCount, SidLength;
//     USHORT SidMetadata;
//     PAGED_CODE();
//
//     /* Check if a principal SID was given, and this is our current SID already */
//     if ((PrincipalSelfSid) && (RtlEqualSid(SePrincipalSelfSid, Sid)))
//     {
//         /* Just use the principal SID in this case */
//         Sid = PrincipalSelfSid;
//     }
//
//     /* Check if this is a restricted token or not */
//     if (Restricted)
//     {
//         /* Use the restricted SIDs and count */
//         SidAndAttributes = Token->RestrictedSids;
//         SidCount = Token->RestrictedSidCount;
//     }
//     else
//     {
//         /* Use the normal SIDs and count */
//         SidAndAttributes = Token->UserAndGroups;
//         SidCount = Token->UserAndGroupCount;
//     }
//
//     /* Do checks here by hand instead of the usual 4 function calls */
//     SidLength = FIELD_OFFSET(SID,
//                              SubAuthority[Sid->SubAuthorityCount]);
//     SidMetadata = *(PUSHORT)&Sid->Revision;
//
//     /* Loop every SID */
//     for (SidIndex = 0; SidIndex < SidCount; SidIndex++)
//     {
//         TokenSid = (PISID)SidAndAttributes->Sid;
// #if SE_SID_DEBUG
//         UNICODE_STRING sidString;
//         RtlConvertSidToUnicodeString(&sidString, TokenSid, TRUE);
//         DPRINT1("SID in Token: %wZ\n", &sidString);
//         RtlFreeUnicodeString(&sidString);
// #endif
//         /* Check if the SID metadata matches */
//         if (*(PUSHORT)&TokenSid->Revision == SidMetadata)
//         {
//             /* Check if the SID data matches */
//             if (RtlEqualMemory(Sid, TokenSid, SidLength))
//             {
//                 /*
//                  * Check if the group is enabled, or used for deny only.
//                  * Otherwise we have to check if this is the first user.
//                  * We understand that by looking if this SID is not
//                  * restricted, this is the first element we are iterating
//                  * and that it doesn't have SE_GROUP_USE_FOR_DENY_ONLY
//                  * attribute.
//                  */
//                 if ((!Restricted && (SidIndex == 0) && !(SidAndAttributes->Attributes & SE_GROUP_USE_FOR_DENY_ONLY)) ||
//                     (SidAndAttributes->Attributes & SE_GROUP_ENABLED) ||
//                     ((Deny) && (SidAndAttributes->Attributes & SE_GROUP_USE_FOR_DENY_ONLY)))
//                 {
//                     /* SID is present */
//                     return TRUE;
//                 }
//                 else
//                 {
//                     /* SID is not present */
//                     return FALSE;
//                 }
//             }
//         }
//
//         /* Move to the next SID */
//         SidAndAttributes++;
//     }
//
//     /* SID is not present */
//     return FALSE;
// }
//
// /**
//  * @brief
//  * Checks if a SID is present in a token.
//  *
//  * @param[in] _Token
//  * A valid token object.
//  *
//  * @param[in] _Sid
//  * A regular SID.
//  *
//  * @return
//  * Returns TRUE if the specified SID in the call is present in the token,
//  * FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SepSidInToken(
//     _In_ PACCESS_TOKEN _Token,
//     _In_ PSID Sid)
// {
//     /* Call extended API */
//     return SepSidInTokenEx(_Token, NULL, Sid, FALSE, FALSE);
// }
//
// /**
//  * @brief
//  * Captures a security identifier from a
//  * given access control entry. This identifier
//  * is valid for the whole of its lifetime.
//  *
//  * @param[in] Ace
//  * A pointer to an access control entry, which
//  * can be obtained from a DACL.
//  *
//  * @return
//  * Returns a pointer to a security identifier (SID),
//  * otherwise NULL is returned if an unsupported ACE
//  * type was given to the function.
//  */
// PSID
// NTAPI
// SepGetSidFromAce(
//     _In_ PACE Ace)
// {
//     PULONG Flags;
//     ULONG GuidSize = 0;
//     PSID Sid = NULL;
//     PAGED_CODE();
//
//     /* Sanity check */
//     ASSERT(Ace);
//
//     /* Obtain the SID based upon ACE type */
//     switch (Ace->Header.AceType)
//     {
//         case ACCESS_DENIED_ACE_TYPE:
//         case ACCESS_ALLOWED_ACE_TYPE:
//         case SYSTEM_AUDIT_ACE_TYPE:
//         case SYSTEM_ALARM_ACE_TYPE:
//         {
//             Sid = (PSID)&((PKNOWN_ACE)Ace)->SidStart;
//             break;
//         }
//
//         case ACCESS_DENIED_OBJECT_ACE_TYPE:
//         case ACCESS_ALLOWED_OBJECT_ACE_TYPE:
//         {
//             Flags = (PULONG)&((PKNOWN_OBJECT_ACE)Ace)->Flags;
//             if (*Flags & ACE_OBJECT_TYPE_PRESENT)
//             {
//                 GuidSize += sizeof(GUID);
//             }
//
//             if (*Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT)
//             {
//                 GuidSize += sizeof(GUID);
//             }
//
//             Sid = (PSID)((ULONG_PTR)&((PKNOWN_OBJECT_ACE)Ace)->SidStart + GuidSize);
//             break;
//         }
//
//         default:
//         {
//             DPRINT1("SepGetSidFromAce(): Unknown ACE type (Ace 0x%p, Type %u)\n", Ace, Ace->Header.AceType);
//             break;
//         }
//     }
//
//     return Sid;
// }
//
// /**
//  * @brief
//  * Captures a SID with attributes.
//  *
//  * @param[in] SrcSidAndAttributes
//  * Source of the SID with attributes to be captured.
//  *
//  * @param[in] AttributeCount
//  * The number count of attributes, in total.
//  *
//  * @param[in] PreviousMode
//  * Processor access level mode.
//  *
//  * @param[in] AllocatedMem
//  * The allocated memory buffer for the captured SID. If the caller
//  * supplies no allocated block of memory then the function will
//  * allocate some buffer block of memory for the captured SID
//  * automatically.
//  *
//  * @param[in] AllocatedLength
//  * The length of the buffer that points to the allocated memory,
//  * in bytes.
//  *
//  * @param[in] PoolType
//  * The pool type for the captured SID and attributes to assign.
//  *
//  * @param[in] CaptureIfKernel
//  * If set to TRUE, the capturing is done within the kernel.
//  * Otherwise the capturing is done in a kernel mode driver.
//  *
//  * @param[out] CapturedSidAndAttributes
//  * The captured SID and attributes.
//  *
//  * @param[out] ResultLength
//  * The length of the captured SID and attributes, in bytes.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if SID and attributes capturing
//  * has been completed successfully. STATUS_INVALID_PARAMETER
//  * is returned if the count of attributes exceeds the maximum
//  * threshold that the kernel can permit, with the purpose of
//  * avoiding integer overflows. STATUS_INSUFFICIENT_RESOURCES
//  * is returned if memory pool allocation for the captured SID has failed.
//  * STATUS_BUFFER_TOO_SMALL is returned if the length of the allocated
//  * buffer is less than the required size. STATUS_INVALID_SID is returned
//  * if a SID doesn't meet certain requirements to be considered a valid
//  * SID by the security manager. A failure NTSTATUS code is returned
//  * otherwise.
//  *
//  * @remarks
//  * A security identifier (SID) is a variable-length data structure that
//  * can change in size over time, depending on the factors that influence
//  * this effect in question. An attacker can take advantage of this fact
//  * and can potentially modify certain properties of a SID making it
//  * different in size than it was originally before. This is what we'd
//  * call, a TOCTOU vulnerability.
//  *
//  * For this reason, the logic of copying the SIDs and their attributes
//  * into a new buffer goes like this: first, allocate a buffer array
//  * that just holds the lengths and subauthority count of each SID.
//  * Such information is copied in the first loop. Then in a second loop,
//  * iterate over the array with SID provided and copy them into the final
//  * array. The moment we're doing this, validate the lengths of each SID
//  * basing upon the captured lengths we've got before. In this way we
//  * ensure that the SIDs have remained intact. The validation checks are
//  * done in user mode as a general rule that we just cannot trust UM and
//  * whatever data is coming from it.
//  */
// NTSTATUS
// NTAPI
// SeCaptureSidAndAttributesArray(
//     _In_ PSID_AND_ATTRIBUTES SrcSidAndAttributes,
//     _In_ ULONG AttributeCount,
//     _In_ KPROCESSOR_MODE PreviousMode,
//     _In_opt_ PVOID AllocatedMem,
//     _In_ ULONG AllocatedLength,
//     _In_ POOL_TYPE PoolType,
//     _In_ BOOLEAN CaptureIfKernel,
//     _Out_ PSID_AND_ATTRIBUTES *CapturedSidAndAttributes,
//     _Out_ PULONG ResultLength)
// {
//     ULONG ArraySize, RequiredLength, SidLength, i;
//     ULONG TempArrayValidate, TempLengthValidate;
//     PSID_AND_ATTRIBUTES SidAndAttributes;
//     _SEH2_VOLATILE PSID_VALIDATE ValidateArray;
//     PUCHAR CurrentDest;
//     PISID Sid;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     ValidateArray = NULL;
//     SidAndAttributes = NULL;
//     *CapturedSidAndAttributes = NULL;
//     *ResultLength = 0;
//
//     if (AttributeCount == 0)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     if (AttributeCount > SE_MAXIMUM_GROUP_LIMIT)
//     {
//         DPRINT1("SeCaptureSidAndAttributesArray(): Maximum group limit exceeded!\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if ((PreviousMode == KernelMode) && !CaptureIfKernel)
//     {
//         *CapturedSidAndAttributes = SrcSidAndAttributes;
//         return STATUS_SUCCESS;
//     }
//
//     ArraySize = AttributeCount * sizeof(SID_AND_ATTRIBUTES);
//     RequiredLength = ALIGN_UP_BY(ArraySize, sizeof(ULONG));
//
//     if (PreviousMode != KernelMode)
//     {
//         /* Check for user mode data */
//         _SEH2_TRY
//         {
//             /* First probe the whole array */
//             ProbeForRead(SrcSidAndAttributes, ArraySize, sizeof(ULONG));
//
//             /* We're in user mode, set up the size for the temporary array */
//             TempArrayValidate = AttributeCount * sizeof(SID_VALIDATE);
//             TempLengthValidate = ALIGN_UP_BY(TempArrayValidate, sizeof(ULONG));
//
//             /*
//              * Allocate a buffer for the array that we're going to
//              * temporarily hold the subauthority count and the SID
//              * elements. We'll be going to use this array to perform
//              * validation checks later.
//              */
//             ValidateArray = ExAllocatePoolWithTag(PoolType | POOL_RAISE_IF_ALLOCATION_FAILURE,
//                                                   TempLengthValidate,
//                                                   TAG_SID_VALIDATE);
//
//             /* Loop the array elements */
//             for (i = 0; i < AttributeCount; i++)
//             {
//                 /* Get the SID and probe the minimal structure */
//                 Sid = SrcSidAndAttributes[i].Sid;
//                 ProbeForRead(Sid, sizeof(*Sid), sizeof(ULONG));
//
//                 /*
//                  * Capture the subauthority count and hold it
//                  * into the temporary array for later validation.
//                  * This way we ensure that the said count of each
//                  * SID has remained the same.
//                  */
//                 ValidateArray[i].SubAuthorityCount = Sid->SubAuthorityCount;
//
//                 /* Capture the SID */
//                 ValidateArray[i].ProbeSid = Sid;
//
//                 /* Calculate the SID length and probe the full SID */
//                 SidLength = RtlLengthRequiredSid(ValidateArray[i].SubAuthorityCount);
//                 ProbeForRead(ValidateArray[i].ProbeSid, SidLength, sizeof(ULONG));
//
//                 /* Add the aligned length to the required length */
//                 RequiredLength += ALIGN_UP_BY(SidLength, sizeof(ULONG));
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = _SEH2_GetExceptionCode();
//             _SEH2_YIELD(goto Cleanup);
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* Loop the array elements */
//         for (i = 0; i < AttributeCount; i++)
//         {
//             /* Get the SID and it's length */
//             Sid = SrcSidAndAttributes[i].Sid;
//             SidLength = RtlLengthRequiredSid(Sid->SubAuthorityCount);
//
//             /* Add the aligned length to the required length */
//             RequiredLength += ALIGN_UP_BY(SidLength, sizeof(ULONG));
//         }
//     }
//
//     /* Assume success */
//     Status = STATUS_SUCCESS;
//     *ResultLength = RequiredLength;
//
//     /* Check if we have no buffer */
//     if (AllocatedMem == NULL)
//     {
//         /* Allocate a new buffer */
//         SidAndAttributes = ExAllocatePoolWithTag(PoolType,
//                                                  RequiredLength,
//                                                  TAG_SID_AND_ATTRIBUTES);
//         if (SidAndAttributes == NULL)
//         {
//             DPRINT1("SeCaptureSidAndAttributesArray(): Failed to allocate memory for SID and attributes array (requested size -> %lu)!\n", RequiredLength);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//     }
//     /* Otherwise check if the buffer is large enough */
//     else if (AllocatedLength >= RequiredLength)
//     {
//         /* Buffer is large enough, use it */
//         SidAndAttributes = AllocatedMem;
//     }
//     else
//     {
//         /* Buffer is too small, fail */
//         DPRINT1("SeCaptureSidAndAttributesArray(): The provided buffer is small (expected size -> %lu || current size -> %lu)!\n", RequiredLength, AllocatedLength);
//         Status = STATUS_BUFFER_TOO_SMALL;
//         goto Cleanup;
//     }
//
//     *CapturedSidAndAttributes = SidAndAttributes;
//
//     /* Check again for user mode */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* The rest of the data starts after the array */
//             CurrentDest = (PUCHAR)SidAndAttributes;
//             CurrentDest += ALIGN_UP_BY(ArraySize, sizeof(ULONG));
//
//             /* Loop the array elements */
//             for (i = 0; i < AttributeCount; i++)
//             {
//                 /*
//                  * Get the SID length from the subauthority
//                  * count we've captured before.
//                  */
//                 SidLength = RtlLengthRequiredSid(ValidateArray[i].SubAuthorityCount);
//
//                 /* Copy attributes */
//                 SidAndAttributes[i].Attributes = SrcSidAndAttributes[i].Attributes;
//
//                 /* Copy the SID to the current destination address */
//                 SidAndAttributes[i].Sid = (PSID)CurrentDest;
//                 RtlCopyMemory(CurrentDest, ValidateArray[i].ProbeSid, SidLength);
//
//                 /* Obtain the SID we've captured before for validation */
//                 Sid = SidAndAttributes[i].Sid;
//
//                 /* Validate that the subauthority count hasn't changed */
//                 if (ValidateArray[i].SubAuthorityCount !=
//                     Sid->SubAuthorityCount)
//                 {
//                     /* It's changed, bail out */
//                     DPRINT1("SeCaptureSidAndAttributesArray(): The subauthority counts have changed (captured count -> %u || current count -> %u)\n",
//                             ValidateArray[i].SubAuthorityCount, Sid->SubAuthorityCount);
//                     Status = STATUS_INVALID_SID;
//                     goto Cleanup;
//                 }
//
//                 /* Validate that the SID length is the same */
//                 if (SidLength != RtlLengthSid(Sid))
//                 {
//                     /* They're no longer the same, bail out */
//                     DPRINT1("SeCaptureSidAndAttributesArray(): The SID lengths have changed (captured length -> %lu || current length -> %lu)\n",
//                             SidLength, RtlLengthSid(Sid));
//                     Status = STATUS_INVALID_SID;
//                     goto Cleanup;
//                 }
//
//                 /* Check that the SID is valid */
//                 if (!RtlValidSid(Sid))
//                 {
//                     DPRINT1("SeCaptureSidAndAttributesArray(): The SID is not valid!\n");
//                     Status = STATUS_INVALID_SID;
//                     goto Cleanup;
//                 }
//
//                 /* Update the current destination address */
//                 CurrentDest += ALIGN_UP_BY(SidLength, sizeof(ULONG));
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* The rest of the data starts after the array */
//         CurrentDest = (PUCHAR)SidAndAttributes;
//         CurrentDest += ALIGN_UP_BY(ArraySize, sizeof(ULONG));
//
//         /* Loop the array elements */
//         for (i = 0; i < AttributeCount; i++)
//         {
//             /* Get the SID and it's length */
//             Sid = SrcSidAndAttributes[i].Sid;
//             SidLength = RtlLengthRequiredSid(Sid->SubAuthorityCount);
//
//             /* Copy attributes */
//             SidAndAttributes[i].Attributes = SrcSidAndAttributes[i].Attributes;
//
//             /* Copy the SID to the current destination address */
//             SidAndAttributes[i].Sid = (PSID)CurrentDest;
//             RtlCopyMemory(CurrentDest, SrcSidAndAttributes[i].Sid, SidLength);
//
//             /* Update the current destination address */
//             CurrentDest += ALIGN_UP_BY(SidLength, sizeof(ULONG));
//         }
//     }
//
// Cleanup:
//     /* Check for failure */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Check if we allocated a new array */
//         if ((SidAndAttributes != AllocatedMem) && (SidAndAttributes != NULL))
//         {
//             /* Free the array */
//             ExFreePoolWithTag(SidAndAttributes, TAG_SID_AND_ATTRIBUTES);
//         }
//
//         /* Set returned address to NULL */
//         *CapturedSidAndAttributes = NULL;
//     }
//
//     /* Free the temporary validation array */
//     if ((PreviousMode != KernelMode) && (ValidateArray != NULL))
//     {
//         ExFreePoolWithTag(ValidateArray, TAG_SID_VALIDATE);
//     }
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Releases a captured SID with attributes.
//  *
//  * @param[in] CapturedSidAndAttributes
//  * The captured SID with attributes to be released.
//  *
//  * @param[in] AccessMode
//  * Processor access level mode.
//  *
//  * @param[in] CaptureIfKernel
//  * If set to TRUE, the releasing is done within the kernel.
//  * Otherwise the releasing is done in a kernel mode driver.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeReleaseSidAndAttributesArray(
//     _In_ _Post_invalid_ PSID_AND_ATTRIBUTES CapturedSidAndAttributes,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _In_ BOOLEAN CaptureIfKernel)
// {
//     PAGED_CODE();
//
//     if ((CapturedSidAndAttributes != NULL) &&
//         ((AccessMode != KernelMode) || CaptureIfKernel))
//     {
//         ExFreePoolWithTag(CapturedSidAndAttributes, TAG_SID_AND_ATTRIBUTES);
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
