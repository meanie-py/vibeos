// types.rs — hidport.h types defined locally + the HID_USB_DEVICE_EXTENSION /
//            HID_USB_RESET_CONTEXT structs from drivers/hid/hidusb/hidusb.h.
//
// The HID minidriver port types (HID_MINIDRIVER_REGISTRATION, HID_DEVICE_EXTENSION,
// HID_DEVICE_ATTRIBUTES, HID_DESCRIPTOR, ...) live in sdk/include/ddk/hidport.h +
// hidclass.h and are NOT bindgen'd into any -sys crate, so they are defined here
// with their exact C layouts (CONTRACT2).

use ntkrnl_sys::base::PVOID;
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP, PIO_WORKITEM, KEVENT,
    PDRIVER_OBJECT, PUNICODE_STRING,
};
use usb_sys::{
    PUSB_DEVICE_DESCRIPTOR, PUSB_CONFIGURATION_DESCRIPTOR,
    PUSBD_INTERFACE_INFORMATION, USBD_CONFIGURATION_HANDLE,
};

// ============================================================================
// hidport.h — HID port-driver type surface (defined locally; NOT in -sys crates)
// ============================================================================

#[repr(C)]
pub struct HID_MINIDRIVER_REGISTRATION {
    pub Revision: u32,
    pub DriverObject: PDRIVER_OBJECT,
    pub RegistryPath: PUNICODE_STRING,
    pub DeviceExtensionSize: u32,
    pub DevicesArePolled: u8,
    pub Reserved: [u8; 3],
}
pub type PHID_MINIDRIVER_REGISTRATION = *mut HID_MINIDRIVER_REGISTRATION;

#[repr(C)]
pub struct HID_DEVICE_EXTENSION {
    pub PhysicalDeviceObject: PDEVICE_OBJECT,
    pub NextDeviceObject: PDEVICE_OBJECT,
    pub MiniDeviceExtension: PVOID,
}
pub type PHID_DEVICE_EXTENSION = *mut HID_DEVICE_EXTENSION;

#[repr(C)]
pub struct HID_DEVICE_ATTRIBUTES {
    pub Size: u32,
    pub VendorID: u16,
    pub ProductID: u16,
    pub VersionNumber: u16,
    pub Reserved: [u16; 11],
}
pub type PHID_DEVICE_ATTRIBUTES = *mut HID_DEVICE_ATTRIBUTES;

#[repr(C, packed)]
pub struct HID_DESCRIPTOR_DESC_LIST {
    pub bReportType: u8,
    pub wReportLength: u16,
}

#[repr(C, packed)]
pub struct HID_DESCRIPTOR {
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bcdHID: u16,
    pub bCountry: u8,
    pub bNumDescriptors: u8,
    pub DescriptorList: [HID_DESCRIPTOR_DESC_LIST; 1],
}
pub type PHID_DESCRIPTOR = *mut HID_DESCRIPTOR;

#[repr(C)]
pub struct HID_COLLECTION_INFORMATION {
    pub DescriptorSize: u32,
    pub Polled: u8,
    pub Reserved1: [u8; 1],
    pub VendorID: u16,
    pub ProductID: u16,
    pub VersionNumber: u16,
}
pub type PHID_COLLECTION_INFORMATION = *mut HID_COLLECTION_INFORMATION;

// ============================================================================
// hidusb.h — driver private device extension + reset context
// ============================================================================

#[repr(C)]
pub struct HID_USB_DEVICE_EXTENSION {
    //
    // event for completion
    //
    pub Event: KEVENT,

    //
    // device descriptor
    //
    pub DeviceDescriptor: PUSB_DEVICE_DESCRIPTOR,

    //
    // configuration descriptor
    //
    pub ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,

    //
    // interface information
    //
    pub InterfaceInfo: PUSBD_INTERFACE_INFORMATION,

    //
    // configuration handle
    //
    pub ConfigurationHandle: USBD_CONFIGURATION_HANDLE,

    //
    // hid descriptor
    //
    pub HidDescriptor: PHID_DESCRIPTOR,
}
pub type PHID_USB_DEVICE_EXTENSION = *mut HID_USB_DEVICE_EXTENSION;

#[repr(C)]
pub struct HID_USB_RESET_CONTEXT {
    //
    // request irp
    //
    pub Irp: PIRP,

    //
    // work item
    //
    pub WorkItem: PIO_WORKITEM,

    //
    // device object
    //
    pub DeviceObject: PDEVICE_OBJECT,
}
pub type PHID_USB_RESET_CONTEXT = *mut HID_USB_RESET_CONTEXT;
