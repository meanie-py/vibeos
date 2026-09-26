// types.rs — hidport.h / hidclass.h local types + the HIDCLASS_* device extensions.
// Translated faithfully from drivers/hid/hidclass/precomp.h + sdk/include/ddk/hidport.h.
//
// These structs are NOT in any -sys crate (hidport.h is not bindgen'd), so they are
// defined locally with exact #[repr(C)] layouts. DDK pointer types come from
// ntkrnl_sys::ddk; HID parser types from hid_sys.

use ntkrnl_sys::base::{ULONG, USHORT, UCHAR, BOOLEAN, PVOID};
use ntkrnl_sys::ddk::{
    PDRIVER_OBJECT, PUNICODE_STRING, PDEVICE_OBJECT, PDRIVER_DISPATCH, PDRIVER_ADD_DEVICE,
    PDRIVER_UNLOAD, KSPIN_LOCK, LIST_ENTRY, KEVENT, UNICODE_STRING, DEVICE_CAPABILITIES,
    PDEVICE_RELATIONS, PIRP, PIO_WORKITEM,
};
use hid_sys::HIDP_DEVICE_DESC;

// ===========================================================================
// USAGE / USAGE_AND_PAGE — USAGE_AND_PAGE is missing from the substrate.
// ===========================================================================
pub type USAGE = u16;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct USAGE_AND_PAGE {
    pub Usage: USAGE,
    pub UsagePage: USAGE,
}
pub type PUSAGE_AND_PAGE = *mut USAGE_AND_PAGE;
pub type PUSAGE = *mut USAGE;

// ===========================================================================
// hidport.h — HID_MINIDRIVER_REGISTRATION
// ===========================================================================
#[repr(C)]
pub struct HID_MINIDRIVER_REGISTRATION {
    pub Revision: ULONG,
    pub DriverObject: PDRIVER_OBJECT,
    pub RegistryPath: PUNICODE_STRING,
    pub DeviceExtensionSize: ULONG,
    pub DevicesArePolled: BOOLEAN,
    pub Reserved: [UCHAR; 3],
}
pub type PHID_MINIDRIVER_REGISTRATION = *mut HID_MINIDRIVER_REGISTRATION;

// ===========================================================================
// hidport.h — HID_DEVICE_EXTENSION
// ===========================================================================
#[repr(C)]
pub struct HID_DEVICE_EXTENSION {
    pub PhysicalDeviceObject: PDEVICE_OBJECT,
    pub NextDeviceObject: PDEVICE_OBJECT,
    pub MiniDeviceExtension: PVOID,
}
pub type PHID_DEVICE_EXTENSION = *mut HID_DEVICE_EXTENSION;

// ===========================================================================
// hidport.h — HID_DEVICE_ATTRIBUTES
// ===========================================================================
#[repr(C)]
#[derive(Copy, Clone)]
pub struct HID_DEVICE_ATTRIBUTES {
    pub Size: ULONG,
    pub VendorID: USHORT,
    pub ProductID: USHORT,
    pub VersionNumber: USHORT,
    pub Reserved: [USHORT; 11],
}
pub type PHID_DEVICE_ATTRIBUTES = *mut HID_DEVICE_ATTRIBUTES;

// ===========================================================================
// hidport.h — HID_DESCRIPTOR (packed; pshpack1) + DescriptorList element
// ===========================================================================
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct HID_DESCRIPTOR_DESC_LIST {
    pub bReportType: UCHAR,
    pub wReportLength: USHORT,
}
#[repr(C, packed)]
pub struct HID_DESCRIPTOR {
    pub bLength: UCHAR,
    pub bDescriptorType: UCHAR,
    pub bcdHID: USHORT,
    pub bCountry: UCHAR,
    pub bNumDescriptors: UCHAR,
    pub DescriptorList: [HID_DESCRIPTOR_DESC_LIST; 1],
}
pub type PHID_DESCRIPTOR = *mut HID_DESCRIPTOR;

// ===========================================================================
// hidclass.h — HID_COLLECTION_INFORMATION
// ===========================================================================
#[repr(C)]
pub struct HID_COLLECTION_INFORMATION {
    pub DescriptorSize: ULONG,
    pub Polled: BOOLEAN,
    pub Reserved1: [UCHAR; 1],
    pub VendorID: USHORT,
    pub ProductID: USHORT,
    pub VersionNumber: USHORT,
}
pub type PHID_COLLECTION_INFORMATION = *mut HID_COLLECTION_INFORMATION;

// ===========================================================================
// hidclass.h — HID_XFER_PACKET
// ===========================================================================
#[repr(C)]
pub struct HID_XFER_PACKET {
    pub reportBuffer: *mut UCHAR,
    pub reportBufferLen: ULONG,
    pub reportId: UCHAR,
}

// ===========================================================================
// precomp.h — HIDCLASS_DRIVER_EXTENSION
//   MajorFunction[IRP_MJ_MAXIMUM_FUNCTION + 1] = [PDRIVER_DISPATCH; 28]
// ===========================================================================
#[repr(C)]
pub struct HIDCLASS_DRIVER_EXTENSION {
    pub DriverObject: PDRIVER_OBJECT,
    pub DeviceExtensionSize: ULONG,
    pub DevicesArePolled: BOOLEAN,
    pub MajorFunction: [PDRIVER_DISPATCH; 28], // IRP_MJ_MAXIMUM_FUNCTION + 1
    pub AddDevice: PDRIVER_ADD_DEVICE,
    pub DriverUnload: PDRIVER_UNLOAD,
    pub Lock: KSPIN_LOCK,
}
pub type PHIDCLASS_DRIVER_EXTENSION = *mut HIDCLASS_DRIVER_EXTENSION;

// ===========================================================================
// precomp.h — HIDCLASS_COMMON_DEVICE_EXTENSION
// ===========================================================================
#[repr(C)]
pub struct HIDCLASS_COMMON_DEVICE_EXTENSION {
    pub HidDeviceExtension: HID_DEVICE_EXTENSION,
    pub IsFDO: BOOLEAN,
    pub DriverExtension: PHIDCLASS_DRIVER_EXTENSION,
    pub DeviceDescription: HIDP_DEVICE_DESC,
    pub Attributes: HID_DEVICE_ATTRIBUTES,
}
pub type PHIDCLASS_COMMON_DEVICE_EXTENSION = *mut HIDCLASS_COMMON_DEVICE_EXTENSION;

// ===========================================================================
// precomp.h — HIDCLASS_FDO_EXTENSION
// ===========================================================================
#[repr(C)]
pub struct HIDCLASS_FDO_EXTENSION {
    pub Common: HIDCLASS_COMMON_DEVICE_EXTENSION,
    pub Capabilities: DEVICE_CAPABILITIES,
    pub HidDescriptor: HID_DESCRIPTOR,
    pub ReportDescriptor: *mut UCHAR,
    pub DeviceRelations: PDEVICE_RELATIONS,
}
pub type PHIDCLASS_FDO_EXTENSION = *mut HIDCLASS_FDO_EXTENSION;

// ===========================================================================
// precomp.h — HIDCLASS_PDO_DEVICE_EXTENSION
// ===========================================================================
#[repr(C)]
pub struct HIDCLASS_PDO_DEVICE_EXTENSION {
    pub Common: HIDCLASS_COMMON_DEVICE_EXTENSION,
    pub Capabilities: DEVICE_CAPABILITIES,
    pub CollectionNumber: ULONG,
    pub DeviceInterface: UNICODE_STRING,
    pub FDODeviceObject: PDEVICE_OBJECT,
    pub FDODeviceExtension: PHIDCLASS_FDO_EXTENSION,
}
pub type PHIDCLASS_PDO_DEVICE_EXTENSION = *mut HIDCLASS_PDO_DEVICE_EXTENSION;

// ===========================================================================
// precomp.h — HIDCLASS_FILEOP_CONTEXT
// ===========================================================================
#[repr(C)]
pub struct HIDCLASS_FILEOP_CONTEXT {
    pub DeviceExtension: PHIDCLASS_PDO_DEVICE_EXTENSION,
    pub Lock: KSPIN_LOCK,
    pub ReadPendingIrpListHead: LIST_ENTRY,
    pub IrpCompletedListHead: LIST_ENTRY,
    pub StopInProgress: BOOLEAN,
    pub IrpReadComplete: KEVENT,
}
pub type PHIDCLASS_FILEOP_CONTEXT = *mut HIDCLASS_FILEOP_CONTEXT;

// ===========================================================================
// precomp.h — HIDCLASS_IRP_CONTEXT
// ===========================================================================
#[repr(C)]
pub struct HIDCLASS_IRP_CONTEXT {
    pub OriginalIrp: PIRP,
    pub FileOp: PHIDCLASS_FILEOP_CONTEXT,
    pub InputReportBuffer: PVOID,
    pub InputReportBufferLength: ULONG,
    pub CompletionWorkItem: PIO_WORKITEM,
}
pub type PHIDCLASS_IRP_CONTEXT = *mut HIDCLASS_IRP_CONTEXT;

// ===========================================================================
// hidclass.h — the QUERY_INTERFACE interfaces a HID PDO can hand out.
// Not bindgen'd (hidclass.h is not in the -sys island), so defined locally.
// The C expands INTERFACE inline for the non-C++ build ("GCC doesn't support
// including unnamed structs"), so the first five fields are that expansion and
// their order/width IS the ABI the caller allocated.
// ===========================================================================

// typedef NTSTATUS (NTAPI *PHIDP_GETCAPS)(PHIDP_PREPARSED_DATA, PHIDP_CAPS);
pub type PHIDP_GETCAPS = Option<
    unsafe extern "system" fn(
        PreparsedData: hid_sys::PHIDP_PREPARSED_DATA,
        Capabilities: hid_sys::PHIDP_CAPS,
    ) -> ntkrnl_sys::base::NTSTATUS,
>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct HID_INTERFACE_HIDPARSE {
    pub Size: USHORT,
    pub Version: USHORT,
    pub Context: PVOID,
    pub InterfaceReference: ntkrnl_sys::ddk::PINTERFACE_REFERENCE,
    pub InterfaceDereference: ntkrnl_sys::ddk::PINTERFACE_DEREFERENCE,
    pub HidpGetCaps: PHIDP_GETCAPS,
}
pub type PHID_INTERFACE_HIDPARSE = *mut HID_INTERFACE_HIDPARSE;

// enum DeviceObjectState — hidclass.h, used only through PHID_STATUS_CHANGE.
pub type PHID_STATUS_CHANGE =
    Option<unsafe extern "system" fn(Context: PVOID, State: ntkrnl_sys::base::ULONG)>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct HID_INTERFACE_NOTIFY_PNP {
    pub Size: USHORT,
    pub Version: USHORT,
    pub Context: PVOID,
    pub InterfaceReference: ntkrnl_sys::ddk::PINTERFACE_REFERENCE,
    pub InterfaceDereference: ntkrnl_sys::ddk::PINTERFACE_DEREFERENCE,
    pub StatusChangeFn: PHID_STATUS_CHANGE,
    pub CallbackContext: PVOID,
}
pub type PHID_INTERFACE_NOTIFY_PNP = *mut HID_INTERFACE_NOTIFY_PNP;
