use super::prelude::*;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_45D9BF3DA53D: [u16; 5] = [0x0052u16, 0x006F, 0x006F, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_5A8FB783E7EA: [u16; 8] = [0x0052u16, 0x006F, 0x006F, 0x0074, 0x0049, 0x0052, 0x0051, 0x0000];

use ntkrnl_sys::ddk::{HalGetInterruptVector, INT32, KAFFINITY, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PIO_RESOURCE_DESCRIPTOR, PUINT64, UINT64};

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "system" fn IopArbIrqUnpackRequirements(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    OutMinimumAddress: PUINT64,
    OutMaximumAddress: PUINT64,
    OutLength: PUINT64,
    OutAlignment: PUINT64,
) -> NTSTATUS {
    *OutMinimumAddress = (*IoDescriptor).u.Interrupt.MinimumVector as UINT64;
    *OutMaximumAddress = (*IoDescriptor).u.Interrupt.MaximumVector as UINT64;
    *OutLength = 1;
    *OutAlignment = 1;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn IopArbIrqPackResource(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: UINT64,
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    (*CmDescriptor).Type = CmResourceTypeInterrupt as _;
    (*CmDescriptor).ShareDisposition = (*IoDescriptor).ShareDisposition;
    (*CmDescriptor).Flags = (*IoDescriptor).Flags;

    (*CmDescriptor).u.Interrupt.Level = Start as u32;
    (*CmDescriptor).u.Interrupt.Vector = Start as u32;
    (*CmDescriptor).u.Interrupt.Affinity = (1 as KAFFINITY).wrapping_neg();

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn IopArbIrqUnpackResource(
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PUINT64,
    OutLength: PUINT64,
) -> NTSTATUS {
    *Start = (*CmDescriptor).u.Interrupt.Vector as UINT64;
    *OutLength = 1;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn IopArbIrqScoreRequirement(IoDescriptor: PIO_RESOURCE_DESCRIPTOR) -> INT32 {
    ((*IoDescriptor).u.Interrupt.MaximumVector
        .wrapping_sub((*IoDescriptor).u.Interrupt.MinimumVector)
        .wrapping_add((1) as u32)) as INT32
}
#[no_mangle]
pub unsafe extern "system" fn IopArbIrqTranslateOrdering(
    OutIoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    let mut Irql: KIRQL = 0;
    let mut Affinity: KAFFINITY = 0;

    *OutIoDescriptor = *IoDescriptor;

    if (*IoDescriptor).Type != CmResourceTypeInterrupt {
        return STATUS_SUCCESS;
    }

    (*OutIoDescriptor).u.Interrupt.MinimumVector = HalGetInterruptVector(
        Isa,
        0,
        (*IoDescriptor).u.Interrupt.MinimumVector,
        (*IoDescriptor).u.Interrupt.MinimumVector,
        &mut Irql,
        &mut Affinity,
    );

    if Affinity != 0 {
        Affinity = 0;
        (*OutIoDescriptor).u.Interrupt.MaximumVector = HalGetInterruptVector(
            Isa,
            0,
            (*IoDescriptor).u.Interrupt.MaximumVector,
            (*IoDescriptor).u.Interrupt.MaximumVector,
            &mut Irql,
            &mut Affinity,
        );
    }

    if Affinity == 0 {
        *OutIoDescriptor = *IoDescriptor;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IopArbIrqInitialize() -> NTSTATUS {
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    IopRootIrqArbiter.Name = RUSTCONV_WSTR_5A8FB783E7EA.as_ptr();
    IopRootIrqArbiter.UnpackRequirement = Some(IopArbIrqUnpackRequirements);
    IopRootIrqArbiter.PackResource = Some(IopArbIrqPackResource);
    IopRootIrqArbiter.UnpackResource = Some(IopArbIrqUnpackResource);
    IopRootIrqArbiter.ScoreRequirement = Some(IopArbIrqScoreRequirement);

    Status = ArbiterLibInitializeInstance(
        core::ptr::addr_of_mut!(IopRootIrqArbiter),
        core::ptr::null_mut(),
        CmResourceTypeInterrupt as CM_RESOURCE_TYPE,
        IopRootIrqArbiter.Name,
        RUSTCONV_WSTR_45D9BF3DA53D.as_ptr(),
        Some(IopArbIrqTranslateOrdering),
    );
    if !NT_SUCCESS(Status) {
        DPRINT1!(
            b"IopArbIrqInitialize: Failed with %X\n\0".as_ptr() as PCHAR,
            Status,
        );
    }

    Status
}
