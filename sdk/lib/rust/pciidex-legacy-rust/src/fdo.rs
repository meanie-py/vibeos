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
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CM_RESOURCE_TYPE, DEVICE_RELATIONS, ExAcquireFastMutex, ExReleaseFastMutex, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoForwardIrpSynchronously, MmMapIoSpace, MmUnmapIoSpace, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PDEVICE_OBJECT, PDEVICE_RELATIONS, PIO_STACK_LOCATION, PIRP, PTRANSLATOR_INTERFACE, RtlInitUnicodeString, TRANSLATOR_INTERFACE};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{GUID, PCSTR, PDRIVER_OBJECT, RtlAssert, _DEVICE_RELATIONS, _GUID, _IRP, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports

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
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoParseResources(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    ResourcesTranslated: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    // PAGED_CODE expanded
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if true {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/storage/ide/pciidex/fdo.c\0".as_ptr() as PVOID,
                25u32 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    let mut bus_master_descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut io_base: PVOID ;

    if ResourcesTranslated.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let count = (*ResourcesTranslated).List[0].PartialResourceList.Count;
    let descriptors_ptr = core::ptr::addr_of_mut!((*ResourcesTranslated).List[0].PartialResourceList.PartialDescriptors[0]);
    let mut i: u32 = 0;
    while i < count {
        let descriptor = descriptors_ptr.add(i as usize);
        match *descriptor {
            __CM_PARTIAL_RESOURCE_DESCRIPTOR {
                Type: t @ CmResourceTypePort
                | t @ CmResourceTypeMemory,
                ..
            } => match (*descriptor).u.Port.Length {
                16 => {
                    if bus_master_descriptor.is_null() {
                        bus_master_descriptor = (descriptor).cast();
                    }
                }
                _ => {}
            },
            _ => {}
        }
        i = i.wrapping_add(1);
    }

    if bus_master_descriptor.is_null() {
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }

    if (*bus_master_descriptor).Type == CmResourceTypePort
        && ((*bus_master_descriptor).Flags & (CM_RESOURCE_PORT_IO) as u16) != 0
    {
        io_base = (*bus_master_descriptor).u.Port.Start.QuadPart as ULONG_PTR as PVOID;
    } else {
        io_base = MmMapIoSpace(
            (*bus_master_descriptor).u.Memory.Start,
            (16 as SIZE_T) as _,
            MmNonCached,
        );
        if io_base.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        (*FdoExtension).IoBaseMapped = 1u8; // TRUE as BOOLEAN
    }
    (*FdoExtension).BusMasterPortBase = io_base as PUCHAR;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoStartDevice(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    // PAGED_CODE()
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0" as *const u8 as PVOID,
            b"drivers/storage/ide/pciidex/fdo.c\0"
                as *const u8 as PVOID,
            91 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    // NT_VERIFY(IoForwardIrpSynchronously(...))
    let forward_status = IoForwardIrpSynchronously((*FdoExtension).Ldo, Irp);
    if !NT_SUCCESS((forward_status) as i32) {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR,
            b"drivers/storage/ide/pciidex/fdo.c\0"
                as *const u8 as PCSTR,
            93 as i32,
            b"IoForwardIrpSynchronously(FdoExtension->Ldo, Irp)\0" as *const u8 as PCSTR,
        );
        crate::inlines::rust_breakpoint();
        return STATUS_UNSUCCESSFUL;
    }

    let status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    if !NT_SUCCESS(status) {
        return status;
    }

    let io_stack = IoGetCurrentIrpStackLocation(Irp);
    let parse_status = PciIdeXFdoParseResources(
        FdoExtension,
        (*io_stack).Parameters.StartDevice.AllocatedResourcesTranslated,
    );
    if !NT_SUCCESS(parse_status) {
        DbgPrint(
            b"(%s:%d) Failed to parse resources 0x%lx\n\0" as *const u8 as PCSTR,
            b"drivers/storage/ide/pciidex/fdo.c\0"
                as *const u8 as PCSTR,
            109 as i32,
            parse_status,
        );
        return parse_status;
    }

    let miniport_status = PciIdeXStartMiniport(FdoExtension);
    if !NT_SUCCESS(miniport_status) {
        DbgPrint(
            b"(%s:%d) Miniport initialization failed 0x%lx\n\0" as *const u8 as PCSTR,
            b"drivers/storage/ide/pciidex/fdo.c\0"
                as *const u8 as PCSTR,
            116 as i32,
            miniport_status,
        );
        return miniport_status;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoFreeResources(FdoExtension: *mut FDO_DEVICE_EXTENSION) {
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if false {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut c_void,
                b"drivers/storage/ide/pciidex/fdo.c\0".as_ptr() as *const i8 as *mut c_void,
                129u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    if (*FdoExtension).IoBaseMapped != 0 {
        MmUnmapIoSpace((*FdoExtension).BusMasterPortBase as *mut c_void, (16usize) as _);
        (*FdoExtension).IoBaseMapped = 0u8;
    }
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoStopDevice(FdoExtension: *mut FDO_DEVICE_EXTENSION) -> NTSTATUS {
    if KeGetCurrentIrql() as u32 > APC_LEVEL as u32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/storage/ide/pciidex/fdo.c\0".as_ptr() as *const i8 as PVOID,
                144 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    PciIdeXFdoFreeResources(FdoExtension);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoRemoveDevice(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut PdoExtension: PPDO_DEVICE_EXTENSION ;
    let mut Status: NTSTATUS ;
    let mut i: ULONG ;

    if KeGetCurrentIrql() as core::ffi::c_int > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        if 0 as core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const core::ffi::c_char as PVOID,
                b"drivers/storage/ide/pciidex/fdo.c\0"
                    .as_ptr() as *const core::ffi::c_char as PVOID,
                162 as ULONG,
                core::ptr::null_mut::<core::ffi::c_char>(),
            );
        }
    }

    PciIdeXFdoFreeResources(FdoExtension);

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*FdoExtension).DeviceSyncMutex));

    i = 0;
    while i < MAX_IDE_CHANNEL as ULONG {
        PdoExtension = (*FdoExtension).Channels[i as usize] as PPDO_DEVICE_EXTENSION;
        if !PdoExtension.is_null() {
            IoDeleteDevice((*PdoExtension).Common.Self_);
            (*FdoExtension).Channels[i as usize] = core::ptr::null_mut::<_PDO_DEVICE_EXTENSION>();
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }

    ExReleaseFastMutex(core::ptr::addr_of_mut!((*FdoExtension).DeviceSyncMutex));

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoSkipCurrentIrpStackLocation(Irp);
    Status = IoCallDriver((*FdoExtension).Ldo, Irp);
    IoDetachDevice((*FdoExtension).Ldo);
    IoDeleteDevice((*FdoExtension).Common.Self_);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoQueryPnpDeviceState(
    FdoExtension: *mut _FDO_DEVICE_EXTENSION,
    Irp: *mut _IRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8
                as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *mut core::ffi::c_void,
                b"drivers/storage/ide/pciidex/fdo.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                200u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    if (*FdoExtension).Common.PageFiles != 0
        || (*FdoExtension).Common.HibernateFiles != 0
        || (*FdoExtension).Common.DumpFiles != 0
    {
        (*Irp).IoStatus.Information |= PNP_DEVICE_NOT_DISABLEABLE as ntkrnl_sys::ddk::ULONG_PTR;
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoCreateDevice(
    FdoExtension: *mut _FDO_DEVICE_EXTENSION,
    ChannelNumber: u32,
) -> *mut _PDO_DEVICE_EXTENSION {
    let mut Status: i32;
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DeviceNameBuffer: [u16; 34] = [0u16; 34];
    let mut Pdo: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut PdoExtension: PPDO_DEVICE_EXTENSION;
    let mut Alignment: u32;
    static mut DeviceNumber: u32 = 0;

    // PAGED_CODE()
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                b"drivers/storage/ide/pciidex/fdo.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                229,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    let fresh6 = unsafe {
        let val = DeviceNumber;
        DeviceNumber = DeviceNumber.wrapping_add(1);
        val
    };

    let format_bytes: &[u8; 66] =
        b"\\\0D\0e\0v\0i\0c\0e\0\\\0I\0d\0e\0\\\0P\0c\0i\0I\0d\0e\0%\0u\0C\0h\0a\0n\0n\0e\0l\0%\0u\0-\0%\0x\0\0\0";
    let format_ptr: *const u16 = core::mem::transmute(format_bytes);

    Status = RtlStringCbPrintfW(
        DeviceNameBuffer.as_mut_ptr(),
        core::mem::size_of::<[u16; 34]>(),
        format_ptr,
        (*FdoExtension).ControllerNumber,
        ChannelNumber,
        fresh6,
    );
    if !(Status >= 0) {
        RtlAssert(
            b"NT_SUCCESS(Status)\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"drivers/storage/ide/pciidex/fdo.c\0"
                as *const u8 as *const i8 as *mut core::ffi::c_void,
            237,
            core::ptr::null_mut::<i8>(),
        );
    }

    RtlInitUnicodeString(
        &raw mut DeviceName,
        DeviceNameBuffer.as_ptr() as *const u16,
    );

    Status = IoCreateDevice(
        (*(*FdoExtension).Common.Self_).DriverObject as PDRIVER_OBJECT,
        core::mem::size_of::<_PDO_DEVICE_EXTENSION>() as u32,
        &raw mut DeviceName,
        FILE_DEVICE_CONTROLLER as u32,
        crate::glue::FILE_DEVICE_SECURE_OPEN as u32,
        0u8,
        &raw mut Pdo,
    );
    if !(Status >= 0) {
        if DbgPrint(
            b"(%s:%d) Failed to create PDO 0x%lx\n\0" as *const u8 as *const i8,
            b"drivers/storage/ide/pciidex/fdo.c\0"
                as *const u8 as *const i8,
            249i32,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                b"drivers/storage/ide/pciidex/fdo.c\0"
                    as *const u8 as *const i8,
                249i32,
            );
        }
        return core::ptr::null_mut();
    }

    Alignment = (*FdoExtension).Properties.AlignmentRequirement;
    Alignment = core::cmp::max(
        Alignment,
        (*(*FdoExtension).Common.Self_).AlignmentRequirement,
    );
    Alignment = core::cmp::max(Alignment, 1u32);
    (*Pdo).AlignmentRequirement = Alignment;

    PdoExtension = (*Pdo).DeviceExtension as *mut _PDO_DEVICE_EXTENSION;

    core::ptr::write_bytes(
        PdoExtension as *mut u8,
        0u8,
        core::mem::size_of::<_PDO_DEVICE_EXTENSION>(),
    );
    (*PdoExtension).Common.Self_ = Pdo;
    (*PdoExtension).Channel = ChannelNumber;
    (*PdoExtension).ParentController = FdoExtension;

    (*Pdo).Flags &= !(DO_DEVICE_INITIALIZING as u32);
    PdoExtension
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoQueryBusRelations(
    FdoExtension: *mut _FDO_DEVICE_EXTENSION,
    Irp: *mut _IRP,
) -> NTSTATUS {
    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8
                as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const core::ffi::c_char as PVOID,
                b"drivers/storage/ide/pciidex/fdo.c\0"
                    .as_ptr() as *const core::ffi::c_char as PVOID,
                284u32,
                core::ptr::null_mut::<core::ffi::c_char>(),
            );
        }
    }

    let DeviceRelations: *mut _DEVICE_RELATIONS = ExAllocatePoolWithTag(
        PagedPool,
        (core::mem::size_of::<_DEVICE_RELATIONS>()
            + (MAX_IDE_CHANNEL as usize - 1) * core::mem::size_of::<PDEVICE_OBJECT>())
            as SIZE_T,
        TAG_PCIIDEX,
    ) as *mut _DEVICE_RELATIONS;

    if DeviceRelations.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*DeviceRelations).Count = 0u32;

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*FdoExtension).DeviceSyncMutex));

    for i in 0..MAX_IDE_CHANNEL {
        let mut PdoExtension: *mut _PDO_DEVICE_EXTENSION = (*FdoExtension).Channels[i as usize];

        let ChannelState = PciIdeXChannelState(FdoExtension, i);

        if ChannelState as u32 == _IDE_CHANNEL_STATE_ChannelDisabled as u32 {
            if !PdoExtension.is_null() {
                (*PdoExtension).ReportedMissing = 1u8; // TRUE
            }
            continue;
        }

        if PdoExtension.is_null() {
            PdoExtension = PciIdeXPdoCreateDevice(FdoExtension, i);
            (*FdoExtension).Channels[i as usize] = PdoExtension;
        }

        if !PdoExtension.is_null() && (*PdoExtension).ReportedMissing == 0 {
            let fresh4 = (*DeviceRelations).Count;
            (*DeviceRelations).Count = (*DeviceRelations).Count.wrapping_add(1);
            *((*DeviceRelations).Objects.as_mut_ptr().offset(fresh4 as isize)) =
                (*PdoExtension).Common.Self_;
            ObReferenceObject((*PdoExtension).Common.Self_ as PVOID);
        }
    }

    ExReleaseFastMutex(core::ptr::addr_of_mut!((*FdoExtension).DeviceSyncMutex));

    (*Irp).IoStatus.Information = (DeviceRelations as ULONG_PTR) as _;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoQueryDeviceUsageNotification(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() as core::ffi::c_int > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const core::ffi::c_char,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const core::ffi::c_char as PVOID,
            b"drivers/storage/ide/pciidex/fdo.c\0"
                .as_ptr() as *const core::ffi::c_char as PVOID,
            346 as ULONG,
            core::ptr::null_mut::<core::ffi::c_char>(),
        );
    }

    if IoForwardIrpSynchronously((*FdoExtension).Ldo, Irp) == 0 {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const core::ffi::c_char,
            b"drivers/storage/ide/pciidex/fdo.c\0"
                .as_ptr() as *const core::ffi::c_char,
            348 as core::ffi::c_int,
            b"IoForwardIrpSynchronously(FdoExtension->Ldo, Irp)\0".as_ptr() as *const core::ffi::c_char,
        );
        __int2c();
        return STATUS_UNSUCCESSFUL;
    }

    let Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    if Status < 0 {
        return Status;
    }

    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let Counter: *mut LONG = match (*IoStack).Parameters.UsageNotification.Type as core::ffi::c_uint {
        1 => &raw mut (*FdoExtension).Common.PageFiles,
        2 => &raw mut (*FdoExtension).Common.HibernateFiles,
        3 => &raw mut (*FdoExtension).Common.DumpFiles,
        _ => return Status,
    };

    if (*IoStack).Parameters.UsageNotification.InPath != 0 {
        _InterlockedIncrement(Counter as *mut core::ffi::c_long);
    } else {
        _InterlockedDecrement(Counter as *mut core::ffi::c_long);
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoQueryInterface(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/storage/ide/pciidex/fdo.c\0"
                    .as_ptr() as *const i8 as PVOID,
                389 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    if ((*IoStack).Parameters.QueryInterface.InterfaceType as *const LONGLONG).read()
        == (core::ptr::addr_of!(GUID_TRANSLATOR_INTERFACE_STANDARD) as *const LONGLONG).read()
        && ((*IoStack).Parameters.QueryInterface.InterfaceType as *const LONGLONG)
            .offset(1)
            .read()
            == (core::ptr::addr_of!(GUID_TRANSLATOR_INTERFACE_STANDARD) as *const LONGLONG)
                .offset(1)
                .read()
    {
        let ResourceType: CM_RESOURCE_TYPE;
        let mut BusNumber: ULONG = 0;

        ResourceType = (*IoStack).Parameters.QueryInterface.InterfaceSpecificData as *const c_void
            as usize as CM_RESOURCE_TYPE;

        if (*FdoExtension).InNativeMode as i32 != 0
            || ResourceType != (CmResourceTypeInterrupt) as i32
            || ((*IoStack).Parameters.QueryInterface.Size as usize)
                < core::mem::size_of::<TRANSLATOR_INTERFACE>()
        {
            return STATUS_NOT_SUPPORTED;
        }

        return HalGetInterruptTranslator(
            PCIBus,
            0 as ULONG,
            InterfaceTypeUndefined,
            core::mem::size_of::<TRANSLATOR_INTERFACE>() as USHORT,
            (*IoStack).Parameters.QueryInterface.Version,
            (*IoStack).Parameters.QueryInterface.Interface as *mut core::ffi::c_void,
            &raw mut BusNumber,
        );
    }

    return STATUS_NOT_SUPPORTED;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoDispatchPnp(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let mut Status: NTSTATUS;
    match (*IoStack).MinorFunction {
        IRP_MN_START_DEVICE => {
            Status = PciIdeXFdoStartDevice(FdoExtension, Irp);
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return Status;
        }
        IRP_MN_STOP_DEVICE => {
            PciIdeXFdoStopDevice(FdoExtension);
        }
        IRP_MN_REMOVE_DEVICE => {
            return PciIdeXFdoRemoveDevice(FdoExtension, Irp);
        }
        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            PciIdeXFdoQueryPnpDeviceState(FdoExtension, Irp);
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            if (*IoStack).Parameters.QueryDeviceRelations.Type != BusRelations {
                // Falls through to IoCallDriver below
            } else {
                Status = PciIdeXFdoQueryBusRelations(FdoExtension, Irp);
                if !NT_SUCCESS(Status) {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                    IoCompleteRequest(Irp, IO_NO_INCREMENT);
                    return Status;
                }
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            }
        }
        IRP_MN_DEVICE_USAGE_NOTIFICATION => {
            PciIdeXFdoQueryDeviceUsageNotification(FdoExtension, Irp);
        }
        IRP_MN_QUERY_INTERFACE => {
            Status = PciIdeXFdoQueryInterface(FdoExtension, IoStack);
            if Status != STATUS_NOT_SUPPORTED {
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            }
        }
        IRP_MN_QUERY_STOP_DEVICE
        | IRP_MN_QUERY_REMOVE_DEVICE
        | IRP_MN_SURPRISE_REMOVAL
        | IRP_MN_CANCEL_STOP_DEVICE
        | IRP_MN_CANCEL_REMOVE_DEVICE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        }
        _ => {}
    }
    IoSkipCurrentIrpStackLocation(Irp);
    IoCallDriver((*FdoExtension).Ldo, Irp)
}
