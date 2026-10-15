use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{BUS_INTERFACE_STANDARD, _KWAIT_REASON_Executive, GUID, _IO_STATUS_BLOCK, IoBuildSynchronousFsdRequest, IoGetNextIrpStackLocation, KEVENT, KeInitializeEvent, KeWaitForSingleObject, LPGUID, LPWSTR, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PINTERFACE, PIO_STACK_LOCATION, PIRP, RtlInitUnicodeString};
// IRP_MN_QUERY_INTERFACE = 0x08 (xdk/iotypes.h:1929). hal_ddk is x86-only -> define the
// cross-arch const locally so arm64 links.
const IRP_MN_QUERY_INTERFACE: u8 = 0x08;
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
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
}
extern "system" {
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
}
// variadic — cdecl on all arches (never stdcall)
extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> u32;
}
extern "system" {
    fn USBCCGP_GetStringDescriptor( DeviceObject: PDEVICE_OBJECT, DescriptorLength: ULONG, DescriptorIndex: u8, LanguageId: u16, OutDescriptor: *mut PVOID, ) -> NTSTATUS;
    fn USBD_ParseConfigurationDescriptorEx( ConfigurationDescriptor: PVOID, StartPosition: PVOID, InterfaceNumber: LONG, AlternateSetting: LONG, InterfaceClass: LONG, InterfaceSubClass: LONG, InterfaceProtocol: LONG, ) -> PUSB_INTERFACE_DESCRIPTOR;
}
unsafe extern "C" {
    fn AllocateItem(PoolType: u32, NumberOfBytes: ULONG) -> *mut core::ffi::c_void;
}
unsafe extern "C" {
    fn FreeItem(Item: PVOID);
}
unsafe extern "C" {
    fn AllocateInterfaceDescriptorsArray( ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR, InterfaceDescriptorArray: *mut *mut PUSB_INTERFACE_DESCRIPTOR, ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG

// Faithful rendering of C `L"..."` wide-string literals in no_std (no Vec/alloc).
// The C source uses ASCII-only `L"..."` constants; each byte maps 1:1 to a UTF-16 unit.
macro_rules! wstr {
    ($s:literal) => {{
        const BYTES: &[u8] = $s;
        const N: usize = BYTES.len();
        const W: [u16; N] = {
            let mut a = [0u16; N];
            let mut i = 0;
            while i < N {
                a[i] = BYTES[i] as u16;
                i += 1;
            }
            a
        };
        W.as_ptr()
    }};
}

#[no_mangle]
pub unsafe extern "C" fn USBCCGP_QueryInterface(
    DeviceObject: PDEVICE_OBJECT,
    mut BusInterface: PUSBC_DEVICE_CONFIGURATION_INTERFACE_V1,
) -> NTSTATUS {

    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Irp: PIRP;
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Stack: PIO_STACK_LOCATION;

    /* Sanity checks */
    if DeviceObject.is_null() {
        RtlAssert(
            b"DeviceObject\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/function.c\0".as_ptr() as *const i8 as PVOID,
            29 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    /* Initialize event */
    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0 as BOOLEAN);

    /* Init interface */
    RtlZeroMemory(
        BusInterface as *mut core::ffi::c_void,
        core::mem::size_of::<USBC_DEVICE_CONFIGURATION_INTERFACE_V1>(),
    );
    (*BusInterface).Version = USBC_DEVICE_CONFIGURATION_INTERFACE_VERSION_1 as USHORT;
    (*BusInterface).Size = core::mem::size_of::<USBC_DEVICE_CONFIGURATION_INTERFACE_V1>() as USHORT;

    /* Create irp */
    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as ULONG,
        DeviceObject,
        core::ptr::null_mut(),
        0 as ULONG,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatus),
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Stack = IoGetNextIrpStackLocation(Irp);
    (*Stack).MajorFunction = IRP_MJ_PNP as UCHAR;
    (*Stack).MinorFunction = IRP_MN_QUERY_INTERFACE as UCHAR;
    (*Stack).Parameters.QueryInterface.Size = core::mem::size_of::<BUS_INTERFACE_STANDARD>() as USHORT;
    (*Stack).Parameters.QueryInterface.InterfaceType =
        core::ptr::addr_of!(USB_BUS_INTERFACE_USBC_CONFIGURATION_GUID) as LPGUID as *const GUID;
    (*Stack).Parameters.QueryInterface.Version = 2 as USHORT;
    (*Stack).Parameters.QueryInterface.Interface = core::ptr::addr_of_mut!(BusInterface) as PINTERFACE;
    (*Stack).Parameters.QueryInterface.InterfaceSpecificData = core::ptr::null_mut();
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    Status = IoCallDriver(DeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as KPROCESSOR_MODE,
            0 as BOOLEAN,
            core::ptr::null_mut(),
        );
        Status = IoStatus.__bindgen_anon_1.Status;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_CustomEnumWithInterface(
    mut DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut FunctionDescriptorBufferLength: ULONG = 0 as ULONG;
    let mut Status: NTSTATUS;
    let mut FunctionDescriptorBuffer: PUSBC_FUNCTION_DESCRIPTOR = core::ptr::null_mut();

    //
    // get device extension
    //
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    if (*FDODeviceExtension).Common.IsFDO == 0 {
        RtlAssert(
            b"FDODeviceExtension->Common.IsFDO\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/function.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            109 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    if (*FDODeviceExtension).BusInterface.StartDeviceCallback.is_none() {
        //
        // not supported
        //
        return STATUS_NOT_SUPPORTED;
    }

    //
    // invoke callback
    //
    Status = (*FDODeviceExtension).BusInterface.StartDeviceCallback.expect("non-null function pointer")(
        (*FDODeviceExtension).DeviceDescriptor,
        (*FDODeviceExtension).ConfigurationDescriptor,
        core::ptr::addr_of_mut!(FunctionDescriptorBuffer),
        core::ptr::addr_of_mut!(FunctionDescriptorBufferLength),
        DeviceObject,
        (*FDODeviceExtension).PhysicalDeviceObject,
    );

    if !NT_SUCCESS(Status) {
        //
        // failed
        //
        return Status;
    }

    //
    // assume length % function buffer size
    //
    if FunctionDescriptorBufferLength == 0 {
        RtlAssert(
            b"FunctionDescriptorBufferLength\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/function.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            144 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    if !((FunctionDescriptorBufferLength as usize)
        .wrapping_rem(core::mem::size_of::<USBC_FUNCTION_DESCRIPTOR>() as usize)
        == 0 as usize)
    {
        RtlAssert(
            b"FunctionDescriptorBufferLength % sizeof(USBC_FUNCTION_DESCRIPTOR) == 0\0" as *const u8
                as *const core::ffi::c_char as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/function.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            145 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    //
    // store result
    //
    (*FDODeviceExtension).FunctionDescriptor = FunctionDescriptorBuffer;
    (*FDODeviceExtension).FunctionDescriptorCount = (FunctionDescriptorBufferLength as usize)
        .wrapping_div(core::mem::size_of::<USBC_FUNCTION_DESCRIPTOR>() as usize)
        as ULONG;

    //
    // success
    //
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_CountAssociationDescriptors(
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
) -> ULONG {
    let mut Descriptor: PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR;
    let mut Offset: PUCHAR;
    let End: PUCHAR;
    let mut Count: ULONG = 0;

    //
    // init offsets
    //
    Offset = (ConfigurationDescriptor as PUCHAR)
        .offset((*ConfigurationDescriptor).bLength as isize);
    End = (ConfigurationDescriptor as PUCHAR)
        .offset((*ConfigurationDescriptor).wTotalLength as isize);

    while Offset < End {
        //
        // get association descriptor
        //
        Descriptor = Offset as PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR;

        if (*Descriptor).bLength as usize
            == core::mem::size_of::<USB_INTERFACE_ASSOCIATION_DESCRIPTOR>()
            && (*Descriptor).bDescriptorType as i32
                == USB_INTERFACE_ASSOCIATION_DESCRIPTOR_TYPE as i32
        {
            //
            // found descriptor
            //
            Count = Count.wrapping_add(1);
        }

        //
        // move to next descriptor
        //
        Offset = Offset.offset((*Descriptor).bLength as isize);
    }

    //
    // done
    //
    return Count;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_GetAssociationDescriptorAtIndex(
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    Index: ULONG,
) -> PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR {
    let mut Descriptor: PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR;
    let mut Offset: PUCHAR;
    let End: PUCHAR;
    let mut Count: ULONG = 0 as ULONG;

    //
    // init offsets
    //
    Offset = (ConfigurationDescriptor as PUCHAR)
        .offset((*ConfigurationDescriptor).bLength as isize);
    End = (ConfigurationDescriptor as PUCHAR)
        .offset((*ConfigurationDescriptor).wTotalLength as isize);

    while Offset < End {
        //
        // get association descriptor
        //
        Descriptor = Offset as PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR;

        if (*Descriptor).bLength as usize
            == core::mem::size_of::<USB_INTERFACE_ASSOCIATION_DESCRIPTOR>()
            && (*Descriptor).bDescriptorType as i32 == USB_INTERFACE_ASSOCIATION_DESCRIPTOR_TYPE as i32
        {
            if Index == Count {
                //
                // found descriptor
                //
                return Descriptor;
            }

            //
            // not the searched one
            //
            Count = Count.wrapping_add(1);
        }

        //
        // move to next descriptor
        //
        Offset = Offset.offset((*Descriptor).bLength as isize);
    }

    //
    // failed to find descriptor at the specified index
    //
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn USBCCGP_InitInterfaceListOfFunctionDescriptor(
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    AssociationDescriptor: PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR,
    FunctionDescriptor: PUSBC_FUNCTION_DESCRIPTOR,
) -> NTSTATUS {

    let mut Descriptor: PUSB_INTERFACE_DESCRIPTOR;
    let mut Offset: PUCHAR;
    let End: PUCHAR;
    let mut Count: ULONG = 0;

    //
    // init offsets
    //
    Offset = (AssociationDescriptor as PUCHAR)
        .offset((*AssociationDescriptor).bLength as isize);
    End = (ConfigurationDescriptor as PUCHAR)
        .offset((*ConfigurationDescriptor).wTotalLength as isize);

    while Offset < End {
        //
        // get association descriptor
        //
        Descriptor = Offset as PUSB_INTERFACE_DESCRIPTOR;

        if (*Descriptor).bLength as usize == core::mem::size_of::<USB_INTERFACE_DESCRIPTOR>()
            && (*Descriptor).bDescriptorType as i32 == USB_INTERFACE_DESCRIPTOR_TYPE as i32
        {
            //
            // store interface descriptor
            //
            *(*FunctionDescriptor).InterfaceDescriptorList.offset(Count as isize) = Descriptor;
            Count = Count.wrapping_add(1);

            if Count == (*AssociationDescriptor).bInterfaceCount as ULONG {
                //
                // got all interfaces
                //
                return STATUS_SUCCESS;
            }
        }

        if (*Descriptor).bLength as usize
            == core::mem::size_of::<USB_INTERFACE_ASSOCIATION_DESCRIPTOR>()
            && (*Descriptor).bDescriptorType as i32 == USB_INTERFACE_ASSOCIATION_DESCRIPTOR_TYPE as i32
        {
            //
            // WTF? a association descriptor which overlaps the next association descriptor
            //
            if DbgPrint(
                b"(%s:%d) Invalid association descriptor\n\0" as *const u8 as PCSTR,
                b"function.c\0" as *const u8 as *const core::ffi::c_char,
                295 as i32,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"function.c\0" as *const u8 as *const core::ffi::c_char,
                    295 as i32,
                );
            }
            if 0 == 0 {
                RtlAssert(
                    b"FALSE\0" as *const u8 as *const core::ffi::c_char as PVOID,
                    b"function.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
                    296 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            } else {
            }
            return STATUS_UNSUCCESSFUL;
        }

        //
        // move to next descriptor
        //
        Offset = Offset.offset((*Descriptor).bLength as isize);
    }

    //
    // invalid association descriptor
    //
    if DbgPrint(
        b"(%s:%d) Invalid association descriptor\n\0" as *const u8 as PCSTR,
        b"function.c\0" as *const u8 as *const core::ffi::c_char,
        309 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"function.c\0" as *const u8 as *const core::ffi::c_char,
            309 as i32,
        );
    }
    return STATUS_UNSUCCESSFUL;
}

#[no_mangle]
pub unsafe extern "C" fn USBCCGP_InitFunctionDescriptor(
    FDODeviceExtension: PFDO_DEVICE_EXTENSION,
    FunctionNumber: ULONG,
    FunctionDescriptor: PUSBC_FUNCTION_DESCRIPTOR,
) -> NTSTATUS {
    let mut Descriptor: PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR;
    let mut Status: NTSTATUS;
    let mut DescriptionBuffer: LPWSTR = core::ptr::null_mut();
    let mut Buffer: [WCHAR; 100] = [0; 100];
    let mut Index: ULONG;

    // init function number
    (*FunctionDescriptor).FunctionNumber = FunctionNumber as UCHAR;

    // get association descriptor
    Descriptor = USBCCGP_GetAssociationDescriptorAtIndex(
        (*FDODeviceExtension).ConfigurationDescriptor,
        FunctionNumber,
    );
    // ASSERT(Descriptor);

    // store number interfaces
    (*FunctionDescriptor).NumberOfInterfaces = (*Descriptor).bInterfaceCount;

    // allocate array for interface count
    (*FunctionDescriptor).InterfaceDescriptorList = AllocateItem(
        NonPagedPool,
        (core::mem::size_of::<PUSB_INTERFACE_DESCRIPTOR>()
            * (*Descriptor).bInterfaceCount as usize) as ULONG,
    ) as *mut PUSB_INTERFACE_DESCRIPTOR;
    if (*FunctionDescriptor).InterfaceDescriptorList.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // init interface list
    Status = USBCCGP_InitInterfaceListOfFunctionDescriptor(
        (*FDODeviceExtension).ConfigurationDescriptor,
        Descriptor,
        FunctionDescriptor,
    );
    if !NT_SUCCESS(Status) {
        //
        // failed
        //
        return Status;
    }

    //
    // now init interface description
    //
    if (*Descriptor).iFunction != 0 {
        //
        // get interface description
        //
        Status = USBCCGP_GetStringDescriptor(
            (*FDODeviceExtension).NextDeviceObject,
            (100 * core::mem::size_of::<WCHAR>()) as ULONG,
            (*Descriptor).iFunction,
            0x0409, //FIXME
            core::ptr::addr_of_mut!(DescriptionBuffer) as *mut PVOID,
        );
        if !NT_SUCCESS(Status) {
            //
            // no description
            //
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!((*FunctionDescriptor).FunctionDescription),
                w!("").as_ptr() as PCWSTR,
            );
        } else {
            //
            // init description
            //
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!((*FunctionDescriptor).FunctionDescription),
                DescriptionBuffer as PCWSTR,
            );
        }
        // DPRINT1("FunctionDescription %wZ\n", ...);
    }

    //
    // now init hardware id
    //
    Index = (swprintf(
        Buffer.as_mut_ptr(),
        (&[
            0x55u16, 0x53, 0x42, 0x5c, 0x56, 0x49, 0x44, 0x5f, 0x25, 0x30, 0x34, 0x78, 0x26,
            0x50, 0x49, 0x44, 0x5f, 0x25, 0x30, 0x34, 0x78, 0x26, 0x52, 0x65, 0x76, 0x5f,
            0x25, 0x30, 0x34, 0x78, 0x26, 0x4d, 0x49, 0x5f, 0x25, 0x30, 0x32, 0x78, 0x00,
        ]) as *const WCHAR,
        (*(*FDODeviceExtension).DeviceDescriptor).idVendor as ::core::ffi::c_int,
        (*(*FDODeviceExtension).DeviceDescriptor).idProduct as ::core::ffi::c_int,
        (*(*FDODeviceExtension).DeviceDescriptor).bcdDevice as ::core::ffi::c_int,
        (*Descriptor).bFirstInterface as ::core::ffi::c_int,
    ) + 1) as ULONG;
    Index = Index.wrapping_add((swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        (&[
            0x55u16, 0x53, 0x42, 0x5c, 0x56, 0x49, 0x44, 0x5f, 0x25, 0x30, 0x34, 0x78, 0x26,
            0x50, 0x49, 0x44, 0x5f, 0x25, 0x30, 0x34, 0x78, 0x26, 0x4d, 0x49, 0x5f, 0x25,
            0x30, 0x32, 0x78, 0x00,
        ]) as *const WCHAR,
        (*(*FDODeviceExtension).DeviceDescriptor).idVendor as ::core::ffi::c_int,
        (*(*FDODeviceExtension).DeviceDescriptor).idProduct as ::core::ffi::c_int,
        (*Descriptor).bFirstInterface as ::core::ffi::c_int,
    ) + 1) as ULONG);

    // allocate result buffer
    DescriptionBuffer = AllocateItem(
        NonPagedPool,
        ((Index + 1) as usize * core::mem::size_of::<WCHAR>()) as ULONG,
    ) as LPWSTR;
    if DescriptionBuffer.is_null() {
        //
        // failed to allocate memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // copy description
    RtlCopyMemory(
        DescriptionBuffer as *mut ::core::ffi::c_void,
        Buffer.as_ptr() as *const ::core::ffi::c_void,
        (Index + 1) as usize * core::mem::size_of::<WCHAR>(),
    );
    (*FunctionDescriptor).HardwareId.Buffer = DescriptionBuffer as PWCH;
    (*FunctionDescriptor).HardwareId.Length =
        (Index as usize * core::mem::size_of::<WCHAR>()) as USHORT;
    (*FunctionDescriptor).HardwareId.MaximumLength =
        ((Index + 1) as usize * core::mem::size_of::<WCHAR>()) as USHORT;

    //
    // now init the compatible id
    //
    Index = (swprintf(
        Buffer.as_mut_ptr(),
        (&[
            0x55u16, 0x53, 0x42, 0x5c, 0x43, 0x6c, 0x61, 0x73, 0x73, 0x5f, 0x25, 0x30, 0x32,
            0x78, 0x26, 0x53, 0x75, 0x62, 0x43, 0x6c, 0x61, 0x73, 0x73, 0x5f, 0x25, 0x30,
            0x32, 0x78, 0x26, 0x50, 0x72, 0x6f, 0x74, 0x5f, 0x25, 0x30, 0x32, 0x78, 0x00,
        ]) as *const WCHAR,
        (*Descriptor).bFunctionClass as ::core::ffi::c_int,
        (*Descriptor).bFunctionSubClass as ::core::ffi::c_int,
        (*Descriptor).bFunctionProtocol as ::core::ffi::c_int,
    ) + 1) as ULONG;
    Index = Index.wrapping_add((swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        (&[
            0x55u16, 0x53, 0x42, 0x5c, 0x43, 0x6c, 0x61, 0x73, 0x73, 0x5f, 0x25, 0x30, 0x32,
            0x78, 0x26, 0x53, 0x75, 0x62, 0x43, 0x6c, 0x61, 0x73, 0x73, 0x5f, 0x25, 0x30,
            0x32, 0x78, 0x00,
        ]) as *const WCHAR,
        (*Descriptor).bFunctionClass as ::core::ffi::c_int,
        (*Descriptor).bFunctionSubClass as ::core::ffi::c_int,
    ) + 1) as ULONG);
    Index = Index.wrapping_add((swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        (&[
            0x55u16, 0x53, 0x42, 0x5c, 0x43, 0x6c, 0x61, 0x73, 0x73, 0x5f, 0x25, 0x30, 0x32,
            0x78, 0x00,
        ]) as *const WCHAR,
        (*Descriptor).bFunctionClass as ::core::ffi::c_int,
    ) + 1) as ULONG);

    // allocate result buffer
    DescriptionBuffer = AllocateItem(
        NonPagedPool,
        ((Index + 1) as usize * core::mem::size_of::<WCHAR>()) as ULONG,
    ) as LPWSTR;
    if DescriptionBuffer.is_null() {
        //
        // failed to allocate memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // copy description
    RtlCopyMemory(
        DescriptionBuffer as *mut ::core::ffi::c_void,
        Buffer.as_ptr() as *const ::core::ffi::c_void,
        (Index + 1) as usize * core::mem::size_of::<WCHAR>(),
    );
    (*FunctionDescriptor).CompatibleId.Buffer = DescriptionBuffer as PWCH;
    (*FunctionDescriptor).CompatibleId.Length =
        (Index as usize * core::mem::size_of::<WCHAR>()) as USHORT;
    (*FunctionDescriptor).CompatibleId.MaximumLength =
        ((Index + 1) as usize * core::mem::size_of::<WCHAR>()) as USHORT;

    //
    // done
    //
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_EnumWithAssociationDescriptor(
    DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {

    let mut DescriptorCount: ULONG;
    let mut Index: ULONG;
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    //
    // get device extension
    //
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    if (*FDODeviceExtension).Common.IsFDO == 0 {
        RtlAssert(
            b"FDODeviceExtension->Common.IsFDO\0" as *const u8 as *const i8 as PVOID,
            b"function.c\0" as *const u8 as *const i8 as PVOID,
            453 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    //
    // count association descriptors
    //
    DescriptorCount =
        USBCCGP_CountAssociationDescriptors((*FDODeviceExtension).ConfigurationDescriptor);
    if DescriptorCount == 0 {
        //
        // no descriptors found
        //
        return STATUS_NOT_SUPPORTED;
    }

    //
    // allocate function descriptor array
    //
    // Room for the IADs' functions and for every interface no IAD covers (see below).
    let NumInterfaces = (*(*FDODeviceExtension).ConfigurationDescriptor).bNumInterfaces as ULONG;
    (*FDODeviceExtension).FunctionDescriptor = AllocateItem(
        NonPagedPool,
        (core::mem::size_of::<USBC_FUNCTION_DESCRIPTOR>() as usize)
            .wrapping_mul((DescriptorCount + NumInterfaces) as usize) as ULONG,
    ) as PUSBC_FUNCTION_DESCRIPTOR;
    if (*FDODeviceExtension).FunctionDescriptor.is_null() {
        //
        // no memory
        //
        if DbgPrint(
            b"(%s:%d) USBCCGP_EnumWithAssociationDescriptor failed to allocate function descriptor count %x\n\0"
                as *const u8 as PCSTR,
            b"function.c\0" as *const u8 as *const i8,
            476 as i32,
            DescriptorCount,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"function.c\0" as *const u8 as *const i8,
                476 as i32,
            );
        }
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Index = 0 as ULONG;
    while Index < DescriptorCount {
        //
        // init function descriptors
        //
        Status = USBCCGP_InitFunctionDescriptor(
            FDODeviceExtension,
            Index,
            (*FDODeviceExtension).FunctionDescriptor.offset(Index as isize)
                as PUSBC_FUNCTION_DESCRIPTOR,
        );
        if !(Status >= 0 as i32) {
            //
            // failed
            //
            return Status;
        }
        Index = Index.wrapping_add(1);
    }

    //
    // An interface no IAD covers is a function of its own. The C enumerated IADs only, so on
    // a device with both it dropped every such interface: the RTL8821CU's Bluetooth is an IAD
    // (interfaces 0-1) and its Wi-Fi, interface 2, never got a device. Windows' usbccgp gives
    // the ungrouped interface its own MI_xx function, as the legacy path below would.
    //
    let mut Count = DescriptorCount;
    let mut Interface: ULONG = 0;
    while Interface < NumInterfaces {
        let mut Covered = false;
        let mut I: ULONG = 0;
        while I < DescriptorCount {
            let Iad = USBCCGP_GetAssociationDescriptorAtIndex((*FDODeviceExtension).ConfigurationDescriptor, I);
            if !Iad.is_null()
                && Interface >= (*Iad).bFirstInterface as ULONG
                && Interface < (*Iad).bFirstInterface as ULONG + (*Iad).bInterfaceCount as ULONG
            {
                Covered = true;
                break;
            }
            I += 1;
        }
        if !Covered {
            let InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(
                (*FDODeviceExtension).ConfigurationDescriptor as PVOID,
                (*FDODeviceExtension).ConfigurationDescriptor as PVOID,
                Interface as LONG,
                0,
                -1,
                -1,
                -1,
            );
            if !InterfaceDescriptor.is_null() {
                let Function = (*FDODeviceExtension).FunctionDescriptor.offset(Count as isize);
                (*Function).FunctionNumber = Interface as UCHAR;
                (*Function).NumberOfInterfaces = 1;
                (*Function).InterfaceDescriptorList =
                    AllocateItem(NonPagedPool, core::mem::size_of::<PUSB_INTERFACE_DESCRIPTOR>() as ULONG)
                        as *mut PUSB_INTERFACE_DESCRIPTOR;
                if (*Function).InterfaceDescriptorList.is_null() {
                    return STATUS_INSUFFICIENT_RESOURCES;
                }
                *(*Function).InterfaceDescriptorList = InterfaceDescriptor;
                Status = USBCCG_InitIdsWithInterfaceDescriptor(FDODeviceExtension, InterfaceDescriptor, Interface, Function);
                if !NT_SUCCESS(Status) {
                    return Status;
                }
                DbgPrint(
                    b"[USBCCGP] interface %lu is in no IAD: function MI_%02lx of its own\n\0".as_ptr() as PCSTR,
                    Interface,
                    Interface,
                );
                Count += 1;
            }
        }
        Interface += 1;
    }

    //
    // store function descriptor count
    //
    (*FDODeviceExtension).FunctionDescriptorCount = Count;

    //
    // done
    //
    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn USBCCG_InitIdsWithInterfaceDescriptor(
    FDODeviceExtension: PFDO_DEVICE_EXTENSION,
    Descriptor: PUSB_INTERFACE_DESCRIPTOR,
    FunctionIndex: ULONG,
    FunctionDescriptor: PUSBC_FUNCTION_DESCRIPTOR,
) -> NTSTATUS {
    let mut Index: ULONG;
    let mut Buffer: [u16; 200] = [0; 200];
    let mut DescriptionBuffer: LPWSTR = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if (*Descriptor).iInterface != 0 {
        Status = USBCCGP_GetStringDescriptor(
            (*FDODeviceExtension).NextDeviceObject,
            (100usize).wrapping_mul(core::mem::size_of::<u16>()) as ULONG,
            (*Descriptor).iInterface,
            0x0409, //FIXME
            core::ptr::addr_of_mut!(DescriptionBuffer) as *mut PVOID,
        );
        if !NT_SUCCESS(Status) {
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!((*FunctionDescriptor).FunctionDescription),
                wstr!(b"\0"),
            );
        } else {
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!((*FunctionDescriptor).FunctionDescription),
                DescriptionBuffer,
            );
        }
        // DPRINT1("FunctionDescription %wZ\n", &FunctionDescriptor->FunctionDescription);
    }

    Index = (swprintf(
        Buffer.as_mut_ptr(),
        wstr!(b"USB\\VID_%04x&PID_%04x&Rev_%04x&MI_%02x\0"),
        (*(*FDODeviceExtension).DeviceDescriptor).idVendor as core::ffi::c_int,
        (*(*FDODeviceExtension).DeviceDescriptor).idProduct as core::ffi::c_int,
        (*(*FDODeviceExtension).DeviceDescriptor).bcdDevice as core::ffi::c_int,
        FunctionIndex,
    ) + 1) as ULONG;
    Index = (Index as core::ffi::c_ulong).wrapping_add(
        (swprintf(
            Buffer.as_mut_ptr().offset(Index as isize),
            wstr!(b"USB\\VID_%04x&PID_%04x&MI_%02x\0"),
            (*(*FDODeviceExtension).DeviceDescriptor).idVendor as core::ffi::c_int,
            (*(*FDODeviceExtension).DeviceDescriptor).idProduct as core::ffi::c_int,
            FunctionIndex,
        ) + 1) as core::ffi::c_ulong,
    ) as ULONG;

    DescriptionBuffer = AllocateItem(
        NonPagedPool,
        ((Index as core::ffi::c_ulong).wrapping_add(1) as usize)
            .wrapping_mul(core::mem::size_of::<u16>()) as ULONG,
    ) as LPWSTR;
    if DescriptionBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyMemory(
        DescriptionBuffer as *mut core::ffi::c_void,
        Buffer.as_ptr() as *const core::ffi::c_void,
        ((Index as core::ffi::c_ulong).wrapping_add(1) as usize)
            .wrapping_mul(core::mem::size_of::<u16>()),
    );
    (*FunctionDescriptor).HardwareId.Buffer = DescriptionBuffer;
    (*FunctionDescriptor).HardwareId.Length =
        (Index as usize).wrapping_mul(core::mem::size_of::<u16>()) as u16;
    (*FunctionDescriptor).HardwareId.MaximumLength =
        ((Index as core::ffi::c_ulong).wrapping_add(1) as usize)
            .wrapping_mul(core::mem::size_of::<u16>()) as u16;

    Index = (swprintf(
        Buffer.as_mut_ptr(),
        wstr!(b"USB\\Class_%02x&SubClass_%02x&Prot_%02x\0"),
        (*Descriptor).bInterfaceClass as core::ffi::c_int,
        (*Descriptor).bInterfaceSubClass as core::ffi::c_int,
        (*Descriptor).bInterfaceProtocol as core::ffi::c_int,
    ) + 1) as ULONG;
    Index = (Index as core::ffi::c_ulong).wrapping_add(
        (swprintf(
            Buffer.as_mut_ptr().offset(Index as isize),
            wstr!(b"USB\\Class_%02x&SubClass_%02x\0"),
            (*Descriptor).bInterfaceClass as core::ffi::c_int,
            (*Descriptor).bInterfaceSubClass as core::ffi::c_int,
        ) + 1) as core::ffi::c_ulong,
    ) as ULONG;
    Index = (Index as core::ffi::c_ulong).wrapping_add(
        (swprintf(
            Buffer.as_mut_ptr().offset(Index as isize),
            wstr!(b"USB\\Class_%02x\0"),
            (*Descriptor).bInterfaceClass as core::ffi::c_int,
        ) + 1) as core::ffi::c_ulong,
    ) as ULONG;

    DescriptionBuffer = AllocateItem(
        NonPagedPool,
        ((Index as core::ffi::c_ulong).wrapping_add(1) as usize)
            .wrapping_mul(core::mem::size_of::<u16>()) as ULONG,
    ) as LPWSTR;
    if DescriptionBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyMemory(
        DescriptionBuffer as *mut core::ffi::c_void,
        Buffer.as_ptr() as *const core::ffi::c_void,
        ((Index as core::ffi::c_ulong).wrapping_add(1) as usize)
            .wrapping_mul(core::mem::size_of::<u16>()),
    );
    (*FunctionDescriptor).CompatibleId.Buffer = DescriptionBuffer;
    (*FunctionDescriptor).CompatibleId.Length =
        (Index as usize).wrapping_mul(core::mem::size_of::<u16>()) as u16;
    (*FunctionDescriptor).CompatibleId.MaximumLength =
        ((Index as core::ffi::c_ulong).wrapping_add(1) as usize)
            .wrapping_mul(core::mem::size_of::<u16>()) as u16;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_LegacyEnum(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    const USB_DEVICE_CLASS_AUDIO: i32 = 0x01;

    let mut Index: ULONG;
    let mut SubIndex: ULONG;
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR;
    let mut NextInterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR;

    //
    // get device extension
    //
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    // ASSERT(FDODeviceExtension->Common.IsFDO);

    //
    // sanity check
    //
    // ASSERT(FDODeviceExtension->ConfigurationDescriptor->bNumInterfaces);

    //
    // allocate function array
    //
    (*FDODeviceExtension).FunctionDescriptor = AllocateItem(
        NonPagedPool,
        (core::mem::size_of::<USBC_FUNCTION_DESCRIPTOR>() as usize).wrapping_mul(
            (*(*FDODeviceExtension).ConfigurationDescriptor).bNumInterfaces as usize,
        ) as ULONG,
    ) as PUSBC_FUNCTION_DESCRIPTOR;
    if (*FDODeviceExtension).FunctionDescriptor.is_null() {
        //
        // no memory
        //
        // DPRINT1("USBCCGP_EnumWithAssociationDescriptor failed to allocate function descriptor %lu\n", ...);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // init function descriptors
    //
    (*FDODeviceExtension).FunctionDescriptorCount =
        (*(*FDODeviceExtension).ConfigurationDescriptor).bNumInterfaces as ULONG;
    Index = 0 as ULONG;
    while Index < (*(*FDODeviceExtension).ConfigurationDescriptor).bNumInterfaces as ULONG {
        // get interface descriptor
        InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(
            (*FDODeviceExtension).ConfigurationDescriptor as PVOID,
            (*FDODeviceExtension).ConfigurationDescriptor as PVOID,
            Index as LONG,
            0 as LONG,
            -(1 as i32) as LONG,
            -(1 as i32) as LONG,
            -(1 as i32) as LONG,
        );
        if InterfaceDescriptor.is_null() {
            //
            // failed to find interface descriptor
            //
            // DPRINT1("[USBCCGP] Failed to find interface descriptor index %lu\n", Index);
            // ASSERT(FALSE);
            return STATUS_UNSUCCESSFUL;
        }

        SubIndex = 0 as ULONG;
        if (*InterfaceDescriptor).bInterfaceClass as i32 == USB_DEVICE_CLASS_AUDIO {
            // AUDIO CLASS lets group all audio interfaces together
            //
            // init function descriptor
            //
            (*(*FDODeviceExtension).FunctionDescriptor.offset(Index as isize)).FunctionNumber =
                Index as UCHAR;
            (*(*FDODeviceExtension).FunctionDescriptor.offset(Index as isize)).NumberOfInterfaces =
                1 as UCHAR;
            (*(*FDODeviceExtension).FunctionDescriptor.offset(Index as isize))
                .InterfaceDescriptorList = AllocateItem(
                NonPagedPool,
                (core::mem::size_of::<PUSB_INTERFACE_DESCRIPTOR>() as usize).wrapping_mul(
                    (*(*FDODeviceExtension).ConfigurationDescriptor).bNumInterfaces as usize,
                ) as ULONG,
            ) as *mut PUSB_INTERFACE_DESCRIPTOR;
            if (*(*FDODeviceExtension).FunctionDescriptor.offset(Index as isize))
                .InterfaceDescriptorList
                .is_null()
            {
                //
                // no memory
                //
                FreeItem((*FDODeviceExtension).FunctionDescriptor as PVOID);
                (*FDODeviceExtension).FunctionDescriptor = core::ptr::null_mut();
                (*FDODeviceExtension).FunctionDescriptorCount = 0 as ULONG;
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            //
            // store interface descriptor
            //
            *(*(*FDODeviceExtension).FunctionDescriptor.offset(Index as isize))
                .InterfaceDescriptorList
                .offset(SubIndex as isize) = InterfaceDescriptor;
            loop {
                NextInterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(
                    (*FDODeviceExtension).ConfigurationDescriptor as PVOID,
                    (*FDODeviceExtension).ConfigurationDescriptor as PVOID,
                    Index.wrapping_add(SubIndex).wrapping_add(1) as LONG,
                    0 as LONG,
                    -(1 as i32) as LONG,
                    -(1 as i32) as LONG,
                    -(1 as i32) as LONG,
                );
                if NextInterfaceDescriptor.is_null()
                    || (*NextInterfaceDescriptor).bInterfaceClass as i32 != USB_DEVICE_CLASS_AUDIO
                {
                    // end of collection
                    break;
                }
                SubIndex = SubIndex.wrapping_add(1);
                // ASSERT(SubIndex < FDODeviceExtension->ConfigurationDescriptor->bNumInterfaces);
                let ref mut fresh2 = (*(*FDODeviceExtension)
                    .FunctionDescriptor
                    .offset(Index as isize))
                .NumberOfInterfaces;
                *fresh2 = (*fresh2).wrapping_add(1);
                *(*(*FDODeviceExtension).FunctionDescriptor.offset(Index as isize))
                    .InterfaceDescriptorList
                    .offset(SubIndex as isize) = NextInterfaceDescriptor;
            }
        } else {
            //
            // init function descriptor
            //
            (*(*FDODeviceExtension).FunctionDescriptor.offset(Index as isize)).FunctionNumber =
                Index as UCHAR;
            (*(*FDODeviceExtension).FunctionDescriptor.offset(Index as isize)).NumberOfInterfaces =
                1 as UCHAR;
            (*(*FDODeviceExtension).FunctionDescriptor.offset(Index as isize))
                .InterfaceDescriptorList = AllocateItem(
                NonPagedPool,
                core::mem::size_of::<PUSB_INTERFACE_DESCRIPTOR>() as ULONG,
            ) as *mut PUSB_INTERFACE_DESCRIPTOR;
            if (*(*FDODeviceExtension).FunctionDescriptor.offset(Index as isize))
                .InterfaceDescriptorList
                .is_null()
            {
                //
                // no memory
                //
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            //
            // store interface descriptor
            //
            *(*(*FDODeviceExtension).FunctionDescriptor.offset(Index as isize))
                .InterfaceDescriptorList
                .offset(0 as isize) = InterfaceDescriptor;
        }

        //
        // now init the device ids
        //
        Status = USBCCG_InitIdsWithInterfaceDescriptor(
            FDODeviceExtension,
            InterfaceDescriptor,
            Index,
            (*FDODeviceExtension).FunctionDescriptor.offset(Index as isize)
                as PUSBC_FUNCTION_DESCRIPTOR,
        );
        if !NT_SUCCESS(Status) {
            //
            // failed to init ids
            //
            // DPRINT1("[USBCCGP] Failed to init ids with %lx\n", Status);
            return Status;
        }
        //
        // increment interface count
        //
        Index = Index.wrapping_add(SubIndex);
        Index = Index.wrapping_add(1);
    }

    //
    // done
    //
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_EnumWithUnionFunctionDescriptors(
    mut DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"USBCCGP_EnumWithUnionFunctionDescriptors\0".as_ptr() as *const i8,
        b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/function.c\0"
            .as_ptr() as *const i8,
        746 as i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_EnumWithAudioLegacy(
    DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {

    const USB_DEVICE_CLASS_AUDIO: i32 = 0x01;

    let mut Index: ULONG;
    let mut InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR;
    let mut FirstDescriptor: PUSB_INTERFACE_DESCRIPTOR = core::ptr::null_mut();
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    //
    // get device extension
    //
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    // ASSERT(FDODeviceExtension->Common.IsFDO);

    //
    // first check if all interfaces belong to the same audio class
    //
    Index = 0;
    while Index
        < (*(*FDODeviceExtension).ConfigurationDescriptor).bNumInterfaces as ULONG
    {
        //
        // get interface descriptor
        //
        InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(
            (*FDODeviceExtension).ConfigurationDescriptor as PVOID,
            (*FDODeviceExtension).ConfigurationDescriptor as PVOID,
            Index as LONG,
            0 as LONG,
            -1 as LONG,
            -1 as LONG,
            -1 as LONG,
        );
        // DPRINT1("Index %lu Descriptor %p\n", Index, InterfaceDescriptor);
        // ASSERT(InterfaceDescriptor);

        if (*InterfaceDescriptor).bInterfaceClass as i32 != USB_DEVICE_CLASS_AUDIO {
            //
            // collection contains non audio class
            //
            return STATUS_UNSUCCESSFUL;
        }

        if FirstDescriptor.is_null() {
            //
            // store interface descriptor
            //
            FirstDescriptor = InterfaceDescriptor;
            Index = Index.wrapping_add(1);
            continue;
        }

        if (*FirstDescriptor).bInterfaceSubClass as i32
            == (*InterfaceDescriptor).bInterfaceSubClass as i32
        {
            //
            // interface subclass must be different from the first interface
            //
            return STATUS_UNSUCCESSFUL;
        }

        Index = Index.wrapping_add(1);
    }

    //
    // this is an composite audio device
    //
    // DPRINT("[USBCCGP] Audio Composite Device detected\n");

    //
    // audio interfaces are all grouped into one single function
    //
    (*FDODeviceExtension).FunctionDescriptor = AllocateItem(
        NonPagedPool,
        core::mem::size_of::<USBC_FUNCTION_DESCRIPTOR>() as ULONG,
    ) as PUSBC_FUNCTION_DESCRIPTOR;
    if (*FDODeviceExtension).FunctionDescriptor.is_null() {
        //
        // no memory
        //
        // DPRINT1("USBCCGP_EnumWithAssociationDescriptor failed to allocate function descriptor count\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // init function number
    //
    (*(*FDODeviceExtension).FunctionDescriptor.offset(0)).FunctionNumber = 0 as UCHAR;

    //
    // store interfaces
    //
    Status = AllocateInterfaceDescriptorsArray(
        (*FDODeviceExtension).ConfigurationDescriptor,
        core::ptr::addr_of_mut!(
            (*(*FDODeviceExtension).FunctionDescriptor.offset(0)).InterfaceDescriptorList
        ),
    );
    if !NT_SUCCESS(Status) {
        //
        // failed to allocate descriptor array
        //
        // DPRINT1("[USBCCGP] Failed to allocate descriptor array %lx\n", Status);
        return Status;
    }

    //
    // now init the device ids
    //
    Status = USBCCG_InitIdsWithInterfaceDescriptor(
        FDODeviceExtension,
        FirstDescriptor,
        0 as ULONG,
        (*FDODeviceExtension).FunctionDescriptor.offset(0) as PUSBC_FUNCTION_DESCRIPTOR,
    );
    if !NT_SUCCESS(Status) {
        //
        // failed to init ids
        //
        // DPRINT1("[USBCCGP] Failed to init ids with %lx\n", Status);
        return Status;
    }

    //
    // number of interfaces
    //
    (*(*FDODeviceExtension).FunctionDescriptor.offset(0)).NumberOfInterfaces =
        (*(*FDODeviceExtension).ConfigurationDescriptor).bNumInterfaces;

    //
    // store function count
    //
    (*FDODeviceExtension).FunctionDescriptorCount = 1 as ULONG;

    //
    // done
    //
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_EnumerateFunctions(
    DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    //
    // get device extension
    //
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    if (*FDODeviceExtension).Common.IsFDO == 0 {
        RtlAssert(
            b"FDODeviceExtension->Common.IsFDO\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/function.c\0".as_ptr() as *const i8 as PVOID,
            880 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    //
    // first try with filter driver
    //
    Status = USBCCGP_CustomEnumWithInterface(DeviceObject);
    if NT_SUCCESS(Status) {
        //
        // succeeded
        //
        return Status;
    }

    //
    // enumerate functions with interface association descriptor
    //
    Status = USBCCGP_EnumWithAssociationDescriptor(DeviceObject);
    if NT_SUCCESS(Status) {
        //
        // succeeded
        //
        return Status;
    }

    //
    // try with legacy audio methods
    //
    Status = USBCCGP_EnumWithAudioLegacy(DeviceObject);
    if NT_SUCCESS(Status) {
        //
        // succeeded
        //
        return Status;
    }

    //
    // try with legacy enumeration
    //
    return USBCCGP_LegacyEnum(DeviceObject);
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
// USBCCGP_QueryInterface(
//     IN PDEVICE_OBJECT DeviceObject,
//     OUT PUSBC_DEVICE_CONFIGURATION_INTERFACE_V1 BusInterface)
// {
//     KEVENT Event;
//     NTSTATUS Status;
//     PIRP Irp;
//     IO_STATUS_BLOCK IoStatus;
//     PIO_STACK_LOCATION Stack;
//
//     /* Sanity checks */
//     ASSERT(DeviceObject);
//
//     /* Initialize event */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* Init interface */
//     RtlZeroMemory(BusInterface, sizeof(USBC_DEVICE_CONFIGURATION_INTERFACE_V1));
//     BusInterface->Version = USBC_DEVICE_CONFIGURATION_INTERFACE_VERSION_1;
//     BusInterface->Size = sizeof(USBC_DEVICE_CONFIGURATION_INTERFACE_V1);
//
//     /* Create irp */
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_PNP,
//                                        DeviceObject,
//                                        NULL,
//                                        0,
//                                        NULL,
//                                        &Event,
//                                        &IoStatus);
//
//     //
//     // was irp built
//     //
//     if (Irp == NULL)
//     {
//         //
//         // no memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // initialize request
//     //
//     Stack = IoGetNextIrpStackLocation(Irp);
//     Stack->MajorFunction = IRP_MJ_PNP;
//     Stack->MinorFunction = IRP_MN_QUERY_INTERFACE;
//     Stack->Parameters.QueryInterface.Size = sizeof(BUS_INTERFACE_STANDARD);
//     Stack->Parameters.QueryInterface.InterfaceType = (LPGUID)&USB_BUS_INTERFACE_USBC_CONFIGURATION_GUID;
//     Stack->Parameters.QueryInterface.Version = 2;
//     Stack->Parameters.QueryInterface.Interface = (PINTERFACE)&BusInterface;
//     Stack->Parameters.QueryInterface.InterfaceSpecificData = NULL;
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//
//     //
//     // call driver
//     //
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     //
//     // did operation complete
//     //
//     if (Status == STATUS_PENDING)
//     {
//         //
//         // wait for completion
//         //
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//
//         //
//         // collect status
//         //
//         Status = IoStatus.Status;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// USBCCGP_CustomEnumWithInterface(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     ULONG FunctionDescriptorBufferLength = 0;
//     NTSTATUS Status;
//     PUSBC_FUNCTION_DESCRIPTOR  FunctionDescriptorBuffer = NULL;
//
//     //
//     // get device extension
//     //
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     if (FDODeviceExtension->BusInterface.StartDeviceCallback == NULL)
//     {
//         //
//         // not supported
//         //
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     //
//     // invoke callback
//     //
//     Status = FDODeviceExtension->BusInterface.StartDeviceCallback(FDODeviceExtension->DeviceDescriptor,
//                                                                   FDODeviceExtension->ConfigurationDescriptor,
//                                                                   &FunctionDescriptorBuffer,
//                                                                   &FunctionDescriptorBufferLength,
//                                                                   DeviceObject,
//                                                                   FDODeviceExtension->PhysicalDeviceObject);
//
//     DPRINT("USBCCGP_CustomEnumWithInterface Status %lx\n", Status);
//     if (!NT_SUCCESS(Status))
//     {
//         //
//         // failed
//         //
//         return Status;
//     }
//
//     DPRINT("FunctionDescriptorBufferLength %lu\n", FunctionDescriptorBufferLength);
//     DPRINT("FunctionDescriptorBuffer %p\n", FunctionDescriptorBuffer);
//
//     //
//     // assume length % function buffer size
//     //
//     ASSERT(FunctionDescriptorBufferLength);
//     ASSERT(FunctionDescriptorBufferLength % sizeof(USBC_FUNCTION_DESCRIPTOR) == 0);
//
//     //
//     // store result
//     //
//     FDODeviceExtension->FunctionDescriptor = FunctionDescriptorBuffer;
//     FDODeviceExtension->FunctionDescriptorCount = FunctionDescriptorBufferLength / sizeof(USBC_FUNCTION_DESCRIPTOR);
//
//     //
//     // success
//     //
//     return STATUS_SUCCESS;
// }
//
// ULONG
// USBCCGP_CountAssociationDescriptors(
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor)
// {
//     PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR Descriptor;
//     PUCHAR Offset, End;
//     ULONG Count = 0;
//
//     //
//     // init offsets
//     //
//     Offset = (PUCHAR)ConfigurationDescriptor + ConfigurationDescriptor->bLength;
//     End = (PUCHAR)ConfigurationDescriptor + ConfigurationDescriptor->wTotalLength;
//
//     while (Offset < End)
//     {
//         //
//         // get association descriptor
//         //
//         Descriptor = (PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR)Offset;
//
//         if (Descriptor->bLength == sizeof(USB_INTERFACE_ASSOCIATION_DESCRIPTOR) && Descriptor->bDescriptorType == USB_INTERFACE_ASSOCIATION_DESCRIPTOR_TYPE)
//         {
//             //
//             // found descriptor
//             //
//             Count++;
//         }
//
//         //
//         // move to next descriptor
//         //
//         Offset += Descriptor->bLength;
//     }
//
//     //
//     // done
//     //
//     return Count;
// }
//
// PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR
// USBCCGP_GetAssociationDescriptorAtIndex(
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     IN ULONG Index)
// {
//     PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR Descriptor;
//     PUCHAR Offset, End;
//     ULONG Count = 0;
//
//     //
//     // init offsets
//     //
//     Offset = (PUCHAR)ConfigurationDescriptor + ConfigurationDescriptor->bLength;
//     End = (PUCHAR)ConfigurationDescriptor + ConfigurationDescriptor->wTotalLength;
//
//     while (Offset < End)
//     {
//         //
//         // get association descriptor
//         //
//         Descriptor = (PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR)Offset;
//
//         if (Descriptor->bLength == sizeof(USB_INTERFACE_ASSOCIATION_DESCRIPTOR) && Descriptor->bDescriptorType == USB_INTERFACE_ASSOCIATION_DESCRIPTOR_TYPE)
//         {
//             if (Index == Count)
//             {
//                 //
//                 // found descriptor
//                 //
//                 return Descriptor;
//             }
//
//             //
//             // not the searched one
//             //
//             Count++;
//         }
//
//         //
//         // move to next descriptor
//         //
//         Offset += Descriptor->bLength;
//     }
//
//     //
//     // failed to find descriptor at the specified index
//     //
//     return NULL;
// }
//
// NTSTATUS
// USBCCGP_InitInterfaceListOfFunctionDescriptor(
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     IN PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR AssociationDescriptor,
//     OUT PUSBC_FUNCTION_DESCRIPTOR FunctionDescriptor)
// {
//     PUSB_INTERFACE_DESCRIPTOR Descriptor;
//     PUCHAR Offset, End;
//     ULONG Count = 0;
//
//     //
//     // init offsets
//     //
//     Offset = (PUCHAR)AssociationDescriptor + AssociationDescriptor->bLength;
//     End = (PUCHAR)ConfigurationDescriptor + ConfigurationDescriptor->wTotalLength;
//
//     while (Offset < End)
//     {
//         //
//         // get association descriptor
//         //
//         Descriptor = (PUSB_INTERFACE_DESCRIPTOR)Offset;
//
//         if (Descriptor->bLength == sizeof(USB_INTERFACE_DESCRIPTOR) && Descriptor->bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE)
//         {
//             //
//             // store interface descriptor
//             //
//             FunctionDescriptor->InterfaceDescriptorList[Count] = Descriptor;
//             Count++;
//
//             if (Count == AssociationDescriptor->bInterfaceCount)
//             {
//                 //
//                 // got all interfaces
//                 //
//                 return STATUS_SUCCESS;
//             }
//         }
//
//         if (Descriptor->bLength == sizeof(USB_INTERFACE_ASSOCIATION_DESCRIPTOR) && Descriptor->bDescriptorType == USB_INTERFACE_ASSOCIATION_DESCRIPTOR_TYPE)
//         {
//             //
//             // WTF? a association descriptor which overlaps the next association descriptor
//             //
//             DPRINT1("Invalid association descriptor\n");
//             ASSERT(FALSE);
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         //
//         // move to next descriptor
//         //
//         Offset += Descriptor->bLength;
//     }
//
//     //
//     // invalid association descriptor
//     //
//     DPRINT1("Invalid association descriptor\n");
//     return STATUS_UNSUCCESSFUL;
// }
//
// NTSTATUS
// USBCCGP_InitFunctionDescriptor(
//     IN PFDO_DEVICE_EXTENSION FDODeviceExtension,
//     IN ULONG FunctionNumber,
//     OUT PUSBC_FUNCTION_DESCRIPTOR FunctionDescriptor)
// {
//     PUSB_INTERFACE_ASSOCIATION_DESCRIPTOR Descriptor;
//     NTSTATUS Status;
//     LPWSTR DescriptionBuffer;
//     WCHAR Buffer[100];
//     ULONG Index;
//
//     // init function number
//     FunctionDescriptor->FunctionNumber = (UCHAR)FunctionNumber;
//
//     // get association descriptor
//     Descriptor = USBCCGP_GetAssociationDescriptorAtIndex(FDODeviceExtension->ConfigurationDescriptor, FunctionNumber);
//     ASSERT(Descriptor);
//
//     // store number interfaces
//     FunctionDescriptor->NumberOfInterfaces = Descriptor->bInterfaceCount;
//
//     // allocate array for interface count
//     FunctionDescriptor->InterfaceDescriptorList = AllocateItem(NonPagedPool, sizeof(PUSB_INTERFACE_DESCRIPTOR) * Descriptor->bInterfaceCount);
//     if (!FunctionDescriptor->InterfaceDescriptorList)
//     {
//         //
//         // no memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // init interface list
//     Status = USBCCGP_InitInterfaceListOfFunctionDescriptor(FDODeviceExtension->ConfigurationDescriptor, Descriptor, FunctionDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         //
//         // failed
//         //
//         return Status;
//     }
//
//     //
//     // now init interface description
//     //
//     if (Descriptor->iFunction)
//     {
//         //
//         // get interface description
//         //
//          Status = USBCCGP_GetStringDescriptor(FDODeviceExtension->NextDeviceObject,
//                                               100 * sizeof(WCHAR),
//                                               Descriptor->iFunction,
//                                               0x0409, //FIXME
//                                               (PVOID*)&DescriptionBuffer);
//         if (!NT_SUCCESS(Status))
//         {
//             //
//             // no description
//             //
//             RtlInitUnicodeString(&FunctionDescriptor->FunctionDescription, L"");
//         }
//         else
//         {
//             //
//             // init description
//             //
//             RtlInitUnicodeString(&FunctionDescriptor->FunctionDescription, DescriptionBuffer);
//         }
//         DPRINT1("FunctionDescription %wZ\n", &FunctionDescriptor->FunctionDescription);
//     }
//
//     //
//     // now init hardware id
//     //
//     Index = swprintf(Buffer, L"USB\\VID_%04x&PID_%04x&Rev_%04x&MI_%02x", FDODeviceExtension->DeviceDescriptor->idVendor,
//                                                                          FDODeviceExtension->DeviceDescriptor->idProduct,
//                                                                          FDODeviceExtension->DeviceDescriptor->bcdDevice,
//                                                                          Descriptor->bFirstInterface) + 1;
//     Index += swprintf(&Buffer[Index], L"USB\\VID_%04x&PID_%04x&MI_%02x", FDODeviceExtension->DeviceDescriptor->idVendor,
//                                                                          FDODeviceExtension->DeviceDescriptor->idProduct,
//                                                                          Descriptor->bFirstInterface) + 1;
//
//     // allocate result buffer
//     DescriptionBuffer = AllocateItem(NonPagedPool, (Index + 1) * sizeof(WCHAR));
//     if (!DescriptionBuffer)
//     {
//         //
//         // failed to allocate memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // copy description
//     RtlCopyMemory(DescriptionBuffer, Buffer, (Index + 1) * sizeof(WCHAR));
//     FunctionDescriptor->HardwareId.Buffer = DescriptionBuffer;
//     FunctionDescriptor->HardwareId.Length = Index * sizeof(WCHAR);
//     FunctionDescriptor->HardwareId.MaximumLength = (Index + 1) * sizeof(WCHAR);
//
//     //
//     // now init the compatible id
//     //
//     Index = swprintf(Buffer, L"USB\\Class_%02x&SubClass_%02x&Prot_%02x", Descriptor->bFunctionClass, Descriptor->bFunctionSubClass, Descriptor->bFunctionProtocol) + 1;
//     Index += swprintf(&Buffer[Index], L"USB\\Class_%02x&SubClass_%02x",  Descriptor->bFunctionClass, Descriptor->bFunctionSubClass) + 1;
//     Index += swprintf(&Buffer[Index], L"USB\\Class_%02x", Descriptor->bFunctionClass) + 1;
//
//     // allocate result buffer
//     DescriptionBuffer = AllocateItem(NonPagedPool, (Index + 1) * sizeof(WCHAR));
//     if (!DescriptionBuffer)
//     {
//         //
//         // failed to allocate memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // copy description
//     RtlCopyMemory(DescriptionBuffer, Buffer, (Index + 1) * sizeof(WCHAR));
//     FunctionDescriptor->CompatibleId.Buffer = DescriptionBuffer;
//     FunctionDescriptor->CompatibleId.Length = Index * sizeof(WCHAR);
//     FunctionDescriptor->CompatibleId.MaximumLength = (Index + 1) * sizeof(WCHAR);
//
//     //
//     // done
//     //
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USBCCGP_EnumWithAssociationDescriptor(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     ULONG DescriptorCount, Index;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     //
//     // get device extension
//     //
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     //
//     // count association descriptors
//     //
//     DescriptorCount = USBCCGP_CountAssociationDescriptors(FDODeviceExtension->ConfigurationDescriptor);
//     if (!DescriptorCount)
//     {
//         //
//         // no descriptors found
//         //
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     //
//     // allocate function descriptor array
//     //
//     FDODeviceExtension->FunctionDescriptor = AllocateItem(NonPagedPool, sizeof(USBC_FUNCTION_DESCRIPTOR) * DescriptorCount);
//     if (!FDODeviceExtension->FunctionDescriptor)
//     {
//         //
//         // no memory
//         //
//         DPRINT1("USBCCGP_EnumWithAssociationDescriptor failed to allocate function descriptor count %x\n", DescriptorCount);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     for (Index = 0; Index < DescriptorCount; Index++)
//     {
//         //
//         // init function descriptors
//         //
//         Status = USBCCGP_InitFunctionDescriptor(FDODeviceExtension, Index, &FDODeviceExtension->FunctionDescriptor[Index]);
//         if (!NT_SUCCESS(Status))
//         {
//             //
//             // failed
//             //
//             return Status;
//         }
//     }
//
//     //
//     // store function descriptor count
//     //
//     FDODeviceExtension->FunctionDescriptorCount = DescriptorCount;
//
//     //
//     // done
//     //
//     return Status;
// }
//
// NTSTATUS
// USBCCG_InitIdsWithInterfaceDescriptor(
//     IN PFDO_DEVICE_EXTENSION FDODeviceExtension,
//     IN PUSB_INTERFACE_DESCRIPTOR Descriptor,
//     IN ULONG FunctionIndex,
//     OUT PUSBC_FUNCTION_DESCRIPTOR FunctionDescriptor)
// {
//     ULONG Index;
//     WCHAR Buffer[200];
//     LPWSTR DescriptionBuffer;
//     NTSTATUS Status;
//
//     //
//     // now init interface description
//     //
//     if (Descriptor->iInterface)
//     {
//         //
//         // get interface description
//         //
//          Status = USBCCGP_GetStringDescriptor(FDODeviceExtension->NextDeviceObject,
//                                               100 * sizeof(WCHAR),
//                                               Descriptor->iInterface,
//                                               0x0409, //FIXME
//                                               (PVOID*)&DescriptionBuffer);
//         if (!NT_SUCCESS(Status))
//         {
//             //
//             // no description
//             //
//             RtlInitUnicodeString(&FunctionDescriptor->FunctionDescription, L"");
//         }
//         else
//         {
//             //
//             // init description
//             //
//             RtlInitUnicodeString(&FunctionDescriptor->FunctionDescription, DescriptionBuffer);
//         }
//         DPRINT1("FunctionDescription %wZ\n", &FunctionDescriptor->FunctionDescription);
//     }
//
//
//     //
//     // now init hardware id
//     //
//     Index = swprintf(Buffer, L"USB\\VID_%04x&PID_%04x&Rev_%04x&MI_%02x", FDODeviceExtension->DeviceDescriptor->idVendor,
//                                                                          FDODeviceExtension->DeviceDescriptor->idProduct,
//                                                                          FDODeviceExtension->DeviceDescriptor->bcdDevice,
//                                                                          FunctionIndex) + 1;
//     Index += swprintf(&Buffer[Index], L"USB\\VID_%04x&PID_%04x&MI_%02x", FDODeviceExtension->DeviceDescriptor->idVendor,
//                                                                          FDODeviceExtension->DeviceDescriptor->idProduct,
//                                                                          FunctionIndex) + 1;
//
//     // allocate result buffer
//     DescriptionBuffer = AllocateItem(NonPagedPool, (Index + 1) * sizeof(WCHAR));
//     if (!DescriptionBuffer)
//     {
//         //
//         // failed to allocate memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // copy description
//     RtlCopyMemory(DescriptionBuffer, Buffer, (Index + 1) * sizeof(WCHAR));
//     FunctionDescriptor->HardwareId.Buffer = DescriptionBuffer;
//     FunctionDescriptor->HardwareId.Length = Index * sizeof(WCHAR);
//     FunctionDescriptor->HardwareId.MaximumLength = (Index + 1) * sizeof(WCHAR);
//
//     //
//     // now init the compatible id
//     //
//     Index = swprintf(Buffer, L"USB\\Class_%02x&SubClass_%02x&Prot_%02x", Descriptor->bInterfaceClass, Descriptor->bInterfaceSubClass, Descriptor->bInterfaceProtocol) + 1;
//     Index += swprintf(&Buffer[Index], L"USB\\Class_%02x&SubClass_%02x",  Descriptor->bInterfaceClass, Descriptor->bInterfaceSubClass) + 1;
//     Index += swprintf(&Buffer[Index], L"USB\\Class_%02x", Descriptor->bInterfaceClass) + 1;
//
//     // allocate result buffer
//     DescriptionBuffer = AllocateItem(NonPagedPool, (Index + 1) * sizeof(WCHAR));
//     if (!DescriptionBuffer)
//     {
//         //
//         // failed to allocate memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // copy description
//     RtlCopyMemory(DescriptionBuffer, Buffer, (Index + 1) * sizeof(WCHAR));
//     FunctionDescriptor->CompatibleId.Buffer = DescriptionBuffer;
//     FunctionDescriptor->CompatibleId.Length = Index * sizeof(WCHAR);
//     FunctionDescriptor->CompatibleId.MaximumLength = (Index + 1) * sizeof(WCHAR);
//
//     //
//     // done
//     //
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// USBCCGP_LegacyEnum(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     ULONG Index, SubIndex;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor, NextInterfaceDescriptor;
//
//     //
//     // get device extension
//     //
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     //
//     // sanity check
//     //
//     ASSERT(FDODeviceExtension->ConfigurationDescriptor->bNumInterfaces);
//
//     //
//     // allocate function array
//     //
//     FDODeviceExtension->FunctionDescriptor = AllocateItem(NonPagedPool, sizeof(USBC_FUNCTION_DESCRIPTOR) * FDODeviceExtension->ConfigurationDescriptor->bNumInterfaces);
//     if (!FDODeviceExtension->FunctionDescriptor)
//     {
//         //
//         // no memory
//         //
//         DPRINT1("USBCCGP_EnumWithAssociationDescriptor failed to allocate function descriptor %lu\n", FDODeviceExtension->ConfigurationDescriptor->bNumInterfaces);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // init function descriptors
//     //
//     FDODeviceExtension->FunctionDescriptorCount = FDODeviceExtension->ConfigurationDescriptor->bNumInterfaces;
//     for (Index = 0; Index < FDODeviceExtension->ConfigurationDescriptor->bNumInterfaces; Index++)
//     {
//         // get interface descriptor
//         InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(FDODeviceExtension->ConfigurationDescriptor, FDODeviceExtension->ConfigurationDescriptor, Index, 0, -1, -1, -1);
//         if (InterfaceDescriptor == NULL)
//         {
//             //
//             // failed to find interface descriptor
//             //
//             DPRINT1("[USBCCGP] Failed to find interface descriptor index %lu\n", Index);
//             ASSERT(FALSE);
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         SubIndex = 0;
//         if (InterfaceDescriptor->bInterfaceClass == USB_DEVICE_CLASS_AUDIO)
//         {
//             // AUDIO CLASS lets group all audio interfaces together
//             //
//             // init function descriptor
//             //
//             FDODeviceExtension->FunctionDescriptor[Index].FunctionNumber = Index;
//             FDODeviceExtension->FunctionDescriptor[Index].NumberOfInterfaces = 1;
//             FDODeviceExtension->FunctionDescriptor[Index].InterfaceDescriptorList = AllocateItem(NonPagedPool, sizeof(PUSB_INTERFACE_DESCRIPTOR) * FDODeviceExtension->ConfigurationDescriptor->bNumInterfaces);
//             if (!FDODeviceExtension->FunctionDescriptor[Index].InterfaceDescriptorList)
//             {
//                 //
//                 // no memory
//                 //
//                 FreeItem(FDODeviceExtension->FunctionDescriptor);
//                 FDODeviceExtension->FunctionDescriptor = NULL;
//                 FDODeviceExtension->FunctionDescriptorCount = 0;
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             //
//             // store interface descriptor
//             //
//             FDODeviceExtension->FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex] = InterfaceDescriptor;
//             while (TRUE)
//             {
//                 NextInterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(FDODeviceExtension->ConfigurationDescriptor, FDODeviceExtension->ConfigurationDescriptor, Index + SubIndex + 1, 0, -1, -1, -1);
//                 if (!NextInterfaceDescriptor || NextInterfaceDescriptor->bInterfaceClass != USB_DEVICE_CLASS_AUDIO)
//                 {
//                     // end of collection
//                     break;
//                 }
//                 SubIndex++;
//                 ASSERT(SubIndex < FDODeviceExtension->ConfigurationDescriptor->bNumInterfaces);
//                 FDODeviceExtension->FunctionDescriptor[Index].NumberOfInterfaces++;
//                 FDODeviceExtension->FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex] = NextInterfaceDescriptor;
//             }
//         }
//         else
//         {
//             //
//             // init function descriptor
//             //
//             FDODeviceExtension->FunctionDescriptor[Index].FunctionNumber = Index;
//             FDODeviceExtension->FunctionDescriptor[Index].NumberOfInterfaces = 1;
//             FDODeviceExtension->FunctionDescriptor[Index].InterfaceDescriptorList = AllocateItem(NonPagedPool, sizeof(PUSB_INTERFACE_DESCRIPTOR));
//             if (!FDODeviceExtension->FunctionDescriptor[Index].InterfaceDescriptorList)
//             {
//                 //
//                 // no memory
//                 //
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             //
//             // store interface descriptor
//             //
//             FDODeviceExtension->FunctionDescriptor[Index].InterfaceDescriptorList[0] = InterfaceDescriptor;
//         }
//
//         //
//         // now init the device ids
//         //
//         Status = USBCCG_InitIdsWithInterfaceDescriptor(FDODeviceExtension, InterfaceDescriptor, Index, &FDODeviceExtension->FunctionDescriptor[Index]);
//         if (!NT_SUCCESS(Status))
//         {
//             //
//             // failed to init ids
//             //
//             DPRINT1("[USBCCGP] Failed to init ids with %lx\n", Status);
//             return Status;
//         }
//         //
//         // increment interface count
//         //
//         Index += SubIndex;
//     }
//
//     //
//     // done
//     //
//     return Status;
// }
//
// NTSTATUS
// USBCCGP_EnumWithUnionFunctionDescriptors(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// USBCCGP_EnumWithAudioLegacy(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     ULONG Index;
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor, FirstDescriptor = NULL;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     //
//     // get device extension
//     //
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//
//     //
//     // first check if all interfaces belong to the same audio class
//     //
//     for (Index = 0; Index < FDODeviceExtension->ConfigurationDescriptor->bNumInterfaces; Index++)
//     {
//         //
//         // get interface descriptor
//         //
//         InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(FDODeviceExtension->ConfigurationDescriptor, FDODeviceExtension->ConfigurationDescriptor, Index, 0, -1, -1, -1);
//         DPRINT1("Index %lu Descriptor %p\n", Index, InterfaceDescriptor);
//         ASSERT(InterfaceDescriptor);
//
//         if (InterfaceDescriptor->bInterfaceClass != USB_DEVICE_CLASS_AUDIO)
//         {
//             //
//             // collection contains non audio class
//             //
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         if (FirstDescriptor == NULL)
//         {
//             //
//             // store interface descriptor
//             //
//             FirstDescriptor = InterfaceDescriptor;
//             continue;
//         }
//
//         if (FirstDescriptor->bInterfaceSubClass == InterfaceDescriptor->bInterfaceSubClass)
//         {
//             //
//             // interface subclass must be different from the first interface
//             //
//             return STATUS_UNSUCCESSFUL;
//         }
//     }
//
//     //
//     // this is an composite audio device
//     //
//     DPRINT("[USBCCGP] Audio Composite Device detected\n");
//
//     //
//     // audio interfaces are all grouped into one single function
//     //
//     FDODeviceExtension->FunctionDescriptor = AllocateItem(NonPagedPool, sizeof(USBC_FUNCTION_DESCRIPTOR));
//     if (!FDODeviceExtension->FunctionDescriptor)
//     {
//         //
//         // no memory
//         //
//         DPRINT1("USBCCGP_EnumWithAssociationDescriptor failed to allocate function descriptor count\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // init function number
//     //
//     FDODeviceExtension->FunctionDescriptor[0].FunctionNumber = 0;
//
//     //
//     // store interfaces
//     //
//     Status = AllocateInterfaceDescriptorsArray(FDODeviceExtension->ConfigurationDescriptor, &FDODeviceExtension->FunctionDescriptor[0].InterfaceDescriptorList);
//     if (!NT_SUCCESS(Status))
//     {
//         //
//         // failed to allocate descriptor array
//         //
//         DPRINT1("[USBCCGP] Failed to allocate descriptor array %lx\n", Status);
//         return Status;
//     }
//
//     //
//     // now init the device ids
//     //
//     Status = USBCCG_InitIdsWithInterfaceDescriptor(FDODeviceExtension, FirstDescriptor, 0, &FDODeviceExtension->FunctionDescriptor[0]);
//     if (!NT_SUCCESS(Status))
//     {
//         //
//         // failed to init ids
//         //
//         DPRINT1("[USBCCGP] Failed to init ids with %lx\n", Status);
//         return Status;
//     }
//
//     //
//     // number of interfaces
//     //
//     FDODeviceExtension->FunctionDescriptor[0].NumberOfInterfaces = FDODeviceExtension->ConfigurationDescriptor->bNumInterfaces;
//
//     //
//     // store function count
//     //
//     FDODeviceExtension->FunctionDescriptorCount = 1;
//
//     //
//     // done
//     //
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USBCCGP_EnumerateFunctions(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     NTSTATUS Status;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     //
//     // get device extension
//     //
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     //
//     // first try with filter driver
//     //
//     Status = USBCCGP_CustomEnumWithInterface(DeviceObject);
//     if (NT_SUCCESS(Status))
//     {
//         //
//         // succeeded
//         //
//         return Status;
//     }
//
//     //
//     // enumerate functions with interface association descriptor
//     //
//     Status = USBCCGP_EnumWithAssociationDescriptor(DeviceObject);
//     if (NT_SUCCESS(Status))
//     {
//         //
//         // succeeded
//         //
//         return Status;
//     }
//
//
//     //
//     // try with legacy audio methods
//     //
//     Status = USBCCGP_EnumWithAudioLegacy(DeviceObject);
//     if (NT_SUCCESS(Status))
//     {
//         //
//         // succeeded
//         //
//         return Status;
//     }
//
//     //
//     // try with legacy enumeration
//     //
//     return USBCCGP_LegacyEnum(DeviceObject);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================