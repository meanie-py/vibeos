//! ACPI HAL bus-handler emulation — passes most operations through to
//! the underlying CMOS / PCI primitives, plus the Hal* exports the
//! kernel and drivers use for resource allocation, bus address
//! translation, and config-space access.
//!
//! Translated from `hal/halx86/acpi/busemul.c` — amd64-only.  The i386
//! PIC HAL has parallel impls in legacy/bussupp.c (1701 LOC, still C).
//!
//! Cfg-gated to `target_arch = "x86_64"` to avoid clashing with the
//! still-C i386 impls.

#![cfg(target_arch = "x86_64")]

use ntkrnl_sys::hal_ddk::{
    HalPrivateDispatchTable, HalpAssignPCISlotResources, HalpCheckPowerButton, HalpFakePciBusHandler,
    HalpGetCmosData, HalpGetPCIData, HalpGetRootInterruptVector, HalpMapPhysicalMemory64Vista,
    HalpMaxPciBus, HalpMinPciBus, HalpPCIConfigInitialized, HalpReleasePciDeviceForDebugging,
    HalpSetCmosData, HalpSetPCIData, HalpSetupPciDeviceForDebugging, HalpUnmapVirtualAddressVista,
    BOOLEAN, BUS_DATA_TYPE, BUS_HANDLER, INTERFACE_TYPE, NTSTATUS, PCM_RESOURCE_LIST,
    PDEVICE_OBJECT, PDRIVER_OBJECT, PHYSICAL_ADDRESS, PIO_RESOURCE_REQUIREMENTS_LIST, PKAFFINITY,
    PKIRQL, PPHYSICAL_ADDRESS, PULONG, PULONG_PTR, PUNICODE_STRING, PVOID, ULONG,
    _BUS_DATA_TYPE_Cmos as DT_CMOS,
    _BUS_DATA_TYPE_EisaConfiguration as DT_EISA,
    _BUS_DATA_TYPE_PCIConfiguration as DT_PCI,
    _INTERFACE_TYPE_PCIBus as IFT_PCIBUS,
};

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS;

// ---------------------------------------------------------------------------
// HalpRegisterKdSupportFunctions — wire HAL primitives into the
// HalPrivateDispatchTable function-pointer table that ntoskrnl reads
// when servicing KD-PCI / map-physical / power-button requests.  C uses
// `KdSetupPciDeviceForDebugging = ...` — that's a macro for
// HALPRIVATEDISPATCH->KdSetupPciDeviceForDebugging (per ndk/halfuncs.h).
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpRegisterKdSupportFunctions() {
    unsafe {
        HalPrivateDispatchTable.KdSetupPciDeviceForDebugging =
            Some(core::mem::transmute(HalpSetupPciDeviceForDebugging as usize));
        HalPrivateDispatchTable.KdReleasePciDeviceforDebugging =
            Some(core::mem::transmute(HalpReleasePciDeviceForDebugging as usize));

        // Vista+ map/unmap entry points.  Our HAL targets >= NTDDI_VISTA
        // (matches the C #if NTDDI_VERSION >= NTDDI_VISTA branch).
        HalPrivateDispatchTable.KdMapPhysicalMemory64 =
            Some(core::mem::transmute(HalpMapPhysicalMemory64Vista as usize));
        HalPrivateDispatchTable.KdUnmapVirtualAddress =
            Some(core::mem::transmute(HalpUnmapVirtualAddressVista as usize));

        HalPrivateDispatchTable.KdCheckPowerButton =
            Some(core::mem::transmute(HalpCheckPowerButton as usize));
    }
}

// ---------------------------------------------------------------------------
// HalpAssignSlotResources — non-PCI internal HAL helper; only PCI
// supported, anything else returns STATUS_NOT_IMPLEMENTED.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpAssignSlotResources(
    RegistryPath: PUNICODE_STRING,
    DriverClassName: PUNICODE_STRING,
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    BusType: INTERFACE_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    AllocatedResources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    if BusType != IFT_PCIBUS {
        return STATUS_NOT_IMPLEMENTED;
    }
    unsafe {
        let mut bus_handler: BUS_HANDLER = core::mem::zeroed();
        // RtlCopyMemory is a memcpy macro in NT.
        core::ptr::copy_nonoverlapping(
            &raw const HalpFakePciBusHandler,
            &mut bus_handler,
            1,
        );
        bus_handler.BusNumber = BusNumber;
        HalpAssignPCISlotResources(
            &mut bus_handler,
            &mut bus_handler,
            RegistryPath,
            DriverClassName,
            DriverObject,
            DeviceObject,
            SlotNumber,
            AllocatedResources,
        )
    }
}

// ---------------------------------------------------------------------------
// HalpTranslateBusAddress — internal helper, identity translation.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpTranslateBusAddress(
    _InterfaceType: INTERFACE_TYPE,
    _BusNumber: ULONG,
    BusAddress: PHYSICAL_ADDRESS,
    _AddressSpace: PULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    unsafe {
        (*TranslatedAddress).QuadPart = BusAddress.QuadPart;
        1 /* TRUE */
    }
}

// ---------------------------------------------------------------------------
// HalpFindBusAddressTranslation — context-based passthrough.  First
// call (Context=0) succeeds and stamps Context=1; subsequent calls with
// NextBus=TRUE fail to indicate "no more matches."
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpFindBusAddressTranslation(
    BusAddress: PHYSICAL_ADDRESS,
    _AddressSpace: PULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
    Context: PULONG_PTR,
    NextBus: BOOLEAN,
) -> BOOLEAN {
    if Context.is_null() {
        return 0;
    }
    unsafe {
        if *Context != 0 && NextBus != 0 {
            return 0;
        }
        (*TranslatedAddress).QuadPart = BusAddress.QuadPart;
        *Context = 1;
        1
    }
}

// ---------------------------------------------------------------------------
// hal.spec exports.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAdjustResourceList(
    _pRequirementsList: *mut PIO_RESOURCE_REQUIREMENTS_LIST,
) -> NTSTATUS {
    STATUS_SUCCESS // deprecated
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAssignSlotResources(
    RegistryPath: PUNICODE_STRING,
    DriverClassName: PUNICODE_STRING,
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    BusType: INTERFACE_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    AllocatedResources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    unsafe {
        if BusType != IFT_PCIBUS {
            HalpAssignSlotResources(
                RegistryPath,
                DriverClassName,
                DriverObject,
                DeviceObject,
                BusType,
                BusNumber,
                SlotNumber,
                AllocatedResources,
            )
        } else {
            // HalPciAssignSlotResources lives in HalPrivateDispatchTable.
            let f = HalPrivateDispatchTable.HalPciAssignSlotResources;
            (f.unwrap_unchecked())(
                RegistryPath,
                DriverClassName,
                DriverObject,
                DeviceObject,
                IFT_PCIBUS,
                BusNumber,
                SlotNumber,
                AllocatedResources,
            )
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalGetBusData(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    unsafe { HalGetBusDataByOffset(BusDataType, BusNumber, SlotNumber, Buffer, 0, Length) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalGetBusDataByOffset(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    unsafe {
        if BusDataType == DT_CMOS {
            return HalpGetCmosData(0, SlotNumber, Buffer, Length);
        }
        if BusDataType == DT_EISA {
            // FIXME (matches C: ASSERT(FALSE)).
            return 0;
        }
        let init = core::ptr::read_volatile(&raw const HalpPCIConfigInitialized);
        let min = core::ptr::read_volatile(&raw const HalpMinPciBus);
        let max = core::ptr::read_volatile(&raw const HalpMaxPciBus);
        if BusDataType == DT_PCI && init != 0 && BusNumber >= min && BusNumber <= max {
            let mut bus_handler: BUS_HANDLER = core::mem::zeroed();
            core::ptr::copy_nonoverlapping(&raw const HalpFakePciBusHandler, &mut bus_handler, 1);
            bus_handler.BusNumber = BusNumber;
            return HalpGetPCIData(
                &mut bus_handler,
                &mut bus_handler,
                SlotNumber,
                Buffer,
                Offset,
                Length,
            );
        }
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalGetInterruptVector(
    _InterfaceType: INTERFACE_TYPE,
    _BusNumber: ULONG,
    BusInterruptLevel: ULONG,
    BusInterruptVector: ULONG,
    Irql: PKIRQL,
    Affinity: PKAFFINITY,
) -> ULONG {
    if crate::message::is_message_token(BusInterruptVector) {
        return unsafe { crate::message::allocate(Irql, Affinity) };
    }
    unsafe { HalpGetRootInterruptVector(BusInterruptLevel, BusInterruptVector, Irql, Affinity) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetBusData(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    unsafe { HalSetBusDataByOffset(BusDataType, BusNumber, SlotNumber, Buffer, 0, Length) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetBusDataByOffset(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    unsafe {
        if BusDataType == DT_CMOS {
            return HalpSetCmosData(0, SlotNumber, Buffer, Length);
        }
        let init = core::ptr::read_volatile(&raw const HalpPCIConfigInitialized);
        if BusDataType == DT_PCI && init != 0 {
            let mut bus_handler: BUS_HANDLER = core::mem::zeroed();
            core::ptr::copy_nonoverlapping(&raw const HalpFakePciBusHandler, &mut bus_handler, 1);
            bus_handler.BusNumber = BusNumber;
            return HalpSetPCIData(
                &mut bus_handler,
                &mut bus_handler,
                SlotNumber,
                Buffer,
                Offset,
                Length,
            );
        }
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalTranslateBusAddress(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
    BusAddress: PHYSICAL_ADDRESS,
    AddressSpace: PULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    unsafe {
        if InterfaceType == IFT_PCIBUS {
            // HalPciTranslateBusAddress lives in HalPrivateDispatchTable.
            let f = HalPrivateDispatchTable.HalPciTranslateBusAddress;
            (f.unwrap_unchecked())(
                IFT_PCIBUS,
                BusNumber,
                BusAddress,
                AddressSpace,
                TranslatedAddress,
            )
        } else {
            (*TranslatedAddress).QuadPart = BusAddress.QuadPart;
            1
        }
    }
}
