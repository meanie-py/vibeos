#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{LEGACY_DEVICE_DETECTION_INTERFACE, PINTERFACE};
use ntkrnl_sys::ddk::{GUID, PCSTR, RtlAssert};
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ke::DbgBreakPoint;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut PciLegacyDeviceDetectionInterface: PCI_INTERFACE = unsafe {
    PCI_INTERFACE {
        InterfaceType: &raw const GUID_LEGACY_DEVICE_DETECTION_STANDARD,
        MinSize: ::core::mem::size_of::<LEGACY_DEVICE_DETECTION_INTERFACE>() as USHORT,
        MinVersion: 0 as USHORT,
        MaxVersion: 0 as USHORT,
        Flags: PCI_INTERFACE_FDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciInterface_LegacyDeviceDetection,
        Constructor: Some(core::mem::transmute(lddintrf_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(lddintrf_Initializer as *const ())),
    }
};

#[no_mangle]
pub unsafe extern "system" fn lddintrf_Initializer(Instance: *mut core::ffi::c_void) -> NTSTATUS {
    RtlAssert(
        (b"FALSE\0" as *const u8) as *const core::ffi::c_char as *mut core::ffi::c_void,
        (b"drivers/bus/pcix/intrface/lddintrf.c\0" as *const u8) as *const core::ffi::c_char as *mut core::ffi::c_void,
        39 as u32,
        (b"PCI lddintrf_Initializer, unexpected call.\n\0" as *const u8) as *const core::ffi::c_char as *mut i8,
    );
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn lddintrf_Constructor(
    DeviceExtension: PVOID,
    Instance: PVOID,
    InterfaceData: PVOID,
    Version: USHORT,
    Size: USHORT,
    Interface: PINTERFACE,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"lddintrf_Constructor\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/intrface/lddintrf.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        60i32,
    );
    DbgPrint(b"\0" as *const u8 as PCSTR);
    DbgBreakPoint();
    STATUS_NOT_IMPLEMENTED
}
