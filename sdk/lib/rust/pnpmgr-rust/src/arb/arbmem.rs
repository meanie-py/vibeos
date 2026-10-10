use super::prelude::*;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_31DC8C1B8FC3: [u16; 11] = [
    0x0052u16, 0x006F, 0x006F, 0x0074, 0x004D, 0x0065, 0x006D, 0x006F, 0x0072, 0x0079,
    0x0000,
];
pub static RUSTCONV_WSTR_45D9BF3DA53D: [u16; 5] = [0x0052u16, 0x006F, 0x006F, 0x0074, 0x0000];

use ntkrnl_sys::ddk::{HalTranslateBusAddress, INT32, PARBITER_ALLOCATION_STATE, PARBITER_INSTANCE, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PHYSICAL_ADDRESS, PIO_RESOURCE_DESCRIPTOR, PPHYSICAL_ADDRESS, PUINT64, PULONGLONG, RtlAddRange, RtlCmDecodeMemIoResource, RtlIoDecodeMemIoResource, UINT64};

// ---- file-scope preamble (translated) ----
pub const MEMORY_24_BIT_MAX_ADDRESS: i32 = 0xFFFFFF;
unsafe extern "C" fn IopArbMemTranslateAddress(
    SourceType: u8,
    SourceAddress: PHYSICAL_ADDRESS,
    TranslatedAddress: PPHYSICAL_ADDRESS,
    TranslatedType: *mut u8,
) -> NTSTATUS {
    let mut AddressSpace: u32;

    PAGED_CODE!();

    if SourceType == CmResourceTypeMemory || SourceType == CmResourceTypeMemoryLarge {
        AddressSpace = 0;
    } else if SourceType == CmResourceTypePort {
        AddressSpace = 1;
    } else {
        return STATUS_INVALID_PARAMETER;
    }

    if HalTranslateBusAddress(
        Isa,
        0,
        SourceAddress,
        &mut AddressSpace,
        TranslatedAddress,
    ) == 0
    {
        return STATUS_UNSUCCESSFUL;
    }

    if AddressSpace == 1 {
        *TranslatedType = CmResourceTypePort;
    } else if AddressSpace == 0 {
        *TranslatedType = if SourceType == CmResourceTypeMemoryLarge {
            CmResourceTypeMemoryLarge
        } else {
            CmResourceTypeMemory
        };
    } else {
        return STATUS_INVALID_PARAMETER;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IopArbMemUnpackRequirements(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    OutMinimumAddress: PUINT64,
    OutMaximumAddress: PUINT64,
    OutLength: PUINT64,
    OutAlignment: PUINT64,
) -> NTSTATUS {
    core::ptr::write_volatile(OutLength, RtlIoDecodeMemIoResource(
        IoDescriptor,
        OutAlignment,
        OutMinimumAddress,
        OutMaximumAddress,
    ));

    if *OutAlignment == 0 {
        core::ptr::write_volatile(OutAlignment, 1);
    }

    if (*IoDescriptor).Type == CmResourceTypeMemory
        && ((*IoDescriptor).Flags & CM_RESOURCE_MEMORY_24) != 0
        && *OutMaximumAddress > (MEMORY_24_BIT_MAX_ADDRESS as u64)
    {
        core::ptr::write_volatile(OutMaximumAddress, (MEMORY_24_BIT_MAX_ADDRESS as u64));
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IopArbMemPackResource(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: UINT64,
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    (*CmDescriptor).Type = (*IoDescriptor).Type;
    (*CmDescriptor).Flags = (*IoDescriptor).Flags;
    (*CmDescriptor).ShareDisposition = (*IoDescriptor).ShareDisposition;

    (*CmDescriptor).u.Generic.Start.QuadPart = Start as _;
    (*CmDescriptor).u.Generic.Length = (*IoDescriptor).u.Generic.Length;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IopArbMemUnpackResource(
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PUINT64,
    OutLength: PUINT64,
) -> NTSTATUS {
    core::ptr::write_volatile(OutLength, RtlCmDecodeMemIoResource(CmDescriptor, Start));

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn IopArbMemScoreRequirement(IoDescriptor: PIO_RESOURCE_DESCRIPTOR) -> INT32 {
    let mut Length: UINT64;
    let mut Alignment: UINT64 = 0;
    let mut Minimum: UINT64 = 0;
    let mut Maximum: UINT64 = 0;
    let mut AlignedMinimum: UINT64;
    let mut Span: UINT64;
    let mut Placements: UINT64;

    PAGED_CODE!();

    Length = RtlIoDecodeMemIoResource(
        IoDescriptor,
        core::ptr::addr_of_mut!(Alignment),
        core::ptr::addr_of_mut!(Minimum),
        core::ptr::addr_of_mut!(Maximum),
    );

    if Alignment == 0 {
        Alignment = 1;
    }

    AlignedMinimum = Minimum
        .wrapping_add(Alignment)
        .wrapping_sub(1)
        & !(Alignment.wrapping_sub(1));
    if AlignedMinimum < Minimum || AlignedMinimum > Maximum {
        return -1;
    }

    Span = Maximum.wrapping_sub(AlignedMinimum);

    if Length != 0 {
        if Length.wrapping_sub(1) > Span {
            return -1;
        }

        Span = Span.wrapping_sub(Length.wrapping_sub(1));
    }

    Placements = (Span / Alignment).wrapping_add(1);
    return min(Placements, MAXLONG as ULONGLONG) as INT32;
}
#[no_mangle]
pub unsafe extern "system" fn IopArbMemTranslateOrdering(
    OutIoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    let mut SourceType: u8;
    let mut MinimumType: u8;
    let mut MaximumType: u8;

    core::ptr::write_volatile(OutIoDescriptor, *IoDescriptor);

    SourceType = (*IoDescriptor).Type;
    if SourceType != CmResourceTypePort
        && SourceType != CmResourceTypeMemory
        && SourceType != CmResourceTypeMemoryLarge
    {
        return STATUS_SUCCESS;
    }

    MinimumType = SourceType;
    MaximumType = SourceType;

    if !NT_SUCCESS(IopArbMemTranslateAddress(
        SourceType,
        (*IoDescriptor).u.Generic.MinimumAddress,
        core::ptr::addr_of_mut!((*OutIoDescriptor).u.Generic.MinimumAddress),
        &mut MinimumType,
    )) || !NT_SUCCESS(IopArbMemTranslateAddress(
        SourceType,
        (*IoDescriptor).u.Generic.MaximumAddress,
        core::ptr::addr_of_mut!((*OutIoDescriptor).u.Generic.MaximumAddress),
        &mut MaximumType,
    )) {
        (*OutIoDescriptor).Type = CmResourceTypeNull;
    } else {
        (*OutIoDescriptor).Type = MaximumType;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IopArbMemFindSuitableRange(
    Arbiter: PARBITER_INSTANCE,
    ArbState: PARBITER_ALLOCATION_STATE,
) -> u8 {
    PAGED_CODE!();

    if !(*ArbState).Entry.is_null() && ((*(*ArbState).Entry).Flags & ARBITER_FLAG_BOOT_CONFIG) != 0 {
        (*ArbState).RangeAvailableAttributes |= ARBITER_RANGE_BOOT_ALLOCATED;
    }

    ArbiterLibFindSuitableRange(Arbiter, ArbState)
}
#[no_mangle]
pub unsafe extern "system" fn IopArbMemInitialize() -> NTSTATUS {
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    IopRootMemArbiter.Name = RUSTCONV_WSTR_31DC8C1B8FC3.as_ptr() as _;
    IopRootMemArbiter.UnpackRequirement = Some(IopArbMemUnpackRequirements);
    IopRootMemArbiter.PackResource = Some(IopArbMemPackResource);
    IopRootMemArbiter.UnpackResource = Some(IopArbMemUnpackResource);
    IopRootMemArbiter.ScoreRequirement = Some(IopArbMemScoreRequirement);
    IopRootMemArbiter.FindSuitableRange = Some(IopArbMemFindSuitableRange);

    Status = ArbiterLibInitializeInstance(
        core::ptr::addr_of_mut!(IopRootMemArbiter),
        core::ptr::null_mut(),
        CmResourceTypeMemory as CM_RESOURCE_TYPE,
        IopRootMemArbiter.Name,
        RUSTCONV_WSTR_45D9BF3DA53D.as_ptr(),
        Some(IopArbMemTranslateOrdering),
    );
    if !NT_SUCCESS(Status) {
        DPRINT1!("IopArbMemInitialize: Failed with %X\n", Status);
        return Status;
    }

    Status = RtlAddRange(
        IopRootMemArbiter.Allocation,
        0,
        0xFFF,
        0,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        DPRINT1!("IopArbMemInitialize: Reserving page 0 failed with %X\n", Status);
    }

    Status
}
