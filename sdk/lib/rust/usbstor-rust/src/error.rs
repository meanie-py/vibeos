use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{ExQueueWorkItem, IoQueueWorkItem, PDEVICE_OBJECT, UINT32, PIRP, PWORKER_THREAD_ROUTINE, PIO_WORKITEM_ROUTINE};
use ntkrnl_sys::ddk::_WORK_QUEUE_TYPE_CriticalWorkQueue as CriticalWorkQueue;
use ntkrnl_sys::ddk::_WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue;
use ntkrnl_sys::inline::ex::ExInitializeWorkItem;
use ntkrnl_sys::ke::PKSPIN_LOCK;
use core::ffi::c_void;

extern "C" {
    fn AllocateItem(PoolType: POOL_TYPE, ItemSize: ULONG) -> PVOID;
    fn FreeItem(Item: PVOID);
    fn USBSTOR_SyncUrbRequest(DeviceObject: PDEVICE_OBJECT, Urb: PURB) -> NTSTATUS;
    fn UsbBuildGetStatusRequest( Urb: PURB, Op: USHORT, Index: USHORT, TransferBuffer: PVOID, TransferBufferMDL: PVOID, Link: PVOID, );
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
    pub fn USBSTOR_SendCSWRequest(FDODeviceExtension: PFDO_DEVICE_EXTENSION, Irp: PIRP) -> NTSTATUS;
    fn USBSTOR_ResetDevice( DeviceObject: PDEVICE_OBJECT, FDODeviceExtension: PFDO_DEVICE_EXTENSION, ) -> NTSTATUS;
    fn USBSTOR_QueueNextRequest(FdoDevice: PDEVICE_OBJECT);
}
extern "system" {
    // NTAPI (stdcall on i386) per ntoskrnl.spec
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG

#[no_mangle]
pub unsafe extern "C" fn USBSTOR_GetEndpointStatus(
    mut DeviceObject: PDEVICE_OBJECT,
    mut bEndpointAddress: UCHAR,
    mut Value: PUSHORT,
) -> NTSTATUS {

    const URB_FUNCTION_GET_STATUS_FROM_ENDPOINT: USHORT = 0x15;

    let mut Urb: PURB;
    let mut Status: NTSTATUS;

    Urb = AllocateItem(
        NonPagedPool,
        ::core::mem::size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as ULONG,
    ) as PURB;
    if Urb.is_null() {
        if DbgPrint(
            b"(%s:%d) OutofMemory!\n\0" as *const u8 as PCSTR,
            b"error.c\0" as *const u8 as *const ::core::ffi::c_char,
            30 as ::core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"error.c\0" as *const u8 as *const ::core::ffi::c_char,
                30 as ::core::ffi::c_int,
            );
        }
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    UsbBuildGetStatusRequest(
        Urb,
        URB_FUNCTION_GET_STATUS_FROM_ENDPOINT,
        (bEndpointAddress as ::core::ffi::c_int & 0x0f as ::core::ffi::c_int) as USHORT,
        Value as PVOID,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if DbgPrint(
        b"(%s:%d) Sending Request DeviceObject %p, Urb %p\n\0" as *const u8 as PCSTR,
        b"error.c\0" as *const u8 as *const ::core::ffi::c_char,
        38 as ::core::ffi::c_int,
        DeviceObject,
        Urb,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"error.c\0" as *const u8 as *const ::core::ffi::c_char,
            38 as ::core::ffi::c_int,
        );
    }

    Status = USBSTOR_SyncUrbRequest(DeviceObject, Urb);

    FreeItem(Urb as PVOID);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_ResetPipeWithHandle(
    mut DeviceObject: PDEVICE_OBJECT,
    mut PipeHandle: USBD_PIPE_HANDLE,
) -> NTSTATUS {
    const URB_FUNCTION_SYNC_RESET_PIPE_AND_CLEAR_STALL: u32 = 0x001E;

    let mut Urb: PURB = core::ptr::null_mut::<_URB>();
    let mut Status: NTSTATUS;

    if DbgPrint(
        b"(%s:%d) Allocating URB\n\0" as *const u8 as PCSTR,
        b"error.c\0" as *const u8 as *const core::ffi::c_char,
        49 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"error.c\0" as *const u8 as *const core::ffi::c_char,
            49 as core::ffi::c_int,
        );
    }
    Urb = AllocateItem(
        NonPagedPool,
        core::mem::size_of::<_URB_PIPE_REQUEST>() as ULONG,
    ) as PURB;
    if Urb.is_null() {
        if DbgPrint(
            b"(%s:%d) OutofMemory!\n\0" as *const u8 as PCSTR,
            b"error.c\0" as *const u8 as *const core::ffi::c_char,
            57 as core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"error.c\0" as *const u8 as *const core::ffi::c_char,
                57 as core::ffi::c_int,
            );
        }
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Urb).__bindgen_anon_1.UrbPipeRequest.as_mut().Hdr.Length =
        core::mem::size_of::<_URB_PIPE_REQUEST>() as USHORT;
    (*Urb).__bindgen_anon_1.UrbPipeRequest.as_mut().Hdr.Function =
        URB_FUNCTION_SYNC_RESET_PIPE_AND_CLEAR_STALL as USHORT;
    (*Urb).__bindgen_anon_1.UrbPipeRequest.as_mut().PipeHandle = PipeHandle;

    if DbgPrint(
        b"(%s:%d) Sending Request DeviceObject %p, Urb %p\n\0" as *const u8 as PCSTR,
        b"error.c\0" as *const u8 as *const core::ffi::c_char,
        66 as core::ffi::c_int,
        DeviceObject,
        Urb,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"error.c\0" as *const u8 as *const core::ffi::c_char,
            66 as core::ffi::c_int,
        );
    }
    Status = USBSTOR_SyncUrbRequest(DeviceObject, Urb);

    FreeItem(Urb as PVOID);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_ResetPipeWorkItemRoutine(
    mut FdoDevice: PDEVICE_OBJECT,
    mut Ctx: PVOID,
) {
    let mut Status: NTSTATUS;
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION = Ctx as PFDO_DEVICE_EXTENSION;
    let mut Context: PIRP_CONTEXT = core::ptr::addr_of_mut!((*FDODeviceExtension).CurrentIrpContext);

    // clear stall on the corresponding pipe
    Status = USBSTOR_ResetPipeWithHandle(
        (*FDODeviceExtension).LowerDeviceObject,
        (*Context).Urb.__bindgen_anon_1.UrbBulkOrInterruptTransfer.as_ref().PipeHandle,
    );
    if DbgPrint(
        b"(%s:%d) USBSTOR_ResetPipeWithHandle Status %x\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/error.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        85 as ::core::ffi::c_int,
        Status,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/error.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            85 as ::core::ffi::c_int,
        );
    }

    // now resend the csw as the stall got cleared
    USBSTOR_SendCSWRequest(FDODeviceExtension, (*Context).Irp);
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_ResetDeviceWorkItemRoutine(
    mut FdoDevice: PDEVICE_OBJECT,
    mut Context: PVOID,
) {

    const USBSTOR_FDO_FLAGS_DEVICE_RESETTING: u32 = 0x00000001;

    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut ix: UINT32;
    let mut Status: NTSTATUS;
    let mut OldIrql: KIRQL = 0;

    FDODeviceExtension = (*FdoDevice).DeviceExtension as PFDO_DEVICE_EXTENSION;

    ix = 0 as UINT32;
    while ix < 3 {
        Status = USBSTOR_ResetDevice((*FDODeviceExtension).LowerDeviceObject, FDODeviceExtension);
        if NT_SUCCESS(Status) {
            Status = USBSTOR_ResetPipeWithHandle(
                (*FDODeviceExtension).LowerDeviceObject,
                (*(core::ptr::addr_of_mut!(
                    (*(*FDODeviceExtension).InterfaceInformation).Pipes
                ) as *mut USBD_PIPE_INFORMATION)
                    .offset((*FDODeviceExtension).BulkInPipeIndex as isize))
                .PipeHandle,
            );
            if NT_SUCCESS(Status) {
                Status = USBSTOR_ResetPipeWithHandle(
                    (*FDODeviceExtension).LowerDeviceObject,
                    (*(core::ptr::addr_of_mut!(
                        (*(*FDODeviceExtension).InterfaceInformation).Pipes
                    ) as *mut USBD_PIPE_INFORMATION)
                        .offset((*FDODeviceExtension).BulkOutPipeIndex as isize))
                    .PipeHandle,
                );
                if NT_SUCCESS(Status) {
                    break;
                }
            }
        }
        ix = ix.wrapping_add(1);
    }

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*FDODeviceExtension).CommonLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(OldIrql),
    );
    (*FDODeviceExtension).Flags &= !USBSTOR_FDO_FLAGS_DEVICE_RESETTING as core::ffi::c_ulong;
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*FDODeviceExtension).CommonLock) as PKSPIN_LOCK,
        OldIrql,
    );

    USBSTOR_QueueNextRequest(FdoDevice);
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_QueueResetPipe(FDODeviceExtension: PFDO_DEVICE_EXTENSION) {
    // DPRINT("USBSTOR_QueueResetPipe\n");

    IoQueueWorkItem(
        (*FDODeviceExtension).ResetDeviceWorkItem,
        core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PVOID),
            PIO_WORKITEM_ROUTINE,
        >(USBSTOR_ResetPipeWorkItemRoutine),
        CriticalWorkQueue,
        FDODeviceExtension as PVOID,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_QueueResetDevice(FDODeviceExtension: PFDO_DEVICE_EXTENSION) {
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*FDODeviceExtension).CommonLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(OldIrql),
    );
    (*FDODeviceExtension).Flags |= USBSTOR_FDO_FLAGS_DEVICE_RESETTING as u32;
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*FDODeviceExtension).CommonLock) as PKSPIN_LOCK,
        OldIrql,
    );

    IoQueueWorkItem(
        (*FDODeviceExtension).ResetDeviceWorkItem,
        core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, *mut core::ffi::c_void),
            PIO_WORKITEM_ROUTINE,
        >(USBSTOR_ResetDeviceWorkItemRoutine),
        CriticalWorkQueue,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_TimerWorkerRoutine(mut Context: PVOID) {
    const USB_STOR_TAG: ULONG = 0x73627375;

    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;
    let mut WorkItemData: PERRORHANDLER_WORKITEM_DATA = Context as PERRORHANDLER_WORKITEM_DATA;

    FDODeviceExtension =
        (*(*WorkItemData).DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    if (*FDODeviceExtension).Common.IsFDO == 0 {
        RtlAssert(
            b"FDODeviceExtension->Common.IsFDO\0".as_ptr() as *const i8 as PVOID,
            b"error.c\0".as_ptr() as *const i8 as PVOID,
            175 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    // first perform a mass storage reset step 1 in 5.3.4 USB Mass Storage Bulk Only Specification
    Status = USBSTOR_ResetDevice((*FDODeviceExtension).LowerDeviceObject, FDODeviceExtension);
    if NT_SUCCESS(Status) {
        // step 2 reset bulk in pipe section 5.3.4
        Status = USBSTOR_ResetPipeWithHandle(
            (*FDODeviceExtension).LowerDeviceObject,
            (*(core::ptr::addr_of_mut!((*(*FDODeviceExtension).InterfaceInformation).Pipes)
                as *mut USBD_PIPE_INFORMATION)
                .offset((*FDODeviceExtension).BulkInPipeIndex as isize))
            .PipeHandle,
        );
        if NT_SUCCESS(Status) {
            // finally reset bulk out pipe
            Status = USBSTOR_ResetPipeWithHandle(
                (*FDODeviceExtension).LowerDeviceObject,
                (*(core::ptr::addr_of_mut!((*(*FDODeviceExtension).InterfaceInformation).Pipes)
                    as *mut USBD_PIPE_INFORMATION)
                    .offset((*FDODeviceExtension).BulkOutPipeIndex as isize))
                .PipeHandle,
            );
        }
    }
    if DbgPrint(
        b"(%s:%d) Status %x\n\0".as_ptr() as PCSTR,
        b"error.c\0".as_ptr() as *const i8,
        189 as i32,
        Status,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"error.c\0".as_ptr() as *const i8,
            189 as i32,
        );
    }

    // clear timer srb
    (*FDODeviceExtension).LastTimerActiveSrb = core::ptr::null_mut::<_SCSI_REQUEST_BLOCK>();

    // do not retry for the same packet again
    (*FDODeviceExtension).TimerWorkQueueEnabled = 0 as ULONG;

    ExFreePoolWithTag(WorkItemData as PVOID, USB_STOR_TAG as ULONG);
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_TimerRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID,
) {
    const USB_STOR_TAG: ULONG = 0x73627375;

    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut ResetDevice: BOOLEAN = 0 as BOOLEAN;
    let mut WorkItemData: PERRORHANDLER_WORKITEM_DATA;

    FDODeviceExtension = Context as PFDO_DEVICE_EXTENSION;
    if DbgPrint(
        b"(%s:%d) [USBSTOR] TimerRoutine entered\n\0" as *const u8 as PCSTR,
        b"error.c\0" as *const u8 as *const ::core::ffi::c_char,
        214 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"error.c\0" as *const u8 as *const ::core::ffi::c_char,
            214 as ::core::ffi::c_int,
        );
    }

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock) as PKSPIN_LOCK);

    if !(*FDODeviceExtension).ActiveSrb.is_null()
        && (*FDODeviceExtension).TimerWorkQueueEnabled != 0
    {
        if !(*FDODeviceExtension).LastTimerActiveSrb.is_null()
            && (*FDODeviceExtension).LastTimerActiveSrb == (*FDODeviceExtension).ActiveSrb
        {
            if DbgPrint(
                b"(%s:%d) [USBSTOR] ActiveSrb %p hang detected\n\0" as *const u8 as PCSTR,
                b"error.c\0" as *const u8 as *const ::core::ffi::c_char,
                225 as ::core::ffi::c_int,
                (*FDODeviceExtension).ActiveSrb,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"error.c\0" as *const u8 as *const ::core::ffi::c_char,
                    225 as ::core::ffi::c_int,
                );
            }
            ResetDevice = 1 as BOOLEAN;
        } else {
            (*FDODeviceExtension).LastTimerActiveSrb = (*FDODeviceExtension).ActiveSrb;
        }
    } else {
        (*FDODeviceExtension).LastTimerActiveSrb = core::ptr::null_mut();
    }

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock) as PKSPIN_LOCK);

    if ResetDevice as ::core::ffi::c_int != 0
        && (*FDODeviceExtension).TimerWorkQueueEnabled != 0
        && (*FDODeviceExtension).SrbErrorHandlingActive == 0
    {
        WorkItemData = ExAllocatePoolWithTag(
            NonPagedPool,
            ::core::mem::size_of::<ERRORHANDLER_WORKITEM_DATA>() as SIZE_T,
            USB_STOR_TAG,
        ) as PERRORHANDLER_WORKITEM_DATA;
        if !WorkItemData.is_null() {
            ExInitializeWorkItem(
                core::ptr::addr_of_mut!((*WorkItemData).WorkQueueItem),
                core::mem::transmute::<
                    unsafe extern "system" fn(PVOID),
                    PWORKER_THREAD_ROUTINE,
                >(USBSTOR_TimerWorkerRoutine),
                WorkItemData as PVOID,
            );

            (*WorkItemData).DeviceObject = (*FDODeviceExtension).FunctionalDeviceObject;

            if DbgPrint(
                b"(%s:%d) [USBSTOR] Queing Timer WorkItem\n\0" as *const u8 as PCSTR,
                b"error.c\0" as *const u8 as *const ::core::ffi::c_char,
                257 as ::core::ffi::c_int,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"error.c\0" as *const u8 as *const ::core::ffi::c_char,
                    257 as ::core::ffi::c_int,
                );
            }
            ExQueueWorkItem(
                core::ptr::addr_of_mut!((*WorkItemData).WorkQueueItem),
                DelayedWorkQueue,
            );
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Universal Serial Bus Bulk Storage Driver
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     USB block storage device driver.
//  * COPYRIGHT:   2005-2006 James Tabor
//  *              2011-2012 Michael Martin (michael.martin@reactos.org)
//  *              2011-2013 Johannes Anderwald (johannes.anderwald@reactos.org)
//  *              2019 Victor Perevertkin (victor.perevertkin@reactos.org)
//  */
//
// #include "usbstor.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// NTSTATUS
// USBSTOR_GetEndpointStatus(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN UCHAR bEndpointAddress,
//     OUT PUSHORT Value)
// {
//     PURB Urb;
//     NTSTATUS Status;
//
//     DPRINT("Allocating URB\n");
//     Urb = (PURB)AllocateItem(NonPagedPool, sizeof(struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST));
//     if (!Urb)
//     {
//         DPRINT1("OutofMemory!\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // build status
//     UsbBuildGetStatusRequest(Urb, URB_FUNCTION_GET_STATUS_FROM_ENDPOINT, bEndpointAddress & 0x0F, Value, NULL, NULL);
//
//     // send the request
//     DPRINT1("Sending Request DeviceObject %p, Urb %p\n", DeviceObject, Urb);
//     Status = USBSTOR_SyncUrbRequest(DeviceObject, Urb);
//
//     FreeItem(Urb);
//     return Status;
// }
//
// NTSTATUS
// USBSTOR_ResetPipeWithHandle(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN USBD_PIPE_HANDLE PipeHandle)
// {
//     PURB Urb;
//     NTSTATUS Status;
//
//     DPRINT("Allocating URB\n");
//     Urb = (PURB)AllocateItem(NonPagedPool, sizeof(struct _URB_PIPE_REQUEST));
//     if (!Urb)
//     {
//         DPRINT1("OutofMemory!\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Urb->UrbPipeRequest.Hdr.Length = sizeof(struct _URB_PIPE_REQUEST);
//     Urb->UrbPipeRequest.Hdr.Function = URB_FUNCTION_SYNC_RESET_PIPE_AND_CLEAR_STALL;
//     Urb->UrbPipeRequest.PipeHandle = PipeHandle;
//
//     // send the request
//     DPRINT1("Sending Request DeviceObject %p, Urb %p\n", DeviceObject, Urb);
//     Status = USBSTOR_SyncUrbRequest(DeviceObject, Urb);
//
//     FreeItem(Urb);
//     return Status;
// }
//
// VOID
// NTAPI
// USBSTOR_ResetPipeWorkItemRoutine(
//     IN PDEVICE_OBJECT FdoDevice,
//     IN PVOID Ctx)
// {
//     NTSTATUS Status;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension = (PFDO_DEVICE_EXTENSION)Ctx;
//     PIRP_CONTEXT Context = &FDODeviceExtension->CurrentIrpContext;
//
//     // clear stall on the corresponding pipe
//     Status = USBSTOR_ResetPipeWithHandle(FDODeviceExtension->LowerDeviceObject, Context->Urb.UrbBulkOrInterruptTransfer.PipeHandle);
//     DPRINT1("USBSTOR_ResetPipeWithHandle Status %x\n", Status);
//
//     // now resend the csw as the stall got cleared
//     USBSTOR_SendCSWRequest(FDODeviceExtension, Context->Irp);
// }
//
// VOID
// NTAPI
// USBSTOR_ResetDeviceWorkItemRoutine(
//     IN PDEVICE_OBJECT FdoDevice,
//     IN PVOID Context)
// {
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     UINT32 ix;
//     NTSTATUS Status;
//     KIRQL OldIrql;
//
//     DPRINT("USBSTOR_ResetDeviceWorkItemRoutine\n");
//
//     FDODeviceExtension = FdoDevice->DeviceExtension;
//
//     for (ix = 0; ix < 3; ++ix)
//     {
//         // first perform a mass storage reset step 1 in 5.3.4 USB Mass Storage Bulk Only Specification
//         Status = USBSTOR_ResetDevice(FDODeviceExtension->LowerDeviceObject, FDODeviceExtension);
//         if (NT_SUCCESS(Status))
//         {
//             // step 2 reset bulk in pipe section 5.3.4
//             Status = USBSTOR_ResetPipeWithHandle(FDODeviceExtension->LowerDeviceObject, FDODeviceExtension->InterfaceInformation->Pipes[FDODeviceExtension->BulkInPipeIndex].PipeHandle);
//             if (NT_SUCCESS(Status))
//             {
//                 // finally reset bulk out pipe
//                 Status = USBSTOR_ResetPipeWithHandle(FDODeviceExtension->LowerDeviceObject, FDODeviceExtension->InterfaceInformation->Pipes[FDODeviceExtension->BulkOutPipeIndex].PipeHandle);
//                 if (NT_SUCCESS(Status))
//                 {
//                     break;
//                 }
//             }
//         }
//     }
//
//     KeAcquireSpinLock(&FDODeviceExtension->CommonLock, &OldIrql);
//     FDODeviceExtension->Flags &= ~USBSTOR_FDO_FLAGS_DEVICE_RESETTING;
//     KeReleaseSpinLock(&FDODeviceExtension->CommonLock, OldIrql);
//
//     USBSTOR_QueueNextRequest(FdoDevice);
// }
//
// VOID
// NTAPI
// USBSTOR_QueueResetPipe(
//     IN PFDO_DEVICE_EXTENSION FDODeviceExtension)
// {
//     DPRINT("USBSTOR_QueueResetPipe\n");
//
//     IoQueueWorkItem(FDODeviceExtension->ResetDeviceWorkItem,
//                     USBSTOR_ResetPipeWorkItemRoutine,
//                     CriticalWorkQueue,
//                     FDODeviceExtension);
// }
//
// VOID
// NTAPI
// USBSTOR_QueueResetDevice(
//     IN PFDO_DEVICE_EXTENSION FDODeviceExtension)
// {
//     KIRQL OldIrql;
//
//     DPRINT("USBSTOR_QueueResetDevice\n");
//
//     KeAcquireSpinLock(&FDODeviceExtension->CommonLock, &OldIrql);
//     FDODeviceExtension->Flags |= USBSTOR_FDO_FLAGS_DEVICE_RESETTING;
//     KeReleaseSpinLock(&FDODeviceExtension->CommonLock, OldIrql);
//
//     IoQueueWorkItem(FDODeviceExtension->ResetDeviceWorkItem,
//                     USBSTOR_ResetDeviceWorkItemRoutine,
//                     CriticalWorkQueue,
//                     NULL);
// }
//
// VOID
// NTAPI
// USBSTOR_TimerWorkerRoutine(
//     IN PVOID Context)
// {
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     NTSTATUS Status;
//     PERRORHANDLER_WORKITEM_DATA WorkItemData = (PERRORHANDLER_WORKITEM_DATA)Context;
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)WorkItemData->DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     // first perform a mass storage reset step 1 in 5.3.4 USB Mass Storage Bulk Only Specification
//     Status = USBSTOR_ResetDevice(FDODeviceExtension->LowerDeviceObject, FDODeviceExtension);
//     if (NT_SUCCESS(Status))
//     {
//         // step 2 reset bulk in pipe section 5.3.4
//         Status = USBSTOR_ResetPipeWithHandle(FDODeviceExtension->LowerDeviceObject, FDODeviceExtension->InterfaceInformation->Pipes[FDODeviceExtension->BulkInPipeIndex].PipeHandle);
//         if (NT_SUCCESS(Status))
//         {
//             // finally reset bulk out pipe
//             Status = USBSTOR_ResetPipeWithHandle(FDODeviceExtension->LowerDeviceObject, FDODeviceExtension->InterfaceInformation->Pipes[FDODeviceExtension->BulkOutPipeIndex].PipeHandle);
//         }
//     }
//     DPRINT1("Status %x\n", Status);
//
//     // clear timer srb
//     FDODeviceExtension->LastTimerActiveSrb = NULL;
//
//     // re-schedule request
//     //USBSTOR_HandleExecuteSCSI(WorkItemData->Context->PDODeviceExtension->Self, WorkItemData->Context->Irp, Context->RetryCount + 1);
//
//     // do not retry for the same packet again
//     FDODeviceExtension->TimerWorkQueueEnabled = FALSE;
//
//     ExFreePoolWithTag(WorkItemData, USB_STOR_TAG);
// }
//
// VOID
// NTAPI
// USBSTOR_TimerRoutine(
//     PDEVICE_OBJECT DeviceObject,
//      PVOID Context)
// {
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     BOOLEAN ResetDevice = FALSE;
//     PERRORHANDLER_WORKITEM_DATA WorkItemData;
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)Context;
//     DPRINT1("[USBSTOR] TimerRoutine entered\n");
//     // DPRINT1("[USBSTOR] ActiveSrb %p ResetInProgress %x LastTimerActiveSrb %p\n", FDODeviceExtension->ActiveSrb, FDODeviceExtension->ResetInProgress, FDODeviceExtension->LastTimerActiveSrb);
//
//     KeAcquireSpinLockAtDpcLevel(&FDODeviceExtension->IrpListLock);
//
//     // is there an active srb and no global reset is in progress
//     if (FDODeviceExtension->ActiveSrb && /* FDODeviceExtension->ResetInProgress == FALSE && */ FDODeviceExtension->TimerWorkQueueEnabled)
//     {
//         if (FDODeviceExtension->LastTimerActiveSrb != NULL && FDODeviceExtension->LastTimerActiveSrb == FDODeviceExtension->ActiveSrb)
//         {
//             // check if empty
//             DPRINT1("[USBSTOR] ActiveSrb %p hang detected\n", FDODeviceExtension->ActiveSrb);
//             ResetDevice = TRUE;
//         }
//         else
//         {
//             // update pointer
//             FDODeviceExtension->LastTimerActiveSrb = FDODeviceExtension->ActiveSrb;
//         }
//     }
//     else
//     {
//         // reset srb
//         FDODeviceExtension->LastTimerActiveSrb = NULL;
//     }
//
//     KeReleaseSpinLockFromDpcLevel(&FDODeviceExtension->IrpListLock);
//
//
//     if (ResetDevice && FDODeviceExtension->TimerWorkQueueEnabled && FDODeviceExtension->SrbErrorHandlingActive == FALSE)
//     {
//         WorkItemData = ExAllocatePoolWithTag(NonPagedPool,
//                                              sizeof(ERRORHANDLER_WORKITEM_DATA),
//                                              USB_STOR_TAG);
//         if (WorkItemData)
//         {
//            // Initialize and queue the work item to handle the error
//            ExInitializeWorkItem(&WorkItemData->WorkQueueItem,
//                                  USBSTOR_TimerWorkerRoutine,
//                                  WorkItemData);
//
//            WorkItemData->DeviceObject = FDODeviceExtension->FunctionalDeviceObject;
//
//            DPRINT1("[USBSTOR] Queing Timer WorkItem\n");
//            ExQueueWorkItem(&WorkItemData->WorkQueueItem, DelayedWorkQueue);
//         }
//      }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
