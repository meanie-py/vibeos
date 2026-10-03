//! sdk/lib/rtl/memres.c (upstream, 2026): encode/decode the Start/Length/Alignment of
//! port and memory resource descriptors, including the CmResourceTypeMemoryLarge forms that
//! carry 64-bit lengths in the 32-bit field by dropping known-zero low bits. ntoskrnl exports
//! the family (ntoskrnl.spec) for pcix and the arbiters, as upstream does.

use ntkrnl_sys::base::{NTSTATUS, STATUS_SUCCESS};
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_UNSUCCESSFUL};
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, IO_RESOURCE_DESCRIPTOR};

// xdk/cmtypes.h. ntkrnl_sys::hal_ddk carries these on x86/x64 only, so they are spelled here.
const CmResourceTypePort: u32 = 1;
const CmResourceTypeMemory: u32 = 3;
const CmResourceTypeMemoryLarge: u32 = 7;
const CM_RESOURCE_MEMORY_LARGE: u32 = 0x0E00;
const CM_RESOURCE_MEMORY_LARGE_40: u32 = 0x0200;
const CM_RESOURCE_MEMORY_LARGE_48: u32 = 0x0400;
const CM_RESOURCE_MEMORY_LARGE_64: u32 = 0x0800;

const MAXULONG: u64 = 0xFFFF_FFFF;

/// RtlpDecodeMemIoValue: a plain Port/Memory field is the value itself; a
/// MemoryLarge field drops 8/16/32 low bits according to the LARGE_* flag.
fn RtlpDecodeMemIoValue(Type: u8, Flags: u16, Encoded: u32) -> u64 {
    if Type as u32 == CmResourceTypePort || Type as u32 == CmResourceTypeMemory {
        return Encoded as u64;
    }
    if Flags as u32 & CM_RESOURCE_MEMORY_LARGE_40 != 0 {
        return (Encoded as u64) << 8;
    }
    if Flags as u32 & CM_RESOURCE_MEMORY_LARGE_48 != 0 {
        return (Encoded as u64) << 16;
    }
    if Flags as u32 & CM_RESOURCE_MEMORY_LARGE_64 != 0 {
        return (Encoded as u64) << 32;
    }
    0
}

/// `ULONGLONG RtlCmDecodeMemIoResource(PCM_PARTIAL_RESOURCE_DESCRIPTOR, PULONGLONG Start OPTIONAL)`
#[no_mangle]
pub unsafe extern "system" fn RtlCmDecodeMemIoResource(
    Descriptor: *const CM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: *mut u64,
) -> u64 {
    if !Start.is_null() {
        *Start = (*Descriptor).u.Generic.Start.QuadPart as u64;
    }
    RtlpDecodeMemIoValue((*Descriptor).Type, (*Descriptor).Flags, (*Descriptor).u.Generic.Length)
}

/// `ULONGLONG RtlIoDecodeMemIoResource(PIO_RESOURCE_DESCRIPTOR, PULONGLONG Alignment OPTIONAL,
/// PULONGLONG MinimumAddress OPTIONAL, PULONGLONG MaximumAddress OPTIONAL)`
#[no_mangle]
pub unsafe extern "system" fn RtlIoDecodeMemIoResource(
    Descriptor: *const IO_RESOURCE_DESCRIPTOR,
    Alignment: *mut u64,
    MinimumAddress: *mut u64,
    MaximumAddress: *mut u64,
) -> u64 {
    if !Alignment.is_null() {
        *Alignment = RtlpDecodeMemIoValue(
            (*Descriptor).Type,
            (*Descriptor).Flags,
            (*Descriptor).u.Generic.Alignment,
        );
    }
    if !MinimumAddress.is_null() {
        *MinimumAddress = (*Descriptor).u.Generic.MinimumAddress.QuadPart as u64;
    }
    if !MaximumAddress.is_null() {
        *MaximumAddress = (*Descriptor).u.Generic.MaximumAddress.QuadPart as u64;
    }
    RtlpDecodeMemIoValue((*Descriptor).Type, (*Descriptor).Flags, (*Descriptor).u.Generic.Length)
}

/// RtlpEncodeLargeAlignment: scale the alignment up until it is a multiple of
/// the large form's granularity; FALSE if that overflows 64 bits.
fn RtlpEncodeLargeAlignment(mut Alignment: u64, Shift: u32, Encoded: &mut u32) -> bool {
    let LowMask: u64 = (1u64 << Shift) - 1;
    while Alignment & LowMask != 0 {
        let Doubled = Alignment << 1;
        if Doubled < Alignment {
            return false;
        }
        Alignment = Doubled;
    }
    *Encoded = (Alignment >> Shift) as u32;
    true
}

/// `NTSTATUS RtlIoEncodeMemIoResource(PIO_RESOURCE_DESCRIPTOR, UCHAR Type, ULONGLONG Length,
/// ULONGLONG Alignment, ULONGLONG MinimumAddress, ULONGLONG MaximumAddress)`
#[no_mangle]
pub unsafe extern "system" fn RtlIoEncodeMemIoResource(
    Descriptor: *mut IO_RESOURCE_DESCRIPTOR,
    Type: u8,
    Length: u64,
    Alignment: u64,
    MinimumAddress: u64,
    MaximumAddress: u64,
) -> NTSTATUS {
    let Shift: u32;
    let LargeFlag: u32;

    if Type as u32 != CmResourceTypePort
        && Type as u32 != CmResourceTypeMemory
        && Type as u32 != CmResourceTypeMemoryLarge
    {
        return STATUS_INVALID_PARAMETER;
    }

    /* A port descriptor is limited to 32-bit length and alignment. */
    if Type as u32 == CmResourceTypePort {
        if Length > MAXULONG || Alignment > MAXULONG {
            return STATUS_INVALID_PARAMETER;
        }
        (*Descriptor).Type = CmResourceTypePort as u8;
        (*Descriptor).u.Generic.MinimumAddress.QuadPart = MinimumAddress as i64;
        (*Descriptor).u.Generic.MaximumAddress.QuadPart = MaximumAddress as i64;
        (*Descriptor).u.Generic.Length = Length as u32;
        (*Descriptor).u.Generic.Alignment = Alignment as u32;
        return STATUS_SUCCESS;
    }

    /* Memory: reset any stale large-form flags and record the address bounds. */
    (*Descriptor).Flags &= !(CM_RESOURCE_MEMORY_LARGE as u16);
    (*Descriptor).u.Generic.MinimumAddress.QuadPart = MinimumAddress as i64;
    (*Descriptor).u.Generic.MaximumAddress.QuadPart = MaximumAddress as i64;

    /* Both values fit 32 bits: a plain memory descriptor. */
    if Length <= MAXULONG && Alignment <= MAXULONG {
        (*Descriptor).Type = CmResourceTypeMemory as u8;
        (*Descriptor).u.Generic.Length = Length as u32;
        (*Descriptor).u.Generic.Alignment = Alignment as u32;
        return STATUS_SUCCESS;
    }

    /* Select the large form from the length; the alignment must fit that tier. */
    if Length <= (MAXULONG << 8) {
        if Alignment > (MAXULONG << 8) {
            return STATUS_UNSUCCESSFUL;
        }
        Shift = 8;
        LargeFlag = CM_RESOURCE_MEMORY_LARGE_40;
    } else if Length <= (MAXULONG << 16) {
        if Alignment > (MAXULONG << 16) {
            return STATUS_UNSUCCESSFUL;
        }
        Shift = 16;
        LargeFlag = CM_RESOURCE_MEMORY_LARGE_48;
    } else if Length <= (MAXULONG << 32) {
        Shift = 32;
        LargeFlag = CM_RESOURCE_MEMORY_LARGE_64;
    } else {
        return STATUS_UNSUCCESSFUL;
    }

    /* The length must be exactly representable at this granularity. */
    if Length & ((1u64 << Shift) - 1) != 0 {
        return STATUS_UNSUCCESSFUL;
    }

    let mut EncodedAlignment: u32 = 0;
    if !RtlpEncodeLargeAlignment(Alignment, Shift, &mut EncodedAlignment) {
        return STATUS_UNSUCCESSFUL;
    }
    (*Descriptor).u.Generic.Alignment = EncodedAlignment;

    (*Descriptor).Type = CmResourceTypeMemoryLarge as u8;
    (*Descriptor).u.Generic.Length = (Length >> Shift) as u32;
    (*Descriptor).Flags |= LargeFlag as u16;
    STATUS_SUCCESS
}

/// `NTSTATUS RtlCmEncodeMemIoResource(PCM_PARTIAL_RESOURCE_DESCRIPTOR, UCHAR Type,
/// ULONGLONG Length, ULONGLONG Start)`: the assigned-resource counterpart of the Io encoder.
/// Picks the tightest large form whose dropped low bits are zero.
#[no_mangle]
pub unsafe extern "system" fn RtlCmEncodeMemIoResource(
    Descriptor: *mut CM_PARTIAL_RESOURCE_DESCRIPTOR,
    Type: u8,
    Length: u64,
    Start: u64,
) -> NTSTATUS {
    if Type as u32 != CmResourceTypePort
        && Type as u32 != CmResourceTypeMemory
        && Type as u32 != CmResourceTypeMemoryLarge
    {
        return STATUS_INVALID_PARAMETER;
    }

    /* A port descriptor has only a 32-bit length and no large form. */
    if Type as u32 == CmResourceTypePort {
        if Length > MAXULONG {
            return STATUS_INVALID_PARAMETER;
        }
        (*Descriptor).Type = CmResourceTypePort as u8;
        (*Descriptor).u.Generic.Start.QuadPart = Start as i64;
        (*Descriptor).u.Generic.Length = Length as u32;
        return STATUS_SUCCESS;
    }

    /* Drop any stale large-form flags and record the start address. */
    (*Descriptor).Flags &= !(CM_RESOURCE_MEMORY_LARGE as u16);
    (*Descriptor).u.Generic.Start.QuadPart = Start as i64;

    /* A length that fits 32 bits stays a plain memory descriptor. */
    if Length <= MAXULONG {
        (*Descriptor).Type = CmResourceTypeMemory as u8;
        (*Descriptor).u.Generic.Length = Length as u32;
        return STATUS_SUCCESS;
    }

    /* Otherwise pick the tightest large form whose dropped low bits are zero. */
    if Length <= (MAXULONG << 8) && (Length & 0xFF) == 0 {
        (*Descriptor).u.Generic.Length = (Length >> 8) as u32;
        (*Descriptor).Flags |= CM_RESOURCE_MEMORY_LARGE_40 as u16;
    } else if Length <= (MAXULONG << 16) && (Length & 0xFFFF) == 0 {
        (*Descriptor).u.Generic.Length = (Length >> 16) as u32;
        (*Descriptor).Flags |= CM_RESOURCE_MEMORY_LARGE_48 as u16;
    } else if Length <= (MAXULONG << 32) && (Length & 0xFFFF_FFFF) == 0 {
        (*Descriptor).u.Generic.Length = (Length >> 32) as u32;
        (*Descriptor).Flags |= CM_RESOURCE_MEMORY_LARGE_64 as u16;
    } else {
        return STATUS_UNSUCCESSFUL;
    }

    (*Descriptor).Type = CmResourceTypeMemoryLarge as u8;
    STATUS_SUCCESS
}

/// `NTSTATUS RtlFindClosestEncodableLength(ULONGLONG SourceLength, PULONGLONG TargetLength)`:
/// round a length up to the nearest value a descriptor can carry.
#[no_mangle]
pub unsafe extern "system" fn RtlFindClosestEncodableLength(SourceLength: u64, TargetLength: *mut u64) -> NTSTATUS {
    if SourceLength <= MAXULONG {
        *TargetLength = SourceLength;
    } else if SourceLength <= (MAXULONG << 8) {
        *TargetLength = (SourceLength + 0xFF) & !0xFFu64;
    } else if SourceLength <= (MAXULONG << 16) {
        *TargetLength = (SourceLength + 0xFFFF) & !0xFFFFu64;
    } else if SourceLength <= (MAXULONG << 32) {
        // At most (MAXULONG << 32) + MAXULONG == u64::MAX: cannot overflow.
        *TargetLength = (SourceLength + 0xFFFF_FFFF) & !0xFFFF_FFFFu64;
    } else {
        *TargetLength = 0;
        return STATUS_UNSUCCESSFUL;
    }
    STATUS_SUCCESS
}
