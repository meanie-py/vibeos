use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, ExAllocatePool, ExFreePool, IoOpenDeviceRegistryKey, KEY_VALUE_PARTIAL_INFORMATION, PDEVICE_OBJECT, PDRIVER_OBJECT, PKEY_VALUE_PARTIAL_INFORMATION, PUNICODE_STRING, SIZE_T, ULONG_PTR, USHORT, ZwClose, ZwQueryValueKey, _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as KeyValuePartialInformation, _UNICODE_STRING};
use core::ffi::c_void;
use usb_sys::{PUSB_CONFIGURATION_DESCRIPTOR, PUSBD_INTERFACE_INFORMATION, PUSBD_INTERFACE_LIST_ENTRY, URB, USB_CONFIGURATION_DESCRIPTOR, USB_INTERFACE_DESCRIPTOR, USBD_INTERFACE_INFORMATION, USBD_PIPE_INFORMATION, USBD_VERSION_INFORMATION, USBDI_VERSION, URB_FUNCTION_SELECT_CONFIGURATION, _URB, _URB_SELECT_CONFIGURATION, _USB_COMMON_DESCRIPTOR};
type PURB = *mut _URB;

extern "C" {
    fn DbgPrint(format: *const core::ffi::c_char, ...);
}
// RtlAssert is NTAPI (stdcall `_RtlAssert@16` on i386) — must be extern "system",
// not extern "C", or the i386 link fails. (DbgPrint above stays cdecl-variadic.)
extern "system" {
    fn RtlAssert( pv1: *const core::ffi::c_void, pv2: *const core::ffi::c_void, ul: u32, pv3: *const core::ffi::c_void, );
}

// ---- file-scope preamble (translated) ----
// PnP registry-key selectors (cfgmgr/wdm): DEVICE=1, DRIVER=2. Defined locally
// (cross-arch; the hal_ddk module is x86-only / configured out on arm64).
pub const PLUGPLAY_REGKEY_DEVICE: u32 = 1;
pub const PLUGPLAY_REGKEY_DRIVER: u32 = 2;

const USB_INTERFACE_DESCRIPTOR_TYPE: i32 = 4;

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DllInitialize(Unknown: ULONG) -> ULONG {
    return 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn DllUnload() -> u32 {
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn USBD_Debug_GetHeap(
    Unknown1: ULONG,
    PoolType: POOL_TYPE,
    NumberOfBytes: ULONG,
    Tag: ULONG,
) -> PVOID {
    return ExAllocatePoolWithTag(PoolType, NumberOfBytes as usize, Tag);
}

#[no_mangle]
pub unsafe extern "system" fn USBD_Debug_RetHeap(
    Heap: PVOID,
    Unknown2: ULONG,
    Unknown3: ULONG,
) {
    ExFreePool(Heap);
}

#[no_mangle]
pub unsafe extern "system" fn USBD_Debug_LogEntry(
    Name: *mut i8,
    Info1: usize,
    Info2: usize,
    Info3: usize,
) {
}

#[no_mangle]
pub unsafe extern "system" fn USBD_AllocateDeviceName(Unknown: u32) -> *mut core::ffi::c_void {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
        b"USBD_AllocateDeviceName\0".as_ptr() as *const core::ffi::c_char,
        b"drivers/usb/usbd/usbd.c\0".as_ptr()
            as *const core::ffi::c_char,
        104i32,
    );
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn USBD_CalculateUsbBandwidth(
    MaxPacketSize: u32,
    EndpointType: u8,
    LowSpeed: u8,
) -> u32 {
    let mut OverheadTable: [u32; 4] = [0x00, 0x09, 0x00, 0x0d];
    let Result: u32;
    if OverheadTable[EndpointType as usize] != 0 {
        Result = MaxPacketSize
            .wrapping_add(OverheadTable[EndpointType as usize])
            .wrapping_mul(8)
            .wrapping_mul(7)
            .wrapping_div(6);
        if LowSpeed != 0 {
            return Result << 3;
        }
        return Result;
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn USBD_Dispatch(
    Unknown1: u32,
    Unknown2: u32,
    Unknown3: u32,
    Unknown4: u32,
) -> u32 {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8,
        b"USBD_Dispatch\0" as *const u8 as *const i8,
        b"drivers/usb/usbd/usbd.c\0" as *const u8 as *const i8,
        142i32,
    );
    return 1u32;
}

#[no_mangle]
pub unsafe extern "system" fn USBD_FreeDeviceMutex(Unknown: *mut core::ffi::c_void) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const core::ffi::c_char,
        b"USBD_FreeDeviceMutex\0" as *const u8 as *const core::ffi::c_char,
        b"drivers/usb/usbd/usbd.c\0" as *const u8 as *const core::ffi::c_char,
        152 as core::ffi::c_int,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBD_FreeDeviceName(Unknown: PVOID) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"USBD_FreeDeviceName\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/usb/usbd/usbd.c\0".as_ptr()
            as *const ::core::ffi::c_char,
        161 as ::core::ffi::c_int,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBD_WaitDeviceMutex(Unknown: *mut core::ffi::c_void) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"USBD_WaitDeviceMutex\0".as_ptr() as *const i8,
        b"drivers/usb/usbd/usbd.c\0".as_ptr() as *const i8,
        170 as i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBD_GetSuspendPowerState(Unknown1: ULONG) -> ULONG {
    DbgPrint(
        c"WARNING:  %s at %s:%d is unimplemented!()\n".as_ptr(),
        c"USBD_GetSuspendPowerState".as_ptr(),
        c"drivers/usb/usbd/usbd.c".as_ptr(),
        179i32,
    );
    return 0u32;
}
#[no_mangle]
pub unsafe extern "system" fn USBD_InitializeDevice(
    Unknown1: u32,
    Unknown2: u32,
    Unknown3: u32,
    Unknown4: u32,
    Unknown5: u32,
    Unknown6: u32,
) -> i32 {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"USBD_InitializeDevice\0".as_ptr() as *const i8,
        b"drivers/usb/usbd/usbd.c\0".as_ptr() as *const i8,
        190i32,
    );
    return STATUS_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn USBD_RegisterHostController(
    Unknown1: u32,
    Unknown2: u32,
    Unknown3: u32,
    Unknown4: u32,
    Unknown5: u32,
    Unknown6: u32,
    Unknown7: u32,
    Unknown8: u32,
    Unknown9: u32,
    Unknown10: u32,
) -> i32 {

    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8,
        b"USBD_RegisterHostController\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/usb/usbd/usbd.c\0" as *const u8
            as *const ::core::ffi::c_char,
        202 as ::core::ffi::c_int,
    );
    return STATUS_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn USBD_GetDeviceInformation(
    Unknown1: u32,
    Unknown2: u32,
    Unknown3: u32,
) -> i32 {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"USBD_GetDeviceInformation\0".as_ptr() as *const i8,
        b"drivers/usb/usbd/usbd.c\0".as_ptr() as *const i8,
        212i32,
    );
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn USBD_CreateDevice(
    Unknown1: ULONG,
    Unknown2: ULONG,
    Unknown3: ULONG,
    Unknown4: ULONG,
    Unknown5: ULONG,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"USBD_CreateDevice\0".as_ptr() as *const i8,
        b"drivers/usb/usbd/usbd.c\0".as_ptr() as *const i8,
        223i32,
    );
    return STATUS_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn USBD_RemoveDevice(
    Unknown1: u32,
    Unknown2: u32,
    Unknown3: u32,
) -> i32 {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"USBD_RemoveDevice\0".as_ptr() as *const i8,
        b"drivers/usb/usbd/usbd.c\0".as_ptr() as *const i8,
        233i32,
    );
    return STATUS_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn USBD_CompleteRequest(Unknown1: ULONG, Unknown2: ULONG) {

    unsafe {
        DbgPrint(
            b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
            b"USBD_CompleteRequest\0".as_ptr() as *const i8,
            b"drivers/usb/usbd/usbd.c\0".as_ptr() as *const i8,
            243i32,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBD_RegisterHcFilter(
    DeviceObject: PDEVICE_OBJECT,
    FilterDeviceObject: PDEVICE_OBJECT,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8,
        b"USBD_RegisterHcFilter\0" as *const u8 as *const i8,
        b"drivers/usb/usbd/usbd.c\0" as *const u8
            as *const i8,
        255 as ::core::ffi::c_int,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBD_SetSuspendPowerState(Unknown1: ULONG, Unknown2: ULONG) {
    let format_str = b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8;
    let func_name = b"USBD_SetSuspendPowerState\0".as_ptr() as *const i8;
    let file_name = b"drivers/usb/usbd/usbd.c\0".as_ptr() as *const i8;
    DbgPrint(format_str, func_name, file_name, 264 as core::ffi::c_int);
}

#[no_mangle]
pub unsafe extern "system" fn USBD_MakePdoName(Unknown1: u32, Unknown2: u32) -> i32 {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"USBD_MakePdoName\0".as_ptr() as *const i8,
        b"drivers/usb/usbd/usbd.c\0".as_ptr() as *const i8,
        273i32,
    );
    return STATUS_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn USBD_QueryBusTime(
    RootHubPdo: PDEVICE_OBJECT,
    CurrentFrame: PULONG,
) -> NTSTATUS {

    let fmt = b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8;
    let funcname = b"USBD_QueryBusTime\0".as_ptr() as *const i8;
    let filename = b"drivers/usb/usbd/usbd.c\0".as_ptr()
        as *const i8;

    DbgPrint(fmt, funcname, filename, 286i32);

    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn USBD_GetUSBDIVersion(
    Version: *mut USBD_VERSION_INFORMATION,
) {
    if !Version.is_null() {
        (*Version).USBDI_Version = USBDI_VERSION as u32;
        (*Version).Supported_USB_Version = 0x200u32;
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBD_RestoreDevice(
    Unknown1: u32,
    Unknown2: u32,
    Unknown3: u32,
) -> i32 {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"USBD_RestoreDevice\0".as_ptr() as *const i8,
        b"usbd.c\0".as_ptr() as *const i8,
        311,
    );
    return STATUS_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn USBD_RegisterHcDeviceCapabilities(
    Unknown1: u32,
    Unknown2: u32,
    Unknown3: u32,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"USBD_RegisterHcDeviceCapabilities\0".as_ptr() as *const i8,
        b"drivers/usb/usbd/usbd.c\0".as_ptr()
            as *const i8,
        322i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBD_CreateConfigurationRequestEx(
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    InterfaceList: PUSBD_INTERFACE_LIST_ENTRY,
) -> PURB {
    unsafe {
        let mut Urb: PURB = core::ptr::null_mut::<_URB>();
        let mut UrbSize: ULONG = 0;
        let mut InterfaceCount: ULONG = 0;
        let mut PipeCount: ULONG = 0;
        let mut InterfaceNumber: ULONG;
        let mut EndPointNumber: ULONG;
        let mut InterfaceInfo: PUSBD_INTERFACE_INFORMATION;

        while !(*InterfaceList.offset(InterfaceCount as isize)).InterfaceDescriptor.is_null() {
            PipeCount = (PipeCount as usize)
                .wrapping_add(
                    (*(*InterfaceList.offset(InterfaceCount as isize)).InterfaceDescriptor)
                        .bNumEndpoints as usize,
                )
                as ULONG;
            InterfaceCount = InterfaceCount.wrapping_add(1);
        }

        UrbSize = (core::mem::size_of::<_URB_SELECT_CONFIGURATION>() as usize)
            .wrapping_add(
                ((InterfaceCount as usize).wrapping_sub(1))
                    .wrapping_mul(core::mem::size_of::<USBD_INTERFACE_INFORMATION>() as usize),
            )
            .wrapping_add(
                ((PipeCount as usize).wrapping_sub(InterfaceCount as usize))
                    .wrapping_mul(core::mem::size_of::<USBD_PIPE_INFORMATION>() as usize),
            )
            as ULONG;

        Urb = ExAllocatePool(NonPagedPool, UrbSize as SIZE_T) as PURB;
        if Urb.is_null() {
            return core::ptr::null_mut::<_URB>();
        }

        core::ptr::write_bytes(Urb as *mut u8, 0, UrbSize as usize);

        (*Urb)
            .__bindgen_anon_1
            .UrbSelectConfiguration
            .Hdr
            .Function = URB_FUNCTION_SELECT_CONFIGURATION as USHORT;
        (*Urb)
            .__bindgen_anon_1
            .UrbSelectConfiguration
            .Hdr
            .Length = UrbSize as USHORT;
        (*Urb)
            .__bindgen_anon_1
            .UrbSelectConfiguration
            .ConfigurationDescriptor = ConfigurationDescriptor;

        InterfaceInfo = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbSelectConfiguration.Interface)
            as PUSBD_INTERFACE_INFORMATION;

        for InterfaceNumber in 0..InterfaceCount {
            (*InterfaceList.offset(InterfaceNumber as isize)).Interface = InterfaceInfo;

            (*InterfaceInfo).InterfaceNumber =
                (*(*InterfaceList.offset(InterfaceNumber as isize)).InterfaceDescriptor)
                    .bInterfaceNumber;
            (*InterfaceInfo).AlternateSetting =
                (*(*InterfaceList.offset(InterfaceNumber as isize)).InterfaceDescriptor)
                    .bAlternateSetting;
            (*InterfaceInfo).NumberOfPipes =
                (*(*InterfaceList.offset(InterfaceNumber as isize)).InterfaceDescriptor)
                    .bNumEndpoints as ULONG;

            (*InterfaceInfo).Length = (core::mem::size_of::<USBD_INTERFACE_INFORMATION>() as usize)
                .wrapping_add(
                    (core::mem::size_of::<USBD_PIPE_INFORMATION>() as usize).wrapping_mul(
                        (*(*InterfaceList.offset(InterfaceNumber as isize)).InterfaceDescriptor)
                            .bNumEndpoints as usize,
                    ),
                )
                .wrapping_sub(core::mem::size_of::<USBD_PIPE_INFORMATION>() as usize)
                as USHORT;

            for EndPointNumber in 0..(*InterfaceInfo).NumberOfPipes {
                (*(core::ptr::addr_of_mut!((*InterfaceInfo).Pipes) as *mut USBD_PIPE_INFORMATION)
                    .offset(EndPointNumber as isize))
                    .MaximumTransferSize = PAGE_SIZE as ULONG;
            }

            InterfaceInfo = (InterfaceInfo as *mut u8)
                .add((*InterfaceInfo).Length as usize) as PUSBD_INTERFACE_INFORMATION;
        }

        Urb
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBD_CreateConfigurationRequest(
    ConfigurationDescriptor: *mut USB_CONFIGURATION_DESCRIPTOR,
    Size: *mut u16,
) -> *mut URB {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn USBD_GetInterfaceLength(
    InterfaceDescriptor: *mut USB_INTERFACE_DESCRIPTOR,
    BufferEnd: *mut u8,
) -> u32 {
    let mut Current: usize;
    let mut CurrentDescriptor: *mut USB_INTERFACE_DESCRIPTOR = InterfaceDescriptor;
    let mut Length: u32 = 0;
    let mut InterfaceFound: u8 = 0; // FALSE
    Current = CurrentDescriptor as usize;
    while Current < BufferEnd as usize {
        CurrentDescriptor = Current as *mut USB_INTERFACE_DESCRIPTOR;
        if (*CurrentDescriptor).bDescriptorType as i32 == USB_INTERFACE_DESCRIPTOR_TYPE && InterfaceFound != 0 {
            break;
        }
        if (*CurrentDescriptor).bDescriptorType as i32 == USB_INTERFACE_DESCRIPTOR_TYPE {
            InterfaceFound = 1; // TRUE
        }
        Length += (*CurrentDescriptor).bLength as u32;
        Current += (*CurrentDescriptor).bLength as usize;
    }
    Length
}

#[no_mangle]
pub unsafe extern "system" fn USBD_ParseDescriptors(
    DescriptorBuffer: *mut core::ffi::c_void,
    TotalLength: u32,
    StartPosition: *mut core::ffi::c_void,
    DescriptorType: i32,
) -> *mut _USB_COMMON_DESCRIPTOR {
    let mut CommonDescriptor: *mut _USB_COMMON_DESCRIPTOR = core::ptr::null_mut();
    CommonDescriptor = StartPosition as *mut _USB_COMMON_DESCRIPTOR;
    while !CommonDescriptor.is_null() {
        if (CommonDescriptor as usize) >= (DescriptorBuffer as usize).wrapping_add(TotalLength as usize) {
            return core::ptr::null_mut::<_USB_COMMON_DESCRIPTOR>();
        }
        if (*CommonDescriptor).bDescriptorType as i32 == DescriptorType {
            return CommonDescriptor;
        }
        if (*CommonDescriptor).bLength == 0 {
            return core::ptr::null_mut::<_USB_COMMON_DESCRIPTOR>();
        }
        CommonDescriptor = ((CommonDescriptor as usize).wrapping_add((*CommonDescriptor).bLength as usize)) as *mut _USB_COMMON_DESCRIPTOR;
    }
    return core::ptr::null_mut::<_USB_COMMON_DESCRIPTOR>();
}

#[no_mangle]
pub unsafe extern "system" fn USBD_ParseConfigurationDescriptorEx(
    ConfigurationDescriptor: *mut USB_CONFIGURATION_DESCRIPTOR,
    StartPosition: *mut core::ffi::c_void,
    InterfaceNumber: i32,
    AlternateSetting: i32,
    InterfaceClass: i32,
    InterfaceSubClass: i32,
    InterfaceProtocol: i32,
) -> *mut USB_INTERFACE_DESCRIPTOR {
    let mut Found: i32 = 0;
    let mut InterfaceDescriptor: *mut USB_INTERFACE_DESCRIPTOR = StartPosition as *mut USB_INTERFACE_DESCRIPTOR;

    while !InterfaceDescriptor.is_null() {
        InterfaceDescriptor = USBD_ParseDescriptors(
            ConfigurationDescriptor as *mut core::ffi::c_void,
            (*ConfigurationDescriptor).wTotalLength as u32,
            InterfaceDescriptor as *mut core::ffi::c_void,
            USB_INTERFACE_DESCRIPTOR_TYPE,
        ) as *mut USB_INTERFACE_DESCRIPTOR;

        if InterfaceDescriptor.is_null() {
            break;
        }

        Found = 1;

        if InterfaceNumber != -(1 as i32) {
            if InterfaceNumber != (*InterfaceDescriptor).bInterfaceNumber as i32 {
                Found = 0;
            }
        }

        if AlternateSetting != -(1 as i32) {
            if AlternateSetting != (*InterfaceDescriptor).bAlternateSetting as i32 {
                Found = 0;
            }
        }

        if InterfaceClass != -(1 as i32) {
            if InterfaceClass != (*InterfaceDescriptor).bInterfaceClass as i32 {
                Found = 0;
            }
        }

        if InterfaceSubClass != -(1 as i32) {
            if InterfaceSubClass != (*InterfaceDescriptor).bInterfaceSubClass as i32 {
                Found = 0;
            }
        }

        if InterfaceProtocol != -(1 as i32) {
            if InterfaceProtocol != (*InterfaceDescriptor).bInterfaceProtocol as i32 {
                Found = 0;
            }
        }

        if Found != 0 {
            return InterfaceDescriptor;
        }

        if (*InterfaceDescriptor).bLength == 0 {
            RtlAssert(
                b"InterfaceDescriptor->bLength\0".as_ptr() as *const core::ffi::c_void,
                b"drivers/usb/usbd/usbd.c\0".as_ptr()
                    as *const core::ffi::c_void,
                588u32,
                core::ptr::null_mut(),
            );
        }

        InterfaceDescriptor = ((InterfaceDescriptor as usize)
            .wrapping_add((*InterfaceDescriptor).bLength as usize))
            as *mut USB_INTERFACE_DESCRIPTOR;
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn USBD_ParseConfigurationDescriptor(
    ConfigurationDescriptor: *mut USB_CONFIGURATION_DESCRIPTOR,
    InterfaceNumber: u8,
    AlternateSetting: u8,
) -> *mut USB_INTERFACE_DESCRIPTOR {
    USBD_ParseConfigurationDescriptorEx(
        ConfigurationDescriptor,
        ConfigurationDescriptor as *mut core::ffi::c_void,
        InterfaceNumber as i32,
        AlternateSetting as i32,
        -1i32,
        -1i32,
        -1i32,
    )
}

#[no_mangle]
pub unsafe extern "system" fn USBD_GetPdoRegistryParameter(
    PhysicalDeviceObject: PDEVICE_OBJECT,
    Parameter: PVOID,
    ParameterLength: ULONG,
    KeyName: PWCHAR,
    KeyNameLength: ULONG,
) -> ULONG {
    let mut Status: NTSTATUS = 0;
    let mut DevInstRegKey: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();

    Status = IoOpenDeviceRegistryKey(
        PhysicalDeviceObject,
        PLUGPLAY_REGKEY_DEVICE as ULONG,
        STANDARD_RIGHTS_ALL as ACCESS_MASK,
        &raw mut DevInstRegKey,
    );

    if Status >= 0 {
        let mut PartialInfo: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut::<KEY_VALUE_PARTIAL_INFORMATION>();
        let mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut::<u16>(),
        };
        let mut Length: ULONG;

        ValueName.Buffer = KeyName;
        ValueName.MaximumLength = KeyNameLength as u16;
        ValueName.Length = ValueName.MaximumLength;

        Length = (ParameterLength as usize)
            .wrapping_add(core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>()) as ULONG;

        PartialInfo = ExAllocatePool(PagedPool, Length as SIZE_T) as PKEY_VALUE_PARTIAL_INFORMATION;

        if !PartialInfo.is_null() {
            Status = ZwQueryValueKey(
                DevInstRegKey,
                &raw mut ValueName,
                KeyValuePartialInformation,
                PartialInfo as PVOID,
                Length,
                &raw mut Length,
            );

            if Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL {
                ExFreePool(PartialInfo as PVOID);
                PartialInfo = ExAllocatePool(PagedPool, Length as SIZE_T) as PKEY_VALUE_PARTIAL_INFORMATION;

                if !PartialInfo.is_null() {
                    Status = ZwQueryValueKey(
                        DevInstRegKey,
                        &raw mut ValueName,
                        KeyValuePartialInformation,
                        PartialInfo as PVOID,
                        Length,
                        &raw mut Length,
                    );
                } else {
                    Status = STATUS_NO_MEMORY;
                }
            }

            if Status >= 0 {
                if ParameterLength < (*PartialInfo).DataLength {
                    Length = ParameterLength;
                } else {
                    Length = (*PartialInfo).DataLength;
                }

                core::ptr::copy_nonoverlapping(
                    &raw mut (*PartialInfo).Data as *mut u8 as *const u8,
                    Parameter as *mut u8,
                    Length as usize,
                );
            }

            if !PartialInfo.is_null() {
                ExFreePool(PartialInfo as PVOID);
            }
        } else {
            Status = STATUS_NO_MEMORY;
        }

        ZwClose(DevInstRegKey);
    }

    Status as ULONG
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Universal Serial Bus Driver/Helper Library
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        drivers/usb/usbd/usbd.c
//  * PURPOSE:     Helper Library for USB
//  * PROGRAMMERS:
//  *              Filip Navara <xnavara@volny.cz>
//  *              Michael Martin <michael.martin@reactos.org>
//  *
//  */
//
// /*
//  * Universal Serial Bus Driver/Helper Library
//  *
//  * Written by Filip Navara <xnavara@volny.cz>
//  *
//  * Notes:
//  *    This driver was obsoleted in Windows XP and most functions
//  *    became pure stubs. But some of them were retained for backward
//  *    compatibility with existing drivers.
//  *
//  *    Preserved functions:
//  *
//  *    USBD_Debug_GetHeap (implemented)
//  *    USBD_Debug_RetHeap (implemented)
//  *    USBD_CalculateUsbBandwidth (implemented, tested)
//  *    USBD_CreateConfigurationRequestEx (implemented)
//  *    USBD_CreateConfigurationRequest
//  *    USBD_GetInterfaceLength (implemented)
//  *    USBD_ParseConfigurationDescriptorEx (implemented)
//  *    USBD_ParseDescriptors (implemented)
//  *    USBD_GetPdoRegistryParameters (implemented)
//  */
//
// #define _USBD_
// #define NDEBUG
// #include <ntddk.h>
// #include <usbdi.h>
// #include <usbdlib.h>
// #include <debug.h>
// #ifndef PLUGPLAY_REGKEY_DRIVER
// #define PLUGPLAY_REGKEY_DRIVER              2
// #endif
//
// NTSTATUS NTAPI
// DriverEntry(PDRIVER_OBJECT DriverObject,
//             PUNICODE_STRING RegistryPath)
// {
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// ULONG NTAPI
// DllInitialize(ULONG Unknown)
// {
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// ULONG NTAPI
// DllUnload(VOID)
// {
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// PVOID NTAPI
// USBD_Debug_GetHeap(ULONG Unknown1, POOL_TYPE PoolType, ULONG NumberOfBytes,
//                    ULONG Tag)
// {
//     return ExAllocatePoolWithTag(PoolType, NumberOfBytes, Tag);
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// USBD_Debug_RetHeap(PVOID Heap, ULONG Unknown2, ULONG Unknown3)
// {
//     ExFreePool(Heap);
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// USBD_Debug_LogEntry(PCHAR Name, ULONG_PTR Info1, ULONG_PTR Info2,
//     ULONG_PTR Info3)
// {
// }
//
// /*
//  * @implemented
//  */
// PVOID NTAPI
// USBD_AllocateDeviceName(ULONG Unknown)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @implemented
//  */
// ULONG NTAPI
// USBD_CalculateUsbBandwidth(
//     ULONG MaxPacketSize,
//     UCHAR EndpointType,
//     BOOLEAN LowSpeed
//     )
// {
//     ULONG OverheadTable[] = {
//             0x00, /* UsbdPipeTypeControl */
//             0x09, /* UsbdPipeTypeIsochronous */
//             0x00, /* UsbdPipeTypeBulk */
//             0x0d  /* UsbdPipeTypeInterrupt */
//         };
//     ULONG Result;
//
//     if (OverheadTable[EndpointType] != 0)
//     {
//         Result = ((MaxPacketSize + OverheadTable[EndpointType]) * 8 * 7) / 6;
//         if (LowSpeed)
//            return Result << 3;
//         return Result;
//     }
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// ULONG NTAPI
// USBD_Dispatch(ULONG Unknown1, ULONG Unknown2, ULONG Unknown3, ULONG Unknown4)
// {
//     UNIMPLEMENTED;
//     return 1;
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// USBD_FreeDeviceMutex(PVOID Unknown)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// USBD_FreeDeviceName(PVOID Unknown)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// USBD_WaitDeviceMutex(PVOID Unknown)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @implemented
//  */
// ULONG NTAPI
// USBD_GetSuspendPowerState(ULONG Unknown1)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS NTAPI
// USBD_InitializeDevice(ULONG Unknown1, ULONG Unknown2, ULONG Unknown3,
//     ULONG Unknown4, ULONG Unknown5, ULONG Unknown6)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_SUPPORTED;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS NTAPI
// USBD_RegisterHostController(ULONG Unknown1, ULONG Unknown2, ULONG Unknown3,
//     ULONG Unknown4, ULONG Unknown5, ULONG Unknown6, ULONG Unknown7,
//     ULONG Unknown8, ULONG Unknown9, ULONG Unknown10)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_SUPPORTED;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS NTAPI
// USBD_GetDeviceInformation(ULONG Unknown1, ULONG Unknown2, ULONG Unknown3)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_SUPPORTED;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS NTAPI
// USBD_CreateDevice(ULONG Unknown1, ULONG Unknown2, ULONG Unknown3,
//     ULONG Unknown4, ULONG Unknown5)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_SUPPORTED;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS NTAPI
// USBD_RemoveDevice(ULONG Unknown1, ULONG Unknown2, ULONG Unknown3)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_SUPPORTED;
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// USBD_CompleteRequest(ULONG Unknown1, ULONG Unknown2)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// USBD_RegisterHcFilter(
//     PDEVICE_OBJECT DeviceObject,
//     PDEVICE_OBJECT FilterDeviceObject
//     )
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// USBD_SetSuspendPowerState(ULONG Unknown1, ULONG Unknown2)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS NTAPI
// USBD_MakePdoName(ULONG Unknown1, ULONG Unknown2)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_SUPPORTED;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS NTAPI
// USBD_QueryBusTime(
//     PDEVICE_OBJECT RootHubPdo,
//     PULONG CurrentFrame
//     )
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_SUPPORTED;
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// USBD_GetUSBDIVersion(
//     PUSBD_VERSION_INFORMATION Version
//     )
// {
//     if (Version != NULL)
//     {
//         Version->USBDI_Version = USBDI_VERSION;
//         Version->Supported_USB_Version = 0x200;
//     }
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS NTAPI
// USBD_RestoreDevice(ULONG Unknown1, ULONG Unknown2, ULONG Unknown3)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_SUPPORTED;
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// USBD_RegisterHcDeviceCapabilities(ULONG Unknown1, ULONG Unknown2,
//     ULONG Unknown3)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @implemented
//  */
// PURB NTAPI
// USBD_CreateConfigurationRequestEx(
//     PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     PUSBD_INTERFACE_LIST_ENTRY InterfaceList
//     )
// {
//     PURB Urb;
//     ULONG UrbSize = 0;
//     ULONG InterfaceCount = 0, PipeCount = 0;
//     ULONG InterfaceNumber, EndPointNumber;
//     PUSBD_INTERFACE_INFORMATION InterfaceInfo;
//
//     while(InterfaceList[InterfaceCount].InterfaceDescriptor)
//     {
//         // pipe count
//         PipeCount += InterfaceList[InterfaceCount].InterfaceDescriptor->bNumEndpoints;
//
//         // interface count
//         InterfaceCount++;
//     }
//
//     // size of urb
//     UrbSize = GET_SELECT_CONFIGURATION_REQUEST_SIZE(InterfaceCount, PipeCount);
//
//     // allocate urb
//     Urb = ExAllocatePool(NonPagedPool, UrbSize);
//     if (!Urb)
//     {
//         // no memory
//         return NULL;
//     }
//
//     // zero urb
//     RtlZeroMemory(Urb, UrbSize);
//
//     // init urb header
//     Urb->UrbSelectConfiguration.Hdr.Function =  URB_FUNCTION_SELECT_CONFIGURATION;
//     Urb->UrbSelectConfiguration.Hdr.Length = UrbSize;
//     Urb->UrbSelectConfiguration.ConfigurationDescriptor = ConfigurationDescriptor;
//
//     // init interface information
//     InterfaceInfo = &Urb->UrbSelectConfiguration.Interface;
//     for (InterfaceNumber = 0; InterfaceNumber < InterfaceCount; InterfaceNumber++)
//     {
//         // init interface info
//         InterfaceList[InterfaceNumber].Interface = InterfaceInfo;
//         InterfaceInfo->InterfaceNumber = InterfaceList[InterfaceNumber].InterfaceDescriptor->bInterfaceNumber;
//         InterfaceInfo->AlternateSetting = InterfaceList[InterfaceNumber].InterfaceDescriptor->bAlternateSetting;
//         InterfaceInfo->NumberOfPipes = InterfaceList[InterfaceNumber].InterfaceDescriptor->bNumEndpoints;
//
//         // store length
//         InterfaceInfo->Length = GET_USBD_INTERFACE_SIZE(InterfaceList[InterfaceNumber].InterfaceDescriptor->bNumEndpoints);
//
//         // sanity check
//         //C_ASSERT(FIELD_OFFSET(USBD_INTERFACE_INFORMATION, Pipes) == 16);
//
//         for (EndPointNumber = 0; EndPointNumber < InterfaceInfo->NumberOfPipes; EndPointNumber++)
//         {
//             // init max transfer size
//             InterfaceInfo->Pipes[EndPointNumber].MaximumTransferSize = PAGE_SIZE;
//         }
//
//         // next interface info
//         InterfaceInfo = (PUSBD_INTERFACE_INFORMATION) ((ULONG_PTR)InterfaceInfo + InterfaceInfo->Length);
//     }
//
//     return Urb;
// }
//
// /*
//  * @implemented
//  */
// PURB NTAPI
// USBD_CreateConfigurationRequest(
//     PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     PUSHORT Size
//     )
// {
//     /* WindowsXP returns NULL */
//     return NULL;
// }
//
// /*
//  * @implemented
//  */
// ULONG NTAPI
// USBD_GetInterfaceLength(
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor,
//     PUCHAR BufferEnd
//     )
// {
//     ULONG_PTR Current;
//     PUSB_INTERFACE_DESCRIPTOR CurrentDescriptor = InterfaceDescriptor;
//     ULONG Length = 0;
//     BOOLEAN InterfaceFound = FALSE;
//
//     for (Current = (ULONG_PTR)CurrentDescriptor;
//          Current < (ULONG_PTR)BufferEnd;
//          Current += CurrentDescriptor->bLength)
//     {
//         CurrentDescriptor = (PUSB_INTERFACE_DESCRIPTOR)Current;
//
//         if ((CurrentDescriptor->bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE) && (InterfaceFound))
//             break;
//         else if (CurrentDescriptor->bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE)
//             InterfaceFound = TRUE;
//
//         Length += CurrentDescriptor->bLength;
//     }
//
//     return Length;
// }
//
// /*
//  * @implemented
//  */
// PUSB_COMMON_DESCRIPTOR NTAPI
// USBD_ParseDescriptors(
//     PVOID  DescriptorBuffer,
//     ULONG  TotalLength,
//     PVOID  StartPosition,
//     LONG  DescriptorType
//     )
// {
//     PUSB_COMMON_DESCRIPTOR CommonDescriptor;
//
//     /* use start position */
//     CommonDescriptor = (PUSB_COMMON_DESCRIPTOR)StartPosition;
//
//
//     /* find next available descriptor */
//     while(CommonDescriptor)
//     {
//        if ((ULONG_PTR)CommonDescriptor >= ((ULONG_PTR)DescriptorBuffer + TotalLength))
//        {
//            /* end reached */
//            DPRINT("End reached %p\n", CommonDescriptor);
//            return NULL;
//        }
//
//        DPRINT("CommonDescriptor Type %x Length %x\n", CommonDescriptor->bDescriptorType, CommonDescriptor->bLength);
//
//        /* is the requested one */
//        if (CommonDescriptor->bDescriptorType == DescriptorType)
//        {
//            /* it is */
//            return CommonDescriptor;
//        }
//
//        if (CommonDescriptor->bLength == 0)
//        {
//            /* invalid usb descriptor */
//            return NULL;
//        }
//
//        /* move to next descriptor */
//        CommonDescriptor = (PUSB_COMMON_DESCRIPTOR)((ULONG_PTR)CommonDescriptor + CommonDescriptor->bLength);
//     }
//
//     /* no descriptor found */
//     return NULL;
// }
//
//
// /*
//  * @implemented
//  */
// PUSB_INTERFACE_DESCRIPTOR NTAPI
// USBD_ParseConfigurationDescriptorEx(
//     PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     PVOID StartPosition,
//     LONG InterfaceNumber,
//     LONG AlternateSetting,
//     LONG InterfaceClass,
//     LONG InterfaceSubClass,
//     LONG InterfaceProtocol
//     )
// {
//     BOOLEAN Found;
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor;
//
//     /* set to start position */
//     InterfaceDescriptor = (PUSB_INTERFACE_DESCRIPTOR)StartPosition;
//
//     DPRINT("USBD_ParseConfigurationDescriptorEx\n");
//     DPRINT("ConfigurationDescriptor %p Length %lu\n", ConfigurationDescriptor, ConfigurationDescriptor->wTotalLength);
//     DPRINT("CurrentOffset %p Offset %lu\n", StartPosition, ((ULONG_PTR)StartPosition - (ULONG_PTR)ConfigurationDescriptor));
//
//     while(InterfaceDescriptor)
//     {
//        /* get interface descriptor */
//        InterfaceDescriptor = (PUSB_INTERFACE_DESCRIPTOR) USBD_ParseDescriptors(ConfigurationDescriptor, ConfigurationDescriptor->wTotalLength, InterfaceDescriptor, USB_INTERFACE_DESCRIPTOR_TYPE);
//        if (!InterfaceDescriptor)
//        {
//            /* no more descriptors available */
//            break;
//        }
//
//        DPRINT("InterfaceDescriptor %p InterfaceNumber %x AlternateSetting %x Length %lu\n", InterfaceDescriptor, InterfaceDescriptor->bInterfaceNumber, InterfaceDescriptor->bAlternateSetting, InterfaceDescriptor->bLength);
//
//        /* set found */
//        Found = TRUE;
//
//        /* is there an interface number provided */
//        if(InterfaceNumber != -1)
//        {
//           if(InterfaceNumber != InterfaceDescriptor->bInterfaceNumber)
//           {
//               /* interface number does not match */
//               Found = FALSE;
//           }
//        }
//
//        /* is there an alternate setting provided */
//        if(AlternateSetting != -1)
//        {
//           if(AlternateSetting != InterfaceDescriptor->bAlternateSetting)
//           {
//               /* alternate setting does not match */
//               Found = FALSE;
//           }
//        }
//
//        /* match on interface class */
//        if(InterfaceClass != -1)
//        {
//           if(InterfaceClass != InterfaceDescriptor->bInterfaceClass)
//           {
//               /* no match with interface class criteria */
//               Found = FALSE;
//           }
//        }
//
//        /* match on interface sub class */
//        if(InterfaceSubClass != -1)
//        {
//           if(InterfaceSubClass != InterfaceDescriptor->bInterfaceSubClass)
//           {
//               /* no interface sub class match */
//               Found = FALSE;
//           }
//        }
//
//        /* interface protocol criteria */
//        if(InterfaceProtocol != -1)
//        {
//           if(InterfaceProtocol != InterfaceDescriptor->bInterfaceProtocol)
//           {
//               /* no interface protocol match */
//               Found = FALSE;
//           }
//        }
//
//        if (Found)
//        {
//            /* the chosen one */
//            return InterfaceDescriptor;
//        }
//
//        /* sanity check */
//        ASSERT(InterfaceDescriptor->bLength);
//
//        /* move to next descriptor */
//        InterfaceDescriptor = (PUSB_INTERFACE_DESCRIPTOR)((ULONG_PTR)InterfaceDescriptor + InterfaceDescriptor->bLength);
//     }
//
//     DPRINT("No Descriptor With InterfaceNumber %ld AlternateSetting %ld InterfaceClass %ld InterfaceSubClass %ld InterfaceProtocol %ld found\n", InterfaceNumber,
//             AlternateSetting, InterfaceClass, InterfaceSubClass, InterfaceProtocol);
//
//     return NULL;
// }
//
// /*
//  * @implemented
//  */
// PUSB_INTERFACE_DESCRIPTOR NTAPI
// USBD_ParseConfigurationDescriptor(
//     PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     UCHAR InterfaceNumber,
//     UCHAR AlternateSetting
//     )
// {
//     return USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor,
//         (PVOID)ConfigurationDescriptor, InterfaceNumber, AlternateSetting,
//         -1, -1, -1);
// }
//
//
// /*
//  * @implemented
//  */
// ULONG NTAPI
// USBD_GetPdoRegistryParameter(
//     PDEVICE_OBJECT PhysicalDeviceObject,
//     PVOID Parameter,
//     ULONG ParameterLength,
//     PWCHAR KeyName,
//     ULONG KeyNameLength
//     )
// {
//     NTSTATUS Status;
//     HANDLE DevInstRegKey;
//
//     /* Open the device key */
//     Status = IoOpenDeviceRegistryKey(PhysicalDeviceObject,
//         PLUGPLAY_REGKEY_DEVICE, STANDARD_RIGHTS_ALL, &DevInstRegKey);
//     if (NT_SUCCESS(Status))
//     {
//         PKEY_VALUE_PARTIAL_INFORMATION PartialInfo;
//         UNICODE_STRING ValueName;
//         ULONG Length;
//
//         /* Initialize the unicode string based on caller data */
//         ValueName.Buffer = KeyName;
//         ValueName.Length = ValueName.MaximumLength = KeyNameLength;
//
//         Length = ParameterLength + sizeof(KEY_VALUE_PARTIAL_INFORMATION);
//         PartialInfo = ExAllocatePool(PagedPool, Length);
//         if (PartialInfo)
//         {
//             Status = ZwQueryValueKey(DevInstRegKey, &ValueName,
//                 KeyValuePartialInformation, PartialInfo, Length, &Length);
//             if (Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL)
//             {
//                 /* The caller doesn't want all the data */
//                 ExFreePool(PartialInfo);
//                 PartialInfo = ExAllocatePool(PagedPool, Length);
//                 if (PartialInfo)
//                 {
//                     Status = ZwQueryValueKey(DevInstRegKey, &ValueName,
//                        KeyValuePartialInformation, PartialInfo, Length, &Length);
//                 }
//                 else
//                 {
//                     Status = STATUS_NO_MEMORY;
//                 }
//             }
//
//             if (NT_SUCCESS(Status))
//             {
//                 /* Compute the length to copy back */
//                 if (ParameterLength < PartialInfo->DataLength)
//                     Length = ParameterLength;
//                 else
//                     Length = PartialInfo->DataLength;
//
//                 RtlCopyMemory(Parameter,
//                               PartialInfo->Data,
//                               Length);
//             }
//
//             if (PartialInfo)
//             {
//                 ExFreePool(PartialInfo);
//             }
//         } else
//             Status = STATUS_NO_MEMORY;
//         ZwClose(DevInstRegKey);
//     }
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
