#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{PINTERFACE};
use ntkrnl_sys::ddk::{PCSTR, PSTR, RtlAssert, PHYSICAL_ADDRESS};
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ke::DbgBreakPoint;
use core::ffi::c_void;

// AGP_BUS_INTERFACE_STANDARD layout mirror (from ddk/ntagp.h) — used only for size_of.
#[repr(C)]
struct AGP_BUS_INTERFACE_STANDARD {
    Size: USHORT,
    Version: USHORT,
    AgpContext: PVOID,
    InterfaceReference: PVOID,
    InterfaceDereference: PVOID,
    Capabilities: ULONG,
    ReserveMemory: PVOID,
    ReleaseMemory: PVOID,
    CommitMemory: PVOID,
    FreeMemory: PVOID,
    GetMappedPages: PVOID,
    SetRate: PVOID,
    AgpSize: SIZE_T,
    AgpBase: PHYSICAL_ADDRESS,
    MaxPhysicalAddress: PHYSICAL_ADDRESS,
    MapMemory: PVOID,
    UnMapMemory: PVOID,
    FlushChipsetCaches: PVOID,
    CheckIntegrity: PVOID,
    MapMemoryEx: PVOID,
    UnMapMemoryEx: PVOID,
    FlushGartTLB: PVOID,
    CheckGuardPage: PVOID,
    GetAgpInfo: PVOID,
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut AgpTargetInterface: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_AGP_TARGET_BUS_INTERFACE_STANDARD,
        MinSize: ::core::mem::size_of::<AGP_BUS_INTERFACE_STANDARD>() as USHORT,
        MinVersion: AGP_BUS_INTERFACE_V1 as USHORT,
        MaxVersion: AGP_BUS_INTERFACE_V1 as USHORT,
        Flags: PCI_INTERFACE_PDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciInterface_AgpTarget,
        Constructor: Some(core::mem::transmute(agpintrf_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(agpintrf_Initializer as *const ())),
    }
};
#[no_mangle]
pub unsafe extern "system" fn agpintrf_Initializer(Instance: PVOID) -> NTSTATUS {
    let _ = Instance;
    if 0 as core::ffi::c_int == 0 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/intrface/agpintrf.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            41 as ULONG,
            b"PCI agpintrf_Initializer, unexpected call.\n\0" as *const u8
                as *const core::ffi::c_char as PSTR,
        );
    }
    return STATUS_UNSUCCESSFUL;
}
#[no_mangle]
pub unsafe extern "system" fn agpintrf_Constructor(
    DeviceExtension: *mut core::ffi::c_void,
    Instance: *mut core::ffi::c_void,
    InterfaceData: *mut core::ffi::c_void,
    Version: u16,
    Size: u16,
    Interface: PINTERFACE,
) -> NTSTATUS {
    let PdoExtension: PPCI_PDO_EXTENSION = DeviceExtension as PPCI_PDO_EXTENSION;
    if (*PdoExtension).BaseClass as ::core::ffi::c_int != PCI_CLASS_BRIDGE_DEV as ::core::ffi::c_int
        || (*PdoExtension).SubClass as ::core::ffi::c_int != PCI_SUBCLASS_BR_PCI_TO_PCI as ::core::ffi::c_int
    {
        return STATUS_NOT_SUPPORTED;
    }
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"agpintrf_Constructor\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/intrface/agpintrf.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        71 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as PCSTR);
    DbgBreakPoint();
    return STATUS_NOT_IMPLEMENTED;
}
