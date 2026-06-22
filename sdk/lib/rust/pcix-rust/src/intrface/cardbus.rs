#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{PCI_CARDBUS_INTERFACE_PRIVATE, PINTERFACE, PIO_RESOURCE_DESCRIPTOR, PPCI_COMMON_HEADER};
use ntkrnl_sys::ddk::{PCSTR, PSTR, RtlAssert};
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ke::DbgBreakPoint;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut PciCardbusPrivateInterface: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_PCI_CARDBUS_INTERFACE_PRIVATE,
        MinSize: ::core::mem::size_of::<PCI_CARDBUS_INTERFACE_PRIVATE>() as USHORT,
        MinVersion: PCI_CB_INTRF_VERSION as USHORT,
        MaxVersion: PCI_CB_INTRF_VERSION as USHORT,
        Flags: PCI_INTERFACE_PDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciInterface_PciCb,
        Constructor: Some(core::mem::transmute(pcicbintrf_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(pcicbintrf_Initializer as *const ())),
    }
};
#[no_mangle]
pub unsafe extern "system" fn Cardbus_SaveCurrentSettings(Context: PPCI_CONFIGURATOR_CONTEXT) {
    let _ = &Context;
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
        b"Cardbus_SaveCurrentSettings\0".as_ptr() as *const core::ffi::c_char,
        b"drivers/bus/pcix/intrface/cardbus.c\0".as_ptr() as *const core::ffi::c_char,
        38i32,
    );
    DbgPrint(b"\0".as_ptr() as *const core::ffi::c_char);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn Cardbus_SaveLimits(Context: PPCI_CONFIGURATOR_CONTEXT) {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"Cardbus_SaveLimits\0".as_ptr() as *const i8,
        b"drivers/bus/pcix/intrface/cardbus.c\0".as_ptr() as *const i8,
        46i32,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn Cardbus_MassageHeaderForLimitsDetermination(
    Context: PPCI_CONFIGURATOR_CONTEXT,
) {
    let _ = Context;
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"Cardbus_MassageHeaderForLimitsDetermination\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/intrface/cardbus.c\0".as_ptr() as *const ::core::ffi::c_char,
        54 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn Cardbus_RestoreCurrent(Context: PPCI_CONFIGURATOR_CONTEXT) {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"Cardbus_RestoreCurrent\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/intrface/cardbus.c\0".as_ptr() as *const ::core::ffi::c_char,
        62 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
}

#[no_mangle]
pub unsafe extern "system" fn Cardbus_GetAdditionalResourceDescriptors(
    Context: PPCI_CONFIGURATOR_CONTEXT,
    PciData: PPCI_COMMON_HEADER,
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
) {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"Cardbus_GetAdditionalResourceDescriptors\0" as *const u8 as *const core::ffi::c_char,
        b"drivers/bus/pcix/intrface/cardbus.c\0"
            as *const u8 as *const core::ffi::c_char,
        74 as core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as PCSTR);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn Cardbus_ResetDevice(
    PdoExtension: PPCI_PDO_EXTENSION,
    PciData: PPCI_COMMON_HEADER,
) {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"Cardbus_ResetDevice\0" as *const u8 as PCSTR,
        b"drivers/bus/pcix/intrface/cardbus.c\0"
            as *const u8 as PCSTR,
        84i32,
    );
    DbgPrint(b"\0" as *const u8 as PCSTR);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn Cardbus_ChangeResourceSettings(
    PdoExtension: PPCI_PDO_EXTENSION,
    PciData: PPCI_COMMON_HEADER,
) {
    let _ = PdoExtension;
    let _ = PciData;
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"Cardbus_ChangeResourceSettings\0".as_ptr() as *const core::ffi::c_char,
        b"drivers/bus/pcix/intrface/cardbus.c\0".as_ptr()
            as *const core::ffi::c_char,
        94i32,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
}
#[no_mangle]
pub unsafe extern "system" fn pcicbintrf_Initializer(Instance: PVOID) -> NTSTATUS {
    let _ = Instance;
    if 0i32 == 0i32 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/intrface/cardbus.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            103 as ULONG,
            b"PCI pcicbintrf_Initializer, unexpected call.\n\0" as *const u8
                as *const core::ffi::c_char as PSTR,
        );
    }
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn pcicbintrf_Constructor(
    DeviceExtension: PVOID,
    Instance: PVOID,
    InterfaceData: PVOID,
    Version: USHORT,
    Size: USHORT,
    Interface: PINTERFACE,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"pcicbintrf_Constructor\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/intrface/cardbus.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        124 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
    return STATUS_NOT_IMPLEMENTED;
}
