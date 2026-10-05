use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, TRUE, FALSE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::PagedPool;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    pub static mut SeWorldSid: PSID;
    pub static mut SeLocalSystemSid: PSID;
    pub static mut SeAliasAdminsSid: PSID;
    pub static mut SeRestrictedCodeSid: PSID;
    pub static mut SeAnonymousLogonSid: PSID;
    pub static mut SeCreatorOwnerSid: PSID;
    pub static mut SeCreatorGroupSid: PSID;
}

const TAG_ACL: u32 = 0x63416553;

const ACL_REVISION: UCHAR = 2;
const ACCESS_MAX_MS_V2_ACE_TYPE: UCHAR = 0x03;
const ACCESS_ALLOWED_ACE_TYPE: UCHAR = 0x00;
const ACCESS_DENIED_ACE_TYPE: UCHAR = 0x01;
const SYSTEM_AUDIT_ACE_TYPE: UCHAR = 0x02;
const SYSTEM_ALARM_ACE_TYPE: UCHAR = 0x03;

const OBJECT_INHERIT_ACE: UCHAR = 0x01;
const CONTAINER_INHERIT_ACE: UCHAR = 0x02;
const NO_PROPAGATE_INHERIT_ACE: UCHAR = 0x04;
const INHERIT_ONLY_ACE: UCHAR = 0x08;
const VALID_INHERIT_FLAGS: UCHAR = 0x1F;

const GENERIC_EXECUTE: ACCESS_MASK = 0x20000000;
const GENERIC_ALL: ACCESS_MASK = 0x10000000;
const GENERIC_READ: ACCESS_MASK = 0x80000000;
const GENERIC_WRITE: ACCESS_MASK = 0x40000000;
const READ_CONTROL: ACCESS_MASK = 0x00020000;

const STATUS_INVALID_ACL: NTSTATUS = 0xC0000077u32 as NTSTATUS;
const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC0000023u32 as NTSTATUS;

#[no_mangle]
pub static mut SePublicDefaultDacl: PACL = core::ptr::null_mut();

#[no_mangle]
pub static mut SeSystemDefaultDacl: PACL = core::ptr::null_mut();

#[no_mangle]
pub static mut SePublicDefaultUnrestrictedDacl: PACL = core::ptr::null_mut();

#[no_mangle]
pub static mut SePublicOpenDacl: PACL = core::ptr::null_mut();

#[no_mangle]
pub static mut SePublicOpenUnrestrictedDacl: PACL = core::ptr::null_mut();

#[no_mangle]
pub static mut SeUnrestrictedDacl: PACL = core::ptr::null_mut();

#[no_mangle]
pub static mut SeSystemAnonymousLogonDacl: PACL = core::ptr::null_mut();

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn SepInitDACLs() -> BOOLEAN {
    let mut acl_length: ULONG;

    acl_length = core::mem::size_of::<ACL>() as ULONG +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeWorldSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeLocalSystemSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeAliasAdminsSid));

    SePublicDefaultDacl = ExAllocatePoolWithTag(PagedPool, acl_length as SIZE_T, TAG_ACL) as PACL;
    if SePublicDefaultDacl.is_null() {
        return 0u8;
    }

    RtlCreateAcl(SePublicDefaultDacl, acl_length, ACL_REVISION as ULONG);

    RtlAddAccessAllowedAce(SePublicDefaultDacl, ACL_REVISION as ULONG, GENERIC_EXECUTE, SeWorldSid);
    RtlAddAccessAllowedAce(SePublicDefaultDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeLocalSystemSid);
    RtlAddAccessAllowedAce(SePublicDefaultDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeAliasAdminsSid);

    acl_length = core::mem::size_of::<ACL>() as ULONG +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeWorldSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeLocalSystemSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeAliasAdminsSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeRestrictedCodeSid));

    SePublicDefaultUnrestrictedDacl = ExAllocatePoolWithTag(PagedPool, acl_length as SIZE_T, TAG_ACL) as PACL;
    if SePublicDefaultUnrestrictedDacl.is_null() {
        return 0u8;
    }

    RtlCreateAcl(SePublicDefaultUnrestrictedDacl, acl_length, ACL_REVISION as ULONG);

    RtlAddAccessAllowedAce(SePublicDefaultUnrestrictedDacl, ACL_REVISION as ULONG, GENERIC_EXECUTE, SeWorldSid);
    RtlAddAccessAllowedAce(SePublicDefaultUnrestrictedDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeLocalSystemSid);
    RtlAddAccessAllowedAce(SePublicDefaultUnrestrictedDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeAliasAdminsSid);
    RtlAddAccessAllowedAce(SePublicDefaultUnrestrictedDacl, ACL_REVISION as ULONG, GENERIC_READ | GENERIC_EXECUTE | READ_CONTROL, SeRestrictedCodeSid);

    acl_length = core::mem::size_of::<ACL>() as ULONG +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeWorldSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeLocalSystemSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeAliasAdminsSid));

    SePublicOpenDacl = ExAllocatePoolWithTag(PagedPool, acl_length as SIZE_T, TAG_ACL) as PACL;
    if SePublicOpenDacl.is_null() {
        return 0u8;
    }

    RtlCreateAcl(SePublicOpenDacl, acl_length, ACL_REVISION as ULONG);

    RtlAddAccessAllowedAce(SePublicOpenDacl, ACL_REVISION as ULONG, GENERIC_READ | GENERIC_WRITE | GENERIC_EXECUTE, SeWorldSid);
    RtlAddAccessAllowedAce(SePublicOpenDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeLocalSystemSid);
    RtlAddAccessAllowedAce(SePublicOpenDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeAliasAdminsSid);

    acl_length = core::mem::size_of::<ACL>() as ULONG +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeWorldSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeLocalSystemSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeAliasAdminsSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeRestrictedCodeSid));

    SePublicOpenUnrestrictedDacl = ExAllocatePoolWithTag(PagedPool, acl_length as SIZE_T, TAG_ACL) as PACL;
    if SePublicOpenUnrestrictedDacl.is_null() {
        return 0u8;
    }

    RtlCreateAcl(SePublicOpenUnrestrictedDacl, acl_length, ACL_REVISION as ULONG);

    RtlAddAccessAllowedAce(SePublicOpenUnrestrictedDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeWorldSid);
    RtlAddAccessAllowedAce(SePublicOpenUnrestrictedDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeLocalSystemSid);
    RtlAddAccessAllowedAce(SePublicOpenUnrestrictedDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeAliasAdminsSid);
    RtlAddAccessAllowedAce(SePublicOpenUnrestrictedDacl, ACL_REVISION as ULONG, GENERIC_READ | GENERIC_EXECUTE, SeRestrictedCodeSid);

    acl_length = core::mem::size_of::<ACL>() as ULONG +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeLocalSystemSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeAliasAdminsSid));

    SeSystemDefaultDacl = ExAllocatePoolWithTag(PagedPool, acl_length as SIZE_T, TAG_ACL) as PACL;
    if SeSystemDefaultDacl.is_null() {
        return 0u8;
    }

    RtlCreateAcl(SeSystemDefaultDacl, acl_length, ACL_REVISION as ULONG);

    RtlAddAccessAllowedAce(SeSystemDefaultDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeLocalSystemSid);
    RtlAddAccessAllowedAce(SeSystemDefaultDacl, ACL_REVISION as ULONG, GENERIC_READ | GENERIC_EXECUTE | READ_CONTROL, SeAliasAdminsSid);

    acl_length = core::mem::size_of::<ACL>() as ULONG +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeWorldSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeRestrictedCodeSid));

    SeUnrestrictedDacl = ExAllocatePoolWithTag(PagedPool, acl_length as SIZE_T, TAG_ACL) as PACL;
    if SeUnrestrictedDacl.is_null() {
        return 0u8;
    }

    RtlCreateAcl(SeUnrestrictedDacl, acl_length, ACL_REVISION as ULONG);

    RtlAddAccessAllowedAce(SeUnrestrictedDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeWorldSid);
    RtlAddAccessAllowedAce(SeUnrestrictedDacl, ACL_REVISION as ULONG, GENERIC_READ | GENERIC_EXECUTE, SeRestrictedCodeSid);

    acl_length = core::mem::size_of::<ACL>() as ULONG +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeWorldSid)) +
                 (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeAnonymousLogonSid));

    SeSystemAnonymousLogonDacl = ExAllocatePoolWithTag(PagedPool, acl_length as SIZE_T, TAG_ACL) as PACL;
    if SeSystemAnonymousLogonDacl.is_null() {
        return 0u8;
    }

    RtlCreateAcl(SeSystemAnonymousLogonDacl, acl_length, ACL_REVISION as ULONG);

    RtlAddAccessAllowedAce(SeSystemAnonymousLogonDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeWorldSid);
    RtlAddAccessAllowedAce(SeSystemAnonymousLogonDacl, ACL_REVISION as ULONG, GENERIC_ALL, SeAnonymousLogonSid);

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn SepCreateImpersonationTokenDacl(
    Token: PTOKEN,
    PrimaryToken: PTOKEN,
    Dacl: *mut PACL,
) -> NTSTATUS {
    let Token = opaque_ptr(Token);
    let PrimaryToken = opaque_ptr(PrimaryToken);
    let Dacl = opaque_ptr(Dacl);
    let mut acl_length: ULONG;
    let token_dacl: PACL;

    *Dacl = core::ptr::null_mut();

    acl_length = core::mem::size_of::<ACL>() as ULONG +
        (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeAliasAdminsSid)) +
        (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeLocalSystemSid)) +
        (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid(SeRestrictedCodeSid)) +
        (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid((*(*Token).UserAndGroups).Sid)) +
        (core::mem::size_of::<ACE>() as ULONG + RtlLengthSid((*(*PrimaryToken).UserAndGroups).Sid));

    token_dacl = ExAllocatePoolWithTag(PagedPool, acl_length as SIZE_T, TAG_ACL) as PACL;
    if token_dacl.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCreateAcl(token_dacl, acl_length, ACL_REVISION as ULONG);
    RtlAddAccessAllowedAce(token_dacl, ACL_REVISION as ULONG, GENERIC_ALL, (*(*Token).UserAndGroups).Sid);
    RtlAddAccessAllowedAce(token_dacl, ACL_REVISION as ULONG, GENERIC_ALL, (*(*PrimaryToken).UserAndGroups).Sid);
    RtlAddAccessAllowedAce(token_dacl, ACL_REVISION as ULONG, GENERIC_ALL, SeAliasAdminsSid);
    RtlAddAccessAllowedAce(token_dacl, ACL_REVISION as ULONG, GENERIC_ALL, SeLocalSystemSid);

    if !(*Token).RestrictedSids.is_null() || !(*PrimaryToken).RestrictedSids.is_null() {
        RtlAddAccessAllowedAce(token_dacl, ACL_REVISION as ULONG, GENERIC_ALL, SeRestrictedCodeSid);
    }

    *Dacl = token_dacl;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SepCaptureAcl(
    InputAcl: PACL,
    AccessMode: KPROCESSOR_MODE,
    PoolType: POOL_TYPE,
    CaptureIfKernel: BOOLEAN,
    CapturedAcl: *mut PACL,
) -> NTSTATUS {
    let InputAcl = opaque_ptr(InputAcl);
    let CapturedAcl = opaque_ptr(CapturedAcl);
    let new_acl: PACL;
    let mut acl_size: ULONG;

    if (AccessMode == 0i8) && CaptureIfKernel == 0u8 {
        *CapturedAcl = InputAcl;
        return STATUS_SUCCESS;
    }

    if AccessMode != 0i8 {
        // First probe: read the ACL header to get AclSize
        if let Err(s) = probe_user_buffer(InputAcl as _, core::mem::size_of::<ACL>(), core::mem::size_of::<ULONG>(), AccessMode) {
            return s;
        }
        // AclSize lives in user memory: read it fault-tolerantly (the C reads it
        // inside _SEH2_TRY).
        let mut user_acl_size: USHORT = 0;
        if let Err(s) = safe_copy_from_user(
            core::ptr::addr_of_mut!(user_acl_size) as PVOID,
            core::ptr::addr_of!((*InputAcl).AclSize) as _,
            core::mem::size_of::<USHORT>(),
            1,
            AccessMode,
        ) {
            return s;
        }
        acl_size = user_acl_size as ULONG;

        // Second probe: validate the full ACL size
        if let Err(s) = probe_user_buffer(InputAcl as _, acl_size as usize, core::mem::size_of::<ULONG>(), AccessMode) {
            return s;
        }

        if acl_size < core::mem::size_of::<ACL>() as ULONG {
            return STATUS_INVALID_ACL;
        }

        new_acl = ExAllocatePoolWithTag(PoolType, acl_size as SIZE_T, TAG_ACL) as PACL;
        if new_acl.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        let _guard = PoolAllocGuard::new(new_acl as PVOID, TAG_ACL);
        
        // Copy from user-mode to kernel buffer
        if let Err(s) = safe_copy_from_user(new_acl as PVOID, InputAcl as _, acl_size as usize, core::mem::size_of::<ULONG>(), AccessMode) {
            return s;
        }
        // The caller can grow AclSize between the read above and the copy, and
        // RtlValidAcl walks the copy by that field. Pin it to the size allocated.
        (*new_acl).AclSize = user_acl_size;

        _guard.disarm();  // Success — keep the allocation
    } else {
        acl_size = (*InputAcl).AclSize as ULONG;

        if acl_size < core::mem::size_of::<ACL>() as ULONG {
            return STATUS_INVALID_ACL;
        }

        new_acl = ExAllocatePoolWithTag(PoolType, acl_size as SIZE_T, TAG_ACL) as PACL;
        if new_acl.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::copy_nonoverlapping(InputAcl as *const u8, new_acl as *mut u8, acl_size as usize);
    }

    if RtlValidAcl(new_acl) == 0u8 {
        ExFreePoolWithTag(new_acl as PVOID, TAG_ACL);
        return STATUS_INVALID_ACL;
    }

    *CapturedAcl = new_acl;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SepReleaseAcl(
    CapturedAcl: PACL,
    AccessMode: KPROCESSOR_MODE,
    CaptureIfKernel: BOOLEAN,
) {
    let CapturedAcl = opaque_ptr(CapturedAcl);
    if !CapturedAcl.is_null() &&
        (AccessMode != 0i8 || (AccessMode == 0i8 && CaptureIfKernel != 0u8))
    {
        ExFreePoolWithTag(CapturedAcl as PVOID, TAG_ACL);
    }
}

unsafe fn SepShouldPropagateAce(
    AceFlags: UCHAR,
    NewAceFlags: *mut UCHAR,
    IsInherited: BOOLEAN,
    IsDirectoryObject: BOOLEAN,
) -> BOOLEAN {
    if IsInherited == 0u8 {
        *NewAceFlags = AceFlags;
        return 1u8;
    }

    if IsDirectoryObject == 0u8 {
        if (AceFlags & OBJECT_INHERIT_ACE) != 0 {
            *NewAceFlags = AceFlags & !VALID_INHERIT_FLAGS;
            return 1u8;
        }
        return 0u8;
    }

    if (AceFlags & NO_PROPAGATE_INHERIT_ACE) != 0 {
        if (AceFlags & CONTAINER_INHERIT_ACE) != 0 {
            *NewAceFlags = AceFlags & !VALID_INHERIT_FLAGS;
            return 1u8;
        }
        return 0u8;
    }

    if (AceFlags & CONTAINER_INHERIT_ACE) != 0 {
        *NewAceFlags = CONTAINER_INHERIT_ACE | (AceFlags & OBJECT_INHERIT_ACE) | (AceFlags & !VALID_INHERIT_FLAGS);
        return 1u8;
    }

    if (AceFlags & OBJECT_INHERIT_ACE) != 0 {
        *NewAceFlags = INHERIT_ONLY_ACE | OBJECT_INHERIT_ACE | (AceFlags & !VALID_INHERIT_FLAGS);
        return 1u8;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "C" fn SepPropagateAcl(
    AclDest: PACL,
    AclLength: *mut ULONG,
    AclSource: PACL,
    Owner: PSID,
    Group: PSID,
    IsInherited: BOOLEAN,
    IsDirectoryObject: BOOLEAN,
    GenericMapping: PGENERIC_MAPPING,
) -> NTSTATUS {
    let AclDest = opaque_ptr(AclDest);
    let AclLength = opaque_ptr(AclLength);
    let AclSource = opaque_ptr(AclSource);
    let Owner = opaque_ptr(Owner);
    let Group = opaque_ptr(Group);
    let GenericMapping = opaque_ptr(GenericMapping);
    let mut mask: ACCESS_MASK;
    let mut ace_source: PACCESS_ALLOWED_ACE;
    let mut ace_dest: PACCESS_ALLOWED_ACE;
    let mut current_dest: *mut UCHAR;
    let mut current_source: *mut UCHAR;
    let mut i: ULONG;
    let mut written: ULONG;
    let mut ace_flags: UCHAR = 0;
    let mut ace_size: USHORT;
    let mut ace_count: USHORT = 0;
    let mut sid: PSID;
    let mut write_two_aces: BOOLEAN;

    written = 0;
    if *AclLength >= written + core::mem::size_of::<ACL>() as ULONG {
        core::ptr::copy_nonoverlapping(AclSource as *const u8, AclDest as *mut u8, core::mem::size_of::<ACL>());
    }
    written += core::mem::size_of::<ACL>() as ULONG;

    current_dest = (AclDest as *mut ACL).add(1) as *mut UCHAR;
    current_source = (AclSource as *mut ACL).add(1) as *mut UCHAR;
    for i in 0..(*AclSource).AceCount as ULONG {
        ace_dest = current_dest as PACCESS_ALLOWED_ACE;
        ace_source = current_source as PACCESS_ALLOWED_ACE;

        if (*ace_source).Header.AceType > ACCESS_MAX_MS_V2_ACE_TYPE {
            ace_size = (*ace_source).Header.AceSize;

            if *AclLength >= written + ace_size as ULONG {
                core::ptr::copy_nonoverlapping(ace_source as *const u8, ace_dest as *mut u8, ace_size as usize);
            }
            current_dest = current_dest.add(ace_size as usize);
            current_source = current_source.add(ace_size as usize);
            written += ace_size as ULONG;
            ace_count += 1;
            continue;
        }

        if SepShouldPropagateAce(
            (*ace_source).Header.AceFlags,
            &mut ace_flags,
            IsInherited,
            IsDirectoryObject,
        ) == 0u8
        {
            current_source = current_source.add((*ace_source).Header.AceSize as usize);
            continue;
        }

        ace_size = (*ace_source).Header.AceSize;
        mask = (*ace_source).Mask;
        sid = &mut (*ace_source).SidStart as *mut u32 as PSID;

        write_two_aces = 0u8;
        if (ace_flags & INHERIT_ONLY_ACE) == 0 {
            RtlMapGenericMask(&mut mask, GenericMapping);
            mask &= (*GenericMapping).GenericAll;

            if IsInherited != 0u8 {
                if RtlEqualSid(sid, SeCreatorOwnerSid) != 0u8 {
                    sid = Owner;
                } else if RtlEqualSid(sid, SeCreatorGroupSid) != 0u8 {
                    sid = Group;
                }
                let sid_start_offset = core::mem::offset_of!(ACCESS_ALLOWED_ACE, SidStart);
                ace_size = (sid_start_offset + RtlLengthSid(sid) as usize) as USHORT;

                if IsDirectoryObject != 0u8 &&
                    (ace_flags & CONTAINER_INHERIT_ACE) != 0 &&
                    (mask != (*ace_source).Mask || sid != (&mut (*ace_source).SidStart as *mut u32 as PSID))
                {
                    write_two_aces = 1u8;
                }
            }
        }

        loop {
            if *AclLength >= written + ace_size as ULONG {
                (*ace_dest).Header.AceType = (*ace_source).Header.AceType;
                (*ace_dest).Header.AceFlags = if write_two_aces != 0u8 { ace_flags & !VALID_INHERIT_FLAGS } else { ace_flags };
                (*ace_dest).Header.AceSize = ace_size;
                (*ace_dest).Mask = mask;
                let sid_start_offset = core::mem::offset_of!(ACCESS_ALLOWED_ACE, SidStart);
                RtlCopySid(
                    (ace_size as usize - sid_start_offset) as ULONG,
                    &mut (*ace_dest).SidStart as *mut u32 as PSID,
                    sid,
                );
            }
            written += ace_size as ULONG;

            ace_count += 1;
            current_dest = current_dest.add(ace_size as usize);

            if write_two_aces == 0u8 {
                break;
            }

            write_two_aces = 0u8;
            ace_dest = current_dest as PACCESS_ALLOWED_ACE;
            ace_size = (*ace_source).Header.AceSize;
            mask = (*ace_source).Mask;
            sid = &mut (*ace_source).SidStart as *mut u32 as PSID;
            ace_flags |= INHERIT_ONLY_ACE;
        }

        current_source = current_source.add((*ace_source).Header.AceSize as usize);
    }

    if *AclLength >= core::mem::size_of::<ACL>() as ULONG {
        (*AclDest).AceCount = ace_count;
        (*AclDest).AclSize = written as USHORT;
    }

    if written > *AclLength {
        *AclLength = written;
        return STATUS_BUFFER_TOO_SMALL;
    }
    *AclLength = written;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn SepSelectAcl(
    ExplicitAcl: PACL,
    ExplicitPresent: BOOLEAN,
    ExplicitDefaulted: BOOLEAN,
    ParentAcl: PACL,
    DefaultAcl: PACL,
    AclLength: *mut ULONG,
    Owner: PSID,
    Group: PSID,
    AclPresent: PBOOLEAN,
    IsInherited: *mut BOOLEAN,
    IsDirectoryObject: BOOLEAN,
    GenericMapping: PGENERIC_MAPPING,
) -> PACL {
    let ExplicitAcl = opaque_ptr(ExplicitAcl);
    let ParentAcl = opaque_ptr(ParentAcl);
    let DefaultAcl = opaque_ptr(DefaultAcl);
    let AclLength = opaque_ptr(AclLength);
    let Owner = opaque_ptr(Owner);
    let Group = opaque_ptr(Group);
    let AclPresent = opaque_ptr(AclPresent);
    let IsInherited = opaque_ptr(IsInherited);
    let GenericMapping = opaque_ptr(GenericMapping);
    let acl: PACL;
    let mut status: NTSTATUS;

    *AclPresent = 1u8;
    if ExplicitPresent != 0u8 && ExplicitDefaulted == 0u8 {
        acl = ExplicitAcl;
    } else {
        if !ParentAcl.is_null() {
            *IsInherited = 1u8;
            *AclLength = 0;
            status = SepPropagateAcl(
                core::ptr::null_mut(),
                AclLength,
                ParentAcl,
                Owner,
                Group,
                *IsInherited,
                IsDirectoryObject,
                GenericMapping,
            );

            if *AclLength != core::mem::size_of::<ACL>() as ULONG {
                return ParentAcl;
            }
        }

        if ExplicitPresent != 0u8 {
            acl = ExplicitAcl;
        } else if !DefaultAcl.is_null() {
            acl = DefaultAcl;
        } else {
            *AclPresent = 0u8;
            acl = core::ptr::null_mut();
        }
    }

    *IsInherited = 0u8;
    *AclLength = 0;
    if !acl.is_null() {
        status = SepPropagateAcl(
            core::ptr::null_mut(),
            AclLength,
            acl,
            Owner,
            Group,
            *IsInherited,
            IsDirectoryObject,
            GenericMapping,
        );
    }
    acl
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Access control lists (ACLs) implementation
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
// PACL SePublicDefaultDacl = NULL;
// PACL SeSystemDefaultDacl = NULL;
// PACL SePublicDefaultUnrestrictedDacl = NULL;
// PACL SePublicOpenDacl = NULL;
// PACL SePublicOpenUnrestrictedDacl = NULL;
// PACL SeUnrestrictedDacl = NULL;
// PACL SeSystemAnonymousLogonDacl = NULL;
//
// /* FUNCTIONS ******************************************************************/
//
// /**
//  * @brief
//  * Initializes known discretionary access control lists in the system upon
//  * kernel and Executive initialization procedure.
//  *
//  * @return
//  * Returns TRUE if all the DACLs have been successfully initialized,
//  * FALSE otherwise.
//  */
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// SepInitDACLs(VOID)
// {
//     ULONG AclLength;
//
//     /* create PublicDefaultDacl */
//     AclLength = sizeof(ACL) +
//                 (sizeof(ACE) + RtlLengthSid(SeWorldSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeLocalSystemSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeAliasAdminsSid));
//
//     SePublicDefaultDacl = ExAllocatePoolWithTag(PagedPool,
//                                                 AclLength,
//                                                 TAG_ACL);
//     if (SePublicDefaultDacl == NULL)
//         return FALSE;
//
//     RtlCreateAcl(SePublicDefaultDacl,
//                  AclLength,
//                  ACL_REVISION);
//
//     RtlAddAccessAllowedAce(SePublicDefaultDacl,
//                            ACL_REVISION,
//                            GENERIC_EXECUTE,
//                            SeWorldSid);
//
//     RtlAddAccessAllowedAce(SePublicDefaultDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeLocalSystemSid);
//
//     RtlAddAccessAllowedAce(SePublicDefaultDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeAliasAdminsSid);
//
//     /* create PublicDefaultUnrestrictedDacl */
//     AclLength = sizeof(ACL) +
//                 (sizeof(ACE) + RtlLengthSid(SeWorldSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeLocalSystemSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeAliasAdminsSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeRestrictedCodeSid));
//
//     SePublicDefaultUnrestrictedDacl = ExAllocatePoolWithTag(PagedPool,
//                                                             AclLength,
//                                                             TAG_ACL);
//     if (SePublicDefaultUnrestrictedDacl == NULL)
//         return FALSE;
//
//     RtlCreateAcl(SePublicDefaultUnrestrictedDacl,
//                  AclLength,
//                  ACL_REVISION);
//
//     RtlAddAccessAllowedAce(SePublicDefaultUnrestrictedDacl,
//                            ACL_REVISION,
//                            GENERIC_EXECUTE,
//                            SeWorldSid);
//
//     RtlAddAccessAllowedAce(SePublicDefaultUnrestrictedDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeLocalSystemSid);
//
//     RtlAddAccessAllowedAce(SePublicDefaultUnrestrictedDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeAliasAdminsSid);
//
//     RtlAddAccessAllowedAce(SePublicDefaultUnrestrictedDacl,
//                            ACL_REVISION,
//                            GENERIC_READ | GENERIC_EXECUTE | READ_CONTROL,
//                            SeRestrictedCodeSid);
//
//     /* create PublicOpenDacl */
//     AclLength = sizeof(ACL) +
//                 (sizeof(ACE) + RtlLengthSid(SeWorldSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeLocalSystemSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeAliasAdminsSid));
//
//     SePublicOpenDacl = ExAllocatePoolWithTag(PagedPool,
//                                              AclLength,
//                                              TAG_ACL);
//     if (SePublicOpenDacl == NULL)
//         return FALSE;
//
//     RtlCreateAcl(SePublicOpenDacl,
//                  AclLength,
//                  ACL_REVISION);
//
//     RtlAddAccessAllowedAce(SePublicOpenDacl,
//                            ACL_REVISION,
//                            GENERIC_READ | GENERIC_WRITE | GENERIC_EXECUTE,
//                            SeWorldSid);
//
//     RtlAddAccessAllowedAce(SePublicOpenDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeLocalSystemSid);
//
//     RtlAddAccessAllowedAce(SePublicOpenDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeAliasAdminsSid);
//
//     /* create PublicOpenUnrestrictedDacl */
//     AclLength = sizeof(ACL) +
//                 (sizeof(ACE) + RtlLengthSid(SeWorldSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeLocalSystemSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeAliasAdminsSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeRestrictedCodeSid));
//
//     SePublicOpenUnrestrictedDacl = ExAllocatePoolWithTag(PagedPool,
//                                                          AclLength,
//                                                          TAG_ACL);
//     if (SePublicOpenUnrestrictedDacl == NULL)
//         return FALSE;
//
//     RtlCreateAcl(SePublicOpenUnrestrictedDacl,
//                  AclLength,
//                  ACL_REVISION);
//
//     RtlAddAccessAllowedAce(SePublicOpenUnrestrictedDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeWorldSid);
//
//     RtlAddAccessAllowedAce(SePublicOpenUnrestrictedDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeLocalSystemSid);
//
//     RtlAddAccessAllowedAce(SePublicOpenUnrestrictedDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeAliasAdminsSid);
//
//     RtlAddAccessAllowedAce(SePublicOpenUnrestrictedDacl,
//                            ACL_REVISION,
//                            GENERIC_READ | GENERIC_EXECUTE,
//                            SeRestrictedCodeSid);
//
//     /* create SystemDefaultDacl */
//     AclLength = sizeof(ACL) +
//                 (sizeof(ACE) + RtlLengthSid(SeLocalSystemSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeAliasAdminsSid));
//
//     SeSystemDefaultDacl = ExAllocatePoolWithTag(PagedPool,
//                                                 AclLength,
//                                                 TAG_ACL);
//     if (SeSystemDefaultDacl == NULL)
//         return FALSE;
//
//     RtlCreateAcl(SeSystemDefaultDacl,
//                  AclLength,
//                  ACL_REVISION);
//
//     RtlAddAccessAllowedAce(SeSystemDefaultDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeLocalSystemSid);
//
//     RtlAddAccessAllowedAce(SeSystemDefaultDacl,
//                            ACL_REVISION,
//                            GENERIC_READ | GENERIC_EXECUTE | READ_CONTROL,
//                            SeAliasAdminsSid);
//
//     /* create UnrestrictedDacl */
//     AclLength = sizeof(ACL) +
//                 (sizeof(ACE) + RtlLengthSid(SeWorldSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeRestrictedCodeSid));
//
//     SeUnrestrictedDacl = ExAllocatePoolWithTag(PagedPool,
//                                                AclLength,
//                                                TAG_ACL);
//     if (SeUnrestrictedDacl == NULL)
//         return FALSE;
//
//     RtlCreateAcl(SeUnrestrictedDacl,
//                  AclLength,
//                  ACL_REVISION);
//
//     RtlAddAccessAllowedAce(SeUnrestrictedDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeWorldSid);
//
//     RtlAddAccessAllowedAce(SeUnrestrictedDacl,
//                            ACL_REVISION,
//                            GENERIC_READ | GENERIC_EXECUTE,
//                            SeRestrictedCodeSid);
//
//     /* create SystemAnonymousLogonDacl */
//     AclLength = sizeof(ACL) +
//                 (sizeof(ACE) + RtlLengthSid(SeWorldSid)) +
//                 (sizeof(ACE) + RtlLengthSid(SeAnonymousLogonSid));
//
//     SeSystemAnonymousLogonDacl = ExAllocatePoolWithTag(PagedPool,
//                                                        AclLength,
//                                                        TAG_ACL);
//     if (SeSystemAnonymousLogonDacl == NULL)
//         return FALSE;
//
//     RtlCreateAcl(SeSystemAnonymousLogonDacl,
//                  AclLength,
//                  ACL_REVISION);
//
//     RtlAddAccessAllowedAce(SeSystemAnonymousLogonDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeWorldSid);
//
//     RtlAddAccessAllowedAce(SeSystemAnonymousLogonDacl,
//                            ACL_REVISION,
//                            GENERIC_ALL,
//                            SeAnonymousLogonSid);
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Allocates a discretionary access control list based on certain properties
//  * of a regular and primary access tokens.
//  *
//  * @param[in] Token
//  * An access token.
//  *
//  * @param[in] PrimaryToken
//  * A primary access token.
//  *
//  * @param[out] Dacl
//  * The returned allocated DACL.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if DACL creation from tokens has completed
//  * successfully. STATUS_INSUFFICIENT_RESOURCES is returned if DACL
//  * allocation from memory pool fails otherwise.
//  */
// NTSTATUS
// NTAPI
// SepCreateImpersonationTokenDacl(
//     _In_ PTOKEN Token,
//     _In_ PTOKEN PrimaryToken,
//     _Out_ PACL* Dacl)
// {
//     ULONG AclLength;
//     PACL TokenDacl;
//
//     PAGED_CODE();
//
//     *Dacl = NULL;
//
//     AclLength = sizeof(ACL) +
//         (sizeof(ACE) + RtlLengthSid(SeAliasAdminsSid)) +
//         (sizeof(ACE) + RtlLengthSid(SeLocalSystemSid)) +
//         (sizeof(ACE) + RtlLengthSid(SeRestrictedCodeSid)) +
//         (sizeof(ACE) + RtlLengthSid(Token->UserAndGroups->Sid)) +
//         (sizeof(ACE) + RtlLengthSid(PrimaryToken->UserAndGroups->Sid));
//
//     TokenDacl = ExAllocatePoolWithTag(PagedPool, AclLength, TAG_ACL);
//     if (TokenDacl == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlCreateAcl(TokenDacl, AclLength, ACL_REVISION as ULONG);
//     RtlAddAccessAllowedAce(TokenDacl, ACL_REVISION, GENERIC_ALL,
//                            Token->UserAndGroups->Sid);
//     RtlAddAccessAllowedAce(TokenDacl, ACL_REVISION, GENERIC_ALL,
//                            PrimaryToken->UserAndGroups->Sid);
//     RtlAddAccessAllowedAce(TokenDacl, ACL_REVISION, GENERIC_ALL,
//                            SeAliasAdminsSid);
//     RtlAddAccessAllowedAce(TokenDacl, ACL_REVISION, GENERIC_ALL,
//                            SeLocalSystemSid);
//
//     if (Token->RestrictedSids != NULL || PrimaryToken->RestrictedSids != NULL)
//     {
//         RtlAddAccessAllowedAce(TokenDacl, ACL_REVISION, GENERIC_ALL,
//                                SeRestrictedCodeSid);
//     }
//
//     *Dacl = TokenDacl;
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Captures an access control list from an already valid input ACL.
//  *
//  * @param[in] InputAcl
//  * A valid ACL.
//  *
//  * @param[in] AccessMode
//  * Processor level access mode. The processor mode determines how
//  * the input arguments are probed.
//  *
//  * @param[in] PoolType
//  * Pool type for new captured ACL for creation. The pool type determines
//  * in which memory pool the ACL data should reside.
//  *
//  * @param[in] CaptureIfKernel
//  * If set to TRUE and the processor access mode being KernelMode, we are
//  * capturing an ACL directly in the kernel. Otherwise we are capturing
//  * within a kernel mode driver.
//  *
//  * @param[out] CapturedAcl
//  * The returned and allocated captured ACL.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the ACL has been successfully captured.
//  * STATUS_INSUFFICIENT_RESOURCES is returned otherwise.
//  */
// NTSTATUS
// NTAPI
// SepCaptureAcl(
//     _In_ PACL InputAcl,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _In_ POOL_TYPE PoolType,
//     _In_ BOOLEAN CaptureIfKernel,
//     _Out_ PACL *CapturedAcl)
// {
//     PACL NewAcl;
//     ULONG AclSize;
//
//     PAGED_CODE();
//
//     /* If in kernel mode and we do not capture, just
//      * return the given ACL and don't validate it. */
//     if ((AccessMode == KernelMode) && !CaptureIfKernel)
//     {
//         *CapturedAcl = InputAcl;
//         return STATUS_SUCCESS;
//     }
//
//     /* Otherwise, capture and validate the ACL, depending on the access mode */
//     if (AccessMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForRead(InputAcl,
//                          sizeof(ACL),
//                          sizeof(ULONG));
//             AclSize = InputAcl->AclSize;
//             ProbeForRead(InputAcl,
//                          AclSize,
//                          sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//
//         /* Validate the minimal size an ACL can have */
//         if (AclSize < sizeof(ACL))
//             return STATUS_INVALID_ACL;
//
//         NewAcl = ExAllocatePoolWithTag(PoolType,
//                                        AclSize,
//                                        TAG_ACL);
//         if (!NewAcl)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         _SEH2_TRY
//         {
//             RtlCopyMemory(NewAcl, InputAcl, AclSize);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Free the ACL and return the exception code */
//             ExFreePoolWithTag(NewAcl, TAG_ACL);
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         AclSize = InputAcl->AclSize;
//
//         /* Validate the minimal size an ACL can have */
//         if (AclSize < sizeof(ACL))
//             return STATUS_INVALID_ACL;
//
//         NewAcl = ExAllocatePoolWithTag(PoolType,
//                                        AclSize,
//                                        TAG_ACL);
//         if (!NewAcl)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         RtlCopyMemory(NewAcl, InputAcl, AclSize);
//     }
//
//     /* Validate the captured ACL */
//     if (!RtlValidAcl(NewAcl))
//     {
//         /* Free the ACL and fail */
//         ExFreePoolWithTag(NewAcl, TAG_ACL);
//         return STATUS_INVALID_ACL;
//     }
//
//     /* It's valid, return it */
//     *CapturedAcl = NewAcl;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Releases (frees) a captured ACL from the memory pool.
//  *
//  * @param[in] CapturedAcl
//  * A valid captured ACL to free.
//  *
//  * @param[in] AccessMode
//  * Processor level access mode.
//  *
//  * @param[in] CaptureIfKernel
//  * If set to TRUE and the processor access mode being KernelMode, we're
//  * releasing an ACL directly in the kernel. Otherwise we're releasing
//  * within a kernel mode driver.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SepReleaseAcl(
//     _In_ PACL CapturedAcl,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _In_ BOOLEAN CaptureIfKernel)
// {
//     PAGED_CODE();
//
//     if (CapturedAcl != NULL &&
//         (AccessMode != KernelMode ||
//          (AccessMode == KernelMode && CaptureIfKernel)))
//     {
//         ExFreePoolWithTag(CapturedAcl, TAG_ACL);
//     }
// }
//
// /**
//  * @brief
//  * Determines if a certain ACE can or cannot be propagated based on
//  * ACE inheritation flags and whatnot.
//  *
//  * @param[in] AceFlags
//  * Bit flags of an ACE to perform propagation checks.
//  *
//  * @param[out] NewAceFlags
//  * New ACE bit blags based on the specific ACE flags of the first
//  * argument parameter.
//  *
//  * @param[in] IsInherited
//  * If set to TRUE, an ACE is deemed as directly inherited from another
//  * instance. In that case we're allowed to propagate.
//  *
//  * @param[in] IsDirectoryObject
//  * If set to TRUE, an object directly inherits this ACE so we can propagate
//  * it.
//  *
//  * @return
//  * Returns TRUE if an ACE can be propagated, FALSE otherwise.
//  */
// BOOLEAN
// SepShouldPropagateAce(
//     _In_ UCHAR AceFlags,
//     _Out_ PUCHAR NewAceFlags,
//     _In_ BOOLEAN IsInherited,
//     _In_ BOOLEAN IsDirectoryObject)
// {
//     if (!IsInherited)
//     {
//         *NewAceFlags = AceFlags;
//         return TRUE;
//     }
//
//     if (!IsDirectoryObject)
//     {
//         if (AceFlags & OBJECT_INHERIT_ACE)
//         {
//             *NewAceFlags = AceFlags & ~VALID_INHERIT_FLAGS;
//             return TRUE;
//         }
//         return FALSE;
//     }
//
//     if (AceFlags & NO_PROPAGATE_INHERIT_ACE)
//     {
//         if (AceFlags & CONTAINER_INHERIT_ACE)
//         {
//             *NewAceFlags = AceFlags & ~VALID_INHERIT_FLAGS;
//             return TRUE;
//         }
//         return FALSE;
//     }
//
//     if (AceFlags & CONTAINER_INHERIT_ACE)
//     {
//         *NewAceFlags = CONTAINER_INHERIT_ACE | (AceFlags & OBJECT_INHERIT_ACE) | (AceFlags & ~VALID_INHERIT_FLAGS);
//         return TRUE;
//     }
//
//     if (AceFlags & OBJECT_INHERIT_ACE)
//     {
//         *NewAceFlags = INHERIT_ONLY_ACE | OBJECT_INHERIT_ACE | (AceFlags & ~VALID_INHERIT_FLAGS);
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// /**
//  * @brief
//  * Propagates (copies) an access control list.
//  *
//  * @param[out] AclDest
//  * The destination parameter with propagated ACL.
//  *
//  * @param[in,out] AclLength
//  * The length of the ACL that we propagate.
//  *
//  * @param[in] AclSource
//  * The source instance of a valid ACL.
//  *
//  * @param[in] Owner
//  * A SID that represents the main user that identifies the ACL.
//  *
//  * @param[in] Group
//  * A SID that represents a group that identifies the ACL.
//  *
//  * @param[in] IsInherited
//  * If set to TRUE, that means the ACL is directly inherited.
//  *
//  * @param[in] IsDirectoryObject
//  * If set to TRUE, that means the ACL is directly inherited because
//  * of the object that inherits it.
//  *
//  * @param[in] GenericMapping
//  * Generic mapping of access rights to map only certain effective
//  * ACEs.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if ACL has been propagated successfully.
//  * STATUS_BUFFER_TOO_SMALL is returned if the ACL length is not greater
//  * than the maximum written size of the buffer for ACL propagation
//  * otherwise.
//  */
// NTSTATUS
// SepPropagateAcl(
//     _Out_writes_bytes_opt_(AclLength) PACL AclDest,
//     _Inout_ PULONG AclLength,
//     _In_reads_bytes_(AclSource->AclSize) PACL AclSource,
//     _In_ PSID Owner,
//     _In_ PSID Group,
//     _In_ BOOLEAN IsInherited,
//     _In_ BOOLEAN IsDirectoryObject,
//     _In_ PGENERIC_MAPPING GenericMapping)
// {
//     ACCESS_MASK Mask;
//     PACCESS_ALLOWED_ACE AceSource;
//     PACCESS_ALLOWED_ACE AceDest;
//     PUCHAR CurrentDest;
//     PUCHAR CurrentSource;
//     ULONG i;
//     ULONG Written;
//     UCHAR AceFlags;
//     USHORT AceSize;
//     USHORT AceCount = 0;
//     PSID Sid;
//     BOOLEAN WriteTwoAces;
//
//     ASSERT(RtlValidAcl(AclSource));
//     ASSERT(AclSource->AclSize % sizeof(ULONG) == 0);
//     ASSERT(AclSource->Sbz1 == 0);
//     ASSERT(AclSource->Sbz2 == 0);
//
//     Written = 0;
//     if (*AclLength >= Written + sizeof(ACL))
//     {
//         RtlCopyMemory(AclDest,
//                       AclSource,
//                       sizeof(ACL));
//     }
//     Written += sizeof(ACL);
//
//     CurrentDest = (PUCHAR)(AclDest + 1);
//     CurrentSource = (PUCHAR)(AclSource + 1);
//     for (i = 0; i < AclSource->AceCount; i++)
//     {
//         ASSERT((ULONG_PTR)CurrentDest % sizeof(ULONG) == 0);
//         ASSERT((ULONG_PTR)CurrentSource % sizeof(ULONG) == 0);
//         AceDest = (PACCESS_ALLOWED_ACE)CurrentDest;
//         AceSource = (PACCESS_ALLOWED_ACE)CurrentSource;
//
//         if (AceSource->Header.AceType > ACCESS_MAX_MS_V2_ACE_TYPE)
//         {
//             /* FIXME: handle object & compound ACEs */
//             AceSize = AceSource->Header.AceSize;
//
//             if (*AclLength >= Written + AceSize)
//             {
//                 RtlCopyMemory(AceDest, AceSource, AceSize);
//             }
//             CurrentDest += AceSize;
//             CurrentSource += AceSize;
//             Written += AceSize;
//             AceCount++;
//             continue;
//         }
//
//         /* These all have the same structure */
//         ASSERT(AceSource->Header.AceType == ACCESS_ALLOWED_ACE_TYPE ||
//                AceSource->Header.AceType == ACCESS_DENIED_ACE_TYPE ||
//                AceSource->Header.AceType == SYSTEM_AUDIT_ACE_TYPE ||
//                AceSource->Header.AceType == SYSTEM_ALARM_ACE_TYPE);
//
//         ASSERT(AceSource->Header.AceSize % sizeof(ULONG) == 0);
//         ASSERT(AceSource->Header.AceSize >= sizeof(*AceSource));
//         if (!SepShouldPropagateAce(AceSource->Header.AceFlags,
//                                    &AceFlags,
//                                    IsInherited,
//                                    IsDirectoryObject))
//         {
//             CurrentSource += AceSource->Header.AceSize;
//             continue;
//         }
//
//         /* FIXME: filter out duplicate ACEs */
//         AceSize = AceSource->Header.AceSize;
//         Mask = AceSource->Mask;
//         Sid = (PSID)&AceSource->SidStart;
//         ASSERT(AceSize >= FIELD_OFFSET(ACCESS_ALLOWED_ACE, SidStart) + RtlLengthSid(Sid));
//
//         WriteTwoAces = FALSE;
//         /* Map effective ACE to specific rights */
//         if (!(AceFlags & INHERIT_ONLY_ACE))
//         {
//             RtlMapGenericMask(&Mask, GenericMapping);
//             Mask &= GenericMapping->GenericAll;
//
//             if (IsInherited)
//             {
//                 if (RtlEqualSid(Sid, SeCreatorOwnerSid))
//                     Sid = Owner;
//                 else if (RtlEqualSid(Sid, SeCreatorGroupSid))
//                     Sid = Group;
//                 AceSize = FIELD_OFFSET(ACCESS_ALLOWED_ACE, SidStart) + RtlLengthSid(Sid);
//
//                 /*
//                  * A generic container ACE becomes two ACEs:
//                  * - a specific effective ACE with no inheritance flags
//                  * - an inherit-only ACE that keeps the generic rights
//                  */
//                 if (IsDirectoryObject &&
//                     (AceFlags & CONTAINER_INHERIT_ACE) &&
//                     (Mask != AceSource->Mask || Sid != (PSID)&AceSource->SidStart))
//                 {
//                     WriteTwoAces = TRUE;
//                 }
//             }
//         }
//
//         while (1)
//         {
//             if (*AclLength >= Written + AceSize)
//             {
//                 AceDest->Header.AceType = AceSource->Header.AceType;
//                 AceDest->Header.AceFlags = WriteTwoAces ? AceFlags & ~VALID_INHERIT_FLAGS
//                                                         : AceFlags;
//                 AceDest->Header.AceSize = AceSize;
//                 AceDest->Mask = Mask;
//                 RtlCopySid(AceSize - FIELD_OFFSET(ACCESS_ALLOWED_ACE, SidStart),
//                            (PSID)&AceDest->SidStart,
//                            Sid);
//             }
//             Written += AceSize;
//
//             AceCount++;
//             CurrentDest += AceSize;
//
//             if (!WriteTwoAces)
//                 break;
//
//             /* Second ACE keeps all the generics from the source ACE */
//             WriteTwoAces = FALSE;
//             AceDest = (PACCESS_ALLOWED_ACE)CurrentDest;
//             AceSize = AceSource->Header.AceSize;
//             Mask = AceSource->Mask;
//             Sid = (PSID)&AceSource->SidStart;
//             AceFlags |= INHERIT_ONLY_ACE;
//         }
//
//         CurrentSource += AceSource->Header.AceSize;
//     }
//
//     if (*AclLength >= sizeof(ACL))
//     {
//         AclDest->AceCount = AceCount;
//         AclDest->AclSize = Written;
//     }
//
//     if (Written > *AclLength)
//     {
//         *AclLength = Written;
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//     *AclLength = Written;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Selects an ACL and returns it to the caller.
//  *
//  * @param[in] ExplicitAcl
//  * If specified, the specified ACL to the call will be
//  * the selected ACL for the caller.
//  *
//  * @param[in] ExplicitPresent
//  * If set to TRUE and with specific ACL filled to the call, the
//  * function will immediately return the specific ACL as the selected
//  * ACL for the caller.
//  *
//  * @param[in] ExplicitDefaulted
//  * If set to FALSE and with specific ACL filled to the call, the ACL
//  * is not a default ACL. Otherwise it's a default ACL that we cannot
//  * select it as is.
//  *
//  * @param[in] ParentAcl
//  * If specified, the parent ACL will be used to determine the exact ACL
//  * length to check  if the ACL in question is not empty. If the list
//  * is not empty then the function will select such ACL to the caller.
//  *
//  * @param[in] DefaultAcl
//  * If specified, the default ACL will be the selected one for the caller.
//  *
//  * @param[out] AclLength
//  * The size length of an ACL.
//  *
//  * @param[in] Owner
//  * A SID that represents the main user that identifies the ACL.
//  *
//  * @param[in] Group
//  * A SID that represents a group that identifies the ACL.
//  *
//  * @param[out] AclPresent
//  * The returned boolean value, indicating if the ACL that we want to select
//  * does actually exist.
//  *
//  * @param[out] IsInherited
//  * The returned boolean value, indicating if the ACL we want to select it
//  * is actually inherited or not.
//  *
//  * @param[in] IsDirectoryObject
//  * If set to TRUE, the object inherits this ACL.
//  *
//  * @param[in] GenericMapping
//  * Generic mapping of access rights to map only certain effective
//  * ACEs of an ACL that we want to select it.
//  *
//  * @return
//  * Returns the selected access control list (ACL) to the caller,
//  * NULL otherwise.
//  */
// PACL
// SepSelectAcl(
//     _In_opt_ PACL ExplicitAcl,
//     _In_ BOOLEAN ExplicitPresent,
//     _In_ BOOLEAN ExplicitDefaulted,
//     _In_opt_ PACL ParentAcl,
//     _In_opt_ PACL DefaultAcl,
//     _Out_ PULONG AclLength,
//     _In_ PSID Owner,
//     _In_ PSID Group,
//     _Out_ PBOOLEAN AclPresent,
//     _Out_ PBOOLEAN IsInherited,
//     _In_ BOOLEAN IsDirectoryObject,
//     _In_ PGENERIC_MAPPING GenericMapping)
// {
//     PACL Acl;
//     NTSTATUS Status;
//
//     *AclPresent = TRUE;
//     if (ExplicitPresent && !ExplicitDefaulted)
//     {
//         Acl = ExplicitAcl;
//     }
//     else
//     {
//         if (ParentAcl)
//         {
//             *IsInherited = TRUE;
//             *AclLength = 0;
//             Status = SepPropagateAcl(NULL,
//                                      AclLength,
//                                      ParentAcl,
//                                      Owner,
//                                      Group,
//                                      *IsInherited,
//                                      IsDirectoryObject,
//                                      GenericMapping);
//             ASSERT(Status == STATUS_BUFFER_TOO_SMALL);
//
//             /* Use the parent ACL only if it's not empty */
//             if (*AclLength != sizeof(ACL))
//                 return ParentAcl;
//         }
//
//         if (ExplicitPresent)
//         {
//             Acl = ExplicitAcl;
//         }
//         else if (DefaultAcl)
//         {
//             Acl = DefaultAcl;
//         }
//         else
//         {
//             *AclPresent = FALSE;
//             Acl = NULL;
//         }
//     }
//
//     *IsInherited = FALSE;
//     *AclLength = 0;
//     if (Acl)
//     {
//         /* Get the length */
//         Status = SepPropagateAcl(NULL,
//                                  AclLength,
//                                  Acl,
//                                  Owner,
//                                  Group,
//                                  *IsInherited,
//                                  IsDirectoryObject,
//                                  GenericMapping);
//         ASSERT(Status == STATUS_BUFFER_TOO_SMALL);
//     }
//     return Acl;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
