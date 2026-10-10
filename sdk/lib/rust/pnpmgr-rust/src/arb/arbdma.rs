use super::prelude::*;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_335A166CE937: [u16; 8] = [0x0052u16, 0x006F, 0x006F, 0x0074, 0x0044, 0x004D, 0x0041, 0x0000];
pub static RUSTCONV_WSTR_45D9BF3DA53D: [u16; 5] = [0x0052u16, 0x006F, 0x006F, 0x0074, 0x0000];

use ntkrnl_sys::ddk::{INT32, PARBITER_ALLOCATION_STATE, PARBITER_INSTANCE, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PIO_RESOURCE_DESCRIPTOR, PUINT64, UINT64};

// ---- file-scope preamble (translated) ----
// `#define NDEBUG` has no value and is omitted.
// `extern ARBITER_INSTANCE IopRootDmaArbiter;` is defined elsewhere in the crate and is referenced directly by name.
#[no_mangle]
pub unsafe extern "system" fn IopArbDmaUnpackRequirements(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    OutMinimumAddress: PUINT64,
    OutMaximumAddress: PUINT64,
    OutLength: PUINT64,
    OutAlignment: PUINT64,
) -> NTSTATUS {
    *OutMinimumAddress = ((*IoDescriptor).u.Dma.MinimumChannel) as _;
    *OutMaximumAddress = ((*IoDescriptor).u.Dma.MaximumChannel) as _;
    *OutLength = 1;
    *OutAlignment = 1;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn IopArbDmaPackResource(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: UINT64,
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    (*CmDescriptor).Type = CmResourceTypeDma;
    (*CmDescriptor).ShareDisposition = (*IoDescriptor).ShareDisposition;
    (*CmDescriptor).Flags = (*IoDescriptor).Flags;

    // The C's CM_RESOURCE_DMA_V3 branch is `#if (NTDDI_VERSION >= NTDDI_WIN8)`; this build is
    // NTDDI 0x502, so it is not compiled.

    (*CmDescriptor).u.Dma.Channel = Start as u32;
    (*CmDescriptor).u.Dma.Port = 0;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IopArbDmaUnpackResource(
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PUINT64,
    OutLength: PUINT64,
) -> NTSTATUS {
    *Start = (*CmDescriptor).u.Dma.Channel as UINT64;
    *OutLength = 1;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn IopArbDmaScoreRequirement(IoDescriptor: PIO_RESOURCE_DESCRIPTOR) -> INT32 {
    (*IoDescriptor).u.Dma.MaximumChannel.wrapping_sub((*IoDescriptor).u.Dma.MinimumChannel) as INT32
}
#[no_mangle]
pub unsafe extern "system" fn IopArbDmaOverrideConflict(Arbiter: PARBITER_INSTANCE, ArbState: PARBITER_ALLOCATION_STATE) -> u8 {
    let _ = Arbiter;
    let _ = ArbState;

    return FALSE;
}
#[no_mangle]
pub unsafe extern "system" fn IopArbDmaInitialize() -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_UNSUCCESSFUL;

    PAGED_CODE!();
    IopRootDmaArbiter.Name = RUSTCONV_WSTR_335A166CE937.as_ptr() as PWSTR;
    IopRootDmaArbiter.UnpackRequirement = Some(IopArbDmaUnpackRequirements);
    IopRootDmaArbiter.PackResource = Some(IopArbDmaPackResource);
    IopRootDmaArbiter.UnpackResource = Some(IopArbDmaUnpackResource);
    IopRootDmaArbiter.ScoreRequirement = Some(IopArbDmaScoreRequirement);
    IopRootDmaArbiter.OverrideConflict = Some(IopArbDmaOverrideConflict);

    Status = ArbiterLibInitializeInstance(
        core::ptr::addr_of_mut!(IopRootDmaArbiter),
        core::ptr::null_mut(),
        CmResourceTypeDma as CM_RESOURCE_TYPE,
        IopRootDmaArbiter.Name,
        RUSTCONV_WSTR_45D9BF3DA53D.as_ptr() as PWSTR,
        None,
    );
    if !NT_SUCCESS(Status) {
        DPRINT1!("IopArbDmaInitialize: Failed with %X\n", Status);
    }

    Status
}
