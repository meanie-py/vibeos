//! SE subsystem inline functions — security descriptor accessors, token locks.

use crate::base::*;
use crate::constants::*;
use crate::ddk;

// ============================================================================
// Security descriptor accessors (FORCEINLINE in ntoskrnl/include/internal/se.h)
// ============================================================================

/// SepGetGroupFromDescriptor — get group SID from security descriptor.
///
/// If SE_SELF_RELATIVE, returns pointer within descriptor at Group offset.
/// Otherwise returns the Group pointer directly.
#[inline]
pub unsafe fn SepGetGroupFromDescriptor(
    descriptor: ddk::PISECURITY_DESCRIPTOR,
) -> ddk::PSID {
    if (*descriptor).Control & SE_SELF_RELATIVE != 0 {
        let sd_rel = descriptor as ddk::PISECURITY_DESCRIPTOR_RELATIVE;
        let offset = (*sd_rel).Group;
        if offset == 0 {
            core::ptr::null_mut()
        } else {
            (descriptor as *mut u8).add(offset as usize) as ddk::PSID
        }
    } else {
        (*descriptor).Group
    }
}

/// SepGetOwnerFromDescriptor — get owner SID from security descriptor.
#[inline]
pub unsafe fn SepGetOwnerFromDescriptor(
    descriptor: ddk::PISECURITY_DESCRIPTOR,
) -> ddk::PSID {
    if (*descriptor).Control & SE_SELF_RELATIVE != 0 {
        let sd_rel = descriptor as ddk::PISECURITY_DESCRIPTOR_RELATIVE;
        let offset = (*sd_rel).Owner;
        if offset == 0 {
            core::ptr::null_mut()
        } else {
            (descriptor as *mut u8).add(offset as usize) as ddk::PSID
        }
    } else {
        (*descriptor).Owner
    }
}

/// SepGetDaclFromDescriptor — get DACL from security descriptor.
///
/// Returns NULL if SE_DACL_PRESENT is not set.
#[inline]
pub unsafe fn SepGetDaclFromDescriptor(
    descriptor: ddk::PISECURITY_DESCRIPTOR,
) -> ddk::PACL {
    if (*descriptor).Control & SE_DACL_PRESENT == 0 {
        return core::ptr::null_mut();
    }

    if (*descriptor).Control & SE_SELF_RELATIVE != 0 {
        let sd_rel = descriptor as ddk::PISECURITY_DESCRIPTOR_RELATIVE;
        let offset = (*sd_rel).Dacl;
        if offset == 0 {
            core::ptr::null_mut()
        } else {
            (descriptor as *mut u8).add(offset as usize) as ddk::PACL
        }
    } else {
        (*descriptor).Dacl
    }
}

/// SepGetSaclFromDescriptor — get SACL from security descriptor.
///
/// Returns NULL if SE_SACL_PRESENT is not set.
#[inline]
pub unsafe fn SepGetSaclFromDescriptor(
    descriptor: ddk::PISECURITY_DESCRIPTOR,
) -> ddk::PACL {
    if (*descriptor).Control & SE_SACL_PRESENT == 0 {
        return core::ptr::null_mut();
    }

    if (*descriptor).Control & SE_SELF_RELATIVE != 0 {
        let sd_rel = descriptor as ddk::PISECURITY_DESCRIPTOR_RELATIVE;
        let offset = (*sd_rel).Sacl;
        if offset == 0 {
            core::ptr::null_mut()
        } else {
            (descriptor as *mut u8).add(offset as usize) as ddk::PACL
        }
    } else {
        (*descriptor).Sacl
    }
}

// ============================================================================
// Token lock macros (from ntoskrnl/include/internal/se.h)
// ============================================================================

/// SepAcquireTokenLockExclusive — enter critical region + acquire exclusive.
#[inline]
pub unsafe fn SepAcquireTokenLockExclusive(token: ddk::PTOKEN) {
    ddk::KeEnterCriticalRegion();
    ddk::ExAcquireResourceExclusiveLite((*token).TokenLock, TRUE as u8);
}

/// SepAcquireTokenLockShared — enter critical region + acquire shared.
#[inline]
pub unsafe fn SepAcquireTokenLockShared(token: ddk::PTOKEN) {
    ddk::KeEnterCriticalRegion();
    ddk::ExAcquireResourceSharedLite((*token).TokenLock, TRUE as u8);
}

/// SepReleaseTokenLock — release resource + leave critical region.
#[inline]
pub unsafe fn SepReleaseTokenLock(token: ddk::PTOKEN) {
    ddk::ExReleaseResourceLite((*token).TokenLock);
    ddk::KeLeaveCriticalRegion();
}

// ============================================================================
// SeQuerySubjectContextToken (macro from sefuncs.h)
// ============================================================================

/// SeQuerySubjectContextToken — return effective token (client or primary).
#[inline]
pub unsafe fn SeQuerySubjectContextToken(
    ctx: *const ddk::_SECURITY_SUBJECT_CONTEXT,
) -> PVOID {
    let client = (*ctx).ClientToken;
    if !client.is_null() {
        client
    } else {
        (*ctx).PrimaryToken
    }
}

// ============================================================================
// SeLengthSid (macro from sefuncs.h)
// ============================================================================

/// SeLengthSid — compute byte length of a SID.
/// C: `8 + 4 * ((SID*)Sid)->SubAuthorityCount`
#[inline]
pub unsafe fn SeLengthSid(sid: ddk::PSID) -> ULONG {
    let s = sid as *const ddk::_SID;
    8 + 4 * (*s).SubAuthorityCount as ULONG
}
