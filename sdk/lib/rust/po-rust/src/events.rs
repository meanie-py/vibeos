use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    HANDLE,
    DEVICE_INTERFACE_CHANGE_NOTIFICATION, PDEVICE_INTERFACE_CHANGE_NOTIFICATION,
    GUID, _IO_STATUS_BLOCK, KEVENT, _OBJECT_ATTRIBUTES, _UNICODE_STRING,
    PDEVICE_OBJECT, PFILE_OBJECT, PIO_WORKITEM, PIRP,
    PKWIN32_POWEREVENT_CALLOUT, PCALLBACK_OBJECT,
    _EVENT_TYPE_NotificationEvent,
    _KWAIT_REASON_Suspended,
    ExNotifyCallback, IoAllocateWorkItem, IoBuildDeviceIoControlRequest,
    IoFreeWorkItem, IofCallDriver, IoGetRelatedDeviceObject, IoQueueWorkItem,
    IoSetCompletionRoutine, KeInitializeEvent, KeWaitForSingleObject,
    ObReferenceObjectByHandle, RtlCompareMemory, ZwClose, ZwOpenFile,
    ZwShutdownSystem,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn InitializeObjectAttributes(
        p: *mut _OBJECT_ATTRIBUTES,
        n: *mut _UNICODE_STRING,
        a: ULONG,
        r: HANDLE,
        s: PVOID,
    );
    static IoFileObjectType: ntkrnl_sys::ddk::POBJECT_TYPE;
    static mut PopCapabilities: POP_SYSTEM_POWER_CAPABILITIES;
    static GUID_DEVICE_INTERFACE_ARRIVAL: GUID;
    static GUID_DEVICE_INTERFACE_REMOVAL: GUID;
    fn KeRaiseIrql(new_irql: KIRQL, old_irql: *mut KIRQL);
    fn KeLowerIrql(irql: KIRQL);
}

extern "system" {
    fn IoCallDriver(device: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS;
}

const IOCTL_GET_SYS_BUTTON_EVENT: ULONG = 0x220000;
const IOCTL_GET_SYS_BUTTON_CAPS: ULONG = 0x220004;
const SYS_BUTTON_POWER: ULONG = 0x01;
const SYS_BUTTON_SLEEP: ULONG = 0x02;
const SYS_BUTTON_LID: ULONG = 0x04;
const DelayedWorkQueue: u32 = 1;
const ShutdownNoReboot: u32 = 0;
const FILE_READ_DATA: ULONG = 0x0001;
const FILE_SHARE_READ: ULONG = 0x00000001;
const FILE_SHARE_WRITE: ULONG = 0x00000002;

#[repr(C)]
struct POP_SYSTEM_POWER_CAPABILITIES {
    PowerButtonPresent: BOOLEAN,
    SleepButtonPresent: BOOLEAN,
    LidPresent: BOOLEAN,
    SystemS1: BOOLEAN,
    SystemS2: BOOLEAN,
    SystemS3: BOOLEAN,
    SystemS4: BOOLEAN,
    SystemS5: BOOLEAN,
    HiberFilePresent: BOOLEAN,
    FullWake: BOOLEAN,
    VideoDimPresent: BOOLEAN,
    ApmPresent: BOOLEAN,
    UpsPresent: BOOLEAN,
    ThermalControl: BOOLEAN,
    ProcessorThrottle: BOOLEAN,
    ProcessorMinThrottle: UCHAR,
    ProcessorMaxThrottle: UCHAR,
    FastSystemS4: BOOLEAN,
    Hiberboot: BOOLEAN,
    WakeAlarmPresent: BOOLEAN,
    AoAc: BOOLEAN,
    DiskSpinDown: BOOLEAN,
    HiberFileType: UCHAR,
    AoAcConnectivitySupported: BOOLEAN,
    spare3: [UCHAR; 6],
    SystemBatteriesPresent: BOOLEAN,
    BatteriesAreShortTerm: BOOLEAN,
    BatteryScale: [u8; 12],
    AcOnLineWake: i32,
    SoftLidWake: i32,
    RtcWake: i32,
    MinDeviceWakeState: i32,
    DefaultLowLatencyWake: i32,
}

#[repr(C)]
struct SYS_BUTTON_CONTEXT {
    DeviceObject: PDEVICE_OBJECT,
    WorkItem: PIO_WORKITEM,
    Event: KEVENT,
    IoStatusBlock: _IO_STATUS_BLOCK,
    SysButton: ULONG,
}

type PSYS_BUTTON_CONTEXT = *mut SYS_BUTTON_CONTEXT;

#[repr(C)]
enum POP_POLICY_DEVICE_TYPE {
    PolicyDeviceSystemButton = 0,
    PolicyDeviceThermalZone = 1,
    PolicyDeviceBattery = 2,
    PolicyDeviceMemory = 3,
    PolicyInitiatePowerActionAPI = 4,
    PolicySetPowerStateAPI = 5,
    PolicyImmediateDozeS4 = 6,
    PolicySystemIdle = 7,
    PolicyDeviceMax = 8,
}

const PolicyDeviceBattery: POP_POLICY_DEVICE_TYPE = POP_POLICY_DEVICE_TYPE::PolicyDeviceBattery;

#[no_mangle]
pub static mut PopEventCallout: PKWIN32_POWEREVENT_CALLOUT = None;

extern "C" {
    static mut SetSystemTimeCallback: PCALLBACK_OBJECT;
}

#[no_mangle]
pub unsafe extern "system" fn PoNotifySystemTimeSet() {
    let mut old_irql: KIRQL = 0;

    if PopEventCallout.is_some() {
        // [guard] KeRaiseIrql(DISPATCH_LEVEL, &mut old_irql);
        let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(DISPATCH_LEVEL) };
        ExNotifyCallback(SetSystemTimeCallback, core::ptr::null_mut(), core::ptr::null_mut());
        drop(irql_guard);
    }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "stdcall" fn PopGetSysButtonCompletion(
    _device_object: PDEVICE_OBJECT,
    irp: PIRP,
    context: PVOID,
) -> NTSTATUS {
    let _device_object = opaque_ptr(_device_object);
    let irp = opaque_ptr(irp);
    let context = opaque_ptr(context);
    PopGetSysButtonCompletionInner(_device_object, irp, context)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn PopGetSysButtonCompletion(
    _device_object: PDEVICE_OBJECT,
    irp: PIRP,
    context: PVOID,
) -> NTSTATUS {
    let _device_object = opaque_ptr(_device_object);
    let irp = opaque_ptr(irp);
    let context = opaque_ptr(context);
    PopGetSysButtonCompletionInner(_device_object, irp, context)
}

#[no_mangle]
#[inline(always)]
pub unsafe extern "system" fn PopGetSysButtonCompletionInner(
    _device_object: PDEVICE_OBJECT,
    irp: PIRP,
    context: PVOID,
) -> NTSTATUS {
    let _device_object = opaque_ptr(_device_object);
    let irp = opaque_ptr(irp);
    let context = opaque_ptr(context);
    let sys_button_context = context as PSYS_BUTTON_CONTEXT;
    let sys_button: ULONG;

    let device_object = (*sys_button_context).DeviceObject;

    sys_button = *((*irp).AssociatedIrp.SystemBuffer as *mut ULONG);

    (*sys_button_context).WorkItem = IoAllocateWorkItem(device_object);
    if (*sys_button_context).WorkItem.is_null() {
        ExFreePoolWithTag(sys_button_context as PVOID, 0x49574F50);
        return STATUS_SUCCESS;
    }
    IoQueueWorkItem(
        (*sys_button_context).WorkItem,
        Some(PopGetSysButton),
        DelayedWorkQueue,
        sys_button_context as PVOID,
    );

    STATUS_SUCCESS
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "stdcall" fn PopGetSysButton(
    device_object: PDEVICE_OBJECT,
    context: PVOID,
) {
    let device_object = opaque_ptr(device_object);
    let context = opaque_ptr(context);
    PopGetSysButtonInner(device_object, context)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn PopGetSysButton(
    device_object: PDEVICE_OBJECT,
    context: PVOID,
) {
    let device_object = opaque_ptr(device_object);
    let context = opaque_ptr(context);
    PopGetSysButtonInner(device_object, context)
}

#[no_mangle]
#[inline(always)]
pub unsafe extern "system" fn PopGetSysButtonInner(
    device_object: PDEVICE_OBJECT,
    context: PVOID,
) {
    let device_object = opaque_ptr(device_object);
    let context = opaque_ptr(context);
    let sys_button_context = context as PSYS_BUTTON_CONTEXT;
    let current_work_item = (*sys_button_context).WorkItem;
    let irp: PIRP;

    KeInitializeEvent(
        core::ptr::addr_of_mut!((*sys_button_context).Event),
        _EVENT_TYPE_NotificationEvent,
        0u8,
    );
    irp = IoBuildDeviceIoControlRequest(
        IOCTL_GET_SYS_BUTTON_EVENT,
        device_object,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!((*sys_button_context).SysButton) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        0u8,
        core::ptr::addr_of_mut!((*sys_button_context).Event),
        core::ptr::addr_of_mut!((*sys_button_context).IoStatusBlock),
    );
    if !irp.is_null() {
        IoSetCompletionRoutine(
            irp,
            Some(PopGetSysButtonCompletion),
            sys_button_context as PVOID,
            1u8,
            0u8,
            0u8,
        );
        IoCallDriver(device_object, irp);
    } else {
        ExFreePoolWithTag(sys_button_context as PVOID, 0x49574F50);
    }

    IoFreeWorkItem(current_work_item);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn PopAddRemoveSysCapsCallback(
    notification_structure: PVOID,
    context: PVOID,
) -> NTSTATUS {
    let notification_structure = opaque_ptr(notification_structure);
    let context = opaque_ptr(context);
    PopAddRemoveSysCapsCallbackInner(notification_structure, context)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PopAddRemoveSysCapsCallback(
    notification_structure: PVOID,
    context: PVOID,
) -> NTSTATUS {
    let notification_structure = opaque_ptr(notification_structure);
    let context = opaque_ptr(context);
    PopAddRemoveSysCapsCallbackInner(notification_structure, context)
}

#[no_mangle]
#[inline(always)]
pub unsafe extern "system" fn PopAddRemoveSysCapsCallbackInner(
    notification_structure: PVOID,
    context: PVOID,
) -> NTSTATUS {
    let notification_structure = opaque_ptr(notification_structure);
    let context = opaque_ptr(context);
    let notification: PDEVICE_INTERFACE_CHANGE_NOTIFICATION;
    let mut sys_button_context: PSYS_BUTTON_CONTEXT;
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut file_handle: HANDLE = core::ptr::null_mut();
    let mut device_object: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut file_object: PFILE_OBJECT = core::ptr::null_mut();
    let irp: PIRP;
    let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut event: KEVENT = core::mem::zeroed();
    let arrival: BOOLEAN;
    let mut caps: ULONG = 0;
    let mut status: NTSTATUS;
    let device_type: POP_POLICY_DEVICE_TYPE = core::mem::transmute((context as usize) as u32);

    notification = notification_structure as PDEVICE_INTERFACE_CHANGE_NOTIFICATION;
    if (*notification).Version != 1 {
        return STATUS_REVISION_MISMATCH;
    }
    if (*notification).Size != core::mem::size_of::<DEVICE_INTERFACE_CHANGE_NOTIFICATION>() as u16 {
        return STATUS_INVALID_PARAMETER;
    }
    let guid_size = core::mem::size_of::<GUID>() as ntkrnl_sys::ddk::SIZE_T;
    if RtlCompareMemory(
        core::ptr::addr_of!((*notification).Event) as PVOID,
        core::ptr::addr_of!(GUID_DEVICE_INTERFACE_ARRIVAL) as PVOID,
        guid_size,
    ) == guid_size
    {
        arrival = 1u8;
    } else if RtlCompareMemory(
        core::ptr::addr_of!((*notification).Event) as PVOID,
        core::ptr::addr_of!(GUID_DEVICE_INTERFACE_REMOVAL) as PVOID,
        guid_size,
    ) == guid_size
    {
        arrival = 0u8;
    } else {
        return STATUS_INVALID_PARAMETER;
    }

    if arrival != 0 && matches!(device_type, POP_POLICY_DEVICE_TYPE::PolicyDeviceBattery) {
        PopCapabilities.SystemBatteriesPresent = 1u8;
        return STATUS_SUCCESS;
    }

    if arrival != 0 {
        InitializeObjectAttributes(
            &mut object_attributes,
            (*notification).SymbolicLinkName,
            OBJ_KERNEL_HANDLE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        status = ZwOpenFile(
            &mut file_handle,
            FILE_READ_DATA,
            &mut object_attributes,
            &mut io_status_block,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            0,
        );
        if status < 0 {
            return status;
        }
        status = ObReferenceObjectByHandle(
            file_handle,
            FILE_READ_DATA,
            IoFileObjectType,
            0i8,
            core::ptr::addr_of_mut!(file_object) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if status < 0 {
            ZwClose(file_handle);
            return status;
        }
        device_object = IoGetRelatedDeviceObject(file_object);
        ObDereferenceObject(file_object as PVOID);

        KeInitializeEvent(&mut event, _EVENT_TYPE_NotificationEvent, 0u8);
        irp = IoBuildDeviceIoControlRequest(
            IOCTL_GET_SYS_BUTTON_CAPS,
            device_object,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(caps) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
            0u8,
            &mut event,
            &mut io_status_block,
        );
        if irp.is_null() {
            ZwClose(file_handle);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        status = IoCallDriver(device_object, irp);
        if status == STATUS_PENDING {
            // RunToken pilot: synchronous-IRP wait via the CPS token API; the
            // IOSB read runs in the continuation, after the re-brand.
            status = ntkrnl_sys::token::with_token::<ntkrnl_sys::token::Passive, _>(|tok| {
                ntkrnl_sys::token::wait_for(
                    tok,
                    &mut event as *mut KEVENT as PVOID,
                    _KWAIT_REASON_Suspended as u32,
                    0i8,
                    0u8,
                    core::ptr::null_mut(),
                    |_tok, _wait_status| io_status_block.__bindgen_anon_1.Status,
                )
            });
        }
        if status < 0 {
            ZwClose(file_handle);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        if caps & SYS_BUTTON_POWER != 0 {
            PopCapabilities.PowerButtonPresent = 1u8;
        }

        if caps & SYS_BUTTON_SLEEP != 0 {
            PopCapabilities.SleepButtonPresent = 1u8;
        }

        if caps & SYS_BUTTON_LID != 0 {
            PopCapabilities.LidPresent = 1u8;
        }

        sys_button_context = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<SYS_BUTTON_CONTEXT>(),
            0x49574F50,
        ) as PSYS_BUTTON_CONTEXT;
        if sys_button_context.is_null() {
            ZwClose(file_handle);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*sys_button_context).WorkItem = IoAllocateWorkItem(device_object);
        (*sys_button_context).DeviceObject = device_object;
        if (*sys_button_context).WorkItem.is_null() {
            ZwClose(file_handle);
            ExFreePoolWithTag(sys_button_context as PVOID, 0x49574F50);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        IoQueueWorkItem(
            (*sys_button_context).WorkItem,
            Some(PopGetSysButton),
            DelayedWorkQueue,
            sys_button_context as PVOID,
        );

        ZwClose(file_handle);
        return STATUS_SUCCESS;
    } else {
        return STATUS_NOT_IMPLEMENTED;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/po/events.c
//  * PURPOSE:         Power Manager
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// typedef struct _SYS_BUTTON_CONTEXT
// {
//     PDEVICE_OBJECT DeviceObject;
//     PIO_WORKITEM WorkItem;
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     ULONG SysButton;
// } SYS_BUTTON_CONTEXT, *PSYS_BUTTON_CONTEXT;
//
// static VOID
// NTAPI
// PopGetSysButton(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PVOID Context);
//
// PKWIN32_POWEREVENT_CALLOUT PopEventCallout;
// extern PCALLBACK_OBJECT SetSystemTimeCallback;
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// PoNotifySystemTimeSet(VOID)
// {
//     KIRQL OldIrql;
//
//     /* Check if Win32k registered a notification callback */
//     if (PopEventCallout)
//     {
//         /* Raise to dispatch */
//         KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//
//         /* Notify the callback */
//         ExNotifyCallback(SetSystemTimeCallback, NULL, NULL);
//
//         /* Lower IRQL back */
//         KeLowerIrql(OldIrql);
//     }
// }
//
// static NTSTATUS
// NTAPI
// PopGetSysButtonCompletion(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN PVOID Context)
// {
//     PSYS_BUTTON_CONTEXT SysButtonContext = Context;
//     ULONG SysButton;
//
//     /* The DeviceObject can be NULL, so use the one we stored */
//     DeviceObject = SysButtonContext->DeviceObject;
//
//     /* FIXME: What do do with the sys button event? */
//     SysButton = *(PULONG)Irp->AssociatedIrp.SystemBuffer;
//     {
//         DPRINT1("A device reported the event 0x%x (", SysButton);
//         if (SysButton & SYS_BUTTON_POWER) DbgPrint(" POWER");
//         if (SysButton & SYS_BUTTON_SLEEP) DbgPrint(" SLEEP");
//         if (SysButton & SYS_BUTTON_LID) DbgPrint(" LID");
//         if (SysButton == 0) DbgPrint(" WAKE");
//         DbgPrint(" )\n");
//
//         if (SysButton & SYS_BUTTON_POWER)
//         {
//             /* FIXME: Read registry for the action we should perform here */
//             DPRINT1("Initiating shutdown after power button event\n");
//
//             ZwShutdownSystem(ShutdownNoReboot);
//         }
//     }
//
//     /* Allocate a new workitem to send the next IOCTL_GET_SYS_BUTTON_EVENT */
//     SysButtonContext->WorkItem = IoAllocateWorkItem(DeviceObject);
//     if (!SysButtonContext->WorkItem)
//     {
//         DPRINT("IoAllocateWorkItem() failed\n");
//         ExFreePoolWithTag(SysButtonContext, 'IWOP');
//         return STATUS_SUCCESS;
//     }
//     IoQueueWorkItem(SysButtonContext->WorkItem,
//                     PopGetSysButton,
//                     DelayedWorkQueue,
//                     SysButtonContext);
//
//     return STATUS_SUCCESS /* STATUS_CONTINUE_COMPLETION */;
// }
//
// static VOID
// NTAPI
// PopGetSysButton(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PVOID Context)
// {
//     PSYS_BUTTON_CONTEXT SysButtonContext = Context;
//     PIO_WORKITEM CurrentWorkItem = SysButtonContext->WorkItem;
//     PIRP Irp;
//
//     /* Get button pressed (IOCTL_GET_SYS_BUTTON_EVENT) */
//     KeInitializeEvent(&SysButtonContext->Event, NotificationEvent, FALSE);
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_GET_SYS_BUTTON_EVENT,
//                                         DeviceObject,
//                                         NULL,
//                                         0,
//                                         &SysButtonContext->SysButton,
//                                         sizeof(SysButtonContext->SysButton),
//                                         FALSE,
//                                         &SysButtonContext->Event,
//                                         &SysButtonContext->IoStatusBlock);
//     if (Irp)
//     {
//         IoSetCompletionRoutine(Irp,
//                                PopGetSysButtonCompletion,
//                                SysButtonContext,
//                                TRUE,
//                                FALSE,
//                                FALSE);
//         IoCallDriver(DeviceObject, Irp);
//     }
//     else
//     {
//         DPRINT1("IoBuildDeviceIoControlRequest() failed\n");
//         ExFreePoolWithTag(SysButtonContext, 'IWOP');
//     }
//
//     IoFreeWorkItem(CurrentWorkItem);
// }
//
// NTSTATUS
// NTAPI
// PopAddRemoveSysCapsCallback(IN PVOID NotificationStructure,
//                             IN PVOID Context)
// {
//     PDEVICE_INTERFACE_CHANGE_NOTIFICATION Notification;
//     PSYS_BUTTON_CONTEXT SysButtonContext;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE FileHandle;
//     PDEVICE_OBJECT DeviceObject;
//     PFILE_OBJECT FileObject;
//     PIRP Irp;
//     IO_STATUS_BLOCK IoStatusBlock;
//     KEVENT Event;
//     BOOLEAN Arrival;
//     ULONG Caps;
//     NTSTATUS Status;
//     POP_POLICY_DEVICE_TYPE DeviceType = (POP_POLICY_DEVICE_TYPE)(ULONG_PTR)Context;
//
//     DPRINT("PopAddRemoveSysCapsCallback(%p %p)\n",
//         NotificationStructure, Context);
//
//     Notification = (PDEVICE_INTERFACE_CHANGE_NOTIFICATION)NotificationStructure;
//     if (Notification->Version != 1)
//         return STATUS_REVISION_MISMATCH;
//     if (Notification->Size != sizeof(DEVICE_INTERFACE_CHANGE_NOTIFICATION))
//         return STATUS_INVALID_PARAMETER;
//     if (RtlCompareMemory(&Notification->Event, &GUID_DEVICE_INTERFACE_ARRIVAL, sizeof(GUID)) == sizeof(GUID))
//         Arrival = TRUE;
//     else if (RtlCompareMemory(&Notification->Event, &GUID_DEVICE_INTERFACE_REMOVAL, sizeof(GUID)) == sizeof(GUID))
//         Arrival = FALSE;
//     else
//         return STATUS_INVALID_PARAMETER;
//
//     if (Arrival && DeviceType == PolicyDeviceBattery)
//     {
//         PopCapabilities.SystemBatteriesPresent = TRUE;
//         return STATUS_SUCCESS;
//     }
//
//     if (Arrival)
//     {
//         DPRINT("Arrival of %wZ\n", Notification->SymbolicLinkName);
//
//         /* Open the device */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    Notification->SymbolicLinkName,
//                                    OBJ_KERNEL_HANDLE,
//                                    NULL,
//                                    NULL);
//         Status = ZwOpenFile(&FileHandle,
//                             FILE_READ_DATA,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE,
//                             0);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("ZwOpenFile() failed with status 0x%08lx\n", Status);
//             return Status;
//         }
//         Status = ObReferenceObjectByHandle(FileHandle,
//                                            FILE_READ_DATA,
//                                            IoFileObjectType,
//                                            KernelMode,
//                                            (PVOID*)&FileObject,
//                                            NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("ObReferenceObjectByHandle() failed with status 0x%08lx\n", Status);
//             ZwClose(FileHandle);
//             return Status;
//         }
//         DeviceObject = IoGetRelatedDeviceObject(FileObject);
//         ObDereferenceObject(FileObject);
//
//         /* Get capabilities (IOCTL_GET_SYS_BUTTON_CAPS) */
//         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//         Irp = IoBuildDeviceIoControlRequest(IOCTL_GET_SYS_BUTTON_CAPS,
//                                             DeviceObject,
//                                             NULL,
//                                             0,
//                                             &Caps,
//                                             sizeof(Caps),
//                                             FALSE,
//                                             &Event,
//                                             &IoStatusBlock);
//         if (!Irp)
//         {
//             DPRINT1("IoBuildDeviceIoControlRequest() failed\n");
//             ZwClose(FileHandle);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//         Status = IoCallDriver(DeviceObject, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             DPRINT("IOCTL_GET_SYS_BUTTON_CAPS pending\n");
//             KeWaitForSingleObject(&Event, Suspended, KernelMode, FALSE, NULL);
//             Status = IoStatusBlock.Status;
//         }
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Sending IOCTL_GET_SYS_BUTTON_CAPS failed with status 0x%08x\n", Status);
//             ZwClose(FileHandle);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         DPRINT("Device capabilities: 0x%x\n", Caps);
//         if (Caps & SYS_BUTTON_POWER)
//         {
//             DPRINT("POWER button present\n");
//             PopCapabilities.PowerButtonPresent = TRUE;
//         }
//
//         if (Caps & SYS_BUTTON_SLEEP)
//         {
//             DPRINT("SLEEP button present\n");
//             PopCapabilities.SleepButtonPresent = TRUE;
//         }
//
//         if (Caps & SYS_BUTTON_LID)
//         {
//             DPRINT("LID present\n");
//             PopCapabilities.LidPresent = TRUE;
//         }
//
//         SysButtonContext = ExAllocatePoolWithTag(NonPagedPool,
//                                                  sizeof(SYS_BUTTON_CONTEXT),
//                                                  'IWOP');
//         if (!SysButtonContext)
//         {
//             DPRINT1("ExAllocatePoolWithTag() failed\n");
//             ZwClose(FileHandle);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Queue a work item to get sys button event */
//         SysButtonContext->WorkItem = IoAllocateWorkItem(DeviceObject);
//         SysButtonContext->DeviceObject = DeviceObject;
//         if (!SysButtonContext->WorkItem)
//         {
//             DPRINT1("IoAllocateWorkItem() failed\n");
//             ZwClose(FileHandle);
//             ExFreePoolWithTag(SysButtonContext, 'IWOP');
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//         IoQueueWorkItem(SysButtonContext->WorkItem,
//                         PopGetSysButton,
//                         DelayedWorkQueue,
//                         SysButtonContext);
//
//         ZwClose(FileHandle);
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         DPRINT1("Removal of a power capable device not implemented\n");
//         return STATUS_NOT_IMPLEMENTED;
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
