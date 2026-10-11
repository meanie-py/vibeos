use super::prelude::*;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_4337FEB49C0C: [u16; 9] = [0x0052u16, 0x006F, 0x006F, 0x0074, 0x0050, 0x006F, 0x0072, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_45D9BF3DA53D: [u16; 5] = [0x0052u16, 0x006F, 0x006F, 0x0074, 0x0000];

use ntkrnl_sys::ddk::{HalTranslateBusAddress, INT32, PARBITER_ALLOCATION_STATE, PARBITER_ALTERNATIVE, PARBITER_INSTANCE, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PHYSICAL_ADDRESS, PIO_RESOURCE_DESCRIPTOR, PPHYSICAL_ADDRESS, PUINT64, PULONGLONG, RtlAddRange, RtlCmDecodeMemIoResource, RtlDeleteRange, RtlIoDecodeMemIoResource, UINT64};

// ---- file-scope preamble (translated) ----
pub const PORT_ALIAS_STRIDE_10_BIT: i32 = 0x400;

pub const PORT_ALIAS_STRIDE_12_BIT: i32 = 0x1000;

pub const PORT_MAX_ADDRESS: i32 = 0xFFFF;
unsafe extern "C" fn IopArbPortTranslateAddress(
    SourceType: UCHAR,
    SourceAddress: PHYSICAL_ADDRESS,
    TranslatedAddress: PPHYSICAL_ADDRESS,
    TranslatedType: PUCHAR,
) -> NTSTATUS {
    let mut AddressSpace: ULONG;

    PAGED_CODE!();

    if SourceType == CmResourceTypePort {
        AddressSpace = 1;
    } else if SourceType == CmResourceTypeMemory || SourceType == CmResourceTypeMemoryLarge {
        AddressSpace = 0;
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
unsafe extern "C" fn IopArbPortGetNextAlias(
    DescriptorFlags: u16,
    LastAlias: UINT64,
    NextAlias: PUINT64,
) -> u8 {
    let mut Next: UINT64;

    if (DescriptorFlags & CM_RESOURCE_PORT_10_BIT_DECODE) != 0 {
        Next = LastAlias + (PORT_ALIAS_STRIDE_10_BIT as u64);
    } else if (DescriptorFlags & CM_RESOURCE_PORT_12_BIT_DECODE) != 0 {
        Next = LastAlias + (PORT_ALIAS_STRIDE_12_BIT as u64);
    } else {
        return FALSE;
    }

    if Next > (PORT_MAX_ADDRESS as u64) {
        return FALSE;
    }

    *NextAlias = Next;
    TRUE
}
#[no_mangle]
pub unsafe extern "system" fn IopArbPortUnpackRequirements(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    OutMinimumAddress: PUINT64,
    OutMaximumAddress: PUINT64,
    OutLength: PUINT64,
    OutAlignment: PUINT64,
) -> NTSTATUS {
    PAGED_CODE!();

    core::ptr::write_volatile(OutLength, RtlIoDecodeMemIoResource(
        IoDescriptor,
        OutAlignment,
        OutMinimumAddress,
        OutMaximumAddress,
    ));

    if *OutAlignment == 0 {
        core::ptr::write_volatile(OutAlignment, 1);
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn IopArbPortPackResource(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: UINT64,
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    (*CmDescriptor).Type = (*IoDescriptor).Type;
    (*CmDescriptor).Flags = (*IoDescriptor).Flags;
    (*CmDescriptor).ShareDisposition = (*IoDescriptor).ShareDisposition;

    (*CmDescriptor).u.Generic.Start.QuadPart = Start as _;
    (*CmDescriptor).u.Generic.Length = (*IoDescriptor).u.Generic.Length;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn IopArbPortUnpackResource(
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PUINT64,
    OutLength: PUINT64,
) -> NTSTATUS {
    core::ptr::write_volatile(OutLength, RtlCmDecodeMemIoResource(CmDescriptor, Start));

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IopArbPortScoreRequirement(IoDescriptor: PIO_RESOURCE_DESCRIPTOR) -> INT32 {
    let mut Length: UINT64;
    let mut Alignment: UINT64 = 0;
    let mut Minimum: UINT64 = 0;
    let mut Maximum: UINT64 = 0;
    let mut AlignedMinimum: UINT64;
    let mut Span: UINT64;
    let mut Placements: UINT64;

    Length = RtlIoDecodeMemIoResource(IoDescriptor, &mut Alignment as PUINT64, &mut Minimum as PUINT64, &mut Maximum as PUINT64);

    if Alignment == 0 {
        Alignment = 1;
    }

    AlignedMinimum = Minimum.wrapping_add(Alignment).wrapping_sub(1) & !(Alignment.wrapping_sub(1));
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
pub unsafe extern "system" fn IopArbPortTranslateOrdering(
    OutIoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    let mut SourceType: UCHAR;
    let mut MinimumType: UCHAR;
    let mut MaximumType: UCHAR;

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

    if !NT_SUCCESS(IopArbPortTranslateAddress(
        SourceType,
        (*IoDescriptor).u.Generic.MinimumAddress,
        core::ptr::addr_of_mut!((*OutIoDescriptor).u.Generic.MinimumAddress),
        core::ptr::addr_of_mut!(MinimumType),
    )) || !NT_SUCCESS(IopArbPortTranslateAddress(
        SourceType,
        (*IoDescriptor).u.Generic.MaximumAddress,
        core::ptr::addr_of_mut!((*OutIoDescriptor).u.Generic.MaximumAddress),
        core::ptr::addr_of_mut!(MaximumType),
    ))
    {
        (*OutIoDescriptor).Type = CmResourceTypeNull;
    } else {
        (*OutIoDescriptor).Type = MaximumType;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IopArbPortFindSuitableRange(
    Arbiter: PARBITER_INSTANCE,
    ArbState: PARBITER_ALLOCATION_STATE,
) -> BOOLEAN {
    PAGED_CODE!();

    if (*ArbState).Entry != core::ptr::null_mut()
        && ((*(*ArbState).Entry).Flags & ARBITER_FLAG_BOOT_CONFIG) != 0
    {
        (*ArbState).RangeAvailableAttributes |= ARBITER_RANGE_BOOT_ALLOCATED;
    }

    ArbiterLibFindSuitableRange(Arbiter, ArbState)
}
#[no_mangle]
pub unsafe extern "system" fn IopArbPortAddAllocation(
    Arbiter: PARBITER_INSTANCE,
    ArbState: PARBITER_ALLOCATION_STATE,
) {
    let Alternative: PARBITER_ALTERNATIVE = (*ArbState).CurrentAlternative;
    let Owner: PVOID = if (*ArbState).Entry.is_null() {
        core::ptr::null_mut()
    } else {
        (*(*ArbState).Entry).PhysicalDeviceObject as PVOID
    };
    let mut Flags: ULONG = RTL_RANGE_LIST_ADD_IF_CONFLICT;
    let mut Alias: UINT64;
    let Span: UINT64;

    PAGED_CODE!();

    if !Alternative.is_null() && ((*Alternative).Flags & ARBITER_ALTERNATIVE_FLAG_SHARED) != 0 {
        Flags |= RTL_RANGE_LIST_ADD_SHARED;
    }

    let _ = RtlAddRange(
        (*Arbiter).PossibleAllocation,
        (*ArbState).Start,
        (*ArbState).End,
        (*ArbState).RangeAttributes,
        Flags,
        core::ptr::null_mut(),
        Owner,
    );

    if Alternative.is_null() {
        return;
    }

    /* Alias the width actually granted, not the width asked for. */
    Span = (*ArbState).End - (*ArbState).Start;

    Alias = (*ArbState).Start;
    while IopArbPortGetNextAlias(
        (*(*Alternative).Descriptor).Flags,
        Alias,
        &mut Alias as PUINT64,
    ) != 0
    {
        let _ = RtlAddRange(
            (*Arbiter).PossibleAllocation,
            Alias,
            Alias + Span,
            (*ArbState).RangeAttributes | ARBITER_RANGE_PORT_ALIAS,
            Flags,
            core::ptr::null_mut(),
            Owner,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn IopArbPortBacktrackAllocation(
    Arbiter: PARBITER_INSTANCE,
    ArbState: PARBITER_ALLOCATION_STATE,
) {
    let Alternative: PARBITER_ALTERNATIVE = (*ArbState).CurrentAlternative;
    let Owner: PVOID = if !(*ArbState).Entry.is_null() {
        (*(*ArbState).Entry).PhysicalDeviceObject as PVOID
    } else {
        core::ptr::null_mut()
    };
    let mut Alias: UINT64;
    let mut Span: UINT64;

    PAGED_CODE!();

    if !Alternative.is_null() {
        Span = (*ArbState).End - (*ArbState).Start;

        Alias = (*ArbState).Start;
        while IopArbPortGetNextAlias(
            (*(*Alternative).Descriptor).Flags,
            Alias,
            core::ptr::addr_of_mut!(Alias),
        ) != 0
        {
            RtlDeleteRange(
                (*Arbiter).PossibleAllocation,
                Alias,
                Alias + Span,
                Owner,
            );
        }
    }

    RtlDeleteRange(
        (*Arbiter).PossibleAllocation,
        (*ArbState).Start,
        (*ArbState).End,
        Owner,
    );
}
#[no_mangle]
pub unsafe extern "system" fn IopArbPortInitialize() -> NTSTATUS {
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    IopRootPortArbiter.Name = RUSTCONV_WSTR_4337FEB49C0C.as_ptr();
    IopRootPortArbiter.UnpackRequirement = Some(IopArbPortUnpackRequirements);
    IopRootPortArbiter.PackResource = Some(IopArbPortPackResource);
    IopRootPortArbiter.UnpackResource = Some(IopArbPortUnpackResource);
    IopRootPortArbiter.ScoreRequirement = Some(IopArbPortScoreRequirement);

    /* Port-specific placement: boot-config leniency and ISA decode aliasing. */
    IopRootPortArbiter.FindSuitableRange = Some(IopArbPortFindSuitableRange);
    IopRootPortArbiter.AddAllocation = Some(IopArbPortAddAllocation);
    IopRootPortArbiter.BacktrackAllocation = Some(IopArbPortBacktrackAllocation);

    Status = ArbiterLibInitializeInstance(
        core::ptr::addr_of_mut!(IopRootPortArbiter),
        core::ptr::null_mut(),
        CmResourceTypePort as CM_RESOURCE_TYPE,
        IopRootPortArbiter.Name,
        RUSTCONV_WSTR_45D9BF3DA53D.as_ptr(),
        Some(IopArbPortTranslateOrdering),
    );
    if !NT_SUCCESS(Status) {
        DPRINT1!("IopArbPortInitialize: Failed with %X\n", Status);
    }

    Status
}
