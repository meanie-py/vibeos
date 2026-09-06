// glue.rs — pool tags, IOCTL codes, USB request constants, the UsbBuild* inline
//           replicas (usbdlib.h macros, not exported), USBD_SUCCESS, the extern
//           HidRegisterMinidriver decl, and constants absent from the substrate.
//
// Translated from drivers/hid/hidusb/hidusb.h + hidport.h + hidclass.h + usb.h +
// usbioctl.h + usbiodef.h + usbdlib.h.

use ntkrnl_sys::base::{NTSTATUS, ULONG, USHORT, UCHAR, PVOID};
use usb_sys::{
    URB, PMDL,
    URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER,
    URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE,
    URB_FUNCTION_SELECT_CONFIGURATION,
    USBD_PIPE_HANDLE, USBD_STATUS,
    PUSB_CONFIGURATION_DESCRIPTOR, PUSBD_INTERFACE_LIST_ENTRY,
    PUSB_INTERFACE_DESCRIPTOR,
};
use crate::types::PHID_MINIDRIVER_REGISTRATION;

// ============================================================================
// Pool tags (hidusb.h)
//   HIDUSB_TAG     = 'UdiH'
//   HIDUSB_URB_TAG = 'rUiH'
// ============================================================================
pub const HIDUSB_TAG: ULONG = u32::from_le_bytes(*b"UdiH");
pub const HIDUSB_URB_TAG: ULONG = u32::from_le_bytes(*b"rUiH");

// PURB is not aliased in usb-sys; define it (PURB == *mut URB).
pub type PURB = *mut URB;

// ============================================================================
// hidusb.h USB request codes
// ============================================================================
pub const USB_SET_IDLE_REQUEST: UCHAR = 0xA;
pub const USB_GET_PROTOCOL_REQUEST: UCHAR = 0x3;

// ============================================================================
// hidclass.h
// ============================================================================
pub const HID_REVISION: u32 = 0x00000001;

// ============================================================================
// hidport.h HID descriptor types
// ============================================================================
pub const HID_HID_DESCRIPTOR_TYPE: UCHAR = 0x21;
pub const HID_REPORT_DESCRIPTOR_TYPE: UCHAR = 0x22;

// ============================================================================
// iotypes.h — IRP_MN minor codes absent from the substrate
// ============================================================================
pub const IRP_MN_QUERY_CAPABILITIES: UCHAR = 0x09;
pub const IRP_MN_QUERY_PNP_DEVICE_STATE: UCHAR = 0x14;

// ============================================================================
// iotypes.h — PnP device state flags
// ============================================================================
pub const PNP_DEVICE_NOT_DISABLEABLE: ULONG = 0x00000020;

// ============================================================================
// NTSTATUS values absent from ntkrnl_sys
//   STATUS_CONTINUE_COMPLETION == STATUS_SUCCESS (0)
// ============================================================================
pub const STATUS_CONTINUE_COMPLETION: NTSTATUS = 0x00000000_u32 as i32;
pub const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016_u32 as i32;
pub const STATUS_INVALID_BUFFER_SIZE: NTSTATUS = 0xC0000206_u32 as i32;

// ============================================================================
// usb.h USBD_STATUS values absent from usb-sys
// ============================================================================
pub const USBD_STATUS_SUCCESS: USBD_STATUS = 0x00000000_u32 as i32;
pub const USBD_STATUS_DEVICE_GONE: USBD_STATUS = 0xC0007000_u32 as i32;

// USBD_SUCCESS(Status) == ((USBD_STATUS)(Status) >= 0)   [usb.h:167]
#[inline]
pub fn USBD_SUCCESS(status: USBD_STATUS) -> bool {
    status >= 0
}

// ============================================================================
// CTL_CODE / IOCTL builders (CONTRACT2)
//   CTL_CODE(dev,fn,method,access) = (dev<<16)|(access<<14)|(fn<<2)|method
// ============================================================================
pub const FILE_DEVICE_KEYBOARD: ULONG = 0xb;
pub const FILE_DEVICE_UNKNOWN: ULONG = 0x22; // == FILE_DEVICE_USB
pub const METHOD_BUFFERED: ULONG = 0;
pub const METHOD_IN_DIRECT: ULONG = 1;
pub const METHOD_OUT_DIRECT: ULONG = 2;
pub const METHOD_NEITHER: ULONG = 3;
pub const FILE_ANY_ACCESS: ULONG = 0;

#[inline]
pub const fn ctl_code(device_type: ULONG, function: ULONG, method: ULONG, access: ULONG) -> ULONG {
    (device_type << 16) | (access << 14) | (function << 2) | method
}

// HID class IOCTL variants (hidclass.h:46-52)
#[inline]
pub const fn hid_ctl_code(id: ULONG) -> ULONG {
    ctl_code(FILE_DEVICE_KEYBOARD, id, METHOD_NEITHER, FILE_ANY_ACCESS)
}
#[inline]
pub const fn hid_buffer_ctl_code(id: ULONG) -> ULONG {
    ctl_code(FILE_DEVICE_KEYBOARD, id, METHOD_BUFFERED, FILE_ANY_ACCESS)
}
#[inline]
pub const fn hid_in_ctl_code(id: ULONG) -> ULONG {
    ctl_code(FILE_DEVICE_KEYBOARD, id, METHOD_IN_DIRECT, FILE_ANY_ACCESS)
}
#[inline]
pub const fn hid_out_ctl_code(id: ULONG) -> ULONG {
    ctl_code(FILE_DEVICE_KEYBOARD, id, METHOD_OUT_DIRECT, FILE_ANY_ACCESS)
}

// hidport.h:84-92
pub const IOCTL_HID_GET_DEVICE_DESCRIPTOR: ULONG = hid_ctl_code(0);
pub const IOCTL_HID_GET_REPORT_DESCRIPTOR: ULONG = hid_ctl_code(1);
pub const IOCTL_HID_READ_REPORT: ULONG = hid_ctl_code(2);
pub const IOCTL_HID_WRITE_REPORT: ULONG = hid_ctl_code(3);
pub const IOCTL_HID_GET_DEVICE_ATTRIBUTES: ULONG = hid_ctl_code(9);
pub const IOCTL_HID_SEND_IDLE_NOTIFICATION_REQUEST: ULONG = hid_ctl_code(10);

// hidclass.h:56-77
pub const IOCTL_GET_PHYSICAL_DESCRIPTOR: ULONG = hid_out_ctl_code(102);
pub const IOCTL_HID_GET_FEATURE: ULONG = hid_out_ctl_code(100);
pub const IOCTL_HID_GET_INPUT_REPORT: ULONG = hid_out_ctl_code(104);
pub const IOCTL_HID_GET_INDEXED_STRING: ULONG = hid_out_ctl_code(120);
pub const IOCTL_HID_GET_MS_GENRE_DESCRIPTOR: ULONG = hid_out_ctl_code(121);
pub const IOCTL_HID_SET_FEATURE: ULONG = hid_in_ctl_code(100);
pub const IOCTL_HID_SET_OUTPUT_REPORT: ULONG = hid_in_ctl_code(101);

// usbioctl.h / usbiodef.h internal USB IOCTLs (FILE_DEVICE_USB == 0x22)
//   USB_SUBMIT_URB=0, USB_RESET_PORT=1, USB_GET_PORT_STATUS=4; all METHOD_NEITHER
pub const IOCTL_INTERNAL_USB_SUBMIT_URB: ULONG =
    ctl_code(FILE_DEVICE_UNKNOWN, 0, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_INTERNAL_USB_RESET_PORT: ULONG =
    ctl_code(FILE_DEVICE_UNKNOWN, 1, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_INTERNAL_USB_GET_PORT_STATUS: ULONG =
    ctl_code(FILE_DEVICE_UNKNOWN, 4, METHOD_NEITHER, FILE_ANY_ACCESS);

// ============================================================================
// HidRegisterMinidriver — exported by hidclass.sys, linked at final .sys link.
// ============================================================================
unsafe extern "system" {
    pub fn HidRegisterMinidriver(MinidriverRegistration: PHID_MINIDRIVER_REGISTRATION) -> NTSTATUS;
}

// ============================================================================
// UsbBuild* — usbdlib.h MACROS (no export). Replicated as faithful inline fns
// that fill the corresponding URB-union fields in declaration order.
//
// `urb` aliases the union; field stores go through the __bindgen_anon_1 path.
// PURB_DUMMY_UNUSED is *mut _URB; the C `link` arg is always NULL here so we take
// a *mut URB for the link parameter to mirror the prototype.
// ============================================================================

// UsbBuildInterruptOrBulkTransferRequest(urb,length,pipeHandle,transferBuffer,
//   transferBufferMDL,transferBufferLength,transferFlags,link)
#[inline]
pub unsafe fn UsbBuildInterruptOrBulkTransferRequest(
    urb: *mut URB,
    length: USHORT,
    pipe_handle: USBD_PIPE_HANDLE,
    transfer_buffer: PVOID,
    transfer_buffer_mdl: PMDL,
    transfer_buffer_length: ULONG,
    transfer_flags: ULONG,
    link: *mut URB,
) { unsafe {
    (*urb).__bindgen_anon_1.UrbHeader.Function = URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER as USHORT;
    (*urb).__bindgen_anon_1.UrbHeader.Length = length;
    (*urb).__bindgen_anon_1.UrbBulkOrInterruptTransfer.PipeHandle = pipe_handle;
    (*urb).__bindgen_anon_1.UrbBulkOrInterruptTransfer.TransferBufferLength = transfer_buffer_length;
    (*urb).__bindgen_anon_1.UrbBulkOrInterruptTransfer.TransferBufferMDL = transfer_buffer_mdl;
    (*urb).__bindgen_anon_1.UrbBulkOrInterruptTransfer.TransferBuffer = transfer_buffer;
    (*urb).__bindgen_anon_1.UrbBulkOrInterruptTransfer.TransferFlags = transfer_flags;
    (*urb).__bindgen_anon_1.UrbBulkOrInterruptTransfer.UrbLink = link as *mut _;
}}

// UsbBuildGetDescriptorRequest(urb,length,descriptorType,descriptorIndex,
//   languageId,transferBuffer,transferBufferMDL,transferBufferLength,link)
#[inline]
pub unsafe fn UsbBuildGetDescriptorRequest(
    urb: *mut URB,
    length: USHORT,
    descriptor_type: UCHAR,
    descriptor_index: UCHAR,
    language_id: USHORT,
    transfer_buffer: PVOID,
    transfer_buffer_mdl: PMDL,
    transfer_buffer_length: ULONG,
    link: *mut URB,
) { unsafe {
    (*urb).__bindgen_anon_1.UrbHeader.Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE as USHORT;
    (*urb).__bindgen_anon_1.UrbHeader.Length = length;
    (*urb).__bindgen_anon_1.UrbControlDescriptorRequest.TransferBufferLength = transfer_buffer_length;
    (*urb).__bindgen_anon_1.UrbControlDescriptorRequest.TransferBufferMDL = transfer_buffer_mdl;
    (*urb).__bindgen_anon_1.UrbControlDescriptorRequest.TransferBuffer = transfer_buffer;
    (*urb).__bindgen_anon_1.UrbControlDescriptorRequest.DescriptorType = descriptor_type;
    (*urb).__bindgen_anon_1.UrbControlDescriptorRequest.Index = descriptor_index;
    (*urb).__bindgen_anon_1.UrbControlDescriptorRequest.LanguageId = language_id;
    (*urb).__bindgen_anon_1.UrbControlDescriptorRequest.UrbLink = link as *mut _;
}}

// UsbBuildSelectConfigurationRequest(urb,length,configurationDescriptor)
#[inline]
pub unsafe fn UsbBuildSelectConfigurationRequest(
    urb: *mut URB,
    length: USHORT,
    configuration_descriptor: PUSB_CONFIGURATION_DESCRIPTOR,
) { unsafe {
    (*urb).__bindgen_anon_1.UrbHeader.Function = URB_FUNCTION_SELECT_CONFIGURATION as USHORT;
    (*urb).__bindgen_anon_1.UrbHeader.Length = length;
    (*urb).__bindgen_anon_1.UrbSelectConfiguration.ConfigurationDescriptor = configuration_descriptor;
}}

// UsbBuildVendorRequest(urb,cmd,length,transferFlags,reservedbits,request,value,
//   index,transferBuffer,transferBufferMDL,transferBufferLength,link)
#[inline]
pub unsafe fn UsbBuildVendorRequest(
    urb: *mut URB,
    cmd: USHORT,
    length: USHORT,
    transfer_flags: ULONG,
    reservedbits: UCHAR,
    request: UCHAR,
    value: USHORT,
    index: USHORT,
    transfer_buffer: PVOID,
    transfer_buffer_mdl: PMDL,
    transfer_buffer_length: ULONG,
    link: *mut URB,
) { unsafe {
    (*urb).__bindgen_anon_1.UrbHeader.Function = cmd;
    (*urb).__bindgen_anon_1.UrbHeader.Length = length;
    (*urb).__bindgen_anon_1.UrbControlVendorClassRequest.TransferBufferLength = transfer_buffer_length;
    (*urb).__bindgen_anon_1.UrbControlVendorClassRequest.TransferBufferMDL = transfer_buffer_mdl;
    (*urb).__bindgen_anon_1.UrbControlVendorClassRequest.TransferBuffer = transfer_buffer;
    (*urb).__bindgen_anon_1.UrbControlVendorClassRequest.RequestTypeReservedBits = reservedbits;
    (*urb).__bindgen_anon_1.UrbControlVendorClassRequest.Request = request;
    (*urb).__bindgen_anon_1.UrbControlVendorClassRequest.Value = value;
    (*urb).__bindgen_anon_1.UrbControlVendorClassRequest.Index = index;
    (*urb).__bindgen_anon_1.UrbControlVendorClassRequest.TransferFlags = transfer_flags;
    (*urb).__bindgen_anon_1.UrbControlVendorClassRequest.UrbLink = link as *mut _;
}}

// ============================================================================
// USBD_CreateConfigurationRequestEx / USBD_ParseConfigurationDescriptorEx
// are exported by usbd.sys (linked at the final .sys link). They are NOT in
// usb-sys (no bindgen extern). Declare them here with the usbdlib.h prototypes.
// ============================================================================
unsafe extern "system" {
    pub fn USBD_CreateConfigurationRequestEx(
        ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
        InterfaceList: PUSBD_INTERFACE_LIST_ENTRY,
    ) -> *mut URB;

    pub fn USBD_ParseConfigurationDescriptorEx(
        ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
        StartPosition: PVOID,
        InterfaceNumber: i32,
        AlternateSetting: i32,
        InterfaceClass: i32,
        InterfaceSubClass: i32,
        InterfaceProtocol: i32,
    ) -> PUSB_INTERFACE_DESCRIPTOR;
}

// ============================================================================
// KeGetCurrentIrql — HAL exports it on i386/arm/arm64 (hal.spec: `@ stdcall
// -arch=i386,arm,arm64`) but NOT on amd64, where it is a FORCEINLINE reading
// CR8. hidusb.sys links hidclass/ntoskrnl/usbd/hal, none of which provides it
// there, so the amd64 image needs its own. Same shape as classpnp's ddk_shims.
//
// It only became a link error when `ASSERT(KeGetCurrentIrql() == PASSIVE_LEVEL)`
// at hidusb.c:410 stopped being dead code: as a `debug_assert!` the call was
// compiled out of the release profile entirely, so nothing ever referenced the
// symbol. Making the assertion live is what surfaced it.
// ============================================================================
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentIrql() -> u8 {
    let irql: u64;
    core::arch::asm!("mov {}, cr8", out(reg) irql, options(nomem, nostack, preserves_flags));
    irql as u8
}
