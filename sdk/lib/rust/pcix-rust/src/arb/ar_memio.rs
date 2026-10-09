#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{ARBITER_INTERFACE, PINTERFACE};
use ntkrnl_sys::ddk::{PCSTR, RtlAssert};
use ntkrnl_sys::driver_ddk::DbgPrint;
use core::ffi::c_void;

const CmResourceTypeMemory: ULONG_PTR = 3;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut ArbiterInterfaceMemory: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_ARBITER_INTERFACE_STANDARD,
        MinSize: ::core::mem::size_of::<ARBITER_INTERFACE>() as USHORT,
        MinVersion: 0 as USHORT,
        MaxVersion: 0 as USHORT,
        Flags: PCI_INTERFACE_FDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciArb_Memory,
        Constructor: Some(core::mem::transmute(armem_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(armem_Initializer as *const ())),
    }
};

#[no_mangle]
pub static mut ArbiterInterfaceIo: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_ARBITER_INTERFACE_STANDARD,
        MinSize: ::core::mem::size_of::<ARBITER_INTERFACE>() as USHORT,
        MinVersion: 0 as USHORT,
        MaxVersion: 0 as USHORT,
        Flags: PCI_INTERFACE_FDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciArb_Io,
        Constructor: Some(core::mem::transmute(ario_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(ario_Initializer as *const ())),
    }
};
#[no_mangle]
pub unsafe extern "system" fn ario_Initializer(Instance: *mut core::ffi::c_void) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8,
        b"ario_Initializer\0" as *const u8 as *const i8,
        b"drivers/bus/pcix/arb/ar_memio.c\0" as *const u8 as *const i8,
        53 as i32,
    );
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ario_Constructor(
    DeviceExtension: PVOID,
    PciInterface: PVOID,
    InterfaceData: PVOID,
    Version: USHORT,
    Size: USHORT,
    Interface: PINTERFACE,
) -> NTSTATUS {
    let FdoExtension: PPCI_FDO_EXTENSION = DeviceExtension as PPCI_FDO_EXTENSION;
    let mut Status: NTSTATUS ;

    // PAGED_CODE()
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0_i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/bus/pcix/arb/ar_memio.c\0".as_ptr() as *const i8 as PVOID,
                69 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    // UNREFERENCED_PARAMETER handled by not using the parameters
    // Suppressed warnings: PciInterface, Version, Size, Interface

    if (InterfaceData as usize) != (CmResourceTypePort as usize) {
        // Arbiter support must have been initialized first
        if (*FdoExtension).ArbitersInitialized != 0 {
            DbgPrint(
                b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
                b"ario_Constructor\0".as_ptr() as *const i8,
                b"drivers/bus/pcix/arb/ar_memio.c\0".as_ptr() as *const i8,
                83 as i32,
            );
            loop {}
        } else {
            Status = STATUS_NOT_SUPPORTED;
        }
    } else {
        Status = STATUS_INVALID_PARAMETER_5;
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn ario_ApplyBrokenVideoHack(FdoExtension: PPCI_FDO_EXTENSION) {
    if (*FdoExtension).BrokenVideoHackApplied != 0 {
        RtlAssert(
            b"!FdoExtension->BrokenVideoHackApplied\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/arb/ar_memio.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
            111,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    if (*FdoExtension).BusRootFdoExtension != FdoExtension {
        RtlAssert(
            b"PCI_IS_ROOT_FDO(FdoExtension)\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/arb/ar_memio.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
            112,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    let PciArbiter = PciFindNextSecondaryExtension(
        (*FdoExtension).SecondaryExtension.Next.cast(),
        PciArb_Io,
    ) as PPCI_ARBITER_INSTANCE;

    if PciArbiter.is_null() {
        RtlAssert(
            b"PciArbiter\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/arb/ar_memio.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
            118,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"ario_ApplyBrokenVideoHack\0" as *const u8 as *const core::ffi::c_char,
        b"drivers/bus/pcix/arb/ar_memio.c\0" as *const u8 as *const core::ffi::c_char,
        135,
    );
    loop {}
    (*FdoExtension).BrokenVideoHackApplied = 1u8;
}
#[no_mangle]
pub unsafe extern "system" fn armem_Initializer(Instance: PVOID) -> NTSTATUS {
    let _ = Instance;
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
        b"armem_Initializer\0".as_ptr() as *const core::ffi::c_char,
        b"drivers/bus/pcix/arb/ar_memio.c\0".as_ptr()
            as *const core::ffi::c_char,
        149i32,
    );
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn armem_Constructor(
    DeviceExtension: PVOID,
    PciInterface: PVOID,
    InterfaceData: PVOID,
    Version: USHORT,
    Size: USHORT,
    Interface: PINTERFACE,
) -> NTSTATUS {
    let _ = PciInterface;
    let _ = Version;
    let _ = Size;
    let _ = Interface;

    let FdoExtension = DeviceExtension as PPCI_FDO_EXTENSION;

    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as core::ffi::c_uint,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/arb/ar_memio.c\0"
                    .as_ptr()
                    as PVOID,
                165,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    let Status: NTSTATUS;
    if (InterfaceData as ULONG_PTR) != CmResourceTypeMemory {
        if (*FdoExtension).ArbitersInitialized != 0u8 {
            DbgPrint(
                b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
                b"armem_Constructor\0".as_ptr() as *const i8,
                b"drivers/bus/pcix/arb/ar_memio.c\0"
                    .as_ptr() as *const i8,
                179,
            );
            loop {}
        } else {
            Status = STATUS_NOT_SUPPORTED;
        }
    } else {
        Status = STATUS_INVALID_PARAMETER_5;
    }

    return Status;
}
