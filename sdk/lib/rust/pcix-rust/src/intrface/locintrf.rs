#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{PINTERFACE, PNP_LOCATION_INTERFACE};
use ntkrnl_sys::ddk::{GUID, PCSTR};
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ke::DbgBreakPoint;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut PciLocationInterface: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_PNP_LOCATION_INTERFACE,
        MinSize: ::core::mem::size_of::<PNP_LOCATION_INTERFACE>() as USHORT,
        MinVersion: PNP_LOCATION_INTERFACE_VERSION as USHORT,
        MaxVersion: PNP_LOCATION_INTERFACE_VERSION as USHORT,
        Flags: (PCI_INTERFACE_FDO | PCI_INTERFACE_ROOT | PCI_INTERFACE_PDO) as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciInterface_Location,
        Constructor: Some(core::mem::transmute(locintrf_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(locintrf_Initializer as *const ())),
    }
};

#[no_mangle]
pub unsafe extern "system" fn locintrf_Initializer(Instance: PVOID) -> NTSTATUS {
    UNREFERENCED_PARAMETER!(Instance);
    ASSERTMSG(
        b"PCI locintrf_Initializer, unexpected call.\n\0".as_ptr() as *const i8,
        0u8,
    );
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "system" fn locintrf_Constructor(
    DeviceExtension: PVOID,
    Instance: PVOID,
    InterfaceData: PVOID,
    Version: USHORT,
    Size: USHORT,
    Interface: PINTERFACE,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"locintrf_Constructor\0".as_ptr() as *const core::ffi::c_char,
        b"drivers/bus/pcix/intrface/locintrf.c\0".as_ptr()
            as *const core::ffi::c_char,
        60 as core::ffi::c_int,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
    STATUS_NOT_IMPLEMENTED
}
