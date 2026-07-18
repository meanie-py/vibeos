use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExFreePool, LANGID, PDEVICE_OBJECT};
use core::ffi::c_void;

extern "system" {
    fn USBD_ParseConfigurationDescriptorEx( ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR, StartPosition: PVOID, InterfaceNumber: LONG, AlternateSetting: LONG, InterfaceClass: LONG, InterfaceSubClass: LONG, InterfaceProtocol: LONG, ) -> PUSB_INTERFACE_DESCRIPTOR;
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
    fn USBD_CreateConfigurationRequestEx( ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR, InterfaceList: PUSBD_INTERFACE_LIST_ENTRY, ) -> PURB;
}
unsafe extern "C" {
    fn UsbBuildGetDescriptorRequest( Urb: PURB, Length: USHORT, DescriptorType: UCHAR, DescriptorIndex: UCHAR, LanguageId: USHORT, TransferBuffer: PVOID, TransferBufferMDL: PVOID, TransferBufferLength: ULONG, Link: PURB, );
}
unsafe extern "C" {
    fn AllocateItem(PoolType: u32, ItemSize: ULONG) -> PVOID;
}
unsafe extern "C" {
    fn FreeItem(Item: PVOID);
}
unsafe extern "C" {
    fn USBCCGP_SyncUrbRequest(DeviceObject: PDEVICE_OBJECT, Urb: PURB) -> NTSTATUS;
}
unsafe extern "C" {
    fn UsbBuildSelectInterfaceRequest( Urb: PURB, TotalLength: USHORT, ConfigurationHandle: USBD_CONFIGURATION_HANDLE, InterfaceNumber: UCHAR, AlternateSetting: UCHAR, );
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG (debug-only macro; no runtime declaration)

#[no_mangle]
pub unsafe extern "system" fn USBCCGP_GetDescriptor(
    mut DeviceObject: PDEVICE_OBJECT,
    mut DescriptorType: UCHAR,
    mut DescriptorLength: ULONG,
    mut DescriptorIndex: UCHAR,
    mut LanguageId: LANGID,
    mut OutDescriptor: *mut PVOID,
) -> NTSTATUS {

    let mut Urb: PURB;
    let mut Status: NTSTATUS;
    let mut Descriptor: PVOID;

    //
    // sanity checks
    //
    // ASSERT(DeviceObject);
    // ASSERT(OutDescriptor);
    // ASSERT(DescriptorLength);

    //
    // first allocate descriptor buffer
    //
    Descriptor = AllocateItem(NonPagedPool, DescriptorLength);
    if Descriptor.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // allocate urb
    //
    Urb = AllocateItem(NonPagedPool, core::mem::size_of::<URB>() as ULONG) as PURB;
    if Urb.is_null() {
        //
        // no memory
        //
        FreeItem(Descriptor);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // initialize urb
    //
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

    //
    // submit urb
    //
    Status = USBCCGP_SyncUrbRequest(DeviceObject, Urb);

    //
    // free urb
    //
    FreeItem(Urb as PVOID);

    if NT_SUCCESS(Status) {
        //
        // store result
        //
        *OutDescriptor = Descriptor;
    }

    //
    // done
    //
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBCCGP_GetStringDescriptor(
    mut DeviceObject: PDEVICE_OBJECT,
    mut DescriptorLength: ULONG,
    mut DescriptorIndex: UCHAR,
    mut LanguageId: LANGID,
    mut OutDescriptor: *mut PVOID,
) -> NTSTATUS {

    const USB_STRING_DESCRIPTOR_TYPE: u32 = 0x03;

    let mut Status: NTSTATUS;
    let mut StringDescriptor: PUSB_STRING_DESCRIPTOR;
    let mut Size: ULONG;
    let mut Buffer: PVOID;

    // retrieve descriptor
    Status = USBCCGP_GetDescriptor(
        DeviceObject,
        USB_STRING_DESCRIPTOR_TYPE as UCHAR,
        DescriptorLength,
        DescriptorIndex,
        LanguageId,
        OutDescriptor,
    );
    if !NT_SUCCESS(Status) {
        // failed
        return Status;
    }

    // get descriptor structure
    StringDescriptor = *OutDescriptor as PUSB_STRING_DESCRIPTOR;

    // sanity check
    if !(((*StringDescriptor).bLength as ULONG) < (DescriptorLength).wrapping_sub(2)) {
        RtlAssert(
            b"StringDescriptor->bLength < DescriptorLength - 2\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/descriptor.c\0".as_ptr() as *const i8 as PVOID,
            126 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    if (*StringDescriptor).bLength as i32 == 2 {
        // invalid descriptor
        FreeItem(StringDescriptor as PVOID);
        return STATUS_DEVICE_DATA_ERROR;
    }

    // calculate size
    Size = ((*StringDescriptor).bLength as usize)
        .wrapping_add(core::mem::size_of::<WCHAR>()) as ULONG;

    // allocate buffer
    Buffer = AllocateItem(NonPagedPool, Size);
    if Buffer.is_null() {
        // no memory
        FreeItem(StringDescriptor as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // copy result
    RtlCopyMemory(
        Buffer as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*StringDescriptor).bString) as *mut WCHAR as *const core::ffi::c_void,
        (Size as usize).wrapping_sub(core::mem::offset_of!(USB_STRING_DESCRIPTOR, bString)),
    );

    // free buffer
    FreeItem(StringDescriptor as PVOID);

    // store result
    *OutDescriptor = Buffer;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_GetDescriptors(
    mut DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut DescriptorLength: USHORT;

    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    Status = USBCCGP_GetDescriptor(
        (*DeviceExtension).NextDeviceObject,
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

    Status = USBCCGP_GetDescriptor(
        (*DeviceExtension).NextDeviceObject,
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

    DescriptorLength = (*(*DeviceExtension).ConfigurationDescriptor).wTotalLength;

    FreeItem((*DeviceExtension).ConfigurationDescriptor as PVOID);
    (*DeviceExtension).ConfigurationDescriptor = core::ptr::null_mut();

    Status = USBCCGP_GetDescriptor(
        (*DeviceExtension).NextDeviceObject,
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
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn AllocateInterfaceDescriptorsArray(
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    OutArray: *mut *mut PUSB_INTERFACE_DESCRIPTOR,
) -> NTSTATUS {

    let mut InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR = core::ptr::null_mut();
    let mut Count: ULONG = 0;
    let mut Array: *mut PUSB_INTERFACE_DESCRIPTOR;

    //
    // allocate array
    //
    Array = AllocateItem(
        NonPagedPool,
        (core::mem::size_of::<PUSB_INTERFACE_DESCRIPTOR>()
            .wrapping_mul((*ConfigurationDescriptor).bNumInterfaces as usize)) as ULONG,
    ) as *mut PUSB_INTERFACE_DESCRIPTOR;
    if Array.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // enumerate all interfaces
    //
    Count = 0;
    loop {
        //
        // find next descriptor
        //
        InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(
            ConfigurationDescriptor,
            ConfigurationDescriptor as PVOID,
            Count as LONG,
            0 as LONG,
            -1i32 as LONG,
            -1i32 as LONG,
            -1i32 as LONG,
        );
        if InterfaceDescriptor.is_null() {
            break;
        }

        //
        // store descriptor
        //
        *Array.offset(Count as isize) = InterfaceDescriptor;
        Count = Count.wrapping_add(1);
    }

    //
    // store result
    //
    *OutArray = Array;

    //
    // done
    //
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn DumpFullConfigurationDescriptor(
    mut FDODeviceExtension: PFDO_DEVICE_EXTENSION,
    mut ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
) {
    let mut Descriptor: PUSB_COMMON_DESCRIPTOR;

    Descriptor = ConfigurationDescriptor as PUSB_COMMON_DESCRIPTOR;

    DbgPrint(b"Bogus ConfigurationDescriptor Found\n\0".as_ptr() as *const i8);
    DbgPrint(
        b"InterfaceCount %x\n\0".as_ptr() as *const i8,
        (*ConfigurationDescriptor).bNumInterfaces as core::ffi::c_int,
    );

    loop {
        if (Descriptor as ULONG_PTR)
            >= (ConfigurationDescriptor as ULONG_PTR)
                .wrapping_add((*ConfigurationDescriptor).wTotalLength as ULONG_PTR)
        {
            break;
        }

        DbgPrint(
            b"Descriptor Type %x Length %lu Offset %lu\n\0".as_ptr() as *const i8,
            (*Descriptor).bDescriptorType as core::ffi::c_int,
            (*Descriptor).bLength as core::ffi::c_int,
            (Descriptor as ULONG_PTR).wrapping_sub(ConfigurationDescriptor as ULONG_PTR),
        );

        // check for invalid descriptors
        if (*Descriptor).bLength == 0 {
            DbgPrint(b"Bogus Descriptor!!!\n\0".as_ptr() as *const i8);
            break;
        }

        // advance to next descriptor
        Descriptor = (Descriptor as ULONG_PTR)
            .wrapping_add((*Descriptor).bLength as ULONG_PTR)
            as PUSB_COMMON_DESCRIPTOR;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DumpConfigurationDescriptor(
    mut ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
) {
    DbgPrint(
        b"Dumping ConfigurationDescriptor %x\n\0".as_ptr() as *const i8,
        ConfigurationDescriptor,
    );
    DbgPrint(
        b"bLength %x\n\0".as_ptr() as *const i8,
        (*ConfigurationDescriptor).bLength as core::ffi::c_int,
    );
    DbgPrint(
        b"bDescriptorType %x\n\0".as_ptr() as *const i8,
        (*ConfigurationDescriptor).bDescriptorType as core::ffi::c_int,
    );
    DbgPrint(
        b"wTotalLength %x\n\0".as_ptr() as *const i8,
        (*ConfigurationDescriptor).wTotalLength as core::ffi::c_int,
    );
    DbgPrint(
        b"bNumInterfaces %x\n\0".as_ptr() as *const i8,
        (*ConfigurationDescriptor).bNumInterfaces as core::ffi::c_int,
    );
    DbgPrint(
        b"bConfigurationValue %x\n\0".as_ptr() as *const i8,
        (*ConfigurationDescriptor).bConfigurationValue as core::ffi::c_int,
    );
    DbgPrint(
        b"iConfiguration %x\n\0".as_ptr() as *const i8,
        (*ConfigurationDescriptor).iConfiguration as core::ffi::c_int,
    );
    DbgPrint(
        b"bmAttributes %x\n\0".as_ptr() as *const i8,
        (*ConfigurationDescriptor).bmAttributes as core::ffi::c_int,
    );
    DbgPrint(
        b"MaxPower %x\n\0".as_ptr() as *const i8,
        (*ConfigurationDescriptor).MaxPower as core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "system" fn USBCCGP_ScanConfigurationDescriptor(
    FDODeviceExtension: PFDO_DEVICE_EXTENSION,
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
) -> NTSTATUS {
    let mut InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR;
    let mut InterfaceIndex: ULONG = 0;
    let DescriptorCount: ULONG;

    //
    // sanity checks
    //
    if !(!ConfigurationDescriptor.is_null()) {
        RtlAssert(
            b"ConfigurationDescriptor\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/descriptor.c\0".as_ptr() as *const i8 as PVOID,
            322 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    if !((*ConfigurationDescriptor).bNumInterfaces != 0) {
        RtlAssert(
            b"ConfigurationDescriptor->bNumInterfaces\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/descriptor.c\0".as_ptr() as *const i8 as PVOID,
            323 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    //
    // count all interface descriptors
    //
    DescriptorCount = (*ConfigurationDescriptor).bNumInterfaces as ULONG;
    if DescriptorCount == 0 {
        // DPRINT1("[USBCCGP] DescriptorCount is zero\n");
        return STATUS_INVALID_PARAMETER;
    }

    //
    // allocate array holding the interface descriptors
    //
    (*FDODeviceExtension).InterfaceList = AllocateItem(
        NonPagedPool,
        (core::mem::size_of::<USBD_INTERFACE_LIST_ENTRY>() as ULONG)
            .wrapping_mul(DescriptorCount.wrapping_add(1)),
    ) as PUSBD_INTERFACE_LIST_ENTRY;
    if (*FDODeviceExtension).InterfaceList.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // reset interface list count
    //
    (*FDODeviceExtension).InterfaceListCount = 0;

    loop {
        //
        // parse configuration descriptor
        //
        InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(
            ConfigurationDescriptor,
            ConfigurationDescriptor as PVOID,
            InterfaceIndex as LONG,
            -1 as LONG,
            -1 as LONG,
            -1 as LONG,
            -1 as LONG,
        );
        if !InterfaceDescriptor.is_null() {
            //
            // store in interface list
            //
            if !((*FDODeviceExtension).InterfaceListCount < DescriptorCount) {
                RtlAssert(
                    b"FDODeviceExtension->InterfaceListCount < DescriptorCount\0".as_ptr() as *const i8 as PVOID,
                    b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/descriptor.c\0".as_ptr() as *const i8 as PVOID,
                    363 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            } else {
            }
            (*(*FDODeviceExtension)
                .InterfaceList
                .offset((*FDODeviceExtension).InterfaceListCount as isize))
            .InterfaceDescriptor = InterfaceDescriptor;
            (*FDODeviceExtension).InterfaceListCount =
                (*FDODeviceExtension).InterfaceListCount.wrapping_add(1);
        } else {
            DumpConfigurationDescriptor(ConfigurationDescriptor);
            DumpFullConfigurationDescriptor(FDODeviceExtension, ConfigurationDescriptor);

            //
            // see issue
            // CORE-6574 Test 3 (USB Web Cam)
            //
            if !(*FDODeviceExtension).DeviceDescriptor.is_null()
                && (*(*FDODeviceExtension).DeviceDescriptor).idVendor as i32 == 0x0458
                && (*(*FDODeviceExtension).DeviceDescriptor).idProduct as i32 == 0x705f
            {
                if !(false) {
                    RtlAssert(
                        b"FALSE\0".as_ptr() as *const i8 as PVOID,
                        b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/descriptor.c\0".as_ptr() as *const i8 as PVOID,
                        377 as ULONG,
                        core::ptr::null_mut::<CHAR>(),
                    );
                } else {
                }
            }
        }

        //
        // move to next interface
        //
        InterfaceIndex = InterfaceIndex.wrapping_add(1);

        if !(InterfaceIndex < DescriptorCount) {
            break;
        }
    }

    //
    // sanity check
    //
    if !((*FDODeviceExtension).InterfaceListCount != 0) {
        RtlAssert(
            b"FDODeviceExtension->InterfaceListCount\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/descriptor.c\0".as_ptr() as *const i8 as PVOID,
            390 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    //
    // done
    //
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_SelectInterface(
    DeviceObject: PDEVICE_OBJECT,
    DeviceExtension: PFDO_DEVICE_EXTENSION,
    InterfaceIndex: ULONG,
) -> NTSTATUS {

    let mut Status: NTSTATUS;
    let mut Urb: PURB;

    //
    // allocate urb
    //
    Urb = AllocateItem(
        NonPagedPool,
        (core::mem::size_of::<_URB_SELECT_INTERFACE>()).wrapping_add(
            (((*(*(*DeviceExtension)
                .InterfaceList
                .offset(InterfaceIndex as isize))
            .InterfaceDescriptor)
                .bNumEndpoints as i32
                - 1) as usize)
                .wrapping_mul(core::mem::size_of::<USBD_PIPE_INFORMATION>()),
        ) as ULONG,
    ) as PURB;
    if Urb.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // now prepare interface urb
    //
    UsbBuildSelectInterfaceRequest(
        Urb,
        (core::mem::size_of::<_URB_SELECT_INTERFACE>()).wrapping_add(
            (((*(*(*DeviceExtension)
                .InterfaceList
                .offset(InterfaceIndex as isize))
            .InterfaceDescriptor)
                .bNumEndpoints as i32
                - 1) as usize)
                .wrapping_mul(core::mem::size_of::<USBD_PIPE_INFORMATION>()),
        ) as USHORT,
        (*DeviceExtension).ConfigurationHandle,
        (*(*(*DeviceExtension)
            .InterfaceList
            .offset(InterfaceIndex as isize))
        .InterfaceDescriptor)
            .bInterfaceNumber,
        (*(*(*DeviceExtension)
            .InterfaceList
            .offset(InterfaceIndex as isize))
        .InterfaceDescriptor)
            .bAlternateSetting,
    );

    //
    // now select the interface
    //
    Status = USBCCGP_SyncUrbRequest((*DeviceExtension).NextDeviceObject, Urb);

    //
    // did it succeed
    //
    if NT_SUCCESS(Status) {
        //
        // update configuration info
        //
        if !((*(*Urb).__bindgen_anon_1.UrbSelectInterface.as_ref()).Interface.Length as i32
            == (*(*(*DeviceExtension)
                .InterfaceList
                .offset(InterfaceIndex as isize))
            .Interface)
                .Length as i32)
        {
            RtlAssert(
                b"Urb->UrbSelectInterface.Interface.Length == DeviceExtension->InterfaceList[InterfaceIndex].Interface->Length\0"
                    as *const u8 as *const i8 as PVOID,
                b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/descriptor.c\0"
                    as *const u8 as *const i8 as PVOID,
                451 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
        RtlCopyMemory(
            (*(*DeviceExtension)
                .InterfaceList
                .offset(InterfaceIndex as isize))
            .Interface as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*(*Urb).__bindgen_anon_1.UrbSelectInterface.as_mut()).Interface)
                as *const core::ffi::c_void,
            (*(*Urb).__bindgen_anon_1.UrbSelectInterface.as_ref()).Interface.Length as usize,
        );
    }

    //
    // free urb
    //
    FreeItem(Urb as PVOID);

    //
    // done
    //
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_SelectConfiguration(
    DeviceObject: PDEVICE_OBJECT,
    DeviceExtension: PFDO_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut InterfaceInformation: PUSBD_INTERFACE_INFORMATION;
    let mut Status: NTSTATUS;
    let Urb: PURB;
    let mut Index: ULONG;

    //
    // now scan configuration descriptors
    //
    Status = USBCCGP_ScanConfigurationDescriptor(
        DeviceExtension,
        (*DeviceExtension).ConfigurationDescriptor,
    );
    if !NT_SUCCESS(Status) {
        //
        // failed to scan
        //
        return Status;
    }

    //
    // now allocate the urb
    //
    Urb = USBD_CreateConfigurationRequestEx(
        (*DeviceExtension).ConfigurationDescriptor,
        (*DeviceExtension).InterfaceList,
    );
    if Urb.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // submit urb
    //
    Status = USBCCGP_SyncUrbRequest((*DeviceExtension).NextDeviceObject, Urb);
    if !NT_SUCCESS(Status) {
        //
        // failed to set configuration
        //
        DbgPrint(
            b"USBCCGP_SyncUrbRequest failed to set interface %x\n\0".as_ptr() as *const i8,
            Status,
        );
        ExFreePool(Urb as PVOID);
        return Status;
    }

    //
    // get interface information
    //
    InterfaceInformation = core::ptr::addr_of_mut!(
        (*(*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_mut()).Interface
    ) as PUSBD_INTERFACE_INFORMATION;
    Index = 0;
    while Index < (*DeviceExtension).InterfaceListCount {
        //
        // allocate buffer to store interface information
        //
        (*(*DeviceExtension).InterfaceList.offset(Index as isize)).Interface =
            AllocateItem(NonPagedPool, (*InterfaceInformation).Length as ULONG)
                as PUSBD_INTERFACE_INFORMATION;
        if (*(*DeviceExtension).InterfaceList.offset(Index as isize))
            .Interface
            .is_null()
        {
            //
            // no memory
            //
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        //
        // copy interface information
        //
        RtlCopyMemory(
            (*(*DeviceExtension).InterfaceList.offset(Index as isize)).Interface as PVOID,
            InterfaceInformation as PVOID,
            (*InterfaceInformation).Length as usize,
        );

        //
        // move to next interface
        //
        InterfaceInformation = (InterfaceInformation as usize)
            .wrapping_add((*InterfaceInformation).Length as usize)
            as PUSBD_INTERFACE_INFORMATION;

        Index = Index.wrapping_add(1);
    }

    //
    // store pipe handle
    //
    (*DeviceExtension).ConfigurationHandle =
        (*(*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref()).ConfigurationHandle;

    //
    // free interface list & urb
    //
    ExFreePool(Urb as PVOID);

    //
    // done
    //
    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Universal Serial Bus Bulk Enhanced Host Controller Interface
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        drivers/usb/usbccgp/descriptor.c
//  * PURPOSE:     USB  device driver.
//  * PROGRAMMERS:
//  *              Michael Martin (michael.martin@reactos.org)
//  *              Johannes Anderwald (johannes.anderwald@reactos.org)
//  *              Cameron Gutman
//  */
//
// #include "usbccgp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// NTSTATUS
// NTAPI
// USBCCGP_GetDescriptor(
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
//     //
//     // sanity checks
//     //
//     ASSERT(DeviceObject);
//     ASSERT(OutDescriptor);
//     ASSERT(DescriptorLength);
//
//     //
//     // first allocate descriptor buffer
//     //
//     Descriptor = AllocateItem(NonPagedPool, DescriptorLength);
//     if (!Descriptor)
//     {
//         //
//         // no memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // allocate urb
//     //
//     Urb = (PURB) AllocateItem(NonPagedPool, sizeof(URB));
//     if (!Urb)
//     {
//         //
//         // no memory
//         //
//         FreeItem(Descriptor);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // initialize urb
//     //
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
//     //
//     // submit urb
//     //
//     Status = USBCCGP_SyncUrbRequest(DeviceObject, Urb);
//
//     //
//     // free urb
//     //
//     FreeItem(Urb);
//
//     if (NT_SUCCESS(Status))
//     {
//         //
//         // store result
//         //
//         *OutDescriptor = Descriptor;
//     }
//
//     //
//     // done
//     //
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBCCGP_GetStringDescriptor(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN ULONG DescriptorLength,
//     IN UCHAR DescriptorIndex,
//     IN LANGID LanguageId,
//     OUT PVOID *OutDescriptor)
// {
//     NTSTATUS Status;
//     PUSB_STRING_DESCRIPTOR StringDescriptor;
//     ULONG Size;
//     PVOID Buffer;
//
//     // retrieve descriptor
//     Status = USBCCGP_GetDescriptor(DeviceObject, USB_STRING_DESCRIPTOR_TYPE, DescriptorLength, DescriptorIndex, LanguageId, OutDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         // failed
//         return Status;
//     }
//
//     // get descriptor structure
//     StringDescriptor = (PUSB_STRING_DESCRIPTOR)*OutDescriptor;
//
//     // sanity check
//     ASSERT(StringDescriptor->bLength < DescriptorLength - 2);
//
//     if (StringDescriptor->bLength == 2)
//     {
//         // invalid descriptor
//         FreeItem(StringDescriptor);
//         return STATUS_DEVICE_DATA_ERROR;
//     }
//
//     // calculate size
//     Size = StringDescriptor->bLength + sizeof(WCHAR);
//
//     // allocate buffer
//     Buffer = AllocateItem(NonPagedPool, Size);
//     if (!Buffer)
//     {
//         // no memory
//         FreeItem(StringDescriptor);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // copy result
//     RtlCopyMemory(Buffer, StringDescriptor->bString, Size - FIELD_OFFSET(USB_STRING_DESCRIPTOR, bString));
//
//     // free buffer
//     FreeItem(StringDescriptor);
//
//     // store result
//     *OutDescriptor = (PVOID)Buffer;
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// USBCCGP_GetDescriptors(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     NTSTATUS Status;
//     PFDO_DEVICE_EXTENSION DeviceExtension;
//     USHORT DescriptorLength;
//
//     //
//     // get device extension
//     //
//     DeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//      //
//      // first get device descriptor
//      //
//      Status = USBCCGP_GetDescriptor(DeviceExtension->NextDeviceObject, USB_DEVICE_DESCRIPTOR_TYPE, sizeof(USB_DEVICE_DESCRIPTOR), 0, 0, (PVOID*)&DeviceExtension->DeviceDescriptor);
//      if (!NT_SUCCESS(Status))
//      {
//          //
//          // failed to get device descriptor
//          //
//          DeviceExtension->DeviceDescriptor = NULL;
//          return Status;
//      }
//
//      //
//      // now get basic configuration descriptor
//      //
//      Status = USBCCGP_GetDescriptor(DeviceExtension->NextDeviceObject, USB_CONFIGURATION_DESCRIPTOR_TYPE, sizeof(USB_CONFIGURATION_DESCRIPTOR), 0, 0, (PVOID*)&DeviceExtension->ConfigurationDescriptor);
//      if (!NT_SUCCESS(Status))
//      {
//          //
//          // failed to get configuration descriptor
//          //
//          FreeItem(DeviceExtension->DeviceDescriptor);
//          DeviceExtension->DeviceDescriptor = NULL;
//          return Status;
//      }
//
//      //
//      // backup length
//      //
//      DescriptorLength = DeviceExtension->ConfigurationDescriptor->wTotalLength;
//
//      //
//      // release basic descriptor
//      //
//      FreeItem(DeviceExtension->ConfigurationDescriptor);
//      DeviceExtension->ConfigurationDescriptor = NULL;
//
//      //
//      // allocate full descriptor
//      //
//      Status = USBCCGP_GetDescriptor(DeviceExtension->NextDeviceObject, USB_CONFIGURATION_DESCRIPTOR_TYPE, DescriptorLength, 0, 0, (PVOID*)&DeviceExtension->ConfigurationDescriptor);
//      if (!NT_SUCCESS(Status))
//      {
//          //
//          // failed to get configuration descriptor
//          //
//          FreeItem(DeviceExtension->DeviceDescriptor);
//          DeviceExtension->DeviceDescriptor = NULL;
//          return Status;
//      }
//      return Status;
// }
//
// NTSTATUS
// AllocateInterfaceDescriptorsArray(
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     OUT PUSB_INTERFACE_DESCRIPTOR **OutArray)
// {
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor;
//     ULONG Count = 0;
//     PUSB_INTERFACE_DESCRIPTOR *Array;
//
//     //
//     // allocate array
//     //
//     Array = AllocateItem(NonPagedPool, sizeof(PUSB_INTERFACE_DESCRIPTOR) * ConfigurationDescriptor->bNumInterfaces);
//     if (!Array)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     //
//     // enumerate all interfaces
//     //
//     Count = 0;
//     do
//     {
//         //
//         // find next descriptor
//         //
//         InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, ConfigurationDescriptor, Count, 0, -1, -1, -1);
//         if (!InterfaceDescriptor)
//             break;
//
//         //
//         // store descriptor
//         //
//         Array[Count] = InterfaceDescriptor;
//         Count++;
//
//     }while(TRUE);
//
//     //
//     // store result
//     //
//     *OutArray = Array;
//
//     //
//     // done
//     //
//     return STATUS_SUCCESS;
// }
//
// VOID
// DumpFullConfigurationDescriptor(
//     IN PFDO_DEVICE_EXTENSION FDODeviceExtension,
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor)
// {
//     PUSB_COMMON_DESCRIPTOR Descriptor;
//
//     Descriptor = (PUSB_COMMON_DESCRIPTOR)ConfigurationDescriptor;
//
//     DbgPrint("Bogus ConfigurationDescriptor Found\n");
//     DbgPrint("InterfaceCount %x\n", ConfigurationDescriptor->bNumInterfaces);
//
//     do
//     {
//         if (((ULONG_PTR)Descriptor) >= ((ULONG_PTR)ConfigurationDescriptor + ConfigurationDescriptor->wTotalLength))
//             break;
//
//         DbgPrint("Descriptor Type %x Length %lu Offset %lu\n", Descriptor->bDescriptorType, Descriptor->bLength, ((ULONG_PTR)Descriptor - (ULONG_PTR)ConfigurationDescriptor));
//
//         // check for invalid descriptors
//         if (!Descriptor->bLength)
//         {
//             DbgPrint("Bogus Descriptor!!!\n");
//             break;
//         }
//
//         // advance to next descriptor
//         Descriptor = (PUSB_COMMON_DESCRIPTOR)((ULONG_PTR)Descriptor + Descriptor->bLength);
//
//     }while(TRUE);
//
//
// }
//
//
// NTSTATUS
// NTAPI
// USBCCGP_ScanConfigurationDescriptor(
//     IN OUT PFDO_DEVICE_EXTENSION FDODeviceExtension,
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor)
// {
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor;
//     ULONG InterfaceIndex = 0;
//     ULONG DescriptorCount;
//
//     //
//     // sanity checks
//     //
//     ASSERT(ConfigurationDescriptor);
//     ASSERT(ConfigurationDescriptor->bNumInterfaces);
//
//     //
//     // count all interface descriptors
//     //
//     DescriptorCount = ConfigurationDescriptor->bNumInterfaces;
//     if (DescriptorCount == 0)
//     {
//         DPRINT1("[USBCCGP] DescriptorCount is zero\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     // allocate array holding the interface descriptors
//     //
//     FDODeviceExtension->InterfaceList = AllocateItem(NonPagedPool, sizeof(USBD_INTERFACE_LIST_ENTRY) * (DescriptorCount + 1));
//     if (!FDODeviceExtension->InterfaceList)
//     {
//         //
//         // no memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // reset interface list count
//     //
//     FDODeviceExtension->InterfaceListCount = 0;
//
//     do
//     {
//         //
//         // parse configuration descriptor
//         //
//         InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, ConfigurationDescriptor, InterfaceIndex, -1, -1, -1, -1);
//         if (InterfaceDescriptor)
//         {
//             //
//             // store in interface list
//             //
//             ASSERT(FDODeviceExtension->InterfaceListCount < DescriptorCount);
//             FDODeviceExtension->InterfaceList[FDODeviceExtension->InterfaceListCount].InterfaceDescriptor = InterfaceDescriptor;
//             FDODeviceExtension->InterfaceListCount++;
//         }
//         else
//         {
//             DumpConfigurationDescriptor(ConfigurationDescriptor);
//             DumpFullConfigurationDescriptor(FDODeviceExtension, ConfigurationDescriptor);
//
//             //
//             // see issue
//             // CORE-6574 Test 3 (USB Web Cam)
//             //
//             if (FDODeviceExtension->DeviceDescriptor && FDODeviceExtension->DeviceDescriptor->idVendor == 0x0458 && FDODeviceExtension->DeviceDescriptor->idProduct == 0x705f)
//                 ASSERT(FALSE);
//         }
//
//         //
//         // move to next interface
//         //
//         InterfaceIndex++;
//
//     }while(InterfaceIndex < DescriptorCount);
//
//     //
//     // sanity check
//     //
//     ASSERT(FDODeviceExtension->InterfaceListCount);
//
//     //
//     // done
//     //
//     return STATUS_SUCCESS;
// }
//
// VOID
// DumpConfigurationDescriptor(PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor)
// {
//     DbgPrint("Dumping ConfigurationDescriptor %x\n", ConfigurationDescriptor);
//     DbgPrint("bLength %x\n", ConfigurationDescriptor->bLength);
//     DbgPrint("bDescriptorType %x\n", ConfigurationDescriptor->bDescriptorType);
//     DbgPrint("wTotalLength %x\n", ConfigurationDescriptor->wTotalLength);
//     DbgPrint("bNumInterfaces %x\n", ConfigurationDescriptor->bNumInterfaces);
//     DbgPrint("bConfigurationValue %x\n", ConfigurationDescriptor->bConfigurationValue);
//     DbgPrint("iConfiguration %x\n", ConfigurationDescriptor->iConfiguration);
//     DbgPrint("bmAttributes %x\n", ConfigurationDescriptor->bmAttributes);
//     DbgPrint("MaxPower %x\n", ConfigurationDescriptor->MaxPower);
// }
//
// NTSTATUS
// USBCCGP_SelectInterface(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PFDO_DEVICE_EXTENSION DeviceExtension,
//     IN ULONG InterfaceIndex)
// {
//     NTSTATUS Status;
//     PURB Urb;
//
//     //
//     // allocate urb
//     //
//     Urb = AllocateItem(NonPagedPool, GET_SELECT_INTERFACE_REQUEST_SIZE(DeviceExtension->InterfaceList[InterfaceIndex].InterfaceDescriptor->bNumEndpoints));
//     if (!Urb)
//     {
//         //
//         // no memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // now prepare interface urb
//     //
//     UsbBuildSelectInterfaceRequest(Urb, GET_SELECT_INTERFACE_REQUEST_SIZE(DeviceExtension->InterfaceList[InterfaceIndex].InterfaceDescriptor->bNumEndpoints), DeviceExtension->ConfigurationHandle, DeviceExtension->InterfaceList[InterfaceIndex].InterfaceDescriptor->bInterfaceNumber, DeviceExtension->InterfaceList[InterfaceIndex].InterfaceDescriptor->bAlternateSetting);
//
//     //
//     // now select the interface
//     //
//     Status = USBCCGP_SyncUrbRequest(DeviceExtension->NextDeviceObject, Urb);
//
//     //
//     // did it succeed
//     //
//     if (NT_SUCCESS(Status))
//     {
//         //
//         // update configuration info
//         //
//         ASSERT(Urb->UrbSelectInterface.Interface.Length == DeviceExtension->InterfaceList[InterfaceIndex].Interface->Length);
//         RtlCopyMemory(DeviceExtension->InterfaceList[InterfaceIndex].Interface, &Urb->UrbSelectInterface.Interface, Urb->UrbSelectInterface.Interface.Length);
//     }
//
//     //
//     // free urb
//     //
//     FreeItem(Urb);
//
//     //
//     // done
//     //
//     return Status;
// }
//
// NTSTATUS
// USBCCGP_SelectConfiguration(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PFDO_DEVICE_EXTENSION DeviceExtension)
// {
//     PUSBD_INTERFACE_INFORMATION InterfaceInformation;
//     NTSTATUS Status;
//     PURB Urb;
//     ULONG Index;
//
//     //
//     // now scan configuration descriptors
//     //
//     Status = USBCCGP_ScanConfigurationDescriptor(DeviceExtension, DeviceExtension->ConfigurationDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         //
//         // failed to scan
//         //
//         return Status;
//     }
//
//     //
//     // now allocate the urb
//     //
//     Urb = USBD_CreateConfigurationRequestEx(DeviceExtension->ConfigurationDescriptor, DeviceExtension->InterfaceList);
//     if (!Urb)
//     {
//         //
//         // no memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // submit urb
//     //
//     Status = USBCCGP_SyncUrbRequest(DeviceExtension->NextDeviceObject, Urb);
//     if (!NT_SUCCESS(Status))
//     {
//         //
//         // failed to set configuration
//         //
//         DPRINT1("USBCCGP_SyncUrbRequest failed to set interface %x\n", Status);
//         ExFreePool(Urb);
//         return Status;
//     }
//
//     //
//     // get interface information
//     //
//     InterfaceInformation = &Urb->UrbSelectConfiguration.Interface;
//     for(Index = 0; Index < DeviceExtension->InterfaceListCount; Index++)
//     {
//         //
//         // allocate buffer to store interface information
//         //
//         DeviceExtension->InterfaceList[Index].Interface = AllocateItem(NonPagedPool, InterfaceInformation->Length);
//         if (!DeviceExtension->InterfaceList[Index].Interface)
//         {
//             //
//             // no memory
//             //
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         //
//         // copy interface information
//         //
//         RtlCopyMemory(DeviceExtension->InterfaceList[Index].Interface, InterfaceInformation, InterfaceInformation->Length);
//
//         //
//         // move to next interface
//         //
//         InterfaceInformation = (PUSBD_INTERFACE_INFORMATION)((ULONG_PTR)InterfaceInformation + InterfaceInformation->Length);
//     }
//
//
//     //
//     // store pipe handle
//     //
//     DeviceExtension->ConfigurationHandle = Urb->UrbSelectConfiguration.ConfigurationHandle;
//
//     //
//     // free interface list & urb
//     //
//     ExFreePool(Urb);
//
//     //
//     // done
//     //
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================