use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    ARBITER_INSTANCE, INT32, PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    PIO_RESOURCE_DESCRIPTOR, PUINT32, PUINT64, UINT64,
};

extern "system" {
    static mut IopRootIrqArbiter: ARBITER_INSTANCE;
    fn ArbInitializeArbiterInstance( arbiter: *mut ARBITER_INSTANCE, device: PVOID, resource_type: i32, name: *const u16, ordering_name: *const u16, translation: PVOID, ) -> NTSTATUS;
}

static ROOTIRQ_NAME: [u16; 8] = [
    b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16,
    b'I' as u16, b'R' as u16, b'Q' as u16, 0,
];

static ROOT_NAME: [u16; 5] = [
    b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16, 0,
];

const CmResourceTypeInterrupt: i32 = 2;

#[no_mangle]
pub unsafe extern "system" fn IopArbIrqUnpackRequirements(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    OutMinimumAddress: PUINT64,
    OutMaximumAddress: PUINT64,
    OutLength: PUINT32,
    OutAlignment: PUINT32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopArbIrqPackResource(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: UINT64,
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopArbIrqUnpackResource(
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PUINT64,
    OutLength: PUINT32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IopArbIrqScoreRequirement(
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
) -> INT32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn IopArbIrqInitialize() -> NTSTATUS {
    IopRootIrqArbiter.Name = ROOTIRQ_NAME.as_ptr();
    IopRootIrqArbiter.UnpackRequirement = core::mem::transmute(IopArbIrqUnpackRequirements as usize);
    IopRootIrqArbiter.PackResource = core::mem::transmute(IopArbIrqPackResource as usize);
    IopRootIrqArbiter.UnpackResource = core::mem::transmute(IopArbIrqUnpackResource as usize);
    IopRootIrqArbiter.ScoreRequirement = core::mem::transmute(IopArbIrqScoreRequirement as usize);

    let Status = ArbInitializeArbiterInstance(
        core::ptr::addr_of_mut!(IopRootIrqArbiter),
        core::ptr::null_mut(),
        CmResourceTypeInterrupt,
        IopRootIrqArbiter.Name as *const u16,
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
//  * PURPOSE:     PnP manager Root IRQ Arbiter
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
// extern ARBITER_INSTANCE IopRootIrqArbiter;
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// IopArbIrqUnpackRequirements(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor,
//     _Out_ PUINT64 OutMinimumAddress,
//     _Out_ PUINT64 OutMaximumAddress,
//     _Out_ PUINT32 OutLength,
//     _Out_ PUINT32 OutAlignment)
// {
//     PAGED_CODE();
//     DPRINT("IopArbIrqUnpackRequirements: IoDescriptor: %p, OutMinimumAddress: %p, OutMaximumAddress: %p, OutLength: %p, OutAlignment: %p\n",
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
// IopArbIrqPackResource(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor,
//     _In_ UINT64 Start,
//     _Out_ PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDescriptor)
// {
//     PAGED_CODE();
//     DPRINT("IopArbIrqPackResource: IoDescriptor: %p, Start: %p, CmDescriptor: %p\n",
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
// IopArbIrqUnpackResource(
//     _In_ PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDescriptor,
//     _Out_ PUINT64 Start,
//     _Out_ PUINT32 OutLength)
// {
//     PAGED_CODE();
//     DPRINT("IopArbIrqUnpackResource: CmDescriptor: %p, Start: %p, OutLength: %p\n",
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
// IopArbIrqScoreRequirement(
//     _In_ PIO_RESOURCE_DESCRIPTOR IoDescriptor)
// {
//     PAGED_CODE();
//     DPRINT("IopArbIrqScoreRequirement: IoDescriptor: %p\n",
//            IoDescriptor);
//
//     UNIMPLEMENTED;
//     return 0;
// }
//
// NTSTATUS
// NTAPI
// IopArbIrqInitialize(VOID)
// {
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//
//     PAGED_CODE();
//     IopRootIrqArbiter.Name = L"RootIRQ";
//     IopRootIrqArbiter.UnpackRequirement = IopArbIrqUnpackRequirements;
//     IopRootIrqArbiter.PackResource = IopArbIrqPackResource;
//     IopRootIrqArbiter.UnpackResource = IopArbIrqUnpackResource;
//     IopRootIrqArbiter.ScoreRequirement = IopArbIrqScoreRequirement;
//
//     Status = ArbInitializeArbiterInstance(&IopRootIrqArbiter,
//                                           NULL,
//                                           CmResourceTypeInterrupt,
//                                           IopRootIrqArbiter.Name,
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
