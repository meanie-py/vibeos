// ---- crate-internal glue (private structs, no-op DDK shims, inlines) ----
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
use crate::glue::{IRP_MN_DEVICE_USAGE_NOTIFICATION, IRP_MN_QUERY_CAPABILITIES, IRP_MN_QUERY_DEVICE_TEXT, IRP_MN_QUERY_ID, IRP_MN_QUERY_INTERFACE, IRP_MN_QUERY_PNP_DEVICE_STATE, IRP_MN_QUERY_RESOURCES, IRP_MN_QUERY_RESOURCE_REQUIREMENTS}; // explicit: disambiguate vs ntkrnl_sys::driver_ddk glob
#[allow(unused_imports)] use crate::inlines::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql; // FORCEINLINE on amd64/arm64

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{BUS_INTERFACE_STANDARD, _KWAIT_REASON_Executive, GUID, _IO_STATUS_BLOCK, IoAllocateDriverObjectExtension, IoAttachDeviceToDeviceStackSafe, IoBuildSynchronousFsdRequest, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoGetDriverObjectExtension, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _OBJECT_ATTRIBUTES, PCI_COMMON_HEADER, PDEVICE_OBJECT, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, PPCI_COMMON_HEADER, PUNICODE_STRING, RtlInitUnicodeString, _EVENT_TYPE_SynchronizationEvent, ZwClose, ZwCreateDirectoryObject};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{KPROCESSOR_MODE, NTSTRSAFE_PCWSTR, NTSTRSAFE_PWSTR, PCSTR, PCUNICODE_STRING, PINTERFACE, RtlAssert, _DEVICE_OBJECT, _GUID, _IRP, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{HANDLE, OBJECT_ATTRIBUTES};

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut GUID_HWPROFILE_QUERY_CHANGE: GUID = _GUID {
    Data1: 0xcb3a4001 as ::core::ffi::c_uint,
    Data2: 0x46f0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_HWPROFILE_CHANGE_CANCELLED: GUID = _GUID {
    Data1: 0xcb3a4002 as ::core::ffi::c_uint,
    Data2: 0x46f0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_HWPROFILE_CHANGE_COMPLETE: GUID = _GUID {
    Data1: 0xcb3a4003 as ::core::ffi::c_uint,
    Data2: 0x46f0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DEVICE_INTERFACE_ARRIVAL: GUID = _GUID {
    Data1: 0xcb3a4004 as ::core::ffi::c_uint,
    Data2: 0x46f0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DEVICE_INTERFACE_REMOVAL: GUID = _GUID {
    Data1: 0xcb3a4005 as ::core::ffi::c_uint,
    Data2: 0x46f0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_TARGET_DEVICE_QUERY_REMOVE: GUID = _GUID {
    Data1: 0xcb3a4006 as ::core::ffi::c_uint,
    Data2: 0x46f0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_TARGET_DEVICE_REMOVE_CANCELLED: GUID = _GUID {
    Data1: 0xcb3a4007 as ::core::ffi::c_uint,
    Data2: 0x46f0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_TARGET_DEVICE_REMOVE_COMPLETE: GUID = _GUID {
    Data1: 0xcb3a4008 as ::core::ffi::c_uint,
    Data2: 0x46f0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PNP_CUSTOM_NOTIFICATION: GUID = _GUID {
    Data1: 0xaca73f8e as ::core::ffi::c_uint,
    Data2: 0x8d23 as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PNP_POWER_NOTIFICATION: GUID = _GUID {
    Data1: 0xc2cf0660 as ::core::ffi::c_uint,
    Data2: 0xeb7a as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0x496b8280 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x6f25 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PCI_BUS_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0x496b8281 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x6f25 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_AGP_TARGET_BUS_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0xb15cfce8 as ::core::ffi::c_uint,
    Data2: 0x6d1 as ::core::ffi::c_ushort,
    Data3: 0x4d37 as ::core::ffi::c_ushort,
    Data4: [
        0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ARBITER_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0xe644f185 as ::core::ffi::c_uint,
    Data2: 0x8c0e as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_TRANSLATOR_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0x6c154a92 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xaacf as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ACPI_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0xb091a08a as ::core::ffi::c_uint,
    Data2: 0xba97 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_INT_ROUTE_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0x70941bf4 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x73 as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PCMCIA_BUS_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0x76173af0 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xc504 as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xee as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ACPI_REGS_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0x6141966 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x7245 as ::core::ffi::c_ushort,
    Data3: 0x6369 as ::core::ffi::c_ushort,
    Data4: [
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x65 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x73 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_LEGACY_DEVICE_DETECTION_STANDARD: GUID = _GUID {
    Data1: 0x50feb0de as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x596a as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PCI_DEVICE_PRESENT_INTERFACE: GUID = _GUID {
    Data1: 0xd1b82c26 as ::core::ffi::c_uint,
    Data2: 0xbf49 as ::core::ffi::c_ushort,
    Data3: 0x45ef as ::core::ffi::c_ushort,
    Data4: [
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_MF_ENUMERATION_INTERFACE: GUID = _GUID {
    Data1: 0xaeb895f0 as ::core::ffi::c_uint,
    Data2: 0x5586 as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ACPI_CMOS_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0x3a8d0384 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x6505 as ::core::ffi::c_ushort,
    Data3: 0x40ca as ::core::ffi::c_ushort,
    Data4: [
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x39 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x56 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ACPI_PORT_RANGES_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0xf14f609b as ::core::ffi::c_uint,
    Data2: 0xcbbd as ::core::ffi::c_ushort,
    Data3: 0x4957 as ::core::ffi::c_ushort,
    Data4: [
        0xa6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PNP_LOCATION_INTERFACE: GUID = _GUID {
    Data1: 0x70211b0e as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xafb as ::core::ffi::c_ushort,
    Data3: 0x47db as ::core::ffi::c_ushort,
    Data4: [
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x41 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_D3COLD_SUPPORT_INTERFACE: GUID = _GUID {
    Data1: 0xb38290e5 as ::core::ffi::c_uint,
    Data2: 0x3cd0 as ::core::ffi::c_ushort,
    Data3: 0x4f9d as ::core::ffi::c_ushort,
    Data4: [
        0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_REENUMERATE_SELF_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0x2aeb0243 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x6a6e as ::core::ffi::c_ushort,
    Data3: 0x486b as ::core::ffi::c_ushort,
    Data4: [
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_INTERNAL: GUID = _GUID {
    Data1: 0x1530ea73 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x86b as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_PCMCIA: GUID = _GUID {
    Data1: 0x9343630 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xaf9f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x92 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x30 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_PCI: GUID = _GUID {
    Data1: 0xc8ebdfb0 as ::core::ffi::c_uint,
    Data2: 0xb510 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_ISAPNP: GUID = _GUID {
    Data1: 0xe676f854 as ::core::ffi::c_uint,
    Data2: 0xd87d as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x92 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_EISA: GUID = _GUID {
    Data1: 0xddc35509 as ::core::ffi::c_uint,
    Data2: 0xf3fc as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_MCA: GUID = _GUID {
    Data1: 0x1c75997a as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xdc33 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x92 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_LPTENUM: GUID = _GUID {
    Data1: 0xc4ca1000 as ::core::ffi::c_uint,
    Data2: 0x2ddc as ::core::ffi::c_ushort,
    Data3: 0x11d5 as ::core::ffi::c_ushort,
    Data4: [
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_USBPRINT: GUID = _GUID {
    Data1: 0x441ee000 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4342 as ::core::ffi::c_ushort,
    Data3: 0x11d5 as ::core::ffi::c_ushort,
    Data4: [
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_DOT4PRT: GUID = _GUID {
    Data1: 0x441ee001 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4342 as ::core::ffi::c_ushort,
    Data3: 0x11d5 as ::core::ffi::c_ushort,
    Data4: [
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_SERENUM: GUID = _GUID {
    Data1: 0x77114a87 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x8944 as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x90 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_USB: GUID = _GUID {
    Data1: 0x9d7debbc as ::core::ffi::c_uint,
    Data2: 0xc85d as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_1394: GUID = _GUID {
    Data1: 0xf74e73eb as ::core::ffi::c_uint,
    Data2: 0x9ac5 as ::core::ffi::c_ushort,
    Data3: 0x45eb as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_HID: GUID = _GUID {
    Data1: 0xeeaf37d0 as ::core::ffi::c_uint,
    Data2: 0x1963 as ::core::ffi::c_ushort,
    Data3: 0x47c4 as ::core::ffi::c_ushort,
    Data4: [
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x48 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x72 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_AVC: GUID = _GUID {
    Data1: 0xc06ff265 as ::core::ffi::c_uint,
    Data2: 0xae09 as ::core::ffi::c_ushort,
    Data3: 0x48f0 as ::core::ffi::c_ushort,
    Data4: [
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_IRDA: GUID = _GUID {
    Data1: 0x7ae17dc1 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xc944 as ::core::ffi::c_ushort,
    Data3: 0x44d6 as ::core::ffi::c_ushort,
    Data4: [
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x61 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BUS_TYPE_SD: GUID = _GUID {
    Data1: 0xe700cc04 as ::core::ffi::c_uint,
    Data2: 0x4036 as ::core::ffi::c_ushort,
    Data3: 0x4e89 as ::core::ffi::c_ushort,
    Data4: [
        0x95 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xeb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcd as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_POWER_DEVICE_ENABLE: GUID = _GUID {
    Data1: 0x827c0a6f as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xfeb0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x26 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_POWER_DEVICE_TIMEOUTS: GUID = _GUID {
    Data1: 0xa45da735 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xfeb0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x26 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_POWER_DEVICE_WAKE_ENABLE: GUID = _GUID {
    Data1: 0xa9546a82 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xfeb0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x26 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut PciIdeControllerNumber: ULONG = 0 as ULONG;

#[no_mangle]
pub unsafe extern "system" fn PciIdeXDispatchWmi(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/storage/ide/pciidex/pciidex.c\0"
                    as *const u8 as *const i8 as PVOID,
                24 as u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    let Status: NTSTATUS;
    if (*((*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION)).IsFDO != 0 {
        let FdoExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*FdoExtension).Ldo, Irp);
    } else {
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXUnload(DriverObject: PDRIVER_OBJECT) {
    let _ = DriverObject;
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8).cast(),
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"FALSE\0".as_ptr() as *mut core::ffi::c_void,
            b"drivers/storage/ide/pciidex/pciidex.c\0"
                .as_ptr() as *mut core::ffi::c_void,
            50 as u32,
            core::ptr::null_mut::<i8>(),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXQueryInterface(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    Guid: *const GUID,
    Interface: PVOID,
    Size: ULONG,
) -> NTSTATUS {
    // PAGED_CODE expansion
    if (KeGetCurrentIrql() as i32) > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            0 as PVOID,
            b"drivers/storage/ide/pciidex/pciidex.c\0"
                .as_ptr() as *const i8 as PVOID,
            70,
            core::ptr::null_mut(),
        );
    }

    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();

    KeInitializeEvent(&raw mut Event, _EVENT_TYPE_SynchronizationEvent, 0u8);

    let Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as ULONG,
        (*FdoExtension).Ldo,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        &raw mut Event,
        &raw mut IoStatus,
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    (*Irp).IoStatus.Information = 0;

    let Stack = IoGetNextIrpStackLocation(Irp);
    (*Stack).MinorFunction = IRP_MN_QUERY_INTERFACE as UCHAR;
    (*Stack).Parameters.QueryInterface.InterfaceType = Guid;
    (*Stack).Parameters.QueryInterface.Version = 1u16;
    (*Stack).Parameters.QueryInterface.Size = Size as USHORT;
    (*Stack).Parameters.QueryInterface.Interface = Interface as PINTERFACE;
    (*Stack).Parameters.QueryInterface.InterfaceSpecificData = core::ptr::null_mut();

    let mut Status = IoCallDriver((*FdoExtension).Ldo, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &raw mut Event as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as KPROCESSOR_MODE,
            0u8,
            core::ptr::null_mut(),
        );
        Status = IoStatus.__bindgen_anon_1.Status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn PciIdeXGetConfigurationInfo(
    FdoExtension: PFDO_DEVICE_EXTENSION,
) -> NTSTATUS {
    // Buffer large enough for PCI_COMMON_HEADER up to BaseClass (12 bytes)
    let mut buffer: [u8; 12] = [0; 12];
    let pci_config: PPCI_COMMON_HEADER = buffer.as_mut_ptr() as PPCI_COMMON_HEADER;

    // PAGED_CODE
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        // Always true; force assertion in debug builds
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PCSTR as PVOID,
                b"drivers/storage/ide/pciidex/pciidex.c\0".as_ptr() as PCSTR as PVOID,
                115 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    let bytes_read: ULONG = Some((*FdoExtension).BusInterface.GetBusData.expect("non-null function pointer"))
        .expect("non-null function pointer")(
            (*FdoExtension).BusInterface.Context,
            PCI_WHICHSPACE_CONFIG as ULONG,
            buffer.as_mut_ptr() as PVOID,
            0 as ULONG,
            core::mem::size_of::<[u8; 12]>() as ULONG,
        );

    if bytes_read as usize != core::mem::size_of::<[u8; 12]>() {
        return STATUS_IO_DEVICE_ERROR;
    }

    (*FdoExtension).VendorId = (*pci_config).VendorID;
    (*FdoExtension).DeviceId = (*pci_config).DeviceID;

    if (*pci_config).BaseClass as i32 == PCI_CLASS_MASS_STORAGE_CTLR as i32 {
        if (*pci_config).SubClass as i32 == PCI_SUBCLASS_MSC_IDE_CTLR as i32 {
            (*FdoExtension).InNativeMode =
                (((*pci_config).ProgIf as i32 & PCIIDE_PROGIF_PRIMARY_CHANNEL_NATIVE_MODE as i32) != 0
                    && ((*pci_config).ProgIf as i32 & PCIIDE_PROGIF_SECONDARY_CHANNEL_NATIVE_MODE as i32) != 0)
                    as BOOLEAN;
        } else if (*pci_config).SubClass as i32 == PCI_SUBCLASS_MSC_RAID_CTLR as i32 {
            (*FdoExtension).InNativeMode = TRUE as BOOLEAN;
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut DriverExtension: *mut _PCIIDEX_DRIVER_EXTENSION ;
    let mut FdoExtension: *mut _FDO_DEVICE_EXTENSION ;
    let mut DeviceExtensionSize: ULONG = 0;
    let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DeviceNameBuffer: [WCHAR; 22] = [0; 22];
    let mut Status: NTSTATUS;

    // PAGED_CODE()
    if KeGetCurrentIrql() as core::ffi::c_int > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const core::ffi::c_char as PVOID,
                b"drivers/storage/ide/pciidex/pciidex.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
                167 as LVL,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    // L"\Device\Ide\PciIde%u" — bind to a local so we don't take the address of a temporary.
    let ide_dev_fmt: [u16; 21] = core::mem::transmute::<[u8; 42], [u16; 21]>(
        *b"\\\0D\0e\0v\0i\0c\0e\0\\\0I\0d\0e\0\\\0P\0c\0i\0I\0d\0e\0%\0u\0\0\0",
    );
    Status = RtlStringCbPrintfW(
        &raw mut DeviceNameBuffer as NTSTRSAFE_PWSTR,
        core::mem::size_of::<[WCHAR; 22]>() as size_t,
        ide_dev_fmt.as_ptr() as NTSTRSAFE_PCWSTR,
        PciIdeControllerNumber,
    );
    if !(Status >= 0) {
        RtlAssert(
            b"NT_SUCCESS(Status)\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/storage/ide/pciidex/pciidex.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
            173 as LVL,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    RtlInitUnicodeString(
        &raw mut DeviceName,
        &raw mut DeviceNameBuffer as *mut WCHAR as PCWSTR,
    );

    DbgPrint(
        b"%s(%p, %p) '%wZ'\n\0" as *const u8 as PCSTR,
        b"PciIdeXAddDevice\0" as *const u8 as *const core::ffi::c_char,
        &raw const DriverObject as PVOID,
        &raw const PhysicalDeviceObject as PVOID,
        &raw const DeviceName as PCUNICODE_STRING,
    );

    DriverExtension = IoGetDriverObjectExtension(DriverObject, DriverObject as PVOID) as *mut _PCIIDEX_DRIVER_EXTENSION;
    if DriverExtension.is_null() {
        RtlAssert(
            b"DriverExtension\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/storage/ide/pciidex/pciidex.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
            179 as LVL,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    Status = RtlULongAdd(
        (*DriverExtension).MiniControllerExtensionSize,
        core::mem::size_of::<FDO_DEVICE_EXTENSION>() as LVL,
        &raw mut DeviceExtensionSize,
    );
    if !(Status >= 0) {
        if DbgPrint(
            b"(%s:%d) Invalid miniport extension size %lx\n\0" as *const u8 as PCSTR,
            b"drivers/storage/ide/pciidex/pciidex.c\0" as *const u8 as *const core::ffi::c_char,
            187 as core::ffi::c_int,
            (*DriverExtension).MiniControllerExtensionSize,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"drivers/storage/ide/pciidex/pciidex.c\0" as *const u8 as *const core::ffi::c_char,
                187 as core::ffi::c_int,
            );
        }
        return Status;
    }

    Status = IoCreateDevice(
        DriverObject,
        DeviceExtensionSize,
        &raw mut DeviceName,
        FileDeviceBusExtender as LVL,
        FileDeviceSecureOpen as LVL,
        0u8,
        &raw mut Fdo,
    );
    if !(Status >= 0) {
        if DbgPrint(
            b"(%s:%d) Failed to create FDO 0x%lx\n\0" as *const u8 as PCSTR,
            b"drivers/storage/ide/pciidex/pciidex.c\0" as *const u8 as *const core::ffi::c_char,
            200 as core::ffi::c_int,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"drivers/storage/ide/pciidex/pciidex.c\0" as *const u8 as *const core::ffi::c_char,
                200 as core::ffi::c_int,
            );
        }
        return Status;
    }

    FdoExtension = (*Fdo).DeviceExtension as *mut _FDO_DEVICE_EXTENSION;

    core::ptr::write_bytes(FdoExtension as *mut u8, 0, core::mem::size_of::<FDO_DEVICE_EXTENSION>());

    (*FdoExtension).Common.IsFDO = 1u8;
    (*FdoExtension).Common.Self_ = Fdo;
    (*FdoExtension).DriverObject = DriverObject;
    let fresh0 = PciIdeControllerNumber;
    PciIdeControllerNumber = PciIdeControllerNumber.wrapping_add(1);
    (*FdoExtension).ControllerNumber = fresh0;

    KeInitializeSpinLock(&raw mut (*FdoExtension).BusDataLock);
    ExInitializeFastMutex(&raw mut (*FdoExtension).DeviceSyncMutex);

    Status = IoAttachDeviceToDeviceStackSafe(Fdo, PhysicalDeviceObject, &raw mut (*FdoExtension).Ldo);
    if Status >= 0 {
        (*Fdo).AlignmentRequirement = if (*(*FdoExtension).Ldo).AlignmentRequirement > 0x1 as LVL {
            (*(*FdoExtension).Ldo).AlignmentRequirement
        } else {
            0x1 as LVL
        };

        Status = PciIdeXQueryInterface(
            FdoExtension,
            &raw const GUID_BUS_INTERFACE_STANDARD,
            &raw mut (*FdoExtension).BusInterface as PVOID,
            core::mem::size_of::<BUS_INTERFACE_STANDARD>() as LVL,
        );
        if !(Status >= 0) {
            if DbgPrint(
                b"(%s:%d) No bus interface 0x%lx\n\0" as *const u8 as PCSTR,
                b"drivers/storage/ide/pciidex/pciidex.c\0" as *const u8 as *const core::ffi::c_char,
                231 as core::ffi::c_int,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"drivers/storage/ide/pciidex/pciidex.c\0" as *const u8 as *const core::ffi::c_char,
                    231 as core::ffi::c_int,
                );
            }
        } else {
            Status = PciIdeXGetConfigurationInfo(FdoExtension);
            if !(Status >= 0) {
                if DbgPrint(
                    b"(%s:%d) Unable to retrieve the configuration info %lx\n\0" as *const u8 as PCSTR,
                    b"drivers/storage/ide/pciidex/pciidex.c\0" as *const u8 as *const core::ffi::c_char,
                    238 as core::ffi::c_int,
                    Status,
                ) != 0
                {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                        b"drivers/storage/ide/pciidex/pciidex.c\0" as *const u8 as *const core::ffi::c_char,
                        238 as core::ffi::c_int,
                    );
                }
            } else {
                (*Fdo).Flags &= !DO_DEVICE_INITIALIZING as LVL;
                return STATUS_SUCCESS;
            }
        }
    }

    // Failure path
    if !(*FdoExtension).Ldo.is_null() {
        IoDetachDevice((*FdoExtension).Ldo);
    }
    IoDeleteDevice(Fdo);
    return Status;
}

// Recovered from the stitcher-dropped .responses/.../05_PciIdeXCreateIdeDirectory.partial
// (truncated mid-body) + completed faithfully from pciidex.c: creates the permanent
// \Device\Ide directory object (failures intentionally ignored — another driver may have
// created it). VOID return.
unsafe fn PciIdeXCreateIdeDirectory() {
    // L"\Device\Ide" (RTL_CONSTANT_STRING): 11 chars, Length=22, MaximumLength=24.
    static DIRECTORY_NAME_BUFFER: [u16; 12] = [
        0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0049, 0x0064, 0x0065, 0x0000,
    ];
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut DirectoryName: UNICODE_STRING = UNICODE_STRING {
        Length: 22,
        MaximumLength: 24,
        Buffer: DIRECTORY_NAME_BUFFER.as_ptr() as *mut u16,
    };
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    ObjectAttributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut DirectoryName as PUNICODE_STRING;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_PERMANENT | OBJ_KERNEL_HANDLE) as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    const DIRECTORY_ALL_ACCESS: ULONG = 0x000F000F;
    let Status = ZwCreateDirectoryObject(&raw mut Handle, DIRECTORY_ALL_ACCESS, &raw mut ObjectAttributes);
    if Status >= 0 {
        // Permanent object — we don't need to keep the handle.
        ZwClose(Handle);
    }
    // Ignore directory-creation failures (the directory may already exist).
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXInitialize(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
    HwGetControllerProperties: PCONTROLLER_PROPERTIES,
    ExtensionSize: ULONG,
) -> NTSTATUS {
    // PAGED_CODE() expansion
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const core::ffi::c_char,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as PVOID,
                b"drivers/storage/ide/pciidex/pciidex.c\0"
                    as *const u8 as PVOID,
                298u32,
                core::ptr::null_mut::<core::ffi::c_char>(),
            );
        }
    }

    let mut DriverExtension: PPCIIDEX_DRIVER_EXTENSION = core::ptr::null_mut::<_PCIIDEX_DRIVER_EXTENSION>();
    let Status: NTSTATUS = IoAllocateDriverObjectExtension(
        DriverObject,
        DriverObject as PVOID,
        core::mem::size_of::<PCIIDEX_DRIVER_EXTENSION>() as ULONG,
        &raw mut DriverExtension as *mut PVOID,
    );
    if !(Status >= 0) {
        return Status;
    }

    core::ptr::write_bytes(
        DriverExtension as *mut u8,
        0u8,
        core::mem::size_of::<PCIIDEX_DRIVER_EXTENSION>(),
    );

    (*DriverExtension).MiniControllerExtensionSize = ExtensionSize;
    (*DriverExtension).HwGetControllerProperties = HwGetControllerProperties;

    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] =
        Some(core::mem::transmute(PciIdeXDispatchPnp as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] =
        Some(core::mem::transmute(PciIdeXDispatchPower as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] =
        Some(core::mem::transmute(PciIdeXDispatchWmi as *const ()));

    (*(*DriverObject).DriverExtension).AddDevice =
        Some(core::mem::transmute(PciIdeXAddDevice as *const ()));
    (*DriverObject).DriverUnload =
        Some(core::mem::transmute(PciIdeXUnload as *const ()));

    PciIdeXCreateIdeDirectory();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    _driver_object: PDRIVER_OBJECT,
    _registry_path: PUNICODE_STRING,
) -> NTSTATUS {
    // UNREFERENCED_PARAMETER
    let _ = (_driver_object, _registry_path);

    // PAGED_CODE macro expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if true {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/storage/ide/pciidex/pciidex.c\0"
                    .as_ptr() as PVOID,
                336 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    STATUS_SUCCESS
}
