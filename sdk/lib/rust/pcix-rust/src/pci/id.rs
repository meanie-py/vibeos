#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::pci::config::RtlAssert;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ANSI_STRING, MESSAGE_RESOURCE_ENTRY, PANSI_STRING, PCCH, PMESSAGE_RESOURCE_ENTRY, RtlAnsiStringToUnicodeString, RtlFindMessage, RtlInitAnsiString, va_list};
use ntkrnl_sys::ddk::{_MESSAGE_RESOURCE_ENTRY, _UNICODE_STRING, PCSZ, PWCH};
use ntkrnl_sys::ddk::_BUS_QUERY_ID_TYPE as BUS_QUERY_ID_TYPE;
use ntkrnl_sys::ddk::_DEVICE_TEXT_TYPE as DEVICE_TEXT_TYPE;
use ntkrnl_sys::ddk::{
    _BUS_QUERY_ID_TYPE_BusQueryCompatibleIDs as BusQueryCompatibleIDs,
    _BUS_QUERY_ID_TYPE_BusQueryDeviceID as BusQueryDeviceID,
    _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs as BusQueryHardwareIDs,
    _BUS_QUERY_ID_TYPE_BusQueryInstanceID as BusQueryInstanceID,
    _DEVICE_TEXT_TYPE_DeviceTextDescription as DeviceTextDescription,
    _DEVICE_TEXT_TYPE_DeviceTextLocationInformation as DeviceTextLocationInformation,
};
use ntkrnl_sys::ke::KeGetCurrentIrql;
use ntkrnl_sys::driver_ddk::DbgPrint;
use core::ffi::c_void;

type PCSTR = *const i8;
#[allow(non_camel_case_types)]
type c_char = i8;

extern "system" {
}
extern "C" {
    fn sprintf(buffer: *mut ::core::ffi::c_char, format: *const ::core::ffi::c_char, ...) -> i32;
    fn swprintf(buffer: *mut WCHAR, format: *const WCHAR, ...) -> i32;
}

// ---- the C-variadic boundary (drivers/bus/pcix/rust_varargs_shim.c) -----------------
//
// Stable Rust cannot DEFINE a C-variadic fn (`va_start`) nor `va_arg` — that is the
// nightly `c_variadic` feature (rust-lang#44930). It CAN declare and CALL one. So the
// variadic ENTRY of PciIdPrintf / PciIdPrintfAppend lives in the shim, which immediately
// calls the fixed-arity `_v` twin below holding ALL of id.c's logic. The call sites in
// PciQueryId are unchanged — they still call `PciIdPrintf(...)`, now the shim's trampoline.
extern "C" {
    fn PciIdPrintf(IdBuffer: PPCI_ID_BUFFER, Format: PCCH, ...) -> ULONG;
    fn PciIdPrintfAppend(IdBuffer: PPCI_ID_BUFFER, Format: PCCH, ...) -> ULONG;

    // Neither function pulls args itself: both hand the whole va_list to ntoskrnl's
    // exported `vsprintf`. Rust only ever holds an opaque POINTER to the C frame's
    // va_list, so it cannot pass one BY VALUE — this shim forwarder dereferences the
    // pointer and makes that call. It does nothing else; the destination pointer, the
    // length accounting and every ANSI_STRING field update stay in the `_v` twins.
    fn vibeos_vsprintf(
        buffer: *mut ::core::ffi::c_char,
        format: *const ::core::ffi::c_char,
        ap: *mut c_void,
    ) -> i32;
}
// RtlAssert / RtlxAnsiStringToUnicodeSize are NTAPI (stdcall on i386, _Name@N).
extern "system" {
    fn RtlxAnsiStringToUnicodeSize(AnsiString: *const ANSI_STRING) -> ULONG;
}
// NlsMbCodePageTag is an ntoskrnl data export (cross-DLL data import → raw-dylib).
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    // BOOLEAN (u8) in sdk/lib/rtl/nls.c — an i32 decl over-reads 3 neighbor bytes
    static mut NlsMbCodePageTag: u8;
}
#[no_mangle]
pub unsafe extern "system" fn PciGetDescriptionMessage(
    Identifier: ULONG,
    Length: PULONG,
) -> PWCHAR {
    let mut Entry: PMESSAGE_RESOURCE_ENTRY = core::ptr::null_mut::<_MESSAGE_RESOURCE_ENTRY>();
    let mut TextLength: ULONG;
    let mut Description: PWCHAR;
    let mut Buffer: PWCHAR;
    let mut MessageString: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut UnicodeString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let Status: NTSTATUS;

    /* Find the message identifier in the message table */
    MessageString.Buffer = core::ptr::null_mut();
    Status = RtlFindMessage(
        (*PciDriverObject).DriverStart,
        11u32, // RT_MESSAGETABLE
        ((0u16 as u32) << 10 | (0u16 as u32)) as ULONG,
        Identifier,
        core::ptr::addr_of_mut!(Entry),
    );
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut::<u16>();
    }

    /* Check if the resource data is Unicode or ANSI */
    if (*Entry).Flags as u32 & MESSAGE_RESOURCE_UNICODE != 0 {
        /* Subtract one space for the end-of-message terminator */
        TextLength = ((*Entry).Length as ULONG)
            .wrapping_sub(core::mem::offset_of!(MESSAGE_RESOURCE_ENTRY, Text) as ULONG)
            .wrapping_sub(core::mem::size_of::<WCHAR>() as ULONG);

        /* Grab the text */
        Description = core::ptr::addr_of_mut!((*Entry).Text) as PWCHAR;

        // Divergence from the C: entries are padded to a 4-byte boundary, so an entry can
        // end in two NULs, and the C's "one terminator" arithmetic then lands on padding
        // (the ASSERT below fired for "IDE Controller" and the description kept its line
        // break). Drop the padding before looking for the newline.
        while TextLength as usize > core::mem::size_of::<WCHAR>()
            && *Description.add(TextLength as usize / core::mem::size_of::<WCHAR>() - 1) == 0
        {
            TextLength -= core::mem::size_of::<WCHAR>() as ULONG;
        }

        /* Validate the message length, ending with a newline character */
        if !(TextLength as usize > core::mem::size_of::<WCHAR>()) {
            RtlAssert(
                b"TextLength > sizeof(WCHAR)\0" as *const u8 as PVOID,
                b"drivers/bus/pcix/pci/id.c\0" as *const u8 as PVOID,
                52u32,
                (core::ptr::null_mut::<i8>()).cast(),
            );
        }
        if !(*(Description.add(
            (TextLength as usize / core::mem::size_of::<WCHAR>()).wrapping_sub(1),
        )) as i32 == '\n' as i32)
        {
            RtlAssert(
                b"Description[(TextLength / sizeof(WCHAR)) - 1] == L'\\n'\0" as *const u8 as PVOID,
                b"drivers/bus/pcix/pci/id.c\0" as *const u8 as PVOID,
                53u32,
                (core::ptr::null_mut::<i8>()).cast(),
            );
        }

        /* Allocate the buffer to hold the message string */
        Buffer = ExAllocatePoolWithTag(
            PagedPool,
            TextLength as SIZE_T,
            0x50636942u32, // (0x42696350u32) (little-endian)
        ) as PWCHAR;
        if Buffer.is_null() {
            return core::ptr::null_mut::<u16>();
        }

        /* Copy the message, minus the newline character, and terminate it */
        core::ptr::copy_nonoverlapping(
            Description as *const u16 as *const u8,
            Buffer as *mut u16 as *mut u8,
            (TextLength as usize).wrapping_sub(core::mem::size_of::<WCHAR>()),
        );
        *Buffer.add(
            (TextLength as usize / core::mem::size_of::<WCHAR>()).wrapping_sub(1),
        ) = 0u16; // UNICODE_NULL
        // The .mc source has CRLF line ends, so the line break is "\r\n"; the C strips
        // only the '\n' and the description kept a trailing carriage return.
        if TextLength as usize >= 2 * core::mem::size_of::<WCHAR>()
            && *Buffer.add(TextLength as usize / core::mem::size_of::<WCHAR>() - 2) == b'\r' as u16
        {
            *Buffer.add(TextLength as usize / core::mem::size_of::<WCHAR>() - 2) = 0u16;
            TextLength -= core::mem::size_of::<WCHAR>() as ULONG;
        }

        /* Return the length to the caller, minus the terminating NULL */
        if !Length.is_null() {
            *Length =
                TextLength.wrapping_sub(core::mem::size_of::<WCHAR>() as ULONG);
        }
    } else {
        /* Initialize the entry as a string */
        RtlInitAnsiString(
            &mut MessageString,
            core::ptr::addr_of_mut!((*Entry).Text) as *mut u8 as PCSZ,
        );

        /* Remove the newline character */
        MessageString.Length = (MessageString.Length as u64)
            .wrapping_sub(core::mem::size_of::<u8>() as u64) as u16;

        /* Convert it to Unicode */
        RtlAnsiStringToUnicodeString(
            &mut UnicodeString,
            &mut MessageString,
            1u8, // TRUE
        );
        Buffer = UnicodeString.Buffer;

        /* Return the length to the caller */
        if !Length.is_null() {
            *Length = UnicodeString.Length as ULONG;
        }
    }

    /* Return the message buffer to the caller */
    Buffer
}
#[no_mangle]
pub unsafe extern "system" fn PciGetDeviceDescriptionMessage(
    BaseClass: UCHAR,
    SubClass: UCHAR,
) -> PWCHAR {
    let Identifier: ULONG = ((BaseClass as u32) << 8) | (SubClass as u32);
    let mut Message: PWCHAR = PciGetDescriptionMessage(Identifier, core::ptr::null_mut());

    if Message.is_null() {
        Message = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<[WCHAR; 11]>() as SIZE_T,
            0x50494362u32,
        ) as PWCHAR;

        if !Message.is_null() {
            const WIDE_PCI_DEVICE: [WCHAR; 11] = [
                0x0050, // P
                0x0043, // C
                0x0049, // I
                0x0020, // space
                0x0044, // D
                0x0065, // e
                0x0076, // v
                0x0069, // i
                0x0063, // c
                0x0065, // e
                0x0000, // null
            ];
            core::ptr::copy_nonoverlapping(
                WIDE_PCI_DEVICE.as_ptr() as *const u8,
                Message as *mut u8,
                core::mem::size_of::<[WCHAR; 11]>(),
            );
        }
    }

    Message
}

#[no_mangle]
pub unsafe extern "system" fn PciInitIdBuffer(IdBuffer: PPCI_ID_BUFFER) {
    (*IdBuffer).TotalLength = 0u32;
    (*IdBuffer).Count = 0u32;
    (*IdBuffer).CharBuffer = core::ptr::addr_of_mut!((*IdBuffer).BufferData) as PCHAR;
}
/// Fixed-arity twin of the `PciIdPrintf` C-variadic entry point in
/// `drivers/bus/pcix/rust_varargs_shim.c`. The C side exists only to do the `va_start`
/// that stable Rust cannot; `ap` is a pointer to its `va_list`. All of id.c's logic —
/// the RtlAssert, RtlInitAnsiString, StringSize/TotalLength/Count bookkeeping and the
/// CharBuffer bump — stays here.
#[no_mangle]
pub unsafe extern "C" fn PciIdPrintf_v(mut IdBuffer: PPCI_ID_BUFFER, Format: PCCH, ap: *mut c_void) -> ULONG {

    if !((*IdBuffer).Count < 0x8u32) {
        RtlAssert(
            b"IdBuffer->Count < MAX_ANSI_STRINGS\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/pci/id.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            130u32,
            (::core::ptr::null_mut::<::core::ffi::c_char>()).cast(),
        );
    }

    vibeos_vsprintf(
        (*IdBuffer).CharBuffer as *mut ::core::ffi::c_char,
        Format,
        ap,
    );

    let AnsiString: PANSI_STRING = (*IdBuffer).Strings.as_mut_ptr().add((*IdBuffer).Count as usize) as PANSI_STRING;
    RtlInitAnsiString(AnsiString, (*IdBuffer).CharBuffer as *const ::core::ffi::c_char);

    let Size: ULONG = if NlsMbCodePageTag as i32 != 0 {
        RtlxAnsiStringToUnicodeSize(AnsiString as *const ANSI_STRING)
    } else {
        ((*AnsiString).Length as usize)
            .wrapping_add(::core::mem::size_of::<::core::ffi::c_char>()) // ANSI_NULL
            .wrapping_mul(::core::mem::size_of::<u16>()) // WCHAR size
            as ULONG
    };

    (*IdBuffer).StringSize[(*IdBuffer).Count as usize] = Size;
    (*IdBuffer).TotalLength = ((*IdBuffer).TotalLength as usize).wrapping_add(Size as usize) as ULONG;

    let Length: ULONG = ((*AnsiString).Length as usize)
        .wrapping_add(::core::mem::size_of::<::core::ffi::c_char>()) as ULONG; // ANSI_NULL

    (*IdBuffer).CharBuffer = (*IdBuffer).CharBuffer.add(Length as usize);

    (*IdBuffer).Count = (*IdBuffer).Count.wrapping_add(1);

    Length
}
/// Fixed-arity twin of the `PciIdPrintfAppend` C-variadic entry point — see
/// `PciIdPrintf_v` above.
#[no_mangle]
pub unsafe extern "C" fn PciIdPrintfAppend_v(
    mut IdBuffer: PPCI_ID_BUFFER,
    mut Format: PCCH,
    ap: *mut c_void,
) -> ULONG {
    let mut NextId: ULONG;
    let mut Size: ULONG;
    let mut Length: ULONG;
    let mut MaxLength: ULONG;
    let mut AnsiString: PANSI_STRING;

    if (*IdBuffer).Count == 0 {
        RtlAssert(
            b"IdBuffer->Count\0".as_ptr() as *const i8 as PVOID,
            b"drivers/bus/pcix/pci/id.c\0".as_ptr()
                as *const i8 as PVOID,
            167,
            (core::ptr::null_mut::<i8>()).cast(),
        );
    }

    NextId = ((*IdBuffer).Count as u32).wrapping_sub(1) as ULONG;
    MaxLength = (IdBuffer.offset(1) as PCHAR).offset_from((*IdBuffer).CharBuffer) as i64 as ULONG;

    Length = vibeos_vsprintf(
        (*IdBuffer).CharBuffer.offset(-(1isize)),
        Format as *const i8,
        ap,
    ) as ULONG;

    if !(Length < MaxLength) {
        RtlAssert(
            b"Length < MaxLength\0".as_ptr() as *const i8 as PVOID,
            b"drivers/bus/pcix/pci/id.c\0".as_ptr()
                as *const i8 as PVOID,
            179,
            (core::ptr::null_mut::<i8>()).cast(),
        );
    }

    AnsiString = &mut (*IdBuffer).Strings[NextId as usize] as *mut ANSI_STRING as PANSI_STRING;

    (*AnsiString).Length = ((*AnsiString).Length as u32).wrapping_add(Length as u32) as u16;
    (*AnsiString).MaximumLength = ((*AnsiString).MaximumLength as u32).wrapping_add(Length as u32)
        as u16;

    Size = if NlsMbCodePageTag as i32 != 0 {
        RtlxAnsiStringToUnicodeSize(AnsiString as *const ANSI_STRING) as usize
    } else {
        ((*AnsiString).Length as usize)
            .wrapping_add(core::mem::size_of::<i8>())
            .wrapping_mul(core::mem::size_of::<u16>())
    } as ULONG;

    (*IdBuffer).StringSize[NextId as usize] = Size;
    (*IdBuffer).TotalLength = ((*IdBuffer).TotalLength as u32).wrapping_add(Size) as ULONG;
    (*IdBuffer).CharBuffer = (*IdBuffer).CharBuffer.offset(Length as isize);

    Size
}
#[no_mangle]
pub unsafe extern "system" fn PciQueryId(
    DeviceExtension: PPCI_PDO_EXTENSION,
    QueryType: BUS_QUERY_ID_TYPE,
    Buffer: *mut PWCHAR,
) -> NTSTATUS {
    let mut SubsysId: ULONG;
    let mut VendorString: [CHAR; 22] = [0; 22];
    let mut PdoExtension: PPCI_PDO_EXTENSION;
    let mut ParentExtension: PPCI_FDO_EXTENSION;
    let mut StringBuffer: PWCHAR;
    let mut i: ULONG;
    let mut Size: ULONG;
    let mut Status: NTSTATUS;
    let mut NextString: PANSI_STRING;
    let mut DestinationString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut IdBuffer: PCI_ID_BUFFER = PCI_ID_BUFFER {
        Count: 0,
        Strings: [
            ANSI_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut::<CHAR>(),
            },
            ANSI_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut::<CHAR>(),
            },
            ANSI_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut::<CHAR>(),
            },
            ANSI_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut::<CHAR>(),
            },
            ANSI_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut::<CHAR>(),
            },
            ANSI_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut::<CHAR>(),
            },
            ANSI_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut::<CHAR>(),
            },
            ANSI_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut::<CHAR>(),
            },
        ],
        StringSize: [0; 8],
        TotalLength: 0,
        CharBuffer: core::ptr::null_mut::<CHAR>(),
        BufferData: [0; 256],
    };

    // PAGED_CODE
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/bus/pcix/pci/id.c\0".as_ptr() as *const i8 as PVOID,
                214 as ULONG,
                (core::ptr::null_mut::<CHAR>()).cast(),
            );
        }
    }

    /* Assume failure */
    Status = STATUS_SUCCESS;
    *Buffer = core::ptr::null_mut::<WCHAR>();

    /* Start with the genric vendor string, which is the vendor ID + device ID */
    sprintf(
        &raw mut VendorString as *mut i8,
        b"PCI\\VEN_%04X&DEV_%04X\0".as_ptr() as *const i8,
        (*DeviceExtension).VendorId as i32,
        (*DeviceExtension).DeviceId as i32,
    );

    /* Initialize the PCI ID Buffer */
    PciInitIdBuffer(&raw mut IdBuffer);

    /* Build the subsystem ID as shown in PCI ID Strings */
    SubsysId = ((*DeviceExtension).SubsystemVendorId as i32 | ((*DeviceExtension).SubsystemId as i32) << 16i32) as ULONG;

    /* Check what the caller is requesting */
    match QueryType {
        BusQueryDeviceID => {
            PciIdPrintf(
                &raw mut IdBuffer,
                b"%s&SUBSYS_%08X&REV_%02X\0".as_ptr() as PCCH,
                &raw mut VendorString as *mut CHAR,
                SubsysId,
                (*DeviceExtension).RevisionId as i32,
            );
        }
        BusQueryHardwareIDs => {
            PciIdPrintf(
                &raw mut IdBuffer,
                b"%s&SUBSYS_%08X&REV_%02X\0".as_ptr() as PCCH,
                &raw mut VendorString as *mut CHAR,
                SubsysId,
                (*DeviceExtension).RevisionId as i32,
            );
            PciIdPrintf(
                &raw mut IdBuffer,
                b"%s&SUBSYS_%08X\0".as_ptr() as PCCH,
                &raw mut VendorString as *mut CHAR,
                SubsysId,
            );
            PciIdPrintf(
                &raw mut IdBuffer,
                b"%s&CC_%02X%02X%02X\0".as_ptr() as PCCH,
                &raw mut VendorString as *mut CHAR,
                (*DeviceExtension).BaseClass as i32,
                (*DeviceExtension).SubClass as i32,
                (*DeviceExtension).ProgIf as i32,
            );
            PciIdPrintf(
                &raw mut IdBuffer,
                b"%s&CC_%02X%02X\0".as_ptr() as PCCH,
                &raw mut VendorString as *mut CHAR,
                (*DeviceExtension).BaseClass as i32,
                (*DeviceExtension).SubClass as i32,
            );
            PciIdPrintf(&raw mut IdBuffer, b"\0\0".as_ptr() as PCCH);
        }
        BusQueryCompatibleIDs => {
            PciIdPrintf(
                &raw mut IdBuffer,
                b"%s&REV_%02X\0".as_ptr() as PCCH,
                &raw mut VendorString as *mut CHAR,
                (*DeviceExtension).RevisionId as i32,
            );
            PciIdPrintf(
                &raw mut IdBuffer,
                b"%s\0".as_ptr() as PCCH,
                &raw mut VendorString as *mut CHAR,
            );
            PciIdPrintf(
                &raw mut IdBuffer,
                b"PCI\\VEN_%04X&CC_%02X%02X%02X\0".as_ptr() as PCCH,
                (*DeviceExtension).VendorId as i32,
                (*DeviceExtension).BaseClass as i32,
                (*DeviceExtension).SubClass as i32,
                (*DeviceExtension).ProgIf as i32,
            );
            PciIdPrintf(
                &raw mut IdBuffer,
                b"PCI\\VEN_%04X&CC_%02X%02X\0".as_ptr() as PCCH,
                (*DeviceExtension).VendorId as i32,
                (*DeviceExtension).BaseClass as i32,
                (*DeviceExtension).SubClass as i32,
            );
            PciIdPrintf(
                &raw mut IdBuffer,
                b"PCI\\VEN_%04X\0".as_ptr() as PCCH,
                (*DeviceExtension).VendorId as i32,
            );
            PciIdPrintf(
                &raw mut IdBuffer,
                b"PCI\\CC_%02X%02X%02X\0".as_ptr() as PCCH,
                (*DeviceExtension).BaseClass as i32,
                (*DeviceExtension).SubClass as i32,
                (*DeviceExtension).ProgIf as i32,
            );
            PciIdPrintf(
                &raw mut IdBuffer,
                b"PCI\\CC_%02X%02X\0".as_ptr() as PCCH,
                (*DeviceExtension).BaseClass as i32,
                (*DeviceExtension).SubClass as i32,
            );
            PciIdPrintf(&raw mut IdBuffer, b"\0\0".as_ptr() as PCCH);
        }
        BusQueryInstanceID => {
            PciIdPrintf(&raw mut IdBuffer, b"\0\0".as_ptr() as PCCH);
            PciIdPrintfAppend(
                &raw mut IdBuffer,
                b"%02X\0".as_ptr() as PCCH,
                ((*DeviceExtension).Slot.u.bits.DeviceNumber() as i32) << 3i32 | (*DeviceExtension).Slot.u.bits.FunctionNumber() as i32,
            );
            ParentExtension = (*DeviceExtension).ParentFdoExtension;
            while !((*ParentExtension).BusRootFdoExtension == ParentExtension) {
                PdoExtension = (*(*ParentExtension).PhysicalDeviceObject).DeviceExtension as PPCI_PDO_EXTENSION;
                PciIdPrintfAppend(
                    &raw mut IdBuffer,
                    b"%02X\0".as_ptr() as PCCH,
                    ((*PdoExtension).Slot.u.bits.DeviceNumber() as i32) << 3i32 | (*PdoExtension).Slot.u.bits.FunctionNumber() as i32,
                );
                // Up one bus: the bus the bridge itself sits on. The C never advances,
                // so a device behind a bridge appended the bridge's slot forever and ran
                // the 256-byte ID buffer off the stack (the OptiPlex's first device behind
                // a root port; QEMU's smoke machines have no bridges).
                ParentExtension = (*PdoExtension).ParentFdoExtension;
            }
        }
        _ => {
            // DPRINT removed
            return STATUS_NOT_SUPPORTED;
        }
    }

    /* Something should've been generated if this has been reached */
    if !(IdBuffer.Count > 0) {
        RtlAssert(
            b"IdBuffer.Count > 0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/bus/pcix/pci/id.c\0".as_ptr() as *const i8 as PVOID,
            359 as ULONG,
            (core::ptr::null_mut::<CHAR>()).cast(),
        );
    }

    /* Allocate the final string buffer to hold the ID */
    StringBuffer = ExAllocatePoolWithTag(PagedPool, IdBuffer.TotalLength as SIZE_T, (0x42696350u32) as ULONG) as PWCHAR;
    if StringBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // DPRINT removed

    /* Build the _UNICODE_STRING structure for it */
    DestinationString.Buffer = StringBuffer as PWCH;
    DestinationString.MaximumLength = IdBuffer.TotalLength as USHORT;

    /* Loop every ID in the buffer */
    i = 0;
    while i < IdBuffer.Count {
        NextString = (&raw mut IdBuffer.Strings as *mut ANSI_STRING).offset(i as isize) as PANSI_STRING;
        // DPRINT removed
        Status = RtlAnsiStringToUnicodeString(&raw mut DestinationString, NextString, FALSE as BOOLEAN);
        if !(Status >= 0) {
            RtlAssert(
                b"NT_SUCCESS(Status)\0".as_ptr() as *const i8 as PVOID,
                b"drivers/bus/pcix/pci/id.c\0".as_ptr() as *const i8 as PVOID,
                379 as ULONG,
                (core::ptr::null_mut::<CHAR>()).cast(),
            );
        }
        Size = IdBuffer.StringSize[i as usize];
        DestinationString.MaximumLength = (DestinationString.MaximumLength as u32).wrapping_sub(Size as u32) as USHORT;
        DestinationString.Buffer = DestinationString.Buffer.offset(
            (Size as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize,
        );
        i = i.wrapping_add(1);
    }

    /* Return the buffer to the caller and return status (should be success) */
    *Buffer = StringBuffer;
    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn PciQueryDeviceText(
    PdoExtension: PPCI_PDO_EXTENSION,
    QueryType: DEVICE_TEXT_TYPE,
    Locale: ULONG,
    Buffer: *mut PWCHAR,
) -> NTSTATUS {
    let _ = Locale;
    let mut message_buffer: PWCHAR;
    let mut location_buffer: PWCHAR;
    let mut length: ULONG = 0;
    let status: NTSTATUS;

    match QueryType {
        DeviceTextDescription => {
            message_buffer = PciGetDeviceDescriptionMessage(
                (*PdoExtension).BaseClass as UCHAR,
                (*PdoExtension).SubClass as UCHAR,
            );
            *Buffer = message_buffer;
            status = if !message_buffer.is_null() {
                STATUS_SUCCESS
            } else {
                STATUS_NOT_SUPPORTED
            };
        }
        DeviceTextLocationInformation => {
            message_buffer =
                PciGetDescriptionMessage(0x10000u32, core::ptr::addr_of_mut!(length));
            if message_buffer.is_null() {
                status = STATUS_NOT_SUPPORTED;
            } else {
                length += 2 * core::mem::size_of::<WCHAR>() as ULONG;
                location_buffer = ExAllocatePoolWithTag(
                    PagedPool,
                    (length as usize) * core::mem::size_of::<WCHAR>(),
                    0x42796350u32,
                ) as PWCHAR;
                *Buffer = location_buffer;
                if !location_buffer.is_null() {
                    swprintf(
                        location_buffer,
                        message_buffer,
                        (*(*PdoExtension).ParentFdoExtension).BaseBus as core::ffi::c_int,
                        (*PdoExtension).Slot.u.bits.FunctionNumber() as core::ffi::c_int,
                        (*PdoExtension).Slot.u.bits.DeviceNumber() as core::ffi::c_int,
                    );
                }
                ExFreePoolWithTag(message_buffer as *mut core::ffi::c_void, 0u32);
                status = if !location_buffer.is_null() {
                    STATUS_SUCCESS
                } else {
                    STATUS_INSUFFICIENT_RESOURCES
                };
            }
        }
        _ => {
            status = STATUS_NOT_SUPPORTED;
        }
    }

    status
}
