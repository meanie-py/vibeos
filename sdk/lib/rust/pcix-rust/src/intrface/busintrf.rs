#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{BUS_INTERFACE_STANDARD, PBUS_INTERFACE_STANDARD, PINTERFACE, PDEVICE_DESCRIPTION,
    PDMA_ADAPTER, PHYSICAL_ADDRESS, PPHYSICAL_ADDRESS, HalTranslateBusAddress, HalGetBusDataByOffset,
    HalSetBusDataByOffset, IoGetDmaAdapter};
use ntkrnl_sys::ddk::_INTERFACE_TYPE_PCIBus as PCIBus;
use ntkrnl_sys::ddk::_BUS_DATA_TYPE_PCIConfiguration as PCIConfiguration;
use ntkrnl_sys::ddk::PSTR;
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ke::DbgBreakPoint;
use core::ffi::c_void;
use ntkrnl_sys::interlocked::{InterlockedDecrement, InterlockedIncrement};

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
    UNREFERENCED_PARAMETER(Instance);
    ASSERTMSG(b"PCI busintrf_Initializer, unexpected call.\n\0".as_ptr() as PSTR, 0u8);
    STATUS_UNSUCCESSFUL
}
// The PCI bus interface for a PDO (upstream 81f04b39ba4). Context is the PDO extension.
#[no_mangle]
pub unsafe extern "system" fn PciBusInterface_Reference(Context: PVOID) {
    let PdoExtension = Context as PPCI_PDO_EXTENSION;
    InterlockedIncrement(&raw mut (*PdoExtension).BusInterfaceReferenceCount);
}

#[no_mangle]
pub unsafe extern "system" fn PciBusInterface_Dereference(Context: PVOID) {
    let PdoExtension = Context as PPCI_PDO_EXTENSION;
    InterlockedDecrement(&raw mut (*PdoExtension).BusInterfaceReferenceCount);
}

#[no_mangle]
pub unsafe extern "system" fn PciBusInterface_TranslateBusAddress(
    Context: PVOID,
    BusAddress: PHYSICAL_ADDRESS,
    Length: ULONG,
    AddressSpace: PULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    let PdoExtension = Context as PPCI_PDO_EXTENSION;
    let FdoExtension = (*PdoExtension).ParentFdoExtension;
    let _ = Length;
    HalTranslateBusAddress(
        PCIBus,
        (*FdoExtension).BaseBus as ULONG,
        BusAddress,
        AddressSpace,
        TranslatedAddress,
    )
}

#[no_mangle]
pub unsafe extern "system" fn PciBusInterface_GetDmaAdapter(
    Context: PVOID,
    DeviceDescriptor: PDEVICE_DESCRIPTION,
    NumberOfMapRegisters: PULONG,
) -> PDMA_ADAPTER {
    let PdoExtension = Context as PPCI_PDO_EXTENSION;
    let FdoExtension = (*PdoExtension).ParentFdoExtension;
    if (*DeviceDescriptor).InterfaceType == PCIBus {
        (*DeviceDescriptor).BusNumber = (*FdoExtension).BaseBus as ULONG;
    }
    IoGetDmaAdapter((*FdoExtension).PhysicalDeviceObject, DeviceDescriptor, NumberOfMapRegisters)
}

#[no_mangle]
pub unsafe extern "system" fn PciBusInterface_GetBusData(
    Context: PVOID,
    WhichSpace: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    let PdoExtension = Context as PPCI_PDO_EXTENSION;
    let _ = WhichSpace;
    HalGetBusDataByOffset(
        PCIConfiguration,
        (*(*PdoExtension).ParentFdoExtension).BaseBus as ULONG,
        (*PdoExtension).Slot.u.AsULONG,
        Buffer,
        Offset,
        Length,
    )
}

#[no_mangle]
pub unsafe extern "system" fn PciBusInterface_SetBusData(
    Context: PVOID,
    WhichSpace: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    let PdoExtension = Context as PPCI_PDO_EXTENSION;
    let _ = WhichSpace;
    HalSetBusDataByOffset(
        PCIConfiguration,
        (*(*PdoExtension).ParentFdoExtension).BaseBus as ULONG,
        (*PdoExtension).Slot.u.AsULONG,
        Buffer,
        Offset,
        Length,
    )
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
    let _ = (Instance, InterfaceData, Version);
    PAGED_CODE!();

    if (Size as usize) < core::mem::size_of::<BUS_INTERFACE_STANDARD>() {
        return STATUS_UNSUCCESSFUL;
    }

    let BusInterface = Interface as PBUS_INTERFACE_STANDARD;
    let PdoExtension = DeviceExtension as PPCI_PDO_EXTENSION;

    (*BusInterface).Size = core::mem::size_of::<BUS_INTERFACE_STANDARD>() as USHORT;
    (*BusInterface).Version = 1;
    (*BusInterface).Context = PdoExtension as PVOID;
    (*BusInterface).InterfaceReference = Some(PciBusInterface_Reference);
    (*BusInterface).InterfaceDereference = Some(PciBusInterface_Dereference);

    (*BusInterface).TranslateBusAddress = Some(PciBusInterface_TranslateBusAddress);
    (*BusInterface).GetDmaAdapter = Some(PciBusInterface_GetDmaAdapter);
    (*BusInterface).SetBusData = Some(PciBusInterface_SetBusData);
    (*BusInterface).GetBusData = Some(PciBusInterface_GetBusData);

    STATUS_SUCCESS
}
