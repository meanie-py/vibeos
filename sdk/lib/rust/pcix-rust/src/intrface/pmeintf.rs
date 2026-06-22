#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{PCI_PME_INTERFACE, PINTERFACE};
use ntkrnl_sys::ddk::{PSTR, RtlAssert};
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ke::DbgBreakPoint;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut PciPmeInterface: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_PCI_PME_INTERFACE,
        MinSize: ::core::mem::size_of::<PCI_PME_INTERFACE>() as USHORT,
        MinVersion: PCI_PME_INTRF_STANDARD_VER as USHORT,
        MaxVersion: PCI_PME_INTRF_STANDARD_VER as USHORT,
        Flags: (PCI_INTERFACE_FDO | PCI_INTERFACE_ROOT) as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciInterface_PmeHandler,
        Constructor: Some(core::mem::transmute(PciPmeInterfaceConstructor as *const ())),
        Initializer: Some(core::mem::transmute(PciPmeInterfaceInitializer as *const ())),
    }
};
#[no_mangle]
pub unsafe extern "system" fn PciPmeInterfaceInitializer(Instance: PVOID) -> NTSTATUS {
    let _ = Instance;
    if (0 as i32) == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/intrface/pmeintf.c\0".as_ptr() as PVOID,
            39u32,
            b"PCI PciPmeInterfaceInitializer, unexpected call.\n\0".as_ptr() as *const core::ffi::c_char as PSTR,
        );
    } else {
    }
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "system" fn PciPmeInterfaceConstructor(
    DeviceExtension: *mut core::ffi::c_void,
    Instance: *mut core::ffi::c_void,
    InterfaceData: *mut core::ffi::c_void,
    Version: u16,
    Size: u16,
    Interface: PINTERFACE,
) -> NTSTATUS {
    if Version as i32 != PCI_PME_INTRF_STANDARD_VER as i32 {
        return STATUS_NOINTERFACE;
    }
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"PciPmeInterfaceConstructor\0".as_ptr() as *const i8,
        b"drivers/bus/pcix/intrface/pmeintf.c\0".as_ptr() as *const i8,
        62i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
    STATUS_NOT_IMPLEMENTED
}
