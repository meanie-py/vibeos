// ---- crate-internal glue (private structs, no-op DDK shims, inlines) ----
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql; // FORCEINLINE on amd64/arm64

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{IoGetDriverObjectExtension, PCCHAR, va_list};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{GUID, PCSTR, RtlAssert, _GUID};
use ntkrnl_sys::driver_ddk::{DbgPrint};

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

pub static mut PciIdeDebug: ULONG = 0 as ULONG;
#[no_mangle]
pub unsafe extern "C" fn PciIdeXStartMiniport(mut FdoExtension: PFDO_DEVICE_EXTENSION) -> NTSTATUS {
    let mut DriverExtension: PPCIIDEX_DRIVER_EXTENSION;
    let mut Status: NTSTATUS;

    if KeGetCurrentIrql() as u32 > (APC_LEVEL) as u32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/storage/ide/pciidex/miniport.c\0"
                    as *const u8 as *const i8 as PVOID,
                24 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    if (*FdoExtension).MiniportStarted != 0 {
        return STATUS_SUCCESS;
    }

    DriverExtension = IoGetDriverObjectExtension(
        (*FdoExtension).DriverObject,
        (*FdoExtension).DriverObject as PVOID,
    ) as PPCIIDEX_DRIVER_EXTENSION;

    if DriverExtension.is_null() {
        RtlAssert(
            b"DriverExtension\0" as *const u8 as *const i8 as PVOID,
            b"drivers/storage/ide/pciidex/miniport.c\0"
                as *const u8 as *const i8 as PVOID,
            33 as ULONG,
            core::ptr::null_mut::<i8>(),
        );
    }

    (*FdoExtension).Properties.Size = core::mem::size_of::<IDE_CONTROLLER_PROPERTIES>() as ULONG;
    (*FdoExtension).Properties.ExtensionSize = (*DriverExtension).MiniControllerExtensionSize;

    Status = (*DriverExtension)
        .HwGetControllerProperties
        .expect("non-null function pointer")(
        &raw mut (*FdoExtension).MiniControllerExtension as PVOID,
        &raw mut (*FdoExtension).Properties,
    );

    if !(Status >= 0) {
        return Status;
    }

    (*FdoExtension).MiniportStarted = 1u8;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXChannelState(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    Channel: ULONG,
) -> IDE_CHANNEL_STATE {
    if KeGetCurrentIrql() as ::core::ffi::c_int > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/storage/ide/pciidex/miniport.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                54 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }
    let MiniportChannelEnabled: PCIIDE_CHANNEL_ENABLED =
        (*FdoExtension).Properties.PciIdeChannelEnabled;
    if MiniportChannelEnabled.is_some() {
        return MiniportChannelEnabled.expect("non-null function pointer")(
            (*FdoExtension).MiniControllerExtension.as_mut_ptr() as *mut core::ffi::c_void,
            Channel,
        );
    }
    IDE_CHANNEL_STATE_ChannelStateUnknown
}

// PciIdeXDebugPrint — an EXPORT (pciidex.spec: `@ varargs PciIdeXDebugPrint(long str)`;
// uniata.sys / pciide.sys import it). The variadic ENTRY that keeps that exact symbol name
// and cdecl-varargs ABI is in drivers/storage/ide/pciidex/rust_varargs_shim.c, because
// stable Rust cannot `va_start` (c_variadic, rust-lang#44930). It calls straight back into
// this fixed-arity twin, which keeps the level filter and the vDbgPrintEx dispatch; `ap` is
// a pointer to the shim frame's va_list, forwarded to vDbgPrintEx by `vibeos_vDbgPrintEx`
// (Rust cannot pass a va_list by value).
#[no_mangle]
pub unsafe extern "C" fn PciIdeXDebugPrint_v(
    mut DebugPrintLevel: ULONG,
    DebugMessage: PCCHAR,
    ap: *mut core::ffi::c_void,
) {
    if DebugPrintLevel <= PciIdeDebug {
        DebugPrintLevel = 0 as ULONG;
    }
    vibeos_vDbgPrintEx(DPFLTR_PCIIDE_ID as ULONG, DebugPrintLevel, DebugMessage, ap);
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXGetBusData(
    DeviceExtension: PVOID,
    Buffer: PVOID,
    ConfigDataOffset: ULONG,
    BufferLength: ULONG,
) -> NTSTATUS {
    let fdo_extension: PFDO_DEVICE_EXTENSION = (DeviceExtension as usize)
        .wrapping_sub(
            core::mem::offset_of!(FDO_DEVICE_EXTENSION, MiniControllerExtension),
        ) as PFDO_DEVICE_EXTENSION;

    let bytes_read = ((*fdo_extension).BusInterface.GetBusData).unwrap()(
        (*fdo_extension).BusInterface.Context,
        PCI_WHICHSPACE_CONFIG as ULONG,
        Buffer,
        ConfigDataOffset,
        BufferLength,
    );
    if bytes_read != BufferLength {
        return STATUS_UNSUCCESSFUL;
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXSetBusData(
    DeviceExtension: PVOID,
    Buffer: PVOID,
    DataMask: PVOID,
    ConfigDataOffset: u32,
    BufferLength: u32,
) -> NTSTATUS {
    let CurrentBuffer = ExAllocatePoolWithTag(NonPagedPool, BufferLength as usize, TAG_PCIIDEX as u32) as *mut u8;
    if CurrentBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let FdoExtension = (DeviceExtension as usize).wrapping_sub(
        core::mem::offset_of!(_FDO_DEVICE_EXTENSION, MiniControllerExtension),
    ) as *mut _FDO_DEVICE_EXTENSION;

    let mut OldIrql: u8 = 0;

    KeAcquireSpinLock((core::ptr::addr_of_mut!((*FdoExtension).BusDataLock)).cast(), &mut OldIrql);

    let mut Status = PciIdeXGetBusData(DeviceExtension, Buffer, ConfigDataOffset, BufferLength);
    if !NT_SUCCESS(Status) {
        KeReleaseSpinLock((core::ptr::addr_of_mut!((*FdoExtension).BusDataLock)).cast(), OldIrql);
        ExFreePoolWithTag(CurrentBuffer as PVOID, TAG_PCIIDEX as u32);
        return Status;
    }

    let mask_bytes = DataMask as *mut u8;
    let buffer_bytes = Buffer as *mut u8;
    for i in 0..BufferLength {
        let i = i as usize;
        let mask_byte = *mask_bytes.add(i);
        let buf_byte = *buffer_bytes.add(i);
        *CurrentBuffer.add(i) = (*CurrentBuffer.add(i) & !mask_byte) | (mask_byte & buf_byte);
    }

    let set_bus_data = (*FdoExtension).BusInterface.SetBusData;
    let BytesWritten = set_bus_data.unwrap()(
        (*FdoExtension).BusInterface.Context,
        PCI_WHICHSPACE_CONFIG as u32,
        CurrentBuffer as PVOID,
        ConfigDataOffset,
        BufferLength,
    );

    if BytesWritten != BufferLength {
        Status = STATUS_UNSUCCESSFUL;
    } else {
        Status = STATUS_SUCCESS;
    }

    KeReleaseSpinLock((core::ptr::addr_of_mut!((*FdoExtension).BusDataLock)).cast(), OldIrql);
    ExFreePoolWithTag(CurrentBuffer as PVOID, TAG_PCIIDEX as u32);
    Status
}
