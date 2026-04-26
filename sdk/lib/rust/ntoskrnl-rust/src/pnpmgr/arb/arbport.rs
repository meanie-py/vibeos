use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    ARBITER_INSTANCE, INT32, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PIO_RESOURCE_DESCRIPTOR,
    PUINT32, PUINT64, UINT64,
};

extern "system" {
    static mut IopRootPortArbiter: ARBITER_INSTANCE;
    fn ArbInitializeArbiterInstance( Arbiter: *mut ARBITER_INSTANCE, BusDeviceObject: PVOID, ResourceType: i32, ArbiterName: *const u16, OrderingName: *const u16, TranslateOrdering: PVOID, ) -> NTSTATUS;
}

const CmResourceTypePort: i32 = 1;

#[no_mangle]
pub unsafe extern "system" fn IopPortMemUnpackRequirements(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    OutMinimumAddress: PUINT64,
    OutMaximumAddress: PUINT64,
    OutLength: PUINT32,
    OutAlignment: PUINT32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopPortMemPackResource(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: UINT64,
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopPortMemUnpackResource(
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PUINT64,
    OutLength: PUINT32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopPortMemScoreRequirement(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
) -> INT32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn IopArbPortInitialize() -> NTSTATUS {
    static ROOT_PORT_NAME: [u16; 9] = [
        b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16,
        b'P' as u16, b'o' as u16, b'r' as u16, b't' as u16,
        0,
    ];
    static ROOT_NAME: [u16; 5] = [
        b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16,
        0,
    ];

    IopRootPortArbiter.Name = ROOT_PORT_NAME.as_ptr();
    IopRootPortArbiter.UnpackRequirement = core::mem::transmute(IopPortMemUnpackRequirements as usize);
    IopRootPortArbiter.PackResource = core::mem::transmute(IopPortMemPackResource as usize);
    IopRootPortArbiter.UnpackResource = core::mem::transmute(IopPortMemUnpackResource as usize);
    IopRootPortArbiter.ScoreRequirement = core::mem::transmute(IopPortMemScoreRequirement as usize);

    let Status = ArbInitializeArbiterInstance(
        core::ptr::addr_of_mut!(IopRootPortArbiter),
        core::ptr::null_mut(),
        CmResourceTypePort,
        IopRootPortArbiter.Name,
        ROOT_NAME.as_ptr(),
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
//  * PURPOSE:     PnP manager Root Port Arbiter
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
// extern ARBITER_INSTANCE IopRootPortArbiter;
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// IopPortMemUnpackRequirements(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor,
//     _Out_ PUINT64 OutMinimumAddress,
//     _Out_ PUINT64 OutMaximumAddress,
//     _Out_ PUINT32 OutLength,
//     _Out_ PUINT32 OutAlignment)
// {
//     PAGED_CODE();
//     DPRINT("IopPortMemUnpackRequirements: IoDescriptor: %p, OutMinimumAddress: %p, OutMaximumAddress: %p, OutLength: %p, OutAlignment: %p\n",
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
// IopPortMemPackResource(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor,
//     _In_ UINT64 Start,
//     _Out_ PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDescriptor)
// {
//     PAGED_CODE();
//     DPRINT("IopPortMemPackResource: IoDescriptor: %p, Start: %p, CmDescriptor: %p\n",
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
// IopPortMemUnpackResource(
//     _In_ PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDescriptor,
//     _Out_ PUINT64 Start,
//     _Out_ PUINT32 OutLength)
// {
//     PAGED_CODE();
//     DPRINT("IopPortMemUnpackResource: CmDescriptor: %p, Start: %p, OutLength: %p\n",
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
// IopPortMemScoreRequirement(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor)
// {
//     PAGED_CODE();
//     DPRINT("IopPortMemScoreRequirement: IoDescriptor: %p\n",
//            IoDescriptor);
//
//     UNIMPLEMENTED;
//     return 0;
// }
//
// NTSTATUS
// NTAPI
// IopArbPortInitialize(VOID)
// {
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//
//     PAGED_CODE();
//     IopRootPortArbiter.Name = L"RootPort";
//     IopRootPortArbiter.UnpackRequirement = IopPortMemUnpackRequirements;
//     IopRootPortArbiter.PackResource = IopPortMemPackResource;
//     IopRootPortArbiter.UnpackResource = IopPortMemUnpackResource;
//     IopRootPortArbiter.ScoreRequirement = IopPortMemScoreRequirement;
//
//     Status = ArbInitializeArbiterInstance(&IopRootPortArbiter,
//                                           NULL,
//                                           CmResourceTypePort,
//                                           IopRootPortArbiter.Name,
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
