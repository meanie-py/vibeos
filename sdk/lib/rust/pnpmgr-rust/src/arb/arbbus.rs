use super::prelude::*;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_1CD86EED4B3E: [u16; 14] = [
    0x0052u16, 0x006F, 0x006F, 0x0074, 0x0042, 0x0075, 0x0073, 0x004E, 0x0075, 0x006D,
    0x0062, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_45D9BF3DA53D: [u16; 5] = [0x0052u16, 0x006F, 0x006F, 0x0074, 0x0000];

use ntkrnl_sys::ddk::{INT32, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PIO_RESOURCE_DESCRIPTOR, PUINT64, UINT64};

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "system" fn IopArbBusNumberUnpackRequirements(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    OutMinimumAddress: PUINT64,
    OutMaximumAddress: PUINT64,
    OutLength: PUINT64,
    OutAlignment: PUINT64,
) -> NTSTATUS {
    *OutMinimumAddress = ((*IoDescriptor).u.BusNumber.MinBusNumber) as _;
    *OutMaximumAddress = ((*IoDescriptor).u.BusNumber.MaxBusNumber) as _;
    *OutLength = ((*IoDescriptor).u.BusNumber.Length) as _;
    *OutAlignment = 1;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn IopArbBusNumberPackResource(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: UINT64,
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> i32 {
    (*CmDescriptor).Type = CmResourceTypeBusNumber;
    (*CmDescriptor).ShareDisposition = (*IoDescriptor).ShareDisposition;
    (*CmDescriptor).Flags = (*IoDescriptor).Flags;
    (*CmDescriptor).u.BusNumber.Start = Start as u32;
    (*CmDescriptor).u.BusNumber.Length = (*IoDescriptor).u.BusNumber.Length;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn IopArbBusNumberUnpackResource(
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PUINT64,
    OutLength: PUINT64,
) -> NTSTATUS {
    *Start = (*CmDescriptor).u.BusNumber.Start as UINT64;
    *OutLength = (*CmDescriptor).u.BusNumber.Length as UINT64;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IopArbBusNumberScoreRequirement(IoDescriptor: PIO_RESOURCE_DESCRIPTOR) -> INT32 {
    ntkrnl_sys::ASSERT!((*IoDescriptor).u.BusNumber.Length != 0);
    if (*IoDescriptor).u.BusNumber.Length == 0 {
        return -1;
    }
    (((*IoDescriptor).u.BusNumber.MaxBusNumber.wrapping_sub((*IoDescriptor).u.BusNumber.MinBusNumber))
        / (*IoDescriptor).u.BusNumber.Length) as INT32
}
#[no_mangle]
pub unsafe extern "system" fn IopArbBusNumberInitialize() -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_UNSUCCESSFUL;

    PAGED_CODE!();
    let arb = core::ptr::addr_of_mut!(IopRootBusNumberArbiter);
    (*arb).Name = RUSTCONV_WSTR_1CD86EED4B3E.as_ptr();
    (*arb).UnpackRequirement = Some(IopArbBusNumberUnpackRequirements);
    (*arb).PackResource = Some(IopArbBusNumberPackResource);
    (*arb).UnpackResource = Some(IopArbBusNumberUnpackResource);
    (*arb).ScoreRequirement = Some(IopArbBusNumberScoreRequirement);

    Status = ArbiterLibInitializeInstance(
        arb,
        core::ptr::null_mut(),
        CmResourceTypeBusNumber as CM_RESOURCE_TYPE,
        (*arb).Name,
        RUSTCONV_WSTR_45D9BF3DA53D.as_ptr(),
        None,
    );
    if !NT_SUCCESS(Status) {
        DPRINT1!("IopArbBusNumberInitialize: Failed with %X\n", Status);
    }

    Status
}
