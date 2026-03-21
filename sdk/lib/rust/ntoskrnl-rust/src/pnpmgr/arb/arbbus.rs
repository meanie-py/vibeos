use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    ARBITER_INSTANCE, INT32, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PIO_RESOURCE_DESCRIPTOR,
    PUINT32, PUINT64, UINT64,
};

extern "system" {
    static mut IopRootBusNumberArbiter: ARBITER_INSTANCE;
    fn ArbInitializeArbiterInstance( Arbiter: *mut ARBITER_INSTANCE, BusDeviceObject: PVOID, ResourceType: i32, Name: *const u16, OrderingName: *const u16, TranslateOrdering: PVOID, ) -> NTSTATUS;
}

const CmResourceTypeBusNumber: i32 = 6;

#[no_mangle]
pub unsafe extern "system" fn IopArbBusNumberUnpackRequirements(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    OutMinimumAddress: PUINT64,
    OutMaximumAddress: PUINT64,
    OutLength: PUINT32,
    OutAlignment: PUINT32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopArbBusNumberPackResource(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: UINT64,
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopArbBusNumberUnpackResource(
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PUINT64,
    OutLength: PUINT32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopArbBusNumberScoreRequirement(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
) -> INT32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn IopArbBusNumberInitialize() -> NTSTATUS {
    static ROOT_BUS_NUMBER: [u16; 14] = [
        b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16,
        b'B' as u16, b'u' as u16, b's' as u16, b'N' as u16,
        b'u' as u16, b'm' as u16, b'b' as u16, b'e' as u16,
        b'r' as u16, 0,
    ];
    static ROOT: [u16; 5] = [
        b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16, 0,
    ];

    IopRootBusNumberArbiter.Name = ROOT_BUS_NUMBER.as_ptr();
    IopRootBusNumberArbiter.UnpackRequirement = core::mem::transmute(IopArbBusNumberUnpackRequirements as usize);
    IopRootBusNumberArbiter.PackResource = core::mem::transmute(IopArbBusNumberPackResource as usize);
    IopRootBusNumberArbiter.UnpackResource = core::mem::transmute(IopArbBusNumberUnpackResource as usize);
    IopRootBusNumberArbiter.ScoreRequirement = core::mem::transmute(IopArbBusNumberScoreRequirement as usize);

    let Status = ArbInitializeArbiterInstance(
        core::ptr::addr_of_mut!(IopRootBusNumberArbiter),
        core::ptr::null_mut(),
        CmResourceTypeBusNumber,
        IopRootBusNumberArbiter.Name,
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
//  * PURPOSE:     PnP manager Root Bus Arbiter
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
// extern ARBITER_INSTANCE IopRootBusNumberArbiter;
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// IopArbBusNumberUnpackRequirements(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor,
//     _Out_ PUINT64 OutMinimumAddress,
//     _Out_ PUINT64 OutMaximumAddress,
//     _Out_ PUINT32 OutLength,
//     _Out_ PUINT32 OutAlignment)
// {
//     PAGED_CODE();
//     DPRINT("IopArbBusNumberUnpackRequirements: IoDescriptor: %p, OutMinimumAddress: %p, OutMaximumAddress: %p, OutLength: %p, OutAlignment: %p\n",
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
// IopArbBusNumberPackResource(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor,
//     _In_ UINT64 Start,
//     _Out_ PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDescriptor)
// {
//     PAGED_CODE();
//     DPRINT("IopArbBusNumberPackResource: IoDescriptor: %p, Start: %I64x, CmDescriptor: %p\n",
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
// IopArbBusNumberUnpackResource(
//     _In_ PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDescriptor,
//     _Out_ PUINT64 Start,
//     _Out_ PUINT32 OutLength)
// {
//     PAGED_CODE();
//     DPRINT("IopArbBusNumberUnpackResource: CmDescriptor: %p, Start: %p, OutLength: %p\n",
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
// IopArbBusNumberScoreRequirement(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor)
// {
//     PAGED_CODE();
//     DPRINT("IopArbBusNumberScoreRequirement: IoDescriptor: %p\n",
//            IoDescriptor);
//
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /**
//  * @brief Initialize the RootBusArbiter
//  *
//  * Initializes rootbus arbiter against IopRootBusNumberArbiter for use in PCI
//  * every "bus" gets to check against this.
//  * @return NTSTATUS
//  * @retval STATUS_SUCCESS
//  * @retval STATUS_UNSUCCESSFUL
//  * @retval STATUS_INSUFFICIENT_RESOURCES
//  */
// NTSTATUS
// NTAPI
// IopArbBusNumberInitialize(VOID)
// {
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//
//     PAGED_CODE();
//     IopRootBusNumberArbiter.Name = L"RootBusNumber";
//     IopRootBusNumberArbiter.UnpackRequirement = IopArbBusNumberUnpackRequirements;
//     IopRootBusNumberArbiter.PackResource = IopArbBusNumberPackResource;
//     IopRootBusNumberArbiter.UnpackResource = IopArbBusNumberUnpackResource;
//     IopRootBusNumberArbiter.ScoreRequirement = IopArbBusNumberScoreRequirement;
//
//     Status = ArbInitializeArbiterInstance(&IopRootBusNumberArbiter,
//                                           NULL,
//                                           CmResourceTypeBusNumber,
//                                           IopRootBusNumberArbiter.Name,
//                                           L"Root",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopArbBusNumberInitialize: Failed with %X", Status);
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
