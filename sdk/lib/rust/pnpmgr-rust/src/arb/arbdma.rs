use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    ARBITER_INSTANCE, INT32, PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    PIO_RESOURCE_DESCRIPTOR, PUINT32, PUINT64, UINT64,
};

const CmResourceTypeDma: i32 = 4; // header value; the hand copy said 5

extern "system" {
    static mut IopRootDmaArbiter: ARBITER_INSTANCE;
    fn ArbInitializeArbiterInstance( arbiter: *mut ARBITER_INSTANCE, bus_device: PVOID, resource_type: i32, name: *const u16, order_name: *const u16, translate_ordering: PVOID, ) -> NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn IopArbDmaUnpackRequirements(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    OutMinimumAddress: PUINT64,
    OutMaximumAddress: PUINT64,
    OutLength: PUINT32,
    OutAlignment: PUINT32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopArbDmaPackResource(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: UINT64,
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopArbDmaUnpackResource(
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PUINT64,
    OutLength: PUINT32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopArbDmaScoreRequirement(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
) -> INT32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn IopArbDmaInitialize() -> NTSTATUS {
    static ROOT_DMA: [u16; 8] = [
        b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16,
        b'D' as u16, b'm' as u16, b'a' as u16, 0,
    ];
    static ROOT: [u16; 5] = [
        b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16, 0,
    ];

    IopRootDmaArbiter.Name = ROOT_DMA.as_ptr();
    IopRootDmaArbiter.UnpackRequirement = core::mem::transmute(IopArbDmaUnpackRequirements as usize);
    IopRootDmaArbiter.PackResource = core::mem::transmute(IopArbDmaPackResource as usize);
    IopRootDmaArbiter.UnpackResource = core::mem::transmute(IopArbDmaUnpackResource as usize);
    IopRootDmaArbiter.ScoreRequirement = core::mem::transmute(IopArbDmaScoreRequirement as usize);

    let Status = ArbInitializeArbiterInstance(
        core::ptr::addr_of_mut!(IopRootDmaArbiter),
        core::ptr::null_mut(),
        CmResourceTypeDma,
        ROOT_DMA.as_ptr(),
        ROOT.as_ptr(),
        core::ptr::null_mut(),
    );

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     PnP manager Root DMA Arbiter
//  * COPYRIGHT:   Copyright 2025 Justin Miller <justin.miller@reactos.org>
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// extern ARBITER_INSTANCE IopRootDmaArbiter;
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// IopArbDmaUnpackRequirements(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor,
//     _Out_ PUINT64 OutMinimumAddress,
//     _Out_ PUINT64 OutMaximumAddress,
//     _Out_ PUINT32 OutLength,
//     _Out_ PUINT32 OutAlignment)
// {
//     PAGED_CODE();
//     DPRINT("IopArbDmaUnpackRequirements: IoDescriptor: %p, OutMinimumAddress: %p, OutMaximumAddress: %p, OutLength: %p, OutAlignment: %p\n",
//            IoDescriptor,
//            OutMinimumAddress,
//            OutMaximumAddress,
//            OutLength,
//            OutAlignment);
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IopArbDmaPackResource(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor,
//     _In_ UINT64 Start,
//     _Out_ PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDescriptor)
// {
//     PAGED_CODE();
//     DPRINT("IopArbDmaPackResource: IoDescriptor: %p, Start: %p, CmDescriptor: %p\n",
//            IoDescriptor,
//            Start,
//            CmDescriptor);
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IopArbDmaUnpackResource(
//     _In_ PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDescriptor,
//     _Out_ PUINT64 Start,
//     _Out_ PUINT32 OutLength)
// {
//     PAGED_CODE();
//     DPRINT("IopArbDmaUnpackResource: CmDescriptor: %p, Start: %p, OutLength: %p\n",
//            CmDescriptor,
//            Start,
//            OutLength);
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// INT32
// NTAPI
// IopArbDmaScoreRequirement(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor)
// {
//     PAGED_CODE();
//     DPRINT("IopArbDmaScoreRequirement: IoDescriptor: %p\n",
//            IoDescriptor);
//
//     UNIMPLEMENTED;
//     return 0;
// }
//
// NTSTATUS
// NTAPI
// IopArbDmaInitialize(VOID)
// {
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//
//     PAGED_CODE();
//     IopRootDmaArbiter.Name = L"RootDma";
//     IopRootDmaArbiter.UnpackRequirement = IopArbDmaUnpackRequirements;
//     IopRootDmaArbiter.PackResource = IopArbDmaPackResource;
//     IopRootDmaArbiter.UnpackResource = IopArbDmaUnpackResource;
//     IopRootDmaArbiter.ScoreRequirement = IopArbDmaScoreRequirement;
//
//     Status = ArbInitializeArbiterInstance(&IopRootDmaArbiter,
//                                           NULL,
//                                           CmResourceTypeBusNumber,
//                                           IopRootDmaArbiter.Name,
//                                           L"Root",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopArbDmaInitialize: Failed with %X", Status);
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
