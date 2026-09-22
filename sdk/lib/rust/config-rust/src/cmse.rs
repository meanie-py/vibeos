use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::inline::se::SeLengthSid;
use ntkrnl_sys::ddk::{
    ACE, ACL, CM_KEY_SECURITY_CACHE, CmpAcquireKcbLockExclusive, CmpLockRegistry,
    CmpReleaseKcbLock, CmpUnlockRegistry, PACE_HEADER, PACL, PCM_KEY_BODY,
    HCELL_INDEX, PCM_KEY_CONTROL_BLOCK, PCM_KEY_NODE, PCM_KEY_SECURITY,
    PCM_KEY_SECURITY_CACHE, PGENERIC_MAPPING, PHHIVE,
    PISECURITY_DESCRIPTOR, PISECURITY_DESCRIPTOR_RELATIVE, PSECURITY_DESCRIPTOR,
    PSECURITY_INFORMATION, PSID, RtlAbsoluteToSelfRelativeSD, RtlAddAccessAllowedAce,
    RtlCreateAcl, RtlCreateSecurityDescriptor,
    RtlCreateSecurityDescriptorRelative, RtlGetAce, RtlInitializeSid, RtlLengthRequiredSid,
    RtlLengthSecurityDescriptor, RtlLengthSid, RtlSetDaclSecurityDescriptor, RtlSubAuthoritySid,
    SeQuerySecurityDescriptorInfo, SeSetSecurityDescriptorInfo, _SECURITY_DESCRIPTOR,
    _SECURITY_DESCRIPTOR_RELATIVE, SECURITY_DESCRIPTOR_CONTROL, SECURITY_INFORMATION,
    SECURITY_OPERATION_CODE, SID_IDENTIFIER_AUTHORITY,
    _SECURITY_OPERATION_CODE_AssignSecurityDescriptor,
    _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor,
    _SECURITY_OPERATION_CODE_QuerySecurityDescriptor,
    _SECURITY_OPERATION_CODE_SetSecurityDescriptor,
};

use ntkrnl_sys::opaque_ptr;
use ntkrnl_sys::ddk::KPROCESSOR_MODE;
// SECURITY_SYSTEM bugcheck code (bugcodes.mc: 0x29)
const SECURITY_SYSTEM: u32 = 0x29;

extern "C" {
    fn CmpAcquireKcbLockShared(Kcb: PCM_KEY_CONTROL_BLOCK);
}

/// Pool tag ("CmSc") for a KCB's cached security descriptor.
const TAG_CM_SECURITY_CACHE: u32 = 0x6353_6D43;

/// Pool tag ("dSeS", ntoskrnl/include/internal/tag.h) that the Se layer allocates
/// replacement descriptors with. Freeing one under any other tag is a bugcheck.
const TAG_SD: u32 = 0x6453_6553;

/// KEY_ALL_ACCESS. Declared locally because the bindgen islands carry it only for
/// i386/amd64 (`hal_bindgen_*`) and not arm64, so importing it would break the arm64
/// build. Pinned to the island below wherever the island has it, so a hand-typed value
/// can never drift -- this file already shipped one wrong hand-typed SID RID.
const KEY_ALL_ACCESS: ULONG = 0x000F_003F;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const _: () = assert!(KEY_ALL_ACCESS == ntkrnl_sys::hal_ddk::KEY_ALL_ACCESS as ULONG);

/// A cell index that names no cell. Cached descriptors do not own the hive
/// `sk` cell identity in the private cache; hive persistence owns that separately.
const HCELL_NIL: u32 = 0xFFFF_FFFF;

/// "sk" / "nk" — the signatures a CM_KEY_SECURITY and a CM_KEY_NODE cell carry
/// (cmlib's cmdata.h). Declared here rather than imported because cmlib-rust keeps
/// its copies `pub(crate)`.
const CM_KEY_SECURITY_SIGNATURE: u16 = 0x6B73;
const CM_KEY_NODE_SIGNATURE: u16 = 0x6B6E;

extern "C" {
    fn HvGetCell(Hive: PHHIVE, Cell: HCELL_INDEX) -> *mut core::ffi::c_void;
    fn HvReleaseCell(Hive: PHHIVE, Cell: HCELL_INDEX);
}

/// CM_KEY_SECURITY_CACHE ends in a variable-length SECURITY_DESCRIPTOR_RELATIVE, so a
/// block is allocated as header + descriptor bytes and the descriptor starts here.
/// Using the struct's own size instead would truncate every descriptor to the size of
/// an empty one.
const CM_SECURITY_CACHE_HEADER: usize = core::mem::offset_of!(CM_KEY_SECURITY_CACHE, Descriptor);

/// The descriptor cached on `Kcb`, or NULL if the key has none.
unsafe fn CmpGetCachedSecurity(Kcb: PCM_KEY_CONTROL_BLOCK) -> PSECURITY_DESCRIPTOR {
    if Kcb.is_null() {
        return core::ptr::null_mut();
    }
    let Cached = (*Kcb).CachedSecurity;
    if Cached.is_null() {
        return core::ptr::null_mut();
    }
    core::ptr::addr_of_mut!((*Cached).Descriptor) as PSECURITY_DESCRIPTOR
}

/// Release the descriptor cached on a KCB, if any.
///
/// Exported: the KCB teardown path in cmkcbncb owns the last reference to the block.
#[no_mangle]
pub unsafe extern "system" fn CmpFreeKcbCachedSecurity(Kcb: PCM_KEY_CONTROL_BLOCK) {
    if Kcb.is_null() {
        return;
    }
    let Cached = (*Kcb).CachedSecurity;
    if !Cached.is_null() {
        (*Kcb).CachedSecurity = core::ptr::null_mut();
        ExFreePoolWithTag(Cached as *mut core::ffi::c_void, TAG_CM_SECURITY_CACHE);
    }
}

/// Attach a private self-relative copy of `SecurityDescriptor` to `Kcb`, replacing
/// whatever was cached there before.
///
/// The descriptor is copied, never aliased: the object manager frees its own copy as
/// soon as the security method returns, and an ABSOLUTE descriptor additionally holds
/// pointers to SIDs and ACLs in separate allocations the caller owns. Converting to
/// self-relative flattens all of that into one block we control.
unsafe fn CmpStoreCachedSecurity(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    if Kcb.is_null() || SecurityDescriptor.is_null() {
        return STATUS_SUCCESS;
    }

    // Reports the size the SELF-RELATIVE form needs, for either input form. For an
    // absolute descriptor it counts the (larger) absolute header, so this can
    // over-estimate; the conversion writes the length actually used back.
    let mut Length: ULONG = RtlLengthSecurityDescriptor(SecurityDescriptor);
    if Length == 0 {
        return STATUS_INVALID_SECURITY_DESCR;
    }

    let Block = ExAllocatePoolWithTag(
        PagedPool,
        CM_SECURITY_CACHE_HEADER + Length as usize,
        TAG_CM_SECURITY_CACHE,
    ) as PCM_KEY_SECURITY_CACHE;
    if Block.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let Target = core::ptr::addr_of_mut!((*Block).Descriptor) as PSECURITY_DESCRIPTOR;
    let Control = (*(SecurityDescriptor as PISECURITY_DESCRIPTOR)).Control;
    let Status = if (Control & SE_SELF_RELATIVE) != 0 {
        RtlCopyMemory(
            Target as *mut core::ffi::c_void,
            SecurityDescriptor as *const core::ffi::c_void,
            Length as usize,
        );
        STATUS_SUCCESS
    } else {
        RtlAbsoluteToSelfRelativeSD(SecurityDescriptor, Target, &mut Length)
    };

    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(Block as *mut core::ffi::c_void, TAG_CM_SECURITY_CACHE);
        return Status;
    }

    (*Block).Cell = HCELL_NIL;
    (*Block).ConvKey = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey;
    (*Block).DescriptorLength = Length;
    InitializeListHead(&mut (*Block).List);

    // Commit hive storage before replacing the cache. Cache eviction must never
    // discard the only descriptor, and allocation failure keeps the old state.
    let Status = crate::cmsecurity_persist::persist(Kcb, Target.cast::<u8>(), Length);
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(Block as *mut core::ffi::c_void, TAG_CM_SECURITY_CACHE);
        return Status;
    }

    CmpFreeKcbCachedSecurity(Kcb);
    (*Kcb).CachedSecurity = Block;
    STATUS_SUCCESS
}

/// Cache `Length` bytes of an already-self-relative descriptor verbatim.
///
/// The caller supplies the length from a trusted source instead of having it measured,
/// so nothing here parses the descriptor. That is the point: this serves hive bytes,
/// which may be corrupt, and a measuring pass would have to walk the SIDs and ACLs it
/// is not yet entitled to trust.
unsafe fn CmpStoreCachedSecurityOfLength(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    Length: ULONG,
) -> NTSTATUS {
    if Kcb.is_null() || SecurityDescriptor.is_null() {
        return STATUS_INVALID_SECURITY_DESCR;
    }
    // Too short to even hold the fixed header, so the offsets inside it cannot be read.
    if (Length as usize) < core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() {
        return STATUS_INVALID_SECURITY_DESCR;
    }

    let Block = ExAllocatePoolWithTag(
        PagedPool,
        CM_SECURITY_CACHE_HEADER + Length as usize,
        TAG_CM_SECURITY_CACHE,
    ) as PCM_KEY_SECURITY_CACHE;
    if Block.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*Block).Descriptor) as *mut core::ffi::c_void,
        SecurityDescriptor as *const core::ffi::c_void,
        Length as usize,
    );

    (*Block).Cell = HCELL_NIL;
    (*Block).ConvKey = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey;
    (*Block).DescriptorLength = Length;
    InitializeListHead(&mut (*Block).List);

    CmpFreeKcbCachedSecurity(Kcb);
    (*Kcb).CachedSecurity = Block;
    STATUS_SUCCESS
}

/// Populate the KCB's cache from the key's on-disk `sk` cell, and return the cached
/// descriptor (NULL if the key has none).
///
/// Every key in a hive FILE carries its real ACL in a CM_KEY_SECURITY cell named by
/// `CM_KEY_NODE.Security`; mkhive writes one on each hive root and points every child at
/// it (sdk/tools/mkhive/cmi.c). Until this existed nothing in the kernel ever READ that
/// field, so those keys fell through to the grant-World-everything default below —
/// SAM, SECURITY, SOFTWARE and SYSTEM included.
///
/// The descriptor is COPIED into the KCB cache rather than returned as a pointer into
/// the hive: a cell pointer is only valid between HvGetCell and HvReleaseCell, and the
/// caller (SeQuerySecurityDescriptorInfo, and the Set path underneath it) outlives that
/// window. Copying also means the rest of this file needs no cell-lifetime awareness —
/// there is exactly one representation of "this key's descriptor", the cached one.
pub(crate) unsafe fn CmpCacheSecurityFromHive(Kcb: PCM_KEY_CONTROL_BLOCK) -> PSECURITY_DESCRIPTOR {
    if Kcb.is_null() {
        return core::ptr::null_mut();
    }
    let Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
    let Cell = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell;
    if Hive.is_null() || Cell == HCELL_NIL {
        return core::ptr::null_mut();
    }

    // A cell index is just an offset into the hive, so EVERY cell this walk touches has
    // its signature checked before its contents are believed. Skipping the CM_KEY_NODE
    // check cost a boot: a KCB whose KeyCell did not name a key node yielded a garbage
    // Security index, and the descriptor built from it faulted inside RtlLengthSid
    // (BUGCHECK 0x1E at ntoskrnl+0x6E7C5).
    let Node = HvGetCell(Hive, Cell) as PCM_KEY_NODE;
    if Node.is_null() {
        return core::ptr::null_mut();
    }
    let SecurityCell = if (*Node).Signature == CM_KEY_NODE_SIGNATURE {
        (*Node).Security
    } else {
        HCELL_NIL
    };
    HvReleaseCell(Hive, Cell);
    if SecurityCell == HCELL_NIL {
        return core::ptr::null_mut();
    }

    let Sk = HvGetCell(Hive, SecurityCell) as PCM_KEY_SECURITY;
    if Sk.is_null() {
        return core::ptr::null_mut();
    }
    // `&&` short-circuits, so a bad signature never reaches the copy. The cell's own
    // DescriptorLength is the authority on size -- do NOT measure this descriptor with
    // RtlLengthSecurityDescriptor, which WALKS it (owner SID, group SID, both ACLs) and
    // so faults on exactly the malformed input this guard exists to reject. Length first,
    // parse never: the bytes are copied verbatim and only the Se layer interprets them.
    let Cached = (*Sk).Signature == CM_KEY_SECURITY_SIGNATURE
        && NT_SUCCESS(CmpStoreCachedSecurityOfLength(
            Kcb,
            core::ptr::addr_of_mut!((*Sk).Descriptor) as PSECURITY_DESCRIPTOR,
            (*Sk).DescriptorLength,
        ));
    HvReleaseCell(Hive, SecurityCell);

    if !Cached {
        return core::ptr::null_mut();
    }
    CmpGetCachedSecurity(Kcb)
}

#[no_mangle]
pub unsafe extern "system" fn CmpHiveRootSecurityDescriptor() -> PSECURITY_DESCRIPTOR {
    let mut Status: NTSTATUS;
    let SecurityDescriptor: PSECURITY_DESCRIPTOR;
    let mut Acl: PACL;
    let AclCopy: PACL;
    let mut Sid: [PSID; 4] = [core::ptr::null_mut(); 4];
    let WorldAuthority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 1],
    };
    let NtAuthority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    let mut AceLength: ULONG;
    let mut AclLength: ULONG;
    let mut SidLength: ULONG;
    let mut AceHeader: PACE_HEADER = core::ptr::null_mut();
    let mut i: ULONG;

    SidLength = RtlLengthRequiredSid(1);
    Sid[0] = ExAllocatePoolWithTag(PagedPool, SidLength as usize, 0x44534D43u32);
    Sid[1] = ExAllocatePoolWithTag(PagedPool, SidLength as usize, 0x44534D43u32);
    Sid[2] = ExAllocatePoolWithTag(PagedPool, SidLength as usize, 0x44534D43u32);
    SidLength = RtlLengthRequiredSid(2);
    Sid[3] = ExAllocatePoolWithTag(PagedPool, SidLength as usize, 0x44534D43u32);

    if Sid[0].is_null() || Sid[1].is_null() || Sid[2].is_null() || Sid[3].is_null() {
        KeBugCheckEx(ntkrnl_sys::bugcodes::REGISTRY_ERROR, 11, 1, 0, 0);
    }

    Status = RtlInitializeSid(Sid[0], &WorldAuthority as *const _ as *mut _, 1);
    Status |= RtlInitializeSid(Sid[1], &NtAuthority as *const _ as *mut _, 1);
    Status |= RtlInitializeSid(Sid[2], &NtAuthority as *const _ as *mut _, 1);
    Status |= RtlInitializeSid(Sid[3], &NtAuthority as *const _ as *mut _, 2);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(ntkrnl_sys::bugcodes::REGISTRY_ERROR, 11, 2, 0, 0);
    }

    /* Named, not numeric: the translation had lowered SECURITY_RESTRICTED_CODE_RID to a
     * literal 33 (that is SECURITY_WRITE_RESTRICTED_CODE_RID) instead of 12, so the hive
     * root DACL granted KEY_READ to S-1-5-33 rather than S-1-5-12. */
    *RtlSubAuthoritySid(Sid[0], 0) = SECURITY_WORLD_RID;
    *RtlSubAuthoritySid(Sid[1], 0) = SECURITY_RESTRICTED_CODE_RID;
    *RtlSubAuthoritySid(Sid[2], 0) = SECURITY_LOCAL_SYSTEM_RID;
    *RtlSubAuthoritySid(Sid[3], 0) = SECURITY_BUILTIN_DOMAIN_RID;
    *RtlSubAuthoritySid(Sid[3], 1) = DOMAIN_ALIAS_RID_ADMINS;

    AclLength = core::mem::size_of::<ACL>() as ULONG;
    i = 0;
    while i < 4 {
        AceLength = 8;
        AceLength += SeLengthSid(Sid[i as usize]);
        AclLength += AceLength;
        i += 1;
    }

    Acl = ExAllocatePoolWithTag(PagedPool, AclLength as usize, 0x44534D43u32) as PACL;
    if Acl.is_null() {
        KeBugCheckEx(ntkrnl_sys::bugcodes::REGISTRY_ERROR, 11, 3, 0, 0);
    }

    Status = RtlCreateAcl(Acl, AclLength, ACL_REVISION);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(ntkrnl_sys::bugcodes::REGISTRY_ERROR, 11, 4, Status as ULONG_PTR, 0);
    }

    Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION, KEY_ALL_ACCESS, Sid[2]);
    Status |= RtlAddAccessAllowedAce(Acl, ACL_REVISION, KEY_ALL_ACCESS, Sid[3]);
    Status |= RtlAddAccessAllowedAce(Acl, ACL_REVISION, KEY_READ, Sid[0]);
    Status |= RtlAddAccessAllowedAce(Acl, ACL_REVISION, KEY_READ, Sid[1]);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(ntkrnl_sys::bugcodes::REGISTRY_ERROR, 11, 5, Status as ULONG_PTR, 0);
    }

    Status = RtlGetAce(Acl, 0, &mut AceHeader as *mut _ as *mut *mut core::ffi::c_void);
    (*AceHeader).AceFlags |= CONTAINER_INHERIT_ACE;
    Status = RtlGetAce(Acl, 1, &mut AceHeader as *mut _ as *mut *mut core::ffi::c_void);
    (*AceHeader).AceFlags |= CONTAINER_INHERIT_ACE;
    Status = RtlGetAce(Acl, 2, &mut AceHeader as *mut _ as *mut *mut core::ffi::c_void);
    (*AceHeader).AceFlags |= CONTAINER_INHERIT_ACE;
    Status = RtlGetAce(Acl, 3, &mut AceHeader as *mut _ as *mut *mut core::ffi::c_void);
    (*AceHeader).AceFlags |= CONTAINER_INHERIT_ACE;

    SecurityDescriptor = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_SECURITY_DESCRIPTOR>() + AclLength as usize,
        0x44534D43u32,
    );
    if SecurityDescriptor.is_null() {
        KeBugCheckEx(ntkrnl_sys::bugcodes::REGISTRY_ERROR, 11, 6, 0, 0);
    }

    AclCopy = (SecurityDescriptor as PISECURITY_DESCRIPTOR).offset(1) as PACL;
    core::ptr::copy_nonoverlapping(Acl as *const u8, AclCopy as *mut u8, AclLength as usize);

    Status = RtlCreateSecurityDescriptor(SecurityDescriptor, 1);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(ntkrnl_sys::bugcodes::REGISTRY_ERROR, 11, 7, Status as ULONG_PTR, 0);
    }

    Status = RtlSetDaclSecurityDescriptor(SecurityDescriptor, 1u8, AclCopy, 0u8);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(ntkrnl_sys::bugcodes::REGISTRY_ERROR, 11, 8, Status as ULONG_PTR, 0);
    }

    i = 0;
    while i < 4 {
        ExFreePoolWithTag(Sid[i as usize] as *mut core::ffi::c_void, 0x44534D43u32);
        i += 1;
    }
    ExFreePoolWithTag(Acl as *mut core::ffi::c_void, 0x44534D43u32);

    SecurityDescriptor
}

#[no_mangle]
pub unsafe extern "system" fn CmpQuerySecurityDescriptor(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    BufferLength: *mut u32,
) -> i32 {
    let Kcb = opaque_ptr(Kcb);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let BufferLength = opaque_ptr(BufferLength);

    if SecurityInformation == 0 {
        return STATUS_ACCESS_DENIED;
    }

    // Assigned descriptor first, then the key's own `sk` cell. A descriptor assigned in
    // this boot is the newer truth: CmpSetSecurityDescriptor caches without yet writing
    // the cell back, so preferring the hive here would serve a stale ACL.
    let mut ObjectSd = CmpGetCachedSecurity(Kcb);
    if ObjectSd.is_null() {
        ObjectSd = CmpCacheSecurityFromHive(Kcb);
    }
    if !ObjectSd.is_null() {
        let mut SecInfo: SECURITY_INFORMATION = SecurityInformation;
        return SeQuerySecurityDescriptorInfo(
            &mut SecInfo,
            SecurityDescriptor,
            BufferLength,
            &mut ObjectSd,
        );
    }

    CmpQueryDefaultSecurityDescriptor(SecurityInformation, SecurityDescriptor, BufferLength)
}

/// Fallback for a key that carries no descriptor at all: grant World everything.
///
/// This is upstream ReactOS's original whole-CM behaviour, now narrowed twice — first to
/// keys we have not assigned, then to keys whose hive holds no `sk` cell for them either.
/// What still lands here is a VOLATILE key created without an explicit descriptor, since
/// a volatile hive has no cells on disk to inherit from.
///
/// It remains wrong (world-writable is not a safe default) and is only tolerable because
/// the keys that reach it are the ones with no ACL to honour. Do not mistake it for a
/// designed default.
unsafe fn CmpQueryDefaultSecurityDescriptor(
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    BufferLength: *mut u32,
) -> i32 {
    let mut RelSd: PISECURITY_DESCRIPTOR_RELATIVE;
    let mut SidSize: u32;
    let mut AclSize: u32 = 0;
    let mut SdSize: u32;
    let mut Status: i32;
    let mut Control: SECURITY_DESCRIPTOR_CONTROL = 0;
    let mut Owner: u32 = 0;
    let mut Group: u32 = 0;
    let mut Dacl: u32 = 0;

    if SecurityInformation == 0 {
        return STATUS_ACCESS_DENIED;
    }

    SidSize = RtlLengthSid(SeWorldSid);
    RelSd = SecurityDescriptor as PISECURITY_DESCRIPTOR_RELATIVE;
    SdSize = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as u32;

    if (SecurityInformation & OWNER_SECURITY_INFORMATION) != 0 {
        Owner = SdSize;
        SdSize += SidSize;
    }

    if (SecurityInformation & GROUP_SECURITY_INFORMATION) != 0 {
        Group = SdSize;
        SdSize += SidSize;
    }

    if (SecurityInformation & DACL_SECURITY_INFORMATION) != 0 {
        Control |= SE_DACL_PRESENT;
        Dacl = SdSize;
        AclSize = core::mem::size_of::<ACL>() as u32 + core::mem::size_of::<ACE>() as u32 + SidSize;
        SdSize += AclSize;
    }

    if (SecurityInformation & SACL_SECURITY_INFORMATION) != 0 {
        Control |= SE_SACL_PRESENT;
    }

    if *BufferLength < SdSize {
        *BufferLength = SdSize;
        return STATUS_BUFFER_TOO_SMALL;
    }

    *BufferLength = SdSize;

    Status = RtlCreateSecurityDescriptorRelative(
        RelSd,
        SECURITY_DESCRIPTOR_REVISION,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*RelSd).Control |= Control;
    (*RelSd).Owner = Owner;
    (*RelSd).Group = Group;
    (*RelSd).Dacl = Dacl;

    if Owner != 0 {
        RtlCopyMemory(
            (RelSd as *mut u8).add(Owner as usize) as *mut core::ffi::c_void,
            SeWorldSid as *const core::ffi::c_void,
            SidSize as usize,
        );
    }

    if Group != 0 {
        RtlCopyMemory(
            (RelSd as *mut u8).add(Group as usize) as *mut core::ffi::c_void,
            SeWorldSid as *const core::ffi::c_void,
            SidSize as usize,
        );
    }

    if Dacl != 0 {
        Status = RtlCreateAcl(
            (RelSd as *mut u8).add(Dacl as usize) as PACL,
            AclSize,
            ACL_REVISION,
        );
        if NT_SUCCESS(Status) {
            Status = RtlAddAccessAllowedAce(
                (RelSd as *mut u8).add(Dacl as usize) as PACL,
                ACL_REVISION,
                GENERIC_ALL,
                SeWorldSid,
            );
        }
    }

    Status
}

/// Replace the key's descriptor with the caller's, component by component.
///
/// Was a bare `return STATUS_SUCCESS` that discarded the caller's descriptor. Now that
/// Query serves real data, leaving it a no-op would report the OLD descriptor back after
/// a successful set.
///
/// SeSetSecurityDescriptorInfoEx allocates a fresh TAG_SD block and overwrites the
/// pointer WITHOUT freeing what was there (ntoskrnl/se/sd.c:962,1005), so we keep
/// ownership of the block we passed in and must free the one it hands back.
#[no_mangle]
pub unsafe extern "system" fn CmpSetSecurityDescriptor(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    SecurityInformation: PSECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
) -> i32 {
    let Kcb = opaque_ptr(Kcb);
    let SecurityInformation = opaque_ptr(SecurityInformation);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let GenericMapping = opaque_ptr(GenericMapping);

    // Same order as the query path, and for the same reason: a partial set (say, DACL
    // only) merges INTO the current descriptor, so the key's real on-disk ACL has to be
    // the base. Editing the World fallback instead would silently discard the owner,
    // group and every ACE the hive shipped.
    let mut Original = CmpGetCachedSecurity(Kcb);
    if Original.is_null() {
        Original = CmpCacheSecurityFromHive(Kcb);
    }
    let mut ObjectSd = Original;
    let Status = SeSetSecurityDescriptorInfo(
        core::ptr::null_mut(),
        SecurityInformation,
        SecurityDescriptor,
        &mut ObjectSd,
        PoolType,
        GenericMapping,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // Nothing was substituted, so the cached block still holds the live descriptor.
    // Guards against ever freeing an interior pointer into our own allocation.
    if ObjectSd == Original {
        return STATUS_SUCCESS;
    }

    let Status = CmpStoreCachedSecurity(Kcb, ObjectSd);
    ExFreePoolWithTag(ObjectSd as *mut core::ffi::c_void, TAG_SD);
    Status
}

/// Take ownership of the descriptor the object manager assigned to a newly created key.
///
/// Until this stored the descriptor, every key the kernel created -- including the
/// registry roots, which cmsysini builds a correct 4-ACE DACL for via
/// CmpHiveRootSecurityDescriptor() -- had that descriptor dropped on the floor here, and
/// every query then reported the World fallback below.
#[no_mangle]
pub unsafe extern "C" fn CmpAssignSecurityDescriptor(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let Kcb = opaque_ptr(Kcb);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    CmpStoreCachedSecurity(Kcb, SecurityDescriptor)
}

#[no_mangle]
pub unsafe extern "system" fn CmpSecurityMethod(
    ObjectBody: *mut core::ffi::c_void,
    OperationCode: SECURITY_OPERATION_CODE,
    SecurityInformation: PSECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    BufferLength: *mut u32,
    OldSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
    _AccessMode: KPROCESSOR_MODE,
) -> i32 {
    let ObjectBody = opaque_ptr(ObjectBody);
    let SecurityInformation = opaque_ptr(SecurityInformation);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let BufferLength = opaque_ptr(BufferLength);
    let OldSecurityDescriptor = opaque_ptr(OldSecurityDescriptor);
    let GenericMapping = opaque_ptr(GenericMapping);
    let Kcb: PCM_KEY_CONTROL_BLOCK;
    let mut Status: i32 = STATUS_SUCCESS;

    Kcb = (*(ObjectBody as PCM_KEY_BODY)).KeyControlBlock;

    CmpLockRegistry();

    if OperationCode == _SECURITY_OPERATION_CODE_QuerySecurityDescriptor {
        if (*(ObjectBody as PCM_KEY_BODY)).KcbLocked == 0 {
            CmpAcquireKcbLockShared(Kcb);
        }
    } else {
        CmpAcquireKcbLockExclusive(Kcb);
    }

    if (*Kcb).__bindgen_anon_1.Delete() != 0 {
        CmpReleaseKcbLock(Kcb);
        CmpUnlockRegistry();
        return STATUS_KEY_DELETED;
    }

    match OperationCode {
        _SECURITY_OPERATION_CODE_SetSecurityDescriptor => {
            Status = CmpSetSecurityDescriptor(
                Kcb,
                SecurityInformation,
                SecurityDescriptor,
                PoolType,
                GenericMapping,
            );
        }
        _SECURITY_OPERATION_CODE_QuerySecurityDescriptor => {
            Status = CmpQuerySecurityDescriptor(
                Kcb,
                *SecurityInformation,
                SecurityDescriptor,
                BufferLength,
            );
        }
        _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor => {
        }
        _SECURITY_OPERATION_CODE_AssignSecurityDescriptor => {
            Status = CmpAssignSecurityDescriptor(Kcb, SecurityDescriptor);
        }
        _ => {
            KeBugCheckEx(SECURITY_SYSTEM, 0, STATUS_INVALID_PARAMETER as ULONG_PTR, 0, 0);
        }
    }

    if (*(ObjectBody as PCM_KEY_BODY)).KcbLocked == 0 {
        CmpReleaseKcbLock(Kcb);
    }

    CmpUnlockRegistry();

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmse.c
//  * PURPOSE:         Configuration Manager - Security Subsystem Interface
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// /* GLOBALS *******************************************************************/
//
// /* FUNCTIONS *****************************************************************/
//
// PSECURITY_DESCRIPTOR
// NTAPI
// CmpHiveRootSecurityDescriptor(VOID)
// {
//     NTSTATUS Status;
//     PSECURITY_DESCRIPTOR SecurityDescriptor;
//     PACL Acl, AclCopy;
//     PSID Sid[4];
//     SID_IDENTIFIER_AUTHORITY WorldAuthority = {SECURITY_WORLD_SID_AUTHORITY};
//     SID_IDENTIFIER_AUTHORITY NtAuthority = {SECURITY_NT_AUTHORITY};
//     ULONG AceLength, AclLength, SidLength;
//     PACE_HEADER AceHeader;
//     ULONG i;
//     PAGED_CODE();
//
//     /* Phase 1: Allocate SIDs */
//     SidLength = RtlLengthRequiredSid(1);
//     Sid[0] = ExAllocatePoolWithTag(PagedPool, SidLength, TAG_CMSD);
//     Sid[1] = ExAllocatePoolWithTag(PagedPool, SidLength, TAG_CMSD);
//     Sid[2] = ExAllocatePoolWithTag(PagedPool, SidLength, TAG_CMSD);
//     SidLength = RtlLengthRequiredSid(2);
//     Sid[3] = ExAllocatePoolWithTag(PagedPool, SidLength, TAG_CMSD);
//
//     /* Make sure all SIDs were allocated */
//     if (!Sid[0] || !Sid[1] || !Sid[2] || !Sid[3])
//     {
//         /* Bugcheck */
//         KeBugCheckEx(REGISTRY_ERROR, 11, 1, 0, 0);
//     }
//
//     /* Phase 2: Initialize all SIDs */
//     Status = RtlInitializeSid(Sid[0], &WorldAuthority, 1);
//     Status |= RtlInitializeSid(Sid[1], &NtAuthority, 1);
//     Status |= RtlInitializeSid(Sid[2], &NtAuthority, 1);
//     Status |= RtlInitializeSid(Sid[3], &NtAuthority, 2);
//     if (!NT_SUCCESS(Status)) KeBugCheckEx(REGISTRY_ERROR, 11, 2, 0, 0);
//
//     /* Phase 2: Setup SID Sub Authorities */
//     *RtlSubAuthoritySid(Sid[0], 0) = SECURITY_WORLD_RID;
//     *RtlSubAuthoritySid(Sid[1], 0) = SECURITY_RESTRICTED_CODE_RID;
//     *RtlSubAuthoritySid(Sid[2], 0) = SECURITY_LOCAL_SYSTEM_RID;
//     *RtlSubAuthoritySid(Sid[3], 0) = SECURITY_BUILTIN_DOMAIN_RID;
//     *RtlSubAuthoritySid(Sid[3], 1) = DOMAIN_ALIAS_RID_ADMINS;
//
//     /* Make sure all SIDs are valid */
//     ASSERT(RtlValidSid(Sid[0]));
//     ASSERT(RtlValidSid(Sid[1]));
//     ASSERT(RtlValidSid(Sid[2]));
//     ASSERT(RtlValidSid(Sid[3]));
//
//     /* Phase 3: Calculate ACL Length */
//     AclLength = sizeof(ACL);
//     for (i = 0; i < 4; i++)
//     {
//         /* This is what MSDN says to do */
//         AceLength = FIELD_OFFSET(ACCESS_ALLOWED_ACE, SidStart);
//         AceLength += SeLengthSid(Sid[i]);
//         AclLength += AceLength;
//     }
//
//     /* Phase 3: Allocate the ACL */
//     Acl = ExAllocatePoolWithTag(PagedPool, AclLength, TAG_CMSD);
//     if (!Acl) KeBugCheckEx(REGISTRY_ERROR, 11, 3, 0, 0);
//
//     /* Phase 4: Create the ACL */
//     Status = RtlCreateAcl(Acl, AclLength, ACL_REVISION);
//     if (!NT_SUCCESS(Status)) KeBugCheckEx(REGISTRY_ERROR, 11, 4, Status, 0);
//
//     /* Phase 5: Build the ACL */
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION, KEY_ALL_ACCESS, Sid[2]);
//     Status |= RtlAddAccessAllowedAce(Acl, ACL_REVISION, KEY_ALL_ACCESS, Sid[3]);
//     Status |= RtlAddAccessAllowedAce(Acl, ACL_REVISION, KEY_READ, Sid[0]);
//     Status |= RtlAddAccessAllowedAce(Acl, ACL_REVISION, KEY_READ, Sid[1]);
//     if (!NT_SUCCESS(Status)) KeBugCheckEx(REGISTRY_ERROR, 11, 5, Status, 0);
//
//     /* Phase 5: Make the ACEs inheritable */
//     Status = RtlGetAce(Acl, 0, (PVOID*)&AceHeader);
//     ASSERT(NT_SUCCESS(Status));
//     AceHeader->AceFlags |= CONTAINER_INHERIT_ACE;
//     Status = RtlGetAce(Acl, 1, (PVOID*)&AceHeader);
//     ASSERT(NT_SUCCESS(Status));
//     AceHeader->AceFlags |= CONTAINER_INHERIT_ACE;
//     Status = RtlGetAce(Acl, 2, (PVOID*)&AceHeader);
//     ASSERT(NT_SUCCESS(Status));
//     AceHeader->AceFlags |= CONTAINER_INHERIT_ACE;
//     Status = RtlGetAce(Acl, 3, (PVOID*)&AceHeader);
//     ASSERT(NT_SUCCESS(Status));
//     AceHeader->AceFlags |= CONTAINER_INHERIT_ACE;
//
//     /* Phase 6: Allocate the security descriptor and make space for the ACL */
//     SecurityDescriptor = ExAllocatePoolWithTag(PagedPool,
//                                                sizeof(SECURITY_DESCRIPTOR) +
//                                                AclLength,
//                                                TAG_CMSD);
//     if (!SecurityDescriptor) KeBugCheckEx(REGISTRY_ERROR, 11, 6, 0, 0);
//
//     /* Phase 6: Make a copy of the ACL */
//     AclCopy = (PACL)((PISECURITY_DESCRIPTOR)SecurityDescriptor + 1);
//     RtlCopyMemory(AclCopy, Acl, AclLength);
//
//     /* Phase 7: Create the security descriptor */
//     Status = RtlCreateSecurityDescriptor(SecurityDescriptor,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status)) KeBugCheckEx(REGISTRY_ERROR, 11, 7, Status, 0);
//
//     /* Phase 8: Set the ACL as a DACL */
//     Status = RtlSetDaclSecurityDescriptor(SecurityDescriptor,
//                                           TRUE,
//                                           AclCopy,
//                                           FALSE);
//     if (!NT_SUCCESS(Status)) KeBugCheckEx(REGISTRY_ERROR, 11, 8, Status, 0);
//
//     /* Free the SIDs and original ACL */
//     for (i = 0; i < 4; i++) ExFreePoolWithTag(Sid[i], TAG_CMSD);
//     ExFreePoolWithTag(Acl, TAG_CMSD);
//
//     /* Return the security descriptor */
//     return SecurityDescriptor;
// }
//
// NTSTATUS
// CmpQuerySecurityDescriptor(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                            IN SECURITY_INFORMATION SecurityInformation,
//                            OUT PSECURITY_DESCRIPTOR SecurityDescriptor,
//                            IN OUT PULONG BufferLength)
// {
//     PISECURITY_DESCRIPTOR_RELATIVE RelSd;
//     ULONG SidSize;
//     ULONG AclSize;
//     ULONG SdSize;
//     NTSTATUS Status;
//     SECURITY_DESCRIPTOR_CONTROL Control = 0;
//     ULONG Owner = 0;
//     ULONG Group = 0;
//     ULONG Dacl = 0;
//
//     DBG_UNREFERENCED_PARAMETER(Kcb);
//
//     DPRINT("CmpQuerySecurityDescriptor()\n");
//
//     if (SecurityInformation == 0)
//     {
//         return STATUS_ACCESS_DENIED;
//     }
//
//     SidSize = RtlLengthSid(SeWorldSid);
//     RelSd = SecurityDescriptor;
//     SdSize = sizeof(*RelSd);
//
//     if (SecurityInformation & OWNER_SECURITY_INFORMATION)
//     {
//         Owner = SdSize;
//         SdSize += SidSize;
//     }
//
//     if (SecurityInformation & GROUP_SECURITY_INFORMATION)
//     {
//         Group = SdSize;
//         SdSize += SidSize;
//     }
//
//     if (SecurityInformation & DACL_SECURITY_INFORMATION)
//     {
//         Control |= SE_DACL_PRESENT;
//         Dacl = SdSize;
//         AclSize = sizeof(ACL) + sizeof(ACE) + SidSize;
//         SdSize += AclSize;
//     }
//
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//     {
//         Control |= SE_SACL_PRESENT;
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
//     Status = RtlCreateSecurityDescriptorRelative(RelSd,
//                                                  SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     RelSd->Control |= Control;
//     RelSd->Owner = Owner;
//     RelSd->Group = Group;
//     RelSd->Dacl = Dacl;
//
//     if (Owner)
//         RtlCopyMemory((PUCHAR)RelSd + Owner,
//                       SeWorldSid,
//                       SidSize);
//
//     if (Group)
//         RtlCopyMemory((PUCHAR)RelSd + Group,
//                       SeWorldSid,
//                       SidSize);
//
//     if (Dacl)
//     {
//         Status = RtlCreateAcl((PACL)((PUCHAR)RelSd + Dacl),
//                               AclSize,
//                               ACL_REVISION);
//         if (NT_SUCCESS(Status))
//         {
//             Status = RtlAddAccessAllowedAce((PACL)((PUCHAR)RelSd + Dacl),
//                                             ACL_REVISION,
//                                             GENERIC_ALL,
//                                             SeWorldSid);
//         }
//     }
//
//     ASSERT(Status == STATUS_SUCCESS);
//     return Status;
// }
//
// NTSTATUS
// CmpSetSecurityDescriptor(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                          IN PSECURITY_INFORMATION SecurityInformation,
//                          IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                          IN POOL_TYPE PoolType,
//                          IN PGENERIC_MAPPING GenericMapping)
// {
//     DPRINT("CmpSetSecurityDescriptor()\n");
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// CmpAssignSecurityDescriptor(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                             IN PSECURITY_DESCRIPTOR SecurityDescriptor)
// {
//     DPRINT("CmpAssignSecurityDescriptor(%p %p)\n",
//            Kcb, SecurityDescriptor);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// CmpSecurityMethod(IN PVOID ObjectBody,
//                   IN SECURITY_OPERATION_CODE OperationCode,
//                   IN PSECURITY_INFORMATION SecurityInformation,
//                   IN OUT PSECURITY_DESCRIPTOR SecurityDescriptor,
//                   IN OUT PULONG BufferLength,
//                   IN OUT PSECURITY_DESCRIPTOR *OldSecurityDescriptor,
//                   IN POOL_TYPE PoolType,
//                   IN PGENERIC_MAPPING GenericMapping)
// {
//     PCM_KEY_CONTROL_BLOCK Kcb;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DBG_UNREFERENCED_PARAMETER(OldSecurityDescriptor);
//     DBG_UNREFERENCED_PARAMETER(GenericMapping);
//
//     Kcb = ((PCM_KEY_BODY)ObjectBody)->KeyControlBlock;
//
//     /* Acquire the hive lock */
//     CmpLockRegistry();
//
//     /* Acquire the KCB lock */
//     if (OperationCode == QuerySecurityDescriptor)
//     {
//         /* Avoid recursive locking if somebody already holds it */
//         if (!((PCM_KEY_BODY)ObjectBody)->KcbLocked)
//         {
//             CmpAcquireKcbLockShared(Kcb);
//         }
//     }
//     else
//     {
//         ASSERT(!((PCM_KEY_BODY)ObjectBody)->KcbLocked);
//         CmpAcquireKcbLockExclusive(Kcb);
//     }
//
//     /* Don't touch deleted keys */
//     if (Kcb->Delete)
//     {
//         /* Release the KCB lock */
//         CmpReleaseKcbLock(Kcb);
//
//         /* Release the hive lock */
//         CmpUnlockRegistry();
//         return STATUS_KEY_DELETED;
//     }
//
//     switch (OperationCode)
//     {
//         case SetSecurityDescriptor:
//             DPRINT("Set security descriptor\n");
//             ASSERT((PoolType == PagedPool) || (PoolType == NonPagedPool));
//             Status = CmpSetSecurityDescriptor(Kcb,
//                                               SecurityInformation,
//                                               SecurityDescriptor,
//                                               PoolType,
//                                               GenericMapping);
//             break;
//
//         case QuerySecurityDescriptor:
//             DPRINT("Query security descriptor\n");
//             Status = CmpQuerySecurityDescriptor(Kcb,
//                                                 *SecurityInformation,
//                                                 SecurityDescriptor,
//                                                 BufferLength);
//             break;
//
//         case DeleteSecurityDescriptor:
//             DPRINT("Delete security descriptor\n");
//             /* HACK */
//             break;
//
//         case AssignSecurityDescriptor:
//             DPRINT("Assign security descriptor\n");
//             Status = CmpAssignSecurityDescriptor(Kcb,
//                                                  SecurityDescriptor);
//             break;
//
//         default:
//             KeBugCheckEx(SECURITY_SYSTEM, 0, STATUS_INVALID_PARAMETER, 0, 0);
//     }
//
//     /*
//      * Release the KCB lock, but only if we locked it ourselves and
//      * nobody else was locking it by themselves.
//      */
//     if (!((PCM_KEY_BODY)ObjectBody)->KcbLocked)
//     {
//         CmpReleaseKcbLock(Kcb);
//     }
//
//     /* Release the hive lock */
//     CmpUnlockRegistry();
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
