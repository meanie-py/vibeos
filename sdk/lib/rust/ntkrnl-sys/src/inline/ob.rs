//! OB subsystem inline functions.
//!
//! Object locks, directory locks, lookup contexts, name info, create info,
//! callout tracing, attribute validation.

use crate::base::*;
use crate::constants::*;
use crate::ddk;
use crate::macros;

// OBP_LOCK_STATE signatures from ob_x.h
const OBP_LOCK_STATE_PRE_ACQUISITION_EXCLUSIVE: ULONG = 0xAAAA1234;
const OBP_LOCK_STATE_PRE_ACQUISITION_SHARED: ULONG = 0xBBBB1234;
const OBP_LOCK_STATE_POST_ACQUISITION_EXCLUSIVE: ULONG = 0xCCCC1234;
const OBP_LOCK_STATE_POST_ACQUISITION_SHARED: ULONG = 0xDDDD1234;
const OBP_LOCK_STATE_RELEASED: ULONG = 0xEEEE1234;
const OBP_LOCK_STATE_INITIALIZED: ULONG = 0xFFFF1234;

// ============================================================================
// ObpSelectObjectLockSlot (FORCEINLINE from ob_x.h)
// ============================================================================

/// ObpSelectObjectLockSlot — hash object header address to lock slot index (0-3).
/// C: `return (((ULONG_PTR)ObjectHeader) >> 8) & 3;`
#[inline]
pub unsafe fn ObpSelectObjectLockSlot(
    header: ddk::POBJECT_HEADER,
) -> ULONG {
    (((header as usize) >> 8) & 3) as ULONG
}

// ============================================================================
// ObpValidateAttributes (FORCEINLINE from ob_x.h)
// ============================================================================

/// ObpValidateAttributes — mask object attributes based on processor mode.
#[inline]
pub unsafe fn ObpValidateAttributes(
    attributes: ULONG,
    previous_mode: i8,
) -> ULONG {
    if previous_mode == KernelMode {
        attributes & OBJ_VALID_KERNEL_ATTRIBUTES
    } else {
        (attributes & OBJ_VALID_ATTRIBUTES) & !OBJ_KERNEL_HANDLE
    }
}

// ============================================================================
// ObpAcquireObjectLock / ObpReleaseObjectLock / ObpAcquireObjectLockShared
// ============================================================================

/// ObpAcquireObjectLock — acquire exclusive ERESOURCE lock on object type.
#[inline]
pub unsafe fn ObpAcquireObjectLock(header: ddk::POBJECT_HEADER) {
    // ob_x.h's ASSERTs, restored (scan_dropped_asserts): RtlAssert prints and continues.
    crate::ASSERT!(crate::ke::KeGetCurrentIrql() <= crate::base::APC_LEVEL);
    let object_type = (*header).Type;
    let slot = ObpSelectObjectLockSlot(header) as usize;
    ddk::KeEnterCriticalRegion();
    ddk::ExAcquireResourceExclusiveLite(
        &mut (*object_type).ObjectLocks[slot],
        1, // TRUE = wait
    );
}

/// ObpReleaseObjectLock — release ERESOURCE lock on object type.
#[inline]
pub unsafe fn ObpReleaseObjectLock(header: ddk::POBJECT_HEADER) {
    let object_type = (*header).Type;
    let slot = ObpSelectObjectLockSlot(header) as usize;
    ddk::ExReleaseResourceLite(&mut (*object_type).ObjectLocks[slot]);
    ddk::KeLeaveCriticalRegion();
    crate::ASSERT!(crate::ke::KeGetCurrentIrql() <= crate::base::APC_LEVEL);
}

/// ObpAcquireObjectLockShared — acquire shared ERESOURCE lock on object type.
#[inline]
pub unsafe fn ObpAcquireObjectLockShared(header: ddk::POBJECT_HEADER) {
    crate::ASSERT!(crate::ke::KeGetCurrentIrql() <= crate::base::APC_LEVEL);
    let object_type = (*header).Type;
    let slot = ObpSelectObjectLockSlot(header) as usize;
    ddk::KeEnterCriticalRegion();
    ddk::ExAcquireResourceSharedLite(
        &mut (*object_type).ObjectLocks[slot],
        1, // TRUE = wait
    );
}

// ============================================================================
// ObpEnterObjectTypeMutex / ObpLeaveObjectTypeMutex
// ============================================================================

/// ObpEnterObjectTypeMutex — acquire exclusive ERESOURCE on ObjectType.Mutex.
#[inline]
pub unsafe fn ObpEnterObjectTypeMutex(object_type: ddk::POBJECT_TYPE) {
    crate::ASSERT!(crate::ke::KeGetCurrentIrql() <= crate::base::APC_LEVEL);
    ddk::KeEnterCriticalRegion();
    ddk::ExAcquireResourceExclusiveLite(&mut (*object_type).Mutex, 1);
}

/// ObpLeaveObjectTypeMutex — release ERESOURCE on ObjectType.Mutex.
#[inline]
pub unsafe fn ObpLeaveObjectTypeMutex(object_type: ddk::POBJECT_TYPE) {
    ddk::ExReleaseResourceLite(&mut (*object_type).Mutex);
    ddk::KeLeaveCriticalRegion();
    crate::ASSERT!(crate::ke::KeGetCurrentIrql() <= crate::base::APC_LEVEL);
}

// ============================================================================
// ObpCalloutStart / ObpCalloutEnd (FORCEINLINE from ob_x.h)
// ============================================================================

/// ObpCalloutStart — save IRQL before callout (no-op in release builds).
#[inline]
pub unsafe fn ObpCalloutStart(_callout_irql: *mut u8) {
    // Release build: UNREFERENCED_PARAMETER(CalloutIrql)
}

/// ObpCalloutEnd — verify IRQL after callout (no-op in release builds).
#[inline]
pub unsafe fn ObpCalloutEnd(
    _callout_irql: u8,
    _procedure: *const u8,
    _object_type: ddk::POBJECT_TYPE,
    _object: PVOID,
) {
    // Release build: UNREFERENCED_PARAMETER(CalloutIrql)
}

// ============================================================================
// ObpReferenceNameInfo (FORCEINLINE from ob_x.h)
// ============================================================================

/// ObpReferenceNameInfo — atomically increment name info query references.
/// Returns NULL if name info doesn't exist or references are zero.
#[inline]
pub unsafe fn ObpReferenceNameInfo(
    header: ddk::POBJECT_HEADER,
) -> ddk::POBJECT_HEADER_NAME_INFO {
    use core::sync::atomic::{AtomicI32, Ordering};

    let name_info = macros::OBJECT_HEADER_TO_NAME_INFO(header);
    if name_info.is_null() {
        return core::ptr::null_mut();
    }

    let refs_ptr = core::ptr::addr_of!((*name_info).QueryReferences) as *const AtomicI32;
    let mut references = (*refs_ptr).load(Ordering::Relaxed) as u32;

    loop {
        if references == 0 {
            return core::ptr::null_mut();
        }

        match (*refs_ptr).compare_exchange(
            references as i32,
            (references + 1) as i32,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => break,
            Err(actual) => references = actual as u32,
        }
    }

    // C: the 0x80000000 "magic" flag is an unhandled path.
    if (*refs_ptr).load(Ordering::Relaxed) as u32 & 0x8000_0000 != 0 {
        crate::driver_ddk::DbgPrint(b"OB: Unhandled path\n\0".as_ptr() as *const i8);
        crate::ASSERT!(false);
    }

    name_info
}

// ============================================================================
// ObpDereferenceNameInfo (FORCEINLINE from ob_x.h)
// ============================================================================

/// ObpDereferenceNameInfo — atomically decrement name info query references.
/// When references reach zero, free the name buffer and dereference directory.
#[inline]
pub unsafe fn ObpDereferenceNameInfo(
    name_info: ddk::POBJECT_HEADER_NAME_INFO,
) {
    use core::sync::atomic::{AtomicI32, Ordering};

    if name_info.is_null() {
        return;
    }

    let refs_ptr = core::ptr::addr_of!((*name_info).QueryReferences) as *const AtomicI32;
    let old = (*refs_ptr).fetch_sub(1, Ordering::SeqCst);

    if old == 1 {
        // References dropped to zero
        if !(*name_info).Name.Buffer.is_null() {
            ddk::ExFreePoolWithTag(
                (*name_info).Name.Buffer as PVOID,
                OB_NAME_TAG,
            );
            // RtlInitEmptyUnicodeString
            (*name_info).Name.Length = 0;
            (*name_info).Name.MaximumLength = 0;
            (*name_info).Name.Buffer = core::ptr::null_mut();
        }
        let directory = (*name_info).Directory;
        if !directory.is_null() {
            (*name_info).Directory = core::ptr::null_mut();
            ddk::ObDereferenceObjectDeferDelete(directory as PVOID);
        }
    }
}

// ============================================================================
// ObpInitializeLookupContext (FORCEINLINE from ob_x.h)
// ============================================================================

/// ObpInitializeLookupContext — zero out a lookup context.
#[inline]
pub unsafe fn ObpInitializeLookupContext(
    context: ddk::POBP_LOOKUP_CONTEXT,
) {
    (*context).Object = core::ptr::null_mut();
    (*context).Directory = core::ptr::null_mut();
    (*context).DirectoryLocked = 0; // FALSE
    (*context).LockStateSignature = OBP_LOCK_STATE_INITIALIZED;
}

// ============================================================================
// Directory lock functions (FORCEINLINE from ob_x.h)
// ============================================================================

/// ObpAcquireDirectoryLockShared — acquire push lock shared on directory.
#[inline]
pub unsafe fn ObpAcquireDirectoryLockShared(
    directory: ddk::POBJECT_DIRECTORY,
    context: ddk::POBP_LOOKUP_CONTEXT,
) {
    (*context).LockStateSignature = OBP_LOCK_STATE_PRE_ACQUISITION_SHARED;
    ddk::KeEnterCriticalRegion();
    super::ex::ExAcquirePushLockShared(
        &mut (*directory).Lock as *mut ddk::_EX_PUSH_LOCK,
    );
    (*context).LockStateSignature = OBP_LOCK_STATE_POST_ACQUISITION_SHARED;
}

/// ObpAcquireDirectoryLockExclusive — acquire push lock exclusive on directory.
#[inline]
pub unsafe fn ObpAcquireDirectoryLockExclusive(
    directory: ddk::POBJECT_DIRECTORY,
    context: ddk::POBP_LOOKUP_CONTEXT,
) {
    (*context).LockStateSignature = OBP_LOCK_STATE_PRE_ACQUISITION_EXCLUSIVE;
    ddk::KeEnterCriticalRegion();
    super::ex::ExAcquirePushLockExclusive(
        &mut (*directory).Lock as *mut ddk::_EX_PUSH_LOCK,
    );
    (*context).LockStateSignature = OBP_LOCK_STATE_POST_ACQUISITION_EXCLUSIVE;
}

/// ObpReleaseDirectoryLock — release directory push lock.
#[inline]
pub unsafe fn ObpReleaseDirectoryLock(
    directory: ddk::POBJECT_DIRECTORY,
    context: ddk::POBP_LOOKUP_CONTEXT,
) {
    super::ex::ExReleasePushLock(
        &mut (*directory).Lock as *mut ddk::_EX_PUSH_LOCK,
    );
    (*context).LockStateSignature = OBP_LOCK_STATE_RELEASED;
    ddk::KeLeaveCriticalRegion();
}

// ============================================================================
// Lookup context management (FORCEINLINE from ob_x.h)
// ============================================================================

/// ObpAcquireLookupContextLock — acquire exclusive directory lock + set context.
#[inline]
pub unsafe fn ObpAcquireLookupContextLock(
    context: ddk::POBP_LOOKUP_CONTEXT,
    directory: ddk::POBJECT_DIRECTORY,
) {
    ObpAcquireDirectoryLockExclusive(directory, context);
    (*context).Directory = directory;
    (*context).DirectoryLocked = 1; // TRUE
}

/// ObpReleaseLookupContextObject — dereference name info and object.
#[inline]
pub unsafe fn ObpReleaseLookupContextObject(
    context: ddk::POBP_LOOKUP_CONTEXT,
) {
    if !(*context).Object.is_null() {
        let object_header = macros::OBJECT_TO_OBJECT_HEADER((*context).Object);
        let name_info = macros::OBJECT_HEADER_TO_NAME_INFO(object_header);
        ObpDereferenceNameInfo(name_info);
        crate::ob::ObDereferenceObject((*context).Object);
        (*context).Object = core::ptr::null_mut();
    }
}

/// ObpReleaseLookupContext — release directory lock and cleanup context.
#[inline]
pub unsafe fn ObpReleaseLookupContext(
    context: ddk::POBP_LOOKUP_CONTEXT,
) {
    if (*context).DirectoryLocked != 0 {
        ObpReleaseDirectoryLock((*context).Directory, context);
        (*context).Directory = core::ptr::null_mut();
        (*context).DirectoryLocked = 0;
    }
    ObpReleaseLookupContextObject(context);
}

// ============================================================================
// Object create information (FORCEINLINE from ob_x.h)
// ============================================================================

/// ObpReleaseObjectCreateInformation — release security descriptor from create info.
#[inline]
pub unsafe fn ObpReleaseObjectCreateInformation(
    create_info: ddk::POBJECT_CREATE_INFORMATION,
) {
    if !(*create_info).SecurityDescriptor.is_null() {
        ddk::SeReleaseSecurityDescriptor(
            (*create_info).SecurityDescriptor,
            (*create_info).ProbeMode,
            1, // TRUE
        );
        (*create_info).SecurityDescriptor = core::ptr::null_mut();
    }
}

/// ObpAllocateObjectCreateInfoBuffer — allocate from per-processor lookaside list.
#[inline]
pub unsafe fn ObpAllocateObjectCreateInfoBuffer(
    lookaside_type: ddk::PP_NPAGED_LOOKASIDE_NUMBER,
) -> PVOID {
    let prcb = super::ke::KeGetCurrentPrcb();
    let type_idx = lookaside_type as usize;

    // Try processor-local list first (P)
    let list_p = (*prcb).PPLookasideList[type_idx].P as ddk::PNPAGED_LOOKASIDE_LIST;
    let l_p = core::ptr::addr_of_mut!((*list_p).L);
    (*l_p).TotalAllocates += 1;

    let list_head_p = core::ptr::addr_of_mut!((*l_p).__bindgen_anon_1.ListHead)
        as crate::base::PSLIST_HEADER;
    let buffer = crate::interlocked::InterlockedPopEntrySList(list_head_p);
    if !buffer.is_null() {
        return buffer as PVOID;
    }
    (*l_p).__bindgen_anon_2.AllocateMisses += 1;

    // Fall back to global list (L)
    let list_l = (*prcb).PPLookasideList[type_idx].L as ddk::PNPAGED_LOOKASIDE_LIST;
    let l_l = core::ptr::addr_of_mut!((*list_l).L);
    (*l_l).TotalAllocates += 1;

    let list_head_l = core::ptr::addr_of_mut!((*l_l).__bindgen_anon_1.ListHead)
        as crate::base::PSLIST_HEADER;
    let buffer = crate::interlocked::InterlockedPopEntrySList(list_head_l);
    if !buffer.is_null() {
        return buffer as PVOID;
    }
    (*l_l).__bindgen_anon_2.AllocateMisses += 1;

    // Both lists empty — call the allocator
    match (*l_l).__bindgen_anon_4.Allocate {
        Some(f) => f((*l_l).Type, (*l_l).Size as ddk::SIZE_T, (*l_l).Tag),
        None => core::ptr::null_mut(),
    }
}

/// ObpFreeCapturedAttributes — free to per-processor lookaside list.
#[inline]
pub unsafe fn ObpFreeCapturedAttributes(
    buffer: PVOID,
    lookaside_type: ddk::PP_NPAGED_LOOKASIDE_NUMBER,
) {
    let prcb = super::ke::KeGetCurrentPrcb();
    let type_idx = lookaside_type as usize;

    // Try processor-local list first (P)
    let list_p = (*prcb).PPLookasideList[type_idx].P as ddk::PNPAGED_LOOKASIDE_LIST;
    let l_p = core::ptr::addr_of_mut!((*list_p).L);
    (*l_p).TotalFrees += 1;

    let list_head_p = core::ptr::addr_of_mut!((*l_p).__bindgen_anon_1.ListHead)
        as crate::base::PSLIST_HEADER;
    let depth = crate::interlocked::ExQueryDepthSList(list_head_p);

    if depth >= (*l_p).Depth {
        (*l_p).__bindgen_anon_3.FreeMisses += 1;

        // Try global list (L)
        let list_l = (*prcb).PPLookasideList[type_idx].L as ddk::PNPAGED_LOOKASIDE_LIST;
        let l_l = core::ptr::addr_of_mut!((*list_l).L);
        (*l_l).TotalFrees += 1;

        let list_head_l = core::ptr::addr_of_mut!((*l_l).__bindgen_anon_1.ListHead)
            as crate::base::PSLIST_HEADER;
        let depth_l = crate::interlocked::ExQueryDepthSList(list_head_l);

        if depth_l >= (*l_l).Depth {
            (*l_l).__bindgen_anon_3.FreeMisses += 1;
            if let Some(f) = (*l_l).__bindgen_anon_5.Free {
                f(buffer);
            }
        } else {
            crate::interlocked::InterlockedPushEntrySList(
                list_head_l,
                buffer as crate::base::PSLIST_ENTRY,
            );
        }
    } else {
        crate::interlocked::InterlockedPushEntrySList(
            list_head_p,
            buffer as crate::base::PSLIST_ENTRY,
        );
    }
}

/// ObpFreeObjectCreateInformation — release create info and free buffer.
#[inline]
pub unsafe fn ObpFreeObjectCreateInformation(
    create_info: ddk::POBJECT_CREATE_INFORMATION,
) {
    ObpReleaseObjectCreateInformation(create_info);
    ObpFreeCapturedAttributes(
        create_info as PVOID,
        LookasideCreateInfoList as ddk::PP_NPAGED_LOOKASIDE_NUMBER,
    );
}
