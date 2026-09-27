use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_BUS_QUERY_ID_TYPE_BusQueryCompatibleIDs as BusQueryCompatibleIDs, _BUS_QUERY_ID_TYPE_BusQueryDeviceID as BusQueryDeviceID, _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs as BusQueryHardwareIDs, _BUS_QUERY_ID_TYPE_BusQueryInstanceID as BusQueryInstanceID, DEVICE_CAPABILITIES, DEVICE_RELATIONS, _DEVICE_TEXT_TYPE_DeviceTextDescription as DeviceTextDescription, ExFreePool, IoDeleteDevice, IoForwardIrpSynchronously, IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation, KeBugCheck, LPWSTR, PDEVICE_OBJECT, PDEVICE_RELATIONS, PIO_STACK_LOCATION, PIRP, PUNICODE_STRING, _DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation};
use ntkrnl_sys::ddk::LANGID;
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "C" {
    fn AllocateItem(PoolType: u32, NumberOfBytes: ULONG) -> *mut core::ffi::c_void;
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> core::ffi::c_int;
    fn wcslen(str: *const u16) -> usize;
}
extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> u32;
    fn FreeItem(Item: PVOID);
    fn USBCCGP_SyncUrbRequest(DeviceObject: PDEVICE_OBJECT, UrbRequest: PURB) -> NTSTATUS;
    fn UsbBuildSelectInterfaceRequest( Urb: PURB, Length: USHORT, ConfigurationHandle: USBD_CONFIGURATION_HANDLE, InterfaceNumber: UCHAR, AlternateSetting: UCHAR, );
}
unsafe extern "system" {
    fn USBCCGP_GetDescriptor( DeviceObject: PDEVICE_OBJECT, DescriptorType: UCHAR, DescriptorLength: ULONG, DescriptorIndex: UCHAR, LanguageId: LANGID, OutDescriptor: *mut PVOID, ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG  — empty object-like macro, no constant value

#[no_mangle]
pub unsafe extern "C" fn USBCCGP_PdoHandleQueryDeviceText(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Buffer: LPWSTR;
    let PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let GenericString: LPWSTR = [
        b'C' as u16, b'o' as u16, b'm' as u16, b'p' as u16, b'o' as u16, b's' as u16,
        b'i' as u16, b't' as u16, b'e' as u16, b' ' as u16, b'U' as u16, b'S' as u16,
        b'B' as u16, b' ' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16,
        b'c' as u16, b'e' as u16, 0u16,
    ].as_ptr() as LPWSTR;

    //
    // get current irp stack location
    //
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    //
    // get device extension
    //
    PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

    //
    // check if type is description
    //
    if (*IoStack).Parameters.QueryDeviceText.DeviceTextType as u32
        != DeviceTextDescription as u32
    {
        //
        // we only handle description
        //
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    //
    // is there a device description
    //
    if (*(*PDODeviceExtension).FunctionDescriptor).FunctionDescription.Length != 0 {
        //
        // allocate buffer
        //
        Buffer = AllocateItem(
            NonPagedPool as u32,
            ((*(*PDODeviceExtension).FunctionDescriptor).FunctionDescription.Length as usize)
                .wrapping_add(core::mem::size_of::<u16>()) as ULONG,
        ) as LPWSTR;
        if Buffer.is_null() {
            //
            // no memory
            //
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        //
        // copy buffer
        //
        (*Irp).IoStatus.Information = Buffer as ULONG_PTR as _;
        RtlCopyMemory(
            Buffer as *mut core::ffi::c_void,
            (*(*PDODeviceExtension).FunctionDescriptor).FunctionDescription.Buffer as *const core::ffi::c_void,
            (*(*PDODeviceExtension).FunctionDescriptor).FunctionDescription.Length as usize,
        );
        return STATUS_SUCCESS;
    }

    //
    // FIXME use GenericCompositeUSBDeviceString
    //
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"USBCCGP_PdoHandleQueryDeviceText\0".as_ptr() as *const i8,
        b"pdo.c\0".as_ptr() as *const i8,
        78,
    );
    Buffer = AllocateItem(
        PagedPool as u32,
        (wcslen(GenericString as *const u16).wrapping_add(1))
            .wrapping_mul(core::mem::size_of::<u16>()) as ULONG,
    ) as LPWSTR;
    if Buffer.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlCopyMemory(
        Buffer as *mut core::ffi::c_void,
        GenericString as *const core::ffi::c_void,
        (wcslen(GenericString as *const u16).wrapping_add(1))
            .wrapping_mul(core::mem::size_of::<u16>()),
    );
    (*Irp).IoStatus.Information = Buffer as ULONG_PTR as _;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_PdoHandleDeviceRelations(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceRelations: PDEVICE_RELATIONS = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).Parameters.QueryDeviceRelations.Type as core::ffi::c_uint
        != TargetDeviceRelation as core::ffi::c_int as core::ffi::c_uint
    {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    DeviceRelations =
        AllocateItem(PagedPool, core::mem::size_of::<DEVICE_RELATIONS>() as ULONG)
            as PDEVICE_RELATIONS;
    if DeviceRelations.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*DeviceRelations).Count = 1 as ULONG;
    let fresh1 = &mut *(core::ptr::addr_of_mut!((*DeviceRelations).Objects)
        as *mut PDEVICE_OBJECT)
        .offset(0 as core::ffi::c_int as isize);
    *fresh1 = DeviceObject;
    ObReferenceObject(DeviceObject as PVOID);

    (*Irp).IoStatus.Information = DeviceRelations as ULONG_PTR as _;

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn USBCCGP_PdoAppendInterfaceNumber(
    mut DeviceId: LPWSTR,
    mut InterfaceNumber: ULONG,
    mut OutString: *mut LPWSTR,
) -> NTSTATUS {
    let mut Length: ULONG = 0 as ULONG;
    let mut StringLength: ULONG;
    let mut String: LPWSTR;

    //
    // count length of string
    //
    String = DeviceId;
    while *String != 0 {
        StringLength = (wcslen(String) + 1) as ULONG;
        Length += StringLength;
        Length += 6; //&MI_XX
        String = String.offset(StringLength as isize);
    }

    //
    // now allocate the buffer
    //
    String = AllocateItem(
        NonPagedPool,
        ((Length + 2) as usize * core::mem::size_of::<WCHAR>()) as ULONG,
    ) as LPWSTR;
    if String.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // store result
    //
    *OutString = String;

    while *DeviceId != 0 {
        StringLength = (swprintf(
            String,
            w!("%s&MI_%02x").as_ptr() as *const u16,
            DeviceId,
            InterfaceNumber,
        ) + 1) as ULONG;
        Length = (wcslen(DeviceId) + 1) as ULONG;
        // DPRINT("String %p\n", String);

        //
        // next string
        //
        String = String.offset(StringLength as isize);
        DeviceId = DeviceId.offset(Length as isize);
    }

    //
    // success
    //
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_PdoHandleQueryId(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let IoStack: PIO_STACK_LOCATION;
    let mut DeviceString: PUNICODE_STRING = core::ptr::null_mut();
    let PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;
    let mut Buffer: LPWSTR;

    //
    // get current irp stack location
    //
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    //
    // get device extension
    //
    PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

    if (*IoStack).Parameters.QueryId.IdType as u32 == BusQueryDeviceID as u32 {
        //
        // handle query device id
        //
        if IoForwardIrpSynchronously((*PDODeviceExtension).NextDeviceObject, Irp) != 0 {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        } else {
            Status = STATUS_UNSUCCESSFUL;
        }

        if NT_SUCCESS(Status) {
            //
            // allocate buffer
            //
            Buffer = AllocateItem(
                NonPagedPool as u32,
                ((wcslen((*Irp).IoStatus.Information as LPWSTR as *const u16) + 7)
                    * core::mem::size_of::<u16>()) as ULONG,
            ) as LPWSTR;
            if !Buffer.is_null() {
                //
                // append interface number
                //
                let fmt_mi: [u16; 11] = [
                    b'%' as u16, b's' as u16, b'&' as u16, b'M' as u16, b'I' as u16,
                    b'_' as u16, b'%' as u16, b'0' as u16, b'2' as u16, b'x' as u16, 0,
                ];
                swprintf(
                    Buffer,
                    fmt_mi.as_ptr(),
                    (*Irp).IoStatus.Information as LPWSTR,
                    (*(*PDODeviceExtension).FunctionDescriptor).FunctionNumber as i32,
                );

                ExFreePool((*Irp).IoStatus.Information as PVOID);
                (*Irp).IoStatus.Information = Buffer as ULONG_PTR as _;
            } else {
                //
                // no memory
                //
                Status = STATUS_INSUFFICIENT_RESOURCES;
            }
        }
        return Status;
    } else if (*IoStack).Parameters.QueryId.IdType as u32 == BusQueryHardwareIDs as u32 {
        //
        // handle instance id
        //
        DeviceString =
            core::ptr::addr_of_mut!((*(*PDODeviceExtension).FunctionDescriptor).HardwareId)
                as PUNICODE_STRING;
    } else if (*IoStack).Parameters.QueryId.IdType as u32 == BusQueryInstanceID as u32 {
        //
        // handle instance id
        //
        Buffer = AllocateItem(NonPagedPool as u32, (5 * core::mem::size_of::<u16>()) as ULONG)
            as LPWSTR;
        if Buffer.is_null() {
            //
            // no memory
            //
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        //
        // use function number
        //
        let fmt_04x: [u16; 5] = [
            b'%' as u16, b'0' as u16, b'4' as u16, b'x' as u16, 0,
        ];
        swprintf(
            Buffer,
            fmt_04x.as_ptr(),
            (*(*PDODeviceExtension).FunctionDescriptor).FunctionNumber as i32,
        );
        (*Irp).IoStatus.Information = Buffer as ULONG_PTR as _;
        return STATUS_SUCCESS;
    } else if (*IoStack).Parameters.QueryId.IdType as u32 == BusQueryCompatibleIDs as u32 {
        //
        // handle instance id
        //
        DeviceString =
            core::ptr::addr_of_mut!((*(*PDODeviceExtension).FunctionDescriptor).CompatibleId)
                as PUNICODE_STRING;
    } else {
        //
        // unsupported query
        //
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    //
    // allocate buffer
    //
    Buffer = AllocateItem(
        NonPagedPool as u32,
        ((*DeviceString).Length as usize + core::mem::size_of::<u16>()) as ULONG,
    ) as LPWSTR;
    if Buffer.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // copy buffer
    //
    RtlCopyMemory(
        Buffer as PVOID,
        (*DeviceString).Buffer as PVOID,
        (*DeviceString).Length as usize,
    );
    *Buffer.offset(((*DeviceString).Length as usize / core::mem::size_of::<u16>()) as isize) = 0;
    (*Irp).IoStatus.Information = Buffer as ULONG_PTR as _;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PDO_HandlePnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;
    let mut Index: ULONG;
    let mut bFound: ULONG;

    //
    // get current stack location
    //
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    //
    // get device extension
    //
    PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

    //
    // sanity check
    //
    // ASSERT(PDODeviceExtension->Common.IsFDO == FALSE);

    const IRP_MN_QUERY_INTERFACE: u8 = 0x08;
    const IRP_MN_QUERY_CAPABILITIES: u8 = 0x09;
    const IRP_MN_QUERY_DEVICE_TEXT: u8 = 0x0C;
    const IRP_MN_QUERY_ID: u8 = 0x13;

    match (*IoStack).MinorFunction {
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            //
            // handle device relations
            //
            Status = USBCCGP_PdoHandleDeviceRelations(DeviceObject, Irp);
        }
        IRP_MN_QUERY_DEVICE_TEXT => {
            //
            // handle query device text
            //
            Status = USBCCGP_PdoHandleQueryDeviceText(DeviceObject, Irp);
        }
        IRP_MN_QUERY_ID => {
            //
            // handle request
            //
            Status = USBCCGP_PdoHandleQueryId(DeviceObject, Irp);
        }
        IRP_MN_REMOVE_DEVICE => {
            //
            // remove us from the fdo's pdo list
            //
            bFound = 0;
            Index = 0;
            while Index < (*(*PDODeviceExtension).FDODeviceExtension).FunctionDescriptorCount {
                if *(*(*PDODeviceExtension).FDODeviceExtension)
                    .ChildPDO
                    .offset(Index as isize)
                    == DeviceObject
                {
                    //
                    // remove us
                    //
                    *(*(*PDODeviceExtension).FDODeviceExtension)
                        .ChildPDO
                        .offset(Index as isize) = core::ptr::null_mut();
                    bFound = 1;
                    break;
                }
                Index = Index.wrapping_add(1);
            }

            //
            // Complete the IRP
            //
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);

            if bFound != 0 {
                //
                // Delete the device object
                //
                IoDeleteDevice(DeviceObject);
            }
            return STATUS_SUCCESS;
        }
        IRP_MN_QUERY_CAPABILITIES => {
            //
            // copy device capabilities
            //
            RtlCopyMemory(
                (*IoStack).Parameters.DeviceCapabilities.Capabilities as *mut core::ffi::c_void,
                core::ptr::addr_of!((*PDODeviceExtension).Capabilities) as *const core::ffi::c_void,
                core::mem::size_of::<DEVICE_CAPABILITIES>(),
            );

            /* Complete the IRP */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return STATUS_SUCCESS;
        }
        IRP_MN_QUERY_REMOVE_DEVICE | IRP_MN_QUERY_STOP_DEVICE => {
            //
            // sure
            //
            Status = STATUS_SUCCESS;
        }
        IRP_MN_START_DEVICE => {
            //
            // no-op for PDO
            //
            Status = STATUS_SUCCESS;
        }
        IRP_MN_QUERY_INTERFACE => {
            //
            // forward to lower device object
            //
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*PDODeviceExtension).NextDeviceObject, Irp);
        }
        _ => {
            //
            // do nothing
            //
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    //
    // complete request
    //
    if Status != STATUS_PENDING {
        //
        // store result
        //
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;

        //
        // complete request
        //
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }

    //
    // done processing
    //
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_BuildConfigurationDescriptor(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR;
    let mut InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR;
    let mut TotalSize: ULONG;
    let mut Index: ULONG;
    let mut Size: ULONG;
    let mut Urb: PURB;
    let mut Buffer: PVOID;
    let mut BufferPtr: PUCHAR;
    let mut InterfaceNumber: UCHAR;

    //
    // get current stack location
    //
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    //
    // get device extension
    //
    PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

    //
    // get configuration descriptor
    //
    ConfigurationDescriptor = (*PDODeviceExtension).ConfigurationDescriptor;

    //
    // calculate size of configuration descriptor
    //
    TotalSize = core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>() as ULONG;

    Index = 0 as ULONG;
    while Index < (*(*PDODeviceExtension).FunctionDescriptor).NumberOfInterfaces as ULONG {
        //
        // get current interface descriptor
        //
        InterfaceDescriptor = *(*(*PDODeviceExtension).FunctionDescriptor)
            .InterfaceDescriptorList
            .offset(Index as isize);
        InterfaceNumber = (*InterfaceDescriptor).bInterfaceNumber;

        //
        // add to size and move to next descriptor
        //
        TotalSize = TotalSize.wrapping_add((*InterfaceDescriptor).bLength as ULONG);
        InterfaceDescriptor = (InterfaceDescriptor as ULONG_PTR)
            .wrapping_add((*InterfaceDescriptor).bLength as ULONG_PTR)
            as PUSB_INTERFACE_DESCRIPTOR;

        loop {
            if (InterfaceDescriptor as ULONG_PTR)
                >= (ConfigurationDescriptor as ULONG_PTR)
                    .wrapping_add((*ConfigurationDescriptor).wTotalLength as ULONG_PTR)
            {
                //
                // reached end of configuration descriptor
                //
                break;
            }

            //
            // association descriptors are removed
            //
            if (*InterfaceDescriptor).bDescriptorType as i32
                != USB_INTERFACE_ASSOCIATION_DESCRIPTOR_TYPE as i32
            {
                if (*InterfaceDescriptor).bDescriptorType as i32 == USB_INTERFACE_DESCRIPTOR_TYPE as i32 {
                    if InterfaceNumber as i32 != (*InterfaceDescriptor).bInterfaceNumber as i32 {
                        //
                        // reached next descriptor
                        //
                        break;
                    }

                    //
                    // include alternate descriptor
                    //
                }

                //
                // append size
                //
                TotalSize = TotalSize.wrapping_add((*InterfaceDescriptor).bLength as ULONG);
            }

            //
            // move to next descriptor
            //
            InterfaceDescriptor = (InterfaceDescriptor as ULONG_PTR)
                .wrapping_add((*InterfaceDescriptor).bLength as ULONG_PTR)
                as PUSB_INTERFACE_DESCRIPTOR;
        }

        Index = Index.wrapping_add(1);
    }

    //
    // now allocate temporary buffer for the configuration descriptor
    //
    Buffer = AllocateItem(NonPagedPool, TotalSize);
    if Buffer.is_null() {
        //
        // failed to allocate buffer
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // first copy the configuration descriptor
    //
    RtlCopyMemory(
        Buffer as *mut _,
        ConfigurationDescriptor as *const _,
        core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>(),
    );
    BufferPtr = (Buffer as ULONG_PTR)
        .wrapping_add((*ConfigurationDescriptor).bLength as ULONG_PTR)
        as PUCHAR;

    Index = 0 as ULONG;
    while Index < (*(*PDODeviceExtension).FunctionDescriptor).NumberOfInterfaces as ULONG {
        //
        // get current interface descriptor
        //
        InterfaceDescriptor = *(*(*PDODeviceExtension).FunctionDescriptor)
            .InterfaceDescriptorList
            .offset(Index as isize);
        InterfaceNumber = (*InterfaceDescriptor).bInterfaceNumber;

        //
        // copy descriptor and move to next descriptor
        //
        RtlCopyMemory(
            BufferPtr as *mut _,
            InterfaceDescriptor as *const _,
            (*InterfaceDescriptor).bLength as usize,
        );
        BufferPtr = BufferPtr.offset((*InterfaceDescriptor).bLength as i32 as isize);
        InterfaceDescriptor = (InterfaceDescriptor as ULONG_PTR)
            .wrapping_add((*InterfaceDescriptor).bLength as ULONG_PTR)
            as PUSB_INTERFACE_DESCRIPTOR;

        loop {
            if (InterfaceDescriptor as ULONG_PTR)
                >= (ConfigurationDescriptor as ULONG_PTR)
                    .wrapping_add((*ConfigurationDescriptor).wTotalLength as ULONG_PTR)
            {
                //
                // reached end of configuration descriptor
                //
                break;
            }

            //
            // association descriptors are removed
            //
            if (*InterfaceDescriptor).bDescriptorType as i32
                != USB_INTERFACE_ASSOCIATION_DESCRIPTOR_TYPE as i32
            {
                if (*InterfaceDescriptor).bDescriptorType as i32 == USB_INTERFACE_DESCRIPTOR_TYPE as i32 {
                    if InterfaceNumber as i32 != (*InterfaceDescriptor).bInterfaceNumber as i32 {
                        //
                        // reached next descriptor
                        //
                        break;
                    }

                    //
                    // include alternate descriptor
                    //
                }

                //
                // copy descriptor
                //
                RtlCopyMemory(
                    BufferPtr as *mut _,
                    InterfaceDescriptor as *const _,
                    (*InterfaceDescriptor).bLength as usize,
                );
                BufferPtr = BufferPtr.offset((*InterfaceDescriptor).bLength as i32 as isize);
            }

            //
            // move to next descriptor
            //
            InterfaceDescriptor = (InterfaceDescriptor as ULONG_PTR)
                .wrapping_add((*InterfaceDescriptor).bLength as ULONG_PTR)
                as PUSB_INTERFACE_DESCRIPTOR;
        }

        Index = Index.wrapping_add(1);
    }

    //
    // modify configuration descriptor
    //
    ConfigurationDescriptor = Buffer as PUSB_CONFIGURATION_DESCRIPTOR;
    (*ConfigurationDescriptor).wTotalLength = TotalSize as USHORT;
    (*ConfigurationDescriptor).bNumInterfaces =
        (*(*PDODeviceExtension).FunctionDescriptor).NumberOfInterfaces;

    //
    // get urb
    //
    Urb = (*IoStack).Parameters.Others.Argument1 as PURB;

    //
    // copy descriptor
    //
    Size = if TotalSize
        < (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_ref())
            .TransferBufferLength
    {
        TotalSize
    } else {
        (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_ref())
            .TransferBufferLength
    };
    RtlCopyMemory(
        (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_ref())
            .TransferBuffer as *mut _,
        Buffer as *const _,
        Size as usize,
    );

    //
    // store final size
    //
    (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut())
        .TransferBufferLength = Size;

    //
    // free buffer
    //
    FreeItem(Buffer);

    //
    // done
    //
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_PDOSelectConfiguration(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {

    let mut IoStack: PIO_STACK_LOCATION;
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut Urb: PURB;
    let mut NewUrb: PURB;
    let mut InterfaceInformation: PUSBD_INTERFACE_INFORMATION;
    let mut InterfaceIndex: ULONG;
    let mut Length: ULONG;
    let mut Entry: PUSBD_INTERFACE_LIST_ENTRY;
    let mut NeedSelect: ULONG;
    let mut FoundInterface: ULONG;
    let mut Status: NTSTATUS;

    //
    // get current stack location
    //
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    //
    // get device extension
    //
    PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

    //
    // get urb
    //
    Urb = (*IoStack).Parameters.Others.Argument1 as PURB;
    // ASSERT(Urb);

    //
    // is there already an configuration handle
    //
    if !(*(*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref()).ConfigurationHandle.is_null() {
        //
        // nothing to do
        //
        return STATUS_SUCCESS;
    }

    //
    // if there is no configuration descriptor, unconfigure the device
    //
    if (*(*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref()).ConfigurationDescriptor.is_null() {
        return STATUS_SUCCESS;
    }

    // available buffer length: FIELD_OFFSET(_URB_SELECT_CONFIGURATION,
    // Interface.Length), and Length is the interface record's first field. (The
    // translation once subtracted a further 16 here, so the count never reached
    // 0 and the loop went on to read past the URB: every composite function
    // but the first failed to start with STATUS_INVALID_PARAMETER.)
    Length = ((*(*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref()).Hdr.Length as usize)
        .saturating_sub(core::mem::offset_of!(_URB_SELECT_CONFIGURATION, Interface)) as ULONG;

    //
    // check all interfaces
    //
    InterfaceInformation =
        core::ptr::addr_of_mut!((*(*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_mut()).Interface) as PUSBD_INTERFACE_INFORMATION;

    Entry = core::ptr::null_mut();
    loop {
        // DPRINT1
        // ASSERT(InterfaceInformation->Length);

        //
        // search for the interface in the local interface list
        //
        FoundInterface = 0;
        InterfaceIndex = 0;
        while InterfaceIndex < (*(*PDODeviceExtension).FunctionDescriptor).NumberOfInterfaces as ULONG {
            if (**(*(*PDODeviceExtension).FunctionDescriptor)
                .InterfaceDescriptorList
                .offset(InterfaceIndex as isize))
            .bInterfaceNumber as i32
                == (*InterfaceInformation).InterfaceNumber as i32
            {
                // found interface entry
                FoundInterface = 1;
                break;
            } else {
                InterfaceIndex = InterfaceIndex.wrapping_add(1);
            }
        }

        if FoundInterface == 0 {
            //
            // invalid parameter
            //
            ntkrnl_sys::driver_ddk::DbgPrint(b"[USBCCGP] select config: interface %u is not in this function (%u interfaces)\n\0".as_ptr() as *const i8,
                (*InterfaceInformation).InterfaceNumber as u32, (*(*PDODeviceExtension).FunctionDescriptor).NumberOfInterfaces as u32);
            return STATUS_INVALID_PARAMETER;
        }

        //
        // now query the total interface list
        //
        Entry = core::ptr::null_mut();
        InterfaceIndex = 0;
        while InterfaceIndex < (*PDODeviceExtension).InterfaceListCount {
            if (*(*(*PDODeviceExtension).InterfaceList.offset(InterfaceIndex as isize)).Interface)
                .InterfaceNumber as i32
                == (*InterfaceInformation).InterfaceNumber as i32
            {
                //
                // found entry
                //
                Entry = (*PDODeviceExtension).InterfaceList.offset(InterfaceIndex as isize)
                    as PUSBD_INTERFACE_LIST_ENTRY;
            }
            InterfaceIndex = InterfaceIndex.wrapping_add(1);
        }

        //
        // sanity check
        //
        // ASSERT(Entry);
        if Entry.is_null() {
            //
            // corruption detected
            //
            KeBugCheck(0 as ULONG);
        }

        NeedSelect = 0;
        if (*(*Entry).InterfaceDescriptor).bAlternateSetting as i32
            == (*InterfaceInformation).AlternateSetting as i32
        {
            InterfaceIndex = 0;
            while InterfaceIndex < (*InterfaceInformation).NumberOfPipes {
                if (*(core::ptr::addr_of_mut!((*InterfaceInformation).Pipes)
                    as *mut USBD_PIPE_INFORMATION)
                    .offset(InterfaceIndex as isize))
                .MaximumTransferSize
                    != (*(core::ptr::addr_of_mut!((*(*Entry).Interface).Pipes)
                        as *mut USBD_PIPE_INFORMATION)
                        .offset(InterfaceIndex as isize))
                    .MaximumTransferSize
                {
                    //
                    // changed interface
                    //
                    NeedSelect = 1;
                }
                InterfaceIndex = InterfaceIndex.wrapping_add(1);
            }
        } else {
            //
            // need select as the interface number differ
            //
            NeedSelect = 1;
        }

        if NeedSelect == 0 {
            //
            // interface is already selected
            //
            // ASSERT(Length >= Entry->Interface->Length);
            RtlCopyMemory(
                InterfaceInformation as PVOID,
                (*Entry).Interface as PVOID,
                (*(*Entry).Interface).Length as usize,
            );

            //
            // adjust remaining buffer size
            //
            // ASSERT(Entry->Interface->Length);
            Length = Length.wrapping_sub((*(*Entry).Interface).Length as ULONG);

            //
            // move to next output interface information
            //
            InterfaceInformation = ((InterfaceInformation as usize)
                .wrapping_add((*(*Entry).Interface).Length as usize))
                as PUSBD_INTERFACE_INFORMATION;
        } else {
            //
            // select interface
            //
            // DPRINT1
            // ASSERT(InterfaceInformation->Length == Entry->Interface->Length);

            //
            // build urb
            //
            NewUrb = AllocateItem(
                NonPagedPool,
                (core::mem::size_of::<_URB_SELECT_INTERFACE>()
                    + (((*InterfaceInformation).NumberOfPipes as usize).wrapping_sub(1))
                        * core::mem::size_of::<USBD_PIPE_INFORMATION>())
                    as ULONG,
            ) as PURB;
            if NewUrb.is_null() {
                //
                // no memory
                //
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            //
            // now prepare interface urb
            //
            UsbBuildSelectInterfaceRequest(
                NewUrb,
                (core::mem::size_of::<_URB_SELECT_INTERFACE>()
                    + (((*InterfaceInformation).NumberOfPipes as usize).wrapping_sub(1))
                        * core::mem::size_of::<USBD_PIPE_INFORMATION>()) as USHORT,
                (*PDODeviceExtension).ConfigurationHandle,
                (*InterfaceInformation).InterfaceNumber,
                (*InterfaceInformation).AlternateSetting,
            );

            //
            // now select the interface
            //
            Status = USBCCGP_SyncUrbRequest((*PDODeviceExtension).NextDeviceObject, NewUrb);
            // DPRINT1

            if !NT_SUCCESS(Status) {
                //
                // failed
                //
                break;
            }

            //
            // update configuration info
            //
            // ASSERT(Entry->Interface->Length >= NewUrb->UrbSelectInterface.Interface.Length);
            // ASSERT(Length >= NewUrb->UrbSelectInterface.Interface.Length);
            RtlCopyMemory(
                (*Entry).Interface as PVOID,
                core::ptr::addr_of_mut!((*(*NewUrb).__bindgen_anon_1.UrbSelectInterface.as_mut()).Interface) as PVOID,
                (*(*NewUrb).__bindgen_anon_1.UrbSelectInterface.as_ref()).Interface.Length as usize,
            );

            //
            // update provided interface information
            //
            // ASSERT(Length >= Entry->Interface->Length);
            RtlCopyMemory(
                InterfaceInformation as PVOID,
                (*Entry).Interface as PVOID,
                (*(*Entry).Interface).Length as usize,
            );

            //
            // decrement remaining buffer size
            //
            // ASSERT(Entry->Interface->Length);
            Length = Length.wrapping_sub((*(*Entry).Interface).Length as ULONG);

            //
            // adjust output buffer offset
            //
            InterfaceInformation = ((InterfaceInformation as usize)
                .wrapping_add((*(*Entry).Interface).Length as usize))
                as PUSBD_INTERFACE_INFORMATION;

            //
            // free urb
            //
            FreeItem(NewUrb as PVOID);
        }

        // The C loops while Length != 0; also stop at a remainder too short to
        // hold another interface record, rather than read one past the URB.
        if (Length as usize) < core::mem::offset_of!(USBD_INTERFACE_INFORMATION, Pipes) {
            break;
        }
    }

    //
    // store configuration handle
    //
    (*(*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_mut()).ConfigurationHandle = (*PDODeviceExtension).ConfigurationHandle;

    // DPRINT1

    //
    // done
    //
    return STATUS_SUCCESS;
}

const IOCTL_INTERNAL_USB_SUBMIT_URB: u32 = 0x00220003;
const IOCTL_INTERNAL_USB_GET_PORT_STATUS: u32 = 0x00220013;
pub const IOCTL_INTERNAL_USB_RESET_PORT: u32 = 0x00220007;
pub const IOCTL_INTERNAL_USB_CYCLE_PORT: u32 = 0x0022001F;
const URB_FUNCTION_SELECT_CONFIGURATION: i32 = 0x0000;
const URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE: i32 = 0x000B;
const USB_DEVICE_DESCRIPTOR_TYPE: i32 = 0x01;
const USB_CONFIGURATION_DESCRIPTOR_TYPE: i32 = 0x02;
const USB_STRING_DESCRIPTOR_TYPE: i32 = 0x03;
const STATUS_INVALID_BUFFER_SIZE: NTSTATUS = 0xC0000206u32 as NTSTATUS;
#[no_mangle]
pub unsafe extern "C" fn PDO_HandleInternalDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;
    let mut Urb: PURB;

    //
    // get current stack location
    //
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    //
    // get device extension
    //
    PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

    if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_INTERNAL_USB_SUBMIT_URB as u32 {
        //
        // get urb
        //
        Urb = (*IoStack).Parameters.Others.Argument1 as PURB;
        // ASSERT(Urb);

        if (*(*Urb).__bindgen_anon_1.UrbHeader.as_ref()).Function as i32 == URB_FUNCTION_SELECT_CONFIGURATION {
            //
            // select configuration
            //
            Status = USBCCGP_PDOSelectConfiguration(DeviceObject, Irp);
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return Status;
        } else if (*(*Urb).__bindgen_anon_1.UrbHeader.as_ref()).Function as i32 == URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE {
            if (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_ref()).DescriptorType as i32 == USB_DEVICE_DESCRIPTOR_TYPE {
                //
                // is the buffer big enough
                //
                if ((*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_ref()).TransferBufferLength as usize)
                    < core::mem::size_of::<USB_DEVICE_DESCRIPTOR>()
                {
                    //
                    // invalid buffer size
                    //
                    (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut()).TransferBufferLength =
                        core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() as ULONG;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_BUFFER_SIZE;
                    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
                    return STATUS_INVALID_BUFFER_SIZE;
                }

                //
                // copy device descriptor
                //
                // ASSERT(Urb->UrbControlDescriptorRequest.TransferBuffer);
                RtlCopyMemory(
                    (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_ref()).TransferBuffer as PVOID,
                    core::ptr::addr_of_mut!((*PDODeviceExtension).DeviceDescriptor) as PVOID,
                    core::mem::size_of::<USB_DEVICE_DESCRIPTOR>(),
                );
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
                return STATUS_SUCCESS;
            } else if (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_ref()).DescriptorType as i32
                == USB_CONFIGURATION_DESCRIPTOR_TYPE
            {
                //
                // build configuration descriptor
                //
                Status = USBCCGP_BuildConfigurationDescriptor(DeviceObject, Irp);
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
                return Status;
            } else if (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_ref()).DescriptorType as i32
                == USB_STRING_DESCRIPTOR_TYPE
            {
                let mut StringDescriptor: PUSB_STRING_DESCRIPTOR;

                //
                // get the requested string descriptor
                //
                // ASSERT(Urb->UrbControlDescriptorRequest.TransferBuffer);
                StringDescriptor = core::ptr::null_mut();
                Status = USBCCGP_GetDescriptor(
                    (*(*PDODeviceExtension).FDODeviceExtension).NextDeviceObject,
                    USB_STRING_DESCRIPTOR_TYPE as UCHAR,
                    (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_ref()).TransferBufferLength,
                    (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_ref()).Index,
                    (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_ref()).LanguageId as LANGID,
                    core::ptr::addr_of_mut!(StringDescriptor) as *mut PVOID,
                );
                if NT_SUCCESS(Status) {
                    if (*StringDescriptor).bLength as i32 == 2 {
                        FreeItem(StringDescriptor as PVOID);
                        Status = STATUS_DEVICE_DATA_ERROR;
                    } else {
                        RtlCopyMemory(
                            (*(*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_ref()).TransferBuffer as PVOID,
                            core::ptr::addr_of_mut!((*StringDescriptor).bString) as PVOID,
                            ((*StringDescriptor).bLength as usize)
                                .wrapping_add(core::mem::size_of::<WCHAR>()),
                        );
                        FreeItem(StringDescriptor as PVOID);
                        Status = STATUS_SUCCESS;
                    }
                }
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
                return Status;
            }
        } else {
            IoSkipCurrentIrpStackLocation(Irp);
            Status = IoCallDriver((*PDODeviceExtension).NextDeviceObject, Irp);
            return Status;
        }
    } else if (*IoStack).Parameters.DeviceIoControl.IoControlCode
        == IOCTL_INTERNAL_USB_GET_PORT_STATUS as u32
    {
        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*PDODeviceExtension).NextDeviceObject, Irp);
        return Status;
    } else if (*IoStack).Parameters.DeviceIoControl.IoControlCode
        == IOCTL_INTERNAL_USB_RESET_PORT as u32
    {
        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*PDODeviceExtension).NextDeviceObject, Irp);
        return Status;
    } else if (*IoStack).Parameters.DeviceIoControl.IoControlCode
        == IOCTL_INTERNAL_USB_CYCLE_PORT as u32
    {
        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*PDODeviceExtension).NextDeviceObject, Irp);
        return Status;
    }

    // ASSERT(FALSE);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn PDO_HandlePower(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut IoStack: PIO_STACK_LOCATION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    match (*IoStack).MinorFunction {
        IRP_MN_SET_POWER | IRP_MN_QUERY_POWER => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        }
        _ => {}
    }

    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    PoStartNextPowerIrp(Irp);
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn PDO_Dispatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let Status: NTSTATUS;

    /* get stack location */
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    match (*IoStack).MajorFunction {
        IRP_MJ_PNP => return PDO_HandlePnp(DeviceObject, Irp),
        IRP_MJ_INTERNAL_DEVICE_CONTROL => {
            return PDO_HandleInternalDeviceControl(DeviceObject, Irp);
        }
        IRP_MJ_POWER => return PDO_HandlePower(DeviceObject, Irp),
        _ => {
            if DbgPrint(
                b"(%s:%d) PDO_Dispatch Function %x not implemented\n\0" as *const u8 as PCSTR,
                b"pdo.c\0" as *const u8 as *const i8,
                1145i32,
                (*IoStack).MajorFunction as i32,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"pdo.c\0" as *const u8 as *const i8,
                    1145i32,
                );
            }
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return Status;
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Universal Serial Bus Bulk Enhanced Host Controller Interface
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        drivers/usb/usbccgp/pdo.c
//  * PURPOSE:     USB  device driver.
//  * PROGRAMMERS:
//  *              Michael Martin (michael.martin@reactos.org)
//  *              Johannes Anderwald (johannes.anderwald@reactos.org)
//  *              Cameron Gutman
//  */
//
// #include "usbccgp.h"
//
// #include <ntddk.h>
//
// #define NDEBUG
// #include <debug.h>
//
// NTSTATUS
// USBCCGP_PdoHandleQueryDeviceText(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     LPWSTR Buffer;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     LPWSTR GenericString = L"Composite USB Device";
//
//     //
//     // get current irp stack location
//     //
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     //
//     // get device extension
//     //
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     //
//     // check if type is description
//     //
//     if (IoStack->Parameters.QueryDeviceText.DeviceTextType != DeviceTextDescription)
//     {
//         //
//         // we only handle description
//         //
//         return Irp->IoStatus.Status;
//     }
//
//     //
//     // is there a device description
//     //
//     if (PDODeviceExtension->FunctionDescriptor->FunctionDescription.Length)
//     {
//         //
//         // allocate buffer
//         //
//         Buffer = AllocateItem(NonPagedPool, PDODeviceExtension->FunctionDescriptor->FunctionDescription.Length + sizeof(WCHAR));
//         if (!Buffer)
//         {
//             //
//             // no memory
//             //
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         //
//         // copy buffer
//         //
//         Irp->IoStatus.Information = (ULONG_PTR)Buffer;
//         RtlCopyMemory(Buffer, PDODeviceExtension->FunctionDescriptor->FunctionDescription.Buffer, PDODeviceExtension->FunctionDescriptor->FunctionDescription.Length);
//         return STATUS_SUCCESS;
//     }
//
//     //
//     // FIXME use GenericCompositeUSBDeviceString
//     //
//     UNIMPLEMENTED;
//     Buffer = AllocateItem(PagedPool, (wcslen(GenericString) + 1) * sizeof(WCHAR));
//     if (!Buffer)
//     {
//         //
//         // no memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     RtlCopyMemory(Buffer, GenericString, (wcslen(GenericString) + 1) * sizeof(WCHAR));
//     Irp->IoStatus.Information = (ULONG_PTR)Buffer;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USBCCGP_PdoHandleDeviceRelations(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp)
// {
//     PDEVICE_RELATIONS DeviceRelations;
//     PIO_STACK_LOCATION IoStack;
//
//     DPRINT("USBCCGP_PdoHandleDeviceRelations\n");
//
//     //
//     // get current irp stack location
//     //
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     //
//     // check if relation type is BusRelations
//     //
//     if (IoStack->Parameters.QueryDeviceRelations.Type != TargetDeviceRelation)
//     {
//         //
//         // PDO handles only target device relation
//         //
//         return Irp->IoStatus.Status;
//     }
//
//     //
//     // allocate device relations
//     //
//     DeviceRelations = (PDEVICE_RELATIONS)AllocateItem(PagedPool, sizeof(DEVICE_RELATIONS));
//     if (!DeviceRelations)
//     {
//         //
//         // no memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // initialize device relations
//     //
//     DeviceRelations->Count = 1;
//     DeviceRelations->Objects[0] = DeviceObject;
//     ObReferenceObject(DeviceObject);
//
//     //
//     // store result
//     //
//     Irp->IoStatus.Information = (ULONG_PTR)DeviceRelations;
//
//     //
//     // completed successfully
//     //
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USBCCGP_PdoAppendInterfaceNumber(
//     IN LPWSTR DeviceId,
//     IN ULONG InterfaceNumber,
//     OUT LPWSTR *OutString)
// {
//     ULONG Length = 0, StringLength;
//     LPWSTR String;
//
//     //
//     // count length of string
//     //
//     String = DeviceId;
//     while (*String)
//     {
//         StringLength = wcslen(String) + 1;
//         Length += StringLength;
//         Length += 6; //&MI_XX
//         String += StringLength;
//     }
//
//     //
//     // now allocate the buffer
//     //
//     String = AllocateItem(NonPagedPool, (Length + 2) * sizeof(WCHAR));
//     if (!String)
//     {
//         //
//         // no memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // store result
//     //
//     *OutString = String;
//
//     while (*DeviceId)
//     {
//         StringLength = swprintf(String, L"%s&MI_%02x", DeviceId, InterfaceNumber) + 1;
//         Length = wcslen(DeviceId) + 1;
//         DPRINT("String %p\n", String);
//
//         //
//         // next string
//         //
//         String += StringLength;
//         DeviceId += Length;
//     }
//
//     //
//     // success
//     //
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// USBCCGP_PdoHandleQueryId(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PUNICODE_STRING DeviceString = NULL;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     NTSTATUS Status;
//     LPWSTR Buffer;
//
//     //
//     // get current irp stack location
//     //
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     //
//     // get device extension
//     //
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//
//     if (IoStack->Parameters.QueryId.IdType == BusQueryDeviceID)
//     {
//         //
//         // handle query device id
//         //
//         if (IoForwardIrpSynchronously(PDODeviceExtension->NextDeviceObject, Irp))
//         {
//             Status = Irp->IoStatus.Status;
//         }
//         else
//         {
//             Status = STATUS_UNSUCCESSFUL;
//         }
//
//         if (NT_SUCCESS(Status))
//         {
//             //
//             // allocate buffer
//             //
//             Buffer = AllocateItem(NonPagedPool, (wcslen((LPWSTR)Irp->IoStatus.Information) + 7) * sizeof(WCHAR));
//             if (Buffer)
//             {
//                 //
//                 // append interface number
//                 //
//                 ASSERT(Irp->IoStatus.Information);
//                 swprintf(Buffer, L"%s&MI_%02x", (LPWSTR)Irp->IoStatus.Information, PDODeviceExtension->FunctionDescriptor->FunctionNumber);
//                 DPRINT("BusQueryDeviceID %S\n", Buffer);
//
//                 ExFreePool((PVOID)Irp->IoStatus.Information);
//                 Irp->IoStatus.Information = (ULONG_PTR)Buffer;
//             }
//             else
//             {
//                 //
//                 // no memory
//                 //
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//             }
//         }
//         return Status;
//     }
//     else if (IoStack->Parameters.QueryId.IdType == BusQueryHardwareIDs)
//     {
//         //
//         // handle instance id
//         //
//         DeviceString = &PDODeviceExtension->FunctionDescriptor->HardwareId;
//     }
//     else if (IoStack->Parameters.QueryId.IdType == BusQueryInstanceID)
//     {
//         //
//         // handle instance id
//         //
//         Buffer = AllocateItem(NonPagedPool, 5 * sizeof(WCHAR));
//         if (!Buffer)
//         {
//             //
//             // no memory
//             //
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         //
//         // use function number
//         //
//         swprintf(Buffer, L"%04x", PDODeviceExtension->FunctionDescriptor->FunctionNumber);
//         Irp->IoStatus.Information = (ULONG_PTR)Buffer;
//         return STATUS_SUCCESS;
//     }
//     else if (IoStack->Parameters.QueryId.IdType == BusQueryCompatibleIDs)
//     {
//         //
//         // handle instance id
//         //
//         DeviceString = &PDODeviceExtension->FunctionDescriptor->CompatibleId;
//     }
//     else
//     {
//         //
//         // unsupported query
//         //
//         return Irp->IoStatus.Status;
//     }
//
//     //
//     // sanity check
//     //
//     ASSERT(DeviceString != NULL);
//
//     //
//     // allocate buffer
//     //
//     Buffer = AllocateItem(NonPagedPool, DeviceString->Length + sizeof(WCHAR));
//     if (!Buffer)
//     {
//         //
//         // no memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // copy buffer
//     //
//     RtlCopyMemory(Buffer, DeviceString->Buffer, DeviceString->Length);
//     Buffer[DeviceString->Length / sizeof(WCHAR)] = UNICODE_NULL;
//     Irp->IoStatus.Information = (ULONG_PTR)Buffer;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// PDO_HandlePnp(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     NTSTATUS Status;
//     ULONG Index, bFound;
//
//     //
//     // get current stack location
//     //
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     //
//     // get device extension
//     //
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     //
//     // sanity check
//     //
//     ASSERT(PDODeviceExtension->Common.IsFDO == FALSE);
//
//     switch(IoStack->MinorFunction)
//     {
//         case IRP_MN_QUERY_DEVICE_RELATIONS:
//         {
//             //
//             // handle device relations
//             //
//             Status = USBCCGP_PdoHandleDeviceRelations(DeviceObject, Irp);
//             break;
//         }
//         case IRP_MN_QUERY_DEVICE_TEXT:
//         {
//             //
//             // handle query device text
//             //
//             Status = USBCCGP_PdoHandleQueryDeviceText(DeviceObject, Irp);
//             break;
//         }
//         case IRP_MN_QUERY_ID:
//         {
//             //
//             // handle request
//             //
//             Status = USBCCGP_PdoHandleQueryId(DeviceObject, Irp);
//             break;
//         }
//         case IRP_MN_REMOVE_DEVICE:
//         {
//             //
//             // remove us from the fdo's pdo list
//             //
//             bFound = FALSE;
//             for(Index = 0; Index < PDODeviceExtension->FDODeviceExtension->FunctionDescriptorCount; Index++)
//             {
//                 if (PDODeviceExtension->FDODeviceExtension->ChildPDO[Index] == DeviceObject)
//                 {
//                     //
//                     // remove us
//                     //
//                     PDODeviceExtension->FDODeviceExtension->ChildPDO[Index] = NULL;
//                     bFound = TRUE;
//                     break;
//                 }
//             }
//
//             //
//             // Complete the IRP
//             //
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//             if (bFound)
//             {
//                 //
//                 // Delete the device object
//                 //
//                 IoDeleteDevice(DeviceObject);
//             }
//             return STATUS_SUCCESS;
//         }
//         case IRP_MN_QUERY_CAPABILITIES:
//         {
//             //
//             // copy device capabilities
//             //
//             RtlCopyMemory(IoStack->Parameters.DeviceCapabilities.Capabilities, &PDODeviceExtension->Capabilities, sizeof(DEVICE_CAPABILITIES));
//
//             /* Complete the IRP */
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return STATUS_SUCCESS;
//         }
//         case IRP_MN_QUERY_REMOVE_DEVICE:
//         case IRP_MN_QUERY_STOP_DEVICE:
//         {
//             //
//             // sure
//             //
//             Status = STATUS_SUCCESS;
//             break;
//         }
//         case IRP_MN_START_DEVICE:
//         {
//             //
//             // no-op for PDO
//             //
//             DPRINT("[USBCCGP] PDO IRP_MN_START\n");
//             Status = STATUS_SUCCESS;
//             break;
//         }
//         case IRP_MN_QUERY_INTERFACE:
//         {
//             //
//             // forward to lower device object
//             //
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(PDODeviceExtension->NextDeviceObject, Irp);
//         }
//         default:
//         {
//             //
//             // do nothing
//             //
//             Status = Irp->IoStatus.Status;
//             break;
//         }
//     }
//
//     //
//     // complete request
//     //
//     if (Status != STATUS_PENDING)
//     {
//         //
//         // store result
//         //
//         Irp->IoStatus.Status = Status;
//
//         //
//         // complete request
//         //
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     }
//
//     //
//     // done processing
//     //
//     return Status;
//
// }
//
// NTSTATUS
// USBCCGP_BuildConfigurationDescriptor(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor;
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor;
//     ULONG TotalSize, Index;
//     ULONG Size;
//     PURB Urb;
//     PVOID Buffer;
//     PUCHAR BufferPtr;
//     UCHAR InterfaceNumber;
//
//     //
//     // get current stack location
//     //
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("USBCCGP_BuildConfigurationDescriptor\n");
//
//     //
//     // get device extension
//     //
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     //
//     // get configuration descriptor
//     //
//     ConfigurationDescriptor = PDODeviceExtension->ConfigurationDescriptor;
//
//     //
//     // calculate size of configuration descriptor
//     //
//     TotalSize = sizeof(USB_CONFIGURATION_DESCRIPTOR);
//
//     for (Index = 0; Index < PDODeviceExtension->FunctionDescriptor->NumberOfInterfaces; Index++)
//     {
//         //
//         // get current interface descriptor
//         //
//         InterfaceDescriptor = PDODeviceExtension->FunctionDescriptor->InterfaceDescriptorList[Index];
//         InterfaceNumber = InterfaceDescriptor->bInterfaceNumber;
//
//         //
//         // add to size and move to next descriptor
//         //
//         TotalSize += InterfaceDescriptor->bLength;
//         InterfaceDescriptor = (PUSB_INTERFACE_DESCRIPTOR)((ULONG_PTR)InterfaceDescriptor + InterfaceDescriptor->bLength);
//
//         do
//         {
//             if ((ULONG_PTR)InterfaceDescriptor >= ((ULONG_PTR)ConfigurationDescriptor + ConfigurationDescriptor->wTotalLength))
//             {
//                 //
//                 // reached end of configuration descriptor
//                 //
//                 break;
//             }
//
//             //
//             // association descriptors are removed
//             //
//             if (InterfaceDescriptor->bDescriptorType != USB_INTERFACE_ASSOCIATION_DESCRIPTOR_TYPE as i32)
//             {
//                 if (InterfaceDescriptor->bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE)
//                 {
//                     if (InterfaceNumber != InterfaceDescriptor->bInterfaceNumber)
//                     {
//                         //
//                         // reached next descriptor
//                         //
//                         break;
//                     }
//
//                     //
//                     // include alternate descriptor
//                     //
//                 }
//
//                 //
//                 // append size
//                 //
//                 TotalSize += InterfaceDescriptor->bLength;
//             }
//
//             //
//             // move to next descriptor
//             //
//             InterfaceDescriptor = (PUSB_INTERFACE_DESCRIPTOR)((ULONG_PTR)InterfaceDescriptor + InterfaceDescriptor->bLength);
//         } while(TRUE);
//     }
//
//     //
//     // now allocate temporary buffer for the configuration descriptor
//     //
//     Buffer = AllocateItem(NonPagedPool, TotalSize);
//     if (!Buffer)
//     {
//         //
//         // failed to allocate buffer
//         //
//         DPRINT1("[USBCCGP] Failed to allocate %lu Bytes\n", TotalSize);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // first copy the configuration descriptor
//     //
//     RtlCopyMemory(Buffer, ConfigurationDescriptor, sizeof(USB_CONFIGURATION_DESCRIPTOR));
//     BufferPtr = (PUCHAR)((ULONG_PTR)Buffer + ConfigurationDescriptor->bLength);
//
//     for (Index = 0; Index < PDODeviceExtension->FunctionDescriptor->NumberOfInterfaces; Index++)
//     {
//         //
//         // get current interface descriptor
//         //
//         InterfaceDescriptor = PDODeviceExtension->FunctionDescriptor->InterfaceDescriptorList[Index];
//         InterfaceNumber = InterfaceDescriptor->bInterfaceNumber;
//
//         //
//         // copy descriptor and move to next descriptor
//         //
//         RtlCopyMemory(BufferPtr, InterfaceDescriptor, InterfaceDescriptor->bLength);
//         BufferPtr += InterfaceDescriptor->bLength;
//         InterfaceDescriptor = (PUSB_INTERFACE_DESCRIPTOR)((ULONG_PTR)InterfaceDescriptor + InterfaceDescriptor->bLength);
//
//         do
//         {
//             if ((ULONG_PTR)InterfaceDescriptor >= ((ULONG_PTR)ConfigurationDescriptor + ConfigurationDescriptor->wTotalLength))
//             {
//                 //
//                 // reached end of configuration descriptor
//                 //
//                 break;
//             }
//
//             //
//             // association descriptors are removed
//             //
//             if (InterfaceDescriptor->bDescriptorType != USB_INTERFACE_ASSOCIATION_DESCRIPTOR_TYPE as i32)
//             {
//                 if (InterfaceDescriptor->bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE)
//                 {
//                     if (InterfaceNumber != InterfaceDescriptor->bInterfaceNumber)
//                     {
//                         //
//                         // reached next descriptor
//                         //
//                         break;
//                     }
//
//                     //
//                     // include alternate descriptor
//                     //
//                     DPRINT("InterfaceDescriptor %p Alternate %x InterfaceNumber %x\n", InterfaceDescriptor, InterfaceDescriptor->bAlternateSetting, InterfaceDescriptor->bInterfaceNumber);
//                 }
//
//                 //
//                 // copy descriptor
//                 //
//                 RtlCopyMemory(BufferPtr, InterfaceDescriptor, InterfaceDescriptor->bLength);
//                 BufferPtr += InterfaceDescriptor->bLength;
//             }
//
//             //
//             // move to next descriptor
//             //
//             InterfaceDescriptor = (PUSB_INTERFACE_DESCRIPTOR)((ULONG_PTR)InterfaceDescriptor + InterfaceDescriptor->bLength);
//         } while(TRUE);
//     }
//
//     //
//     // modify configuration descriptor
//     //
//     ConfigurationDescriptor = Buffer;
//     ConfigurationDescriptor->wTotalLength = (USHORT)TotalSize;
//     ConfigurationDescriptor->bNumInterfaces = PDODeviceExtension->FunctionDescriptor->NumberOfInterfaces;
//
//     //
//     // get urb
//     //
//     Urb = (PURB)IoStack->Parameters.Others.Argument1;
//     ASSERT(Urb);
//
//     //
//     // copy descriptor
//     //
//     Size = min(TotalSize, Urb->UrbControlDescriptorRequest.TransferBufferLength);
//     RtlCopyMemory(Urb->UrbControlDescriptorRequest.TransferBuffer, Buffer, Size);
//
//     //
//     // store final size
//     //
//     Urb->UrbControlDescriptorRequest.TransferBufferLength = Size;
//
//     //
//     // free buffer
//     //
//     FreeItem(Buffer);
//
//     //
//     // done
//     //
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USBCCGP_PDOSelectConfiguration(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     PURB Urb, NewUrb;
//     PUSBD_INTERFACE_INFORMATION InterfaceInformation;
//     ULONG InterfaceIndex, Length;
//     PUSBD_INTERFACE_LIST_ENTRY Entry;
//     ULONG NeedSelect, FoundInterface;
//     NTSTATUS Status;
//
//     //
//     // get current stack location
//     //
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     //
//     // get device extension
//     //
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     //
//     // get urb
//     //
//     Urb = (PURB)IoStack->Parameters.Others.Argument1;
//     ASSERT(Urb);
//
//     //
//     // is there already an configuration handle
//     //
//     if (Urb->UrbSelectConfiguration.ConfigurationHandle)
//     {
//         //
//         // nothing to do
//         //
//         return STATUS_SUCCESS;
//     }
//
//     //
//     // if there is no configuration descriptor, unconfigure the device
//     //
//     if (Urb->UrbSelectConfiguration.ConfigurationDescriptor == NULL)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     // sanity checks
//     //C_ASSERT(sizeof(struct _URB_HEADER) == 16);
//     //C_ASSERT(FIELD_OFFSET(struct _URB_SELECT_CONFIGURATION, Interface.Length) == 24);
//     //C_ASSERT(sizeof(USBD_INTERFACE_INFORMATION) == 36);
//     //C_ASSERT(sizeof(struct _URB_SELECT_CONFIGURATION) == 0x3C);
//
//     // available buffer length
//     Length = Urb->UrbSelectConfiguration.Hdr.Length - FIELD_OFFSET(struct _URB_SELECT_CONFIGURATION, Interface.Length);
//
//     //
//     // check all interfaces
//     //
//     InterfaceInformation = &Urb->UrbSelectConfiguration.Interface;
//
//     Entry = NULL;
//     do
//     {
//         DPRINT1("[USBCCGP] SelectConfiguration Function %x InterfaceNumber %x Alternative %x Length %lu InterfaceInformation->Length %lu\n",
//                PDODeviceExtension->FunctionDescriptor->FunctionNumber, InterfaceInformation->InterfaceNumber, InterfaceInformation->AlternateSetting, Length, InterfaceInformation->Length);
//         ASSERT(InterfaceInformation->Length);
//         //
//         // search for the interface in the local interface list
//         //
//         FoundInterface = FALSE;
//         for (InterfaceIndex = 0; InterfaceIndex < PDODeviceExtension->FunctionDescriptor->NumberOfInterfaces; InterfaceIndex++)
//         {
//             if (PDODeviceExtension->FunctionDescriptor->InterfaceDescriptorList[InterfaceIndex]->bInterfaceNumber == InterfaceInformation->InterfaceNumber)
//             {
//                 // found interface entry
//                 FoundInterface = TRUE;
//                 break;
//             }
//         }
//
//         if (!FoundInterface)
//         {
//             //
//             // invalid parameter
//             //
//             DPRINT1("InterfaceInformation InterfaceNumber %x Alternative %x NumberOfPipes %x not found\n", InterfaceInformation->InterfaceNumber, InterfaceInformation->AlternateSetting, InterfaceInformation->NumberOfPipes);
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         //
//         // now query the total interface list
//         //
//         Entry = NULL;
//         for (InterfaceIndex = 0; InterfaceIndex < PDODeviceExtension->InterfaceListCount; InterfaceIndex++)
//         {
//             if (PDODeviceExtension->InterfaceList[InterfaceIndex].Interface->InterfaceNumber == InterfaceInformation->InterfaceNumber)
//             {
//                 //
//                 // found entry
//                 //
//                 Entry = &PDODeviceExtension->InterfaceList[InterfaceIndex];
//             }
//         }
//
//         //
//         // sanity check
//         //
//         ASSERT(Entry);
//         if (!Entry)
//         {
//             //
//             // corruption detected
//             //
//             KeBugCheck(0);
//         }
//
//         NeedSelect = FALSE;
//         if (Entry->InterfaceDescriptor->bAlternateSetting == InterfaceInformation->AlternateSetting)
//         {
//             for(InterfaceIndex = 0; InterfaceIndex < InterfaceInformation->NumberOfPipes; InterfaceIndex++)
//             {
//                 if (InterfaceInformation->Pipes[InterfaceIndex].MaximumTransferSize != Entry->Interface->Pipes[InterfaceIndex].MaximumTransferSize)
//                 {
//                     //
//                     // changed interface
//                     //
//                     NeedSelect = TRUE;
//                 }
//             }
//         }
//         else
//         {
//             //
//             // need select as the interface number differ
//             //
//             NeedSelect = TRUE;
//         }
//
//         if (!NeedSelect)
//         {
//             //
//             // interface is already selected
//             //
//             ASSERT(Length >= Entry->Interface->Length);
//             RtlCopyMemory(InterfaceInformation, Entry->Interface, Entry->Interface->Length);
//
//             //
//             // adjust remaining buffer size
//             //
//             ASSERT(Entry->Interface->Length);
//             Length -= Entry->Interface->Length;
//
//             //
//             // move to next output interface information
//             //
//             InterfaceInformation = (PUSBD_INTERFACE_INFORMATION)((ULONG_PTR)InterfaceInformation + Entry->Interface->Length);
//         }
//         else
//         {
//             //
//             // select interface
//             //
//             DPRINT1("Selecting InterfaceIndex %lu AlternateSetting %lu NumberOfPipes %lu\n", InterfaceInformation->InterfaceNumber, InterfaceInformation->AlternateSetting, InterfaceInformation->NumberOfPipes);
//             ASSERT(InterfaceInformation->Length == Entry->Interface->Length);
//
//             //
//             // build urb
//             //
//             NewUrb = AllocateItem(NonPagedPool, GET_SELECT_INTERFACE_REQUEST_SIZE(InterfaceInformation->NumberOfPipes));
//             if (!NewUrb)
//             {
//                 //
//                 // no memory
//                 //
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             //
//             // now prepare interface urb
//             //
//             UsbBuildSelectInterfaceRequest(NewUrb, (USHORT)GET_SELECT_INTERFACE_REQUEST_SIZE(InterfaceInformation->NumberOfPipes), PDODeviceExtension->ConfigurationHandle, InterfaceInformation->InterfaceNumber, InterfaceInformation->AlternateSetting);
//
//             //
//             // now select the interface
//             //
//             Status = USBCCGP_SyncUrbRequest(PDODeviceExtension->NextDeviceObject, NewUrb);
//             DPRINT1("SelectInterface Status %x\n", Status);
//
//             if (!NT_SUCCESS(Status))
//             {
//                  //
//                  // failed
//                  //
//                  break;
//             }
//
//             //
//             // update configuration info
//             //
//             ASSERT(Entry->Interface->Length >= NewUrb->UrbSelectInterface.Interface.Length);
//             ASSERT(Length >= NewUrb->UrbSelectInterface.Interface.Length);
//             RtlCopyMemory(Entry->Interface, &NewUrb->UrbSelectInterface.Interface, NewUrb->UrbSelectInterface.Interface.Length);
//
//             //
//             // update provided interface information
//             //
//             ASSERT(Length >= Entry->Interface->Length);
//             RtlCopyMemory(InterfaceInformation, Entry->Interface, Entry->Interface->Length);
//
//             //
//             // decrement remaining buffer size
//             //
//             ASSERT(Entry->Interface->Length);
//             Length -= Entry->Interface->Length;
//
//             //
//             // adjust output buffer offset
//             //
//             InterfaceInformation = (PUSBD_INTERFACE_INFORMATION)((ULONG_PTR)InterfaceInformation + Entry->Interface->Length);
//
//             //
//             // free urb
//             //
//             FreeItem(NewUrb);
//         }
//
//     } while(Length);
//
//     //
//     // store configuration handle
//     //
//     Urb->UrbSelectConfiguration.ConfigurationHandle = PDODeviceExtension->ConfigurationHandle;
//
//     DPRINT1("[USBCCGP] SelectConfiguration Function %x Completed\n", PDODeviceExtension->FunctionDescriptor->FunctionNumber);
//
//     //
//     // done
//     //
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// PDO_HandleInternalDeviceControl(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     NTSTATUS Status;
//     PURB Urb;
//
//     //
//     // get current stack location
//     //
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     //
//     // get device extension
//     //
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_INTERNAL_USB_SUBMIT_URB)
//     {
//         //
//         // get urb
//         //
//         Urb = (PURB)IoStack->Parameters.Others.Argument1;
//         ASSERT(Urb);
//         DPRINT("IOCTL_INTERNAL_USB_SUBMIT_URB Function %x\n", Urb->UrbHeader.Function);
//
//         if (Urb->UrbHeader.Function == URB_FUNCTION_SELECT_CONFIGURATION)
//         {
//             //
//             // select configuration
//             //
//             Status = USBCCGP_PDOSelectConfiguration(DeviceObject, Irp);
//             Irp->IoStatus.Status = Status;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return Status;
//         }
//         else if (Urb->UrbHeader.Function == URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE)
//         {
//             if (Urb->UrbControlDescriptorRequest.DescriptorType == USB_DEVICE_DESCRIPTOR_TYPE)
//             {
//                 //
//                 // is the buffer big enough
//                 //
//                 if (Urb->UrbControlDescriptorRequest.TransferBufferLength < sizeof(USB_DEVICE_DESCRIPTOR))
//                 {
//                     //
//                     // invalid buffer size
//                     //
//                     DPRINT1("[USBCCGP] invalid device descriptor size %lu\n", Urb->UrbControlDescriptorRequest.TransferBufferLength);
//                     Urb->UrbControlDescriptorRequest.TransferBufferLength = sizeof(USB_DEVICE_DESCRIPTOR);
//                     Irp->IoStatus.Status = STATUS_INVALID_BUFFER_SIZE;
//                     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//                     return STATUS_INVALID_BUFFER_SIZE;
//                 }
//
//                 //
//                 // copy device descriptor
//                 //
//                 ASSERT(Urb->UrbControlDescriptorRequest.TransferBuffer);
//                 RtlCopyMemory(Urb->UrbControlDescriptorRequest.TransferBuffer, &PDODeviceExtension->DeviceDescriptor, sizeof(USB_DEVICE_DESCRIPTOR));
//                 Irp->IoStatus.Status = STATUS_SUCCESS;
//                 IoCompleteRequest(Irp, IO_NO_INCREMENT);
//                 return STATUS_SUCCESS;
//             }
//             else if (Urb->UrbControlDescriptorRequest.DescriptorType == USB_CONFIGURATION_DESCRIPTOR_TYPE)
//             {
//                 //
//                 // build configuration descriptor
//                 //
//                 Status = USBCCGP_BuildConfigurationDescriptor(DeviceObject, Irp);
//                 Irp->IoStatus.Status = Status;
//                 IoCompleteRequest(Irp, IO_NO_INCREMENT);
//                 return Status;
//             }
//             else if (Urb->UrbControlDescriptorRequest.DescriptorType == USB_STRING_DESCRIPTOR_TYPE)
//             {
//                 PUSB_STRING_DESCRIPTOR StringDescriptor;
//
//                 //
//                 // get the requested string descriptor
//                 //
//                 ASSERT(Urb->UrbControlDescriptorRequest.TransferBuffer);
//                 Status = USBCCGP_GetDescriptor(PDODeviceExtension->FDODeviceExtension->NextDeviceObject,
//                                                USB_STRING_DESCRIPTOR_TYPE,
//                                                Urb->UrbControlDescriptorRequest.TransferBufferLength,
//                                                Urb->UrbControlDescriptorRequest.Index,
//                                                Urb->UrbControlDescriptorRequest.LanguageId,
//                                                (PVOID*)&StringDescriptor);
//                 if (NT_SUCCESS(Status))
//                 {
//                     if (StringDescriptor->bLength == 2)
//                     {
//                         FreeItem(StringDescriptor);
//                         Status = STATUS_DEVICE_DATA_ERROR;
//                     }
//                     else
//                     {
//                         RtlCopyMemory(Urb->UrbControlDescriptorRequest.TransferBuffer,
//                                       StringDescriptor->bString,
//                                       StringDescriptor->bLength + sizeof(WCHAR));
//                         FreeItem(StringDescriptor);
//                         Status = STATUS_SUCCESS;
//                     }
//                 }
//                 Irp->IoStatus.Status = Status;
//                 IoCompleteRequest(Irp, IO_NO_INCREMENT);
//                 return Status;
//             }
//         }
//         else
//         {
//             IoSkipCurrentIrpStackLocation(Irp);
//             Status = IoCallDriver(PDODeviceExtension->NextDeviceObject, Irp);
//             return Status;
//         }
//     }
//     else if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_INTERNAL_USB_GET_PORT_STATUS)
//     {
//         IoSkipCurrentIrpStackLocation(Irp);
//         Status = IoCallDriver(PDODeviceExtension->NextDeviceObject, Irp);
//         return Status;
//     }
//     else if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_INTERNAL_USB_RESET_PORT)
//     {
//         IoSkipCurrentIrpStackLocation(Irp);
//         Status = IoCallDriver(PDODeviceExtension->NextDeviceObject, Irp);
//         return Status;
//     }
//     else if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_INTERNAL_USB_CYCLE_PORT)
//     {
//         IoSkipCurrentIrpStackLocation(Irp);
//         Status = IoCallDriver(PDODeviceExtension->NextDeviceObject, Irp);
//         return Status;
//     }
//
//     DPRINT1("IOCTL %x\n", IoStack->Parameters.DeviceIoControl.IoControlCode);
//     DPRINT1("InputBufferLength %lu\n", IoStack->Parameters.DeviceIoControl.InputBufferLength);
//     DPRINT1("OutputBufferLength %lu\n", IoStack->Parameters.DeviceIoControl.OutputBufferLength);
//     DPRINT1("Type3InputBuffer %p\n", IoStack->Parameters.DeviceIoControl.Type3InputBuffer);
//
//     ASSERT(FALSE);
//
//     Irp->IoStatus.Status = STATUS_NOT_IMPLEMENTED;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// PDO_HandlePower(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStack;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     switch (IoStack->MinorFunction)
//     {
//         case IRP_MN_SET_POWER:
//         case IRP_MN_QUERY_POWER:
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             break;
//     }
//
//     Status = Irp->IoStatus.Status;
//     PoStartNextPowerIrp(Irp);
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
//
// NTSTATUS
// PDO_Dispatch(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//
//     /* get stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     switch(IoStack->MajorFunction)
//     {
//         case IRP_MJ_PNP:
//             return PDO_HandlePnp(DeviceObject, Irp);
//         case IRP_MJ_INTERNAL_DEVICE_CONTROL:
//             return PDO_HandleInternalDeviceControl(DeviceObject, Irp);
//         case IRP_MJ_POWER:
//             return PDO_HandlePower(DeviceObject, Irp);
//         default:
//             DPRINT1("PDO_Dispatch Function %x not implemented\n", IoStack->MajorFunction);
//             Status = Irp->IoStatus.Status;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return Status;
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================