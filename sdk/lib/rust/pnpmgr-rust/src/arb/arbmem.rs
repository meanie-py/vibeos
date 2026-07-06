use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    ARBITER_INSTANCE, INT32, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PIO_RESOURCE_DESCRIPTOR,
    PUINT32, PUINT64, UINT64,
};

const CmResourceTypeMemory: i32 = 3;

extern "system" {
    static mut IopRootMemArbiter: ARBITER_INSTANCE;
    fn ArbInitializeArbiterInstance( Arbiter: *mut ARBITER_INSTANCE, BusDeviceObject: PVOID, ResourceType: i32, Name: *const u16, OrderingName: *const u16, TranslateOrdering: PVOID, ) -> NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn IopArbMemUnpackRequirements(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    OutMinimumAddress: PUINT64,
    OutMaximumAddress: PUINT64,
    OutLength: PUINT32,
    OutAlignment: PUINT32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopArbMemPackResource(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: UINT64,
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopArbMemUnpackResource(
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PUINT64,
    OutLength: PUINT32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopArbMemScoreRequirement(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
) -> INT32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn IopArbMemInitialize() -> NTSTATUS {
    static ROOT_MEMORY: [u16; 11] = [
        b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16, b'M' as u16, b'e' as u16,
        b'm' as u16, b'o' as u16, b'r' as u16, b'y' as u16, 0,
    ];
    static ROOT: [u16; 5] = [b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16, 0];

    IopRootMemArbiter.Name = ROOT_MEMORY.as_ptr();
    IopRootMemArbiter.UnpackRequirement = core::mem::transmute(IopArbMemUnpackRequirements as usize);
    IopRootMemArbiter.PackResource = core::mem::transmute(IopArbMemPackResource as usize);
    IopRootMemArbiter.UnpackResource = core::mem::transmute(IopArbMemUnpackResource as usize);
    IopRootMemArbiter.ScoreRequirement = core::mem::transmute(IopArbMemScoreRequirement as usize);

    let Status = ArbInitializeArbiterInstance(
        core::ptr::addr_of_mut!(IopRootMemArbiter),
        core::ptr::null_mut(),
        CmResourceTypeMemory,
        ROOT_MEMORY.as_ptr(),
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
//  * PURPOSE:     PnP manager Root Memory Arbiter
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
// extern ARBITER_INSTANCE IopRootMemArbiter;
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// IopArbMemUnpackRequirements(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor,
//     _Out_ PUINT64 OutMinimumAddress,
//     _Out_ PUINT64 OutMaximumAddress,
//     _Out_ PUINT32 OutLength,
//     _Out_ PUINT32 OutAlignment)
// {
//     PAGED_CODE();
//     DPRINT("IopArbMemUnpackRequirements: IoDescriptor: %p, OutMinimumAddress: %p, OutMaximumAddress: %p, OutLength: %p, OutAlignment: %p\n",
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
// IopArbMemPackResource(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor,
//     _In_ UINT64 Start,
//     _Out_ PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDescriptor)
// {
//     PAGED_CODE();
//     DPRINT("IopArbMemPackResource: IoDescriptor: %p, Start: %p, CmDescriptor: %p\n",
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
// IopArbMemUnpackResource(
//     _In_ PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDescriptor,
//     _Out_ PUINT64 Start,
//     _Out_ PUINT32 OutLength)
// {
//     PAGED_CODE();
//     DPRINT("IopArbMemUnpackResource: CmDescriptor: %p, Start: %p, OutLength: %p\n",
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
// IopArbMemScoreRequirement(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor)
// {
//     PAGED_CODE();
//     DPRINT("IopArbMemScoreRequirement: IoDescriptor: %p\n",
//            IoDescriptor);
//
//     UNIMPLEMENTED;
//     return 0;
// }
//
// NTSTATUS
// NTAPI
// IopArbMemInitialize(VOID)
// {
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//
//     PAGED_CODE();
//     IopRootMemArbiter.Name = L"RootMemory";
//     IopRootMemArbiter.UnpackRequirement = IopArbMemUnpackRequirements;
//     IopRootMemArbiter.PackResource = IopArbMemPackResource;
//     IopRootMemArbiter.UnpackResource = IopArbMemUnpackResource;
//     IopRootMemArbiter.ScoreRequirement = IopArbMemScoreRequirement;
//
//     Status = ArbInitializeArbiterInstance(&IopRootMemArbiter,
//                                           NULL,
//                                           CmResourceTypeBusNumber,
//                                           IopRootMemArbiter.Name,
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
