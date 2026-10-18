use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, NT_SUCCESS, TRUE, FALSE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::PagedPool;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[inline]
unsafe fn RtlInterlockedSetBits(Flags: *mut ULONG, Flag: ULONG) -> ULONG {
    use core::sync::atomic::{AtomicU32, Ordering};
    let atomic = &*(Flags as *const AtomicU32);
    atomic.fetch_or(Flag, Ordering::SeqCst)
}

extern "system" {
    fn SeQuerySessionIdToken(token: PTOKEN, session_id: PULONG) -> NTSTATUS;
    fn SepCaptureSid(
        input_sid: PSID,
        previous_mode: KPROCESSOR_MODE,
        pool_type: ULONG,
        capture_if_kernel: BOOLEAN,
        captured_sid: *mut PSID,
    ) -> NTSTATUS;
    fn SepReleaseSid(
        captured_sid: PSID,
        previous_mode: KPROCESSOR_MODE,
        capture_if_kernel: BOOLEAN,
    );
    fn SepCaptureAcl(
        input_acl: PACL,
        previous_mode: KPROCESSOR_MODE,
        pool_type: ULONG,
        capture_if_kernel: BOOLEAN,
        captured_acl: *mut PACL,
    ) -> NTSTATUS;
    fn SepReleaseAcl(
        captured_acl: PACL,
        previous_mode: KPROCESSOR_MODE,
        capture_if_kernel: BOOLEAN,
    );
    fn SeTokenIsRestricted(token: PTOKEN) -> BOOLEAN;
    fn SeTokenIsInert(token: PTOKEN) -> u8;
    fn SeSinglePrivilegeCheck(privilege_value: LUID, previous_mode: KPROCESSOR_MODE) -> BOOLEAN;
    fn SepRmRemoveLogonSessionFromToken(token: PTOKEN) -> NTSTATUS;
    
    static mut SeTokenObjectType: *mut _OBJECT_TYPE;
    static mut SeTcbPrivilege: LUID;
}
extern "C" {
    fn DefaultQueryInfoBufferCheck(
        info_class: ULONG,
        info_class_table: *const INFORMATION_CLASS_INFO,
        table_entries: ULONG,
        flags: ULONG,
        buffer: PVOID,
        buffer_length: ULONG,
        return_length: PULONG,
        previous_mode_ptr: *const KPROCESSOR_MODE,
        previous_mode: KPROCESSOR_MODE,
    ) -> NTSTATUS;
    
    fn DefaultSetInfoBufferCheck(
        info_class: ULONG,
        info_class_table: *const INFORMATION_CLASS_INFO,
        table_entries: ULONG,
        buffer: PVOID,
        buffer_length: ULONG,
        previous_mode: KPROCESSOR_MODE,
    ) -> NTSTATUS;
    
    fn SepAcquireTokenLockShared(token: PTOKEN);
    fn SepAcquireTokenLockExclusive(token: PTOKEN);
    fn SepReleaseTokenLock(token: PTOKEN);
}
extern "C" {
    fn SepComputeAvailableDynamicSpace(
        dynamic_charged: ULONG,
        primary_group: PSID,
        default_dacl: PACL,
    ) -> ULONG;
    fn SepFindPrimaryGroupAndDefaultOwner(
        token: PTOKEN,
        primary_group: PSID,
        default_owner: PSID,
        primary_group_index: PULONG,
        default_owner_index: PULONG,
    ) -> NTSTATUS;
    fn SepRebuildDynamicPartOfToken(token: PTOKEN, new_dynamic_length: ULONG) -> NTSTATUS;
    fn SepRmDereferenceLogonSession(logon_id: *const LUID) -> u32;
}

const TAG_SE: ULONG = 0x20206553;

// The translation had ICIF_SIZE_VARIABLE = 0x08 (SET only), so TokenOwner,
// TokenPrimaryGroup and TokenDefaultDacl refused any query length but
// sizeof(TOKEN_OWNER) with STATUS_INFO_LENGTH_MISMATCH, breaking the two-call
// GetTokenInformation size probe.
use ntkrnl_sys::inline::common::{
    ICIF_FORCE_RETURN_LENGTH_PROBE, ICIF_PROBE_READ_WRITE, ICIF_QUERY, ICIF_QUERY_SIZE_VARIABLE,
    ICIF_SET, ICIF_SET_SIZE_VARIABLE, ICIF_SIZE_VARIABLE,
};

const IQS_NONE: INFORMATION_CLASS_INFO = INFORMATION_CLASS_INFO {
    RequiredSizeQUERY: 0,
    AlignmentQUERY: 0,
    RequiredSizeSET: 0,
    AlignmentSET: 0,
    Flags: 0,
};

static SE_TOKEN_INFORMATION_CLASS: [INFORMATION_CLASS_INFO; 18] = [
    IQS_NONE,

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_USER>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_USER>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_GROUPS>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_GROUPS>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_PRIVILEGES>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_PRIVILEGES>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_OWNER>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_OWNER>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_SET | ICIF_SIZE_VARIABLE) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_PRIMARY_GROUP>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_PRIMARY_GROUP>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_SET | ICIF_SIZE_VARIABLE) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_DEFAULT_DACL>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_DEFAULT_DACL>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_SET | ICIF_SIZE_VARIABLE) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_SOURCE>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_SOURCE>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_TYPE>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_TYPE>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<SECURITY_IMPERSONATION_LEVEL>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<SECURITY_IMPERSONATION_LEVEL>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: ICIF_QUERY as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_STATISTICS>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_STATISTICS>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_GROUPS>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_GROUPS>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<ULONG>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<ULONG>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_SET) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_GROUPS_AND_PRIVILEGES>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_GROUPS_AND_PRIVILEGES>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<ULONG>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<ULONG>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: ICIF_SET as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<ULONG>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<ULONG>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: ICIF_QUERY as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_AUDIT_POLICY_INFORMATION>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_AUDIT_POLICY_INFORMATION>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_SET | ICIF_SET_SIZE_VARIABLE) as USHORT,
    },

    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TOKEN_ORIGIN>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: core::mem::size_of::<TOKEN_ORIGIN>() as USHORT,
        AlignmentSET: core::mem::align_of::<ULONG>() as UCHAR,
        Flags: (ICIF_QUERY | ICIF_SET) as USHORT,
    },
];

const TOKEN_QUERY: ULONG = 0x0008;
const TOKEN_QUERY_SOURCE: ULONG = 0x0010;
const TOKEN_ADJUST_DEFAULT: ULONG = 0x0080;
const TOKEN_ADJUST_SESSIONID: ULONG = 0x0100;

const TOKEN_IMPERSONATION: ULONG = 2;

const TOKEN_SESSION_NOT_REFERENCED: ULONG = 0x20;

#[derive(Clone, Copy)]
#[repr(C)]
struct SEP_AUDIT_POLICY_ELEMENTS {
    System: UCHAR,
    Logon: UCHAR,
    ObjectAccess: UCHAR,
    PrivilegeUse: UCHAR,
    DetailedTracking: UCHAR,
    PolicyChange: UCHAR,
    AccountManagement: UCHAR,
    DirectoryServiceAccess: UCHAR,
    AccountLogon: UCHAR,
}

#[repr(C)]
union SEP_AUDIT_POLICY {
    PolicyElements: SEP_AUDIT_POLICY_ELEMENTS,
    PolicyBits: u64,
}

const AUDIT_CATEGORY_SYSTEM: ULONG = 0;
const AUDIT_CATEGORY_LOGON: ULONG = 1;
const AUDIT_CATEGORY_OBJECT_ACCESS: ULONG = 2;
const AUDIT_CATEGORY_PRIVILEGE_USE: ULONG = 3;
const AUDIT_CATEGORY_DETAILED_TRACKING: ULONG = 4;
const AUDIT_CATEGORY_POLICY_CHANGE: ULONG = 5;
const AUDIT_CATEGORY_ACCOUNT_MANAGEMENT: ULONG = 6;
const AUDIT_CATEGORY_DIRECTORY_SERVICE_ACCESS: ULONG = 7;
const AUDIT_CATEGORY_ACCOUNT_LOGON: ULONG = 8;

#[no_mangle]
pub unsafe extern "system" fn SeQueryInformationToken(
    AccessToken: PACCESS_TOKEN,
    TokenInformationClass: TOKEN_INFORMATION_CLASS,
    TokenInformation: *mut PVOID,
) -> NTSTATUS {
    let TokenInformation = opaque_ptr(TokenInformation);
    let mut status: NTSTATUS;
    let token = AccessToken as PTOKEN;
    let mut required_length: ULONG;
    let mut unused_sid: PSID = core::ptr::null_mut();
    let mut unused_ulong: ULONG = 0;

    SepAcquireTokenLockShared(token);

    match TokenInformationClass as ULONG {
        1 => {
            required_length = core::mem::size_of::<TOKEN_USER>() as ULONG
                + RtlLengthSid((*(*token).UserAndGroups.offset(0)).Sid);

            let tu = ExAllocatePoolWithTag(PagedPool, required_length as SIZE_T, TAG_SE) as PTOKEN_USER;
            if tu.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                status = RtlCopySidAndAttributesArray(
                    1,
                    (*token).UserAndGroups as *mut SID_AND_ATTRIBUTES,
                    required_length - core::mem::size_of::<TOKEN_USER>() as ULONG,
                    core::ptr::addr_of_mut!((*tu).User),
                    (tu as *mut u8).offset(core::mem::size_of::<TOKEN_USER>() as isize) as PSID,
                    core::ptr::addr_of_mut!(unused_sid),
                    core::ptr::addr_of_mut!(unused_ulong),
                );

                *TokenInformation = tu as PVOID;
                status = STATUS_SUCCESS;
            }
        }

        2 => {
            required_length = core::mem::size_of::<ULONG>() as ULONG
                + RtlLengthSidAndAttributes(
                    (*token).UserAndGroupCount - 1,
                    ((*token).UserAndGroups as *mut SID_AND_ATTRIBUTES).offset(1),
                );

            let sid_len = required_length - core::mem::size_of::<ULONG>() as ULONG
                - (((*token).UserAndGroupCount - 1) * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG);

            let tg = ExAllocatePoolWithTag(PagedPool, required_length as SIZE_T, TAG_SE) as PTOKEN_GROUPS;
            if tg.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                let sid = ((tg as ULONG_PTR) + core::mem::size_of::<ULONG>() as ULONG_PTR
                    + (((*token).UserAndGroupCount - 1) * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG) as ULONG_PTR) as PSID;

                (*tg).GroupCount = (*token).UserAndGroupCount - 1;
                status = RtlCopySidAndAttributesArray(
                    (*token).UserAndGroupCount - 1,
                    ((*token).UserAndGroups as *mut SID_AND_ATTRIBUTES).offset(1),
                    sid_len,
                    core::ptr::addr_of_mut!((*tg).Groups) as *mut SID_AND_ATTRIBUTES,
                    sid,
                    core::ptr::addr_of_mut!(unused_sid),
                    core::ptr::addr_of_mut!(unused_ulong),
                );

                *TokenInformation = tg as PVOID;
                status = STATUS_SUCCESS;
            }
        }

        3 => {
            required_length = core::mem::size_of::<ULONG>() as ULONG
                + ((*token).PrivilegeCount * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG);

            let tp = ExAllocatePoolWithTag(PagedPool, required_length as SIZE_T, TAG_SE) as PTOKEN_PRIVILEGES;
            if tp.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                (*tp).PrivilegeCount = (*token).PrivilegeCount;
                RtlCopyLuidAndAttributesArray(
                    (*token).PrivilegeCount,
                    (*token).Privileges,
                    core::ptr::addr_of_mut!((*tp).Privileges) as PLUID_AND_ATTRIBUTES,
                );

                *TokenInformation = tp as PVOID;
                status = STATUS_SUCCESS;
            }
        }

        4 => {
            let sid_len = RtlLengthSid((*(*token).UserAndGroups.offset((*token).DefaultOwnerIndex as isize)).Sid);
            required_length = core::mem::size_of::<TOKEN_OWNER>() as ULONG + sid_len;

            let to = ExAllocatePoolWithTag(PagedPool, required_length as SIZE_T, TAG_SE) as PTOKEN_OWNER;
            if to.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                (*to).Owner = (to as *mut u8).offset(core::mem::size_of::<TOKEN_OWNER>() as isize) as PSID;
                status = RtlCopySid(
                    sid_len,
                    (*to).Owner,
                    (*(*token).UserAndGroups.offset((*token).DefaultOwnerIndex as isize)).Sid,
                );

                *TokenInformation = to as PVOID;
                status = STATUS_SUCCESS;
            }
        }

        5 => {
            let sid_len = RtlLengthSid((*token).PrimaryGroup);
            required_length = core::mem::size_of::<TOKEN_PRIMARY_GROUP>() as ULONG + sid_len;

            let tpg = ExAllocatePoolWithTag(PagedPool, required_length as SIZE_T, TAG_SE) as PTOKEN_PRIMARY_GROUP;
            if tpg.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                (*tpg).PrimaryGroup = (tpg as *mut u8).offset(core::mem::size_of::<TOKEN_PRIMARY_GROUP>() as isize) as PSID;
                status = RtlCopySid(sid_len, (*tpg).PrimaryGroup, (*token).PrimaryGroup);

                *TokenInformation = tpg as PVOID;
                status = STATUS_SUCCESS;
            }
        }

        6 => {
            required_length = core::mem::size_of::<TOKEN_DEFAULT_DACL>() as ULONG;

            if !(*token).DefaultDacl.is_null() {
                required_length += (*(*token).DefaultDacl).AclSize as ULONG;
            }

            let tdd = ExAllocatePoolWithTag(PagedPool, required_length as SIZE_T, TAG_SE) as PTOKEN_DEFAULT_DACL;
            if tdd.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                if !(*token).DefaultDacl.is_null() {
                    (*tdd).DefaultDacl = (tdd as *mut u8).offset(core::mem::size_of::<TOKEN_DEFAULT_DACL>() as isize) as PACL;
                    core::ptr::copy_nonoverlapping(
                        (*token).DefaultDacl as *const u8,
                        (*tdd).DefaultDacl as *mut u8,
                        (*(*token).DefaultDacl).AclSize as usize,
                    );
                } else {
                    (*tdd).DefaultDacl = core::ptr::null_mut();
                }

                *TokenInformation = tdd as PVOID;
                status = STATUS_SUCCESS;
            }
        }

        7 => {
            required_length = core::mem::size_of::<TOKEN_SOURCE>() as ULONG;

            let ts = ExAllocatePoolWithTag(PagedPool, required_length as SIZE_T, TAG_SE) as PTOKEN_SOURCE;
            if ts.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                *ts = (*token).TokenSource;

                *TokenInformation = ts as PVOID;
                status = STATUS_SUCCESS;
            }
        }

        8 => {
            required_length = core::mem::size_of::<TOKEN_TYPE>() as ULONG;

            let tt = ExAllocatePoolWithTag(PagedPool, required_length as SIZE_T, TAG_SE) as PTOKEN_TYPE;
            if tt.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                *tt = (*token).TokenType;

                *TokenInformation = tt as PVOID;
                status = STATUS_SUCCESS;
            }
        }

        9 => {
            required_length = core::mem::size_of::<SECURITY_IMPERSONATION_LEVEL>() as ULONG;

            if (*token).TokenType != TOKEN_IMPERSONATION {
                status = STATUS_INVALID_INFO_CLASS;
            } else {
                let sil = ExAllocatePoolWithTag(PagedPool, required_length as SIZE_T, TAG_SE) as PSECURITY_IMPERSONATION_LEVEL;
                if sil.is_null() {
                    status = STATUS_INSUFFICIENT_RESOURCES;
                } else {
                    *sil = (*token).ImpersonationLevel;

                    *TokenInformation = sil as PVOID;
                    status = STATUS_SUCCESS;
                }
            }
        }

        10 => {
            required_length = core::mem::size_of::<TOKEN_STATISTICS>() as ULONG;

            let ts = ExAllocatePoolWithTag(PagedPool, required_length as SIZE_T, TAG_SE) as PTOKEN_STATISTICS;
            if ts.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                (*ts).TokenId = (*token).TokenId;
                (*ts).AuthenticationId = (*token).AuthenticationId;
                (*ts).ExpirationTime = (*token).ExpirationTime;
                (*ts).TokenType = (*token).TokenType;
                (*ts).ImpersonationLevel = (*token).ImpersonationLevel;
                (*ts).DynamicCharged = (*token).DynamicCharged;
                (*ts).DynamicAvailable = SepComputeAvailableDynamicSpace(
                    (*token).DynamicCharged,
                    (*token).PrimaryGroup,
                    (*token).DefaultDacl,
                );
                (*ts).GroupCount = (*token).UserAndGroupCount - 1;
                (*ts).PrivilegeCount = (*token).PrivilegeCount;
                (*ts).ModifiedId = (*token).ModifiedId;

                *TokenInformation = ts as PVOID;
                status = STATUS_SUCCESS;
            }
        }

        12 => {
            status = SeQuerySessionIdToken(token, TokenInformation as PULONG);
        }

        _ => {
            status = STATUS_INVALID_INFO_CLASS;
        }
    }

    SepReleaseTokenLock(token);

    status
}

/// The token classes that return SIDs or ACLs store pointers to them, and those point
/// INTO the output buffer (`TOKEN_USER.User.Sid`, `TOKEN_GROUPS.Groups[i].Sid`, ...),
/// so a bounced query moves them to the caller's buffer before it is copied out.
unsafe fn rebase_token_information(bounce: &OutputBounce, class: ULONG) {
    let info = bounce.buffer();
    let rebase_groups = |groups: PSID_AND_ATTRIBUTES, count: ULONG| {
        for i in 0..count as usize {
            let entry = groups.add(i);
            if !bounce.contains(entry as *const _, core::mem::size_of::<SID_AND_ATTRIBUTES>()) {
                break;
            }
            bounce.rebase(core::ptr::addr_of_mut!((*entry).Sid));
        }
    };
    let fits = |size: usize| bounce.contains(info, size);
    match class {
        // TokenUser
        1 if fits(core::mem::size_of::<TOKEN_USER>()) => {
            bounce.rebase(core::ptr::addr_of_mut!((*(info as PTOKEN_USER)).User.Sid));
        }
        // TokenGroups, TokenRestrictedSids
        2 | 11 if fits(core::mem::size_of::<TOKEN_GROUPS>()) => {
            let tg = info as PTOKEN_GROUPS;
            rebase_groups(core::ptr::addr_of_mut!((*tg).Groups) as PSID_AND_ATTRIBUTES, (*tg).GroupCount);
        }
        // TokenOwner
        4 if fits(core::mem::size_of::<TOKEN_OWNER>()) => {
            bounce.rebase(core::ptr::addr_of_mut!((*(info as PTOKEN_OWNER)).Owner));
        }
        // TokenPrimaryGroup
        5 if fits(core::mem::size_of::<TOKEN_PRIMARY_GROUP>()) => {
            bounce.rebase(core::ptr::addr_of_mut!((*(info as PTOKEN_PRIMARY_GROUP)).PrimaryGroup));
        }
        // TokenDefaultDacl
        6 if fits(core::mem::size_of::<TOKEN_DEFAULT_DACL>()) => {
            bounce.rebase(core::ptr::addr_of_mut!((*(info as PTOKEN_DEFAULT_DACL)).DefaultDacl));
        }
        // TokenGroupsAndPrivileges: three arrays in the buffer, two of them of SIDs.
        // The entries are rebased before the array pointers that locate them.
        13 if fits(core::mem::size_of::<TOKEN_GROUPS_AND_PRIVILEGES>()) => {
            let gp = info as PTOKEN_GROUPS_AND_PRIVILEGES;
            rebase_groups((*gp).Sids, (*gp).SidCount);
            rebase_groups((*gp).RestrictedSids, (*gp).RestrictedSidCount);
            bounce.rebase(core::ptr::addr_of_mut!((*gp).Sids));
            bounce.rebase(core::ptr::addr_of_mut!((*gp).RestrictedSids));
            bounce.rebase(core::ptr::addr_of_mut!((*gp).Privileges));
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryInformationToken(
    TokenHandle: HANDLE,
    TokenInformationClass: TOKEN_INFORMATION_CLASS,
    TokenInformation: PVOID,
    TokenInformationLength: ULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    let TokenHandle = opaque_ptr(TokenHandle);
    let TokenInformation = opaque_ptr(TokenInformation);
    let ReturnLength = opaque_ptr(ReturnLength);
    let mut status: NTSTATUS;
    let previous_mode: KPROCESSOR_MODE;
    let mut token: PTOKEN = core::ptr::null_mut();
    let mut required_length: ULONG = 0;
    let mut unused_sid: PSID = core::ptr::null_mut();
    let mut unused_ulong: ULONG = 0;

    previous_mode = ExGetPreviousMode();

    status = DefaultQueryInfoBufferCheck(
        TokenInformationClass as ULONG,
        SE_TOKEN_INFORMATION_CLASS.as_ptr(),
        SE_TOKEN_INFORMATION_CLASS.len() as ULONG,
        ICIF_PROBE_READ_WRITE | ICIF_FORCE_RETURN_LENGTH_PROBE,
        TokenInformation,
        TokenInformationLength,
        ReturnLength,
        core::ptr::null(),
        previous_mode,
    );
    if status < 0 {
        return status;
    }

    // The C fills TokenInformation in place under _SEH2_TRY, much of it while holding
    // the token lock. A user-mode caller's buffer is replaced by a pool copy here, and
    // copied back with the fault-tolerant copy.
    let bounce = match OutputBounce::new(TokenInformation, TokenInformationLength as usize, previous_mode) {
        Ok(b) => b,
        Err(s) => return s,
    };
    let TokenInformation = bounce.buffer();
    let info_mode = bounce.mode(previous_mode);

    status = ObReferenceObjectByHandle(
        TokenHandle,
        if TokenInformationClass as ULONG == 7 { TOKEN_QUERY_SOURCE } else { TOKEN_QUERY },
        SeTokenObjectType as *mut _OBJECT_TYPE,
        previous_mode,
        core::ptr::addr_of_mut!(token) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status >= 0 {
        SepAcquireTokenLockShared(token);

        match TokenInformationClass as ULONG {
            1 => {
                let tu = TokenInformation as PTOKEN_USER;

                required_length = core::mem::size_of::<TOKEN_USER>() as ULONG
                    + RtlLengthSid((*(*token).UserAndGroups.offset(0)).Sid);

                if TokenInformationLength >= required_length {
                    status = RtlCopySidAndAttributesArray(
                        1,
                        (*token).UserAndGroups as *mut SID_AND_ATTRIBUTES,
                        required_length - core::mem::size_of::<TOKEN_USER>() as ULONG,
                        core::ptr::addr_of_mut!((*tu).User),
                        (tu as *mut u8).offset(core::mem::size_of::<TOKEN_USER>() as isize) as PSID,
                        core::ptr::addr_of_mut!(unused_sid),
                        core::ptr::addr_of_mut!(unused_ulong),
                    );
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }

                // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            2 => {
                let tg = TokenInformation as PTOKEN_GROUPS;

                required_length = core::mem::size_of::<ULONG>() as ULONG
                    + RtlLengthSidAndAttributes(
                        (*token).UserAndGroupCount - 1,
                        ((*token).UserAndGroups as *mut SID_AND_ATTRIBUTES).offset(1),
                    );

                if TokenInformationLength >= required_length {
                    let sid_len = required_length - core::mem::size_of::<ULONG>() as ULONG
                        - (((*token).UserAndGroupCount - 1) * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG);
                    let sid = ((tg as ULONG_PTR) + core::mem::size_of::<ULONG>() as ULONG_PTR
                        + (((*token).UserAndGroupCount - 1) * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG) as ULONG_PTR) as PSID;

                    (*tg).GroupCount = (*token).UserAndGroupCount - 1;
                    status = RtlCopySidAndAttributesArray(
                        (*token).UserAndGroupCount - 1,
                        ((*token).UserAndGroups as *mut SID_AND_ATTRIBUTES).offset(1),
                        sid_len,
                        core::ptr::addr_of_mut!((*tg).Groups) as *mut SID_AND_ATTRIBUTES,
                        sid,
                        core::ptr::addr_of_mut!(unused_sid),
                        core::ptr::addr_of_mut!(unused_ulong),
                    );
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }

                // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            3 => {
                let tp = TokenInformation as PTOKEN_PRIVILEGES;

                required_length = core::mem::size_of::<ULONG>() as ULONG
                    + ((*token).PrivilegeCount * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG);

                if TokenInformationLength >= required_length {
                    (*tp).PrivilegeCount = (*token).PrivilegeCount;
                    RtlCopyLuidAndAttributesArray(
                        (*token).PrivilegeCount,
                        (*token).Privileges,
                        core::ptr::addr_of_mut!((*tp).Privileges) as PLUID_AND_ATTRIBUTES,
                    );
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }

                // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            4 => {
                let to = TokenInformation as PTOKEN_OWNER;
                let sid_len = RtlLengthSid((*(*token).UserAndGroups.offset((*token).DefaultOwnerIndex as isize)).Sid);
                required_length = core::mem::size_of::<TOKEN_OWNER>() as ULONG + sid_len;

                if TokenInformationLength >= required_length {
                    (*to).Owner = (to as *mut u8).offset(core::mem::size_of::<TOKEN_OWNER>() as isize) as PSID;
                    status = RtlCopySid(
                        sid_len,
                        (*to).Owner,
                        (*(*token).UserAndGroups.offset((*token).DefaultOwnerIndex as isize)).Sid,
                    );
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }

                // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            5 => {
                let tpg = TokenInformation as PTOKEN_PRIMARY_GROUP;
                let sid_len = RtlLengthSid((*token).PrimaryGroup);
                required_length = core::mem::size_of::<TOKEN_PRIMARY_GROUP>() as ULONG + sid_len;

                if TokenInformationLength >= required_length {
                    (*tpg).PrimaryGroup = (tpg as *mut u8).offset(core::mem::size_of::<TOKEN_PRIMARY_GROUP>() as isize) as PSID;
                    status = RtlCopySid(sid_len, (*tpg).PrimaryGroup, (*token).PrimaryGroup);
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }

                // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            6 => {
                let tdd = TokenInformation as PTOKEN_DEFAULT_DACL;

                required_length = core::mem::size_of::<TOKEN_DEFAULT_DACL>() as ULONG;

                if !(*token).DefaultDacl.is_null() {
                    required_length += (*(*token).DefaultDacl).AclSize as ULONG;
                }

                if TokenInformationLength >= required_length {
                    if !(*token).DefaultDacl.is_null() {
                        (*tdd).DefaultDacl = (tdd as *mut u8).offset(core::mem::size_of::<TOKEN_DEFAULT_DACL>() as isize) as PACL;
                        core::ptr::copy_nonoverlapping(
                            (*token).DefaultDacl as *const u8,
                            (*tdd).DefaultDacl as *mut u8,
                            (*(*token).DefaultDacl).AclSize as usize,
                        );
                    } else {
                        (*tdd).DefaultDacl = core::ptr::null_mut();
                    }
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }

                // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            7 => {
                let ts = TokenInformation as PTOKEN_SOURCE;

                required_length = core::mem::size_of::<TOKEN_SOURCE>() as ULONG;

                if TokenInformationLength >= required_length {
                    if let Err(s) = probe_write_user(ts, (*token).TokenSource, info_mode) {
                        return s;
                    }
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }

                // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            8 => {
                let tt = TokenInformation as PTOKEN_TYPE;

                required_length = core::mem::size_of::<TOKEN_TYPE>() as ULONG;

                if TokenInformationLength >= required_length {
                    if let Err(s) = probe_write_user(tt, (*token).TokenType, info_mode) {
                        return s;
                    }
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }

                // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            9 => {
                let sil = TokenInformation as PSECURITY_IMPERSONATION_LEVEL;

                if (*token).TokenType != TOKEN_IMPERSONATION {
                    status = STATUS_INVALID_INFO_CLASS;
                } else {
                    required_length = core::mem::size_of::<SECURITY_IMPERSONATION_LEVEL>() as ULONG;

                    if TokenInformationLength >= required_length {
                        if let Err(s) = probe_write_user(sil, (*token).ImpersonationLevel, info_mode) {
                            return s;
                        }
                    } else {
                        status = STATUS_BUFFER_TOO_SMALL;
                    }

                    // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                    if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                        status = s;
                    }
                }
            }

            10 => {
                let ts = TokenInformation as PTOKEN_STATISTICS;

                required_length = core::mem::size_of::<TOKEN_STATISTICS>() as ULONG;

                if TokenInformationLength >= required_length {
                    (*ts).TokenId = (*token).TokenId;
                    (*ts).AuthenticationId = (*token).AuthenticationId;
                    (*ts).ExpirationTime = (*token).ExpirationTime;
                    (*ts).TokenType = (*token).TokenType;
                    (*ts).ImpersonationLevel = (*token).ImpersonationLevel;
                    (*ts).DynamicCharged = (*token).DynamicCharged;
                    (*ts).DynamicAvailable = SepComputeAvailableDynamicSpace(
                        (*token).DynamicCharged,
                        (*token).PrimaryGroup,
                        (*token).DefaultDacl,
                    );
                    (*ts).GroupCount = (*token).UserAndGroupCount - 1;
                    (*ts).PrivilegeCount = (*token).PrivilegeCount;
                    (*ts).ModifiedId = (*token).ModifiedId;
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }

                // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            17 => {
                let to = TokenInformation as PTOKEN_ORIGIN;

                required_length = core::mem::size_of::<TOKEN_ORIGIN>() as ULONG;

                if TokenInformationLength >= required_length {
                    if let Err(s) = probe_write_user(to, TOKEN_ORIGIN { OriginatingLogonSession: (*token).AuthenticationId }, info_mode) {
                        return s;
                    }
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }

                // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            13 => {
                let groups_and_privs = TokenInformation as PTOKEN_GROUPS_AND_PRIVILEGES;

                let user_group_length = RtlLengthSidAndAttributes(
                    (*token).UserAndGroupCount,
                    (*token).UserAndGroups as *mut SID_AND_ATTRIBUTES,
                );
                let restricted_sid_length = RtlLengthSidAndAttributes(
                    (*token).RestrictedSidCount,
                    (*token).RestrictedSids as *mut SID_AND_ATTRIBUTES,
                );
                let privilege_length = (*token).PrivilegeCount * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG;

                required_length = core::mem::size_of::<TOKEN_GROUPS_AND_PRIVILEGES>() as ULONG
                    + user_group_length + restricted_sid_length + privilege_length;

                if TokenInformationLength >= required_length {
                    (*groups_and_privs).SidCount = (*token).UserAndGroupCount;
                    (*groups_and_privs).SidLength = user_group_length;
                    (*groups_and_privs).Sids = (groups_and_privs as *mut u8).offset(core::mem::size_of::<TOKEN_GROUPS_AND_PRIVILEGES>() as isize) as PSID_AND_ATTRIBUTES;

                    let sid = (((*groups_and_privs).Sids as ULONG_PTR) 
                        + ((*token).UserAndGroupCount * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG) as ULONG_PTR) as PSID;
                    let sid_len = user_group_length - ((*token).UserAndGroupCount * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG);
                    status = RtlCopySidAndAttributesArray(
                        (*token).UserAndGroupCount,
                        (*token).UserAndGroups as *mut SID_AND_ATTRIBUTES,
                        sid_len,
                        (*groups_and_privs).Sids,
                        sid,
                        core::ptr::addr_of_mut!(unused_sid),
                        core::ptr::addr_of_mut!(unused_ulong),
                    );

                    (*groups_and_privs).RestrictedSidCount = (*token).RestrictedSidCount;
                    (*groups_and_privs).RestrictedSidLength = restricted_sid_length;
                    (*groups_and_privs).RestrictedSids = core::ptr::null_mut();
                    if SeTokenIsRestricted(token) != 0 {
                        (*groups_and_privs).RestrictedSids = (((*groups_and_privs).Sids as ULONG_PTR) + user_group_length as ULONG_PTR) as PSID_AND_ATTRIBUTES;

                        let restricted_sid = (((*groups_and_privs).RestrictedSids as ULONG_PTR) 
                            + ((*token).RestrictedSidCount * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG) as ULONG_PTR) as PSID;
                        let restricted_sid_len = restricted_sid_length - ((*token).RestrictedSidCount * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG);
                        status = RtlCopySidAndAttributesArray(
                            (*token).RestrictedSidCount,
                            (*token).RestrictedSids as *mut SID_AND_ATTRIBUTES,
                            restricted_sid_len,
                            (*groups_and_privs).RestrictedSids,
                            restricted_sid,
                            core::ptr::addr_of_mut!(unused_sid),
                            core::ptr::addr_of_mut!(unused_ulong),
                        );
                    }

                    (*groups_and_privs).PrivilegeCount = (*token).PrivilegeCount;
                    (*groups_and_privs).PrivilegeLength = privilege_length;
                    (*groups_and_privs).Privileges = ((groups_and_privs as *mut u8).offset(core::mem::size_of::<TOKEN_GROUPS_AND_PRIVILEGES>() as isize) as ULONG_PTR
                        + user_group_length as ULONG_PTR + restricted_sid_length as ULONG_PTR) as PLUID_AND_ATTRIBUTES;
                    RtlCopyLuidAndAttributesArray(
                        (*token).PrivilegeCount,
                        (*token).Privileges,
                        (*groups_and_privs).Privileges,
                    );

                    (*groups_and_privs).AuthenticationId = (*token).AuthenticationId;
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }

                // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            11 => {
                let tg = TokenInformation as PTOKEN_GROUPS;

                required_length = core::mem::size_of::<ULONG>() as ULONG
                    + RtlLengthSidAndAttributes(
                        (*token).RestrictedSidCount,
                        (*token).RestrictedSids as *mut SID_AND_ATTRIBUTES,
                    );

                if TokenInformationLength >= required_length {
                    let sid_len = required_length - core::mem::size_of::<ULONG>() as ULONG
                        - ((*token).RestrictedSidCount * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG);
                    let sid = ((tg as ULONG_PTR) + core::mem::size_of::<ULONG>() as ULONG_PTR
                        + ((*token).RestrictedSidCount * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG) as ULONG_PTR) as PSID;

                    (*tg).GroupCount = (*token).RestrictedSidCount;
                    status = RtlCopySidAndAttributesArray(
                        (*token).RestrictedSidCount,
                        (*token).RestrictedSids as *mut SID_AND_ATTRIBUTES,
                        sid_len,
                        core::ptr::addr_of_mut!((*tg).Groups) as *mut SID_AND_ATTRIBUTES,
                        sid,
                        core::ptr::addr_of_mut!(unused_sid),
                        core::ptr::addr_of_mut!(unused_ulong),
                    );
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }

                // ReturnLength is written unconditionally (even on STATUS_BUFFER_TOO_SMALL)
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            15 => {
                required_length = core::mem::size_of::<ULONG>() as ULONG;
                let is_token_sandbox_inert = SeTokenIsInert(token) as ULONG;
                if let Err(s) = probe_write_user(TokenInformation as PULONG, is_token_sandbox_inert, info_mode) {
                    status = s;
                }
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            12 => {
                let mut session_id: ULONG = 0;
                required_length = core::mem::size_of::<ULONG>() as ULONG;

                status = SeQuerySessionIdToken(token, core::ptr::addr_of_mut!(session_id));
                if status >= 0 {
                    if let Err(s) = probe_write_user(TokenInformation as PULONG, session_id, info_mode) {
                        status = s;
                    }
                }
                // ReturnLength unconditional
                if let Err(s) = probe_write_user::<ULONG>(ReturnLength, required_length, previous_mode) {
                    status = s;
                }
            }

            _ => {
                status = STATUS_INVALID_INFO_CLASS;
            }
        }

        SepReleaseTokenLock(token);
        ObDereferenceObject(token as PVOID);
    }

    if status >= 0 {
        rebase_token_information(&bounce, TokenInformationClass as ULONG);
    }
    if let Err(s) = bounce.finish() {
        status = s;
    }
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetInformationToken(
    TokenHandle: HANDLE,
    TokenInformationClass: TOKEN_INFORMATION_CLASS,
    TokenInformation: PVOID,
    TokenInformationLength: ULONG,
) -> NTSTATUS {
    let TokenHandle = opaque_ptr(TokenHandle);
    let TokenInformation = opaque_ptr(TokenInformation);
    let mut status: NTSTATUS;
    let mut token: PTOKEN = core::ptr::null_mut();
    let previous_mode: KPROCESSOR_MODE;
    let mut needed_access = TOKEN_ADJUST_DEFAULT;

    previous_mode = ExGetPreviousMode();

    status = DefaultSetInfoBufferCheck(
        TokenInformationClass as ULONG,
        SE_TOKEN_INFORMATION_CLASS.as_ptr(),
        SE_TOKEN_INFORMATION_CLASS.len() as ULONG,
        TokenInformation,
        TokenInformationLength,
        previous_mode,
    );
    if status < 0 {
        return status;
    }

    if TokenInformationClass as ULONG == 12 {
        needed_access |= TOKEN_ADJUST_SESSIONID;
    }

    status = ObReferenceObjectByHandle(
        TokenHandle,
        needed_access,
        SeTokenObjectType as *mut _OBJECT_TYPE,
        previous_mode,
        core::ptr::addr_of_mut!(token) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status >= 0 {
        // Own the reference across every early return from a failed capture.
        let _token_ref = ObjRefGuard::new(token as PVOID);
        'cleanup: loop {
            match TokenInformationClass as ULONG {
                4 => {
                    if TokenInformationLength >= core::mem::size_of::<TOKEN_OWNER>() as ULONG {
                        let to = TokenInformation as PTOKEN_OWNER;
                        let input_sid = match probe_read_user::<PSID>(core::ptr::addr_of!((*to).Owner) as _, previous_mode) {
                            Ok(v) => v,
                            Err(s) => return s,
                        };
                        let mut captured_sid: PSID = core::ptr::null_mut();

                        status = SepCaptureSid(
                            input_sid,
                            previous_mode,
                            PagedPool,
                            0u8,
                            core::ptr::addr_of_mut!(captured_sid),
                        );
                        if status >= 0 {
                            let mut default_owner_index: ULONG = 0;

                            SepAcquireTokenLockExclusive(token);

                            status = SepFindPrimaryGroupAndDefaultOwner(
                                token,
                                core::ptr::null_mut(),
                                captured_sid,
                                core::ptr::null_mut(),
                                core::ptr::addr_of_mut!(default_owner_index),
                            );
                            if status >= 0 {
                                (*token).DefaultOwnerIndex = default_owner_index;
                                ExAllocateLocallyUniqueId(core::ptr::addr_of_mut!((*token).ModifiedId));
                            }

                            SepReleaseTokenLock(token);

                            SepReleaseSid(captured_sid, previous_mode, 0u8);
                        }
                    } else {
                        status = STATUS_INFO_LENGTH_MISMATCH;
                    }
                }

                5 => {
                    if TokenInformationLength >= core::mem::size_of::<TOKEN_PRIMARY_GROUP>() as ULONG {
                        let tpg = TokenInformation as PTOKEN_PRIMARY_GROUP;
                        let input_sid = match probe_read_user::<PSID>(core::ptr::addr_of!((*tpg).PrimaryGroup) as _, previous_mode) {
                            Ok(v) => v,
                            Err(s) => return s,
                        };
                        let mut captured_sid: PSID = core::ptr::null_mut();

                        status = SepCaptureSid(
                            input_sid,
                            previous_mode,
                            PagedPool,
                            0u8,
                            core::ptr::addr_of_mut!(captured_sid),
                        );
                        if status >= 0 {
                            SepAcquireTokenLockExclusive(token);

                            let acl_size = if !(*token).DefaultDacl.is_null() {
                                (*(*token).DefaultDacl).AclSize as ULONG
                            } else {
                                0
                            };
                            let new_dynamic_length = RtlLengthSid(captured_sid) + acl_size;
                            if new_dynamic_length > (*token).DynamicCharged {
                                SepReleaseTokenLock(token);
                                SepReleaseSid(captured_sid, previous_mode, 0u8);
                                status = STATUS_ALLOTTED_SPACE_EXCEEDED;
                                break 'cleanup;
                            }

                            status = SepRebuildDynamicPartOfToken(token, new_dynamic_length);
                            if status >= 0 {
                                let mut primary_group_index: ULONG = 0;

                                status = SepFindPrimaryGroupAndDefaultOwner(
                                    token,
                                    captured_sid,
                                    core::ptr::null_mut(),
                                    core::ptr::addr_of_mut!(primary_group_index),
                                    core::ptr::null_mut(),
                                );
                                if status >= 0 {
                                    (*token).DynamicAvailable += RtlLengthSid((*token).PrimaryGroup);

                                    if !(*token).DefaultDacl.is_null()
                                        && ((*token).DefaultDacl as *mut ULONG) != (*token).DynamicPart
                                    {
                                        core::ptr::copy(
                                            (*token).DefaultDacl as *const u8,
                                            (*token).DynamicPart as *mut u8,
                                            RtlLengthSid((*token).PrimaryGroup) as usize,
                                        );
                                        (*token).DefaultDacl = (*token).DynamicPart as PACL;
                                    }

                                    (*token).DynamicAvailable -= RtlLengthSid((*(*token).UserAndGroups.offset(primary_group_index as isize)).Sid);

                                    let primary_group = ((*token).DynamicPart as ULONG_PTR) + acl_size as ULONG_PTR;
                                    RtlCopySid(
                                        RtlLengthSid((*(*token).UserAndGroups.offset(primary_group_index as isize)).Sid),
                                        primary_group as PVOID,
                                        (*(*token).UserAndGroups.offset(primary_group_index as isize)).Sid,
                                    );
                                    (*token).PrimaryGroup = primary_group as PSID;

                                    ExAllocateLocallyUniqueId(core::ptr::addr_of_mut!((*token).ModifiedId));
                                }
                            }

                            SepReleaseTokenLock(token);

                            SepReleaseSid(captured_sid, previous_mode, 0u8);
                        }
                    } else {
                        status = STATUS_INFO_LENGTH_MISMATCH;
                    }
                }

                6 => {
                    if TokenInformationLength >= core::mem::size_of::<TOKEN_DEFAULT_DACL>() as ULONG {
                        let tdd = TokenInformation as PTOKEN_DEFAULT_DACL;
                        let input_acl = match probe_read_user::<PACL>(core::ptr::addr_of!((*tdd).DefaultDacl) as _, previous_mode) {
                            Ok(v) => v,
                            Err(s) => return s,
                        };

                        if !input_acl.is_null() {
                            let mut captured_acl: PACL = core::ptr::null_mut();

                            status = SepCaptureAcl(
                                input_acl,
                                previous_mode,
                                PagedPool,
                                1u8,
                                core::ptr::addr_of_mut!(captured_acl),
                            );
                            if status >= 0 {
                                SepAcquireTokenLockExclusive(token);

                                let new_dynamic_length = (*captured_acl).AclSize as ULONG + RtlLengthSid((*token).PrimaryGroup);
                                if new_dynamic_length > (*token).DynamicCharged {
                                    SepReleaseTokenLock(token);
                                    SepReleaseAcl(captured_acl, previous_mode, 1u8);
                                    status = STATUS_ALLOTTED_SPACE_EXCEEDED;
                                    break 'cleanup;
                                }

                                status = SepRebuildDynamicPartOfToken(token, new_dynamic_length);
                                if status >= 0 {
                                    if !(*token).DefaultDacl.is_null() {
                                        (*token).DynamicAvailable += (*(*token).DefaultDacl).AclSize as ULONG;
                                    }

                                    if ((*token).PrimaryGroup as *mut ULONG) != (*token).DynamicPart {
                                        core::ptr::copy(
                                            (*token).PrimaryGroup as *const u8,
                                            (*token).DynamicPart as *mut u8,
                                            RtlLengthSid((*token).PrimaryGroup) as usize,
                                        );
                                        (*token).PrimaryGroup = (*token).DynamicPart as PSID;
                                    }

                                    (*token).DynamicAvailable -= (*captured_acl).AclSize as ULONG;

                                    let acl = ((*token).DynamicPart as ULONG_PTR) + RtlLengthSid((*token).PrimaryGroup) as ULONG_PTR;
                                    core::ptr::copy_nonoverlapping(
                                        captured_acl as *const u8,
                                        acl as *mut u8,
                                        (*captured_acl).AclSize as usize,
                                    );
                                    (*token).DefaultDacl = acl as PACL;

                                    ExAllocateLocallyUniqueId(core::ptr::addr_of_mut!((*token).ModifiedId));
                                }

                                SepReleaseTokenLock(token);
                                SepReleaseAcl(captured_acl, previous_mode, 1u8);
                            }
                        } else {
                            SepAcquireTokenLockExclusive(token);

                            if !(*token).DefaultDacl.is_null() {
                                (*token).DynamicAvailable += (*(*token).DefaultDacl).AclSize as ULONG;
                                core::ptr::write_bytes((*token).DefaultDacl as *mut u8, 0, (*(*token).DefaultDacl).AclSize as usize);
                                (*token).DefaultDacl = core::ptr::null_mut();

                                ExAllocateLocallyUniqueId(core::ptr::addr_of_mut!((*token).ModifiedId));
                            }

                            SepReleaseTokenLock(token);
                        }
                    } else {
                        status = STATUS_INFO_LENGTH_MISMATCH;
                    }
                }

                12 => {
                    let session_id = match probe_read_user::<ULONG>(TokenInformation as _, previous_mode) {
                        Ok(v) => v,
                        Err(s) => return s,
                    };

                    if SeSinglePrivilegeCheck(SeTcbPrivilege, previous_mode) == 0 {
                        status = STATUS_PRIVILEGE_NOT_HELD;
                        break 'cleanup;
                    }

                    SepAcquireTokenLockExclusive(token);

                    (*token).SessionId = session_id;
                    ExAllocateLocallyUniqueId(core::ptr::addr_of_mut!((*token).ModifiedId));

                    SepReleaseTokenLock(token);
                }

                14 => {
                    let session_reference = match probe_read_user::<ULONG>(TokenInformation as _, previous_mode) {
                        Ok(v) => v,
                        Err(s) => return s,
                    };

                    if SeSinglePrivilegeCheck(SeTcbPrivilege, previous_mode) == 0 {
                        status = STATUS_PRIVILEGE_NOT_HELD;
                        break 'cleanup;
                    }

                    if session_reference == 0 {
                        SepAcquireTokenLockExclusive(token);

                        let old_token_flags = RtlInterlockedSetBits(
                            core::ptr::addr_of_mut!((*token).TokenFlags) as *mut ULONG,
                            TOKEN_SESSION_NOT_REFERENCED,
                        );
                        
                        let session_reference_val = if old_token_flags == (*token).TokenFlags {
                            core::u32::MAX
                        } else {
                            0
                        };

                        if session_reference_val == 0 {
                            SepRmRemoveLogonSessionFromToken(token);
                            SepRmDereferenceLogonSession(core::ptr::addr_of!((*token).AuthenticationId));
                        }

                        SepReleaseTokenLock(token);
                    }
                }

                16 => {
                    let policy_information = TokenInformation as PTOKEN_AUDIT_POLICY_INFORMATION;
                    let mut audit_policy: _SEP_AUDIT_POLICY = core::mem::zeroed();
                    let header_size = core::mem::offset_of!(TOKEN_AUDIT_POLICY_INFORMATION, Policies);
                    if (TokenInformationLength as usize) < header_size {
                        status = STATUS_INFO_LENGTH_MISMATCH;
                        break 'cleanup;
                    }
                    // Capture the count once. Never re-read a caller-controlled
                    // count after checking the extent of its flexible array.
                    let policy_count = match probe_read_user::<ULONG>(TokenInformation as _, previous_mode) {
                        Ok(v) => v,
                        Err(s) => return s,
                    };
                    let policy_size = core::mem::size_of::<_TOKEN_AUDIT_POLICY_INFORMATION__bindgen_ty_1>();
                    let required_length = match (policy_count as usize).checked_mul(policy_size)
                        .and_then(|size| size.checked_add(header_size))
                    {
                        Some(size) if size <= TokenInformationLength as usize => size,
                        _ => {
                            status = STATUS_INFO_LENGTH_MISMATCH;
                            break 'cleanup;
                        }
                    };
                    if let Err(s) = probe_user_buffer(
                        TokenInformation, required_length, core::mem::align_of::<ULONG>(), previous_mode,
                    ) {
                        return s;
                    }

                    for i in 0..policy_count {
                        let policy_ptr = (policy_information as *const u8)
                            .wrapping_add(header_size + i as usize * policy_size)
                            .cast::<_TOKEN_AUDIT_POLICY_INFORMATION__bindgen_ty_1>();
                        let policy = match probe_read_user(policy_ptr, previous_mode) {
                            Ok(v) => v,
                            Err(s) => return s,
                        };
                        let elements = &mut audit_policy.__bindgen_anon_1.PolicyElements;
                        match policy.Category {
                            AUDIT_CATEGORY_SYSTEM => elements.set_System(policy.Value),
                            AUDIT_CATEGORY_LOGON => elements.set_Logon(policy.Value),
                            AUDIT_CATEGORY_OBJECT_ACCESS => elements.set_ObjectAccess(policy.Value),
                            AUDIT_CATEGORY_PRIVILEGE_USE => elements.set_PrivilegeUse(policy.Value),
                            AUDIT_CATEGORY_DETAILED_TRACKING => elements.set_DetailedTracking(policy.Value),
                            AUDIT_CATEGORY_POLICY_CHANGE => elements.set_PolicyChange(policy.Value),
                            AUDIT_CATEGORY_ACCOUNT_MANAGEMENT => elements.set_AccountManagement(policy.Value),
                            AUDIT_CATEGORY_DIRECTORY_SERVICE_ACCESS => elements.set_DirectoryServiceAccess(policy.Value),
                            AUDIT_CATEGORY_ACCOUNT_LOGON => elements.set_AccountLogon(policy.Value),
                            _ => {}
                        }
                    }

                    if SeSinglePrivilegeCheck(SeTcbPrivilege, previous_mode) == 0 {
                        status = STATUS_PRIVILEGE_NOT_HELD;
                        break 'cleanup;
                    }

                    SepAcquireTokenLockExclusive(token);

                    (*token).AuditPolicy = audit_policy;
                    ExAllocateLocallyUniqueId(core::ptr::addr_of_mut!((*token).ModifiedId));

                    SepReleaseTokenLock(token);
                }

                17 => {
                    let token_origin = match probe_read_user::<TOKEN_ORIGIN>(TokenInformation as _, previous_mode) {
                        Ok(v) => v,
                        Err(s) => return s,
                    };

                    if SeSinglePrivilegeCheck(SeTcbPrivilege, previous_mode) == 0 {
                        status = STATUS_PRIVILEGE_NOT_HELD;
                        break 'cleanup;
                    }

                    SepAcquireTokenLockExclusive(token);

                    if ((*token).OriginatingLogonSession.LowPart == 0) && ((*token).OriginatingLogonSession.HighPart == 0) {
                        (*token).OriginatingLogonSession = token_origin.OriginatingLogonSession;

                        ExAllocateLocallyUniqueId(core::ptr::addr_of_mut!((*token).ModifiedId));
                    }

                    SepReleaseTokenLock(token);
                }

                _ => {
                    status = STATUS_INVALID_INFO_CLASS;
                }
            }
            break;
        }

    }

    status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Access token Query/Set information classes implementation
//  * COPYRIGHT:   Copyright David Welch <welch@cwcom.net>
//  *              Copyright 2021-2022 George Bișoc <george.bisoc@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #include <ntlsa.h>
//
// /* INFORMATION CLASSES ********************************************************/
//
// static const INFORMATION_CLASS_INFO SeTokenInformationClass[] = {
//
//     /* Class 0 not used, blame MS! */
//     IQS_NONE,
//
//     /* TokenUser */
//     IQS_SAME(TOKEN_USER, ULONG, ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE),
//     /* TokenGroups */
//     IQS_SAME(TOKEN_GROUPS, ULONG, ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE),
//     /* TokenPrivileges */
//     IQS_SAME(TOKEN_PRIVILEGES, ULONG, ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE),
//     /* TokenOwner */
//     IQS_SAME(TOKEN_OWNER, ULONG, ICIF_QUERY | ICIF_SET | ICIF_SIZE_VARIABLE),
//     /* TokenPrimaryGroup */
//     IQS_SAME(TOKEN_PRIMARY_GROUP, ULONG, ICIF_QUERY | ICIF_SET | ICIF_SIZE_VARIABLE),
//     /* TokenDefaultDacl */
//     IQS_SAME(TOKEN_DEFAULT_DACL, ULONG, ICIF_QUERY | ICIF_SET | ICIF_SIZE_VARIABLE),
//     /* TokenSource */
//     IQS_SAME(TOKEN_SOURCE, ULONG, ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE),
//     /* TokenType */
//     IQS_SAME(TOKEN_TYPE, ULONG, ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE),
//     /* TokenImpersonationLevel */
//     IQS_SAME(SECURITY_IMPERSONATION_LEVEL, ULONG, ICIF_QUERY),
//     /* TokenStatistics */
//     IQS_SAME(TOKEN_STATISTICS, ULONG, ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE),
//     /* TokenRestrictedSids */
//     IQS_SAME(TOKEN_GROUPS, ULONG, ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE),
//     /* TokenSessionId */
//     IQS_SAME(ULONG, ULONG, ICIF_QUERY | ICIF_SET),
//     /* TokenGroupsAndPrivileges */
//     IQS_SAME(TOKEN_GROUPS_AND_PRIVILEGES, ULONG, ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE),
//     /* TokenSessionReference */
//     IQS_SAME(ULONG, ULONG, ICIF_SET),
//     /* TokenSandBoxInert */
//     IQS_SAME(ULONG, ULONG, ICIF_QUERY),
//     /* TokenAuditPolicy */
//     IQS_SAME(TOKEN_AUDIT_POLICY_INFORMATION, ULONG, ICIF_SET | ICIF_SET_SIZE_VARIABLE),
//     /* TokenOrigin */
//     IQS_SAME(TOKEN_ORIGIN, ULONG, ICIF_QUERY | ICIF_SET),
// };
//
// /* PUBLIC FUNCTIONS *****************************************************************/
//
// /**
//  * @brief
//  * Queries information details about the given token to the call. The difference
//  * between NtQueryInformationToken and this routine is that the system call has
//  * user mode buffer data probing and additional protection checks whereas this
//  * routine doesn't have any of these. The routine is used exclusively in kernel
//  * mode.
//  *
//  * @param[in] AccessToken
//  * An access token to be given.
//  *
//  * @param[in] TokenInformationClass
//  * Token information class.
//  *
//  * @param[out] TokenInformation
//  * Buffer with retrieved information. Such information is arbitrary, depending
//  * on the requested information class.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the operation to query the desired information
//  * has completed successfully. STATUS_INSUFFICIENT_RESOURCES is returned if
//  * pool memory allocation has failed to satisfy an operation. Otherwise
//  * STATUS_INVALID_INFO_CLASS is returned indicating that the information
//  * class provided is not supported by the routine.
//  *
//  * @remarks
//  * Only certain information classes are not implemented in this function and
//  * these are TokenOrigin, TokenGroupsAndPrivileges, TokenRestrictedSids and
//  * TokenSandBoxInert. The following classes are implemented in NtQueryInformationToken
//  * only.
//  */
// NTSTATUS
// NTAPI
// SeQueryInformationToken(
//     _In_ PACCESS_TOKEN AccessToken,
//     _In_ TOKEN_INFORMATION_CLASS TokenInformationClass,
//     _Outptr_result_buffer_(_Inexpressible_(token-dependent)) PVOID *TokenInformation)
// {
//     NTSTATUS Status;
//     PTOKEN Token = (PTOKEN)AccessToken;
//     ULONG RequiredLength;
//     union
//     {
//         PSID PSid;
//         ULONG Ulong;
//     } Unused;
//
//     PAGED_CODE();
//
//     /* Lock the token */
//     SepAcquireTokenLockShared(Token);
//
//     switch (TokenInformationClass)
//     {
//         case TokenUser:
//         {
//             PTOKEN_USER tu;
//
//             DPRINT("SeQueryInformationToken(TokenUser)\n");
//             RequiredLength = sizeof(TOKEN_USER) +
//                 RtlLengthSid(Token->UserAndGroups[0].Sid);
//
//             /* Allocate the output buffer */
//             tu = ExAllocatePoolWithTag(PagedPool, RequiredLength, TAG_SE);
//             if (tu == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             Status = RtlCopySidAndAttributesArray(1,
//                                                   &Token->UserAndGroups[0],
//                                                   RequiredLength - sizeof(TOKEN_USER),
//                                                   &tu->User,
//                                                   (PSID)(tu + 1),
//                                                   &Unused.PSid,
//                                                   &Unused.Ulong);
//
//             /* Return the structure */
//             *TokenInformation = tu;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case TokenGroups:
//         {
//             PTOKEN_GROUPS tg;
//             ULONG SidLen;
//             PSID Sid;
//
//             DPRINT("SeQueryInformationToken(TokenGroups)\n");
//             RequiredLength = sizeof(tg->GroupCount) +
//                 RtlLengthSidAndAttributes(Token->UserAndGroupCount - 1, &Token->UserAndGroups[1]);
//
//             SidLen = RequiredLength - sizeof(tg->GroupCount) -
//                 ((Token->UserAndGroupCount - 1) * sizeof(SID_AND_ATTRIBUTES));
//
//             /* Allocate the output buffer */
//             tg = ExAllocatePoolWithTag(PagedPool, RequiredLength, TAG_SE);
//             if (tg == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             Sid = (PSID)((ULONG_PTR)tg + sizeof(tg->GroupCount) +
//                          ((Token->UserAndGroupCount - 1) * sizeof(SID_AND_ATTRIBUTES)));
//
//             tg->GroupCount = Token->UserAndGroupCount - 1;
//             Status = RtlCopySidAndAttributesArray(Token->UserAndGroupCount - 1,
//                                                   &Token->UserAndGroups[1],
//                                                   SidLen,
//                                                   &tg->Groups[0],
//                                                   Sid,
//                                                   &Unused.PSid,
//                                                   &Unused.Ulong);
//
//             /* Return the structure */
//             *TokenInformation = tg;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case TokenPrivileges:
//         {
//             PTOKEN_PRIVILEGES tp;
//
//             DPRINT("SeQueryInformationToken(TokenPrivileges)\n");
//             RequiredLength = sizeof(tp->PrivilegeCount) +
//                 (Token->PrivilegeCount * sizeof(LUID_AND_ATTRIBUTES));
//
//             /* Allocate the output buffer */
//             tp = ExAllocatePoolWithTag(PagedPool, RequiredLength, TAG_SE);
//             if (tp == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             tp->PrivilegeCount = Token->PrivilegeCount;
//             RtlCopyLuidAndAttributesArray(Token->PrivilegeCount,
//                                           Token->Privileges,
//                                           &tp->Privileges[0]);
//
//             /* Return the structure */
//             *TokenInformation = tp;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case TokenOwner:
//         {
//             PTOKEN_OWNER to;
//             ULONG SidLen;
//
//             DPRINT("SeQueryInformationToken(TokenOwner)\n");
//             SidLen = RtlLengthSid(Token->UserAndGroups[Token->DefaultOwnerIndex].Sid);
//             RequiredLength = sizeof(TOKEN_OWNER) + SidLen;
//
//             /* Allocate the output buffer */
//             to = ExAllocatePoolWithTag(PagedPool, RequiredLength, TAG_SE);
//             if (to == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             to->Owner = (PSID)(to + 1);
//             Status = RtlCopySid(SidLen,
//                                 to->Owner,
//                                 Token->UserAndGroups[Token->DefaultOwnerIndex].Sid);
//
//             /* Return the structure */
//             *TokenInformation = to;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case TokenPrimaryGroup:
//         {
//             PTOKEN_PRIMARY_GROUP tpg;
//             ULONG SidLen;
//
//             DPRINT("SeQueryInformationToken(TokenPrimaryGroup)\n");
//             SidLen = RtlLengthSid(Token->PrimaryGroup);
//             RequiredLength = sizeof(TOKEN_PRIMARY_GROUP) + SidLen;
//
//             /* Allocate the output buffer */
//             tpg = ExAllocatePoolWithTag(PagedPool, RequiredLength, TAG_SE);
//             if (tpg == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             tpg->PrimaryGroup = (PSID)(tpg + 1);
//             Status = RtlCopySid(SidLen,
//                                 tpg->PrimaryGroup,
//                                 Token->PrimaryGroup);
//
//             /* Return the structure */
//             *TokenInformation = tpg;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case TokenDefaultDacl:
//         {
//             PTOKEN_DEFAULT_DACL tdd;
//
//             DPRINT("SeQueryInformationToken(TokenDefaultDacl)\n");
//             RequiredLength = sizeof(TOKEN_DEFAULT_DACL);
//
//             if (Token->DefaultDacl != NULL)
//                 RequiredLength += Token->DefaultDacl->AclSize;
//
//             /* Allocate the output buffer */
//             tdd = ExAllocatePoolWithTag(PagedPool, RequiredLength, TAG_SE);
//             if (tdd == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             if (Token->DefaultDacl != NULL)
//             {
//                 tdd->DefaultDacl = (PACL)(tdd + 1);
//                 RtlCopyMemory(tdd->DefaultDacl,
//                               Token->DefaultDacl,
//                               Token->DefaultDacl->AclSize);
//             }
//             else
//             {
//                 tdd->DefaultDacl = NULL;
//             }
//
//             /* Return the structure */
//             *TokenInformation = tdd;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case TokenSource:
//         {
//             PTOKEN_SOURCE ts;
//
//             DPRINT("SeQueryInformationToken(TokenSource)\n");
//             RequiredLength = sizeof(TOKEN_SOURCE);
//
//             /* Allocate the output buffer */
//             ts = ExAllocatePoolWithTag(PagedPool, RequiredLength, TAG_SE);
//             if (ts == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             *ts = Token->TokenSource;
//
//             /* Return the structure */
//             *TokenInformation = ts;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case TokenType:
//         {
//             PTOKEN_TYPE tt;
//
//             DPRINT("SeQueryInformationToken(TokenType)\n");
//             RequiredLength = sizeof(TOKEN_TYPE);
//
//             /* Allocate the output buffer */
//             tt = ExAllocatePoolWithTag(PagedPool, RequiredLength, TAG_SE);
//             if (tt == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             *tt = Token->TokenType;
//
//             /* Return the structure */
//             *TokenInformation = tt;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case TokenImpersonationLevel:
//         {
//             PSECURITY_IMPERSONATION_LEVEL sil;
//
//             DPRINT("SeQueryInformationToken(TokenImpersonationLevel)\n");
//             RequiredLength = sizeof(SECURITY_IMPERSONATION_LEVEL);
//
//             /* Fail if the token is not an impersonation token */
//             if (Token->TokenType != TokenImpersonation)
//             {
//                 Status = STATUS_INVALID_INFO_CLASS;
//                 break;
//             }
//
//             /* Allocate the output buffer */
//             sil = ExAllocatePoolWithTag(PagedPool, RequiredLength, TAG_SE);
//             if (sil == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             *sil = Token->ImpersonationLevel;
//
//             /* Return the structure */
//             *TokenInformation = sil;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case TokenStatistics:
//         {
//             PTOKEN_STATISTICS ts;
//
//             DPRINT("SeQueryInformationToken(TokenStatistics)\n");
//             RequiredLength = sizeof(TOKEN_STATISTICS);
//
//             /* Allocate the output buffer */
//             ts = ExAllocatePoolWithTag(PagedPool, RequiredLength, TAG_SE);
//             if (ts == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             ts->TokenId = Token->TokenId;
//             ts->AuthenticationId = Token->AuthenticationId;
//             ts->ExpirationTime = Token->ExpirationTime;
//             ts->TokenType = Token->TokenType;
//             ts->ImpersonationLevel = Token->ImpersonationLevel;
//             ts->DynamicCharged = Token->DynamicCharged;
//             ts->DynamicAvailable = SepComputeAvailableDynamicSpace(Token->DynamicCharged, Token->PrimaryGroup, Token->DefaultDacl);
//             ts->GroupCount = Token->UserAndGroupCount - 1;
//             ts->PrivilegeCount = Token->PrivilegeCount;
//             ts->ModifiedId = Token->ModifiedId;
//
//             /* Return the structure */
//             *TokenInformation = ts;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case TokenSessionId:
//         {
//             DPRINT("SeQueryInformationToken(TokenSessionId)\n");
//             Status = SeQuerySessionIdToken(Token, (PULONG)TokenInformation);
//             break;
//         }
//
//         default:
//             DPRINT1("SeQueryInformationToken(%d) invalid information class\n", TokenInformationClass);
//             Status = STATUS_INVALID_INFO_CLASS;
//             break;
//     }
//
//     /* Release the lock of the token */
//     SepReleaseTokenLock(Token);
//
//     return Status;
// }
//
// /* SYSTEM CALLS ***************************************************************/
//
// /**
//  * @brief
//  * Queries a specific type of information in regard of an access token based upon
//  * the information class. The calling thread must have specific access rights in order
//  * to obtain specific information about the token.
//  *
//  * @param[in] TokenHandle
//  * A handle of a token where information is to be gathered.
//  *
//  * @param[in] TokenInformationClass
//  * Token information class.
//  *
//  * @param[out] TokenInformation
//  * A returned output buffer with token information, which information is arbitrarily upon
//  * the information class chosen.
//  *
//  * @param[in] TokenInformationLength
//  * Length of the token information buffer, in bytes.
//  *
//  * @param[out] ReturnLength
//  * A pointer to a variable provided by the caller that receives the actual length
//  * of the buffer pointed by TokenInformation, in bytes. If TokenInformation is NULL
//  * and TokenInformationLength is 0, this parameter receives the required length
//  * needed to store the buffer information in memory. This parameter must not
//  * be NULL!
//  *
//  * @return
//  * Returns STATUS_SUCCESS if information querying has completed successfully.
//  * STATUS_BUFFER_TOO_SMALL is returned if the information length that represents
//  * the token information buffer is not greater than the required length.
//  * STATUS_INVALID_HANDLE is returned if the token handle is not a valid one.
//  * STATUS_INVALID_INFO_CLASS is returned if the information class is not a valid
//  * one (that is, the class doesn't belong to TOKEN_INFORMATION_CLASS).
//  * STATUS_ACCESS_VIOLATION is returned if ReturnLength is NULL. A failure NTSTATUS
//  * code is returned otherwise.
//  */
// _Must_inspect_result_
// __kernel_entry
// NTSTATUS
// NTAPI
// NtQueryInformationToken(
//     _In_ HANDLE TokenHandle,
//     _In_ TOKEN_INFORMATION_CLASS TokenInformationClass,
//     _Out_writes_bytes_to_opt_(TokenInformationLength, *ReturnLength)
//         PVOID TokenInformation,
//     _In_ ULONG TokenInformationLength,
//     _Out_ PULONG ReturnLength)
// {
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode;
//     PTOKEN Token;
//     ULONG RequiredLength = 0;
//     union
//     {
//         PSID PSid;
//         ULONG Ulong;
//     } Unused;
//
//     PAGED_CODE();
//
//     PreviousMode = ExGetPreviousMode();
//
//     /* Check buffers and class validity */
//     Status = DefaultQueryInfoBufferCheck(TokenInformationClass,
//                                          SeTokenInformationClass,
//                                          RTL_NUMBER_OF(SeTokenInformationClass),
//                                          ICIF_PROBE_READ_WRITE | ICIF_FORCE_RETURN_LENGTH_PROBE,
//                                          TokenInformation,
//                                          TokenInformationLength,
//                                          ReturnLength,
//                                          NULL,
//                                          PreviousMode);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("NtQueryInformationToken() failed, Status: 0x%x\n", Status);
//         return Status;
//     }
//
//     Status = ObReferenceObjectByHandle(TokenHandle,
//                                        (TokenInformationClass == TokenSource) ? TOKEN_QUERY_SOURCE : TOKEN_QUERY,
//                                        SeTokenObjectType as *mut _OBJECT_TYPE,
//                                        PreviousMode,
//                                        (PVOID*)&Token,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Lock the token */
//         SepAcquireTokenLockShared(Token);
//
//         switch (TokenInformationClass)
//         {
//             case TokenUser:
//             {
//                 PTOKEN_USER tu = (PTOKEN_USER)TokenInformation;
//
//                 DPRINT("NtQueryInformationToken(TokenUser)\n");
//                 RequiredLength = sizeof(TOKEN_USER) +
//                     RtlLengthSid(Token->UserAndGroups[0].Sid);
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         Status = RtlCopySidAndAttributesArray(1,
//                                                               &Token->UserAndGroups[0],
//                                                               RequiredLength - sizeof(TOKEN_USER),
//                                                               &tu->User,
//                                                               (PSID)(tu + 1),
//                                                               &Unused.PSid,
//                                                               &Unused.Ulong);
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                     *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenGroups:
//             {
//                 PTOKEN_GROUPS tg = (PTOKEN_GROUPS)TokenInformation;
//
//                 DPRINT("NtQueryInformationToken(TokenGroups)\n");
//                 RequiredLength = sizeof(tg->GroupCount) +
//                     RtlLengthSidAndAttributes(Token->UserAndGroupCount - 1, &Token->UserAndGroups[1]);
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         ULONG SidLen = RequiredLength - sizeof(tg->GroupCount) -
//                             ((Token->UserAndGroupCount - 1) * sizeof(SID_AND_ATTRIBUTES));
//                         PSID Sid = (PSID_AND_ATTRIBUTES)((ULONG_PTR)tg + sizeof(tg->GroupCount) +
//                                                          ((Token->UserAndGroupCount - 1) * sizeof(SID_AND_ATTRIBUTES)));
//
//                         tg->GroupCount = Token->UserAndGroupCount - 1;
//                         Status = RtlCopySidAndAttributesArray(Token->UserAndGroupCount - 1,
//                                                               &Token->UserAndGroups[1],
//                                                               SidLen,
//                                                               &tg->Groups[0],
//                                                               Sid,
//                                                               &Unused.PSid,
//                                                               &Unused.Ulong);
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                     *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenPrivileges:
//             {
//                 PTOKEN_PRIVILEGES tp = (PTOKEN_PRIVILEGES)TokenInformation;
//
//                 DPRINT("NtQueryInformationToken(TokenPrivileges)\n");
//                 RequiredLength = sizeof(tp->PrivilegeCount) +
//                     (Token->PrivilegeCount * sizeof(LUID_AND_ATTRIBUTES));
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         tp->PrivilegeCount = Token->PrivilegeCount;
//                         RtlCopyLuidAndAttributesArray(Token->PrivilegeCount,
//                                                       Token->Privileges,
//                                                       &tp->Privileges[0]);
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                     *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenOwner:
//             {
//                 PTOKEN_OWNER to = (PTOKEN_OWNER)TokenInformation;
//                 ULONG SidLen;
//
//                 DPRINT("NtQueryInformationToken(TokenOwner)\n");
//                 SidLen = RtlLengthSid(Token->UserAndGroups[Token->DefaultOwnerIndex].Sid);
//                 RequiredLength = sizeof(TOKEN_OWNER) + SidLen;
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         to->Owner = (PSID)(to + 1);
//                         Status = RtlCopySid(SidLen,
//                                             to->Owner,
//                                             Token->UserAndGroups[Token->DefaultOwnerIndex].Sid);
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                    *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenPrimaryGroup:
//             {
//                 PTOKEN_PRIMARY_GROUP tpg = (PTOKEN_PRIMARY_GROUP)TokenInformation;
//                 ULONG SidLen;
//
//                 DPRINT("NtQueryInformationToken(TokenPrimaryGroup)\n");
//                 SidLen = RtlLengthSid(Token->PrimaryGroup);
//                 RequiredLength = sizeof(TOKEN_PRIMARY_GROUP) + SidLen;
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         tpg->PrimaryGroup = (PSID)(tpg + 1);
//                         Status = RtlCopySid(SidLen,
//                                             tpg->PrimaryGroup,
//                                             Token->PrimaryGroup);
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                     *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenDefaultDacl:
//             {
//                 PTOKEN_DEFAULT_DACL tdd = (PTOKEN_DEFAULT_DACL)TokenInformation;
//
//                 DPRINT("NtQueryInformationToken(TokenDefaultDacl)\n");
//                 RequiredLength = sizeof(TOKEN_DEFAULT_DACL);
//
//                 if (Token->DefaultDacl != NULL)
//                     RequiredLength += Token->DefaultDacl->AclSize;
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         if (Token->DefaultDacl != NULL)
//                         {
//                             tdd->DefaultDacl = (PACL)(tdd + 1);
//                             RtlCopyMemory(tdd->DefaultDacl,
//                                           Token->DefaultDacl,
//                                           Token->DefaultDacl->AclSize);
//                         }
//                         else
//                         {
//                             tdd->DefaultDacl = NULL;
//                         }
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                     *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenSource:
//             {
//                 PTOKEN_SOURCE ts = (PTOKEN_SOURCE)TokenInformation;
//
//                 DPRINT("NtQueryInformationToken(TokenSource)\n");
//                 RequiredLength = sizeof(TOKEN_SOURCE);
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         *ts = Token->TokenSource;
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                    *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenType:
//             {
//                 PTOKEN_TYPE tt = (PTOKEN_TYPE)TokenInformation;
//
//                 DPRINT("NtQueryInformationToken(TokenType)\n");
//                 RequiredLength = sizeof(TOKEN_TYPE);
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         *tt = Token->TokenType;
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                     *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenImpersonationLevel:
//             {
//                 PSECURITY_IMPERSONATION_LEVEL sil = (PSECURITY_IMPERSONATION_LEVEL)TokenInformation;
//
//                 DPRINT("NtQueryInformationToken(TokenImpersonationLevel)\n");
//
//                 /* Fail if the token is not an impersonation token */
//                 if (Token->TokenType != TokenImpersonation)
//                 {
//                     Status = STATUS_INVALID_INFO_CLASS;
//                     break;
//                 }
//
//                 RequiredLength = sizeof(SECURITY_IMPERSONATION_LEVEL);
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         *sil = Token->ImpersonationLevel;
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                     *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenStatistics:
//             {
//                 PTOKEN_STATISTICS ts = (PTOKEN_STATISTICS)TokenInformation;
//
//                 DPRINT("NtQueryInformationToken(TokenStatistics)\n");
//                 RequiredLength = sizeof(TOKEN_STATISTICS);
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         ts->TokenId = Token->TokenId;
//                         ts->AuthenticationId = Token->AuthenticationId;
//                         ts->ExpirationTime = Token->ExpirationTime;
//                         ts->TokenType = Token->TokenType;
//                         ts->ImpersonationLevel = Token->ImpersonationLevel;
//                         ts->DynamicCharged = Token->DynamicCharged;
//                         ts->DynamicAvailable = SepComputeAvailableDynamicSpace(Token->DynamicCharged, Token->PrimaryGroup, Token->DefaultDacl);
//                         ts->GroupCount = Token->UserAndGroupCount - 1;
//                         ts->PrivilegeCount = Token->PrivilegeCount;
//                         ts->ModifiedId = Token->ModifiedId;
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                     *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenOrigin:
//             {
//                 PTOKEN_ORIGIN to = (PTOKEN_ORIGIN)TokenInformation;
//
//                 DPRINT("NtQueryInformationToken(TokenOrigin)\n");
//                 RequiredLength = sizeof(TOKEN_ORIGIN);
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         to->OriginatingLogonSession = Token->AuthenticationId;
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                     *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenGroupsAndPrivileges:
//             {
//                 PSID Sid, RestrictedSid;
//                 ULONG SidLen, RestrictedSidLen;
//                 ULONG UserGroupLength, RestrictedSidLength, PrivilegeLength;
//                 PTOKEN_GROUPS_AND_PRIVILEGES GroupsAndPrivs = (PTOKEN_GROUPS_AND_PRIVILEGES)TokenInformation;
//
//                 DPRINT("NtQueryInformationToken(TokenGroupsAndPrivileges)\n");
//                 UserGroupLength = RtlLengthSidAndAttributes(Token->UserAndGroupCount, Token->UserAndGroups);
//                 RestrictedSidLength = RtlLengthSidAndAttributes(Token->RestrictedSidCount, Token->RestrictedSids);
//                 PrivilegeLength = Token->PrivilegeCount * sizeof(LUID_AND_ATTRIBUTES);
//
//                 RequiredLength = sizeof(TOKEN_GROUPS_AND_PRIVILEGES) +
//                                  UserGroupLength + RestrictedSidLength + PrivilegeLength;
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         GroupsAndPrivs->SidCount = Token->UserAndGroupCount;
//                         GroupsAndPrivs->SidLength = UserGroupLength;
//                         GroupsAndPrivs->Sids = (PSID_AND_ATTRIBUTES)(GroupsAndPrivs + 1);
//
//                         Sid = (PSID)((ULONG_PTR)GroupsAndPrivs->Sids + (Token->UserAndGroupCount * sizeof(SID_AND_ATTRIBUTES)));
//                         SidLen = UserGroupLength - (Token->UserAndGroupCount * sizeof(SID_AND_ATTRIBUTES));
//                         Status = RtlCopySidAndAttributesArray(Token->UserAndGroupCount,
//                                                               Token->UserAndGroups,
//                                                               SidLen,
//                                                               GroupsAndPrivs->Sids,
//                                                               Sid,
//                                                               &Unused.PSid,
//                                                               &Unused.Ulong);
//                         NT_ASSERT(NT_SUCCESS(Status));
//
//                         GroupsAndPrivs->RestrictedSidCount = Token->RestrictedSidCount;
//                         GroupsAndPrivs->RestrictedSidLength = RestrictedSidLength;
//                         GroupsAndPrivs->RestrictedSids = NULL;
//                         if (SeTokenIsRestricted(Token))
//                         {
//                             GroupsAndPrivs->RestrictedSids = (PSID_AND_ATTRIBUTES)((ULONG_PTR)GroupsAndPrivs->Sids + UserGroupLength);
//
//                             RestrictedSid = (PSID)((ULONG_PTR)GroupsAndPrivs->RestrictedSids + (Token->RestrictedSidCount * sizeof(SID_AND_ATTRIBUTES)));
//                             RestrictedSidLen = RestrictedSidLength - (Token->RestrictedSidCount * sizeof(SID_AND_ATTRIBUTES));
//                             Status = RtlCopySidAndAttributesArray(Token->RestrictedSidCount,
//                                                                   Token->RestrictedSids,
//                                                                   RestrictedSidLen,
//                                                                   GroupsAndPrivs->RestrictedSids,
//                                                                   RestrictedSid,
//                                                                   &Unused.PSid,
//                                                                   &Unused.Ulong);
//                             NT_ASSERT(NT_SUCCESS(Status));
//                         }
//
//                         GroupsAndPrivs->PrivilegeCount = Token->PrivilegeCount;
//                         GroupsAndPrivs->PrivilegeLength = PrivilegeLength;
//                         GroupsAndPrivs->Privileges = (PLUID_AND_ATTRIBUTES)((ULONG_PTR)(GroupsAndPrivs + 1) +
//                                                      UserGroupLength + RestrictedSidLength);
//                         RtlCopyLuidAndAttributesArray(Token->PrivilegeCount,
//                                                       Token->Privileges,
//                                                       GroupsAndPrivs->Privileges);
//
//                         GroupsAndPrivs->AuthenticationId = Token->AuthenticationId;
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                     *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenRestrictedSids:
//             {
//                 PTOKEN_GROUPS tg = (PTOKEN_GROUPS)TokenInformation;
//
//                 DPRINT("NtQueryInformationToken(TokenRestrictedSids)\n");
//                 RequiredLength = sizeof(tg->GroupCount) +
//                 RtlLengthSidAndAttributes(Token->RestrictedSidCount, Token->RestrictedSids);
//
//                 _SEH2_TRY
//                 {
//                     if (TokenInformationLength >= RequiredLength)
//                     {
//                         ULONG SidLen = RequiredLength - sizeof(tg->GroupCount) -
//                             (Token->RestrictedSidCount * sizeof(SID_AND_ATTRIBUTES));
//                         PSID Sid = (PSID)((ULONG_PTR)tg + sizeof(tg->GroupCount) +
//                                           (Token->RestrictedSidCount * sizeof(SID_AND_ATTRIBUTES)));
//
//                         tg->GroupCount = Token->RestrictedSidCount;
//                         Status = RtlCopySidAndAttributesArray(Token->RestrictedSidCount,
//                                                               Token->RestrictedSids,
//                                                               SidLen,
//                                                               &tg->Groups[0],
//                                                               Sid,
//                                                               &Unused.PSid,
//                                                               &Unused.Ulong);
//                     }
//                     else
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//
//                     *ReturnLength = RequiredLength;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenSandBoxInert:
//             {
//                 ULONG IsTokenSandBoxInert;
//
//                 DPRINT("NtQueryInformationToken(TokenSandBoxInert)\n");
//
//                 IsTokenSandBoxInert = SeTokenIsInert(Token);
//                 _SEH2_TRY
//                 {
//                     /* Buffer size was already verified, no need to check here again */
//                     *(PULONG)TokenInformation = IsTokenSandBoxInert;
//                     *ReturnLength = sizeof(ULONG);
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 break;
//             }
//
//             case TokenSessionId:
//             {
//                 ULONG SessionId = 0;
//
//                 DPRINT("NtQueryInformationToken(TokenSessionId)\n");
//
//                 Status = SeQuerySessionIdToken(Token, &SessionId);
//                 if (NT_SUCCESS(Status))
//                 {
//                     _SEH2_TRY
//                     {
//                         /* Buffer size was already verified, no need to check here again */
//                         *(PULONG)TokenInformation = SessionId;
//                         *ReturnLength = RequiredLength;
//                     }
//                     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                     {
//                         Status = _SEH2_GetExceptionCode();
//                     }
//                     _SEH2_END;
//                 }
//
//                 break;
//             }
//
//             default:
//                 DPRINT1("NtQueryInformationToken(%d) invalid information class\n", TokenInformationClass);
//                 Status = STATUS_INVALID_INFO_CLASS;
//                 break;
//         }
//
//         /* Unlock and dereference the token */
//         SepReleaseTokenLock(Token);
//         ObDereferenceObject(Token);
//     }
//
//     return Status;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Sets (modifies) some specific information in regard of an access token. The
//  * calling thread must have specific access rights in order to modify token's
//  * information data.
//  *
//  * @param[in] TokenHandle
//  * A handle of a token where information is to be modified.
//  *
//  * @param[in] TokenInformationClass
//  * Token information class.
//  *
//  * @param[in] TokenInformation
//  * An arbitrary pointer to a buffer with token information to set. Such
//  * arbitrary buffer depends on the information class chosen that the caller
//  * wants to modify such information data of a token.
//  *
//  * @param[in] TokenInformationLength
//  * Length of the token information buffer, in bytes.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if information setting has completed successfully.
//  * STATUS_INFO_LENGTH_MISMATCH is returned if the information length of the
//  * buffer is less than the required length. STATUS_INSUFFICIENT_RESOURCES is
//  * returned if memory pool allocation has failed. STATUS_PRIVILEGE_NOT_HELD
//  * is returned if the calling thread hasn't the required privileges to perform
//  * the operation in question. A failure NTSTATUS code is returned otherwise.
//  *
//  * @remarks
//  * The function is partly implemented, mainly TokenOrigin.
//  */
// _Must_inspect_result_
// __kernel_entry
// NTSTATUS
// NTAPI
// NtSetInformationToken(
//     _In_ HANDLE TokenHandle,
//     _In_ TOKEN_INFORMATION_CLASS TokenInformationClass,
//     _In_reads_bytes_(TokenInformationLength) PVOID TokenInformation,
//     _In_ ULONG TokenInformationLength)
// {
//     NTSTATUS Status;
//     PTOKEN Token;
//     KPROCESSOR_MODE PreviousMode;
//     ULONG NeededAccess = TOKEN_ADJUST_DEFAULT;
//
//     PAGED_CODE();
//
//     PreviousMode = ExGetPreviousMode();
//
//     Status = DefaultSetInfoBufferCheck(TokenInformationClass,
//                                        SeTokenInformationClass,
//                                        RTL_NUMBER_OF(SeTokenInformationClass),
//                                        TokenInformation,
//                                        TokenInformationLength,
//                                        PreviousMode);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Invalid buffers */
//         DPRINT("NtSetInformationToken() failed, Status: 0x%x\n", Status);
//         return Status;
//     }
//
//     if (TokenInformationClass == TokenSessionId)
//     {
//         NeededAccess |= TOKEN_ADJUST_SESSIONID;
//     }
//
//     Status = ObReferenceObjectByHandle(TokenHandle,
//                                        NeededAccess,
//                                        SeTokenObjectType as *mut _OBJECT_TYPE,
//                                        PreviousMode,
//                                        (PVOID*)&Token,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         switch (TokenInformationClass)
//         {
//             case TokenOwner:
//             {
//                 if (TokenInformationLength >= sizeof(TOKEN_OWNER))
//                 {
//                     PTOKEN_OWNER to = (PTOKEN_OWNER)TokenInformation;
//                     PSID InputSid = NULL, CapturedSid;
//                     ULONG DefaultOwnerIndex;
//
//                     _SEH2_TRY
//                     {
//                         InputSid = to->Owner;
//                     }
//                     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                     {
//                         Status = _SEH2_GetExceptionCode();
//                         _SEH2_YIELD(goto Cleanup);
//                     }
//                     _SEH2_END;
//
//                     Status = SepCaptureSid(InputSid,
//                                            PreviousMode,
//                                            PagedPool,
//                                            FALSE,
//                                            &CapturedSid);
//                     if (NT_SUCCESS(Status))
//                     {
//                         /* Lock the token */
//                         SepAcquireTokenLockExclusive(Token);
//
//                         /* Find the owner amongst the existing token user and groups */
//                         Status = SepFindPrimaryGroupAndDefaultOwner(Token,
//                                                                     NULL,
//                                                                     CapturedSid,
//                                                                     NULL,
//                                                                     &DefaultOwnerIndex);
//                         if (NT_SUCCESS(Status))
//                         {
//                             /* Found it */
//                             Token->DefaultOwnerIndex = DefaultOwnerIndex;
//                             ExAllocateLocallyUniqueId(&Token->ModifiedId);
//                         }
//
//                         /* Unlock the token */
//                         SepReleaseTokenLock(Token);
//
//                         SepReleaseSid(CapturedSid,
//                                       PreviousMode,
//                                       FALSE);
//                     }
//                 }
//                 else
//                 {
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                 }
//                 break;
//             }
//
//             case TokenPrimaryGroup:
//             {
//                 if (TokenInformationLength >= sizeof(TOKEN_PRIMARY_GROUP))
//                 {
//                     PTOKEN_PRIMARY_GROUP tpg = (PTOKEN_PRIMARY_GROUP)TokenInformation;
//                     ULONG AclSize;
//                     ULONG_PTR PrimaryGroup;
//                     PSID InputSid = NULL, CapturedSid;
//                     ULONG PrimaryGroupIndex, NewDynamicLength;
//
//                     _SEH2_TRY
//                     {
//                         InputSid = tpg->PrimaryGroup;
//                     }
//                     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                     {
//                         Status = _SEH2_GetExceptionCode();
//                         _SEH2_YIELD(goto Cleanup);
//                     }
//                     _SEH2_END;
//
//                     Status = SepCaptureSid(InputSid,
//                                            PreviousMode,
//                                            PagedPool,
//                                            FALSE,
//                                            &CapturedSid);
//                     if (NT_SUCCESS(Status))
//                     {
//                         /* Lock the token */
//                         SepAcquireTokenLockExclusive(Token);
//
//                         /*
//                          * We can whack the token's primary group only if
//                          * the charged dynamic space boundary allows us
//                          * to do so. Exceeding this boundary and we're
//                          * busted out.
//                          */
//                         AclSize = Token->DefaultDacl ? Token->DefaultDacl->AclSize : 0;
//                         NewDynamicLength = RtlLengthSid(CapturedSid) + AclSize;
//                         if (NewDynamicLength > Token->DynamicCharged)
//                         {
//                             SepReleaseTokenLock(Token);
//                             SepReleaseSid(CapturedSid, PreviousMode, FALSE);
//                             Status = STATUS_ALLOTTED_SPACE_EXCEEDED;
//                             DPRINT1("NtSetInformationToken(): Couldn't assign new primary group, space exceeded (current length %u, new length %lu)\n",
//                                     Token->DynamicCharged, NewDynamicLength);
//                             goto Cleanup;
//                         }
//
//                         /*
//                          * The dynamic part of the token may require a rebuild
//                          * if the current dynamic area is too small. If not then
//                          * we're pretty much good as is.
//                          */
//                         Status = SepRebuildDynamicPartOfToken(Token, NewDynamicLength);
//                         if (NT_SUCCESS(Status))
//                         {
//                             /* Find the primary group amongst the existing token user and groups */
//                             Status = SepFindPrimaryGroupAndDefaultOwner(Token,
//                                                                         CapturedSid,
//                                                                         NULL,
//                                                                         &PrimaryGroupIndex,
//                                                                         NULL);
//                             if (NT_SUCCESS(Status))
//                             {
//                                 /*
//                                  * We have found it. Add the length of
//                                  * the previous primary group SID to the
//                                  * available dynamic area.
//                                  */
//                                 Token->DynamicAvailable += RtlLengthSid(Token->PrimaryGroup);
//
//                                 /*
//                                  * Move the default DACL if it's not at the
//                                  * head of the dynamic part.
//                                  */
//                                 if ((Token->DefaultDacl) &&
//                                     ((PULONG)(Token->DefaultDacl) != Token->DynamicPart))
//                                 {
//                                     RtlMoveMemory(Token->DynamicPart,
//                                                   Token->DefaultDacl,
//                                                   RtlLengthSid(Token->PrimaryGroup));
//                                     Token->DefaultDacl = (PACL)(Token->DynamicPart);
//                                 }
//
//                                 /* Take away available space from the dynamic area */
//                                 Token->DynamicAvailable -= RtlLengthSid(Token->UserAndGroups[PrimaryGroupIndex].Sid);
//
//                                 /*
//                                  * And assign the new primary group. For that
//                                  * we have to make sure where the primary group
//                                  * is going to stay in memory, so if this token
//                                  * has a default DACL then add up its size with
//                                  * the address of the dynamic part.
//                                  */
//                                 PrimaryGroup = (ULONG_PTR)(Token->DynamicPart) + AclSize;
//                                 RtlCopySid(RtlLengthSid(Token->UserAndGroups[PrimaryGroupIndex].Sid),
//                                            (PVOID)PrimaryGroup,
//                                            Token->UserAndGroups[PrimaryGroupIndex].Sid);
//                                 Token->PrimaryGroup = (PSID)PrimaryGroup;
//
//                                 ExAllocateLocallyUniqueId(&Token->ModifiedId);
//                             }
//                         }
//
//                         /* Unlock the token */
//                         SepReleaseTokenLock(Token);
//
//                         SepReleaseSid(CapturedSid,
//                                       PreviousMode,
//                                       FALSE);
//                     }
//                 }
//                 else
//                 {
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                 }
//                 break;
//             }
//
//             case TokenDefaultDacl:
//             {
//                 if (TokenInformationLength >= sizeof(TOKEN_DEFAULT_DACL))
//                 {
//                     PTOKEN_DEFAULT_DACL tdd = (PTOKEN_DEFAULT_DACL)TokenInformation;
//                     PACL InputAcl = NULL;
//
//                     _SEH2_TRY
//                     {
//                         InputAcl = tdd->DefaultDacl;
//                     }
//                     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                     {
//                         Status = _SEH2_GetExceptionCode();
//                         _SEH2_YIELD(goto Cleanup);
//                     }
//                     _SEH2_END;
//
//                     if (InputAcl != NULL)
//                     {
//                         PACL CapturedAcl;
//
//                         /* Capture, validate, and copy the DACL */
//                         Status = SepCaptureAcl(InputAcl,
//                                                PreviousMode,
//                                                PagedPool,
//                                                TRUE,
//                                                &CapturedAcl);
//                         if (NT_SUCCESS(Status))
//                         {
//                             ULONG NewDynamicLength;
//                             ULONG_PTR Acl;
//
//                             /* Lock the token */
//                             SepAcquireTokenLockExclusive(Token);
//
//                             /*
//                              * We can whack the token's default DACL only if
//                              * the charged dynamic space boundary allows us
//                              * to do so. Exceeding this boundary and we're
//                              * busted out.
//                              */
//                             NewDynamicLength = CapturedAcl->AclSize + RtlLengthSid(Token->PrimaryGroup);
//                             if (NewDynamicLength > Token->DynamicCharged)
//                             {
//                                 SepReleaseTokenLock(Token);
//                                 SepReleaseAcl(CapturedAcl, PreviousMode, TRUE);
//                                 Status = STATUS_ALLOTTED_SPACE_EXCEEDED;
//                                 DPRINT1("NtSetInformationToken(): Couldn't assign new default DACL, space exceeded (current length %u, new length %lu)\n",
//                                         Token->DynamicCharged, NewDynamicLength);
//                                 goto Cleanup;
//                             }
//
//                             /*
//                              * The dynamic part of the token may require a rebuild
//                              * if the current dynamic area is too small. If not then
//                              * we're pretty much good as is.
//                              */
//                             Status = SepRebuildDynamicPartOfToken(Token, NewDynamicLength);
//                             if (NT_SUCCESS(Status))
//                             {
//                                 /*
//                                  * Before setting up a new DACL for the
//                                  * token object we add up the size of
//                                  * the old DACL to the available dynamic
//                                  * area
//                                  */
//                                 if (Token->DefaultDacl)
//                                 {
//                                     Token->DynamicAvailable += Token->DefaultDacl->AclSize;
//                                 }
//
//                                 /*
//                                  * Move the primary group if it's not at the
//                                  * head of the dynamic part.
//                                  */
//                                 if ((PULONG)(Token->PrimaryGroup) != Token->DynamicPart)
//                                 {
//                                     RtlMoveMemory(Token->DynamicPart,
//                                                   Token->PrimaryGroup,
//                                                   RtlLengthSid(Token->PrimaryGroup));
//                                     Token->PrimaryGroup = (PSID)(Token->DynamicPart);
//                                 }
//
//                                 /* Take away available space from the dynamic area */
//                                 Token->DynamicAvailable -= CapturedAcl->AclSize;
//
//                                 /* Set the new dacl */
//                                 Acl = (ULONG_PTR)(Token->DynamicPart) + RtlLengthSid(Token->PrimaryGroup);
//                                 RtlCopyMemory((PVOID)Acl,
//                                               CapturedAcl,
//                                               CapturedAcl->AclSize);
//                                 Token->DefaultDacl = (PACL)Acl;
//
//                                 ExAllocateLocallyUniqueId(&Token->ModifiedId);
//                             }
//
//                             /* Unlock the token and release the ACL */
//                             SepReleaseTokenLock(Token);
//                             SepReleaseAcl(CapturedAcl, PreviousMode, TRUE);
//                         }
//                     }
//                     else
//                     {
//                         /* Lock the token */
//                         SepAcquireTokenLockExclusive(Token);
//
//                         /* Clear the default dacl if present */
//                         if (Token->DefaultDacl != NULL)
//                         {
//                             Token->DynamicAvailable += Token->DefaultDacl->AclSize;
//                             RtlZeroMemory(Token->DefaultDacl, Token->DefaultDacl->AclSize);
//                             Token->DefaultDacl = NULL;
//
//                             ExAllocateLocallyUniqueId(&Token->ModifiedId);
//                         }
//
//                         /* Unlock the token */
//                         SepReleaseTokenLock(Token);
//                     }
//                 }
//                 else
//                 {
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                 }
//                 break;
//             }
//
//             case TokenSessionId:
//             {
//                 ULONG SessionId = 0;
//
//                 _SEH2_TRY
//                 {
//                     /* Buffer size was already verified, no need to check here again */
//                     SessionId = *(PULONG)TokenInformation;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                     _SEH2_YIELD(goto Cleanup);
//                 }
//                 _SEH2_END;
//
//                 /* Check for TCB privilege */
//                 if (!SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode))
//                 {
//                     Status = STATUS_PRIVILEGE_NOT_HELD;
//                     break;
//                 }
//
//                 /* Lock the token */
//                 SepAcquireTokenLockExclusive(Token);
//
//                 Token->SessionId = SessionId;
//                 ExAllocateLocallyUniqueId(&Token->ModifiedId);
//
//                 /* Unlock the token */
//                 SepReleaseTokenLock(Token);
//
//                 break;
//             }
//
//             case TokenSessionReference:
//             {
//                 ULONG SessionReference;
//
//                 _SEH2_TRY
//                 {
//                     /* Buffer size was already verified, no need to check here again */
//                     SessionReference = *(PULONG)TokenInformation;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                     _SEH2_YIELD(goto Cleanup);
//                 }
//                 _SEH2_END;
//
//                 /* Check for TCB privilege */
//                 if (!SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode))
//                 {
//                     Status = STATUS_PRIVILEGE_NOT_HELD;
//                     goto Cleanup;
//                 }
//
//                 /* Check if it is 0 */
//                 if (SessionReference == 0)
//                 {
//                     ULONG OldTokenFlags;
//
//                     /* Lock the token */
//                     SepAcquireTokenLockExclusive(Token);
//
//                     /* Atomically set the flag in the token */
//                     OldTokenFlags = RtlInterlockedSetBits(&Token->TokenFlags,
//                                                           TOKEN_SESSION_NOT_REFERENCED);
//                     /*
//                      * If the flag was already set, do not dereference again
//                      * the logon session. Use SessionReference as an indicator
//                      * to know whether to really dereference the session.
//                      */
//                     if (OldTokenFlags == Token->TokenFlags)
//                         SessionReference = ULONG_MAX;
//
//                     /*
//                      * Otherwise if the flag was never set but just for this first time then
//                      * remove the referenced logon session data from the token and dereference
//                      * the logon session when needed.
//                      */
//                     if (SessionReference == 0)
//                     {
//                         SepRmRemoveLogonSessionFromToken(Token);
//                         SepRmDereferenceLogonSession(&Token->AuthenticationId);
//                     }
//
//                     /* Unlock the token */
//                     SepReleaseTokenLock(Token);
//                 }
//                 break;
//             }
//
//             case TokenAuditPolicy:
//             {
//                 PTOKEN_AUDIT_POLICY_INFORMATION PolicyInformation =
//                     (PTOKEN_AUDIT_POLICY_INFORMATION)TokenInformation;
//                 SEP_AUDIT_POLICY AuditPolicy;
//                 ULONG i;
//
//                 _SEH2_TRY
//                 {
//                     ProbeForRead(PolicyInformation,
//                                  FIELD_OFFSET(TOKEN_AUDIT_POLICY_INFORMATION,
//                                               Policies[PolicyInformation->PolicyCount]),
//                                  sizeof(ULONG));
//
//                     /* Loop all policies in the structure */
//                     for (i = 0; i < PolicyInformation->PolicyCount; i++)
//                     {
//                         /* Set the corresponding bits in the packed structure */
//                         switch (PolicyInformation->Policies[i].Category)
//                         {
//                             case AuditCategorySystem:
//                                 AuditPolicy.PolicyElements.System = PolicyInformation->Policies[i].Value;
//                                 break;
//
//                             case AuditCategoryLogon:
//                                 AuditPolicy.PolicyElements.Logon = PolicyInformation->Policies[i].Value;
//                                 break;
//
//                             case AuditCategoryObjectAccess:
//                                 AuditPolicy.PolicyElements.ObjectAccess = PolicyInformation->Policies[i].Value;
//                                 break;
//
//                             case AuditCategoryPrivilegeUse:
//                                 AuditPolicy.PolicyElements.PrivilegeUse = PolicyInformation->Policies[i].Value;
//                                 break;
//
//                             case AuditCategoryDetailedTracking:
//                                 AuditPolicy.PolicyElements.DetailedTracking = PolicyInformation->Policies[i].Value;
//                                 break;
//
//                             case AuditCategoryPolicyChange:
//                                 AuditPolicy.PolicyElements.PolicyChange = PolicyInformation->Policies[i].Value;
//                                 break;
//
//                             case AuditCategoryAccountManagement:
//                                 AuditPolicy.PolicyElements.AccountManagement = PolicyInformation->Policies[i].Value;
//                                 break;
//
//                             case AuditCategoryDirectoryServiceAccess:
//                                 AuditPolicy.PolicyElements.DirectoryServiceAccess = PolicyInformation->Policies[i].Value;
//                                 break;
//
//                             case AuditCategoryAccountLogon:
//                                 AuditPolicy.PolicyElements.AccountLogon = PolicyInformation->Policies[i].Value;
//                                 break;
//                         }
//                     }
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                     _SEH2_YIELD(goto Cleanup);
//                 }
//                 _SEH2_END;
//
//                 /* Check for TCB privilege */
//                 if (!SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode))
//                 {
//                     Status = STATUS_PRIVILEGE_NOT_HELD;
//                     break;
//                 }
//
//                 /* Lock the token */
//                 SepAcquireTokenLockExclusive(Token);
//
//                 /* Set the new audit policy */
//                 Token->AuditPolicy = AuditPolicy;
//                 ExAllocateLocallyUniqueId(&Token->ModifiedId);
//
//                 /* Unlock the token */
//                 SepReleaseTokenLock(Token);
//
//                 break;
//             }
//
//             case TokenOrigin:
//             {
//                 TOKEN_ORIGIN TokenOrigin;
//
//                 _SEH2_TRY
//                 {
//                     /* Copy the token origin */
//                     TokenOrigin = *(PTOKEN_ORIGIN)TokenInformation;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                     _SEH2_YIELD(goto Cleanup);
//                 }
//                 _SEH2_END;
//
//                 /* Check for TCB privilege */
//                 if (!SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode))
//                 {
//                     Status = STATUS_PRIVILEGE_NOT_HELD;
//                     break;
//                 }
//
//                 /* Lock the token */
//                 SepAcquireTokenLockExclusive(Token);
//
//                 /* Check if there is no token origin set yet */
//                 if (RtlIsZeroLuid(&Token->OriginatingLogonSession))
//                 {
//                     /* Set the token origin */
//                     Token->OriginatingLogonSession =
//                         TokenOrigin.OriginatingLogonSession;
//
//                     ExAllocateLocallyUniqueId(&Token->ModifiedId);
//                 }
//
//                 /* Unlock the token */
//                 SepReleaseTokenLock(Token);
//
//                 break;
//             }
//
//             default:
//             {
//                 DPRINT1("Invalid TokenInformationClass: 0x%lx\n",
//                         TokenInformationClass);
//                 Status = STATUS_INVALID_INFO_CLASS;
//                 break;
//             }
//         }
// Cleanup:
//         ObDereferenceObject(Token);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("NtSetInformationToken failed with Status 0x%lx\n", Status);
//     }
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
