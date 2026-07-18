use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{LANGID, PDEVICE_OBJECT, RtlAssert};
use core::ffi::c_void;

// USBD_PIPE_TYPE::UsbdPipeTypeBulk == 2 per <usb.h>; bindgen names it
// `_USBD_PIPE_TYPE_UsbdPipeTypeBulk`. Re-name to the bare ident the translation uses.
use crate::types::_USBD_PIPE_TYPE_UsbdPipeTypeBulk as UsbdPipeTypeBulk;

extern "C" {
    fn AllocateItem(PoolType: u32, NumberOfBytes: ULONG) -> PVOID;
    fn FreeItem(Item: PVOID);
    fn USBSTOR_SyncUrbRequest(DeviceObject: PDEVICE_OBJECT, Urb: PURB) -> NTSTATUS;
    fn UsbBuildGetDescriptorRequest( Urb: PURB, Length: USHORT, DescriptorType: UCHAR, Index: UCHAR, LanguageId: USHORT, TransferBuffer: PVOID, TransferBufferMDL: PVOID, TransferBufferLength: ULONG, UrbLink: PURB, );
    fn DbgPrint(Format: *const ::core::ffi::c_char, ...) -> ULONG;
}
// libusbd exports this stdcall on i386 (_USBD_CreateConfigurationRequestEx@8) — a cdecl
// decl leaves _USBD_CreateConfigurationRequestEx undefined at link.
unsafe extern "system" {
    fn USBD_CreateConfigurationRequestEx( ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR, InterfaceList: PUSBD_INTERFACE_LIST_ENTRY, ) -> PURB;
}

// ---- file-scope preamble (translated) ----
// (no translatable file-scope declarations; NDEBUG is a function-like/empty macro skipped)

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_GetDescriptor(
    DeviceObject: PDEVICE_OBJECT,
    DescriptorType: UCHAR,
    DescriptorLength: ULONG,
    DescriptorIndex: UCHAR,
    LanguageId: LANGID,
    OutDescriptor: *mut PVOID,
) -> NTSTATUS {

    let mut Urb: PURB;
    let mut Status: NTSTATUS;
    let mut Descriptor: PVOID;

    // ASSERT(DeviceObject);
    // ASSERT(OutDescriptor);
    // ASSERT(DescriptorLength);

    // first allocate descriptor buffer
    Descriptor = AllocateItem(NonPagedPool, DescriptorLength);
    if Descriptor.is_null() {
        // no memory
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Urb = AllocateItem(NonPagedPool, core::mem::size_of::<URB>() as ULONG) as PURB;
    if Urb.is_null() {
        // no memory
        FreeItem(Descriptor);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // initialize urb
    UsbBuildGetDescriptorRequest(
        Urb,
        core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as USHORT,
        DescriptorType,
        DescriptorIndex,
        LanguageId as USHORT,
        Descriptor,
        core::ptr::null_mut(),
        DescriptorLength,
        core::ptr::null_mut(),
    );

    // submit urb
    Status = USBSTOR_SyncUrbRequest(DeviceObject, Urb);

    FreeItem(Urb as PVOID);

    if NT_SUCCESS(Status) {
        *OutDescriptor = Descriptor;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_GetDescriptors(
    DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let DeviceExtension: PFDO_DEVICE_EXTENSION;
    let DescriptorLength: USHORT;

    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    // first get device descriptor
    Status = USBSTOR_GetDescriptor(
        (*DeviceExtension).LowerDeviceObject,
        USB_DEVICE_DESCRIPTOR_TYPE as UCHAR,
        core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() as ULONG,
        0 as UCHAR,
        0 as LANGID,
        core::ptr::addr_of_mut!((*DeviceExtension).DeviceDescriptor) as *mut PVOID,
    );
    if !NT_SUCCESS(Status) {
        (*DeviceExtension).DeviceDescriptor = core::ptr::null_mut();
        return Status;
    }

    // now get basic configuration descriptor
    Status = USBSTOR_GetDescriptor(
        (*DeviceExtension).LowerDeviceObject,
        USB_CONFIGURATION_DESCRIPTOR_TYPE as UCHAR,
        core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>() as ULONG,
        0 as UCHAR,
        0 as LANGID,
        core::ptr::addr_of_mut!((*DeviceExtension).ConfigurationDescriptor) as *mut PVOID,
    );
    if !NT_SUCCESS(Status) {
        FreeItem((*DeviceExtension).DeviceDescriptor as PVOID);
        (*DeviceExtension).DeviceDescriptor = core::ptr::null_mut();
        return Status;
    }

    // backup length
    DescriptorLength = (*(*DeviceExtension).ConfigurationDescriptor).wTotalLength;

    // release basic descriptor
    FreeItem((*DeviceExtension).ConfigurationDescriptor as PVOID);
    (*DeviceExtension).ConfigurationDescriptor = core::ptr::null_mut();

    // allocate full descriptor
    Status = USBSTOR_GetDescriptor(
        (*DeviceExtension).LowerDeviceObject,
        USB_CONFIGURATION_DESCRIPTOR_TYPE as UCHAR,
        DescriptorLength as ULONG,
        0 as UCHAR,
        0 as LANGID,
        core::ptr::addr_of_mut!((*DeviceExtension).ConfigurationDescriptor) as *mut PVOID,
    );
    if !NT_SUCCESS(Status) {
        FreeItem((*DeviceExtension).DeviceDescriptor as PVOID);
        (*DeviceExtension).DeviceDescriptor = core::ptr::null_mut();
        return Status;
    }

    // check if there is a serial number provided
    if (*(*DeviceExtension).DeviceDescriptor).iSerialNumber != 0 {
        // get serial number
        Status = USBSTOR_GetDescriptor(
            (*DeviceExtension).LowerDeviceObject,
            USB_STRING_DESCRIPTOR_TYPE as UCHAR,
            MAXIMUM_USB_STRING_LENGTH as ULONG,
            (*(*DeviceExtension).DeviceDescriptor).iSerialNumber,
            0x0409 as LANGID,
            core::ptr::addr_of_mut!((*DeviceExtension).SerialNumber) as *mut PVOID,
        );
        if !NT_SUCCESS(Status) {
            FreeItem((*DeviceExtension).DeviceDescriptor as PVOID);
            (*DeviceExtension).DeviceDescriptor = core::ptr::null_mut();

            FreeItem((*DeviceExtension).ConfigurationDescriptor as PVOID);
            (*DeviceExtension).ConfigurationDescriptor = core::ptr::null_mut();

            (*DeviceExtension).SerialNumber = core::ptr::null_mut();
            return Status;
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_ScanConfigurationDescriptor(
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    OutInterfaceDescriptor: *mut PUSB_INTERFACE_DESCRIPTOR,
    InEndpointDescriptor: *mut PUSB_ENDPOINT_DESCRIPTOR,
    OutEndpointDescriptor: *mut PUSB_ENDPOINT_DESCRIPTOR,
) -> NTSTATUS {

    const USB_INTERFACE_DESCRIPTOR_TYPE: i32 = 0x04;
    const USB_ENDPOINT_DESCRIPTOR_TYPE: i32 = 0x05;
    const USB_ENDPOINT_TYPE_MASK: i32 = 0x03;
    const USB_ENDPOINT_TYPE_BULK: i32 = 0x02;
    const USB_ENDPOINT_TYPE_INTERRUPT: i32 = 0x03;
    const USB_ENDPOINT_DIRECTION_MASK: i32 = 0x80;

    let mut CurrentDescriptor: PUSB_CONFIGURATION_DESCRIPTOR = core::ptr::null_mut();
    let mut EndpointDescriptor: PUSB_ENDPOINT_DESCRIPTOR = core::ptr::null_mut();

    if ConfigurationDescriptor.is_null() {
        RtlAssert(
            b"ConfigurationDescriptor\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"descriptor.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            149 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    if OutInterfaceDescriptor.is_null() {
        RtlAssert(
            b"OutInterfaceDescriptor\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"descriptor.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            150 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    if InEndpointDescriptor.is_null() {
        RtlAssert(
            b"InEndpointDescriptor\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"descriptor.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            151 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    if OutEndpointDescriptor.is_null() {
        RtlAssert(
            b"OutEndpointDescriptor\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"descriptor.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            152 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    *OutInterfaceDescriptor = core::ptr::null_mut();
    *InEndpointDescriptor = core::ptr::null_mut();
    *OutEndpointDescriptor = core::ptr::null_mut();

    CurrentDescriptor = ConfigurationDescriptor;

    loop {
        if (*CurrentDescriptor).bDescriptorType as i32 == USB_INTERFACE_DESCRIPTOR_TYPE {
            if !(*OutInterfaceDescriptor).is_null() {
                break;
            }
            *OutInterfaceDescriptor = CurrentDescriptor as PUSB_INTERFACE_DESCRIPTOR;
        } else if (*CurrentDescriptor).bDescriptorType as i32 == USB_ENDPOINT_DESCRIPTOR_TYPE {
            EndpointDescriptor = CurrentDescriptor as PUSB_ENDPOINT_DESCRIPTOR;

            if (*OutInterfaceDescriptor).is_null() {
                RtlAssert(
                    b"*OutInterfaceDescriptor\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
                    b"descriptor.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
                    178 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            }

            if (*EndpointDescriptor).bmAttributes as i32 & USB_ENDPOINT_TYPE_MASK
                == USB_ENDPOINT_TYPE_BULK
            {
                if (*EndpointDescriptor).bEndpointAddress as i32 & USB_ENDPOINT_DIRECTION_MASK != 0 {
                    *InEndpointDescriptor = EndpointDescriptor;
                } else {
                    *OutEndpointDescriptor = EndpointDescriptor;
                }
            } else if (*EndpointDescriptor).bmAttributes as i32 & USB_ENDPOINT_TYPE_MASK
                == USB_ENDPOINT_TYPE_INTERRUPT
            {
                DbgPrint(
                    b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const ::core::ffi::c_char,
                    b"USBSTOR_ScanConfigurationDescriptor\0".as_ptr() as *const ::core::ffi::c_char,
                    b"descriptor.c\0".as_ptr() as *const ::core::ffi::c_char,
                    194 as ::core::ffi::c_int,
                );
            }
        }

        CurrentDescriptor = (CurrentDescriptor as ULONG_PTR)
            .wrapping_add((*CurrentDescriptor).bLength as ULONG_PTR)
            as PUSB_CONFIGURATION_DESCRIPTOR;

        if CurrentDescriptor as ULONG_PTR
            >= (ConfigurationDescriptor as ULONG_PTR)
                .wrapping_add((*ConfigurationDescriptor).wTotalLength as ULONG_PTR)
        {
            break;
        }
    }

    if (*OutInterfaceDescriptor).is_null()
        || (*InEndpointDescriptor).is_null()
        || (*OutEndpointDescriptor).is_null()
    {
        if DbgPrint(
            b"(%s:%d) USBSTOR_ScanConfigurationDescriptor: Failed to find InterfaceDescriptor %p InEndpointDescriptor %p OutEndpointDescriptor %p\n\0".as_ptr() as *const ::core::ffi::c_char,
            b"descriptor.c\0".as_ptr() as *const ::core::ffi::c_char,
            212 as ::core::ffi::c_int,
            *OutInterfaceDescriptor,
            *InEndpointDescriptor,
            *OutEndpointDescriptor,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const ::core::ffi::c_char,
                b"descriptor.c\0".as_ptr() as *const ::core::ffi::c_char,
                212 as ::core::ffi::c_int,
            );
        }
        return STATUS_UNSUCCESSFUL;
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn DumpConfigurationDescriptor(
    mut ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
) {
    if DbgPrint(
        b"(%s:%d) Dumping ConfigurationDescriptor %p\n\0" as *const u8 as PCSTR,
        b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
        222 as ::core::ffi::c_int,
        ConfigurationDescriptor,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
            222 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) bLength %x\n\0" as *const u8 as PCSTR,
        b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
        223 as ::core::ffi::c_int,
        (*ConfigurationDescriptor).bLength as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
            223 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) bDescriptorType %x\n\0" as *const u8 as PCSTR,
        b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
        224 as ::core::ffi::c_int,
        (*ConfigurationDescriptor).bDescriptorType as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
            224 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) wTotalLength %x\n\0" as *const u8 as PCSTR,
        b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
        225 as ::core::ffi::c_int,
        (*ConfigurationDescriptor).wTotalLength as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
            225 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) bNumInterfaces %x\n\0" as *const u8 as PCSTR,
        b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
        226 as ::core::ffi::c_int,
        (*ConfigurationDescriptor).bNumInterfaces as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
            226 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) bConfigurationValue %x\n\0" as *const u8 as PCSTR,
        b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
        227 as ::core::ffi::c_int,
        (*ConfigurationDescriptor).bConfigurationValue as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
            227 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) iConfiguration %x\n\0" as *const u8 as PCSTR,
        b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
        228 as ::core::ffi::c_int,
        (*ConfigurationDescriptor).iConfiguration as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
            228 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) bmAttributes %x\n\0" as *const u8 as PCSTR,
        b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
        229 as ::core::ffi::c_int,
        (*ConfigurationDescriptor).bmAttributes as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
            229 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) MaxPower %x\n\0" as *const u8 as PCSTR,
        b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
        230 as ::core::ffi::c_int,
        (*ConfigurationDescriptor).MaxPower as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
            230 as ::core::ffi::c_int,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_SelectConfigurationAndInterface(
    DeviceObject: PDEVICE_OBJECT,
    DeviceExtension: PFDO_DEVICE_EXTENSION,
) -> NTSTATUS {

    const URB_FUNCTION_SELECT_INTERFACE: u32 = 0x0001;

    let mut InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR =
        core::ptr::null_mut();
    let mut InEndpointDescriptor: PUSB_ENDPOINT_DESCRIPTOR =
        core::ptr::null_mut();
    let mut OutEndpointDescriptor: PUSB_ENDPOINT_DESCRIPTOR =
        core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Urb: PURB;
    let mut InterfaceList: PUSBD_INTERFACE_LIST_ENTRY;

    Status = USBSTOR_ScanConfigurationDescriptor(
        (*DeviceExtension).ConfigurationDescriptor,
        core::ptr::addr_of_mut!(InterfaceDescriptor),
        core::ptr::addr_of_mut!(InEndpointDescriptor),
        core::ptr::addr_of_mut!(OutEndpointDescriptor),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    InterfaceList = AllocateItem(
        PagedPool,
        (core::mem::size_of::<USBD_INTERFACE_LIST_ENTRY>() * 2) as ULONG,
    ) as PUSBD_INTERFACE_LIST_ENTRY;
    if InterfaceList.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*InterfaceList.offset(0)).InterfaceDescriptor = InterfaceDescriptor;

    Urb = USBD_CreateConfigurationRequestEx(
        (*DeviceExtension).ConfigurationDescriptor,
        InterfaceList,
    );
    if Urb.is_null() {
        FreeItem(InterfaceList as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if (*InterfaceList.offset(0)).Interface.is_null() {
        RtlAssert(
            b"InterfaceList[0].Interface\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/descriptor.c\0".as_ptr() as *const i8 as PVOID,
            269 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    Status = USBSTOR_SyncUrbRequest((*DeviceExtension).LowerDeviceObject, Urb);
    if !NT_SUCCESS(Status) {
        if DbgPrint(
            b"(%s:%d) USBSTOR_SelectConfiguration failed to set interface %x\n\0".as_ptr() as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/descriptor.c\0".as_ptr() as *const i8,
            276 as i32,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/descriptor.c\0".as_ptr() as *const i8,
                276 as i32,
            );
        }
        FreeItem(InterfaceList as PVOID);
        ExFreePoolWithTag(Urb as PVOID, 0 as ULONG);
        return Status;
    }

    (*DeviceExtension).InterfaceInformation = AllocateItem(
        NonPagedPool,
        (*(*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref()).Interface.Length as ULONG,
    ) as PUSBD_INTERFACE_INFORMATION;
    if (*DeviceExtension).InterfaceInformation.is_null() {
        FreeItem(InterfaceList as PVOID);
        ExFreePoolWithTag(Urb as PVOID, 0 as ULONG);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyMemory(
        (*DeviceExtension).InterfaceInformation as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*(*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_mut()).Interface) as *const core::ffi::c_void,
        (*(*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref()).Interface.Length as usize,
    );

    (*DeviceExtension).ConfigurationHandle =
        (*(*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref()).ConfigurationHandle;

    (*(*Urb).__bindgen_anon_1.UrbHeader.as_mut()).Function = URB_FUNCTION_SELECT_INTERFACE as USHORT;
    (*(*Urb).__bindgen_anon_1.UrbHeader.as_mut()).Length =
        (core::mem::size_of::<_URB_SELECT_INTERFACE>()
            + (((*InterfaceDescriptor).bNumEndpoints as i32 - 1) as usize)
                * core::mem::size_of::<USBD_PIPE_INFORMATION>()) as USHORT;
    (*(*Urb).__bindgen_anon_1.UrbSelectInterface.as_mut()).Interface.AlternateSetting =
        (*InterfaceDescriptor).bAlternateSetting;
    (*(*Urb).__bindgen_anon_1.UrbSelectInterface.as_mut()).Interface.InterfaceNumber =
        (*InterfaceDescriptor).bInterfaceNumber;
    (*(*Urb).__bindgen_anon_1.UrbSelectInterface.as_mut()).Interface.Length =
        ((core::mem::size_of::<_URB_SELECT_INTERFACE>()
            + (((*InterfaceDescriptor).bNumEndpoints as i32 - 1) as usize)
                * core::mem::size_of::<USBD_PIPE_INFORMATION>())
            - core::mem::size_of::<_URB_HEADER>()
            - core::mem::size_of::<USBD_CONFIGURATION_HANDLE>()) as USHORT;
    (*(*Urb).__bindgen_anon_1.UrbSelectInterface.as_mut()).ConfigurationHandle =
        (*DeviceExtension).ConfigurationHandle;

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*(*Urb).__bindgen_anon_1.UrbSelectInterface.as_mut()).Interface) as *mut core::ffi::c_void,
        (*DeviceExtension).InterfaceInformation as *const core::ffi::c_void,
        (*(*DeviceExtension).InterfaceInformation).Length as usize,
    );

    Status = USBSTOR_SyncUrbRequest((*DeviceExtension).LowerDeviceObject, Urb);
    if NT_SUCCESS(Status) {
        if !((*(*Urb).__bindgen_anon_1.UrbSelectInterface.as_ref()).Interface.Length as i32
            == (*(*DeviceExtension).InterfaceInformation).Length as i32)
        {
            RtlAssert(
                b"Urb->UrbSelectInterface.Interface.Length == DeviceExtension->InterfaceInformation->Length\0".as_ptr() as *const i8 as PVOID,
                b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/descriptor.c\0".as_ptr() as *const i8 as PVOID,
                308 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
        RtlCopyMemory(
            (*DeviceExtension).InterfaceInformation as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*(*Urb).__bindgen_anon_1.UrbSelectInterface.as_mut()).Interface) as *const core::ffi::c_void,
            (*(*Urb).__bindgen_anon_1.UrbSelectInterface.as_ref()).Interface.Length as usize,
        );
    }

    FreeItem(InterfaceList as PVOID);
    ExFreePoolWithTag(Urb as PVOID, 0 as ULONG);

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn USBSTOR_GetPipeHandles(
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
) -> NTSTATUS {
    const USB_ENDPOINT_DIRECTION_MASK: ::core::ffi::c_int = 0x80;
    const STATUS_DEVICE_CONFIGURATION_ERROR: NTSTATUS = 0xC0000182u32 as NTSTATUS;
    const FALSE: ::core::ffi::c_int = 0;
    const TRUE: ::core::ffi::c_int = 1;

    let mut Index: ULONG;
    let mut BulkInFound: BOOLEAN = FALSE as BOOLEAN;
    let mut BulkOutFound: BOOLEAN = FALSE as BOOLEAN;

    Index = 0 as ULONG;
    while Index < (*(*DeviceExtension).InterfaceInformation).NumberOfPipes {
        if (*(&raw mut (*(*DeviceExtension).InterfaceInformation).Pipes
            as *mut USBD_PIPE_INFORMATION)
            .offset(Index as isize))
        .PipeType as ::core::ffi::c_uint
            == UsbdPipeTypeBulk as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*(&raw mut (*(*DeviceExtension).InterfaceInformation).Pipes
                as *mut USBD_PIPE_INFORMATION)
                .offset(Index as isize))
            .EndpointAddress as ::core::ffi::c_int
                & USB_ENDPOINT_DIRECTION_MASK
                != 0
            {
                (*DeviceExtension).BulkInPipeIndex = Index as UCHAR;
                if !(BulkInFound as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
                    RtlAssert(
                        b"BulkInFound == FALSE\0" as *const u8
                            as *const ::core::ffi::c_char as PVOID,
                        b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                        335 as ULONG,
                        ::core::ptr::null_mut::<CHAR>(),
                    );
                } else {
                };
                BulkInFound = TRUE as BOOLEAN;
            } else {
                (*DeviceExtension).BulkOutPipeIndex = Index as UCHAR;
                if !(BulkOutFound as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
                    RtlAssert(
                        b"BulkOutFound == FALSE\0" as *const u8
                            as *const ::core::ffi::c_char as PVOID,
                        b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                        343 as ULONG,
                        ::core::ptr::null_mut::<CHAR>(),
                    );
                } else {
                };
                BulkOutFound = TRUE as BOOLEAN;
            }
        }
        Index = Index.wrapping_add(1);
    }
    if BulkInFound == 0 || BulkOutFound == 0 {
        if DbgPrint(
            b"(%s:%d) USBSTOR_GetPipeHandles> BulkInFound %c BulkOutFound %c missing!!!\n\0"
                as *const u8 as PCSTR,
            b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
            352 as ::core::ffi::c_int,
            BulkInFound as ::core::ffi::c_int,
            BulkOutFound as ::core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"descriptor.c\0" as *const u8 as *const ::core::ffi::c_char,
                352 as ::core::ffi::c_int,
            );
        }
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }
    return STATUS_SUCCESS;
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
//  */
//
// #include "usbstor.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// NTSTATUS
// NTAPI
// USBSTOR_GetDescriptor(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN UCHAR DescriptorType,
//     IN ULONG DescriptorLength,
//     IN UCHAR DescriptorIndex,
//     IN LANGID LanguageId,
//     OUT PVOID *OutDescriptor)
// {
//     PURB Urb;
//     NTSTATUS Status;
//     PVOID Descriptor;
//
//     ASSERT(DeviceObject);
//     ASSERT(OutDescriptor);
//     ASSERT(DescriptorLength);
//
//     // first allocate descriptor buffer
//     Descriptor = AllocateItem(NonPagedPool, DescriptorLength);
//     if (!Descriptor)
//     {
//         // no memory
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Urb = (PURB) AllocateItem(NonPagedPool, sizeof(URB));
//     if (!Urb)
//     {
//         // no memory
//         FreeItem(Descriptor);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // initialize urb
//     UsbBuildGetDescriptorRequest(Urb,
//                                  sizeof(Urb->UrbControlDescriptorRequest),
//                                  DescriptorType,
//                                  DescriptorIndex,
//                                  LanguageId,
//                                  Descriptor,
//                                  NULL,
//                                  DescriptorLength,
//                                  NULL);
//
//     // submit urb
//     Status = USBSTOR_SyncUrbRequest(DeviceObject, Urb);
//
//     FreeItem(Urb);
//
//     if (NT_SUCCESS(Status))
//     {
//         *OutDescriptor = Descriptor;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// USBSTOR_GetDescriptors(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     NTSTATUS Status;
//     PFDO_DEVICE_EXTENSION DeviceExtension;
//     USHORT DescriptorLength;
//
//     DeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//      // first get device descriptor
//      Status = USBSTOR_GetDescriptor(DeviceExtension->LowerDeviceObject, USB_DEVICE_DESCRIPTOR_TYPE, sizeof(USB_DEVICE_DESCRIPTOR), 0, 0, (PVOID*)&DeviceExtension->DeviceDescriptor);
//      if (!NT_SUCCESS(Status))
//      {
//          DeviceExtension->DeviceDescriptor = NULL;
//          return Status;
//      }
//
//      // now get basic configuration descriptor
//      Status = USBSTOR_GetDescriptor(DeviceExtension->LowerDeviceObject, USB_CONFIGURATION_DESCRIPTOR_TYPE, sizeof(USB_CONFIGURATION_DESCRIPTOR), 0, 0, (PVOID*)&DeviceExtension->ConfigurationDescriptor);
//      if (!NT_SUCCESS(Status))
//      {
//          FreeItem(DeviceExtension->DeviceDescriptor);
//          DeviceExtension->DeviceDescriptor = NULL;
//          return Status;
//      }
//
//      // backup length
//      DescriptorLength = DeviceExtension->ConfigurationDescriptor->wTotalLength;
//
//      // release basic descriptor
//      FreeItem(DeviceExtension->ConfigurationDescriptor);
//      DeviceExtension->ConfigurationDescriptor = NULL;
//
//      // allocate full descriptor
//      Status = USBSTOR_GetDescriptor(DeviceExtension->LowerDeviceObject, USB_CONFIGURATION_DESCRIPTOR_TYPE, DescriptorLength, 0, 0, (PVOID*)&DeviceExtension->ConfigurationDescriptor);
//      if (!NT_SUCCESS(Status))
//      {
//          FreeItem(DeviceExtension->DeviceDescriptor);
//          DeviceExtension->DeviceDescriptor = NULL;
//          return Status;
//      }
//
//      // check if there is a serial number provided
//      if (DeviceExtension->DeviceDescriptor->iSerialNumber)
//      {
//          // get serial number
//          Status = USBSTOR_GetDescriptor(DeviceExtension->LowerDeviceObject, USB_STRING_DESCRIPTOR_TYPE, MAXIMUM_USB_STRING_LENGTH, DeviceExtension->DeviceDescriptor->iSerialNumber, 0x0409, (PVOID*)&DeviceExtension->SerialNumber);
//          if (!NT_SUCCESS(Status))
//          {
//              FreeItem(DeviceExtension->DeviceDescriptor);
//              DeviceExtension->DeviceDescriptor = NULL;
//
//              FreeItem(DeviceExtension->ConfigurationDescriptor);
//              DeviceExtension->ConfigurationDescriptor = NULL;
//
//              DeviceExtension->SerialNumber = NULL;
//              return Status;
//           }
//      }
//
//      return Status;
// }
//
// NTSTATUS
// NTAPI
// USBSTOR_ScanConfigurationDescriptor(
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     OUT PUSB_INTERFACE_DESCRIPTOR *OutInterfaceDescriptor,
//     OUT PUSB_ENDPOINT_DESCRIPTOR  *InEndpointDescriptor,
//     OUT PUSB_ENDPOINT_DESCRIPTOR  *OutEndpointDescriptor)
// {
//     PUSB_CONFIGURATION_DESCRIPTOR CurrentDescriptor;
//     PUSB_ENDPOINT_DESCRIPTOR EndpointDescriptor;
//
//     ASSERT(ConfigurationDescriptor);
//     ASSERT(OutInterfaceDescriptor);
//     ASSERT(InEndpointDescriptor);
//     ASSERT(OutEndpointDescriptor);
//
//     *OutInterfaceDescriptor = NULL;
//     *InEndpointDescriptor = NULL;
//     *OutEndpointDescriptor = NULL;
//
//     // start scanning
//     CurrentDescriptor = ConfigurationDescriptor;
//
//     do
//     {
//         if (CurrentDescriptor->bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE)
//         {
//             if (*OutInterfaceDescriptor)
//             {
//                 // we only process the first interface descriptor as ms does -> see documentation
//                 break;
//             }
//
//             // store interface descriptor
//             *OutInterfaceDescriptor = (PUSB_INTERFACE_DESCRIPTOR)CurrentDescriptor;
//         }
//         else if (CurrentDescriptor->bDescriptorType == USB_ENDPOINT_DESCRIPTOR_TYPE)
//         {
//             EndpointDescriptor = (PUSB_ENDPOINT_DESCRIPTOR)CurrentDescriptor;
//
//             ASSERT(*OutInterfaceDescriptor);
//
//             // get endpoint type
//             if ((EndpointDescriptor->bmAttributes & USB_ENDPOINT_TYPE_MASK) == USB_ENDPOINT_TYPE_BULK)
//             {
//                 if (USB_ENDPOINT_DIRECTION_IN(EndpointDescriptor->bEndpointAddress))
//                 {
//                     *InEndpointDescriptor = EndpointDescriptor;
//                 }
//                 else
//                 {
//                     *OutEndpointDescriptor = EndpointDescriptor;
//                 }
//             }
//             else if ((EndpointDescriptor->bmAttributes & USB_ENDPOINT_TYPE_MASK) == USB_ENDPOINT_TYPE_INTERRUPT)
//             {
//                 UNIMPLEMENTED;
//             }
//         }
//
//         // move to next descriptor
//         CurrentDescriptor = (PUSB_CONFIGURATION_DESCRIPTOR)((ULONG_PTR)CurrentDescriptor + CurrentDescriptor->bLength);
//
//         // was it the last descriptor
//         if ((ULONG_PTR)CurrentDescriptor >= ((ULONG_PTR)ConfigurationDescriptor + ConfigurationDescriptor->wTotalLength))
//         {
//             break;
//         }
//
//     } while(TRUE);
//
//     // check if everything has been found
//     if (*OutInterfaceDescriptor == NULL || *InEndpointDescriptor == NULL || *OutEndpointDescriptor == NULL)
//     {
//         DPRINT1("USBSTOR_ScanConfigurationDescriptor: Failed to find InterfaceDescriptor %p InEndpointDescriptor %p OutEndpointDescriptor %p\n", *OutInterfaceDescriptor, *InEndpointDescriptor, *OutEndpointDescriptor);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// DumpConfigurationDescriptor(PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor)
// {
//     DPRINT1("Dumping ConfigurationDescriptor %p\n", ConfigurationDescriptor);
//     DPRINT1("bLength %x\n", ConfigurationDescriptor->bLength);
//     DPRINT1("bDescriptorType %x\n", ConfigurationDescriptor->bDescriptorType);
//     DPRINT1("wTotalLength %x\n", ConfigurationDescriptor->wTotalLength);
//     DPRINT1("bNumInterfaces %x\n", ConfigurationDescriptor->bNumInterfaces);
//     DPRINT1("bConfigurationValue %x\n", ConfigurationDescriptor->bConfigurationValue);
//     DPRINT1("iConfiguration %x\n", ConfigurationDescriptor->iConfiguration);
//     DPRINT1("bmAttributes %x\n", ConfigurationDescriptor->bmAttributes);
//     DPRINT1("MaxPower %x\n", ConfigurationDescriptor->MaxPower);
// }
//
// NTSTATUS
// USBSTOR_SelectConfigurationAndInterface(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PFDO_DEVICE_EXTENSION DeviceExtension)
// {
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor;
//     PUSB_ENDPOINT_DESCRIPTOR InEndpointDescriptor, OutEndpointDescriptor;
//     NTSTATUS Status;
//     PURB Urb;
//     PUSBD_INTERFACE_LIST_ENTRY InterfaceList;
//
//     Status = USBSTOR_ScanConfigurationDescriptor(DeviceExtension->ConfigurationDescriptor, &InterfaceDescriptor, &InEndpointDescriptor, &OutEndpointDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     // now allocate one interface entry and terminating null entry
//     InterfaceList = (PUSBD_INTERFACE_LIST_ENTRY)AllocateItem(PagedPool, sizeof(USBD_INTERFACE_LIST_ENTRY) * 2);
//     if (!InterfaceList)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // initialize interface list entry
//     InterfaceList[0].InterfaceDescriptor = InterfaceDescriptor;
//
//     // now allocate the urb
//     Urb = USBD_CreateConfigurationRequestEx(DeviceExtension->ConfigurationDescriptor, InterfaceList);
//     if (!Urb)
//     {
//         FreeItem(InterfaceList);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//
//     ASSERT(InterfaceList[0].Interface);
//
//     // submit urb
//     Status = USBSTOR_SyncUrbRequest(DeviceExtension->LowerDeviceObject, Urb);
//     if (!NT_SUCCESS(Status))
//     {
//         // failed to set configuration
//         DPRINT1("USBSTOR_SelectConfiguration failed to set interface %x\n", Status);
//         FreeItem(InterfaceList);
//         ExFreePoolWithTag(Urb, 0);
//         return Status;
//     }
//
//     // backup interface information
//     DeviceExtension->InterfaceInformation = (PUSBD_INTERFACE_INFORMATION)AllocateItem(NonPagedPool, Urb->UrbSelectConfiguration.Interface.Length);
//     if (!DeviceExtension->InterfaceInformation)
//     {
//         FreeItem(InterfaceList);
//         ExFreePoolWithTag(Urb, 0);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // copy interface information
//     RtlCopyMemory(DeviceExtension->InterfaceInformation, &Urb->UrbSelectConfiguration.Interface, Urb->UrbSelectConfiguration.Interface.Length);
//
//     // store pipe handle
//     DeviceExtension->ConfigurationHandle = Urb->UrbSelectConfiguration.ConfigurationHandle;
//
//     // now prepare interface urb
//     UsbBuildSelectInterfaceRequest(Urb, GET_SELECT_INTERFACE_REQUEST_SIZE(InterfaceDescriptor->bNumEndpoints), DeviceExtension->ConfigurationHandle, InterfaceDescriptor->bInterfaceNumber, InterfaceDescriptor->bAlternateSetting);
//
//     // copy interface information structure back - as offset for SelectConfiguration / SelectInterface request do differ
//     RtlCopyMemory(&Urb->UrbSelectInterface.Interface, DeviceExtension->InterfaceInformation, DeviceExtension->InterfaceInformation->Length);
//
//     // now select the interface
//     Status = USBSTOR_SyncUrbRequest(DeviceExtension->LowerDeviceObject, Urb);
//     if (NT_SUCCESS(Status))
//     {
//         // update configuration info
//         ASSERT(Urb->UrbSelectInterface.Interface.Length == DeviceExtension->InterfaceInformation->Length);
//         RtlCopyMemory(DeviceExtension->InterfaceInformation, &Urb->UrbSelectInterface.Interface, Urb->UrbSelectInterface.Interface.Length);
//     }
//
//     FreeItem(InterfaceList);
//     ExFreePoolWithTag(Urb, 0);
//
//     return Status;
// }
//
// NTSTATUS
// USBSTOR_GetPipeHandles(
//     IN PFDO_DEVICE_EXTENSION DeviceExtension)
// {
//     ULONG Index;
//     BOOLEAN BulkInFound = FALSE, BulkOutFound = FALSE;
//
//     // enumerate all pipes and extract bulk-in / bulk-out pipe handle
//     for (Index = 0; Index < DeviceExtension->InterfaceInformation->NumberOfPipes; Index++)
//     {
//         if (DeviceExtension->InterfaceInformation->Pipes[Index].PipeType == UsbdPipeTypeBulk)
//         {
//             if (USB_ENDPOINT_DIRECTION_IN(DeviceExtension->InterfaceInformation->Pipes[Index].EndpointAddress))
//             {
//                 DeviceExtension->BulkInPipeIndex = Index;
//
//                 // there should not be another bulk in pipe
//                 ASSERT(BulkInFound == FALSE);
//                 BulkInFound = TRUE;
//             }
//             else
//             {
//                 DeviceExtension->BulkOutPipeIndex = Index;
//
//                 // there should not be another bulk out pipe
//                 ASSERT(BulkOutFound == FALSE);
//                 BulkOutFound = TRUE;
//             }
//         }
//     }
//
//     if (!BulkInFound || !BulkOutFound)
//     {
//         // WTF? usb port driver does not give us bulk pipe access
//         DPRINT1("USBSTOR_GetPipeHandles> BulkInFound %c BulkOutFound %c missing!!!\n", BulkInFound, BulkOutFound);
//         return STATUS_DEVICE_CONFIGURATION_ERROR;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================