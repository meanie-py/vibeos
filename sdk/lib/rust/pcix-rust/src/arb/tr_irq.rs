#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use ntkrnl_sys::ddk::HAL_DISPATCH;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{PINTERFACE, PTRANSLATOR_INTERFACE, TRANSLATOR_INTERFACE};
use ntkrnl_sys::ddk::_INTERFACE_TYPE_Internal as Internal;
use ntkrnl_sys::ddk::_INTERFACE_TYPE_PCIBus as PCIBus;
use ntkrnl_sys::ddk::{INTERFACE_TYPE, PCSTR, PSTR, PULONG, RtlAssert};
use ntkrnl_sys::driver_ddk::DbgPrint;
use core::ffi::c_void;

// C `HalGetInterruptTranslator` is a macro: HALDISPATCH->HalGetInterruptTranslator
// (a HAL_DISPATCH dispatch-table member, NOT an export). Call it through the
// ntoskrnl-exported HalDispatchTable global instead of a (nonexistent) extern fn.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    pub static mut HalDispatchTable: HAL_DISPATCH;
}

#[inline(always)]
unsafe fn UlongToPtr(ul: ULONG) -> PVOID {
    ul as ULONG_PTR as PVOID
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut TranslatorInterfaceInterrupt: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_TRANSLATOR_INTERFACE_STANDARD,
        MinSize: ::core::mem::size_of::<TRANSLATOR_INTERFACE>() as USHORT,
        MinVersion: 0 as USHORT,
        MaxVersion: 0 as USHORT,
        Flags: PCI_INTERFACE_FDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciTrans_Interrupt,
        Constructor: Some(core::mem::transmute(tranirq_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(tranirq_Initializer as *const ())),
    }
};
#[no_mangle]
pub unsafe extern "system" fn tranirq_Initializer(Instance: PVOID) -> NTSTATUS {
    let _ = Instance;
    if (true) {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/arb/tr_irq.c\0".as_ptr() as *const core::ffi::c_char as PVOID,
            39u32,
            b"PCI tranirq_Initializer, unexpected call.\n\0".as_ptr() as *const core::ffi::c_char as PSTR,
        );
    }
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "system" fn tranirq_Constructor(
    DeviceExtension: PVOID,
    Instance: PVOID,
    mut InterfaceData: PVOID,
    Version: USHORT,
    Size: USHORT,
    Interface: PINTERFACE,
) -> NTSTATUS {
    // ASSERT_FDO(FdoExtension) expansion from c2rust anchor
    let FdoExtension: PPCI_FDO_EXTENSION = DeviceExtension as PPCI_FDO_EXTENSION;
    if (*FdoExtension).ExtensionType != PciFdoExtensionType as ULONG {
        RtlAssert(
            b"\"(FdoExtension)->ExtensionType == PciFdoExtensionType\"\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/arb/tr_irq.c\0"
                .as_ptr()
                as PVOID,
            55 as ULONG,
            core::ptr::null_mut::<i8>(),
        );
    }

    UNREFERENCED_PARAMETER!(Instance);
    UNREFERENCED_PARAMETER!(Version);
    UNREFERENCED_PARAMETER!(Size);

    if InterfaceData as ULONG_PTR != CmResourceTypeInterrupt as ULONG_PTR {
        if DbgPrint(
            b"(%s:%d) PCI - IRQ trans constructor doesn't like %p in InterfaceSpecificData\n\0"
                .as_ptr() as PCSTR,
            b"drivers/bus/pcix/arb/tr_irq.c\0"
                .as_ptr()
                as *const i8,
            66 as i32,
            InterfaceData,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/arb/tr_irq.c\0"
                    .as_ptr()
                    as *const i8,
                66 as i32,
            );
        }
        return STATUS_INVALID_PARAMETER_3;
    }

    let mut BaseBus: ULONG = (*FdoExtension).BaseBus as ULONG;
    InterfaceData = UlongToPtr(BaseBus as ULONG) as PVOID;

    let ParentBus: ULONG;
    let ParentInterface: INTERFACE_TYPE;
    if (*FdoExtension).BusRootFdoExtension == FdoExtension {
        ParentBus = 0;
        ParentInterface = Internal;
        if DbgPrint(
            b"(%s:%d)       Is root FDO\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/arb/tr_irq.c\0"
                .as_ptr()
                as *const i8,
            80 as i32,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/arb/tr_irq.c\0"
                    .as_ptr()
                    as *const i8,
                80 as i32,
            );
        }
    } else {
        /* It's not, so the translator is rooted on the bus the bridge sits on */
        let PdoExtension: PPCI_PDO_EXTENSION =
            (*(*FdoExtension).PhysicalDeviceObject).DeviceExtension as PPCI_PDO_EXTENSION;
        // ASSERT_PDO(PdoExtension)
        ntkrnl_sys::ASSERT!((*PdoExtension).ExtensionType == PciPdoExtensionType as ULONG);
        ParentBus = (*(*PdoExtension).ParentFdoExtension).BaseBus as ULONG;
        ParentInterface = PCIBus;
        if DbgPrint(
            b"(%s:%d)       Is bridge FDO, parent bus %x, secondary bus %x\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/arb/tr_irq.c\0".as_ptr() as *const i8,
            87 as i32,
            ParentBus,
            BaseBus,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/arb/tr_irq.c\0".as_ptr() as *const i8,
                87 as i32,
            );
        }
    }

    // HALDISPATCH->HalGetInterruptTranslator(...)
    ((*core::ptr::addr_of!(HalDispatchTable))
        .HalGetInterruptTranslator
        .unwrap_unchecked())(
        ParentInterface,
        ParentBus,
        PCIBus,
        core::mem::size_of::<TRANSLATOR_INTERFACE>() as USHORT,
        0 as USHORT,
        Interface as PTRANSLATOR_INTERFACE,
        core::ptr::addr_of_mut!(InterfaceData) as PULONG,
    )
}
