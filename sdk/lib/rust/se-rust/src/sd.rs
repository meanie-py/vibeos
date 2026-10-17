use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, NT_SUCCESS, TRUE, FALSE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::PagedPool;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use core::mem::align_of;

#[no_mangle]
pub static mut SePublicDefaultSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
#[no_mangle]
pub static mut SePublicDefaultUnrestrictedSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
#[no_mangle]
pub static mut SePublicOpenSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
#[no_mangle]
pub static mut SePublicOpenUnrestrictedSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
#[no_mangle]
pub static mut SeSystemDefaultSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
#[no_mangle]
pub static mut SeUnrestrictedSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
#[no_mangle]
pub static mut SeSystemAnonymousLogonSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();

extern "system" {
    static SePublicDefaultDacl: PACL;
    static SePublicDefaultUnrestrictedDacl: PACL;
    static SePublicOpenDacl: PACL;
    static SePublicOpenUnrestrictedDacl: PACL;
    static SeSystemDefaultDacl: PACL;
    static SeUnrestrictedDacl: PACL;
    static SeSystemAnonymousLogonDacl: PACL;
    static SeWorldSid: PSID;
    fn SeLockSubjectContext(SubjectContext: PSECURITY_SUBJECT_CONTEXT);
    fn SeUnlockSubjectContext(SubjectContext: PSECURITY_SUBJECT_CONTEXT);
}
extern "C" {

    fn SepGetOwnerFromDescriptor(SecurityDescriptor: PISECURITY_DESCRIPTOR) -> PSID;
    fn SepGetGroupFromDescriptor(SecurityDescriptor: PISECURITY_DESCRIPTOR) -> PSID;
    fn SepGetDaclFromDescriptor(SecurityDescriptor: PISECURITY_DESCRIPTOR) -> PACL;
    fn SepGetSaclFromDescriptor(SecurityDescriptor: PISECURITY_DESCRIPTOR) -> PACL;
}
extern "C" {
    fn SepSelectAcl(
        ExplicitAcl: PACL,
        ExplicitPresent: BOOLEAN,
        ExplicitDefaulted: BOOLEAN,
        ParentAcl: PACL,
        DefaultAcl: PACL,
        AclLength: *mut ULONG,
        Owner: PSID,
        Group: PSID,
        AclPresent: *mut BOOLEAN,
        IsInherited: *mut BOOLEAN,
        IsDirectoryObject: BOOLEAN,
        GenericMapping: PGENERIC_MAPPING,
    ) -> PACL;
}
extern "C" {
    fn SepPropagateAcl(
        AclDest: PACL,
        AclLength: *mut ULONG,
        AclSource: PACL,
        Owner: PSID,
        Group: PSID,
        IsInherited: BOOLEAN,
        IsDirectoryObject: BOOLEAN,
        GenericMapping: PGENERIC_MAPPING,
    ) -> NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn SepInitSDs() -> BOOLEAN {
    SePublicDefaultSd = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
        TAG_SD,
    ) as PSECURITY_DESCRIPTOR;
    if SePublicDefaultSd.is_null() {
        return 0u8;
    }

    RtlCreateSecurityDescriptor(SePublicDefaultSd, SECURITY_DESCRIPTOR_REVISION);
    RtlSetDaclSecurityDescriptor(SePublicDefaultSd, 1u8, SePublicDefaultDacl, 0u8);

    SePublicDefaultUnrestrictedSd = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
        TAG_SD,
    ) as PSECURITY_DESCRIPTOR;
    if SePublicDefaultUnrestrictedSd.is_null() {
        return 0u8;
    }

    RtlCreateSecurityDescriptor(SePublicDefaultUnrestrictedSd, SECURITY_DESCRIPTOR_REVISION);
    RtlSetDaclSecurityDescriptor(SePublicDefaultUnrestrictedSd, 1u8, SePublicDefaultUnrestrictedDacl, 0u8);

    SePublicOpenSd = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
        TAG_SD,
    ) as PSECURITY_DESCRIPTOR;
    if SePublicOpenSd.is_null() {
        return 0u8;
    }

    RtlCreateSecurityDescriptor(SePublicOpenSd, SECURITY_DESCRIPTOR_REVISION);
    RtlSetDaclSecurityDescriptor(SePublicOpenSd, 1u8, SePublicOpenDacl, 0u8);

    SePublicOpenUnrestrictedSd = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
        TAG_SD,
    ) as PSECURITY_DESCRIPTOR;
    if SePublicOpenUnrestrictedSd.is_null() {
        return 0u8;
    }

    RtlCreateSecurityDescriptor(SePublicOpenUnrestrictedSd, SECURITY_DESCRIPTOR_REVISION);
    RtlSetDaclSecurityDescriptor(SePublicOpenUnrestrictedSd, 1u8, SePublicOpenUnrestrictedDacl, 0u8);

    SeSystemDefaultSd = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
        TAG_SD,
    ) as PSECURITY_DESCRIPTOR;
    if SeSystemDefaultSd.is_null() {
        return 0u8;
    }

    RtlCreateSecurityDescriptor(SeSystemDefaultSd, SECURITY_DESCRIPTOR_REVISION);
    RtlSetDaclSecurityDescriptor(SeSystemDefaultSd, 1u8, SeSystemDefaultDacl, 0u8);

    SeUnrestrictedSd = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
        TAG_SD,
    ) as PSECURITY_DESCRIPTOR;
    if SeUnrestrictedSd.is_null() {
        return 0u8;
    }

    RtlCreateSecurityDescriptor(SeUnrestrictedSd, SECURITY_DESCRIPTOR_REVISION);
    RtlSetDaclSecurityDescriptor(SeUnrestrictedSd, 1u8, SeUnrestrictedDacl, 0u8);

    SeSystemAnonymousLogonSd = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
        TAG_SD,
    ) as PSECURITY_DESCRIPTOR;
    if SeSystemAnonymousLogonSd.is_null() {
        return 0u8;
    }

    RtlCreateSecurityDescriptor(SeSystemAnonymousLogonSd, SECURITY_DESCRIPTOR_REVISION);
    RtlSetDaclSecurityDescriptor(SeSystemAnonymousLogonSd, 1u8, SeSystemAnonymousLogonDacl, 0u8);

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn SeSetWorldSecurityDescriptor(
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PISECURITY_DESCRIPTOR,
    BufferLength: *mut ULONG,
) -> NTSTATUS {
    let BufferLength = opaque_ptr(BufferLength);
    let mut Current: ULONG;
    let SidSize: ULONG;
    let mut SdSize: ULONG;
    let mut Status: NTSTATUS;
    let SdRel = SecurityDescriptor as PISECURITY_DESCRIPTOR_RELATIVE;

    if SecurityInformation == 0 {
        return STATUS_ACCESS_DENIED;
    }

    SidSize = RtlLengthSid(SeWorldSid);
    SdSize = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG;
    if SecurityInformation & OWNER_SECURITY_INFORMATION != 0 {
        SdSize += SidSize;
    }
    if SecurityInformation & GROUP_SECURITY_INFORMATION != 0 {
        SdSize += SidSize;
    }
    if SecurityInformation & DACL_SECURITY_INFORMATION != 0 {
        SdSize += core::mem::size_of::<ACL>() as ULONG + core::mem::size_of::<ACE>() as ULONG + SidSize;
    }
    if SecurityInformation & SACL_SECURITY_INFORMATION != 0 {
        SdSize += core::mem::size_of::<ACL>() as ULONG + core::mem::size_of::<ACE>() as ULONG + SidSize;
    }

    if *BufferLength < SdSize {
        *BufferLength = SdSize;
        return STATUS_BUFFER_TOO_SMALL;
    }

    *BufferLength = SdSize;

    Status = RtlCreateSecurityDescriptorRelative(SdRel, SECURITY_DESCRIPTOR_REVISION);
    if Status < 0 {
        return Status;
    }

    Current = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG;

    if SecurityInformation & OWNER_SECURITY_INFORMATION != 0 {
        core::ptr::copy_nonoverlapping(
            SeWorldSid as *const u8,
            (SdRel as *mut u8).add(Current as usize),
            SidSize as usize,
        );
        (*SdRel).Owner = Current;
        Current += SidSize;
    }

    if SecurityInformation & GROUP_SECURITY_INFORMATION != 0 {
        core::ptr::copy_nonoverlapping(
            SeWorldSid as *const u8,
            (SdRel as *mut u8).add(Current as usize),
            SidSize as usize,
        );
        (*SdRel).Group = Current;
        Current += SidSize;
    }

    if SecurityInformation & DACL_SECURITY_INFORMATION != 0 {
        let Dacl = ((SdRel as *mut u8).add(Current as usize)) as PACL;

        Status = RtlCreateAcl(
            Dacl,
            core::mem::size_of::<ACL>() as ULONG + core::mem::size_of::<ACE>() as ULONG + SidSize,
            ACL_REVISION,
        );
        if Status < 0 {
            return Status;
        }

        Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, SeWorldSid);
        if Status < 0 {
            return Status;
        }

        (*SdRel).Control |= SE_DACL_PRESENT;
        (*SdRel).Dacl = Current;
        Current += SidSize;
    }

    if SecurityInformation & SACL_SECURITY_INFORMATION != 0 {
        let Sacl = ((SdRel as *mut u8).add(Current as usize)) as PACL;

        Status = RtlCreateAcl(
            Sacl,
            core::mem::size_of::<ACL>() as ULONG + core::mem::size_of::<ACE>() as ULONG + SidSize,
            ACL_REVISION,
        );
        if Status < 0 {
            return Status;
        }

        Status = RtlAddAuditAccessAce(
            Sacl,
            ACL_REVISION,
            ACCESS_SYSTEM_SECURITY | STANDARD_RIGHTS_ALL,
            SeWorldSid,
            1u8,
            1u8,
        );
        if Status < 0 {
            return Status;
        }

        (*SdRel).Control |= SE_SACL_PRESENT;
        (*SdRel).Sacl = Current;
        Current += SidSize;
    }

    STATUS_SUCCESS
}

unsafe fn DetermineSIDSize(
    Sid: PISID,
    OutSAC: *mut ULONG,
    ProcessorMode: KPROCESSOR_MODE,
) -> Result<ULONG, NTSTATUS> {
    let Size: ULONG;

    if Sid.is_null() {
        *OutSAC = 0;
        return Ok(0);
    }

    if ProcessorMode != 0 {
        let sac = probe_read_user::<UCHAR>(core::ptr::addr_of!((*Sid).SubAuthorityCount), ProcessorMode)?;
        *OutSAC = sac as ULONG;
        Size = RtlLengthRequiredSid(*OutSAC);
        probe_user_buffer(Sid as _, Size as usize, core::mem::size_of::<ULONG>(), ProcessorMode)?;
    } else {
        *OutSAC = (*Sid).SubAuthorityCount as ULONG;
        Size = RtlLengthRequiredSid(*OutSAC);
    }

    Ok(Size)
}

unsafe fn DetermineACLSize(
    Acl: PACL,
    ProcessorMode: KPROCESSOR_MODE,
) -> Result<ULONG, NTSTATUS> {
    let Size: ULONG;

    if Acl.is_null() {
        return Ok(0);
    }

    if ProcessorMode == 0 {
        return Ok((*Acl).AclSize as ULONG);
    }

    let acl_size = probe_read_user::<USHORT>(core::ptr::addr_of!((*Acl).AclSize), ProcessorMode)?;
    Size = acl_size as ULONG;
    probe_user_buffer(Acl as _, Size as usize, core::mem::size_of::<ULONG>(), ProcessorMode)?;

    Ok(Size)
}

#[no_mangle]
pub unsafe extern "system" fn SeCaptureSecurityDescriptor(
    _OriginalSecurityDescriptor: PSECURITY_DESCRIPTOR,
    CurrentMode: KPROCESSOR_MODE,
    PoolType: POOL_TYPE,
    CaptureIfKernel: BOOLEAN,
    CapturedSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let _OriginalSecurityDescriptor = opaque_ptr(_OriginalSecurityDescriptor);
    let CapturedSecurityDescriptor = opaque_ptr(CapturedSecurityDescriptor);
    let OriginalDescriptor = _OriginalSecurityDescriptor as PISECURITY_DESCRIPTOR;
    let mut DescriptorCopy: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let NewDescriptor: PISECURITY_DESCRIPTOR_RELATIVE;
    let mut OwnerSAC: ULONG = 0;
    let mut GroupSAC: ULONG = 0;
    let mut OwnerSize: ULONG = 0;
    let mut GroupSize: ULONG = 0;
    let mut SaclSize: ULONG = 0;
    let mut DaclSize: ULONG = 0;
    let mut DescriptorSize: ULONG;
    let mut Offset: ULONG;

    if OriginalDescriptor.is_null() {
        *CapturedSecurityDescriptor = core::ptr::null_mut();
        return STATUS_SUCCESS;
    }

    if CurrentMode == 0 && CaptureIfKernel == 0 {
        if (*OriginalDescriptor).Revision != SECURITY_DESCRIPTOR_REVISION1 as u8 {
            return STATUS_UNKNOWN_REVISION;
        }

        *CapturedSecurityDescriptor = _OriginalSecurityDescriptor;
        return STATUS_SUCCESS;
    }

    // From here the descriptor may be user memory, which the C reads inside
    // _SEH2_TRY. Capture the header once into a local, take the component pointers
    // from that copy, and read every component through the fault-tolerant copy.
    // The previous translation read the header, ran RtlValidSid/RtlValidAcl and
    // copied the components IN PLACE: a bad page bugchecked, and a caller could
    // change a SID or ACL between its validation and its copy.
    let mut Header: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let relative_size = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>();
    if let Err(s) = safe_copy_from_user(
        core::ptr::addr_of_mut!(Header) as PVOID,
        OriginalDescriptor as _,
        relative_size,
        core::mem::size_of::<ULONG>(),
        CurrentMode,
    ) {
        return s;
    }

    if Header.Revision != SECURITY_DESCRIPTOR_REVISION1 as u8 {
        return STATUS_UNKNOWN_REVISION;
    }

    // Decide self-relative from THIS first read. The absolute copy below re-reads
    // Control, and a racing caller that flips SE_SELF_RELATIVE between the two reads
    // could otherwise make us copy the absolute struct (raw pointer fields) and then
    // rebase them as self-relative offsets -- a wild kernel address later handed to
    // RtlValidSid. Pin the branch here; the getters below read copy #2 consistently.
    let self_relative = Header.Control & SE_SELF_RELATIVE != 0;

    if !self_relative {
        // Absolute: the header is the larger struct of pointers.
        if let Err(s) = safe_copy_from_user(
            core::ptr::addr_of_mut!(Header) as PVOID,
            OriginalDescriptor as _,
            core::mem::size_of::<_SECURITY_DESCRIPTOR>(),
            core::mem::size_of::<ULONG>(),
            CurrentMode,
        ) {
            return s;
        }
    }

    // The getters resolve self-relative offsets against the descriptor they are
    // given; move those results from the local copy onto the caller's descriptor.
    let header_base = core::ptr::addr_of_mut!(Header) as usize;
    let rebase = move |p: usize| -> usize {
        if p == 0 || !self_relative {
            p
        } else {
            (OriginalDescriptor as usize).wrapping_add(p - header_base)
        }
    };
    let header_ptr = core::ptr::addr_of_mut!(Header) as PISECURITY_DESCRIPTOR;
    DescriptorCopy.Revision = Header.Revision;
    DescriptorCopy.Sbz1 = Header.Sbz1;
    DescriptorCopy.Control = Header.Control & !SE_SELF_RELATIVE;
    DescriptorCopy.Owner = rebase(SepGetOwnerFromDescriptor(header_ptr) as usize) as PSID;
    DescriptorCopy.Group = rebase(SepGetGroupFromDescriptor(header_ptr) as usize) as PSID;
    DescriptorCopy.Sacl = rebase(SepGetSaclFromDescriptor(header_ptr) as usize) as PACL;
    DescriptorCopy.Dacl = rebase(SepGetDaclFromDescriptor(header_ptr) as usize) as PACL;
    DescriptorSize = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG;

    OwnerSize = match DetermineSIDSize(DescriptorCopy.Owner as PISID, &mut OwnerSAC, CurrentMode) {
        Ok(v) => v, Err(s) => return s,
    };
    DescriptorSize += (OwnerSize + 3) & !3 ;
    GroupSize = match DetermineSIDSize(DescriptorCopy.Group as PISID, &mut GroupSAC, CurrentMode) {
        Ok(v) => v, Err(s) => return s,
    };
    DescriptorSize += (GroupSize + 3) & !3 ;

    if DescriptorCopy.Control & SE_SACL_PRESENT != 0 {
        SaclSize = match DetermineACLSize(DescriptorCopy.Sacl, CurrentMode) {
            Ok(v) => v, Err(s) => return s,
        };
        DescriptorSize += (SaclSize + 3) & !3 ;
    }

    if DescriptorCopy.Control & SE_DACL_PRESENT != 0 {
        DaclSize = match DetermineACLSize(DescriptorCopy.Dacl, CurrentMode) {
            Ok(v) => v, Err(s) => return s,
        };
        DescriptorSize += (DaclSize + 3) & !3 ;
    }

    NewDescriptor = ExAllocatePoolWithTag(PoolType, DescriptorSize as SIZE_T, TAG_SD) as PISECURITY_DESCRIPTOR_RELATIVE;
    if NewDescriptor.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let _guard = PoolAllocGuard::new(NewDescriptor as PVOID, TAG_SD);

    core::ptr::write_bytes(NewDescriptor as *mut u8, 0, DescriptorSize as usize);
    (*NewDescriptor).Revision = DescriptorCopy.Revision;
    (*NewDescriptor).Sbz1 = DescriptorCopy.Sbz1;
    (*NewDescriptor).Control = DescriptorCopy.Control | SE_SELF_RELATIVE;

    Offset = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG;

    // Copy one component, then pin the count or size it was measured by (the
    // caller can change it in between) and validate the COPY.
    let copy_component = move |src: PVOID, size: ULONG, at: ULONG| -> Result<PVOID, NTSTATUS> {
        let dst = (NewDescriptor as *mut u8).add(at as usize) as PVOID;
        safe_copy_from_user(dst, src as _, size as usize, 1, CurrentMode)?;
        Ok(dst)
    };

    if !DescriptorCopy.Owner.is_null() {
        let owner = match copy_component(DescriptorCopy.Owner, OwnerSize, Offset) {
            Ok(p) => p as PISID, Err(s) => return s,
        };
        (*owner).SubAuthorityCount = OwnerSAC as UCHAR;
        if RtlValidSid(owner as PSID) == 0 {
            return STATUS_INVALID_SID;
        }
        (*NewDescriptor).Owner = Offset;
        Offset += (OwnerSize + 3) & !3 ;
    }

    if !DescriptorCopy.Group.is_null() {
        let group = match copy_component(DescriptorCopy.Group, GroupSize, Offset) {
            Ok(p) => p as PISID, Err(s) => return s,
        };
        (*group).SubAuthorityCount = GroupSAC as UCHAR;
        if RtlValidSid(group as PSID) == 0 {
            return STATUS_INVALID_SID;
        }
        (*NewDescriptor).Group = Offset;
        Offset += (GroupSize + 3) & !3 ;
    }

    if !DescriptorCopy.Sacl.is_null() {
        let sacl = match copy_component(DescriptorCopy.Sacl as PVOID, SaclSize, Offset) {
            Ok(p) => p as PACL, Err(s) => return s,
        };
        (*sacl).AclSize = SaclSize as USHORT;
        if RtlValidAcl(sacl) == 0 {
            return STATUS_INVALID_ACL;
        }
        (*NewDescriptor).Sacl = Offset;
        Offset += (SaclSize + 3) & !3 ;
    }

    if !DescriptorCopy.Dacl.is_null() {
        let dacl = match copy_component(DescriptorCopy.Dacl as PVOID, DaclSize, Offset) {
            Ok(p) => p as PACL, Err(s) => return s,
        };
        (*dacl).AclSize = DaclSize as USHORT;
        if RtlValidAcl(dacl) == 0 {
            return STATUS_INVALID_ACL;
        }
        (*NewDescriptor).Dacl = Offset;
        Offset += (DaclSize + 3) & !3 ;
    }

    *CapturedSecurityDescriptor = NewDescriptor as PSECURITY_DESCRIPTOR;
    _guard.disarm();
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeQuerySecurityDescriptorInfo(
    SecurityInformation: PSECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    Length: *mut ULONG,
    ObjectsSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let Length = opaque_ptr(Length);
    let ObjectsSecurityDescriptor = opaque_ptr(ObjectsSecurityDescriptor);
    let ObjectSd: PISECURITY_DESCRIPTOR;
    let RelSD: PISECURITY_DESCRIPTOR_RELATIVE;
    let mut Owner: PSID = core::ptr::null_mut();
    let mut Group: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut Sacl: PACL = core::ptr::null_mut();
    let mut OwnerLength: ULONG = 0;
    let mut GroupLength: ULONG = 0;
    let mut DaclLength: ULONG = 0;
    let mut SaclLength: ULONG = 0;
    let mut Control: SECURITY_DESCRIPTOR_CONTROL = 0;
    let mut Current: usize;
    let SdLength: ULONG;

    RelSD = SecurityDescriptor as PISECURITY_DESCRIPTOR_RELATIVE;

    if (*ObjectsSecurityDescriptor).is_null() {
        if *Length < core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG {
            *Length = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG;
            return STATUS_BUFFER_TOO_SMALL;
        }

        *Length = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG;
        let mut Empty: _SECURITY_DESCRIPTOR_RELATIVE = core::mem::zeroed();
        RtlCreateSecurityDescriptorRelative(&mut Empty, SECURITY_DESCRIPTOR_REVISION);
        return match safe_copy_raw(
            RelSD as PVOID,
            &Empty as *const _ as *const core::ffi::c_void,
            core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>(),
        ) {
            Ok(()) => STATUS_SUCCESS,
            Err(s) => s,
        };
    }

    ObjectSd = *ObjectsSecurityDescriptor as PISECURITY_DESCRIPTOR;

    Control = SE_SELF_RELATIVE;
    if *SecurityInformation & OWNER_SECURITY_INFORMATION != 0 {
        Owner = SepGetOwnerFromDescriptor(ObjectSd);
        if !Owner.is_null() {
            OwnerLength = (RtlLengthSid(Owner) + 3) & !3 ;
            Control |= (*ObjectSd).Control & SE_OWNER_DEFAULTED;
        }
    }

    if *SecurityInformation & GROUP_SECURITY_INFORMATION != 0 {
        Group = SepGetGroupFromDescriptor(ObjectSd);
        if !Group.is_null() {
            GroupLength = (RtlLengthSid(Group) + 3) & !3 ;
            Control |= (*ObjectSd).Control & SE_GROUP_DEFAULTED;
        }
    }

    if (*SecurityInformation & DACL_SECURITY_INFORMATION != 0) &&
       ((*ObjectSd).Control & SE_DACL_PRESENT != 0)
    {
        Dacl = SepGetDaclFromDescriptor(ObjectSd);
        if !Dacl.is_null() {
            DaclLength = ((*Dacl).AclSize as ULONG + 3) & !3 ;
        }

        Control |= (*ObjectSd).Control & (SE_DACL_DEFAULTED | SE_DACL_PRESENT);
    }

    if (*SecurityInformation & SACL_SECURITY_INFORMATION != 0) &&
       ((*ObjectSd).Control & SE_SACL_PRESENT != 0)
    {
        Sacl = SepGetSaclFromDescriptor(ObjectSd);
        if !Sacl.is_null() {
            SaclLength = ((*Sacl).AclSize as ULONG + 3) & !3 ;
        }

        Control |= (*ObjectSd).Control & (SE_SACL_DEFAULTED | SE_SACL_PRESENT);
    }

    SdLength = OwnerLength + GroupLength + DaclLength + SaclLength +
               core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG;
    if *Length < SdLength {
        *Length = SdLength;
        return STATUS_BUFFER_TOO_SMALL;
    }

    // NtQuerySecurityObject hands us the caller's buffer, which the C fills inside
    // its _SEH2_TRY. Build the descriptor in pool (it is self-relative, so where it
    // is built does not matter) and copy it out once with the fault-tolerant copy.
    let Built = ExAllocatePoolWithTag(PagedPool, SdLength as SIZE_T, TAG_SD) as PISECURITY_DESCRIPTOR_RELATIVE;
    if Built.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let _built_guard = PoolAllocGuard::new(Built as PVOID, TAG_SD);
    let BuiltBase = Built as usize;

    RtlCreateSecurityDescriptorRelative(Built, SECURITY_DESCRIPTOR_REVISION);
    (*Built).Control = Control;

    Current = BuiltBase + core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>();

    if OwnerLength != 0 {
        core::ptr::copy_nonoverlapping(Owner as *const u8, Current as *mut u8, OwnerLength as usize);
        (*Built).Owner = (Current - BuiltBase) as ULONG;
        Current += OwnerLength as usize;
    }

    if GroupLength != 0 {
        core::ptr::copy_nonoverlapping(Group as *const u8, Current as *mut u8, GroupLength as usize);
        (*Built).Group = (Current - BuiltBase) as ULONG;
        Current += GroupLength as usize;
    }

    if DaclLength != 0 {
        core::ptr::copy_nonoverlapping(Dacl as *const u8, Current as *mut u8, DaclLength as usize);
        (*Built).Dacl = (Current - BuiltBase) as ULONG;
        Current += DaclLength as usize;
    }

    if SaclLength != 0 {
        core::ptr::copy_nonoverlapping(Sacl as *const u8, Current as *mut u8, SaclLength as usize);
        (*Built).Sacl = (Current - BuiltBase) as ULONG;
        Current += SaclLength as usize;
    }

    if let Err(s) = safe_copy_raw(RelSD as PVOID, Built as *const core::ffi::c_void, SdLength as usize) {
        return s;
    }

    *Length = SdLength;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeReleaseSecurityDescriptor(
    CapturedSecurityDescriptor: PSECURITY_DESCRIPTOR,
    CurrentMode: KPROCESSOR_MODE,
    CaptureIfKernelMode: BOOLEAN,
) -> NTSTATUS {
    let CapturedSecurityDescriptor = opaque_ptr(CapturedSecurityDescriptor);
    if !CapturedSecurityDescriptor.is_null() &&
       (CurrentMode != 0 || (CurrentMode == 0 && CaptureIfKernelMode != 0))
    {
        ExFreePoolWithTag(CapturedSecurityDescriptor as PVOID, TAG_SD);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeSetSecurityDescriptorInfo(
    Object: PVOID,
    SecurityInformation: PSECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    ObjectsSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
) -> NTSTATUS {
    let Object = opaque_ptr(Object);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let ObjectsSecurityDescriptor = opaque_ptr(ObjectsSecurityDescriptor);
    let GenericMapping = opaque_ptr(GenericMapping);
    SeSetSecurityDescriptorInfoEx(
        Object,
        SecurityInformation,
        SecurityDescriptor,
        ObjectsSecurityDescriptor,
        0,
        PoolType,
        GenericMapping,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SeSetSecurityDescriptorInfoEx(
    Object: PVOID,
    _SecurityInformation: PSECURITY_INFORMATION,
    _SecurityDescriptor: PSECURITY_DESCRIPTOR,
    ObjectsSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    AutoInheritFlags: ULONG,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
) -> NTSTATUS {
    let Object = opaque_ptr(Object);
    let _SecurityDescriptor = opaque_ptr(_SecurityDescriptor);
    let ObjectsSecurityDescriptor = opaque_ptr(ObjectsSecurityDescriptor);
    let GenericMapping = opaque_ptr(GenericMapping);
    let ObjectSd: PISECURITY_DESCRIPTOR_RELATIVE;
    let NewSd: PISECURITY_DESCRIPTOR_RELATIVE;
    let SecurityDescriptor = _SecurityDescriptor as PISECURITY_DESCRIPTOR;
    let mut Owner: PSID;
    let mut Group: PSID;
    let mut Dacl: PACL;
    let mut Sacl: PACL;
    let OwnerLength: ULONG;
    let GroupLength: ULONG;
    let DaclLength: ULONG;
    let SaclLength: ULONG;
    let mut Control: SECURITY_DESCRIPTOR_CONTROL = 0;
    let mut Current: ULONG;
    let SecurityInformation: SECURITY_INFORMATION;

    ObjectSd = *ObjectsSecurityDescriptor as PISECURITY_DESCRIPTOR_RELATIVE;

    if ObjectSd.is_null() {
        return STATUS_NO_SECURITY_ON_OBJECT;
    }

    SecurityInformation = *_SecurityInformation;

    if SecurityInformation & OWNER_SECURITY_INFORMATION != 0 {
        Owner = SepGetOwnerFromDescriptor(SecurityDescriptor);
        Control |= (*SecurityDescriptor).Control & SE_OWNER_DEFAULTED;
    } else {
        Owner = SepGetOwnerFromDescriptor(ObjectSd as PISECURITY_DESCRIPTOR);
        Control |= (*ObjectSd).Control & SE_OWNER_DEFAULTED;
    }
    OwnerLength = if !Owner.is_null() { RtlLengthSid(Owner) } else { 0 };

    if SecurityInformation & GROUP_SECURITY_INFORMATION != 0 {
        Group = SepGetGroupFromDescriptor(SecurityDescriptor);
        Control |= (*SecurityDescriptor).Control & SE_GROUP_DEFAULTED;
    } else {
        Group = SepGetGroupFromDescriptor(ObjectSd as PISECURITY_DESCRIPTOR);
        Control |= (*ObjectSd).Control & SE_GROUP_DEFAULTED;
    }
    GroupLength = if !Group.is_null() { RtlLengthSid(Group) } else { 0 };

    if SecurityInformation & DACL_SECURITY_INFORMATION != 0 {
        Dacl = SepGetDaclFromDescriptor(SecurityDescriptor);
        Control |= (*SecurityDescriptor).Control & (SE_DACL_DEFAULTED | SE_DACL_PRESENT);
    } else {
        Dacl = SepGetDaclFromDescriptor(ObjectSd as PISECURITY_DESCRIPTOR);
        Control |= (*ObjectSd).Control & (SE_DACL_DEFAULTED | SE_DACL_PRESENT);
    }
    DaclLength = if !Dacl.is_null() { ((*Dacl).AclSize as ULONG + 3) & !3  } else { 0 };

    if SecurityInformation & SACL_SECURITY_INFORMATION != 0 {
        Sacl = SepGetSaclFromDescriptor(SecurityDescriptor);
        Control |= (*SecurityDescriptor).Control & (SE_SACL_DEFAULTED | SE_SACL_PRESENT);
    } else {
        Sacl = SepGetSaclFromDescriptor(ObjectSd as PISECURITY_DESCRIPTOR);
        Control |= (*ObjectSd).Control & (SE_SACL_DEFAULTED | SE_SACL_PRESENT);
    }
    SaclLength = if !Sacl.is_null() { ((*Sacl).AclSize as ULONG + 3) & !3  } else { 0 };

    NewSd = ExAllocatePoolWithTag(
        PoolType,
        (core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG +
         OwnerLength + GroupLength + DaclLength + SaclLength) as SIZE_T,
        TAG_SD,
    ) as PISECURITY_DESCRIPTOR_RELATIVE;
    if NewSd.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCreateSecurityDescriptorRelative(NewSd, SECURITY_DESCRIPTOR_REVISION1);

    Current = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG;

    if OwnerLength != 0 {
        core::ptr::copy_nonoverlapping(
            Owner as *const u8,
            (NewSd as *mut u8).add(Current as usize),
            OwnerLength as usize,
        );
        (*NewSd).Owner = Current;
        Current += OwnerLength;
    }

    if GroupLength != 0 {
        core::ptr::copy_nonoverlapping(
            Group as *const u8,
            (NewSd as *mut u8).add(Current as usize),
            GroupLength as usize,
        );
        (*NewSd).Group = Current;
        Current += GroupLength;
    }

    if DaclLength != 0 {
        core::ptr::copy_nonoverlapping(
            Dacl as *const u8,
            (NewSd as *mut u8).add(Current as usize),
            DaclLength as usize,
        );
        (*NewSd).Dacl = Current;
        Current += DaclLength;
    }

    if SaclLength != 0 {
        core::ptr::copy_nonoverlapping(
            Sacl as *const u8,
            (NewSd as *mut u8).add(Current as usize),
            SaclLength as usize,
        );
        (*NewSd).Sacl = Current;
        Current += SaclLength;
    }

    (*NewSd).Control |= Control;
    *ObjectsSecurityDescriptor = NewSd as PSECURITY_DESCRIPTOR;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeValidSecurityDescriptor(
    Length: ULONG,
    _SecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> BOOLEAN {
    let _SecurityDescriptor = opaque_ptr(_SecurityDescriptor);
    let mut SdLength: ULONG;
    let mut Sid: PISID;
    let mut Acl: PACL;
    let SecurityDescriptor = _SecurityDescriptor as PISECURITY_DESCRIPTOR_RELATIVE;

    if Length < SECURITY_DESCRIPTOR_MIN_LENGTH as ULONG {
        return 0u8;
    }

    if (*SecurityDescriptor).Revision != SECURITY_DESCRIPTOR_REVISION1 as u8 {
        return 0u8;
    }

    if (*SecurityDescriptor).Control & SE_SELF_RELATIVE == 0 {
        return 0u8;
    }

    SdLength = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG;

    if (*SecurityDescriptor).Owner == 0 {
        return 0u8;
    }

    if (*SecurityDescriptor).Owner % core::mem::size_of::<ULONG>() as ULONG != 0 {
        return 0u8;
    }

    if ((*SecurityDescriptor).Owner > Length) ||
       (Length - (*SecurityDescriptor).Owner < core::mem::size_of::<_SID>() as ULONG)
    {
        return 0u8;
    }

    Sid = ((SecurityDescriptor as usize) + (*SecurityDescriptor).Owner as usize) as PISID;
    if (*Sid).Revision != SID_REVISION as u8 {
        return 0u8;
    }

    SdLength += core::mem::size_of::<_SID>() as ULONG + ((*Sid).SubAuthorityCount as ULONG - 1) * core::mem::size_of::<ULONG>() as ULONG ;
    if Length < SdLength {
        return 0u8;
    }

    if (*SecurityDescriptor).Group != 0 {
        if (*SecurityDescriptor).Group % core::mem::size_of::<ULONG>() as ULONG != 0 {
            return 0u8;
        }

        if ((*SecurityDescriptor).Group > Length) ||
           (Length - (*SecurityDescriptor).Group < core::mem::size_of::<_SID>() as ULONG)
        {
            return 0u8;
        }

        Sid = ((SecurityDescriptor as usize) + (*SecurityDescriptor).Group as usize) as PSID as PISID;
        if (*Sid).Revision != SID_REVISION as u8 {
            return 0u8;
        }

        SdLength += core::mem::size_of::<_SID>() as ULONG + ((*Sid).SubAuthorityCount as ULONG - 1) * core::mem::size_of::<ULONG>() as ULONG ;
        if Length < SdLength {
            return 0u8;
        }
    }

    if (*SecurityDescriptor).Dacl != 0 {
        if (*SecurityDescriptor).Dacl % core::mem::size_of::<ULONG>() as ULONG != 0 {
            return 0u8;
        }

        if ((*SecurityDescriptor).Dacl > Length) ||
           (Length - (*SecurityDescriptor).Dacl < core::mem::size_of::<ACL>() as ULONG)
        {
            return 0u8;
        }

        Acl = ((SecurityDescriptor as usize) + (*SecurityDescriptor).Dacl as usize) as PACL;

        SdLength += (*Acl).AclSize as ULONG;
        if Length < SdLength {
            return 0u8;
        }

        if RtlValidAcl(Acl) == 0 {
            return 0u8;
        }
    }

    if (*SecurityDescriptor).Sacl != 0 {
        if (*SecurityDescriptor).Sacl % core::mem::size_of::<ULONG>() as ULONG != 0 {
            return 0u8;
        }

        if ((*SecurityDescriptor).Sacl > Length) ||
           (Length - (*SecurityDescriptor).Sacl < core::mem::size_of::<ACL>() as ULONG)
        {
            return 0u8;
        }

        Acl = ((SecurityDescriptor as usize) + (*SecurityDescriptor).Sacl as usize) as PACL;

        SdLength += (*Acl).AclSize as ULONG;
        if Length < SdLength {
            return 0u8;
        }

        if RtlValidAcl(Acl) == 0 {
            return 0u8;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn SeDeassignSecurity(
    SecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    if !(*SecurityDescriptor).is_null() {
        ExFreePoolWithTag(*SecurityDescriptor as PVOID, TAG_SD);
        *SecurityDescriptor = core::ptr::null_mut();
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeAssignSecurityEx(
    _ParentDescriptor: PSECURITY_DESCRIPTOR,
    _ExplicitDescriptor: PSECURITY_DESCRIPTOR,
    NewDescriptor: *mut PSECURITY_DESCRIPTOR,
    ObjectType: *mut GUID,
    IsDirectoryObject: BOOLEAN,
    AutoInheritFlags: ULONG,
    SubjectContext: PSECURITY_SUBJECT_CONTEXT,
    GenericMapping: PGENERIC_MAPPING,
    PoolType: POOL_TYPE,
) -> NTSTATUS {
    let _ParentDescriptor = opaque_ptr(_ParentDescriptor);
    let _ExplicitDescriptor = opaque_ptr(_ExplicitDescriptor);
    let NewDescriptor = opaque_ptr(NewDescriptor);
    let ObjectType = opaque_ptr(ObjectType);
    let SubjectContext = opaque_ptr(SubjectContext);
    let GenericMapping = opaque_ptr(GenericMapping);
    let ParentDescriptor = _ParentDescriptor as PISECURITY_DESCRIPTOR;
    let ExplicitDescriptor = _ExplicitDescriptor as PISECURITY_DESCRIPTOR;
    let Descriptor: PISECURITY_DESCRIPTOR_RELATIVE;
    let Token: PTOKEN;
    let mut OwnerLength: ULONG;
    let mut GroupLength: ULONG;
    let mut DaclLength: ULONG;
    let mut SaclLength: ULONG;
    let Length: ULONG;
    let mut Control: SECURITY_DESCRIPTOR_CONTROL = 0;
    let mut Current: ULONG;
    let mut Owner: PSID = core::ptr::null_mut();
    let mut Group: PSID = core::ptr::null_mut();
    let mut ExplicitAcl: PACL;
    let mut ExplicitPresent: BOOLEAN;
    let mut ExplicitDefaulted: BOOLEAN;
    let mut ParentAcl: PACL;
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut Sacl: PACL = core::ptr::null_mut();
    let mut DaclIsInherited: BOOLEAN = 0;
    let mut SaclIsInherited: BOOLEAN = 0;
    let mut DaclPresent: BOOLEAN = 0;
    let mut SaclPresent: BOOLEAN = 0;
    let mut Status: NTSTATUS;

    // Windows writes the output only when it returns: a fault inside (a NULL token, a NULL
    // owner SID) leaves the caller's value untouched (SeInheritance.c:532/556). The C
    // NULLed it first; each early return below NULLs it instead.

    if SubjectContext.is_null() {
        core::ptr::write_volatile(NewDescriptor, core::ptr::null_mut());
        return STATUS_NO_TOKEN;
    }

    SeLockSubjectContext(SubjectContext);

    if !(*SubjectContext).ClientToken.is_null() {
        Token = (*SubjectContext).ClientToken as PTOKEN;
    } else {
        Token = (*SubjectContext).PrimaryToken as PTOKEN;
    }

    if !ExplicitDescriptor.is_null() {
        Owner = SepGetOwnerFromDescriptor(ExplicitDescriptor);
    }
    if Owner.is_null() {
        if AutoInheritFlags & SEF_DEFAULT_OWNER_FROM_PARENT != 0 {
            if ParentDescriptor.is_null() {
                SeUnlockSubjectContext(SubjectContext);
                core::ptr::write_volatile(NewDescriptor, core::ptr::null_mut());
                return STATUS_INVALID_OWNER;
            }

            Owner = SepGetOwnerFromDescriptor(ParentDescriptor);
            if Owner.is_null() {
                SeUnlockSubjectContext(SubjectContext);
                core::ptr::write_volatile(NewDescriptor, core::ptr::null_mut());
                return STATUS_INVALID_OWNER;
            }
        } else {
            Owner = (*(*Token).UserAndGroups.add((*Token).DefaultOwnerIndex as usize)).Sid;
        }
    }
    if Owner.is_null() {
        #[cfg(target_arch = "aarch64")]
        {
            // ARM64 maps the low page, so RtlLengthSid(NULL) can consume
            // zero-page garbage instead of producing the access violation
            // required after the subject context has been locked.
            let invalid_address = usize::MAX;
            core::arch::asm!(
                "ldrb wzr, [{address}]",
                address = in(reg) invalid_address,
                options(nostack, readonly),
            );
            ntkrnl_sys::ex::ExRaiseStatus(
                ntkrnl_sys::constants::STATUS_ACCESS_VIOLATION,
            );
        }
    }
    OwnerLength = RtlLengthSid(Owner);

    if !ExplicitDescriptor.is_null() {
        Group = SepGetGroupFromDescriptor(ExplicitDescriptor);
    }
    if Group.is_null() {
        if AutoInheritFlags & SEF_DEFAULT_GROUP_FROM_PARENT != 0 {
            if ParentDescriptor.is_null() {
                SeUnlockSubjectContext(SubjectContext);
                core::ptr::write_volatile(NewDescriptor, core::ptr::null_mut());
                return STATUS_INVALID_PRIMARY_GROUP;
            }

            Group = SepGetGroupFromDescriptor(ParentDescriptor);
            if Group.is_null() {
                SeUnlockSubjectContext(SubjectContext);
                core::ptr::write_volatile(NewDescriptor, core::ptr::null_mut());
                return STATUS_INVALID_PRIMARY_GROUP;
            }
        } else {
            Group = (*Token).PrimaryGroup;
        }
    }
    if Group.is_null() {
        SeUnlockSubjectContext(SubjectContext);
        core::ptr::write_volatile(NewDescriptor, core::ptr::null_mut());
        return STATUS_INVALID_PRIMARY_GROUP;
    }
    GroupLength = RtlLengthSid(Group);

    DaclLength = 0;
    ExplicitAcl = core::ptr::null_mut();
    ExplicitPresent = 0;
    ExplicitDefaulted = 0;
    if !ExplicitDescriptor.is_null() &&
       ((*ExplicitDescriptor).Control & SE_DACL_PRESENT != 0)
    {
        ExplicitAcl = SepGetDaclFromDescriptor(ExplicitDescriptor);
        ExplicitPresent = 1;
        if (*ExplicitDescriptor).Control & SE_DACL_DEFAULTED != 0 {
            ExplicitDefaulted = 1;
        }
    }
    ParentAcl = core::ptr::null_mut();
    if !ParentDescriptor.is_null() &&
       ((*ParentDescriptor).Control & SE_DACL_PRESENT != 0)
    {
        ParentAcl = SepGetDaclFromDescriptor(ParentDescriptor);
    }
    Dacl = SepSelectAcl(
        ExplicitAcl,
        ExplicitPresent,
        ExplicitDefaulted,
        ParentAcl,
        (*Token).DefaultDacl,
        &mut DaclLength,
        Owner,
        Group,
        &mut DaclPresent,
        &mut DaclIsInherited,
        IsDirectoryObject,
        GenericMapping,
    );
    if DaclPresent != 0 {
        Control |= SE_DACL_PRESENT;
    }

    SaclLength = 0;
    ExplicitAcl = core::ptr::null_mut();
    ExplicitPresent = 0;
    ExplicitDefaulted = 0;
    if !ExplicitDescriptor.is_null() &&
       ((*ExplicitDescriptor).Control & SE_SACL_PRESENT != 0)
    {
        ExplicitAcl = SepGetSaclFromDescriptor(ExplicitDescriptor);
        ExplicitPresent = 1;
        if (*ExplicitDescriptor).Control & SE_SACL_DEFAULTED != 0 {
            ExplicitDefaulted = 1;
        }
    }
    ParentAcl = core::ptr::null_mut();
    if !ParentDescriptor.is_null() &&
       ((*ParentDescriptor).Control & SE_SACL_PRESENT != 0)
    {
        ParentAcl = SepGetSaclFromDescriptor(ParentDescriptor);
    }
    Sacl = SepSelectAcl(
        ExplicitAcl,
        ExplicitPresent,
        ExplicitDefaulted,
        ParentAcl,
        core::ptr::null_mut(),
        &mut SaclLength,
        Owner,
        Group,
        &mut SaclPresent,
        &mut SaclIsInherited,
        IsDirectoryObject,
        GenericMapping,
    );
    if SaclPresent != 0 {
        Control |= SE_SACL_PRESENT;
    }

    Length = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG +
        OwnerLength + GroupLength + DaclLength + SaclLength;

    Descriptor = ExAllocatePoolWithTag(PagedPool, Length as SIZE_T, TAG_SD) as PISECURITY_DESCRIPTOR_RELATIVE;
    if Descriptor.is_null() {
        SeUnlockSubjectContext(SubjectContext);
        core::ptr::write_volatile(NewDescriptor, core::ptr::null_mut());
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(Descriptor as *mut u8, 0, Length as usize);
    RtlCreateSecurityDescriptor(Descriptor as PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_REVISION);

    (*Descriptor).Control = Control | SE_SELF_RELATIVE;

    Current = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as ULONG;

    if SaclLength != 0 {
        Status = SepPropagateAcl(
            ((Descriptor as *mut u8).add(Current as usize)) as PACL,
            &mut SaclLength,
            Sacl,
            Owner,
            Group,
            SaclIsInherited,
            IsDirectoryObject,
            GenericMapping,
        );
        (*Descriptor).Sacl = Current;
        Current += SaclLength;
    }

    if DaclLength != 0 {
        Status = SepPropagateAcl(
            ((Descriptor as *mut u8).add(Current as usize)) as PACL,
            &mut DaclLength,
            Dacl,
            Owner,
            Group,
            DaclIsInherited,
            IsDirectoryObject,
            GenericMapping,
        );
        (*Descriptor).Dacl = Current;
        Current += DaclLength;
    }

    if OwnerLength != 0 {
        core::ptr::copy_nonoverlapping(
            Owner as *const u8,
            (Descriptor as *mut u8).add(Current as usize),
            OwnerLength as usize,
        );
        (*Descriptor).Owner = Current;
        Current += OwnerLength;
    }

    if GroupLength != 0 {
        core::ptr::copy_nonoverlapping(
            Group as *const u8,
            (Descriptor as *mut u8).add(Current as usize),
            GroupLength as usize,
        );
        (*Descriptor).Group = Current;
    }

    SeUnlockSubjectContext(SubjectContext);

    *NewDescriptor = Descriptor as PSECURITY_DESCRIPTOR;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeAssignSecurity(
    ParentDescriptor: PSECURITY_DESCRIPTOR,
    ExplicitDescriptor: PSECURITY_DESCRIPTOR,
    NewDescriptor: *mut PSECURITY_DESCRIPTOR,
    IsDirectoryObject: BOOLEAN,
    SubjectContext: PSECURITY_SUBJECT_CONTEXT,
    GenericMapping: PGENERIC_MAPPING,
    PoolType: POOL_TYPE,
) -> NTSTATUS {
    let ParentDescriptor = opaque_ptr(ParentDescriptor);
    let ExplicitDescriptor = opaque_ptr(ExplicitDescriptor);
    let NewDescriptor = opaque_ptr(NewDescriptor);
    let SubjectContext = opaque_ptr(SubjectContext);
    let GenericMapping = opaque_ptr(GenericMapping);
    SeAssignSecurityEx(
        ParentDescriptor,
        ExplicitDescriptor,
        NewDescriptor,
        core::ptr::null_mut(),
        IsDirectoryObject,
        0,
        SubjectContext,
        GenericMapping,
        PoolType,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SeComputeQuotaInformationSize(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    QuotaInfoSize: *mut ULONG,
) -> NTSTATUS {
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let QuotaInfoSize = opaque_ptr(QuotaInfoSize);
    let Group: PSID;
    let Dacl: PACL;

    *QuotaInfoSize = 0;

    if (*(SecurityDescriptor as PISECURITY_DESCRIPTOR)).Revision != SECURITY_DESCRIPTOR_REVISION1 as u8 {
        return STATUS_UNKNOWN_REVISION;
    }

    Group = SepGetGroupFromDescriptor(SecurityDescriptor as PISECURITY_DESCRIPTOR);
    Dacl = SepGetDaclFromDescriptor(SecurityDescriptor as PISECURITY_DESCRIPTOR);

    if !Group.is_null() {
        *QuotaInfoSize = (RtlLengthSid(Group) + 3) & !3 ;
    }

    if !Dacl.is_null() {
        *QuotaInfoSize += ((*Dacl).AclSize as ULONG + 3) & !3 ;
    }

    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Security descriptors (SDs) implementation support
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
// PSECURITY_DESCRIPTOR SePublicDefaultSd = NULL;
// PSECURITY_DESCRIPTOR SePublicDefaultUnrestrictedSd = NULL;
// PSECURITY_DESCRIPTOR SePublicOpenSd = NULL;
// PSECURITY_DESCRIPTOR SePublicOpenUnrestrictedSd = NULL;
// PSECURITY_DESCRIPTOR SeSystemDefaultSd = NULL;
// PSECURITY_DESCRIPTOR SeUnrestrictedSd = NULL;
// PSECURITY_DESCRIPTOR SeSystemAnonymousLogonSd = NULL;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /**
//  * @brief
//  * Initializes the known security descriptors in the system.
//  *
//  * @return
//  * Returns TRUE if all the security descriptors have been initialized,
//  * FALSE otherwise.
//  */
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// SepInitSDs(VOID)
// {
//     /* Create PublicDefaultSd */
//     SePublicDefaultSd = ExAllocatePoolWithTag(PagedPool,
//                                               sizeof(SECURITY_DESCRIPTOR), TAG_SD);
//     if (SePublicDefaultSd == NULL)
//         return FALSE;
//
//     RtlCreateSecurityDescriptor(SePublicDefaultSd,
//                                 SECURITY_DESCRIPTOR_REVISION);
//     RtlSetDaclSecurityDescriptor(SePublicDefaultSd,
//                                  TRUE,
//                                  SePublicDefaultDacl,
//                                  FALSE);
//
//     /* Create PublicDefaultUnrestrictedSd */
//     SePublicDefaultUnrestrictedSd = ExAllocatePoolWithTag(PagedPool,
//                                                           sizeof(SECURITY_DESCRIPTOR), TAG_SD);
//     if (SePublicDefaultUnrestrictedSd == NULL)
//         return FALSE;
//
//     RtlCreateSecurityDescriptor(SePublicDefaultUnrestrictedSd,
//                                 SECURITY_DESCRIPTOR_REVISION);
//     RtlSetDaclSecurityDescriptor(SePublicDefaultUnrestrictedSd,
//                                  TRUE,
//                                  SePublicDefaultUnrestrictedDacl,
//                                  FALSE);
//
//     /* Create PublicOpenSd */
//     SePublicOpenSd = ExAllocatePoolWithTag(PagedPool,
//                                            sizeof(SECURITY_DESCRIPTOR), TAG_SD);
//     if (SePublicOpenSd == NULL)
//         return FALSE;
//
//     RtlCreateSecurityDescriptor(SePublicOpenSd,
//                                 SECURITY_DESCRIPTOR_REVISION);
//     RtlSetDaclSecurityDescriptor(SePublicOpenSd,
//                                  TRUE,
//                                  SePublicOpenDacl,
//                                  FALSE);
//
//     /* Create PublicOpenUnrestrictedSd */
//     SePublicOpenUnrestrictedSd = ExAllocatePoolWithTag(PagedPool,
//                                                        sizeof(SECURITY_DESCRIPTOR), TAG_SD);
//     if (SePublicOpenUnrestrictedSd == NULL)
//         return FALSE;
//
//     RtlCreateSecurityDescriptor(SePublicOpenUnrestrictedSd,
//                                 SECURITY_DESCRIPTOR_REVISION);
//     RtlSetDaclSecurityDescriptor(SePublicOpenUnrestrictedSd,
//                                  TRUE,
//                                  SePublicOpenUnrestrictedDacl,
//                                  FALSE);
//
//     /* Create SystemDefaultSd */
//     SeSystemDefaultSd = ExAllocatePoolWithTag(PagedPool,
//                                               sizeof(SECURITY_DESCRIPTOR), TAG_SD);
//     if (SeSystemDefaultSd == NULL)
//         return FALSE;
//
//     RtlCreateSecurityDescriptor(SeSystemDefaultSd,
//                                 SECURITY_DESCRIPTOR_REVISION);
//     RtlSetDaclSecurityDescriptor(SeSystemDefaultSd,
//                                  TRUE,
//                                  SeSystemDefaultDacl,
//                                  FALSE);
//
//     /* Create UnrestrictedSd */
//     SeUnrestrictedSd = ExAllocatePoolWithTag(PagedPool,
//                                              sizeof(SECURITY_DESCRIPTOR), TAG_SD);
//     if (SeUnrestrictedSd == NULL)
//         return FALSE;
//
//     RtlCreateSecurityDescriptor(SeUnrestrictedSd,
//                                 SECURITY_DESCRIPTOR_REVISION);
//     RtlSetDaclSecurityDescriptor(SeUnrestrictedSd,
//                                  TRUE,
//                                  SeUnrestrictedDacl,
//                                  FALSE);
//
//     /* Create SystemAnonymousLogonSd */
//     SeSystemAnonymousLogonSd = ExAllocatePoolWithTag(PagedPool,
//                                                      sizeof(SECURITY_DESCRIPTOR), TAG_SD);
//     if (SeSystemAnonymousLogonSd == NULL)
//         return FALSE;
//
//     RtlCreateSecurityDescriptor(SeSystemAnonymousLogonSd,
//                                 SECURITY_DESCRIPTOR_REVISION);
//     RtlSetDaclSecurityDescriptor(SeSystemAnonymousLogonSd,
//                                  TRUE,
//                                  SeSystemAnonymousLogonDacl,
//                                  FALSE);
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Sets a "World" security descriptor.
//  *
//  * @param[in] SecurityInformation
//  * Security information details, alongside with the security
//  * descriptor to set the World SD.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor buffer.
//  *
//  * @param[in] BufferLength
//  * Length size of the buffer.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the World security descriptor has been
//  * set. STATUS_ACCESS_DENIED is returned if the caller hasn't
//  * provided security information details thus the SD cannot
//  * be set.
//  */
// NTSTATUS
// NTAPI
// SeSetWorldSecurityDescriptor(
//     _In_ SECURITY_INFORMATION SecurityInformation,
//     _In_ PISECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_ PULONG BufferLength)
// {
//     ULONG Current;
//     ULONG SidSize;
//     ULONG SdSize;
//     NTSTATUS Status;
//     PISECURITY_DESCRIPTOR_RELATIVE SdRel = (PISECURITY_DESCRIPTOR_RELATIVE)SecurityDescriptor;
//
//     DPRINT("SeSetWorldSecurityDescriptor() called\n");
//
//     if (SecurityInformation == 0)
//     {
//         return STATUS_ACCESS_DENIED;
//     }
//
//     /* calculate the minimum size of the buffer */
//     SidSize = RtlLengthSid(SeWorldSid);
//     SdSize = sizeof(SECURITY_DESCRIPTOR_RELATIVE);
//     if (SecurityInformation & OWNER_SECURITY_INFORMATION)
//         SdSize += SidSize;
//     if (SecurityInformation & GROUP_SECURITY_INFORMATION)
//         SdSize += SidSize;
//     if (SecurityInformation & DACL_SECURITY_INFORMATION)
//     {
//         SdSize += sizeof(ACL) + sizeof(ACE) + SidSize;
//     }
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//     {
//         SdSize += sizeof(ACL) + sizeof(ACE) + SidSize;
//     }
//
//     if (*BufferLength < SdSize)
//     {
//         *BufferLength = SdSize;
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     *BufferLength = SdSize;
//
//     Status = RtlCreateSecurityDescriptorRelative(SdRel,
//                                                  SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     Current = sizeof(SECURITY_DESCRIPTOR_RELATIVE);
//
//     if (SecurityInformation & OWNER_SECURITY_INFORMATION)
//     {
//         RtlCopyMemory((PUCHAR)SdRel + Current, SeWorldSid, SidSize);
//         SdRel->Owner = Current;
//         Current += SidSize;
//     }
//
//     if (SecurityInformation & GROUP_SECURITY_INFORMATION)
//     {
//         RtlCopyMemory((PUCHAR)SdRel + Current, SeWorldSid, SidSize);
//         SdRel->Group = Current;
//         Current += SidSize;
//     }
//
//     if (SecurityInformation & DACL_SECURITY_INFORMATION)
//     {
//         PACL Dacl = (PACL)((PUCHAR)SdRel + Current);
//
//         Status = RtlCreateAcl(Dacl,
//                               sizeof(ACL) + sizeof(ACE) + SidSize,
//                               ACL_REVISION);
//         if (!NT_SUCCESS(Status))
//             return Status;
//
//         Status = RtlAddAccessAllowedAce(Dacl,
//                                         ACL_REVISION,
//                                         GENERIC_ALL,
//                                         SeWorldSid);
//         if (!NT_SUCCESS(Status))
//             return Status;
//
//         SdRel->Control |= SE_DACL_PRESENT;
//         SdRel->Dacl = Current;
//         Current += SidSize;
//     }
//
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//     {
//         PACL Sacl = (PACL)((PUCHAR)SdRel + Current);
//
//         Status = RtlCreateAcl(Sacl,
//                               sizeof(ACL) + sizeof(ACE) + SidSize,
//                               ACL_REVISION);
//         if (!NT_SUCCESS(Status))
//             return Status;
//
//         Status = RtlAddAuditAccessAce(Sacl,
//                                       ACL_REVISION,
//                                       ACCESS_SYSTEM_SECURITY | STANDARD_RIGHTS_ALL,
//                                       SeWorldSid,
//                                       TRUE,
//                                       TRUE);
//         if (!NT_SUCCESS(Status))
//             return Status;
//
//         SdRel->Control |= SE_SACL_PRESENT;
//         SdRel->Sacl = Current;
//         Current += SidSize;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /**
//  * @brief
//  * Determines the size of a SID.
//  *
//  * @param[in] Sid
//  * A security identifier where its size is to be determined.
//  *
//  * @param[in,out] OutSAC
//  * The returned sub authority count of the security
//  * identifier.
//  *
//  * @param[in] ProcessorMode
//  * Processor level access mode.
//  *
//  * @return
//  * Returns the size length of a security identifier (SID).
//  */
// static
// ULONG
// DetermineSIDSize(
//     _In_ PISID Sid,
//     _Inout_ PULONG OutSAC,
//     _In_ KPROCESSOR_MODE ProcessorMode)
// {
//     ULONG Size;
//
//     if (!Sid)
//     {
//         *OutSAC = 0;
//         return 0;
//     }
//
//     if (ProcessorMode != KernelMode)
//     {
//         /* Securely access the buffers! */
//         *OutSAC = ProbeForReadUchar(&Sid->SubAuthorityCount);
//         Size = RtlLengthRequiredSid(*OutSAC);
//         ProbeForRead(Sid, Size, sizeof(ULONG));
//     }
//     else
//     {
//         *OutSAC = Sid->SubAuthorityCount;
//         Size = RtlLengthRequiredSid(*OutSAC);
//     }
//
//     return Size;
// }
//
// /**
//  * @brief
//  * Determines the size of an ACL.
//  *
//  * @param[in] Acl
//  * An access control list where its size is to be
//  * determined.
//  *
//  * @param[in] ProcessorMode
//  * Processor level access mode.
//  *
//  * @return
//  * Returns the size length of a an access control
//  * list (ACL).
//  */
// static
// ULONG
// DetermineACLSize(
//     _In_ PACL Acl,
//     _In_ KPROCESSOR_MODE ProcessorMode)
// {
//     ULONG Size;
//
//     if (!Acl) return 0;
//
//     if (ProcessorMode == KernelMode) return Acl->AclSize;
//
//     /* Probe the buffers! */
//     Size = ProbeForReadUshort(&Acl->AclSize);
//     ProbeForRead(Acl, Size, sizeof(ULONG));
//
//     return Size;
// }
//
// /**
//  * @brief
//  * Captures a security descriptor.
//  *
//  * @param[in] _OriginalSecurityDescriptor
//  * An already existing and valid security descriptor
//  * to be captured.
//  *
//  * @param[in] CurrentMode
//  * Processor level access mode.
//  *
//  * @param[in] PoolType
//  * Pool type to be used when allocating the captured
//  * buffer.
//  *
//  * @param[in] CaptureIfKernel
//  * Set this to TRUE if capturing is done within the
//  * kernel.
//  *
//  * @param[out] CapturedSecurityDescriptor
//  * The captured security descriptor.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the operations have been
//  * completed successfully and that the security descriptor
//  * has been captured. STATUS_UNKNOWN_REVISION is returned
//  * if the security descriptor has an unknown revision.
//  * STATUS_INSUFFICIENT_RESOURCES is returned if memory
//  * pool allocation for the captured buffer has failed.
//  * A failure NTSTATUS code is returned otherwise.
//  */
// NTSTATUS
// NTAPI
// SeCaptureSecurityDescriptor(
//     _In_ PSECURITY_DESCRIPTOR _OriginalSecurityDescriptor,
//     _In_ KPROCESSOR_MODE CurrentMode,
//     _In_ POOL_TYPE PoolType,
//     _In_ BOOLEAN CaptureIfKernel,
//     _Out_ PSECURITY_DESCRIPTOR *CapturedSecurityDescriptor)
// {
//     PISECURITY_DESCRIPTOR OriginalDescriptor = _OriginalSecurityDescriptor;
//     SECURITY_DESCRIPTOR DescriptorCopy;
//     PISECURITY_DESCRIPTOR_RELATIVE NewDescriptor;
//     ULONG OwnerSAC = 0, GroupSAC = 0;
//     ULONG OwnerSize = 0, GroupSize = 0;
//     ULONG SaclSize = 0, DaclSize = 0;
//     ULONG DescriptorSize = 0;
//     ULONG Offset;
//
//     if (!OriginalDescriptor)
//     {
//         /* Nothing to do... */
//         *CapturedSecurityDescriptor = NULL;
//         return STATUS_SUCCESS;
//     }
//
//     /* Quick path */
//     if (CurrentMode == KernelMode && !CaptureIfKernel)
//     {
//         /* Check descriptor version */
//         if (OriginalDescriptor->Revision != SECURITY_DESCRIPTOR_REVISION1 as u8)
//         {
//             return STATUS_UNKNOWN_REVISION;
//         }
//
//         *CapturedSecurityDescriptor = _OriginalSecurityDescriptor;
//         return STATUS_SUCCESS;
//     }
//
//     _SEH2_TRY
//     {
//         if (CurrentMode != KernelMode)
//         {
//             ProbeForRead(OriginalDescriptor,
//                          sizeof(SECURITY_DESCRIPTOR_RELATIVE),
//                          sizeof(ULONG));
//         }
//
//         /* Check the descriptor version */
//         if (OriginalDescriptor->Revision != SECURITY_DESCRIPTOR_REVISION1 as u8)
//         {
//             _SEH2_YIELD(return STATUS_UNKNOWN_REVISION);
//         }
//
//         if (CurrentMode != KernelMode)
//         {
//             /* Get the size of the descriptor */
//             DescriptorSize = (OriginalDescriptor->Control & SE_SELF_RELATIVE) ?
//                 sizeof(SECURITY_DESCRIPTOR_RELATIVE) : sizeof(SECURITY_DESCRIPTOR);
//
//             /* Probe the entire security descriptor structure. The SIDs
//              * and ACLs will be probed and copied later though */
//             ProbeForRead(OriginalDescriptor, DescriptorSize, sizeof(ULONG));
//         }
//
//         /* Now capture all fields and convert to an absolute descriptor */
//         DescriptorCopy.Revision = OriginalDescriptor->Revision;
//         DescriptorCopy.Sbz1 = OriginalDescriptor->Sbz1;
//         DescriptorCopy.Control = OriginalDescriptor->Control & ~SE_SELF_RELATIVE;
//         DescriptorCopy.Owner = SepGetOwnerFromDescriptor(OriginalDescriptor);
//         DescriptorCopy.Group = SepGetGroupFromDescriptor(OriginalDescriptor);
//         DescriptorCopy.Sacl = SepGetSaclFromDescriptor(OriginalDescriptor);
//         DescriptorCopy.Dacl = SepGetDaclFromDescriptor(OriginalDescriptor);
//         DescriptorSize = sizeof(SECURITY_DESCRIPTOR_RELATIVE);
//
//         /* Determine owner and group sizes */
//         OwnerSize = DetermineSIDSize(DescriptorCopy.Owner, &OwnerSAC, CurrentMode);
//         DescriptorSize += ROUND_UP(OwnerSize, sizeof(ULONG));
//         GroupSize = DetermineSIDSize(DescriptorCopy.Group, &GroupSAC, CurrentMode);
//         DescriptorSize += ROUND_UP(GroupSize, sizeof(ULONG));
//
//         /* Determine the size of the ACLs */
//         if (DescriptorCopy.Control & SE_SACL_PRESENT)
//         {
//             /* Get the size and probe if user mode */
//             SaclSize = DetermineACLSize(DescriptorCopy.Sacl, CurrentMode);
//             DescriptorSize += ROUND_UP(SaclSize, sizeof(ULONG));
//         }
//
//         if (DescriptorCopy.Control & SE_DACL_PRESENT)
//         {
//             /* Get the size and probe if user mode */
//             DaclSize = DetermineACLSize(DescriptorCopy.Dacl, CurrentMode);
//             DescriptorSize += ROUND_UP(DaclSize, sizeof(ULONG));
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /*
//      * Allocate enough memory to store a complete copy of a self-relative
//      * security descriptor
//      */
//     NewDescriptor = ExAllocatePoolWithTag(PoolType,
//                                           DescriptorSize,
//                                           TAG_SD);
//     if (!NewDescriptor) return STATUS_INSUFFICIENT_RESOURCES;
//
//     RtlZeroMemory(NewDescriptor, DescriptorSize);
//     NewDescriptor->Revision = DescriptorCopy.Revision;
//     NewDescriptor->Sbz1 = DescriptorCopy.Sbz1;
//     NewDescriptor->Control = DescriptorCopy.Control | SE_SELF_RELATIVE;
//
//     _SEH2_TRY
//     {
//         /*
//          * Setup the offsets and copy the SIDs and ACLs to the new
//          * self-relative security descriptor. Probing the pointers is not
//          * neccessary anymore as we did that when collecting the sizes!
//          * Make sure to validate the SIDs and ACLs *again* as they could have
//          * been modified in the meanwhile!
//          */
//         Offset = sizeof(SECURITY_DESCRIPTOR_RELATIVE);
//
//         if (DescriptorCopy.Owner)
//         {
//             if (!RtlValidSid(DescriptorCopy.Owner)) RtlRaiseStatus(STATUS_INVALID_SID);
//             NewDescriptor->Owner = Offset;
//             RtlCopyMemory((PUCHAR)NewDescriptor + Offset,
//                           DescriptorCopy.Owner,
//                           OwnerSize);
//             Offset += ROUND_UP(OwnerSize, sizeof(ULONG));
//         }
//
//         if (DescriptorCopy.Group)
//         {
//             if (!RtlValidSid(DescriptorCopy.Group)) RtlRaiseStatus(STATUS_INVALID_SID);
//             NewDescriptor->Group = Offset;
//             RtlCopyMemory((PUCHAR)NewDescriptor + Offset,
//                           DescriptorCopy.Group,
//                           GroupSize);
//             Offset += ROUND_UP(GroupSize, sizeof(ULONG));
//         }
//
//         if (DescriptorCopy.Sacl)
//         {
//             if (!RtlValidAcl(DescriptorCopy.Sacl)) RtlRaiseStatus(STATUS_INVALID_ACL);
//             NewDescriptor->Sacl = Offset;
//             RtlCopyMemory((PUCHAR)NewDescriptor + Offset,
//                           DescriptorCopy.Sacl,
//                           SaclSize);
//             Offset += ROUND_UP(SaclSize, sizeof(ULONG));
//         }
//
//         if (DescriptorCopy.Dacl)
//         {
//             if (!RtlValidAcl(DescriptorCopy.Dacl)) RtlRaiseStatus(STATUS_INVALID_ACL);
//             NewDescriptor->Dacl = Offset;
//             RtlCopyMemory((PUCHAR)NewDescriptor + Offset,
//                           DescriptorCopy.Dacl,
//                           DaclSize);
//             Offset += ROUND_UP(DaclSize, sizeof(ULONG));
//         }
//
//         /* Make sure the size was correct */
//         ASSERT(Offset == DescriptorSize);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* We failed to copy the data to the new descriptor */
//         ExFreePoolWithTag(NewDescriptor, TAG_SD);
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /*
//      * We're finally done!
//      * Copy the pointer to the captured descriptor to to the caller.
//      */
//     *CapturedSecurityDescriptor = NewDescriptor;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Queries information details about a security
//  * descriptor.
//  *
//  * @param[in] SecurityInformation
//  * Security information details to be queried
//  * from a security descriptor.
//  *
//  * @param[out] SecurityDescriptor
//  * The returned security descriptor with security information
//  * data.
//  *
//  * @param[in,out] Length
//  * The returned length of a security descriptor.
//  *
//  * @param[in,out] ObjectsSecurityDescriptor
//  * The returned object security descriptor.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the operations have been
//  * completed successfully and that the specific information
//  * about the security descriptor has been queried.
//  * STATUS_BUFFER_TOO_SMALL is returned if the buffer size
//  * is too small to contain the queried info about the
//  * security descriptor.
//  */
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// NTAPI
// SeQuerySecurityDescriptorInfo(
//     _In_ PSECURITY_INFORMATION SecurityInformation,
//     _Out_writes_bytes_(*Length) PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _Inout_ PULONG Length,
//     _Inout_ PSECURITY_DESCRIPTOR *ObjectsSecurityDescriptor)
// {
//     PISECURITY_DESCRIPTOR ObjectSd;
//     PISECURITY_DESCRIPTOR_RELATIVE RelSD;
//     PSID Owner = NULL;
//     PSID Group = NULL;
//     PACL Dacl = NULL;
//     PACL Sacl = NULL;
//     ULONG OwnerLength = 0;
//     ULONG GroupLength = 0;
//     ULONG DaclLength = 0;
//     ULONG SaclLength = 0;
//     SECURITY_DESCRIPTOR_CONTROL Control = 0;
//     ULONG_PTR Current;
//     ULONG SdLength;
//
//     PAGED_CODE();
//
//     RelSD = (PISECURITY_DESCRIPTOR_RELATIVE)SecurityDescriptor;
//
//     if (*ObjectsSecurityDescriptor == NULL)
//     {
//         if (*Length < sizeof(SECURITY_DESCRIPTOR_RELATIVE))
//         {
//             *Length = sizeof(SECURITY_DESCRIPTOR_RELATIVE);
//             return STATUS_BUFFER_TOO_SMALL;
//         }
//
//         *Length = sizeof(SECURITY_DESCRIPTOR_RELATIVE);
//         RtlCreateSecurityDescriptorRelative(RelSD,
//                                             SECURITY_DESCRIPTOR_REVISION);
//         return STATUS_SUCCESS;
//     }
//
//     ObjectSd = *ObjectsSecurityDescriptor;
//
//     /* Calculate the required security descriptor length */
//     Control = SE_SELF_RELATIVE;
//     if (*SecurityInformation & OWNER_SECURITY_INFORMATION)
//     {
//         Owner = SepGetOwnerFromDescriptor(ObjectSd);
//         if (Owner != NULL)
//         {
//             OwnerLength = ROUND_UP(RtlLengthSid(Owner), 4);
//             Control |= (ObjectSd->Control & SE_OWNER_DEFAULTED);
//         }
//     }
//
//     if (*SecurityInformation & GROUP_SECURITY_INFORMATION)
//     {
//         Group = SepGetGroupFromDescriptor(ObjectSd);
//         if (Group != NULL)
//         {
//             GroupLength = ROUND_UP(RtlLengthSid(Group), 4);
//             Control |= (ObjectSd->Control & SE_GROUP_DEFAULTED);
//         }
//     }
//
//     if ((*SecurityInformation & DACL_SECURITY_INFORMATION) &&
//         (ObjectSd->Control & SE_DACL_PRESENT))
//     {
//         Dacl = SepGetDaclFromDescriptor(ObjectSd);
//         if (Dacl != NULL)
//         {
//             DaclLength = ROUND_UP((ULONG)Dacl->AclSize, 4);
//         }
//
//         Control |= (ObjectSd->Control & (SE_DACL_DEFAULTED | SE_DACL_PRESENT));
//     }
//
//     if ((*SecurityInformation & SACL_SECURITY_INFORMATION) &&
//         (ObjectSd->Control & SE_SACL_PRESENT))
//     {
//         Sacl = SepGetSaclFromDescriptor(ObjectSd);
//         if (Sacl != NULL)
//         {
//             SaclLength = ROUND_UP(Sacl->AclSize, 4);
//         }
//
//         Control |= (ObjectSd->Control & (SE_SACL_DEFAULTED | SE_SACL_PRESENT));
//     }
//
//     SdLength = OwnerLength + GroupLength + DaclLength +
//     SaclLength + sizeof(SECURITY_DESCRIPTOR_RELATIVE);
//     if (*Length < SdLength)
//     {
//         *Length = SdLength;
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* Build the new security descrtiptor */
//     RtlCreateSecurityDescriptorRelative(RelSD,
//                                         SECURITY_DESCRIPTOR_REVISION);
//     RelSD->Control = Control;
//
//     Current = (ULONG_PTR)(RelSD + 1);
//
//     if (OwnerLength != 0)
//     {
//         RtlCopyMemory((PVOID)Current,
//                       Owner,
//                       OwnerLength);
//         RelSD->Owner = (ULONG)(Current - (ULONG_PTR)SecurityDescriptor);
//         Current += OwnerLength;
//     }
//
//     if (GroupLength != 0)
//     {
//         RtlCopyMemory((PVOID)Current,
//                       Group,
//                       GroupLength);
//         RelSD->Group = (ULONG)(Current - (ULONG_PTR)SecurityDescriptor);
//         Current += GroupLength;
//     }
//
//     if (DaclLength != 0)
//     {
//         RtlCopyMemory((PVOID)Current,
//                       Dacl,
//                       DaclLength);
//         RelSD->Dacl = (ULONG)(Current - (ULONG_PTR)SecurityDescriptor);
//         Current += DaclLength;
//     }
//
//     if (SaclLength != 0)
//     {
//         RtlCopyMemory((PVOID)Current,
//                       Sacl,
//                       SaclLength);
//         RelSD->Sacl = (ULONG)(Current - (ULONG_PTR)SecurityDescriptor);
//         Current += SaclLength;
//     }
//
//     *Length = SdLength;
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Releases a captured security descriptor buffer.
//  *
//  * @param[in] CapturedSecurityDescriptor
//  * The captured security descriptor to be freed.
//  *
//  * @param[in] CurrentMode
//  * Processor level access mode.
//  *
//  * @param[in] CaptureIfKernelMode
//  * Set this to TRUE if the releasing is to be done within
//  * the kernel.
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  */
// NTSTATUS
// NTAPI
// SeReleaseSecurityDescriptor(
//     _In_ PSECURITY_DESCRIPTOR CapturedSecurityDescriptor,
//     _In_ KPROCESSOR_MODE CurrentMode,
//     _In_ BOOLEAN CaptureIfKernelMode)
// {
//     PAGED_CODE();
//
//     /*
//      * WARNING! You need to call this function with the same value for CurrentMode
//      * and CaptureIfKernelMode that you previously passed to
//      * SeCaptureSecurityDescriptor() in order to avoid memory leaks!
//      */
//     if (CapturedSecurityDescriptor != NULL &&
//         (CurrentMode != KernelMode ||
//          (CurrentMode == KernelMode && CaptureIfKernelMode)))
//     {
//         /* Only delete the descriptor when SeCaptureSecurityDescriptor() allocated one! */
//         ExFreePoolWithTag(CapturedSecurityDescriptor, TAG_SD);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Modifies some information data about a security
//  * descriptor.
//  *
//  * @param[in] Object
//  * If specified, the function will use this arbitrary
//  * object that points to an object security descriptor.
//  *
//  * @param[in] SecurityInformation
//  * Security information details to be set.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor where its info is to be changed.
//  *
//  * @param[in,out] ObjectsSecurityDescriptor
//  * The returned pointer to security descriptor objects.
//  *
//  * @param[in] PoolType
//  * Pool type for the new security descriptor to allocate.
//  *
//  * @param[in] GenericMapping
//  * The generic mapping of access rights masks.
//  *
//  * @return
//  * See SeSetSecurityDescriptorInfoEx.
//  */
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// NTAPI
// SeSetSecurityDescriptorInfo(
//     _In_opt_ PVOID Object,
//     _In_ PSECURITY_INFORMATION SecurityInformation,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _Inout_ PSECURITY_DESCRIPTOR *ObjectsSecurityDescriptor,
//     _In_ POOL_TYPE PoolType,
//     _In_ PGENERIC_MAPPING GenericMapping)
// {
//     PAGED_CODE();
//
//     return SeSetSecurityDescriptorInfoEx(Object,
//                                          SecurityInformation,
//                                          SecurityDescriptor,
//                                          ObjectsSecurityDescriptor,
//                                          0,
//                                          PoolType,
//                                          GenericMapping);
// }
//
// /**
//  * @brief
//  * An extended function that sets new information data to
//  * a security descriptor.
//  *
//  * @param[in] Object
//  * If specified, the function will use this arbitrary
//  * object that points to an object security descriptor.
//  *
//  * @param[in] SecurityInformation
//  * Security information details to be set.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor where its info is to be changed.
//  *
//  * @param[in,out] ObjectsSecurityDescriptor
//  * The returned pointer to security descriptor objects.
//  *
//  * @param[in] AutoInheritFlags
//  * Flags bitmask inheritation, influencing how the security
//  * descriptor can be inherited and if it can be in the first
//  * place.
//  *
//  * @param[in] PoolType
//  * Pool type for the new security descriptor to allocate.
//  *
//  * @param[in] GenericMapping
//  * The generic mapping of access rights masks.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the operations have been
//  * completed without problems and that new info has been
//  * set to the security descriptor. STATUS_NO_SECURITY_ON_OBJECT
//  * is returned if the object does not have a security descriptor.
//  * STATUS_INSUFFICIENT_RESOURCES is returned if memory pool allocation
//  * for the new security descriptor with new info set has failed.
//  */
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// NTAPI
// SeSetSecurityDescriptorInfoEx(
//     _In_opt_ PVOID Object,
//     _In_ PSECURITY_INFORMATION _SecurityInformation,
//     _In_ PSECURITY_DESCRIPTOR _SecurityDescriptor,
//     _Inout_ PSECURITY_DESCRIPTOR *ObjectsSecurityDescriptor,
//     _In_ ULONG AutoInheritFlags,
//     _In_ POOL_TYPE PoolType,
//     _In_ PGENERIC_MAPPING GenericMapping)
// {
//     PISECURITY_DESCRIPTOR_RELATIVE ObjectSd;
//     PISECURITY_DESCRIPTOR_RELATIVE NewSd;
//     PISECURITY_DESCRIPTOR SecurityDescriptor = _SecurityDescriptor;
//     PSID Owner;
//     PSID Group;
//     PACL Dacl;
//     PACL Sacl;
//     ULONG OwnerLength;
//     ULONG GroupLength;
//     ULONG DaclLength;
//     ULONG SaclLength;
//     SECURITY_DESCRIPTOR_CONTROL Control = 0;
//     ULONG Current;
//     SECURITY_INFORMATION SecurityInformation;
//
//     PAGED_CODE();
//
//     ObjectSd = *ObjectsSecurityDescriptor;
//
//     /* The object does not have a security descriptor. */
//     if (!ObjectSd)
//         return STATUS_NO_SECURITY_ON_OBJECT;
//
//     ASSERT(ObjectSd->Control & SE_SELF_RELATIVE);
//
//     SecurityInformation = *_SecurityInformation;
//
//     /* Get owner and owner size */
//     if (SecurityInformation & OWNER_SECURITY_INFORMATION)
//     {
//         Owner = SepGetOwnerFromDescriptor(SecurityDescriptor);
//         Control |= (SecurityDescriptor->Control & SE_OWNER_DEFAULTED);
//     }
//     else
//     {
//         Owner = SepGetOwnerFromDescriptor(ObjectSd);
//         Control |= (ObjectSd->Control & SE_OWNER_DEFAULTED);
//     }
//     OwnerLength = Owner ? RtlLengthSid(Owner) : 0;
//     ASSERT(OwnerLength % sizeof(ULONG) == 0);
//
//     /* Get group and group size */
//     if (SecurityInformation & GROUP_SECURITY_INFORMATION)
//     {
//         Group = SepGetGroupFromDescriptor(SecurityDescriptor);
//         Control |= (SecurityDescriptor->Control & SE_GROUP_DEFAULTED);
//     }
//     else
//     {
//         Group = SepGetGroupFromDescriptor(ObjectSd);
//         Control |= (ObjectSd->Control & SE_GROUP_DEFAULTED);
//     }
//     GroupLength = Group ? RtlLengthSid(Group) : 0;
//     ASSERT(GroupLength % sizeof(ULONG) == 0);
//
//     /* Get DACL and DACL size */
//     if (SecurityInformation & DACL_SECURITY_INFORMATION)
//     {
//         Dacl = SepGetDaclFromDescriptor(SecurityDescriptor);
//         Control |= (SecurityDescriptor->Control & (SE_DACL_DEFAULTED | SE_DACL_PRESENT));
//     }
//     else
//     {
//         Dacl = SepGetDaclFromDescriptor(ObjectSd);
//         Control |= (ObjectSd->Control & (SE_DACL_DEFAULTED | SE_DACL_PRESENT));
//     }
//     DaclLength = Dacl ? ROUND_UP((ULONG)Dacl->AclSize, 4) : 0;
//
//     /* Get SACL and SACL size */
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//     {
//         Sacl = SepGetSaclFromDescriptor(SecurityDescriptor);
//         Control |= (SecurityDescriptor->Control & (SE_SACL_DEFAULTED | SE_SACL_PRESENT));
//     }
//     else
//     {
//         Sacl = SepGetSaclFromDescriptor(ObjectSd);
//         Control |= (ObjectSd->Control & (SE_SACL_DEFAULTED | SE_SACL_PRESENT));
//     }
//     SaclLength = Sacl ? ROUND_UP((ULONG)Sacl->AclSize, 4) : 0;
//
//     NewSd = ExAllocatePoolWithTag(PoolType,
//                                   sizeof(SECURITY_DESCRIPTOR_RELATIVE) +
//                                   OwnerLength + GroupLength +
//                                   DaclLength + SaclLength,
//                                   TAG_SD);
//     if (NewSd == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlCreateSecurityDescriptorRelative(NewSd, SECURITY_DESCRIPTOR_REVISION1);
//
//     Current = sizeof(SECURITY_DESCRIPTOR_RELATIVE);
//
//     if (OwnerLength != 0)
//     {
//         RtlCopyMemory((PUCHAR)NewSd + Current, Owner, OwnerLength);
//         NewSd->Owner = Current;
//         Current += OwnerLength;
//     }
//
//     if (GroupLength != 0)
//     {
//         RtlCopyMemory((PUCHAR)NewSd + Current, Group, GroupLength);
//         NewSd->Group = Current;
//         Current += GroupLength;
//     }
//
//     if (DaclLength != 0)
//     {
//         RtlCopyMemory((PUCHAR)NewSd + Current, Dacl, DaclLength);
//         NewSd->Dacl = Current;
//         Current += DaclLength;
//     }
//
//     if (SaclLength != 0)
//     {
//         RtlCopyMemory((PUCHAR)NewSd + Current, Sacl, SaclLength);
//         NewSd->Sacl = Current;
//         Current += SaclLength;
//     }
//
//     NewSd->Control |= Control;
//     *ObjectsSecurityDescriptor = NewSd;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Determines if a security descriptor is valid according
//  * to the general security requirements and conditions
//  * set by the kernel.
//  *
//  * @param[in] Length
//  * The length of a security descriptor.
//  *
//  * @param[in] _SecurityDescriptor
//  * A security descriptor where its properties are to be
//  * checked for validity.
//  *
//  * @return
//  * Returns TRUE if the given security descriptor is valid,
//  * FALSE otherwise.
//  */
// BOOLEAN NTAPI
// SeValidSecurityDescriptor(
//     _In_ ULONG Length,
//     _In_ PSECURITY_DESCRIPTOR _SecurityDescriptor)
// {
//     ULONG SdLength;
//     PISID Sid;
//     PACL Acl;
//     PISECURITY_DESCRIPTOR_RELATIVE SecurityDescriptor = _SecurityDescriptor;
//
//     if (Length < SECURITY_DESCRIPTOR_MIN_LENGTH)
//     {
//         DPRINT1("Invalid Security Descriptor length\n");
//         return FALSE;
//     }
//
//     if (SecurityDescriptor->Revision != SECURITY_DESCRIPTOR_REVISION1 as u8)
//     {
//         DPRINT1("Invalid Security Descriptor revision\n");
//         return FALSE;
//     }
//
//     if (!(SecurityDescriptor->Control & SE_SELF_RELATIVE))
//     {
//         DPRINT1("No self-relative Security Descriptor\n");
//         return FALSE;
//     }
//
//     SdLength = sizeof(*SecurityDescriptor);
//
//     /* Check Owner SID */
//     if (!SecurityDescriptor->Owner)
//     {
//         DPRINT1("No Owner SID\n");
//         return FALSE;
//     }
//
//     if (SecurityDescriptor->Owner % sizeof(ULONG))
//     {
//         DPRINT1("Invalid Owner SID alignment\n");
//         return FALSE;
//     }
//
//     /* Ensure the Owner SID is within the bounds of the security descriptor */
//     if ((SecurityDescriptor->Owner > Length) ||
//         (Length - SecurityDescriptor->Owner < sizeof(SID)))
//     {
//         DPRINT1("Owner SID not within bounds\n");
//         return FALSE;
//     }
//
//     /* Reference it */
//     Sid = (PISID)((ULONG_PTR)SecurityDescriptor + SecurityDescriptor->Owner);
//     if (Sid->Revision != SID_REVISION as u8)
//     {
//         DPRINT1("Invalid Owner SID revision\n");
//         return FALSE;
//     }
//
//     // NOTE: Same as SeLengthSid(Sid); but doesn't use hardcoded values.
//     SdLength += (sizeof(SID) + (Sid->SubAuthorityCount - 1) * sizeof(ULONG));
//     if (Length < SdLength)
//     {
//         DPRINT1("Invalid Owner SID size\n");
//         return FALSE;
//     }
//
//     /* Check Group SID */
//     if (SecurityDescriptor->Group)
//     {
//         if (SecurityDescriptor->Group % sizeof(ULONG))
//         {
//             DPRINT1("Invalid Group SID alignment\n");
//             return FALSE;
//         }
//
//         /* Ensure the Group SID is within the bounds of the security descriptor */
//         if ((SecurityDescriptor->Group > Length) ||
//             (Length - SecurityDescriptor->Group < sizeof(SID)))
//         {
//             DPRINT1("Group SID not within bounds\n");
//             return FALSE;
//         }
//
//         /* Reference it */
//         Sid = (PSID)((ULONG_PTR)SecurityDescriptor + SecurityDescriptor->Group);
//         if (Sid->Revision != SID_REVISION as u8)
//         {
//             DPRINT1("Invalid Group SID revision\n");
//             return FALSE;
//         }
//
//         // NOTE: Same as SeLengthSid(Sid); but doesn't use hardcoded values.
//         SdLength += (sizeof(SID) + (Sid->SubAuthorityCount - 1) * sizeof(ULONG));
//         if (Length < SdLength)
//         {
//             DPRINT1("Invalid Group SID size\n");
//             return FALSE;
//         }
//     }
//
//     /* Check DACL */
//     if (SecurityDescriptor->Dacl)
//     {
//         if (SecurityDescriptor->Dacl % sizeof(ULONG))
//         {
//             DPRINT1("Invalid DACL alignment\n");
//             return FALSE;
//         }
//
//         /* Ensure the DACL is within the bounds of the security descriptor */
//         if ((SecurityDescriptor->Dacl > Length) ||
//             (Length - SecurityDescriptor->Dacl < sizeof(ACL)))
//         {
//             DPRINT1("DACL not within bounds\n");
//             return FALSE;
//         }
//
//         /* Reference it */
//         Acl = (PACL)((ULONG_PTR)SecurityDescriptor + SecurityDescriptor->Dacl);
//
//         SdLength += Acl->AclSize;
//         if (Length < SdLength)
//         {
//             DPRINT1("Invalid DACL size\n");
//             return FALSE;
//         }
//
//         if (!RtlValidAcl(Acl))
//         {
//             DPRINT1("Invalid DACL\n");
//             return FALSE;
//         }
//     }
//
//     /* Check SACL */
//     if (SecurityDescriptor->Sacl)
//     {
//         if (SecurityDescriptor->Sacl % sizeof(ULONG))
//         {
//             DPRINT1("Invalid SACL alignment\n");
//             return FALSE;
//         }
//
//         /* Ensure the SACL is within the bounds of the security descriptor */
//         if ((SecurityDescriptor->Sacl > Length) ||
//             (Length - SecurityDescriptor->Sacl < sizeof(ACL)))
//         {
//             DPRINT1("SACL not within bounds\n");
//             return FALSE;
//         }
//
//         /* Reference it */
//         Acl = (PACL)((ULONG_PTR)SecurityDescriptor + SecurityDescriptor->Sacl);
//
//         SdLength += Acl->AclSize;
//         if (Length < SdLength)
//         {
//             DPRINT1("Invalid SACL size\n");
//             return FALSE;
//         }
//
//         if (!RtlValidAcl(Acl))
//         {
//             DPRINT1("Invalid SACL\n");
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Frees a security descriptor.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor to be freed from memory.
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  */
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// NTAPI
// SeDeassignSecurity(
//     _Inout_ PSECURITY_DESCRIPTOR *SecurityDescriptor)
// {
//     PAGED_CODE();
//
//     if (*SecurityDescriptor != NULL)
//     {
//         ExFreePoolWithTag(*SecurityDescriptor, TAG_SD);
//         *SecurityDescriptor = NULL;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * An extended function that assigns a security descriptor for a new
//  * object.
//  *
//  * @param[in] _ParentDescriptor
//  * A security descriptor of the parent object that is being
//  * created.
//  *
//  * @param[in] _ExplicitDescriptor
//  * An explicit security descriptor that is applied to a new
//  * object.
//  *
//  * @param[out] NewDescriptor
//  * The new allocated security descriptor.
//  *
//  * @param[in] ObjectType
//  * The type of the new object.
//  *
//  * @param[in] IsDirectoryObject
//  * Set this to TRUE if the newly created object is a directory
//  * object, otherwise set this to FALSE.
//  *
//  * @param[in] AutoInheritFlags
//  * Automatic inheritance flags that influence how access control
//  * entries within ACLs from security descriptors are inherited.
//  *
//  * @param[in] SubjectContext
//  * Security subject context of the new object.
//  *
//  * @param[in] GenericMapping
//  * Generic mapping of access mask rights.
//  *
//  * @param[in] PoolType
//  * This parameter is unused.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the operations have been completed
//  * successfully and that the security descriptor has been
//  * assigned to the new object. STATUS_NO_TOKEN is returned
//  * if the caller hasn't supplied a valid argument to a security
//  * subject context. STATUS_INVALID_OWNER is returned if the caller
//  * hasn't supplied a parent descriptor that belongs to the main
//  * user (owner). STATUS_INVALID_PRIMARY_GROUP is returned
//  * by the same reason as with the previous NTSTATUS code.
//  * The two NTSTATUS codes are returned if the calling thread
//  * stated that the owner and/or group is defaulted to the
//  * parent descriptor (SEF_DEFAULT_OWNER_FROM_PARENT and/or
//  * SEF_DEFAULT_GROUP_FROM_PARENT respectively).
//  * STATUS_INSUFFICIENT_RESOURCES is returned if memory pool allocation
//  * for the descriptor buffer has failed. A failure NTSTATUS is returned
//  * otherwise.
//  */
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// NTAPI
// SeAssignSecurityEx(
//     _In_opt_ PSECURITY_DESCRIPTOR _ParentDescriptor,
//     _In_opt_ PSECURITY_DESCRIPTOR _ExplicitDescriptor,
//     _Out_ PSECURITY_DESCRIPTOR *NewDescriptor,
//     _In_opt_ GUID *ObjectType,
//     _In_ BOOLEAN IsDirectoryObject,
//     _In_ ULONG AutoInheritFlags,
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext,
//     _In_ PGENERIC_MAPPING GenericMapping,
//     _In_ POOL_TYPE PoolType)
// {
//     PISECURITY_DESCRIPTOR ParentDescriptor = _ParentDescriptor;
//     PISECURITY_DESCRIPTOR ExplicitDescriptor = _ExplicitDescriptor;
//     PISECURITY_DESCRIPTOR_RELATIVE Descriptor;
//     PTOKEN Token;
//     ULONG OwnerLength;
//     ULONG GroupLength;
//     ULONG DaclLength;
//     ULONG SaclLength;
//     ULONG Length;
//     SECURITY_DESCRIPTOR_CONTROL Control = 0;
//     ULONG Current;
//     PSID Owner = NULL;
//     PSID Group = NULL;
//     PACL ExplicitAcl;
//     BOOLEAN ExplicitPresent;
//     BOOLEAN ExplicitDefaulted;
//     PACL ParentAcl;
//     PACL Dacl = NULL;
//     PACL Sacl = NULL;
//     BOOLEAN DaclIsInherited;
//     BOOLEAN SaclIsInherited;
//     BOOLEAN DaclPresent;
//     BOOLEAN SaclPresent;
//     NTSTATUS Status;
//
//     DBG_UNREFERENCED_PARAMETER(ObjectType);
//     DBG_UNREFERENCED_PARAMETER(AutoInheritFlags);
//     UNREFERENCED_PARAMETER(PoolType);
//
//     PAGED_CODE();
//
//     *NewDescriptor = NULL;
//
//     if (!ARGUMENT_PRESENT(SubjectContext))
//     {
//         return STATUS_NO_TOKEN;
//     }
//
//     /* Lock subject context */
//     SeLockSubjectContext(SubjectContext);
//
//     if (SubjectContext->ClientToken != NULL)
//     {
//         Token = SubjectContext->ClientToken;
//     }
//     else
//     {
//         Token = SubjectContext->PrimaryToken;
//     }
//
//     /* Inherit the Owner SID */
//     if (ExplicitDescriptor != NULL)
//     {
//         DPRINT("Use explicit owner sid!\n");
//         Owner = SepGetOwnerFromDescriptor(ExplicitDescriptor);
//     }
//     if (!Owner)
//     {
//         if (AutoInheritFlags & SEF_DEFAULT_OWNER_FROM_PARENT)
//         {
//             DPRINT("Use parent owner sid!\n");
//             if (!ARGUMENT_PRESENT(ParentDescriptor))
//             {
//                 SeUnlockSubjectContext(SubjectContext);
//                 return STATUS_INVALID_OWNER;
//             }
//
//             Owner = SepGetOwnerFromDescriptor(ParentDescriptor);
//             if (!Owner)
//             {
//                 SeUnlockSubjectContext(SubjectContext);
//                 return STATUS_INVALID_OWNER;
//             }
//         }
//         else
//         {
//             DPRINT("Use token owner sid!\n");
//             Owner = Token->UserAndGroups[Token->DefaultOwnerIndex].Sid;
//         }
//     }
//     OwnerLength = RtlLengthSid(Owner);
//     ASSERT(OwnerLength % sizeof(ULONG) == 0);
//
//     /* Inherit the Group SID */
//     if (ExplicitDescriptor != NULL)
//     {
//         Group = SepGetGroupFromDescriptor(ExplicitDescriptor);
//     }
//     if (!Group)
//     {
//         if (AutoInheritFlags & SEF_DEFAULT_GROUP_FROM_PARENT)
//         {
//             DPRINT("Use parent group sid!\n");
//             if (!ARGUMENT_PRESENT(ParentDescriptor))
//             {
//                 SeUnlockSubjectContext(SubjectContext);
//                 return STATUS_INVALID_PRIMARY_GROUP;
//             }
//
//             Group = SepGetGroupFromDescriptor(ParentDescriptor);
//             if (!Group)
//             {
//                 SeUnlockSubjectContext(SubjectContext);
//                 return STATUS_INVALID_PRIMARY_GROUP;
//             }
//         }
//         else
//         {
//             DPRINT("Use token group sid!\n");
//             Group = Token->PrimaryGroup;
//         }
//     }
//     if (!Group)
//     {
//         SeUnlockSubjectContext(SubjectContext);
//         return STATUS_INVALID_PRIMARY_GROUP;
//     }
//     GroupLength = RtlLengthSid(Group);
//     ASSERT(GroupLength % sizeof(ULONG) == 0);
//
//     /* Inherit the DACL */
//     DaclLength = 0;
//     ExplicitAcl = NULL;
//     ExplicitPresent = FALSE;
//     ExplicitDefaulted = FALSE;
//     if (ExplicitDescriptor != NULL &&
//         (ExplicitDescriptor->Control & SE_DACL_PRESENT))
//     {
//         ExplicitAcl = SepGetDaclFromDescriptor(ExplicitDescriptor);
//         ExplicitPresent = TRUE;
//         if (ExplicitDescriptor->Control & SE_DACL_DEFAULTED)
//             ExplicitDefaulted = TRUE;
//     }
//     ParentAcl = NULL;
//     if (ParentDescriptor != NULL &&
//         (ParentDescriptor->Control & SE_DACL_PRESENT))
//     {
//         ParentAcl = SepGetDaclFromDescriptor(ParentDescriptor);
//     }
//     Dacl = SepSelectAcl(ExplicitAcl,
//                         ExplicitPresent,
//                         ExplicitDefaulted,
//                         ParentAcl,
//                         Token->DefaultDacl,
//                         &DaclLength,
//                         Owner,
//                         Group,
//                         &DaclPresent,
//                         &DaclIsInherited,
//                         IsDirectoryObject,
//                         GenericMapping);
//     if (DaclPresent)
//         Control |= SE_DACL_PRESENT;
//     ASSERT(DaclLength % sizeof(ULONG) == 0);
//
//     /* Inherit the SACL */
//     SaclLength = 0;
//     ExplicitAcl = NULL;
//     ExplicitPresent = FALSE;
//     ExplicitDefaulted = FALSE;
//     if (ExplicitDescriptor != NULL &&
//         (ExplicitDescriptor->Control & SE_SACL_PRESENT))
//     {
//         ExplicitAcl = SepGetSaclFromDescriptor(ExplicitDescriptor);
//         ExplicitPresent = TRUE;
//         if (ExplicitDescriptor->Control & SE_SACL_DEFAULTED)
//             ExplicitDefaulted = TRUE;
//     }
//     ParentAcl = NULL;
//     if (ParentDescriptor != NULL &&
//         (ParentDescriptor->Control & SE_SACL_PRESENT))
//     {
//         ParentAcl = SepGetSaclFromDescriptor(ParentDescriptor);
//     }
//     Sacl = SepSelectAcl(ExplicitAcl,
//                         ExplicitPresent,
//                         ExplicitDefaulted,
//                         ParentAcl,
//                         NULL,
//                         &SaclLength,
//                         Owner,
//                         Group,
//                         &SaclPresent,
//                         &SaclIsInherited,
//                         IsDirectoryObject,
//                         GenericMapping);
//     if (SaclPresent)
//         Control |= SE_SACL_PRESENT;
//     ASSERT(SaclLength % sizeof(ULONG) == 0);
//
//     /* Allocate and initialize the new security descriptor */
//     Length = sizeof(SECURITY_DESCRIPTOR_RELATIVE) +
//         OwnerLength + GroupLength + DaclLength + SaclLength;
//
//     DPRINT("L: sizeof(SECURITY_DESCRIPTOR) %u OwnerLength %lu GroupLength %lu DaclLength %lu SaclLength %lu\n",
//            sizeof(SECURITY_DESCRIPTOR),
//            OwnerLength,
//            GroupLength,
//            DaclLength,
//            SaclLength);
//
//     Descriptor = ExAllocatePoolWithTag(PagedPool, Length, TAG_SD);
//     if (Descriptor == NULL)
//     {
//         DPRINT1("ExAlloctePool() failed\n");
//         SeUnlockSubjectContext(SubjectContext);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(Descriptor, Length);
//     RtlCreateSecurityDescriptor(Descriptor, SECURITY_DESCRIPTOR_REVISION);
//
//     Descriptor->Control = Control | SE_SELF_RELATIVE;
//
//     Current = sizeof(SECURITY_DESCRIPTOR_RELATIVE);
//
//     if (SaclLength != 0)
//     {
//         Status = SepPropagateAcl((PACL)((PUCHAR)Descriptor + Current),
//                                  &SaclLength,
//                                  Sacl,
//                                  Owner,
//                                  Group,
//                                  SaclIsInherited,
//                                  IsDirectoryObject,
//                                  GenericMapping);
//         ASSERT(Status == STATUS_SUCCESS);
//         Descriptor->Sacl = Current;
//         Current += SaclLength;
//     }
//
//     if (DaclLength != 0)
//     {
//         Status = SepPropagateAcl((PACL)((PUCHAR)Descriptor + Current),
//                                  &DaclLength,
//                                  Dacl,
//                                  Owner,
//                                  Group,
//                                  DaclIsInherited,
//                                  IsDirectoryObject,
//                                  GenericMapping);
//         ASSERT(Status == STATUS_SUCCESS);
//         Descriptor->Dacl = Current;
//         Current += DaclLength;
//     }
//
//     if (OwnerLength != 0)
//     {
//         RtlCopyMemory((PUCHAR)Descriptor + Current, Owner, OwnerLength);
//         Descriptor->Owner = Current;
//         Current += OwnerLength;
//         DPRINT("Owner of %p at %x\n", Descriptor, Descriptor->Owner);
//     }
//     else
//     {
//         DPRINT("Owner of %p is zero length\n", Descriptor);
//     }
//
//     if (GroupLength != 0)
//     {
//         RtlCopyMemory((PUCHAR)Descriptor + Current, Group, GroupLength);
//         Descriptor->Group = Current;
//     }
//
//     /* Unlock subject context */
//     SeUnlockSubjectContext(SubjectContext);
//
//     *NewDescriptor = Descriptor;
//
//     DPRINT("Descriptor %p\n", Descriptor);
//     ASSERT(RtlLengthSecurityDescriptor(Descriptor));
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Assigns a security descriptor for a new object.
//  *
//  * @param[in] ParentDescriptor
//  * A security descriptor of the parent object that is being
//  * created.
//  *
//  * @param[in] ExplicitDescriptor
//  * An explicit security descriptor that is applied to a new
//  * object.
//  *
//  * @param[out] NewDescriptor
//  * The new allocated security descriptor.
//  *
//  * @param[in] IsDirectoryObject
//  * Set this to TRUE if the newly created object is a directory
//  * object, otherwise set this to FALSE.
//  *
//  * @param[in] SubjectContext
//  * Security subject context of the new object.
//  *
//  * @param[in] GenericMapping
//  * Generic mapping of access mask rights.
//  *
//  * @param[in] PoolType
//  * This parameter is unused.
//  *
//  * @return
//  * See SeAssignSecurityEx.
//  */
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// NTAPI
// SeAssignSecurity(
//     _In_opt_ PSECURITY_DESCRIPTOR ParentDescriptor,
//     _In_opt_ PSECURITY_DESCRIPTOR ExplicitDescriptor,
//     _Out_ PSECURITY_DESCRIPTOR *NewDescriptor,
//     _In_ BOOLEAN IsDirectoryObject,
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext,
//     _In_ PGENERIC_MAPPING GenericMapping,
//     _In_ POOL_TYPE PoolType)
// {
//     PAGED_CODE();
//
//     return SeAssignSecurityEx(ParentDescriptor,
//                               ExplicitDescriptor,
//                               NewDescriptor,
//                               NULL,
//                               IsDirectoryObject,
//                               0,
//                               SubjectContext,
//                               GenericMapping,
//                               PoolType);
// }
//
// /**
//  * @brief
//  * Computes the quota size of a security descriptor.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @param[out] QuotaInfoSize
//  * The returned quota size of the given security descriptor to
//  * the caller. The function may return 0 to this parameter if
//  * the descriptor doesn't have a group or a discretionary
//  * access control list (DACL) even.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the quota size of a security
//  * descriptor has been computed successfully. STATUS_UNKNOWN_REVISION
//  * is returned if the security descriptor has an invalid revision.
//  */
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// NTAPI
// SeComputeQuotaInformationSize(
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _Out_ PULONG QuotaInfoSize)
// {
//     PSID Group;
//     PACL Dacl;
//
//     PAGED_CODE();
//
//     *QuotaInfoSize = 0;
//
//     /* Validate security descriptor revision */
//     if (((PISECURITY_DESCRIPTOR)SecurityDescriptor)->Revision != SECURITY_DESCRIPTOR_REVISION1 as u8)
//     {
//         return STATUS_UNKNOWN_REVISION;
//     }
//
//     /* Get group and DACL, if any */
//     Group = SepGetGroupFromDescriptor(SecurityDescriptor);
//     Dacl = SepGetDaclFromDescriptor(SecurityDescriptor);
//
//     /* Return SID length if any */
//     if (Group != NULL)
//     {
//         *QuotaInfoSize = ALIGN_UP_BY(RtlLengthSid(Group), sizeof(ULONG));
//     }
//
//     /* Return DACL if any */
//     if (Dacl != NULL)
//     {
//         *QuotaInfoSize += ALIGN_UP_BY(Dacl->AclSize, sizeof(ULONG));
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
