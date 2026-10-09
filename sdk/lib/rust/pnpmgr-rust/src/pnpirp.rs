use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;

use ntkrnl_sys::ddk::{
    _IO_STACK_LOCATION, _IO_STATUS_BLOCK, _KEVENT, _DEVICE_NODE,
    PDEVICE_OBJECT, PIRP, PIO_STACK_LOCATION, PCM_RESOURCE_LIST,
    PIO_RESOURCE_REQUIREMENTS_LIST, PPNP_DEVICE_STATE, PDEVICE_NODE,
    IoAllocateIrp, IoGetAttachedDeviceReference, IoGetNextIrpStackLocation,
    IoQueueThreadIrp, KeInitializeEvent, KeWaitForSingleObject, KeAreAllApcsDisabled,
    PsGetCurrentThread, LCID, DEVICE_RELATION_TYPE, DEVICE_TEXT_TYPE,
};

use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IofCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IofCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}

const IRP_MJ_PNP: u8 = 0x1B;
const IRP_MN_FILTER_RESOURCE_REQUIREMENTS: u8 = 0x0D;
const IRP_MN_START_DEVICE: u8 = 0x00;
const IRP_MN_STOP_DEVICE: u8 = 0x04;
const IRP_MN_QUERY_STOP_DEVICE: u8 = 0x05;
const IRP_MN_CANCEL_STOP_DEVICE: u8 = 0x06;
const IRP_MN_QUERY_DEVICE_RELATIONS: u8 = 0x07;
const IRP_MN_QUERY_RESOURCES: u8 = 0x0A;
const IRP_MN_QUERY_RESOURCE_REQUIREMENTS: u8 = 0x0B;
const IRP_MN_QUERY_DEVICE_TEXT: u8 = 0x0C;
const IRP_MN_QUERY_PNP_DEVICE_STATE: u8 = 0x14;

#[no_mangle]
pub unsafe extern "C" fn IopSynchronousCall(
    DeviceObject: PDEVICE_OBJECT,
    IoStackLocation: PIO_STACK_LOCATION,
    Information: *mut PVOID,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let IoStackLocation = opaque_ptr(IoStackLocation);
    let Information = opaque_ptr(Information);
    let mut Irp: PIRP;
    let mut IrpStack: PIO_STACK_LOCATION;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: _KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let TopDeviceObject: PDEVICE_OBJECT;

    TopDeviceObject = IoGetAttachedDeviceReference(DeviceObject);

    Irp = IoAllocateIrp((*TopDeviceObject).StackSize as i8, 0u8);
    if Irp.is_null() {
        ObDereferenceObject(TopDeviceObject as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    IoStatusBlock.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    (*Irp).IoStatus.Information = 0;
    IoStatusBlock.Information = 0;

    if ((*IoStackLocation).MajorFunction == IRP_MJ_PNP) &&
       ((*IoStackLocation).MinorFunction == IRP_MN_FILTER_RESOURCE_REQUIREMENTS)
    {
        (*Irp).IoStatus.Information = (*IoStackLocation).Parameters.FilterResourceRequirements.IoResourceRequirementList as _;
        IoStatusBlock.Information = (*IoStackLocation).Parameters.FilterResourceRequirements.IoResourceRequirementList as _;
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        ntkrnl_sys::ddk::_EVENT_TYPE_SynchronizationEvent as _,
        0u8,
    );

    (*Irp).UserIosb = core::ptr::addr_of_mut!(IoStatusBlock);
    (*Irp).UserEvent = core::ptr::addr_of_mut!(Event);

    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    IoQueueThreadIrp(Irp);

    IrpStack = IoGetNextIrpStackLocation(Irp);
    *IrpStack = *IoStackLocation;

    Status = IofCallDriver(TopDeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            0u32,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    ObDereferenceObject(TopDeviceObject as PVOID);

    *Information = IoStatusBlock.Information as PVOID;
    Status
}

#[no_mangle]
pub unsafe extern "C" fn PiIrpStartDevice(
    DeviceNode: PDEVICE_NODE,
) -> NTSTATUS {
    let mut info: PVOID = core::ptr::null_mut();
    let mut stack: _IO_STACK_LOCATION = core::mem::zeroed();

    stack.MajorFunction = IRP_MJ_PNP;
    stack.MinorFunction = IRP_MN_START_DEVICE;
    stack.Parameters.StartDevice.AllocatedResources = (*DeviceNode).ResourceList;
    stack.Parameters.StartDevice.AllocatedResourcesTranslated = (*DeviceNode).ResourceListTranslated;

    let status = IopSynchronousCall(
        (*DeviceNode).PhysicalDeviceObject,
        core::ptr::addr_of_mut!(stack),
        core::ptr::addr_of_mut!(info),
    );
    (*DeviceNode).CompletionStatus = status;
    status
}

#[no_mangle]
pub unsafe extern "C" fn PiIrpStopDevice(
    DeviceNode: PDEVICE_NODE,
) -> NTSTATUS {
    let mut info: PVOID = core::ptr::null_mut();
    let mut stack: _IO_STACK_LOCATION = core::mem::zeroed();

    stack.MajorFunction = IRP_MJ_PNP;
    stack.MinorFunction = IRP_MN_STOP_DEVICE;

    let status = IopSynchronousCall(
        (*DeviceNode).PhysicalDeviceObject,
        core::ptr::addr_of_mut!(stack),
        core::ptr::addr_of_mut!(info),
    );
    status
}

#[no_mangle]
pub unsafe extern "C" fn PiIrpQueryStopDevice(
    DeviceNode: PDEVICE_NODE,
) -> NTSTATUS {
    let mut info: PVOID = core::ptr::null_mut();
    let mut stack: _IO_STACK_LOCATION = core::mem::zeroed();

    stack.MajorFunction = IRP_MJ_PNP;
    stack.MinorFunction = IRP_MN_QUERY_STOP_DEVICE;

    let status = IopSynchronousCall(
        (*DeviceNode).PhysicalDeviceObject,
        core::ptr::addr_of_mut!(stack),
        core::ptr::addr_of_mut!(info),
    );
    (*DeviceNode).CompletionStatus = status;
    status
}

#[no_mangle]
pub unsafe extern "C" fn PiIrpCancelStopDevice(
    DeviceNode: PDEVICE_NODE,
) -> NTSTATUS {
    let mut info: PVOID = core::ptr::null_mut();
    let mut stack: _IO_STACK_LOCATION = core::mem::zeroed();

    stack.MajorFunction = IRP_MJ_PNP;
    stack.MinorFunction = IRP_MN_CANCEL_STOP_DEVICE;

    let status = IopSynchronousCall(
        (*DeviceNode).PhysicalDeviceObject,
        core::ptr::addr_of_mut!(stack),
        core::ptr::addr_of_mut!(info),
    );
    status
}

#[no_mangle]
pub unsafe extern "C" fn PiIrpQueryDeviceRelations(
    DeviceNode: PDEVICE_NODE,
    Type: DEVICE_RELATION_TYPE,
) -> NTSTATUS {
    let mut stack: _IO_STACK_LOCATION = core::mem::zeroed();

    stack.MajorFunction = IRP_MJ_PNP;
    stack.MinorFunction = IRP_MN_QUERY_DEVICE_RELATIONS;
    stack.Parameters.QueryDeviceRelations.Type = Type;

    let status = IopSynchronousCall(
        (*DeviceNode).PhysicalDeviceObject,
        core::ptr::addr_of_mut!(stack),
        core::ptr::addr_of_mut!((*DeviceNode).OverUsed1.PendingDeviceRelations) as *mut PVOID,
    );
    (*DeviceNode).CompletionStatus = status;
    status
}

#[no_mangle]
pub unsafe extern "C" fn PiIrpQueryResources(
    DeviceNode: PDEVICE_NODE,
    Resources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let Resources = opaque_ptr(Resources);
    let mut longRes: ULONG_PTR = 0;
    let mut stack: _IO_STACK_LOCATION = core::mem::zeroed();

    stack.MajorFunction = IRP_MJ_PNP;
    stack.MinorFunction = IRP_MN_QUERY_RESOURCES;

    let status = IopSynchronousCall(
        (*DeviceNode).PhysicalDeviceObject,
        core::ptr::addr_of_mut!(stack),
        core::ptr::addr_of_mut!(longRes) as *mut PVOID,
    );
    if status >= 0 {
        *Resources = longRes as PVOID as PCM_RESOURCE_LIST;
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn PiIrpQueryResourceRequirements(
    DeviceNode: PDEVICE_NODE,
    Resources: *mut PIO_RESOURCE_REQUIREMENTS_LIST,
) -> NTSTATUS {
    let Resources = opaque_ptr(Resources);
    let mut longRes: ULONG_PTR = 0;
    let mut stack: _IO_STACK_LOCATION = core::mem::zeroed();

    stack.MajorFunction = IRP_MJ_PNP;
    stack.MinorFunction = IRP_MN_QUERY_RESOURCE_REQUIREMENTS;

    let status = IopSynchronousCall(
        (*DeviceNode).PhysicalDeviceObject,
        core::ptr::addr_of_mut!(stack),
        core::ptr::addr_of_mut!(longRes) as *mut PVOID,
    );
    if status >= 0 {
        *Resources = longRes as PVOID as PIO_RESOURCE_REQUIREMENTS_LIST;
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn PiIrpQueryDeviceText(
    DeviceNode: PDEVICE_NODE,
    LocaleId: LCID,
    Type: DEVICE_TEXT_TYPE,
    DeviceText: *mut PWSTR,
) -> NTSTATUS {
    let DeviceText = opaque_ptr(DeviceText);
    let mut longText: ULONG_PTR = 0;
    let mut stack: _IO_STACK_LOCATION = core::mem::zeroed();

    stack.MajorFunction = IRP_MJ_PNP;
    stack.MinorFunction = IRP_MN_QUERY_DEVICE_TEXT;
    stack.Parameters.QueryDeviceText.DeviceTextType = Type;
    stack.Parameters.QueryDeviceText.LocaleId = LocaleId;

    let status = IopSynchronousCall(
        (*DeviceNode).PhysicalDeviceObject,
        core::ptr::addr_of_mut!(stack),
        core::ptr::addr_of_mut!(longText) as *mut PVOID,
    );
    if status >= 0 {
        *DeviceText = longText as PVOID as PWSTR;
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn PiIrpQueryPnPDeviceState(
    DeviceNode: PDEVICE_NODE,
    DeviceState: PPNP_DEVICE_STATE,
) -> NTSTATUS {
    let mut longState: ULONG_PTR = 0;
    let mut stack: _IO_STACK_LOCATION = core::mem::zeroed();

    stack.MajorFunction = IRP_MJ_PNP;
    stack.MinorFunction = IRP_MN_QUERY_PNP_DEVICE_STATE;

    let status = IopSynchronousCall(
        (*DeviceNode).PhysicalDeviceObject,
        core::ptr::addr_of_mut!(stack),
        core::ptr::addr_of_mut!(longState) as *mut PVOID,
    );
    if status >= 0 {
        *DeviceState = longState as ULONG;
    }

    status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Shortcuts for sending different IRP_MJ_PNP requests
//  * COPYRIGHT:   Copyright 2010 Sir Richard <sir_richard@svn.reactos.org>
//  *              Copyright 2020 Victor Perevertkin <victor.perevertkin@reactos.org>
//  */
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
//
// NTSTATUS
// IopSynchronousCall(
//     _In_ PDEVICE_OBJECT DeviceObject,
//     _In_ PIO_STACK_LOCATION IoStackLocation,
//     _Out_ PVOID *Information)
// {
//     PIRP Irp;
//     PIO_STACK_LOCATION IrpStack;
//     IO_STATUS_BLOCK IoStatusBlock;
//     KEVENT Event;
//     NTSTATUS Status;
//     PDEVICE_OBJECT TopDeviceObject;
//     PAGED_CODE();
//
//     /* Call the top of the device stack */
//     TopDeviceObject = IoGetAttachedDeviceReference(DeviceObject);
//
//     /* Allocate an IRP */
//     Irp = IoAllocateIrp(TopDeviceObject->StackSize, FALSE);
//     if (!Irp)
//     {
//         ObDereferenceObject(TopDeviceObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Initialize to failure */
//     Irp->IoStatus.Status = IoStatusBlock.Status = STATUS_NOT_SUPPORTED;
//     Irp->IoStatus.Information = IoStatusBlock.Information = 0;
//
//     /* Special case for IRP_MN_FILTER_RESOURCE_REQUIREMENTS */
//     if ((IoStackLocation->MajorFunction == IRP_MJ_PNP) &&
//         (IoStackLocation->MinorFunction == IRP_MN_FILTER_RESOURCE_REQUIREMENTS))
//     {
//         /* Copy the resource requirements list into the IOSB */
//         Irp->IoStatus.Information =
//         IoStatusBlock.Information = (ULONG_PTR)IoStackLocation->Parameters.FilterResourceRequirements.IoResourceRequirementList;
//     }
//
//     /* Initialize the event */
//     KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//
//     /* Set them up */
//     Irp->UserIosb = &IoStatusBlock;
//     Irp->UserEvent = &Event;
//
//     /* Queue the IRP */
//     Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//     IoQueueThreadIrp(Irp);
//
//     /* Copy-in the stack */
//     IrpStack = IoGetNextIrpStackLocation(Irp);
//     *IrpStack = *IoStackLocation;
//
//     /* Call the driver */
//     ARM64_UART_TRACE("[IRP] IoCallDriver\r\n");
//     Status = IoCallDriver(TopDeviceObject, Irp);
//     ARM64_UART_TRACE("[IRP] IoCallDriver returned\r\n");
//     /* Otherwise we may get stuck here or have IoStatusBlock not populated */
//     ASSERT(!KeAreAllApcsDisabled());
//     if (Status == STATUS_PENDING)
//     {
//         ARM64_UART_TRACE("[IRP] STATUS_PENDING, waiting\r\n");
//         /* Wait for it */
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     /* Remove the reference */
//     ObDereferenceObject(TopDeviceObject);
//
//     /* Return the information */
//     *Information = (PVOID)IoStatusBlock.Information;
//     return Status;
// }
//
// // IRP_MN_START_DEVICE (0x00)
// NTSTATUS
// PiIrpStartDevice(
//     _In_ PDEVICE_NODE DeviceNode)
// {
//     PAGED_CODE();
//
//     ASSERT(DeviceNode);
//     ASSERT(DeviceNode->State == DeviceNodeResourcesAssigned);
//
//     PVOID info;
//     IO_STACK_LOCATION stack = {
//         .MajorFunction = IRP_MJ_PNP,
//         .MinorFunction = IRP_MN_START_DEVICE,
//         .Parameters.StartDevice.AllocatedResources = DeviceNode->ResourceList,
//         .Parameters.StartDevice.AllocatedResourcesTranslated = DeviceNode->ResourceListTranslated
//     };
//
//     // Vista+ does an asynchronous call
//     NTSTATUS status = IopSynchronousCall(DeviceNode->PhysicalDeviceObject, &stack, &info);
//     DeviceNode->CompletionStatus = status;
//     return status;
// }
//
// // IRP_MN_STOP_DEVICE (0x04)
// NTSTATUS
// PiIrpStopDevice(
//     _In_ PDEVICE_NODE DeviceNode)
// {
//     PAGED_CODE();
//
//     ASSERT(DeviceNode);
//     ASSERT(DeviceNode->State == DeviceNodeQueryStopped);
//
//     PVOID info;
//     IO_STACK_LOCATION stack = {
//         .MajorFunction = IRP_MJ_PNP,
//         .MinorFunction = IRP_MN_STOP_DEVICE
//     };
//
//     // Drivers should never fail a IRP_MN_STOP_DEVICE request
//     NTSTATUS status = IopSynchronousCall(DeviceNode->PhysicalDeviceObject, &stack, &info);
//     ASSERT(NT_SUCCESS(status));
//     return status;
// }
//
// // IRP_MN_QUERY_STOP_DEVICE (0x05)
// NTSTATUS
// PiIrpQueryStopDevice(
//     _In_ PDEVICE_NODE DeviceNode)
// {
//     PAGED_CODE();
//
//     ASSERT(DeviceNode);
//     ASSERT(DeviceNode->State == DeviceNodeStarted);
//
//     PVOID info;
//     IO_STACK_LOCATION stack = {
//         .MajorFunction = IRP_MJ_PNP,
//         .MinorFunction = IRP_MN_QUERY_STOP_DEVICE
//     };
//
//     NTSTATUS status = IopSynchronousCall(DeviceNode->PhysicalDeviceObject, &stack, &info);
//     DeviceNode->CompletionStatus = status;
//     return status;
// }
//
// // IRP_MN_CANCEL_STOP_DEVICE (0x06)
// NTSTATUS
// PiIrpCancelStopDevice(
//     _In_ PDEVICE_NODE DeviceNode)
// {
//     PAGED_CODE();
//
//     ASSERT(DeviceNode);
//     ASSERT(DeviceNode->State == DeviceNodeQueryStopped);
//
//     PVOID info;
//     IO_STACK_LOCATION stack = {
//         .MajorFunction = IRP_MJ_PNP,
//         .MinorFunction = IRP_MN_CANCEL_STOP_DEVICE
//     };
//
//     // in fact we don't care which status is returned here
//     NTSTATUS status = IopSynchronousCall(DeviceNode->PhysicalDeviceObject, &stack, &info);
//     ASSERT(NT_SUCCESS(status));
//     return status;
// }
//
// // IRP_MN_QUERY_DEVICE_RELATIONS (0x07)
// NTSTATUS
// PiIrpQueryDeviceRelations(
//     _In_ PDEVICE_NODE DeviceNode,
//     _In_ DEVICE_RELATION_TYPE Type)
// {
//     PAGED_CODE();
//
//     ASSERT(DeviceNode);
//     ASSERT(DeviceNode->State == DeviceNodeStarted);
//
//     IO_STACK_LOCATION stack = {
//         .MajorFunction = IRP_MJ_PNP,
//         .MinorFunction = IRP_MN_QUERY_DEVICE_RELATIONS,
//         .Parameters.QueryDeviceRelations.Type = Type
//     };
//
//     // Vista+ does an asynchronous call
//     NTSTATUS status = IopSynchronousCall(DeviceNode->PhysicalDeviceObject,
//                                          &stack,
//                                          (PVOID)&DeviceNode->OverUsed1.PendingDeviceRelations);
//     DeviceNode->CompletionStatus = status;
//     return status;
// }
//
// // IRP_MN_QUERY_RESOURCES (0x0A)
// NTSTATUS
// PiIrpQueryResources(
//     _In_ PDEVICE_NODE DeviceNode,
//     _Out_ PCM_RESOURCE_LIST *Resources)
// {
//     PAGED_CODE();
//
//     ASSERT(DeviceNode);
//
//     ULONG_PTR longRes;
//     IO_STACK_LOCATION stack = {
//         .MajorFunction = IRP_MJ_PNP,
//         .MinorFunction = IRP_MN_QUERY_RESOURCES
//     };
//
//     NTSTATUS status;
//     status = IopSynchronousCall(DeviceNode->PhysicalDeviceObject, &stack, (PVOID)&longRes);
//     if (NT_SUCCESS(status))
//     {
//         *Resources = (PVOID)longRes;
//     }
//
//     return status;
// }
//
// // IRP_MN_QUERY_RESOURCE_REQUIREMENTS (0x0B)
// NTSTATUS
// PiIrpQueryResourceRequirements(
//     _In_ PDEVICE_NODE DeviceNode,
//     _Out_ PIO_RESOURCE_REQUIREMENTS_LIST *Resources)
// {
//     PAGED_CODE();
//
//     ASSERT(DeviceNode);
//
//     ULONG_PTR longRes;
//     IO_STACK_LOCATION stack = {
//         .MajorFunction = IRP_MJ_PNP,
//         .MinorFunction = IRP_MN_QUERY_RESOURCE_REQUIREMENTS
//     };
//
//     NTSTATUS status;
//     status = IopSynchronousCall(DeviceNode->PhysicalDeviceObject, &stack, (PVOID)&longRes);
//     if (NT_SUCCESS(status))
//     {
//         *Resources = (PVOID)longRes;
//     }
//
//     return status;
// }
//
// // IRP_MN_QUERY_DEVICE_TEXT (0x0C)
// NTSTATUS
// PiIrpQueryDeviceText(
//     _In_ PDEVICE_NODE DeviceNode,
//     _In_ LCID LocaleId,
//     _In_ DEVICE_TEXT_TYPE Type,
//     _Out_ PWSTR *DeviceText)
// {
//     PAGED_CODE();
//
//     ASSERT(DeviceNode);
//     ASSERT(DeviceNode->State == DeviceNodeUninitialized);
//
//     ULONG_PTR longText;
//     IO_STACK_LOCATION stack = {
//         .MajorFunction = IRP_MJ_PNP,
//         .MinorFunction = IRP_MN_QUERY_DEVICE_TEXT,
//         .Parameters.QueryDeviceText.DeviceTextType = Type,
//         .Parameters.QueryDeviceText.LocaleId = LocaleId
//     };
//
//     NTSTATUS status;
//     status = IopSynchronousCall(DeviceNode->PhysicalDeviceObject, &stack, (PVOID)&longText);
//     if (NT_SUCCESS(status))
//     {
//         *DeviceText = (PVOID)longText;
//     }
//
//     return status;
// }
//
// // IRP_MN_QUERY_PNP_DEVICE_STATE (0x14)
// NTSTATUS
// PiIrpQueryPnPDeviceState(
//     _In_ PDEVICE_NODE DeviceNode,
//     _Out_ PPNP_DEVICE_STATE DeviceState)
// {
//     PAGED_CODE();
//
//     ASSERT(DeviceNode);
//     ASSERT(DeviceNode->State == DeviceNodeStartPostWork ||
//            DeviceNode->State == DeviceNodeStarted);
//
//     ULONG_PTR longState;
//     IO_STACK_LOCATION stack = {
//         .MajorFunction = IRP_MJ_PNP,
//         .MinorFunction = IRP_MN_QUERY_PNP_DEVICE_STATE
//     };
//
//     NTSTATUS status;
//     status = IopSynchronousCall(DeviceNode->PhysicalDeviceObject, &stack, (PVOID)&longState);
//     if (NT_SUCCESS(status))
//     {
//         *DeviceState = longState;
//     }
//
//     return status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
