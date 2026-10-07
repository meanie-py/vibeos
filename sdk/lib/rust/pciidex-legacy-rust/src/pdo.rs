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
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_RESOURCE_LIST, DEVICE_CAPABILITIES, DEVICE_RELATIONS, ExAcquireFastMutex, ExReleaseFastMutex, _KWAIT_REASON_Executive, IO_RESOURCE_DESCRIPTOR, IO_RESOURCE_REQUIREMENTS_LIST, IO_STACK_LOCATION, IoAllocateIrp, IoDeleteDevice, IoFreeIrp, IoGetAttachedDeviceReference, IoInvalidateDeviceState, KAFFINITY, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PIO_RESOURCE_DESCRIPTOR, PIO_RESOURCE_REQUIREMENTS_LIST, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{GUID, NTSTRSAFE_PCWSTR, NTSTRSAFE_PWSTR, PCSTR, RtlAssert, _DEVICE_CAPABILITIES, _GUID, _LARGE_INTEGER};

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

pub const NDEBUG: u32 = 1;
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoStartDevice(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    ResourceList: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let mut IoBase: PUCHAR = core::ptr::null_mut();

    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/storage/ide/pciidex/pdo.c\0"
                .as_ptr() as *const i8 as PVOID,
            23 as ULONG,
            core::ptr::null_mut::<i8>(),
        );
    }

    IoBase = (*(*PdoExtension).ParentController).BusMasterPortBase;

    if (*PdoExtension).Channel != 0 {
        IoBase = IoBase.offset(BM_SECONDARY_CHANNEL_OFFSET as isize);
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoStopDevice(PdoExtension: PPDO_DEVICE_EXTENSION) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/storage/ide/pciidex/pdo.c\0"
                    .as_ptr() as *const i8 as PVOID,
                41u32 as ULONG,
                core::ptr::null_mut::<core::ffi::c_char>(),
            );
        }
    }
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoRemoveDevice(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    FinalRemove: BOOLEAN,
) -> NTSTATUS {
    if KeGetCurrentIrql() as u32 > APC_LEVEL as u32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            core::ptr::null_mut::<core::ffi::c_void>(),
            b"pdo.c\0".as_ptr() as *mut core::ffi::c_void,
            56u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    let FdoExtension: PFDO_DEVICE_EXTENSION = (*PdoExtension).ParentController;
    if FinalRemove != 0 && (*PdoExtension).ReportedMissing != 0 {
        ExAcquireFastMutex(core::ptr::addr_of_mut!((*FdoExtension).DeviceSyncMutex));
        let mut i: u32 = 0;
        while i < MAX_IDE_CHANNEL as u32 {
            if (*FdoExtension).Channels[i as usize] == PdoExtension {
                (*FdoExtension).Channels[i as usize] = core::ptr::null_mut::<_PDO_DEVICE_EXTENSION>();
                break;
            }
            i += 1;
        }
        ExReleaseFastMutex(core::ptr::addr_of_mut!((*FdoExtension).DeviceSyncMutex));
        IoDeleteDevice((*PdoExtension).Common.Self_);
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoQueryStopRemoveDevice(
    PdoExtension: PPDO_DEVICE_EXTENSION,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as PVOID,
                b"drivers/storage/ide/pciidex/pdo.c\0"
                    as *const u8 as PVOID,
                85 as ULONG,
                core::ptr::null_mut::<core::ffi::c_char>(),
            );
        }
    }
    if (*PdoExtension).Common.PageFiles != 0
        || (*PdoExtension).Common.HibernateFiles != 0
        || (*PdoExtension).Common.DumpFiles != 0
    {
        return STATUS_DEVICE_BUSY;
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoQueryTargetDeviceRelations(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if false {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/storage/ide/pciidex/pdo.c\0"
                    as *const u8 as *const i8 as PVOID,
                106 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    let DeviceRelations: PDEVICE_RELATIONS = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<DEVICE_RELATIONS>() as SIZE_T,
        TAG_PCIIDEX as ULONG,
    ) as PDEVICE_RELATIONS;

    if DeviceRelations.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*DeviceRelations).Count = 1 as ULONG;
    (*DeviceRelations).Objects[0] = (*PdoExtension).Common.Self_;
    ObReferenceObject((*PdoExtension).Common.Self_ as PVOID);

    (*Irp).IoStatus.Information = (DeviceRelations as ULONG_PTR) as _;
    STATUS_SUCCESS
}

unsafe extern "system" fn PciIdeXOnRepeaterCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    let _ = DeviceObject;
    if (*Irp).PendingReturned != 0 {
        (KeSetEvent)(Context as *mut core::ffi::c_void, IO_NO_INCREMENT as i32, 0u8);
    }
    STATUS_MORE_PROCESSING_REQUIRED
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoRepeatRequest(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
    DeviceCapabilities: PDEVICE_CAPABILITIES,
) -> NTSTATUS {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            (KeGetCurrentIrql()) as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as PVOID,
                b"drivers/storage/ide/pciidex/pdo.c\0"
                    as *const u8 as PVOID,
                154 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }
    let Fdo: PDEVICE_OBJECT = (*(*PdoExtension).ParentController).Common.Self_;
    let TopDeviceObject: PDEVICE_OBJECT = IoGetAttachedDeviceReference(Fdo);
    let SubIrp: PIRP = IoAllocateIrp((*TopDeviceObject).StackSize, 0u8);
    if SubIrp.is_null() {
        ObDereferenceObject(TopDeviceObject as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let mut Event: KEVENT = core::mem::zeroed();
    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let SubStack: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(SubIrp);
    core::ptr::copy_nonoverlapping(
        IoStack as *const IO_STACK_LOCATION,
        SubStack as *mut IO_STACK_LOCATION,
        1,
    );
    if !DeviceCapabilities.is_null() {
        (*SubStack).Parameters.DeviceCapabilities.Capabilities = DeviceCapabilities;
    }
    IoSetCompletionRoutine(
        SubIrp,
        Some(core::mem::transmute(PciIdeXOnRepeaterCompletion as *const ())),
        core::ptr::addr_of_mut!(Event) as PVOID,
        1u8,
        1u8,
        1u8,
    );
    (*SubIrp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    let mut Status: NTSTATUS = IoCallDriver(TopDeviceObject, SubIrp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as i8,
            0u8,
            core::ptr::null_mut::<_LARGE_INTEGER>(),
        );
    }
    ObDereferenceObject(TopDeviceObject as PVOID);
    Status = (*SubIrp).IoStatus.__bindgen_anon_1.Status;
    IoFreeIrp(SubIrp);
    Status
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoQueryCapabilities(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() as u32 > 1 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8).cast(),
            (KeGetCurrentIrql()) as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/storage/ide/pciidex/pdo.c\0"
                .as_ptr() as *const i8 as PVOID,
            210,
            core::ptr::null_mut::<i8>(),
        );
    }

    let mut ParentCapabilities: DEVICE_CAPABILITIES = core::mem::zeroed();
    core::ptr::write_bytes(
        &raw mut ParentCapabilities as *mut u8,
        0u8,
        core::mem::size_of::<DEVICE_CAPABILITIES>(),
    );
    ParentCapabilities.Size = core::mem::size_of::<DEVICE_CAPABILITIES>() as u16;
    ParentCapabilities.Version = 1u16;
    ParentCapabilities.Address = MAXULONG;
    ParentCapabilities.UINumber = MAXULONG;

    let Status = PciIdeXPdoRepeatRequest(PdoExtension, Irp, &raw mut ParentCapabilities);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let DeviceCapabilities = (*IoStack).Parameters.DeviceCapabilities.Capabilities;
    core::ptr::copy_nonoverlapping(
        &raw const ParentCapabilities as *const DEVICE_CAPABILITIES,
        DeviceCapabilities,
        1,
    );
    (*DeviceCapabilities).set_UniqueID(0u32);
    (*DeviceCapabilities).Address = (*PdoExtension).Channel;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoQueryPnpDeviceState(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0"
                .as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as PVOID,
            b"drivers/storage/ide/pciidex/pdo.c\0"
                .as_ptr() as PVOID,
            240 as ULONG,
            core::ptr::null_mut::<i8>(),
        );
    }
    if (*PdoExtension).Common.PageFiles != 0
        || (*PdoExtension).Common.HibernateFiles != 0
        || (*PdoExtension).Common.DumpFiles != 0
    {
        (*Irp).IoStatus.Information |= PNP_DEVICE_NOT_DISABLEABLE as ntkrnl_sys::ddk::ULONG_PTR;
    }
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoQueryResources(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/storage/ide/pciidex/pdo.c\0"
                .as_ptr() as *const i8 as PVOID,
            266,
            core::ptr::null_mut(),
        );
    }

    let FdoExtension: PFDO_DEVICE_EXTENSION = (*PdoExtension).ParentController;
    if (*FdoExtension).InNativeMode != 0 {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    let ChannelState: IDE_CHANNEL_STATE = PciIdeXChannelState(FdoExtension, (*PdoExtension).Channel);
    if ChannelState == _IDE_CHANNEL_STATE_ChannelDisabled {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    let ListSize: ULONG = (core::mem::size_of::<CM_RESOURCE_LIST>()
        + core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>()
            * (PCIIDE_LEGACY_RESOURCE_COUNT as usize - 1)) as ULONG;
    let ResourceList: PCM_RESOURCE_LIST =
        ExAllocatePoolZero(PagedPool, (ListSize as SIZE_T) as _, TAG_PCIIDEX as ULONG) as _;
    if ResourceList.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*ResourceList).Count = 1;
    let list0 = core::ptr::addr_of_mut!((*ResourceList).List[0]);

    (*list0).InterfaceType = (crate::glue::Isa) as i32;
    (*list0).PartialResourceList.Version = 1;
    (*list0).PartialResourceList.Revision = 1;
    (*list0).PartialResourceList.Count = PCIIDE_LEGACY_RESOURCE_COUNT as ULONG;

    let (CommandPortBase, ControlPortBase, InterruptVector) = if (*PdoExtension).Channel == 0 {
        (
            PCIIDE_LEGACY_PRIMARY_COMMAND_BASE as ULONG,
            PCIIDE_LEGACY_PRIMARY_CONTROL_BASE as ULONG,
            PCIIDE_LEGACY_PRIMARY_IRQ as ULONG,
        )
    } else {
        (
            PCIIDE_LEGACY_SECONDARY_COMMAND_BASE as ULONG,
            PCIIDE_LEGACY_SECONDARY_CONTROL_BASE as ULONG,
            PCIIDE_LEGACY_SECONDARY_IRQ as ULONG,
        )
    };

    let mut Descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
        core::ptr::addr_of_mut!((*list0).PartialResourceList.PartialDescriptors[0]) as *mut _;

    // Command port
    (*Descriptor).Type = CmResourceTypePort as UCHAR;
    (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive as UCHAR;
    (*Descriptor).Flags = (CM_RESOURCE_PORT_IO | CM_RESOURCE_PORT_16_BIT_DECODE) as USHORT;
    (*Descriptor).u.Port.Length = PCIIDE_LEGACY_COMMAND_IO_RANGE_LENGTH as ULONG;
    (*Descriptor).u.Port.Start.__bindgen_anon_1.LowPart = CommandPortBase;
    Descriptor = Descriptor.offset(1);

    // Control port
    (*Descriptor).Type = CmResourceTypePort as UCHAR;
    (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive as UCHAR;
    (*Descriptor).Flags = (CM_RESOURCE_PORT_IO | CM_RESOURCE_PORT_16_BIT_DECODE) as USHORT;
    (*Descriptor).u.Port.Length = PCIIDE_LEGACY_CONTROL_IO_RANGE_LENGTH as ULONG;
    (*Descriptor).u.Port.Start.__bindgen_anon_1.LowPart = ControlPortBase;
    Descriptor = Descriptor.offset(1);

    // Interrupt
    (*Descriptor).Type = CmResourceTypeInterrupt as UCHAR;
    (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive as UCHAR;
    (*Descriptor).Flags = CM_RESOURCE_INTERRUPT_LATCHED as USHORT;
    (*Descriptor).u.Interrupt.Level = InterruptVector;
    (*Descriptor).u.Interrupt.Vector = InterruptVector;
    (*Descriptor).u.Interrupt.Affinity = (-1i32) as KAFFINITY;

    (*Irp).IoStatus.Information = (ResourceList as ULONG_PTR) as _;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoQueryResourceRequirements(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if (KeGetCurrentIrql() as u32) > (APC_LEVEL as u32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/storage/ide/pciidex/pdo.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                346 as u32,
                core::ptr::null_mut(),
            );
        }
    }

    let FdoExtension = (*PdoExtension).ParentController;
    if (*FdoExtension).InNativeMode != 0 {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    let ChannelState = PciIdeXChannelState(FdoExtension, (*PdoExtension).Channel);
    if ChannelState == ChannelDisabled {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    let ListSize: ULONG = (core::mem::size_of::<IO_RESOURCE_REQUIREMENTS_LIST>() as ULONG)
        + (core::mem::size_of::<IO_RESOURCE_DESCRIPTOR>() as ULONG)
            * (PCIIDE_LEGACY_RESOURCE_COUNT - 1) as ULONG;
    let mut RequirementsList = ExAllocatePoolZero(
        PagedPool,
        (ListSize as usize) as _,
        TAG_PCIIDEX,
    ) as PIO_RESOURCE_REQUIREMENTS_LIST;
    if RequirementsList.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*RequirementsList).InterfaceType = (crate::glue::Isa) as i32;
    (*RequirementsList).ListSize = ListSize;
    (*RequirementsList).AlternativeLists = 1;

    let list = &mut (*RequirementsList).List[0];
    list.Version = 1 as u16;
    list.Revision = 1 as u16;
    list.Count = PCIIDE_LEGACY_RESOURCE_COUNT;

    let mut CommandPortBase: ULONG;
    let mut ControlPortBase: ULONG;
    let mut InterruptVector: ULONG;
    if (*PdoExtension).Channel == 0 {
        CommandPortBase = PCIIDE_LEGACY_PRIMARY_COMMAND_BASE;
        ControlPortBase = PCIIDE_LEGACY_PRIMARY_CONTROL_BASE;
        InterruptVector = PCIIDE_LEGACY_PRIMARY_IRQ;
    } else {
        CommandPortBase = PCIIDE_LEGACY_SECONDARY_COMMAND_BASE;
        ControlPortBase = PCIIDE_LEGACY_SECONDARY_CONTROL_BASE;
        InterruptVector = PCIIDE_LEGACY_SECONDARY_IRQ;
    }

    let descriptors_ptr =
        &raw mut list.Descriptors as *mut IO_RESOURCE_DESCRIPTOR;
    let mut Descriptor = descriptors_ptr;

    // Command port base
    (*Descriptor).Type = CmResourceTypePort as u8;
    (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
    (*Descriptor).Flags = (CM_RESOURCE_PORT_IO | CM_RESOURCE_PORT_16_BIT_DECODE) as u16;
    (*Descriptor).u.Port.Length = PCIIDE_LEGACY_COMMAND_IO_RANGE_LENGTH;
    (*Descriptor).u.Port.Alignment = 1;
    (*Descriptor).u.Port.MinimumAddress.__bindgen_anon_1.LowPart = CommandPortBase;
    (*Descriptor).u.Port.MaximumAddress.__bindgen_anon_1.LowPart =
        CommandPortBase + PCIIDE_LEGACY_COMMAND_IO_RANGE_LENGTH - 1;
    Descriptor = Descriptor.offset(1);

    // Control port base
    (*Descriptor).Type = CmResourceTypePort as u8;
    (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
    (*Descriptor).Flags = (CM_RESOURCE_PORT_IO | CM_RESOURCE_PORT_16_BIT_DECODE) as u16;
    (*Descriptor).u.Port.Length = PCIIDE_LEGACY_CONTROL_IO_RANGE_LENGTH;
    (*Descriptor).u.Port.Alignment = 1;
    (*Descriptor).u.Port.MinimumAddress.__bindgen_anon_1.LowPart = ControlPortBase;
    (*Descriptor).u.Port.MaximumAddress.__bindgen_anon_1.LowPart =
        ControlPortBase + PCIIDE_LEGACY_CONTROL_IO_RANGE_LENGTH - 1;
    Descriptor = Descriptor.offset(1);

    // Interrupt
    (*Descriptor).Type = CmResourceTypeInterrupt as u8;
    (*Descriptor).ShareDisposition = CmResourceShareShared as u8;
    (*Descriptor).Flags = CM_RESOURCE_INTERRUPT_LATCHED as u16;
    (*Descriptor).u.Interrupt.MinimumVector = InterruptVector;
    (*Descriptor).u.Interrupt.MaximumVector = InterruptVector;

    (*Irp).IoStatus.Information = (RequirementsList as ULONG_PTR) as _;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXGetControllerVendorId(FdoExtension: PFDO_DEVICE_EXTENSION) -> PCWSTR {
    // PAGED_CODE();
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as PVOID,
            b"drivers/storage/ide/pciidex/pdo.c\0"
                .as_ptr() as *const i8 as PVOID,
            424u32,
            core::ptr::null_mut(),
        );
    }

    static COMPAQ: [u16; 7] = [0x43, 0x6f, 0x6d, 0x70, 0x61, 0x71, 0x0000];
    static SIS: [u16; 4] = [0x53, 0x69, 0x53, 0x0000];
    static WINBOND: [u16; 8] = [0x57, 0x69, 0x6e, 0x42, 0x6f, 0x6e, 0x64, 0x0000];
    static CMD: [u16; 4] = [0x43, 0x4d, 0x44, 0x0000];
    static ALI: [u16; 4] = [0x41, 0x4c, 0x69, 0x0000];
    static INTEL: [u16; 6] = [0x49, 0x6e, 0x74, 0x65, 0x6c, 0x0000];

    match (*FdoExtension).VendorId as i32 {
        0x0E11 => return &COMPAQ as *const u16 as PCWSTR,
        0x1039 => return &SIS as *const u16 as PCWSTR,
        0x1050 => return &WINBOND as *const u16 as PCWSTR,
        0x1095 => return &CMD as *const u16 as PCWSTR,
        0x10B9 => return &ALI as *const u16 as PCWSTR,
        0x8086 => return &INTEL as *const u16 as PCWSTR,
        _ => {}
    }

    core::ptr::null::<u16>()
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXGetControllerDeviceId(FdoExtension: PFDO_DEVICE_EXTENSION) -> PCWSTR {
    /* PAGED_CODE() */
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/storage/ide/pciidex/pdo.c\0"
                    as *const u8
                    as *const i8 as PVOID,
                455 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    if (*FdoExtension).VendorId as i32 == 0x8086i32 {
        match (*FdoExtension).DeviceId as i32 {
            0x1230 => {
                return w!("PIIX").as_ptr() as PCWSTR;
            }
            0x7010 => {
                return w!("PIIX3").as_ptr() as PCWSTR;
            }
            0x7111 => {
                return w!("PIIX4").as_ptr() as PCWSTR;
            }
            _ => {}
        }
    }

    core::ptr::null_mut::<u16>()
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoQueryId(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS = 0;
    let Buffer: PWCHAR;
    let mut End: PWCHAR = core::ptr::null_mut();
    let mut CharCount: usize;
    let mut Remaining: usize = 0;

    static IDECOMPATIBLE_ID: [u16; 9] = [
        '*' as u16,
        'P' as u16,
        'N' as u16,
        'P' as u16,
        '0' as u16,
        '6' as u16,
        '0' as u16,
        '0' as u16,
        0u16,
    ];

    // PAGED_CODE() expanded (c2rust)
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/storage/ide/pciidex/pdo.c\0"
                    .as_ptr() as *const i8 as PVOID,
                490u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    match (*IoStack).Parameters.QueryId.IdType {
        BusQueryDeviceID => {
            static PCI_IDE_DEVICE_ID: [u16; 18] = [
                'P' as u16,
                'C' as u16,
                'I' as u16,
                'I' as u16,
                'D' as u16,
                'E' as u16,
                '\\' as u16,
                'I' as u16,
                'D' as u16,
                'E' as u16,
                'C' as u16,
                'h' as u16,
                'a' as u16,
                'n' as u16,
                'n' as u16,
                'e' as u16,
                'l' as u16,
                0u16,
            ];

            Buffer = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<[u16; 18]>() as SIZE_T,
                TAG_PCIIDEX as u32,
            ) as PWCHAR;

            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            core::ptr::copy_nonoverlapping(
                PCI_IDE_DEVICE_ID.as_ptr() as *const core::ffi::c_void,
                Buffer as *mut core::ffi::c_void,
                core::mem::size_of::<[u16; 18]>(),
            );

            // DPRINT removed
        }

        BusQueryHardwareIDs => {
            let FdoExtension: PFDO_DEVICE_EXTENSION;
            let VendorString: PCWSTR;
            let mut IdStart: PWCHAR = core::ptr::null_mut::<u16>();

            DBG_UNREFERENCED_LOCAL_VARIABLE(&mut IdStart);

            // Maximum string length in WCHAR units (c2rust arithmetic preserved)
            CharCount = (core::mem::size_of::<[i8; 13]>()
                .wrapping_add(core::mem::size_of::<[i8; 22]>())
                .wrapping_add(core::mem::size_of::<[u16; 9]>())
                .wrapping_add(core::mem::size_of::<i8>())) as usize;

            Buffer = ExAllocatePoolWithTag(
                PagedPool,
                (CharCount as SIZE_T).wrapping_mul(core::mem::size_of::<u16>() as SIZE_T),
                TAG_PCIIDEX as u32,
            ) as PWCHAR;

            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            FdoExtension = (*PdoExtension).ParentController;
            VendorString = PciIdeXGetControllerVendorId(FdoExtension);

            // DPRINT removed

            // ID 1
            if !VendorString.is_null() {
                let DeviceString: PCWSTR = PciIdeXGetControllerDeviceId(FdoExtension);
                if !DeviceString.is_null() {
                    static FMT_LS_HYPHEN_LS: [u16; 8] = [
                        '%' as u16,
                        'l' as u16,
                        's' as u16,
                        '-' as u16,
                        '%' as u16,
                        'l' as u16,
                        's' as u16,
                        0u16,
                    ];
                    Status = RtlStringCchPrintfExW(
                        Buffer as NTSTRSAFE_PWSTR,
                        CharCount,
                        &raw mut End,
                        &raw mut Remaining,
                        0u32,
                        FMT_LS_HYPHEN_LS.as_ptr() as NTSTRSAFE_PCWSTR,
                        VendorString,
                        DeviceString,
                    );
                } else {
                    static FMT_LS_HYPHEN_04X: [u16; 9] = [
                        '%' as u16,
                        'l' as u16,
                        's' as u16,
                        '-' as u16,
                        '%' as u16,
                        '0' as u16,
                        '4' as u16,
                        'x' as u16,
                        0u16,
                    ];
                    Status = RtlStringCchPrintfExW(
                        Buffer as NTSTRSAFE_PWSTR,
                        CharCount,
                        &raw mut End,
                        &raw mut Remaining,
                        0u32,
                        FMT_LS_HYPHEN_04X.as_ptr() as NTSTRSAFE_PCWSTR,
                        VendorString,
                        (*FdoExtension).DeviceId as i32,
                    );
                }
            } else {
                static FMT_04X_HYPHEN_04X: [u16; 10] = [
                    '%' as u16,
                    '0' as u16,
                    '4' as u16,
                    'x' as u16,
                    '-' as u16,
                    '%' as u16,
                    '0' as u16,
                    '4' as u16,
                    'x' as u16,
                    0u16,
                ];
                Status = RtlStringCchPrintfExW(
                    Buffer as NTSTRSAFE_PWSTR,
                    CharCount,
                    &raw mut End,
                    &raw mut Remaining,
                    0u32,
                    FMT_04X_HYPHEN_04X.as_ptr() as NTSTRSAFE_PCWSTR,
                    (*FdoExtension).VendorId as i32,
                    (*FdoExtension).DeviceId as i32,
                );
            }

            if !(Status >= 0i32) {
                RtlAssert(
                    b"NT_SUCCESS(Status)\0".as_ptr() as *const i8 as PVOID,
                    b"drivers/storage/ide/pciidex/pdo.c\0"
                        .as_ptr() as *const i8 as PVOID,
                    573u32,
                    core::ptr::null_mut::<i8>(),
                );
            }

            // DPRINT removed

            End = End.add(1);
            Remaining = Remaining.wrapping_sub(1);

            // ID 2
            IdStart = End;
            static PRIMARY_IDE_CHANNEL: [u16; 20] = [
                'P' as u16, 'r' as u16, 'i' as u16, 'm' as u16, 'a' as u16,
                'r' as u16, 'y' as u16, '_' as u16, 'I' as u16, 'D' as u16,
                'E' as u16, '_' as u16, 'C' as u16, 'h' as u16, 'a' as u16,
                'n' as u16, 'n' as u16, 'e' as u16, 'l' as u16, 0u16,
            ];
            static SECONDARY_IDE_CHANNEL: [u16; 22] = [
                'S' as u16, 'e' as u16, 'c' as u16, 'o' as u16, 'n' as u16,
                'd' as u16, 'a' as u16, 'r' as u16, 'y' as u16, '_' as u16,
                'I' as u16, 'D' as u16, 'E' as u16, '_' as u16, 'C' as u16,
                'h' as u16, 'a' as u16, 'n' as u16, 'n' as u16, 'e' as u16,
                'l' as u16, 0u16,
            ];
            static FMT_LS: [u16; 4] = [
                '%' as u16, 'l' as u16, 's' as u16, 0u16,
            ];

            Status = RtlStringCchPrintfExW(
                End as NTSTRSAFE_PWSTR,
                Remaining,
                &raw mut End,
                &raw mut Remaining,
                0u32,
                FMT_LS.as_ptr() as NTSTRSAFE_PCWSTR,
                if (*PdoExtension).Channel == 0u32 {
                    PRIMARY_IDE_CHANNEL.as_ptr()
                } else {
                    SECONDARY_IDE_CHANNEL.as_ptr()
                },
            );

            if !(Status >= 0i32) {
                RtlAssert(
                    b"NT_SUCCESS(Status)\0".as_ptr() as *const i8 as PVOID,
                    b"drivers/storage/ide/pciidex/pdo.c\0"
                        .as_ptr() as *const i8 as PVOID,
                    591u32,
                    core::ptr::null_mut::<i8>(),
                );
            }

            // DPRINT removed

            End = End.add(1);
            Remaining = Remaining.wrapping_sub(1);

            // ID 3
            IdStart = End;
            Status = RtlStringCchPrintfExW(
                End as NTSTRSAFE_PWSTR,
                Remaining,
                &raw mut End,
                &raw mut Remaining,
                0u32,
                FMT_LS.as_ptr() as NTSTRSAFE_PCWSTR,
                IDECOMPATIBLE_ID.as_ptr(),
            );

            if !(Status >= 0i32) {
                RtlAssert(
                    b"NT_SUCCESS(Status)\0".as_ptr() as *const i8 as PVOID,
                    b"drivers/storage/ide/pciidex/pdo.c\0"
                        .as_ptr() as *const i8 as PVOID,
                    607u32,
                    core::ptr::null_mut::<i8>(),
                );
            }

            // DPRINT removed

            End = End.add(1);
            *End = 0u16; // UNICODE_NULL (multi-string terminator)
        }

        BusQueryCompatibleIDs => {
            Buffer = ExAllocatePoolWithTag(
                PagedPool,
                (core::mem::size_of::<[u16; 9]>() as SIZE_T)
                    .wrapping_add(core::mem::size_of::<u16>() as SIZE_T),
                TAG_PCIIDEX as u32,
            ) as PWCHAR;

            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            core::ptr::copy_nonoverlapping(
                IDECOMPATIBLE_ID.as_ptr() as *const core::ffi::c_void,
                Buffer as *mut core::ffi::c_void,
                core::mem::size_of::<[u16; 9]>(),
            );

            // Write UNICODE_NULL after the string for multi-string termination
            *Buffer.add(
                (core::mem::size_of::<[u16; 9]>())
                    .wrapping_div(core::mem::size_of::<u16>()) as usize,
            ) = 0u16;

            // DPRINT removed
        }

        BusQueryInstanceID => {
            CharCount = core::mem::size_of::<[i8; 2]>() as usize;

            Buffer = ExAllocatePoolWithTag(
                PagedPool,
                (CharCount as SIZE_T).wrapping_mul(core::mem::size_of::<u16>() as SIZE_T),
                TAG_PCIIDEX as u32,
            ) as PWCHAR;

            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            static FMT_LU: [u16; 4] = [
                '%' as u16, 'l' as u16, 'u' as u16, 0u16,
            ];

            Status = RtlStringCchPrintfExW(
                Buffer as NTSTRSAFE_PWSTR,
                CharCount,
                core::ptr::null_mut::<NTSTRSAFE_PWSTR>(),
                core::ptr::null_mut::<usize>(),
                0u32,
                FMT_LU.as_ptr() as NTSTRSAFE_PCWSTR,
                (*PdoExtension).Channel,
            );

            if !(Status >= 0i32) {
                RtlAssert(
                    b"NT_SUCCESS(Status)\0".as_ptr() as *const i8 as PVOID,
                    b"drivers/storage/ide/pciidex/pdo.c\0"
                        .as_ptr() as *const i8 as PVOID,
                    648u32,
                    core::ptr::null_mut::<i8>(),
                );
            }

            // DPRINT removed
        }

        _ => {
            return (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    (*Irp).IoStatus.Information = (Buffer as ULONG_PTR) as _;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoQueryDeviceText(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/storage/ide/pciidex/pdo.c\0"
                    as *const u8 as *const i8 as PVOID,
                673 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let Buffer: PWCHAR;
    let Size: ULONG;
    match (*IoStack).Parameters.QueryDeviceText.DeviceTextType as u32 {
        1 => {
            static mut PrimaryChannelText: [WCHAR; 16] = unsafe {
                core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
                    *b"P\0r\0i\0m\0a\0r\0y\0 \0c\0h\0a\0n\0n\0e\0l\0\0\0",
                )
            };
            static mut SecondaryChannelText: [WCHAR; 18] = unsafe {
                core::mem::transmute::<[u8; 36], [WCHAR; 18]>(
                    *b"S\0e\0c\0o\0n\0d\0a\0r\0y\0 \0c\0h\0a\0n\0n\0e\0l\0\0\0",
                )
            };
            if (*PdoExtension).Channel == 0 {
                Size = core::mem::size_of::<[WCHAR; 16]>() as ULONG;
            } else {
                Size = core::mem::size_of::<[WCHAR; 18]>() as ULONG;
            }
            Buffer = ExAllocatePoolWithTag(PagedPool, Size as SIZE_T, TAG_PCIIDEX as ULONG) as PWCHAR;
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            let src: *const WCHAR = if (*PdoExtension).Channel == 0 {
                &raw const PrimaryChannelText as *const WCHAR
            } else {
                &raw const SecondaryChannelText as *const WCHAR
            };
            core::ptr::copy_nonoverlapping(src as *const u8, Buffer as *mut u8, Size as usize);
            DbgPrint(
                b"Device ID: '%S'\n\0" as *const u8 as PCSTR,
                Buffer,
            );
        }
        _ => return (*Irp).IoStatus.__bindgen_anon_1.Status,
    }
    (*Irp).IoStatus.Information = (Buffer as ULONG_PTR) as _;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoQueryDeviceUsageNotification(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() > 1 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/storage/ide/pciidex/pdo.c\0"
                    .as_ptr() as PVOID,
                720u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    let Status = PciIdeXPdoRepeatRequest(
        PdoExtension,
        Irp,
        core::ptr::null_mut::<_DEVICE_CAPABILITIES>(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let mut Counter: *mut i32 = core::ptr::null_mut();

    match (*IoStack).Parameters.UsageNotification.Type as u32 {
        1 => {
            Counter = &raw mut (*PdoExtension).Common.PageFiles;
        }
        2 => {
            Counter = &raw mut (*PdoExtension).Common.HibernateFiles;
        }
        3 => {
            Counter = &raw mut (*PdoExtension).Common.DumpFiles;
        }
        _ => return Status,
    }

    IoAdjustPagingPathCount(Counter, ((*IoStack).Parameters.UsageNotification.InPath) as i32);
    IoInvalidateDeviceState((*PdoExtension).Common.Self_);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoDispatchPnp(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let IoStack: PIO_STACK_LOCATION;

    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/storage/ide/pciidex/pdo.c\0"
                    as *const u8 as *const i8 as PVOID,
                761 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStack).MinorFunction {
        IRP_MN_START_DEVICE => {
            Status = PciIdeXPdoStartDevice(
                PdoExtension,
                (*IoStack).Parameters.StartDevice.AllocatedResources,
            );
        }
        IRP_MN_STOP_DEVICE => {
            Status = PciIdeXPdoStopDevice(PdoExtension);
        }
        IRP_MN_QUERY_STOP_DEVICE | IRP_MN_QUERY_REMOVE_DEVICE => {
            Status = PciIdeXPdoQueryStopRemoveDevice(PdoExtension);
        }
        IRP_MN_CANCEL_REMOVE_DEVICE | IRP_MN_CANCEL_STOP_DEVICE => {
            Status = STATUS_SUCCESS;
        }
        IRP_MN_SURPRISE_REMOVAL | IRP_MN_REMOVE_DEVICE => {
            Status = PciIdeXPdoRemoveDevice(
                PdoExtension,
                ((*IoStack).MinorFunction as i32 == (IRP_MN_REMOVE_DEVICE) as i32) as i32 as BOOLEAN,
            );
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            if (*IoStack).Parameters.QueryDeviceRelations.Type as u32
                == TargetDeviceRelation as i32 as u32
            {
                Status = PciIdeXPdoQueryTargetDeviceRelations(PdoExtension, Irp);
            } else {
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            }
        }
        IRP_MN_QUERY_CAPABILITIES => {
            Status = PciIdeXPdoQueryCapabilities(PdoExtension, Irp);
        }
        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            Status = PciIdeXPdoQueryPnpDeviceState(PdoExtension, Irp);
        }
        IRP_MN_QUERY_RESOURCES => {
            Status = PciIdeXPdoQueryResources(PdoExtension, Irp);
        }
        IRP_MN_QUERY_RESOURCE_REQUIREMENTS => {
            Status = PciIdeXPdoQueryResourceRequirements(PdoExtension, Irp);
        }
        IRP_MN_QUERY_ID => {
            Status = PciIdeXPdoQueryId(PdoExtension, Irp);
        }
        IRP_MN_QUERY_DEVICE_TEXT => {
            Status = PciIdeXPdoQueryDeviceText(PdoExtension, Irp);
        }
        IRP_MN_DEVICE_USAGE_NOTIFICATION => {
            Status = PciIdeXPdoQueryDeviceUsageNotification(PdoExtension, Irp);
        }
        _ => {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn PciIdeXDispatchPnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    if (KeGetCurrentIrql() as i32) > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0" as *const u8 as PVOID,
            b"drivers/storage/ide/pciidex/pdo.c\0"
                as *const u8 as PVOID,
            844 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    if (*((*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION))
        .IsFDO != 0
    {
        return PciIdeXFdoDispatchPnp(
            (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION,
            Irp,
        );
    } else {
        return PciIdeXPdoDispatchPnp(
            (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION,
            Irp,
        );
    }
}
