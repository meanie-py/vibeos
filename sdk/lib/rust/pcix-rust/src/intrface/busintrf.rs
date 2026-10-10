#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{BUS_INTERFACE_STANDARD, PINTERFACE};
use ntkrnl_sys::ddk::PSTR;
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ke::DbgBreakPoint;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut BusHandlerInterface: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_BUS_INTERFACE_STANDARD,
        MinSize: ::core::mem::size_of::<BUS_INTERFACE_STANDARD>() as USHORT,
        MinVersion: 1 as USHORT,
        MaxVersion: 1 as USHORT,
        Flags: PCI_INTERFACE_PDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciInterface_BusHandler,
        Constructor: Some(core::mem::transmute(busintrf_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(busintrf_Initializer as *const ())),
    }
};

pub const NDEBUG: bool = true;
#[no_mangle]
pub unsafe extern "system" fn busintrf_Initializer(Instance: PVOID) -> NTSTATUS {
    UNREFERENCED_PARAMETER!(Instance);
    ASSERTMSG(b"PCI busintrf_Initializer, unexpected call.\n\0".as_ptr() as PSTR, 0u8);
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "system" fn busintrf_Constructor(
    DeviceExtension: *mut core::ffi::c_void,
    Instance: *mut core::ffi::c_void,
    InterfaceData: *mut core::ffi::c_void,
    Version: u16,
    Size: u16,
    Interface: *mut core::ffi::c_void,
) -> NTSTATUS {
    UNREFERENCED_PARAMETER!(DeviceExtension);
    UNREFERENCED_PARAMETER!(Instance);
    UNREFERENCED_PARAMETER!(InterfaceData);
    UNREFERENCED_PARAMETER!(Version as *mut core::ffi::c_void);
    UNREFERENCED_PARAMETER!(Size as *mut core::ffi::c_void);
    UNREFERENCED_PARAMETER!(Interface);

    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
        b"busintrf_Constructor\0".as_ptr() as *const core::ffi::c_char,
        b"drivers/bus/pcix/intrface/busintrf.c\0"
            .as_ptr() as *const core::ffi::c_char,
        60i32,
    );
    DbgPrint(b"\0".as_ptr() as *const core::ffi::c_char);
    DbgBreakPoint();
    STATUS_NOT_IMPLEMENTED
}
