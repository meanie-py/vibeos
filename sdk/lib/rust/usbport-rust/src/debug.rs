use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PCH, PDEVICE_CAPABILITIES, RtlAssert};
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: *const ::core::ffi::c_char, ...) -> u32;
}

// ---- file-scope preamble (translated) ----
// No constant declarations to emit; NDEBUG, NDEBUG_USBPORT_MINIPORT,
// and NDEBUG_USBPORT_URB are object-like macros with no value (compile-time flags).

/// Fixed-arity twin of the `USBPORT_DbgPrint` C-variadic entry point in
/// `drivers/usb/usbport/rust_varargs_shim.c`. The C side exists only to do the
/// `va_start` that stable Rust cannot (`c_variadic`, rust-lang#44930); `ap` is a
/// pointer to its `va_list` (unused — the C original is a FIXME no-op too).
#[no_mangle]
pub unsafe extern "C" fn USBPORT_DbgPrint_v(
    mut MiniPortExtension: PVOID,
    mut Level: ULONG,
    mut Format: PCH,
    _ap: *mut core::ffi::c_void,
) -> ULONG {
    return 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_TestDebugBreak(MiniPortExtension: PVOID) -> ULONG {
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AssertFailure(
    mut MiniPortExtension: PVOID,
    mut FailedAssertion: PVOID,
    mut FileName: PVOID,
    mut LineNumber: ULONG,
    mut Message: PCHAR,
) -> ULONG {
    RtlAssert(FailedAssertion, FileName, LineNumber, Message as PCH);
    return 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_BugCheck(MiniPortExtension: PVOID) {
    if DbgPrint(
        b"(%s:%d) USBPORT_BugCheck: FIXME \n\0".as_ptr() as PCH,
        b"drivers/usb/usbport/debug.c\0".as_ptr() as *const i8,
        53 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCH,
            b"drivers/usb/usbport/debug.c\0".as_ptr() as *const i8,
            53 as i32,
        );
    }
    if 0 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbport/debug.c\0".as_ptr() as *const i8 as PVOID,
            55 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_LogEntry(
    mut MiniPortExtension: PVOID,
    mut DriverTag: ULONG,
    mut EnumTag: ULONG,
    mut P1: ULONG,
    mut P2: ULONG,
    mut P3: ULONG,
) -> ULONG {
    DPRINT_MINIPORT!(
        b"USBPORT_LogEntry: MiniPortExtension - %p, EnumTag - %lx, P1 - %lx, P2 - %lx, P3 - %lx\n\0".as_ptr() as *const i8,
        MiniPortExtension,
        EnumTag,
        P1,
        P2,
        P3,
    );

    return 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DumpingDeviceDescriptor(
    DeviceDescriptor: PUSB_DEVICE_DESCRIPTOR,
) {
    if DeviceDescriptor.is_null() {
        return;
    }

    DPRINT_URB!(
        b"Dumping Device Descriptor - %p\n\0".as_ptr() as *const i8,
        DeviceDescriptor,
    );
    DPRINT_URB!(
        b"bLength             - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).bLength as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"bDescriptorType     - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).bDescriptorType as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"bcdUSB              - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).bcdUSB as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"bDeviceClass        - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).bDeviceClass as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"bDeviceSubClass     - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).bDeviceSubClass as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"bDeviceProtocol     - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).bDeviceProtocol as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"bMaxPacketSize0     - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).bMaxPacketSize0 as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"idVendor            - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).idVendor as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"idProduct           - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).idProduct as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"bcdDevice           - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).bcdDevice as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"iManufacturer       - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).iManufacturer as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"iProduct            - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).iProduct as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"iSerialNumber       - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).iSerialNumber as ::core::ffi::c_int),
    );
    DPRINT_URB!(
        b"bNumConfigurations  - %x\n\0".as_ptr() as *const i8,
        ((*DeviceDescriptor).bNumConfigurations as ::core::ffi::c_int),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DumpingConfiguration(
    ConfigDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
) {
    let mut iDescriptor: PUSB_INTERFACE_DESCRIPTOR =
        core::ptr::null_mut();
    let mut Descriptor: PUSB_ENDPOINT_DESCRIPTOR =
        core::ptr::null_mut();
    let mut ix: ULONG;

    if ConfigDescriptor.is_null()
        || ((*ConfigDescriptor).bLength as usize)
            < core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>()
    {
        return;
    }

    DPRINT_URB!(b"Dumping ConfigDescriptor - %p\n\0".as_ptr() as *const i8, ConfigDescriptor);
    DPRINT_URB!(b"bLength             - %x\n\0".as_ptr() as *const i8, ((*ConfigDescriptor).bLength as ::core::ffi::c_int));
    DPRINT_URB!(b"bDescriptorType     - %x\n\0".as_ptr() as *const i8, ((*ConfigDescriptor).bDescriptorType as ::core::ffi::c_int));
    DPRINT_URB!(b"wTotalLength        - %x\n\0".as_ptr() as *const i8, ((*ConfigDescriptor).wTotalLength as ::core::ffi::c_int));
    DPRINT_URB!(b"bNumInterfaces      - %x\n\0".as_ptr() as *const i8, ((*ConfigDescriptor).bNumInterfaces as ::core::ffi::c_int));
    DPRINT_URB!(b"bConfigurationValue - %x\n\0".as_ptr() as *const i8, ((*ConfigDescriptor).bConfigurationValue as ::core::ffi::c_int));
    DPRINT_URB!(b"iConfiguration      - %x\n\0".as_ptr() as *const i8, ((*ConfigDescriptor).iConfiguration as ::core::ffi::c_int));
    DPRINT_URB!(b"bmAttributes        - %x\n\0".as_ptr() as *const i8, ((*ConfigDescriptor).bmAttributes as ::core::ffi::c_int));
    DPRINT_URB!(b"MaxPower            - %x\n\0".as_ptr() as *const i8, ((*ConfigDescriptor).MaxPower as ::core::ffi::c_int));

    iDescriptor = (ConfigDescriptor as ULONG_PTR)
        .wrapping_add((*ConfigDescriptor).bLength as ULONG_PTR)
        as PUSB_INTERFACE_DESCRIPTOR;

    if ((*iDescriptor).bLength as usize)
        < core::mem::size_of::<USB_INTERFACE_DESCRIPTOR>()
    {
        return;
    }

    DPRINT_URB!(b"Dumping iDescriptor - %p\n\0".as_ptr() as *const i8, iDescriptor);
    DPRINT_URB!(b"bLength             - %x\n\0".as_ptr() as *const i8, ((*iDescriptor).bLength as ::core::ffi::c_int));
    DPRINT_URB!(b"bDescriptorType     - %x\n\0".as_ptr() as *const i8, ((*iDescriptor).bDescriptorType as ::core::ffi::c_int));
    DPRINT_URB!(b"bInterfaceNumber    - %x\n\0".as_ptr() as *const i8, ((*iDescriptor).bInterfaceNumber as ::core::ffi::c_int));
    DPRINT_URB!(b"bAlternateSetting   - %x\n\0".as_ptr() as *const i8, ((*iDescriptor).bAlternateSetting as ::core::ffi::c_int));
    DPRINT_URB!(b"bNumEndpoints       - %x\n\0".as_ptr() as *const i8, ((*iDescriptor).bNumEndpoints as ::core::ffi::c_int));
    DPRINT_URB!(b"bInterfaceClass     - %x\n\0".as_ptr() as *const i8, ((*iDescriptor).bInterfaceClass as ::core::ffi::c_int));
    DPRINT_URB!(b"bInterfaceSubClass  - %x\n\0".as_ptr() as *const i8, ((*iDescriptor).bInterfaceSubClass as ::core::ffi::c_int));
    DPRINT_URB!(b"bInterfaceProtocol  - %x\n\0".as_ptr() as *const i8, ((*iDescriptor).bInterfaceProtocol as ::core::ffi::c_int));
    DPRINT_URB!(b"iInterface          - %x\n\0".as_ptr() as *const i8, ((*iDescriptor).iInterface as ::core::ffi::c_int));

    Descriptor = (iDescriptor as ULONG_PTR)
        .wrapping_add((*iDescriptor).bLength as ULONG_PTR)
        as PUSB_ENDPOINT_DESCRIPTOR;

    ix = 0;
    while ix < (*iDescriptor).bNumEndpoints as ULONG {
        if ((*Descriptor).bLength as usize)
            < core::mem::size_of::<USB_ENDPOINT_DESCRIPTOR>()
        {
            return;
        }

        DPRINT_URB!(b"Dumping Descriptor  - %p\n\0".as_ptr() as *const i8, Descriptor);
        DPRINT_URB!(b"bLength             - %x\n\0".as_ptr() as *const i8, ((*Descriptor).bLength as ::core::ffi::c_int));
        DPRINT_URB!(b"bDescriptorType     - %x\n\0".as_ptr() as *const i8, ((*Descriptor).bDescriptorType as ::core::ffi::c_int));
        DPRINT_URB!(b"bEndpointAddress    - %x\n\0".as_ptr() as *const i8, ((*Descriptor).bEndpointAddress as ::core::ffi::c_int));
        DPRINT_URB!(b"bmAttributes        - %x\n\0".as_ptr() as *const i8, ((*Descriptor).bmAttributes as ::core::ffi::c_int));
        DPRINT_URB!(b"wMaxPacketSize      - %x\n\0".as_ptr() as *const i8, ((*Descriptor).wMaxPacketSize as ::core::ffi::c_int));
        DPRINT_URB!(b"bInterval           - %x\n\0".as_ptr() as *const i8, ((*Descriptor).bInterval as ::core::ffi::c_int));

        Descriptor = Descriptor.offset(1);
        ix = ix.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DumpingCapabilities(Capabilities: PDEVICE_CAPABILITIES) {
    if Capabilities.is_null() {
        return;
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DumpingSetupPacket(
    mut SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET,
) {
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DumpingURB(mut Urb: PURB) {
    let mut SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET;

    DPRINT_URB!(b"UrbHeader.Length           - %x\n\0".as_ptr() as *const i8, ((*Urb).__bindgen_anon_1.UrbHeader.Length as ::core::ffi::c_int));
    DPRINT_URB!(b"UrbHeader.Function         - %x\n\0".as_ptr() as *const i8, ((*Urb).__bindgen_anon_1.UrbHeader.Function as ::core::ffi::c_int));
    DPRINT_URB!(b"UrbHeader.__bindgen_anon_1.Status           - %x\n\0".as_ptr() as *const i8, (*Urb).__bindgen_anon_1.UrbHeader.Status);
    DPRINT_URB!(b"UrbHeader.UsbdDeviceHandle - %p\n\0".as_ptr() as *const i8, (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle);
    DPRINT_URB!(b"UrbHeader.UsbdFlags        - %x\n\0".as_ptr() as *const i8, (*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags);

    if ((*Urb).__bindgen_anon_1.UrbHeader.Length as i32) < 0x48 {
        return;
    }

    DPRINT_URB!(b"PipeHandle                - %p\n\0".as_ptr() as *const i8, (*Urb).__bindgen_anon_1.UrbControlTransfer.PipeHandle);
    DPRINT_URB!(b"TransferFlags             - %x\n\0".as_ptr() as *const i8, (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags);
    DPRINT_URB!(b"TransferBufferLength      - %x\n\0".as_ptr() as *const i8, (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferLength);
    DPRINT_URB!(b"TransferBuffer            - %p\n\0".as_ptr() as *const i8, (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBuffer);
    DPRINT_URB!(b"TransferBufferMDL         - %p\n\0".as_ptr() as *const i8, (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferMDL);
    DPRINT_URB!(b"UrbLink                   - %p\n\0".as_ptr() as *const i8, (*Urb).__bindgen_anon_1.UrbControlTransfer.UrbLink);

    if ((*Urb).__bindgen_anon_1.UrbHeader.Length as i32) < 0x50 {
        return;
    }

    SetupPacket = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbControlTransfer.SetupPacket) as PUSB_DEFAULT_PIPE_SETUP_PACKET;
    USBPORT_DumpingSetupPacket(SetupPacket);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DumpingIDs(Buffer: PVOID) {
    let mut Ptr: PWSTR;
    let mut Length: ULONG;
    let mut TotalLength: ULONG = 0;

    Ptr = Buffer as PWSTR;

    while *Ptr != 0 {
        Length = (wcslen(Ptr as *const wchar_t) as ULONG).wrapping_add(1);

        Ptr = Ptr.offset(Length as isize);
        TotalLength = TotalLength.wrapping_add(Length);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DumpingEndpointProperties(
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
) {
    if DbgPrint(
        b"(%s:%d) DeviceAddress         - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        282 as ::core::ffi::c_int,
        (*EndpointProperties).DeviceAddress as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            282 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) EndpointAddress       - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        283 as ::core::ffi::c_int,
        (*EndpointProperties).EndpointAddress as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            283 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) TotalMaxPacketSize    - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        284 as ::core::ffi::c_int,
        (*EndpointProperties).TotalMaxPacketSize as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            284 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) Period                - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        285 as ::core::ffi::c_int,
        (*EndpointProperties).Period as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            285 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) DeviceSpeed           - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        286 as ::core::ffi::c_int,
        (*EndpointProperties).DeviceSpeed as ::core::ffi::c_uint,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            286 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) UsbBandwidth          - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        287 as ::core::ffi::c_int,
        (*EndpointProperties).UsbBandwidth,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            287 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) ScheduleOffset        - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        288 as ::core::ffi::c_int,
        (*EndpointProperties).ScheduleOffset,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            288 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) TransferType          - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        289 as ::core::ffi::c_int,
        (*EndpointProperties).TransferType,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            289 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) MaxTransferSize       - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        290 as ::core::ffi::c_int,
        (*EndpointProperties).MaxTransferSize,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            290 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) HubAddr               - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        291 as ::core::ffi::c_int,
        (*EndpointProperties).HubAddr as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            291 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) PortNumber            - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        292 as ::core::ffi::c_int,
        (*EndpointProperties).PortNumber as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            292 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) InterruptScheduleMask - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        293 as ::core::ffi::c_int,
        (*EndpointProperties).InterruptScheduleMask as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            293 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) SplitCompletionMask   - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        294 as ::core::ffi::c_int,
        (*EndpointProperties).SplitCompletionMask as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            294 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) MaxPacketSize         - %X\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        295 as ::core::ffi::c_int,
        (*EndpointProperties).MaxPacketSize,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            295 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DumpingTtEndpoint(mut TtEndpoint: PUSB2_TT_ENDPOINT) {
    if DbgPrint(
        b"(%s:%d) MaxPacketSize   - %X\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        302 as ::core::ffi::c_int,
        (*TtEndpoint).MaxPacketSize as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            302 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) Period          - %X\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        303 as ::core::ffi::c_int,
        (*TtEndpoint).Period as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            303 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) TtEndpointParams- %X\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        304 as ::core::ffi::c_int,
        (*TtEndpoint).TtEndpointParams.AsULONG,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            304 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) CalcBusTime     - %X\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        305 as ::core::ffi::c_int,
        (*TtEndpoint).CalcBusTime as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            305 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) StartTime       - %X\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        306 as ::core::ffi::c_int,
        (*TtEndpoint).StartTime as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            306 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) ActualPeriod    - %X\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        307 as ::core::ffi::c_int,
        (*TtEndpoint).ActualPeriod as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            307 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) StartFrame      - %X\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        308 as ::core::ffi::c_int,
        (*TtEndpoint).StartFrame as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            308 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) StartMicroframe - %X\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        309 as ::core::ffi::c_int,
        (*TtEndpoint).StartMicroframe as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            309 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) Nums            - %X\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        310 as ::core::ffi::c_int,
        (*TtEndpoint).Nums.AsULONG,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            310 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) nextTtEndpoint  - %X\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
        311 as ::core::ffi::c_int,
        (*TtEndpoint).NextTtEndpoint,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const ::core::ffi::c_char,
            311 as ::core::ffi::c_int,
        );
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort debugging functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_USBPORT_MINIPORT
// #define NDEBUG_USBPORT_URB
// //#define NDEBUG_USBPORT_USB2
// #include "usbdebug.h"
//
// ULONG
// USBPORT_DbgPrint(IN PVOID MiniPortExtension,
//                  IN ULONG Level,
//                  IN PCH Format,
//                  ...)
// {
//     DPRINT("USBPORT_DbgPrint: UNIMPLEMENTED. FIXME. \n");
//     return 0;
// }
//
// ULONG
// NTAPI
// USBPORT_TestDebugBreak(IN PVOID MiniPortExtension)
// {
//     DPRINT("USBPORT_TestDebugBreak: UNIMPLEMENTED. FIXME. \n");
//     return 0;
// }
//
// ULONG
// NTAPI
// USBPORT_AssertFailure(PVOID MiniPortExtension,
//                       PVOID FailedAssertion,
//                       PVOID FileName,
//                       ULONG LineNumber,
//                       PCHAR Message)
// {
//     DPRINT("USBPORT_AssertFailure: ... \n");
//     RtlAssert(FailedAssertion, FileName, LineNumber, Message);
//     return 0;
// }
//
// VOID
// NTAPI
// USBPORT_BugCheck(IN PVOID MiniPortExtension)
// {
//     DPRINT1("USBPORT_BugCheck: FIXME \n");
//     //KeBugCheckEx(BUGCODE_USB_DRIVER, ...);
//     ASSERT(FALSE);
// }
//
// ULONG
// NTAPI
// USBPORT_LogEntry(IN PVOID MiniPortExtension,
//                  IN ULONG DriverTag,
//                  IN ULONG EnumTag,
//                  IN ULONG P1,
//                  IN ULONG P2,
//                  IN ULONG P3)
// {
//     DPRINT_MINIPORT!("USBPORT_LogEntry: MiniPortExtension - %p, EnumTag - %lx, P1 - %lx, P2 - %lx, P3 - %lx\n",
//            MiniPortExtension,
//            EnumTag,
//            P1,
//            P2,
//            P3);
//
//     return 0;
// }
//
// VOID
// NTAPI
// USBPORT_DumpingDeviceDescriptor(IN PUSB_DEVICE_DESCRIPTOR DeviceDescriptor)
// {
//     if (!DeviceDescriptor)
//     {
//         return;
//     }
//
//     DPRINT_URB!("Dumping Device Descriptor - %p\n", DeviceDescriptor);
//     DPRINT_URB!("bLength             - %x\n", DeviceDescriptor->bLength);
//     DPRINT_URB!("bDescriptorType     - %x\n", DeviceDescriptor->bDescriptorType);
//     DPRINT_URB!("bcdUSB              - %x\n", DeviceDescriptor->bcdUSB);
//     DPRINT_URB!("bDeviceClass        - %x\n", DeviceDescriptor->bDeviceClass);
//     DPRINT_URB!("bDeviceSubClass     - %x\n", DeviceDescriptor->bDeviceSubClass);
//     DPRINT_URB!("bDeviceProtocol     - %x\n", DeviceDescriptor->bDeviceProtocol);
//     DPRINT_URB!("bMaxPacketSize0     - %x\n", DeviceDescriptor->bMaxPacketSize0);
//     DPRINT_URB!("idVendor            - %x\n", DeviceDescriptor->idVendor);
//     DPRINT_URB!("idProduct           - %x\n", DeviceDescriptor->idProduct);
//     DPRINT_URB!("bcdDevice           - %x\n", DeviceDescriptor->bcdDevice);
//     DPRINT_URB!("iManufacturer       - %x\n", DeviceDescriptor->iManufacturer);
//     DPRINT_URB!("iProduct            - %x\n", DeviceDescriptor->iProduct);
//     DPRINT_URB!("iSerialNumber       - %x\n", DeviceDescriptor->iSerialNumber);
//     DPRINT_URB!("bNumConfigurations  - %x\n", DeviceDescriptor->bNumConfigurations);
// }
//
// VOID
// NTAPI
// USBPORT_DumpingConfiguration(IN PUSB_CONFIGURATION_DESCRIPTOR ConfigDescriptor)
// {
//     PUSB_INTERFACE_DESCRIPTOR iDescriptor;
//     PUSB_ENDPOINT_DESCRIPTOR Descriptor;
//     ULONG ix;
//
//     if (!ConfigDescriptor ||
//         ConfigDescriptor->bLength < sizeof(USB_CONFIGURATION_DESCRIPTOR))
//     {
//         return;
//     }
//
//     DPRINT_URB!("Dumping ConfigDescriptor - %p\n", ConfigDescriptor);
//     DPRINT_URB!("bLength             - %x\n", ConfigDescriptor->bLength);
//     DPRINT_URB!("bDescriptorType     - %x\n", ConfigDescriptor->bDescriptorType);
//     DPRINT_URB!("wTotalLength        - %x\n", ConfigDescriptor->wTotalLength);
//     DPRINT_URB!("bNumInterfaces      - %x\n", ConfigDescriptor->bNumInterfaces);
//     DPRINT_URB!("bConfigurationValue - %x\n", ConfigDescriptor->bConfigurationValue);
//     DPRINT_URB!("iConfiguration      - %x\n", ConfigDescriptor->iConfiguration);
//     DPRINT_URB!("bmAttributes        - %x\n", ConfigDescriptor->bmAttributes);
//     DPRINT_URB!("MaxPower            - %x\n", ConfigDescriptor->MaxPower);
//
//     iDescriptor = (PUSB_INTERFACE_DESCRIPTOR)((ULONG_PTR)ConfigDescriptor +
//                                                          ConfigDescriptor->bLength);
//
//     if (iDescriptor->bLength < sizeof(USB_INTERFACE_DESCRIPTOR))
//     {
//         return;
//     }
//
//     DPRINT_URB!("Dumping iDescriptor - %p\n", iDescriptor);
//     DPRINT_URB!("bLength             - %x\n", iDescriptor->bLength);
//     DPRINT_URB!("bDescriptorType     - %x\n", iDescriptor->bDescriptorType);
//     DPRINT_URB!("bInterfaceNumber    - %x\n", iDescriptor->bInterfaceNumber);
//     DPRINT_URB!("bAlternateSetting   - %x\n", iDescriptor->bAlternateSetting);
//     DPRINT_URB!("bNumEndpoints       - %x\n", iDescriptor->bNumEndpoints);
//     DPRINT_URB!("bInterfaceClass     - %x\n", iDescriptor->bInterfaceClass);
//     DPRINT_URB!("bInterfaceSubClass  - %x\n", iDescriptor->bInterfaceSubClass);
//     DPRINT_URB!("bInterfaceProtocol  - %x\n", iDescriptor->bInterfaceProtocol);
//     DPRINT_URB!("iInterface          - %x\n", iDescriptor->iInterface);
//
//     Descriptor = (PUSB_ENDPOINT_DESCRIPTOR)((ULONG_PTR)iDescriptor +
//                                                        iDescriptor->bLength);
//
//     for (ix = 0; ix < iDescriptor->bNumEndpoints; ix++)
//     {
//         if (Descriptor->bLength < sizeof(USB_ENDPOINT_DESCRIPTOR))
//         {
//             return;
//         }
//
//         DPRINT_URB!("Dumping Descriptor  - %p\n", Descriptor);
//         DPRINT_URB!("bLength             - %x\n", Descriptor->bLength);
//         DPRINT_URB!("bDescriptorType     - %x\n", Descriptor->bDescriptorType);
//         DPRINT_URB!("bEndpointAddress    - %x\n", Descriptor->bEndpointAddress);
//         DPRINT_URB!("bmAttributes        - %x\n", Descriptor->bmAttributes);
//         DPRINT_URB!("wMaxPacketSize      - %x\n", Descriptor->wMaxPacketSize);
//         DPRINT_URB!("bInterval           - %x\n", Descriptor->bInterval);
//
//         Descriptor += 1;
//     }
// }
//
// VOID
// NTAPI
// USBPORT_DumpingCapabilities(IN PDEVICE_CAPABILITIES Capabilities)
// {
//     if (!Capabilities)
//     {
//         return;
//     }
//
//     DPRINT("Capabilities->Size              - %x\n", Capabilities->Size);
//     DPRINT("Capabilities->Version           - %x\n", Capabilities->Version);
//
//     DPRINT("Capabilities->DeviceD1          - %x\n", Capabilities->DeviceD1);
//     DPRINT("Capabilities->DeviceD2          - %x\n", Capabilities->DeviceD2);
//     DPRINT("Capabilities->LockSupported     - %x\n", Capabilities->LockSupported);
//     DPRINT("Capabilities->EjectSupported    - %x\n", Capabilities->EjectSupported);
//     DPRINT("Capabilities->Removable         - %x\n", Capabilities->Removable);
//     DPRINT("Capabilities->DockDevice        - %x\n", Capabilities->DockDevice);
//     DPRINT("Capabilities->UniqueID          - %x\n", Capabilities->UniqueID);
//     DPRINT("Capabilities->SilentInstall     - %x\n", Capabilities->SilentInstall);
//     DPRINT("Capabilities->RawDeviceOK       - %x\n", Capabilities->RawDeviceOK);
//     DPRINT("Capabilities->SurpriseRemovalOK - %x\n", Capabilities->SurpriseRemovalOK);
//
//     DPRINT("Capabilities->Address           - %x\n", Capabilities->Address);
//     DPRINT("Capabilities->UINumber          - %x\n", Capabilities->UINumber);
//
//     DPRINT("Capabilities->DeviceState[0]    - %x\n", Capabilities->DeviceState[0]);
//     DPRINT("Capabilities->DeviceState[1]    - %x\n", Capabilities->DeviceState[1]);
//     DPRINT("Capabilities->DeviceState[2]    - %x\n", Capabilities->DeviceState[2]);
//     DPRINT("Capabilities->DeviceState[3]    - %x\n", Capabilities->DeviceState[3]);
//     DPRINT("Capabilities->DeviceState[4]    - %x\n", Capabilities->DeviceState[4]);
//     DPRINT("Capabilities->DeviceState[5]    - %x\n", Capabilities->DeviceState[5]);
//     DPRINT("Capabilities->DeviceState[6]    - %x\n", Capabilities->DeviceState[6]);
//
//     DPRINT("Capabilities->SystemWake        - %x\n", Capabilities->SystemWake);
//     DPRINT("Capabilities->DeviceWake        - %x\n", Capabilities->DeviceWake);
//
//     DPRINT("Capabilities->D1Latency         - %x\n", Capabilities->D1Latency);
//     DPRINT("Capabilities->D2Latency         - %x\n", Capabilities->D2Latency);
//     DPRINT("Capabilities->D3Latency         - %x\n", Capabilities->D3Latency);
// }
//
// VOID
// NTAPI
// USBPORT_DumpingSetupPacket(IN PUSB_DEFAULT_PIPE_SETUP_PACKET SetupPacket)
// {
//     DPRINT("SetupPacket->bmRequestType.B - %x\n", SetupPacket->bmRequestType.B);
//     DPRINT("SetupPacket->bRequest        - %x\n", SetupPacket->bRequest);
//     DPRINT("SetupPacket->wValue.LowByte  - %x\n", SetupPacket->wValue.LowByte);
//     DPRINT("SetupPacket->wValue.HiByte   - %x\n", SetupPacket->wValue.HiByte);
//     DPRINT("SetupPacket->wIndex.W        - %x\n", SetupPacket->wIndex.W);
//     DPRINT("SetupPacket->wLength         - %x\n", SetupPacket->wLength);
// }
//
// VOID
// NTAPI
// USBPORT_DumpingURB(IN PURB Urb)
// {
//     PUSB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//
//     DPRINT_URB!("UrbHeader.Length           - %x\n", Urb->UrbHeader.Length);
//     DPRINT_URB!("UrbHeader.Function         - %x\n", Urb->UrbHeader.Function);
//     DPRINT_URB!("UrbHeader.Status           - %x\n", Urb->UrbHeader.Status);
//     DPRINT_URB!("UrbHeader.UsbdDeviceHandle - %p\n", Urb->UrbHeader.UsbdDeviceHandle);
//     DPRINT_URB!("UrbHeader.UsbdFlags        - %x\n", Urb->UrbHeader.UsbdFlags);
//
//     if (Urb->UrbHeader.Length < 0x48)
//     {
//         return;
//     }
//
//     DPRINT_URB!("PipeHandle                - %p\n", Urb->UrbControlTransfer.PipeHandle);
//     DPRINT_URB!("TransferFlags             - %x\n", Urb->UrbControlTransfer.TransferFlags);
//     DPRINT_URB!("TransferBufferLength      - %x\n", Urb->UrbControlTransfer.TransferBufferLength);
//     DPRINT_URB!("TransferBuffer            - %p\n", Urb->UrbControlTransfer.TransferBuffer);
//     DPRINT_URB!("TransferBufferMDL         - %p\n", Urb->UrbControlTransfer.TransferBufferMDL);
//     DPRINT_URB!("UrbLink                   - %p\n", Urb->UrbControlTransfer.__bindgen_anon_1.UrbLink);
//
//     if (Urb->UrbHeader.Length < 0x50)
//     {
//         return;
//     }
//
//     SetupPacket = (PUSB_DEFAULT_PIPE_SETUP_PACKET)&Urb->UrbControlTransfer.SetupPacket;
//     USBPORT_DumpingSetupPacket(SetupPacket);
// }
//
// VOID
// NTAPI
// USBPORT_DumpingIDs(IN PVOID Buffer)
// {
//     PWSTR Ptr;
//     ULONG Length;
//     ULONG TotalLength = 0;
//
//     Ptr = (PWSTR)Buffer;
//
//     while (*Ptr)
//     {
//         DPRINT("  %S\n", Ptr);
//         Length = (ULONG)wcslen(Ptr) + 1;
//
//         Ptr += Length;
//         TotalLength += Length;
//     }
//
//     DPRINT("TotalLength: %hu\n", TotalLength);
//     DPRINT("\n");
// }
//
// VOID
// NTAPI
// USBPORT_DumpingEndpointProperties(IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties)
// {
//     DPRINT_USB2("DeviceAddress         - %X\n", EndpointProperties->DeviceAddress);
//     DPRINT_USB2("EndpointAddress       - %X\n", EndpointProperties->EndpointAddress);
//     DPRINT_USB2("TotalMaxPacketSize    - %X\n", EndpointProperties->TotalMaxPacketSize);
//     DPRINT_USB2("Period                - %X\n", EndpointProperties->Period);
//     DPRINT_USB2("DeviceSpeed           - %X\n", EndpointProperties->DeviceSpeed);
//     DPRINT_USB2("UsbBandwidth          - %X\n", EndpointProperties->UsbBandwidth);
//     DPRINT_USB2("ScheduleOffset        - %X\n", EndpointProperties->ScheduleOffset);
//     DPRINT_USB2("TransferType          - %X\n", EndpointProperties->TransferType);
//     DPRINT_USB2("MaxTransferSize       - %X\n", EndpointProperties->MaxTransferSize);
//     DPRINT_USB2("HubAddr               - %X\n", EndpointProperties->HubAddr);
//     DPRINT_USB2("PortNumber            - %X\n", EndpointProperties->PortNumber);
//     DPRINT_USB2("InterruptScheduleMask - %X\n", EndpointProperties->InterruptScheduleMask);
//     DPRINT_USB2("SplitCompletionMask   - %X\n", EndpointProperties->SplitCompletionMask);
//     DPRINT_USB2("MaxPacketSize         - %X\n", EndpointProperties->MaxPacketSize);
// }
//
// VOID
// NTAPI
// USBPORT_DumpingTtEndpoint(IN PUSB2_TT_ENDPOINT TtEndpoint)
// {
//     DPRINT_USB2("MaxPacketSize   - %X\n", TtEndpoint->MaxPacketSize);
//     DPRINT_USB2("Period          - %X\n", TtEndpoint->Period);
//     DPRINT_USB2("TtEndpointParams- %X\n", TtEndpoint->TtEndpointParams.AsULONG);
//     DPRINT_USB2("CalcBusTime     - %X\n", TtEndpoint->CalcBusTime);
//     DPRINT_USB2("StartTime       - %X\n", TtEndpoint->StartTime);
//     DPRINT_USB2("ActualPeriod    - %X\n", TtEndpoint->ActualPeriod);
//     DPRINT_USB2("StartFrame      - %X\n", TtEndpoint->StartFrame);
//     DPRINT_USB2("StartMicroframe - %X\n", TtEndpoint->StartMicroframe);
//     DPRINT_USB2("Nums            - %X\n", TtEndpoint->Nums.AsULONG);
//     DPRINT_USB2("nextTtEndpoint  - %X\n", TtEndpoint->NextTtEndpoint);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
