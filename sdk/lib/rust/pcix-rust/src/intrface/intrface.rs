#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{GUID, PINTERFACE, RtlFreeUnicodeString, RtlStringFromGUID};
use ntkrnl_sys::ddk::{PCSTR, _UNICODE_STRING};
use ntkrnl_sys::driver_ddk::DbgPrint;
use core::ffi::c_void;

// IsEqualGUIDAligned (ddk/wdm.h macro): compare two GUIDs as two i64 halves.
#[inline(always)]
unsafe fn IsEqualGUIDAligned(guid1: *const GUID, guid2: *const GUID) -> bool {
    let a = guid1 as *const i64;
    let b = guid2 as *const i64;
    *a == *b && *a.offset(1) == *b.offset(1)
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut PciInterfaces: [PPCI_INTERFACE; 12] = unsafe {
    [
        &raw const ArbiterInterfaceBusNumber as *mut PCI_INTERFACE,
        &raw const ArbiterInterfaceMemory as *mut PCI_INTERFACE,
        &raw const ArbiterInterfaceIo as *mut PCI_INTERFACE,
        &raw const BusHandlerInterface as *mut PCI_INTERFACE,
        &raw const PciRoutingInterface as *mut PCI_INTERFACE,
        &raw const PciCardbusPrivateInterface as *mut PCI_INTERFACE,
        &raw const PciLegacyDeviceDetectionInterface as *mut PCI_INTERFACE,
        &raw const PciPmeInterface as *mut PCI_INTERFACE,
        &raw const PciDevicePresentInterface as *mut PCI_INTERFACE,
        &raw const PciLocationInterface as *mut PCI_INTERFACE,
        &raw const AgpTargetInterface as *mut PCI_INTERFACE,
        ::core::ptr::null::<_PCI_INTERFACE>() as *mut _PCI_INTERFACE,
    ]
};

#[no_mangle]
pub static mut PciInterfacesLastResort: [PPCI_INTERFACE; 2] = unsafe {
    [
        &raw const TranslatorInterfaceInterrupt as *mut PCI_INTERFACE,
        ::core::ptr::null::<_PCI_INTERFACE>() as *mut _PCI_INTERFACE,
    ]
};

pub const NDEBUG: bool = true;

#[no_mangle]
pub unsafe extern "system" fn PciQueryInterface(
    DeviceExtension: PPCI_FDO_EXTENSION,
    InterfaceType: *const GUID,
    Size: ULONG,
    Version: ULONG,
    InterfaceData: PVOID,
    Interface: PINTERFACE,
    LastChance: BOOLEAN,
) -> NTSTATUS {
    let mut GuidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS;

    RtlStringFromGUID(InterfaceType, &raw mut GuidString);
    DbgPrint(
        b"(%s:%d) PCI - PciQueryInterface TYPE = %wZ\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/intrface/intrface.c\0".as_ptr()
            as *const core::ffi::c_char,
        58 as core::ffi::c_int,
        &raw mut GuidString,
    );
    RtlFreeUnicodeString(&raw mut GuidString);

    DbgPrint(
        b"(%s:%d)       Size = %u, Version = %u, InterfaceData = %p, LastChance = %s\n\0".as_ptr()
            as PCSTR,
        b"drivers/bus/pcix/intrface/intrface.c\0".as_ptr()
            as *const core::ffi::c_char,
        64 as core::ffi::c_int,
        Size,
        Version,
        InterfaceData,
        if LastChance != 0 {
            b"TRUE\0".as_ptr() as *const core::ffi::c_char
        } else {
            b"FALSE\0".as_ptr() as *const core::ffi::c_char
        },
    );

    let mut InterfaceList: *mut PPCI_INTERFACE = if LastChance != 0 {
        &raw mut PciInterfacesLastResort as *mut PPCI_INTERFACE
    } else {
        &raw mut PciInterfaces as *mut PPCI_INTERFACE
    };

    while !(*InterfaceList).is_null() {
        let PciInterface: PPCI_INTERFACE = *InterfaceList;

        RtlStringFromGUID((*PciInterface).InterfaceType, &raw mut GuidString);

        if (*DeviceExtension).ExtensionType == PciFdoExtensionType as ULONG {
            if ((*PciInterface).Flags as i32 & PCI_INTERFACE_FDO as i32) == 0 {
                DbgPrint(
                    b"(%s:%d) PCI - PciQueryInterface: guid = %wZ only for FDOs\n\0".as_ptr()
                        as PCSTR,
                    b"drivers/bus/pcix/intrface/intrface.c\0"
                        .as_ptr() as *const core::ffi::c_char,
                    85 as core::ffi::c_int,
                    &raw mut GuidString,
                );
                RtlFreeUnicodeString(&raw mut GuidString);
                InterfaceList = InterfaceList.offset(1);
                continue;
            }
            if ((*PciInterface).Flags as i32 & PCI_INTERFACE_ROOT as i32) != 0
                && !((*DeviceExtension).BusRootFdoExtension == DeviceExtension)
            {
                DbgPrint(
                    b"(%s:%d) PCI - PciQueryInterface: guid = %wZ only for ROOT\n\0".as_ptr()
                        as PCSTR,
                    b"drivers/bus/pcix/intrface/intrface.c\0"
                        .as_ptr() as *const core::ffi::c_char,
                    96 as core::ffi::c_int,
                    &raw mut GuidString,
                );
                RtlFreeUnicodeString(&raw mut GuidString);
                InterfaceList = InterfaceList.offset(1);
                continue;
            }
        } else {
            if ((*PciInterface).Flags as i32 & PCI_INTERFACE_PDO as i32) == 0 {
                DbgPrint(
                    b"(%s:%d) PCI - PciQueryInterface: guid = %wZ only for PDOs\n\0".as_ptr()
                        as PCSTR,
                    b"drivers/bus/pcix/intrface/intrface.c\0"
                        .as_ptr() as *const core::ffi::c_char,
                    108 as core::ffi::c_int,
                    &raw mut GuidString,
                );
                RtlFreeUnicodeString(&raw mut GuidString);
                InterfaceList = InterfaceList.offset(1);
                continue;
            }
        }

        DbgPrint(
            b"(%s:%d) PCI - PciQueryInterface looking at guid = %wZ\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/intrface/intrface.c\0"
                .as_ptr() as *const core::ffi::c_char,
            115 as core::ffi::c_int,
            &raw mut GuidString,
        );
        RtlFreeUnicodeString(&raw mut GuidString);

        if IsEqualGUIDAligned((*PciInterface).InterfaceType, InterfaceType)
            && Version >= (*PciInterface).MinVersion as ULONG
            && Version <= (*PciInterface).MaxVersion as ULONG
            && Size >= (*PciInterface).MinSize as ULONG
        {
            Status = ((*PciInterface).Constructor).unwrap()(
                DeviceExtension as PVOID,
                PciInterface as PVOID,
                InterfaceData,
                Version as USHORT,
                Size as USHORT,
                Interface,
            );
            if !NT_SUCCESS(Status) {
                DbgPrint(
                    b"(%s:%d) PCI - PciQueryInterface - Constructor %p = %08lx\n\0".as_ptr()
                        as PCSTR,
                    b"drivers/bus/pcix/intrface/intrface.c\0"
                        .as_ptr() as *const core::ffi::c_char,
                    135 as core::ffi::c_int,
                    (*PciInterface).Constructor,
                    Status,
                );
                InterfaceList = InterfaceList.offset(1);
                continue;
            }

            ((*Interface).InterfaceReference).unwrap()((*Interface).Context);
            DbgPrint(
                b"(%s:%d) PCI - PciQueryInterface returning SUCCESS\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/intrface/intrface.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                141 as core::ffi::c_int,
            );
            return Status;
        }

        InterfaceList = InterfaceList.offset(1);
    }

    DbgPrint(
        b"(%s:%d) PCI - PciQueryInterface FAILED TO FIND INTERFACE\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/intrface/intrface.c\0".as_ptr()
            as *const core::ffi::c_char,
        147 as core::ffi::c_int,
    );
    STATUS_NOT_SUPPORTED
}
