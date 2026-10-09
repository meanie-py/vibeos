#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use core::ffi::c_void;
use ntkrnl_sys::ddk::{BOOLEAN, UCHAR, CHAR, ULONG, PVOID, PCSTR, RtlAssert};
use ntkrnl_sys::base::APC_LEVEL;
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ke::KeGetCurrentIrql;
#[no_mangle]
pub unsafe extern "system" fn PciAreBusNumbersConfigured(
    PdoExtension: PPCI_PDO_EXTENSION,
) -> BOOLEAN {
    let mut PrimaryBus: UCHAR;
    let mut BaseBus: UCHAR;
    let mut SecondaryBus: UCHAR;
    let mut SubordinateBus: UCHAR;

    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/bus/pcix/pci/busno.c\0"
                    .as_ptr() as *const i8 as PVOID,
                24 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    PrimaryBus = (*PdoExtension).Dependent.type1.as_ref().PrimaryBus;
    BaseBus = (*(*PdoExtension).ParentFdoExtension).BaseBus as UCHAR;
    SecondaryBus = (*PdoExtension).Dependent.type1.as_ref().SecondaryBus;
    SubordinateBus = (*PdoExtension).Dependent.type1.as_ref().SubordinateBus;

    if PrimaryBus as i32 != BaseBus as i32 || SecondaryBus as i32 <= PrimaryBus as i32 {
        return 0u8;
    }

    return (SubordinateBus as i32 >= SecondaryBus as i32) as BOOLEAN;
}
