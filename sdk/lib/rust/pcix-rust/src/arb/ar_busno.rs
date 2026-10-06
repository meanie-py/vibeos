#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ARBITER_INTERFACE, PINTERFACE};
use ntkrnl_sys::ddk::{ARBITER_INSTANCE, CM_RESOURCE_TYPE, RtlAssert};
use ntkrnl_sys::ddk::{PARBITER_INSTANCE, PARB_TRANSLATE_ORDERING, PDEVICE_OBJECT};
use ntkrnl_sys::driver_ddk::DbgPrint;
use core::ffi::c_void;

// The shipping arbiter library (arbiter-rust) still exports the pre-rewrite
// ArbInitializeArbiterInstance. Upstream's arbiter.h now only declares the renamed
// ArbiterLibInitializeInstance (#9502, not ported yet), so the regenerated types island
// no longer carries this prototype; declare the one we actually link against.
extern "system" {
    fn ArbInitializeArbiterInstance(
        Arbiter: PARBITER_INSTANCE,
        BusDeviceObject: PDEVICE_OBJECT,
        ResourceType: CM_RESOURCE_TYPE,
        ArbiterName: PCWSTR,
        OrderName: PCWSTR,
        TranslateOrderingFunction: PARB_TRANSLATE_ORDERING,
    ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut ArbiterInterfaceBusNumber: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_ARBITER_INTERFACE_STANDARD,
        MinSize: ::core::mem::size_of::<ARBITER_INTERFACE>() as USHORT,
        MinVersion: 0 as USHORT,
        MaxVersion: 0 as USHORT,
        Flags: PCI_INTERFACE_FDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciArb_BusNumber,
        Constructor: Some(core::mem::transmute(arbusno_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(arbusno_Initializer as *const ())),
    }
};
#[no_mangle]
pub unsafe extern "system" fn arbusno_Initializer(Instance: PVOID) -> NTSTATUS {
    let Arbiter: PPCI_ARBITER_INSTANCE = Instance as _;
    let FdoExtension: PPCI_FDO_EXTENSION;
    let mut Status: NTSTATUS;

    // PAGED_CODE()
    let current_irql = KeGetCurrentIrql();
    if current_irql as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as _,
            current_irql as i32,
        );
        if true {
            RtlAssert(
                b"0\0".as_ptr() as _,
                b"drivers/bus/pcix/arb/ar_busno.c\0"
                    .as_ptr() as _,
                41,
                core::ptr::null_mut(),
            );
        }
    }

    // RtlZeroMemory(&Arbiter->CommonInstance, sizeof(Arbiter->CommonInstance));
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*Arbiter).CommonInstance) as *mut u8,
        0,
        core::mem::size_of::<ARBITER_INSTANCE>(),
    );

    FdoExtension = (*Arbiter).BusFdoExtension;

    // UNIMPLEMENTED
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"arbusno_Initializer\0".as_ptr() as *const i8,
        b"drivers/bus/pcix/arb/ar_busno.c\0"
            .as_ptr() as *const i8,
        48,
    );

    // ArbInitializeArbiterInstance call
    let pci_str: [u16; 4] = [0x50, 0x63, 0x69, 0x00]; // L"Pci"
    Status = ArbInitializeArbiterInstance(
        core::ptr::addr_of_mut!((*Arbiter).CommonInstance),
        (*FdoExtension).FunctionalDeviceObject,
        CmResourceTypeBusNumber as CM_RESOURCE_TYPE,
        core::ptr::addr_of_mut!((*Arbiter).InstanceName) as *mut u16 as *const u16,
        &pci_str as *const u16,
        None,
    );

    if !NT_SUCCESS(Status) {
        // DPRINT1 expansion
        let ret = DbgPrint(
            b"(%s:%d) arbusno_Initializer: init arbiter return %X\0".as_ptr() as *const i8,
            b"drivers/bus/pcix/arb/ar_busno.c\0"
                .as_ptr() as *const i8,
            65,
            Status,
        );
        if ret != 0 {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"drivers/bus/pcix/arb/ar_busno.c\0"
                    .as_ptr() as *const i8,
                65,
            );
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn arbusno_Constructor(
    DeviceExtension: *mut core::ffi::c_void,
    PciInterface: *mut core::ffi::c_void,
    InterfaceData: *mut core::ffi::c_void,
    Version: u16,
    Size: u16,
    Interface: PINTERFACE,
) -> NTSTATUS {
    let FdoExtension = DeviceExtension as PPCI_FDO_EXTENSION;
    let Status: NTSTATUS;

    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"drivers/bus/pcix/arb/ar_busno.c\0"
                as *const u8 as *const i8 as *mut core::ffi::c_void,
            82u32,
            core::ptr::null_mut::<i8>(),
        );
    }

    if (InterfaceData as usize) != CmResourceTypeBusNumber as usize {
        if (*FdoExtension).ArbitersInitialized != 0 {
            DbgPrint(
                b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8,
                b"arbusno_Constructor\0" as *const u8 as *const i8,
                b"drivers/bus/pcix/arb/ar_busno.c\0"
                    as *const u8 as *const i8,
                96i32,
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
