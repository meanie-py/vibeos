// pnp.rs — descriptor acquisition + configuration selection chain.
//
// Translated faithfully from drivers/hid/hidusb/hidusb.c:
//   Hid_GetDescriptor, Hid_SelectConfiguration, Hid_DisableConfiguration,
//   Hid_SetIdle, Hid_GetProtocol, Hid_PnpStart.

use core::mem::size_of;
use core::ptr::null_mut;

use ntkrnl_sys::base::{NTSTATUS, ULONG, USHORT, UCHAR, PVOID, TRUE, FALSE};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT,
    ExAllocatePoolWithTag, ExFreePoolWithTag,
    _POOL_TYPE_NonPagedPool as NonPagedPool,
};
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_UNSUCCESSFUL};
use ntkrnl_sys::rtl::{RtlZeroMemory, RtlCopyMemory};

use usb_sys::{
    PUSB_INTERFACE_DESCRIPTOR,
    USBD_INTERFACE_LIST_ENTRY,
    URB_FUNCTION_GET_DESCRIPTOR_FROM_INTERFACE,
    URB_FUNCTION_GET_DESCRIPTOR_FROM_ENDPOINT,
    URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE,
    URB_FUNCTION_CLASS_INTERFACE,
    USB_DEVICE_DESCRIPTOR_TYPE, USB_CONFIGURATION_DESCRIPTOR_TYPE,
    USB_INTERFACE_DESCRIPTOR_TYPE, USB_DEVICE_CLASS_HUMAN_INTERFACE,
    USBD_TRANSFER_DIRECTION_IN,
    _URB_CONTROL_DESCRIPTOR_REQUEST, _URB_SELECT_CONFIGURATION,
    _URB_CONTROL_VENDOR_OR_CLASS_REQUEST,
    _USB_INTERFACE_DESCRIPTOR,
    USB_DEVICE_DESCRIPTOR, USB_CONFIGURATION_DESCRIPTOR,
};

use crate::types::{
    PHID_DEVICE_EXTENSION, PHID_USB_DEVICE_EXTENSION, PHID_DESCRIPTOR, HID_DESCRIPTOR,
};
use crate::glue::{
    PURB,
    HIDUSB_TAG, HIDUSB_URB_TAG,
    USB_SET_IDLE_REQUEST, USB_GET_PROTOCOL_REQUEST,
    HID_HID_DESCRIPTOR_TYPE,
    USBD_SUCCESS,
    UsbBuildGetDescriptorRequest, UsbBuildVendorRequest,
    UsbBuildSelectConfigurationRequest,
    USBD_CreateConfigurationRequestEx, USBD_ParseConfigurationDescriptorEx,
};
use crate::urb::Hid_DispatchUrb;

// NT_SUCCESS(Status) == Status >= 0
#[inline]
fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

pub unsafe fn Hid_GetDescriptor(
    DeviceObject: PDEVICE_OBJECT,
    UrbFunction: USHORT,
    UrbLength: USHORT,
    UrbBuffer: *mut PVOID,
    UrbBufferLength: *mut ULONG,
    DescriptorType: UCHAR,
    Index: UCHAR,
    LanguageIndex: USHORT,
) -> NTSTATUS { unsafe {
    let Urb: PURB;
    let Status: NTSTATUS;
    let mut Allocated: UCHAR = FALSE;

    //
    // allocate urb
    //
    Urb = ExAllocatePoolWithTag(NonPagedPool, UrbLength as _, HIDUSB_URB_TAG) as PURB;
    if Urb.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // is there an urb buffer
    //
    if (*UrbBuffer).is_null() {
        //
        // allocate buffer
        //
        *UrbBuffer = ExAllocatePoolWithTag(NonPagedPool, *UrbBufferLength as _, HIDUSB_TAG);
        if (*UrbBuffer).is_null() {
            //
            // no memory
            //
            ExFreePoolWithTag(Urb as PVOID, HIDUSB_URB_TAG);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        //
        // zero buffer
        //
        RtlZeroMemory(*UrbBuffer, *UrbBufferLength as _);
        Allocated = TRUE;
    }

    //
    // zero urb
    //
    RtlZeroMemory(Urb as PVOID, UrbLength as _);

    //
    // build descriptor request
    //
    UsbBuildGetDescriptorRequest(
        Urb,
        UrbLength,
        DescriptorType,
        Index,
        LanguageIndex,
        *UrbBuffer,
        null_mut(),
        *UrbBufferLength,
        null_mut(),
    );

    //
    // set urb function
    //
    (*Urb).__bindgen_anon_1.UrbHeader.Function = UrbFunction;

    //
    // dispatch urb
    //
    Status = Hid_DispatchUrb(DeviceObject, Urb);

    //
    // did the request fail
    //
    if !nt_success(Status) {
        if Allocated != FALSE {
            //
            // free allocated buffer
            //
            ExFreePoolWithTag(*UrbBuffer, HIDUSB_TAG);
            *UrbBuffer = null_mut();
        }

        //
        // free urb
        //
        ExFreePoolWithTag(Urb as PVOID, HIDUSB_URB_TAG);
        *UrbBufferLength = 0;
        return Status;
    }

    //
    // did urb request fail
    //
    if !nt_success((*Urb).__bindgen_anon_1.UrbHeader.Status) {
        if Allocated != FALSE {
            //
            // free allocated buffer
            //
            ExFreePoolWithTag(*UrbBuffer, HIDUSB_TAG);
            *UrbBuffer = null_mut();
        }

        //
        // free urb
        //
        ExFreePoolWithTag(Urb as PVOID, HIDUSB_URB_TAG);
        *UrbBufferLength = 0;
        return STATUS_UNSUCCESSFUL;
    }

    //
    // store result length
    //
    *UrbBufferLength = (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.TransferBufferLength;

    //
    // free urb
    //
    ExFreePoolWithTag(Urb as PVOID, HIDUSB_URB_TAG);

    //
    // completed successfully
    //
    STATUS_SUCCESS
}}

pub unsafe fn Hid_SelectConfiguration(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS { unsafe {
    let InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR;
    let Status: NTSTATUS;
    let mut InterfaceList: [USBD_INTERFACE_LIST_ENTRY; 2] = core::mem::zeroed();
    let Urb: PURB;
    let HidDeviceExtension: PHID_USB_DEVICE_EXTENSION;
    let DeviceExtension: PHID_DEVICE_EXTENSION;

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;
    HidDeviceExtension = (*DeviceExtension).MiniDeviceExtension as PHID_USB_DEVICE_EXTENSION;

    //
    // now parse the descriptors
    //
    InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(
        (*HidDeviceExtension).ConfigurationDescriptor,
        (*HidDeviceExtension).ConfigurationDescriptor as PVOID,
        -1,
        -1,
        USB_DEVICE_CLASS_HUMAN_INTERFACE as i32,
        -1,
        -1,
    );
    if InterfaceDescriptor.is_null() {
        //
        // bogus configuration descriptor
        //
        return STATUS_INVALID_PARAMETER;
    }

    //
    // sanity check
    //
    debug_assert!(!InterfaceDescriptor.is_null());
    debug_assert!((*InterfaceDescriptor).bInterfaceClass == USB_DEVICE_CLASS_HUMAN_INTERFACE as UCHAR);
    debug_assert!((*InterfaceDescriptor).bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE as UCHAR);
    debug_assert!((*InterfaceDescriptor).bLength as usize == size_of::<_USB_INTERFACE_DESCRIPTOR>());

    //
    // setup interface list
    //
    RtlZeroMemory(InterfaceList.as_mut_ptr() as PVOID, size_of::<[USBD_INTERFACE_LIST_ENTRY; 2]>() as _);
    InterfaceList[0].InterfaceDescriptor = InterfaceDescriptor;

    //
    // build urb
    //
    Urb = USBD_CreateConfigurationRequestEx(
        (*HidDeviceExtension).ConfigurationDescriptor,
        InterfaceList.as_mut_ptr(),
    );
    if Urb.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // dispatch request
    //
    Status = Hid_DispatchUrb(DeviceObject, Urb);
    if nt_success(Status) {
        //
        // store configuration handle
        //
        (*HidDeviceExtension).ConfigurationHandle =
            (*Urb).__bindgen_anon_1.UrbSelectConfiguration.ConfigurationHandle;

        //
        // copy interface info
        //
        let iface_len = (*Urb).__bindgen_anon_1.UrbSelectConfiguration.Interface.Length;
        (*HidDeviceExtension).InterfaceInfo =
            ExAllocatePoolWithTag(NonPagedPool, iface_len as _, HIDUSB_TAG) as _;
        if !(*HidDeviceExtension).InterfaceInfo.is_null() {
            //
            // copy interface info
            //
            RtlCopyMemory(
                (*HidDeviceExtension).InterfaceInfo as PVOID,
                &(*Urb).__bindgen_anon_1.UrbSelectConfiguration.Interface as *const _ as PVOID,
                iface_len as _,
            );
        }
    }

    //
    // free urb request
    //
    ExFreePoolWithTag(Urb as PVOID, 0);

    //
    // done
    //
    Status
}}

pub unsafe fn Hid_DisableConfiguration(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS { unsafe {
    let DeviceExtension: PHID_DEVICE_EXTENSION;
    let HidDeviceExtension: PHID_USB_DEVICE_EXTENSION;
    let Status: NTSTATUS;
    let Urb: PURB;

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;
    HidDeviceExtension = (*DeviceExtension).MiniDeviceExtension as PHID_USB_DEVICE_EXTENSION;

    //
    // build urb
    //
    Urb = ExAllocatePoolWithTag(
        NonPagedPool,
        size_of::<_URB_SELECT_CONFIGURATION>() as _,
        HIDUSB_URB_TAG,
    ) as PURB;
    if Urb.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // format urb
    //
    UsbBuildSelectConfigurationRequest(
        Urb,
        size_of::<_URB_SELECT_CONFIGURATION>() as USHORT,
        null_mut(),
    );

    //
    // dispatch request
    //
    Status = Hid_DispatchUrb(DeviceObject, Urb);
    if !nt_success(Status) {
        // DPRINT1: Dispatching unconfigure URB failed
    } else if !USBD_SUCCESS((*Urb).__bindgen_anon_1.UrbHeader.Status) {
        // DPRINT: Unconfigure URB failed
    }

    //
    // free urb
    //
    ExFreePoolWithTag(Urb as PVOID, HIDUSB_URB_TAG);

    //
    // free resources
    //
    (*HidDeviceExtension).ConfigurationHandle = null_mut();

    if !(*HidDeviceExtension).InterfaceInfo.is_null() {
        ExFreePoolWithTag((*HidDeviceExtension).InterfaceInfo as PVOID, HIDUSB_TAG);
        (*HidDeviceExtension).InterfaceInfo = null_mut();
    }

    if !(*HidDeviceExtension).ConfigurationDescriptor.is_null() {
        ExFreePoolWithTag((*HidDeviceExtension).ConfigurationDescriptor as PVOID, HIDUSB_TAG);
        (*HidDeviceExtension).ConfigurationDescriptor = null_mut();
        (*HidDeviceExtension).HidDescriptor = null_mut();
    }

    if !(*HidDeviceExtension).DeviceDescriptor.is_null() {
        ExFreePoolWithTag((*HidDeviceExtension).DeviceDescriptor as PVOID, HIDUSB_TAG);
        (*HidDeviceExtension).DeviceDescriptor = null_mut();
    }

    //
    // done
    //
    Status
}}

pub unsafe fn Hid_SetIdle(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS { unsafe {
    let Urb: PURB;
    let Status: NTSTATUS;

    //
    // allocate urb
    //
    Urb = ExAllocatePoolWithTag(
        NonPagedPool,
        size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as _,
        HIDUSB_URB_TAG,
    ) as PURB;
    if Urb.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // zero urb
    //
    RtlZeroMemory(Urb as PVOID, size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as _);

    //
    // format urb
    //
    UsbBuildVendorRequest(
        Urb,
        URB_FUNCTION_CLASS_INTERFACE as USHORT,
        size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as USHORT,
        0,
        0,
        USB_SET_IDLE_REQUEST, // HID_SET_IDLE
        0,
        0,
        null_mut(),
        null_mut(),
        0,
        null_mut(),
    );

    //
    // dispatch urb
    //
    Status = Hid_DispatchUrb(DeviceObject, Urb);

    //
    // free urb
    //
    ExFreePoolWithTag(Urb as PVOID, HIDUSB_URB_TAG);

    //
    // print status
    //
    Status
}}

pub unsafe fn Hid_GetProtocol(DeviceObject: PDEVICE_OBJECT) { unsafe {
    let HidDeviceExtension: PHID_USB_DEVICE_EXTENSION;
    let DeviceExtension: PHID_DEVICE_EXTENSION;
    let Urb: PURB;
    let mut Protocol: [UCHAR; 1] = [0; 1];

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;
    HidDeviceExtension = (*DeviceExtension).MiniDeviceExtension as PHID_USB_DEVICE_EXTENSION;
    debug_assert!(!(*HidDeviceExtension).InterfaceInfo.is_null());

    if (*(*HidDeviceExtension).InterfaceInfo).SubClass != 0x1 {
        //
        // device does not support the boot protocol
        //
        return;
    }

    //
    // allocate urb
    //
    Urb = ExAllocatePoolWithTag(
        NonPagedPool,
        size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as _,
        HIDUSB_URB_TAG,
    ) as PURB;
    if Urb.is_null() {
        //
        // no memory
        //
        return;
    }

    //
    // zero urb
    //
    RtlZeroMemory(Urb as PVOID, size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as _);

    //
    // format urb
    //
    UsbBuildVendorRequest(
        Urb,
        URB_FUNCTION_CLASS_INTERFACE as USHORT,
        size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as USHORT,
        USBD_TRANSFER_DIRECTION_IN,
        0,
        USB_GET_PROTOCOL_REQUEST,
        0,
        0,
        Protocol.as_mut_ptr() as PVOID,
        null_mut(),
        1,
        null_mut(),
    );
    Protocol[0] = 0xFF;

    //
    // dispatch urb
    //
    Hid_DispatchUrb(DeviceObject, Urb);

    //
    // free urb
    //
    ExFreePoolWithTag(Urb as PVOID, HIDUSB_URB_TAG);

    //
    // boot protocol active 0x00 disabled 0x1
    //
    if Protocol[0] != 0x1 {
        if Protocol[0] == 0x00 {
            // DPRINT1: Need to disable boot protocol!
        } else {
            // DPRINT1: Unexpected protocol value
        }
    }
}}

pub unsafe fn Hid_PnpStart(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS { unsafe {
    let HidDeviceExtension: PHID_USB_DEVICE_EXTENSION;
    let DeviceExtension: PHID_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;
    let mut DescriptorLength: ULONG;
    let InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR;
    let HidDescriptor: PHID_DESCRIPTOR;

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;
    HidDeviceExtension = (*DeviceExtension).MiniDeviceExtension as PHID_USB_DEVICE_EXTENSION;

    //
    // get device descriptor
    //
    DescriptorLength = size_of::<USB_DEVICE_DESCRIPTOR>() as ULONG;
    Status = Hid_GetDescriptor(
        DeviceObject,
        URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE as USHORT,
        size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as USHORT,
        &mut (*HidDeviceExtension).DeviceDescriptor as *mut _ as *mut PVOID,
        &mut DescriptorLength,
        USB_DEVICE_DESCRIPTOR_TYPE as UCHAR,
        0,
        0,
    );
    if !nt_success(Status) {
        //
        // failed to obtain device descriptor
        //
        return Status;
    }

    //
    // now get the configuration descriptor
    //
    DescriptorLength = size_of::<USB_CONFIGURATION_DESCRIPTOR>() as ULONG;
    Status = Hid_GetDescriptor(
        DeviceObject,
        URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE as USHORT,
        size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as USHORT,
        &mut (*HidDeviceExtension).ConfigurationDescriptor as *mut _ as *mut PVOID,
        &mut DescriptorLength,
        USB_CONFIGURATION_DESCRIPTOR_TYPE as UCHAR,
        0,
        0,
    );
    if !nt_success(Status) {
        //
        // failed to obtain device descriptor
        //
        return Status;
    }

    //
    // sanity check
    //
    debug_assert!(DescriptorLength != 0);
    debug_assert!(!(*HidDeviceExtension).ConfigurationDescriptor.is_null());
    debug_assert!((*(*HidDeviceExtension).ConfigurationDescriptor).bLength != 0);

    //
    // store full length
    //
    DescriptorLength = (*(*HidDeviceExtension).ConfigurationDescriptor).wTotalLength as ULONG;

    //
    // delete partial configuration descriptor
    //
    ExFreePoolWithTag((*HidDeviceExtension).ConfigurationDescriptor as PVOID, HIDUSB_TAG);
    (*HidDeviceExtension).ConfigurationDescriptor = null_mut();

    //
    // get full configuration descriptor
    //
    Status = Hid_GetDescriptor(
        DeviceObject,
        URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE as USHORT,
        size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as USHORT,
        &mut (*HidDeviceExtension).ConfigurationDescriptor as *mut _ as *mut PVOID,
        &mut DescriptorLength,
        USB_CONFIGURATION_DESCRIPTOR_TYPE as UCHAR,
        0,
        0,
    );
    if !nt_success(Status) {
        //
        // failed to obtain device descriptor
        //
        return Status;
    }

    //
    // now parse the descriptors
    //
    InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(
        (*HidDeviceExtension).ConfigurationDescriptor,
        (*HidDeviceExtension).ConfigurationDescriptor as PVOID,
        -1,
        -1,
        USB_DEVICE_CLASS_HUMAN_INTERFACE as i32,
        -1,
        -1,
    );
    if InterfaceDescriptor.is_null() {
        //
        // no interface class
        //
        return STATUS_UNSUCCESSFUL;
    }

    //
    // sanity check
    //
    debug_assert!((*InterfaceDescriptor).bInterfaceClass == USB_DEVICE_CLASS_HUMAN_INTERFACE as UCHAR);
    debug_assert!((*InterfaceDescriptor).bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE as UCHAR);
    debug_assert!((*InterfaceDescriptor).bLength as usize == size_of::<_USB_INTERFACE_DESCRIPTOR>());

    //
    // move to next descriptor
    //
    HidDescriptor = ((InterfaceDescriptor as usize)
        + (*InterfaceDescriptor).bLength as usize) as PHID_DESCRIPTOR;
    debug_assert!((*HidDescriptor).bLength >= 2);

    //
    // check if this is the hid descriptor
    //
    if (*HidDescriptor).bLength as usize == size_of::<HID_DESCRIPTOR>()
        && (*HidDescriptor).bDescriptorType == HID_HID_DESCRIPTOR_TYPE
    {
        //
        // found
        //
        (*HidDeviceExtension).HidDescriptor = HidDescriptor;

        //
        // select configuration
        //
        Status = Hid_SelectConfiguration(DeviceObject);

        if nt_success(Status) {
            //
            // now set the device idle
            //
            Hid_SetIdle(DeviceObject);

            //
            // get protocol
            //
            Hid_GetProtocol(DeviceObject);
            return Status;
        }
    } else {
        //
        // FIXME parse hid descriptor / select configuration / set idle / get protocol
        //
        debug_assert!(false);
    }
    Status
}}
