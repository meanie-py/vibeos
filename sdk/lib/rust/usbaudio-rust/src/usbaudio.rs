use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExFreePool, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoBuildDeviceIoControlRequest, IoGetNextIrpStackLocation, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PCM_RESOURCE_LIST, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, POWER_ACTION, PUNICODE_STRING};
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;
use crate::cb;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
// Runtime-initialized: cb!(f) expands to transmute(f as usize) which is invalid in const eval.
static mut KsDeviceDispatch: KSDEVICE_DISPATCH = unsafe { core::mem::zeroed() };

static mut KsDeviceDescriptor: KSDEVICE_DESCRIPTOR = _KSDEVICE_DESCRIPTOR {
    Dispatch: core::ptr::null::<KSDEVICE_DISPATCH>(),
    FilterDescriptorsCount: 0 as ULONG,
    FilterDescriptors: ::core::ptr::null::<*const KSFILTER_DESCRIPTOR>(),
    Version: 0x100 as ULONG,
    Flags: 0 as ULONG,
};
#[no_mangle]
pub unsafe extern "C" fn SubmitUrbSync(
    DeviceObject: PDEVICE_OBJECT,
    Urb: PURB,
) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);

    let Irp = IoBuildDeviceIoControlRequest(
        IOCTL_INTERNAL_USB_SUBMIT_URB,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        1u8,
        &mut Event,
        &mut IoStatus,
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).Parameters.Others.Argument1 = Urb as *mut _;

    Status = IoCallDriver(DeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut _ as *mut _,
            _KWAIT_REASON_Executive,
            0i32 as i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = IoStatus.__bindgen_anon_1.Status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioSelectConfiguration(
    Device: PKSDEVICE,
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
) -> NTSTATUS {
    let mut DeviceExtension: PDEVICE_EXTENSION;
    let mut InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR;
    let mut InterfaceList: PUSBD_INTERFACE_LIST_ENTRY;
    let mut Urb: PURB;
    let mut Status: NTSTATUS;
    let mut InterfaceDescriptorCount: ULONG;

    InterfaceList = AllocFunction(
        (core::mem::size_of::<USBD_INTERFACE_LIST_ENTRY>() as ULONG)
            .wrapping_mul((*ConfigurationDescriptor).bNumInterfaces as ULONG + 1),
    ) as PUSBD_INTERFACE_LIST_ENTRY;
    if InterfaceList.is_null() {
        return USBD_STATUS_INSUFFICIENT_RESOURCES as NTSTATUS;
    }

    InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(
        ConfigurationDescriptor,
        ConfigurationDescriptor as PVOID,
        -1i32 as LONG,
        -1i32 as LONG,
        USB_DEVICE_CLASS_AUDIO as LONG,
        -1i32 as LONG,
        -1i32 as LONG,
    );
    if InterfaceDescriptor.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    InterfaceDescriptorCount = 0;
    while !InterfaceDescriptor.is_null() {
        if (*InterfaceDescriptor).bInterfaceSubClass == 0x01 {
            let idx = InterfaceDescriptorCount;
            InterfaceDescriptorCount = InterfaceDescriptorCount.wrapping_add(1);
            (*InterfaceList.offset(idx as isize)).InterfaceDescriptor = InterfaceDescriptor;
        } else if (*InterfaceDescriptor).bInterfaceSubClass == 0x03 {
            let idx = InterfaceDescriptorCount;
            InterfaceDescriptorCount = InterfaceDescriptorCount.wrapping_add(1);
            (*InterfaceList.offset(idx as isize)).InterfaceDescriptor = InterfaceDescriptor;
        }

        InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(
            ConfigurationDescriptor,
            (InterfaceDescriptor as ULONG_PTR)
                .wrapping_add((*InterfaceDescriptor).bLength as ULONG_PTR) as PVOID,
            -1i32 as LONG,
            -1i32 as LONG,
            USB_DEVICE_CLASS_AUDIO as LONG,
            -1i32 as LONG,
            -1i32 as LONG,
        );
    }

    Urb = USBD_CreateConfigurationRequestEx(ConfigurationDescriptor, InterfaceList);
    if Urb.is_null() {
        FreeFunction(InterfaceList as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DeviceExtension = (*Device).Context as PDEVICE_EXTENSION;

    Status = SubmitUrbSync((*DeviceExtension).LowerDevice, Urb);
    if !NT_SUCCESS(Status) {
        ExFreePool(Urb as PVOID);
        FreeFunction(InterfaceList as PVOID);
        return Status;
    }

    (*DeviceExtension).ConfigurationHandle = (*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref().ConfigurationHandle;

    (*DeviceExtension).InterfaceInfo = AllocFunction(
        (*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref().Interface.Length as ULONG,
    ) as PUSBD_INTERFACE_INFORMATION;
    if !(*DeviceExtension).InterfaceInfo.is_null() {
        core::ptr::copy_nonoverlapping(
            &raw const (*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref().Interface as *const u8,
            (*DeviceExtension).InterfaceInfo as *mut u8,
            (*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref().Interface.Length as usize,
        );
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioStartDevice(
    Device: PKSDEVICE,
) -> NTSTATUS {
    let mut Urb: PURB = core::ptr::null_mut();
    let mut DeviceDescriptor: PUSB_DEVICE_DESCRIPTOR = core::ptr::null_mut();
    let mut ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR = core::ptr::null_mut();
    let DeviceExtension: PDEVICE_EXTENSION = (*Device).Context as PDEVICE_EXTENSION;
    let mut Status: NTSTATUS = 0;
    let mut Length: ULONG = 0;

    Urb = AllocFunction(core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as ULONG) as PURB;
    if Urb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DeviceDescriptor = AllocFunction(core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() as ULONG) as PUSB_DEVICE_DESCRIPTOR;
    if DeviceDescriptor.is_null() {
        FreeFunction(Urb as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Urb).__bindgen_anon_1.UrbHeader.as_mut().Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE as USHORT;
    (*Urb).__bindgen_anon_1.UrbHeader.as_mut().Length = core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as USHORT;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().TransferBufferLength = core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() as ULONG;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().TransferBufferMDL = core::ptr::null_mut();
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().TransferBuffer = DeviceDescriptor as PVOID;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().DescriptorType = 0x1u8;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().Index = 0u8;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().LanguageId = 0u16;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().UrbLink = core::ptr::null_mut();

    Status = SubmitUrbSync((*DeviceExtension).LowerDevice, Urb);
    if !NT_SUCCESS(Status) {
        FreeFunction(Urb as PVOID);
        FreeFunction(DeviceDescriptor as PVOID);
        return Status;
    }

    ConfigurationDescriptor = AllocFunction(core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>() as ULONG) as PUSB_CONFIGURATION_DESCRIPTOR;
    if ConfigurationDescriptor.is_null() {
        FreeFunction(Urb as PVOID);
        FreeFunction(DeviceDescriptor as PVOID);
        return Status;
    }

    (*Urb).__bindgen_anon_1.UrbHeader.as_mut().Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE as USHORT;
    (*Urb).__bindgen_anon_1.UrbHeader.as_mut().Length = core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as USHORT;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().TransferBufferLength = core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>() as ULONG;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().TransferBufferMDL = core::ptr::null_mut();
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().TransferBuffer = ConfigurationDescriptor as PVOID;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().DescriptorType = 0x2u8;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().Index = 0u8;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().LanguageId = 0u16;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().UrbLink = core::ptr::null_mut();

    Status = SubmitUrbSync((*DeviceExtension).LowerDevice, Urb);
    if !NT_SUCCESS(Status) {
        FreeFunction(Urb as PVOID);
        FreeFunction(DeviceDescriptor as PVOID);
        FreeFunction(ConfigurationDescriptor as PVOID);
        return Status;
    }

    Length = (*ConfigurationDescriptor).wTotalLength as ULONG;
    FreeFunction(ConfigurationDescriptor as PVOID);

    ConfigurationDescriptor = AllocFunction(Length) as PUSB_CONFIGURATION_DESCRIPTOR;
    if ConfigurationDescriptor.is_null() {
        FreeFunction(Urb as PVOID);
        FreeFunction(DeviceDescriptor as PVOID);
        return Status;
    }

    (*Urb).__bindgen_anon_1.UrbHeader.as_mut().Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE as USHORT;
    (*Urb).__bindgen_anon_1.UrbHeader.as_mut().Length = core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as USHORT;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().TransferBufferLength = Length;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().TransferBufferMDL = core::ptr::null_mut();
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().TransferBuffer = ConfigurationDescriptor as PVOID;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().DescriptorType = 0x2u8;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().Index = 0u8;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().LanguageId = 0u16;
    (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut().UrbLink = core::ptr::null_mut();

    Status = SubmitUrbSync((*DeviceExtension).LowerDevice, Urb);
    FreeFunction(Urb as PVOID);
    if !NT_SUCCESS(Status) {
        FreeFunction(DeviceDescriptor as PVOID);
        FreeFunction(ConfigurationDescriptor as PVOID);
        return Status;
    }

    KsAddItemToObjectBag(
        (*Device).Bag,
        DeviceDescriptor as PVOID,
        cb!(ExFreePool),
    );
    KsAddItemToObjectBag(
        (*Device).Bag,
        ConfigurationDescriptor as PVOID,
        cb!(ExFreePool),
    );

    Status = USBAudioSelectConfiguration(Device, ConfigurationDescriptor);
    if NT_SUCCESS(Status) {
        (*DeviceExtension).ConfigurationDescriptor = ConfigurationDescriptor;
        (*DeviceExtension).DeviceDescriptor = DeviceDescriptor;
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioAddDevice(
    Device: PKSDEVICE,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPnPStart(
    Device: PKSDEVICE,
    _Irp: PIRP,
    _TranslatedResourceList: PCM_RESOURCE_LIST,
    _UntranslatedResourceList: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut DeviceExtension: PDEVICE_EXTENSION;

    if (*Device).Started == 0 {
        DeviceExtension = AllocFunction(core::mem::size_of::<DEVICE_EXTENSION>() as ULONG) as PDEVICE_EXTENSION;
        if DeviceExtension.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*Device).Context = DeviceExtension as PVOID;
        (*DeviceExtension).LowerDevice = (*Device).NextDeviceObject;

        KsAddItemToObjectBag(
            (*Device).Bag,
            (*Device).Context,
            cb!(ExFreePool),
        );

        Status = USBAudioStartDevice(Device);
        if NT_SUCCESS(Status) {
            Status = USBAudioCreateFilterContext(Device);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPnPQueryStop(
    Device: PKSDEVICE,
    Irp: PIRP,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPnPCancelStop(
    Device: PKSDEVICE,
    Irp: PIRP,
) {
    /* no op */
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPnPStop(
    Device: PKSDEVICE,
    Irp: PIRP,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"USBAudioPnPStop\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/usb/usbaudio/usbaudio.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        361i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPnPQueryRemove(
    Device: PKSDEVICE,
    Irp: PIRP,
) -> NTSTATUS {
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn USBAudioPnPCancelRemove(
    Device: PKSDEVICE,
    Irp: PIRP,
) {
    /* no op */
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPnPRemove(
    Device: PKSDEVICE,
    Irp: PIRP,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"USBAudioPnPRemove\0".as_ptr() as *const i8,
        b"drivers/usb/usbaudio/usbaudio.c\0".as_ptr() as *const i8,
        391i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPnPQueryCapabilities(
    Device: PKSDEVICE,
    Irp: PIRP,
    Capabilities: PDEVICE_CAPABILITIES,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPnPSurpriseRemoval(
    Device: PKSDEVICE,
    Irp: PIRP,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"USBAudioPnPSurpriseRemoval\0".as_ptr() as PCHAR,
        b"drivers/usb/usbaudio/usbaudio.c\0"
            .as_ptr() as PCHAR,
        413i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPnPQueryPower(
    Device: PKSDEVICE,
    Irp: PIRP,
    DeviceTo: DEVICE_POWER_STATE,
    DeviceFrom: DEVICE_POWER_STATE,
    SystemTo: SYSTEM_POWER_STATE,
    SystemFrom: SYSTEM_POWER_STATE,
    Action: POWER_ACTION,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPnPSetPower(
    Device: PKSDEVICE,
    Irp: PIRP,
    To: DEVICE_POWER_STATE,
    From: DEVICE_POWER_STATE,
) {
    /* TODO: stop streams */
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"USBAudioPnPSetPower\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/usb/usbaudio/usbaudio.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        440i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    // Runtime init: cb!(f) = transmute(f as usize) is invalid in const eval,
    // so we wire the dispatch table here. On i386, extern "system" == extern "system"
    // at the machine level; transmute bypasses the ABI-string type distinction.
    KsDeviceDispatch = core::mem::zeroed();
    KsDeviceDispatch.Add = core::mem::transmute(USBAudioAddDevice as unsafe extern "system" fn(PKSDEVICE) -> NTSTATUS);
    KsDeviceDispatch.Start = core::mem::transmute(USBAudioPnPStart as unsafe extern "system" fn(PKSDEVICE, PIRP, PCM_RESOURCE_LIST, PCM_RESOURCE_LIST) -> NTSTATUS);
    KsDeviceDispatch.PostStart = None;
    KsDeviceDispatch.QueryStop = core::mem::transmute(USBAudioPnPQueryStop as unsafe extern "system" fn(PKSDEVICE, PIRP) -> NTSTATUS);
    KsDeviceDispatch.CancelStop = core::mem::transmute(USBAudioPnPCancelStop as unsafe extern "system" fn(PKSDEVICE, PIRP));
    KsDeviceDispatch.Stop = core::mem::transmute(USBAudioPnPStop as unsafe extern "system" fn(PKSDEVICE, PIRP));
    KsDeviceDispatch.QueryRemove = core::mem::transmute(USBAudioPnPQueryRemove as unsafe extern "system" fn(PKSDEVICE, PIRP) -> NTSTATUS);
    KsDeviceDispatch.CancelRemove = core::mem::transmute(USBAudioPnPCancelRemove as unsafe extern "system" fn(PKSDEVICE, PIRP));
    KsDeviceDispatch.Remove = core::mem::transmute(USBAudioPnPRemove as unsafe extern "system" fn(PKSDEVICE, PIRP));
    KsDeviceDispatch.QueryCapabilities = core::mem::transmute(USBAudioPnPQueryCapabilities as unsafe extern "system" fn(PKSDEVICE, PIRP, PDEVICE_CAPABILITIES) -> NTSTATUS);
    KsDeviceDispatch.SurpriseRemoval = core::mem::transmute(USBAudioPnPSurpriseRemoval as unsafe extern "system" fn(PKSDEVICE, PIRP));
    KsDeviceDispatch.QueryPower = core::mem::transmute(USBAudioPnPQueryPower as unsafe extern "system" fn(PKSDEVICE, PIRP, DEVICE_POWER_STATE, DEVICE_POWER_STATE, SYSTEM_POWER_STATE, SYSTEM_POWER_STATE, POWER_ACTION) -> NTSTATUS);
    KsDeviceDispatch.SetPower = core::mem::transmute(USBAudioPnPSetPower as unsafe extern "system" fn(PKSDEVICE, PIRP, DEVICE_POWER_STATE, DEVICE_POWER_STATE));
    KsDeviceDispatch.QueryInterface = None;
    KsDeviceDescriptor.Dispatch = &raw const KsDeviceDispatch as *const KSDEVICE_DISPATCH;

    let Status = KsInitializeDriver(DriverObject, RegistryPath, &mut KsDeviceDescriptor);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:     ReactOS Universal Audio Class Driver
// * LICENSE:     GPL - See COPYING in the top level directory
// * FILE:        drivers/usb/usbaudio/usbaudio.c
// * PURPOSE:     USB Audio device driver.
// * PROGRAMMERS:
// *              Johannes Anderwald (johannes.anderwald@reactos.org)
// */
//
// #include "usbaudio.h"
//
// static KSDEVICE_DISPATCH KsDeviceDispatch = {
//     USBAudioAddDevice,
//     USBAudioPnPStart,
//     NULL,
//     USBAudioPnPQueryStop,
//     USBAudioPnPCancelStop,
//     USBAudioPnPStop,
//     USBAudioPnPQueryRemove,
//     USBAudioPnPCancelRemove,
//     USBAudioPnPRemove,
//     USBAudioPnPQueryCapabilities,
//     USBAudioPnPSurpriseRemoval,
//     USBAudioPnPQueryPower,
//     USBAudioPnPSetPower
// };
//
// static KSDEVICE_DESCRIPTOR KsDeviceDescriptor = {
//     &KsDeviceDispatch,
//     0,
//     NULL,
//     0x100, //KSDEVICE_DESCRIPTOR_VERSION,
//     0
// };
//
// NTSTATUS
// SubmitUrbSync(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PURB Urb)
// {
//     PIRP Irp;
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatus;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//
//     // init event
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     // build irp
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_INTERNAL_USB_SUBMIT_URB,
//         DeviceObject,
//         NULL,
//         0,
//         NULL,
//         0,
//         TRUE,
//         &Event,
//         &IoStatus);
//
//     if (!Irp)
//     {
//         //
//         // no memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // get next stack location
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     // store urb
//     IoStack->Parameters.Others.Argument1 = Urb;
//
//     // call driver
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     // wait for the request to finish
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatus.Status;
//     }
//
//     // done
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBAudioSelectConfiguration(
//     IN PKSDEVICE Device,
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor)
// {
//     PDEVICE_EXTENSION DeviceExtension;
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor;
//     PUSBD_INTERFACE_LIST_ENTRY InterfaceList;
//     PURB Urb;
//     NTSTATUS Status;
//     ULONG InterfaceDescriptorCount;
//
//     /* alloc item for configuration request */
//     InterfaceList = AllocFunction(sizeof(USBD_INTERFACE_LIST_ENTRY) * (ConfigurationDescriptor->bNumInterfaces + 1));
//     if (!InterfaceList)
//     {
//         /* insufficient resources*/
//         return USBD_STATUS_INSUFFICIENT_RESOURCES as NTSTATUS;
//     }
//
//     /* grab interface descriptor */
//     InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, ConfigurationDescriptor, -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1);
//     if (!InterfaceDescriptor)
//     {
//         /* no such interface */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* lets enumerate the interfaces */
//     InterfaceDescriptorCount = 0;
//     while (InterfaceDescriptor != NULL)
//     {
//         if (InterfaceDescriptor->bInterfaceSubClass == 0x01) /* AUDIO_CONTROL*/
//         {
//             InterfaceList[InterfaceDescriptorCount++].InterfaceDescriptor = InterfaceDescriptor;
//         }
//         else if (InterfaceDescriptor->bInterfaceSubClass == 0x03) /* MIDI_STREAMING*/
//         {
//             InterfaceList[InterfaceDescriptorCount++].InterfaceDescriptor = InterfaceDescriptor;
//         }
//
//         InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, (PVOID)((ULONG_PTR)InterfaceDescriptor + InterfaceDescriptor->bLength), -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1);
//     }
//
//     /* build urb */
//     Urb = USBD_CreateConfigurationRequestEx(ConfigurationDescriptor, InterfaceList);
//     if (!Urb)
//     {
//         /* no memory */
//         FreeFunction(InterfaceList);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* device extension */
//     DeviceExtension = Device->Context;
//
//     /* submit configuration urb */
//     Status = SubmitUrbSync(DeviceExtension->LowerDevice, Urb);
//     if (!NT_SUCCESS(Status))
//     {
//         /* free resources */
//         ExFreePool(Urb);
//         FreeFunction(InterfaceList);
//         return Status;
//     }
//
//     /* store configuration handle */
//     DeviceExtension->ConfigurationHandle = Urb->UrbSelectConfiguration.ConfigurationHandle;
//
//     /* alloc interface info */
//     DeviceExtension->InterfaceInfo = AllocFunction(Urb->UrbSelectConfiguration.Interface.Length);
//     if (DeviceExtension->InterfaceInfo)
//     {
//         /* copy interface info */
//         RtlCopyMemory(DeviceExtension->InterfaceInfo, &Urb->UrbSelectConfiguration.Interface, Urb->UrbSelectConfiguration.Interface.Length);
//     }
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBAudioStartDevice(
//     IN PKSDEVICE Device)
// {
//     PURB Urb;
//     PUSB_DEVICE_DESCRIPTOR DeviceDescriptor;
//     PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor;
//     PDEVICE_EXTENSION DeviceExtension;
//     NTSTATUS Status;
//     ULONG Length;
//
//     /* get device extension */
//     DeviceExtension = Device->Context;
//
//     /* allocate urb */
//     Urb = AllocFunction(sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST));
//     if (!Urb)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* alloc buffer for device descriptor */
//     DeviceDescriptor = AllocFunction(sizeof(USB_DEVICE_DESCRIPTOR));
//     if (!DeviceDescriptor)
//     {
//         /* insufficient resources */
//         FreeFunction(Urb);
//        return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* build descriptor request */
//     UsbBuildGetDescriptorRequest(Urb, sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST), USB_DEVICE_DESCRIPTOR_TYPE, 0, 0, DeviceDescriptor, NULL, sizeof(USB_DEVICE_DESCRIPTOR), NULL);
//
//     /* submit urb */
//     Status = SubmitUrbSync(DeviceExtension->LowerDevice, Urb);
//     if (!NT_SUCCESS(Status))
//     {
//         /* free resources */
//         FreeFunction(Urb);
//         FreeFunction(DeviceDescriptor);
//         return Status;
//     }
//
//     /* now allocate some space for partial configuration descriptor */
//     ConfigurationDescriptor = AllocFunction(sizeof(USB_CONFIGURATION_DESCRIPTOR));
//     if (!ConfigurationDescriptor)
//     {
//         /* free resources */
//         FreeFunction(Urb);
//         FreeFunction(DeviceDescriptor);
//         return Status;
//     }
//
//     /* build descriptor request */
//     UsbBuildGetDescriptorRequest(Urb, sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST), USB_CONFIGURATION_DESCRIPTOR_TYPE, 0, 0, ConfigurationDescriptor, NULL, sizeof(USB_CONFIGURATION_DESCRIPTOR), NULL);
//
//     /* submit urb */
//     Status = SubmitUrbSync(DeviceExtension->LowerDevice, Urb);
//     if (!NT_SUCCESS(Status))
//     {
//         /* free resources */
//         FreeFunction(Urb);
//         FreeFunction(DeviceDescriptor);
//         FreeFunction(ConfigurationDescriptor);
//         return Status;
//     }
//
//     /* backup length */
//     Length = ConfigurationDescriptor->wTotalLength;
//
//     /* free old descriptor */
//     FreeFunction(ConfigurationDescriptor);
//
//     /* now allocate some space for full configuration descriptor */
//     ConfigurationDescriptor = AllocFunction(Length);
//     if (!ConfigurationDescriptor)
//     {
//         /* free resources */
//         FreeFunction(Urb);
//         FreeFunction(DeviceDescriptor);
//         return Status;
//     }
//
//     /* build descriptor request */
//     UsbBuildGetDescriptorRequest(Urb, sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST), USB_CONFIGURATION_DESCRIPTOR_TYPE, 0, 0, ConfigurationDescriptor, NULL, Length, NULL);
//
//     /* submit urb */
//     Status = SubmitUrbSync(DeviceExtension->LowerDevice, Urb);
//
//     /* free urb */
//     FreeFunction(Urb);
//     if (!NT_SUCCESS(Status))
//     {
//         /* free resources */
//         FreeFunction(DeviceDescriptor);
//         FreeFunction(ConfigurationDescriptor);
//         return Status;
//     }
//
//     /* lets add to object bag */
//     KsAddItemToObjectBag(Device->Bag, DeviceDescriptor, ExFreePool);
//     KsAddItemToObjectBag(Device->Bag, ConfigurationDescriptor, ExFreePool);
//
//     Status = USBAudioSelectConfiguration(Device, ConfigurationDescriptor);
//     if (NT_SUCCESS(Status))
//     {
//
//         DeviceExtension->ConfigurationDescriptor = ConfigurationDescriptor;
//         DeviceExtension->DeviceDescriptor = DeviceDescriptor;
//     }
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// USBAudioAddDevice(
//    PKSDEVICE Device)
// {
//     /* no op */
//     DPRINT1("USBAudioAddDevice\n");
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBAudioPnPStart(
//        PKSDEVICE         Device,
//        PIRP              Irp,
//    PCM_RESOURCE_LIST TranslatedResourceList,
//    PCM_RESOURCE_LIST UntranslatedResourceList)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PDEVICE_EXTENSION DeviceExtension;
//
//     if (!Device->Started)
//     {
//         /* alloc context  */
//         DeviceExtension = AllocFunction(sizeof(DEVICE_EXTENSION));
//         if (DeviceExtension == NULL)
//         {
//              /* insufficient resources */
//              return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* init context */
//         Device->Context = DeviceExtension;
//         DeviceExtension->LowerDevice = Device->NextDeviceObject;
//
//         /* add to object bag*/
//         KsAddItemToObjectBag(Device->Bag, Device->Context, ExFreePool);
//
//         /* init device*/
//         Status = USBAudioStartDevice(Device);
//         if (NT_SUCCESS(Status))
//         {
//             /* TODO retrieve interface */
//             Status = USBAudioCreateFilterContext(Device);
//         }
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBAudioPnPQueryStop(
//    PKSDEVICE Device,
//    PIRP      Irp)
// {
//     /* no op */
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// USBAudioPnPCancelStop(
//    PKSDEVICE Device,
//    PIRP      Irp)
// {
//     /* no op */
// }
//
// VOID
// NTAPI
// USBAudioPnPStop(
//    PKSDEVICE Device,
//    PIRP      Irp)
// {
//     /* TODO: stop device */
// 	UNIMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// USBAudioPnPQueryRemove(
//    PKSDEVICE Device,
//    PIRP      Irp)
// {
//     /* no op */
//     return STATUS_SUCCESS;
// }
//
//
// VOID
// NTAPI
// USBAudioPnPCancelRemove(
//    PKSDEVICE Device,
//    PIRP      Irp)
// {
//     /* no op */
// }
//
// VOID
// NTAPI
// USBAudioPnPRemove(
//    PKSDEVICE Device,
//    PIRP      Irp)
// {
//     /* TODO: stop device */
// 	UNIMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// USBAudioPnPQueryCapabilities(
//       PKSDEVICE            Device,
//       PIRP                 Irp,
//    PDEVICE_CAPABILITIES Capabilities)
// {
//     /* TODO: set caps */
// 	UNIMPLEMENTED;
// 	return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// USBAudioPnPSurpriseRemoval(
//    PKSDEVICE Device,
//    PIRP      Irp)
// {
//     /* TODO: stop streams */
// 	UNIMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// USBAudioPnPQueryPower(
//    PKSDEVICE          Device,
//    PIRP               Irp,
//    DEVICE_POWER_STATE DeviceTo,
//    DEVICE_POWER_STATE DeviceFrom,
//    SYSTEM_POWER_STATE SystemTo,
//    SYSTEM_POWER_STATE SystemFrom,
//    POWER_ACTION       Action)
// {
//     /* no op */
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// USBAudioPnPSetPower(
//    PKSDEVICE          Device,
//    PIRP               Irp,
//    DEVICE_POWER_STATE To,
//    DEVICE_POWER_STATE From)
// {
//     /* TODO: stop streams */
// 	UNIMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// DriverEntry(
//     IN PDRIVER_OBJECT DriverObject,
//     IN PUNICODE_STRING RegistryPath)
// {
//     NTSTATUS Status;
//
//     // initialize driver
//     Status = KsInitializeDriver(DriverObject, RegistryPath, &KsDeviceDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         // failed to initialize driver
//         DPRINT1("Failed to initialize driver with %x\n", Status);
//         return Status;
//     }
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
