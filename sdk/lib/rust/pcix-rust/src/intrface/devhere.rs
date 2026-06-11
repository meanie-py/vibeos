#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{PCI_DEVICE_PRESENT_INTERFACE, PINTERFACE};
use ntkrnl_sys::ddk::{PCSTR, PSTR, RtlAssert};
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ke::DbgBreakPoint;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut PciDevicePresentInterface: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_PCI_DEVICE_PRESENT_INTERFACE,
        MinSize: ::core::mem::size_of::<PCI_DEVICE_PRESENT_INTERFACE>() as USHORT,
        MinVersion: PCI_DEVICE_PRESENT_INTERFACE_VERSION as USHORT,
        MaxVersion: PCI_DEVICE_PRESENT_INTERFACE_VERSION as USHORT,
        Flags: PCI_INTERFACE_PDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciInterface_DevicePresent,
        Constructor: Some(core::mem::transmute(devpresent_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(devpresent_Initializer as *const ())),
    }
};
#[no_mangle]
pub unsafe extern "system" fn devpresent_Initializer(Instance: PVOID) -> NTSTATUS {
    let _ = Instance;
    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"/home/curi/Projects/os-playground/vibeos/drivers/bus/pcix/intrface/devhere.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            39u32 as ULONG,
            b"PCI devpresent_Initializer, unexpected call.\n\0" as *const u8
                as *const core::ffi::c_char as PSTR,
        );
    } else {
    }
    return STATUS_UNSUCCESSFUL;
}

#[no_mangle]
pub unsafe extern "system" fn devpresent_Constructor(
    DeviceExtension: *mut core::ffi::c_void,
    Instance: *mut core::ffi::c_void,
    InterfaceData: *mut core::ffi::c_void,
    Version: u16,
    Size: u16,
    Interface: PINTERFACE,
) -> NTSTATUS {
    // PAGED_CODE()
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"/home/curi/Projects/os-playground/vibeos/drivers/bus/pcix/intrface/devhere.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                52u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    // UNIMPLEMENTED_DBGBREAK()
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"devpresent_Constructor\0".as_ptr() as *const i8,
        b"/home/curi/Projects/os-playground/vibeos/drivers/bus/pcix/intrface/devhere.c\0".as_ptr() as *const i8,
        62i32,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
    STATUS_NOT_IMPLEMENTED
}
