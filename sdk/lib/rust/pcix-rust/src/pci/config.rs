#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
use crate::glue::IRP_MN_QUERY_INTERFACE; // explicit: disambiguate vs ntkrnl_sys::driver_ddk glob
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, HalGetBusDataByOffset, _IO_STATUS_BLOCK, IoBuildSynchronousFsdRequest, IoGetAttachedDeviceReference, KEVENT, KeInitializeEvent, KeWaitForSingleObject, PBUS_HANDLER, PCI_BUS_INTERFACE_STANDARD, PCI_COMMON_HEADER, PCI_SLOT_NUMBER, PDEVICE_OBJECT, PINTERFACE, PIO_STACK_LOCATION, PIRP, PPCIBUSDATA, PPCI_BUS_INTERFACE_STANDARD, PciReadWriteConfig, _EVENT_TYPE_SynchronizationEvent};
use ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation;
use ntkrnl_sys::ddk::{_BUS_HANDLER, _PCIBUSDATA, _PCI_BUS_INTERFACE_STANDARD, HAL_DISPATCH, PCI_ROOT_BUS_CAPABILITY};
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ddk::_INTERFACE_TYPE_PCIBus as PCIBus;
use ntkrnl_sys::ddk::_BUS_DATA_TYPE_PCIConfiguration as PCIConfiguration;
use ntkrnl_sys::ke::{KeGetCurrentIrql, DbgBreakPoint};
use core::ffi::c_void;

type PCSTR = *const i8;
#[allow(non_camel_case_types)]
type c_char = i8;

// HalDispatchTable is an ntoskrnl data export (cross-DLL data → raw-dylib).
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PVOID);
}
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    pub static mut HalDispatchTable: HAL_DISPATCH;
}
// RtlAssert is NTAPI (stdcall on i386, _RtlAssert@16).

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut PciAssignBusNumbers: BOOLEAN = 0;

pub const NDEBUG: bool = true;
#[no_mangle]
pub unsafe extern "system" fn PciGetAdjustedInterruptLine(
    PdoExtension: PPCI_PDO_EXTENSION,
) -> UCHAR {
    let mut InterruptLine: UCHAR = 0;
    let mut PciInterruptLine: UCHAR = 0;
    let mut Length: ULONG = 0;

    if (*PdoExtension).InterruptPin != 0 {
        Length = HalGetBusDataByOffset(
            PCIConfiguration,
            (*(*PdoExtension).ParentFdoExtension).BaseBus as ULONG,
            (*PdoExtension).Slot.u.AsULONG,
            core::ptr::addr_of_mut!(PciInterruptLine).cast::<core::ffi::c_void>(),
            60u32,
            core::mem::size_of::<UCHAR>() as ULONG,
        );
        if Length != 0 {
            InterruptLine = PciInterruptLine;
        }
    }

    if InterruptLine as i32 != 0 {
        (*PdoExtension).RawInterruptLine
    } else {
        InterruptLine
    }
}
#[no_mangle]
pub unsafe extern "system" fn PciReadWriteConfigSpace(
    DeviceExtension: PPCI_FDO_EXTENSION,
    Slot: PCI_SLOT_NUMBER,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
    Read: BOOLEAN,
) {
    let mut PciInterface: PPCI_BUS_INTERFACE_STANDARD = core::ptr::null_mut::<_PCI_BUS_INTERFACE_STANDARD>();
    let mut BusHandler: PBUS_HANDLER = core::ptr::null_mut::<_BUS_HANDLER>();
    let mut BusData: PPCIBUSDATA = core::ptr::null_mut::<_PCIBUSDATA>();
    let mut HalFunction: PciReadWriteConfig = None;

    if !((*(*DeviceExtension).BusRootFdoExtension).BusRootFdoExtension
        == (*DeviceExtension).BusRootFdoExtension)
    {
        RtlAssert(
            b"PCI_IS_ROOT_FDO(DeviceExtension->BusRootFdoExtension)\0" as *const u8 as PVOID,
            b"drivers/bus/pcix/pci/config.c\0" as *const u8
                as PVOID,
            62,
            core::ptr::null_mut::<u8>() as PVOID,
        );
    }

    /* The legacy mechanism cannot reach extended configuration space */
    if Offset.wrapping_add(Length) > PCI_LEGACY_CONFIG_LENGTH
        && PciEcamReadWriteConfig((*DeviceExtension).BaseBus as ULONG, Slot, Buffer, Offset, Length, Read) != 0
    {
        return;
    }

    PciInterface = (*(*DeviceExtension).BusRootFdoExtension).PciBusInterface;
    if !PciInterface.is_null() {
        /* Use the PCI interface to access configuration space (upstream 81f04b39ba4) */
        let LengthProcessed: ULONG = if Read as i32 != 0 {
            // A NULL slot in the interface is 0 bytes processed: the C would call through it.
            match (*PciInterface).ReadConfig {
                Some(f) => f(
                    (*PciInterface).Context,
                    (*DeviceExtension).BaseBus as ULONG,
                    Slot.u.AsULONG,
                    Buffer,
                    Offset,
                    Length,
                ),
                None => 0,
            }
        } else {
            // A NULL slot in the interface is 0 bytes processed: the C would call through it.
            match (*PciInterface).WriteConfig {
                Some(f) => f(
                    (*PciInterface).Context,
                    (*DeviceExtension).BaseBus as ULONG,
                    Slot.u.AsULONG,
                    Buffer,
                    Offset,
                    Length,
                ),
                None => 0,
            }
        };

        /* Verify the operation succeeded */
        if LengthProcessed != Length {
            DbgPrint(
                b"(%s:%d) PCI: Config space %s failed - Bus %x, Slot %x, Offset %x, Expected %x bytes, got %x\n\0"
                    as *const u8 as PCSTR,
                b"drivers/bus/pcix/pci/config.c\0" as *const u8 as PCSTR,
                103,
                if Read as i32 != 0 { b"read\0".as_ptr() } else { b"write\0".as_ptr() } as PCSTR,
                (*DeviceExtension).BaseBus as ULONG,
                Slot.u.AsULONG,
                Offset,
                Length,
                LengthProcessed,
            );
        }
    } else {
        if (*DeviceExtension).BusHandler.is_null() {
            RtlAssert(
                b"DeviceExtension->BusHandler\0" as *const u8 as PVOID,
                b"drivers/bus/pcix/pci/config.c\0"
                    as *const u8 as PVOID,
                74,
                core::ptr::null_mut::<u8>() as PVOID,
            );
        }
        if PciAssignBusNumbers != 0 {
            RtlAssert(
                b"!PciAssignBusNumbers\0" as *const u8 as PVOID,
                b"drivers/bus/pcix/pci/config.c\0"
                    as *const u8 as PVOID,
                77,
                core::ptr::null_mut::<u8>() as PVOID,
            );
        }

        BusHandler = (*DeviceExtension).BusHandler;
        BusData = (*BusHandler).BusData as PPCIBUSDATA;

        HalFunction = if Read as i32 != 0 {
            (*BusData).ReadConfig
        } else {
            (*BusData).WriteConfig
        };
        HalFunction.expect("non-null function pointer")(
            BusHandler as *mut _BUS_HANDLER,
            Slot,
            Buffer,
            Offset,
            Length,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn PciWriteDeviceConfig(
    DeviceExtension: PPCI_PDO_EXTENSION,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) {
    PciReadWriteConfigSpace(
        (*DeviceExtension).ParentFdoExtension,
        (*DeviceExtension).Slot,
        Buffer,
        Offset,
        Length,
        0u8,
    );
}

#[no_mangle]
pub unsafe extern "system" fn PciReadDeviceConfig(
    DeviceExtension: PPCI_PDO_EXTENSION,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) {
    PciReadWriteConfigSpace(
        (*DeviceExtension).ParentFdoExtension,
        (*DeviceExtension).Slot,
        Buffer,
        Offset,
        Length,
        1u8,
    );
}
#[no_mangle]
pub unsafe extern "system" fn PciReadSlotConfig(
    DeviceExtension: PPCI_FDO_EXTENSION,
    Slot: PCI_SLOT_NUMBER,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) {
    PciReadWriteConfigSpace(DeviceExtension, Slot, Buffer, Offset, Length, 1u8);
}
#[no_mangle]
pub unsafe extern "system" fn PciQueryForPciBusInterface(
    FdoExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    let mut AttachedDevice: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Irp: PIRP;
    let mut IoStackLocation: PIO_STACK_LOCATION;
    let mut PciInterface: PPCI_BUS_INTERFACE_STANDARD;

    // PAGED_CODE()
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/pci/config.c\0".as_ptr()
                    as PVOID,
                144u32,
                core::ptr::null_mut(),
            );
        }
    }

    // ASSERT(PCI_IS_ROOT_FDO(FdoExtension))
    if !((*FdoExtension).BusRootFdoExtension == FdoExtension) {
        RtlAssert(
            b"PCI_IS_ROOT_FDO(FdoExtension)\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/pci/config.c\0".as_ptr()
                as PVOID,
            145u32,
            core::ptr::null_mut(),
        );
    }

    PciInterface = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<PCI_BUS_INTERFACE_STANDARD>() as usize,
        PCI_POOL_TAG,
    ) as PPCI_BUS_INTERFACE_STANDARD;

    if PciInterface.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    AttachedDevice = IoGetAttachedDeviceReference((*FdoExtension).PhysicalDeviceObject);
    KeInitializeEvent(&raw mut Event, _EVENT_TYPE_SynchronizationEvent, 0u8);
    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as u32,
        AttachedDevice,
        core::ptr::null_mut(),
        0u32,
        core::ptr::null_mut(),
        &raw mut Event,
        &raw mut IoStatusBlock,
    );

    if !Irp.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
        (*Irp).IoStatus.Information = 0;
        IoStackLocation = IoGetNextIrpStackLocation(Irp);

        if (*IoStackLocation).MajorFunction as i32 != IRP_MJ_PNP as i32 {
            RtlAssert(
                b"IoStackLocation->MajorFunction == IRP_MJ_PNP\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/pci/config.c\0"
                    .as_ptr() as PVOID,
                173u32,
                core::ptr::null_mut(),
            );
        }

        (*IoStackLocation).MinorFunction = IRP_MN_QUERY_INTERFACE as u8;
        (*IoStackLocation).Parameters.QueryInterface.InterfaceType =
            &raw const GUID_PCI_BUS_INTERFACE_STANDARD;
        (*IoStackLocation).Parameters.QueryInterface.Size =
            core::mem::size_of::<PCI_BUS_INTERFACE_STANDARD>() as u16;
        (*IoStackLocation).Parameters.QueryInterface.Version =
            PCI_BUS_INTERFACE_STANDARD_VERSION as u16;
        (*IoStackLocation).Parameters.QueryInterface.Interface = PciInterface as PINTERFACE;
        (*IoStackLocation).Parameters.QueryInterface.InterfaceSpecificData =
            core::ptr::null_mut::<c_void>();

        Status = IoCallDriver(AttachedDevice, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                &raw mut Event as PVOID,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        if (Status as i32) < 0 {
            (*FdoExtension).PciBusInterface = core::ptr::null_mut();
            ExFreePoolWithTag(PciInterface as PVOID, 0u32);
        } else {
            (*FdoExtension).PciBusInterface = PciInterface;
        }

        ObDereferenceObject(AttachedDevice as PVOID);
    } else {
        if !AttachedDevice.is_null() {
            ObDereferenceObject(AttachedDevice as PVOID);
        }
        ExFreePoolWithTag(PciInterface as PVOID, 0u32);
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn PciGetConfigHandlers(FdoExtension: PPCI_FDO_EXTENSION) -> NTSTATUS {
    let mut BusHandler: PBUS_HANDLER = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    if !((*FdoExtension).BusHandler).is_null() {
        RtlAssert(
            b"FdoExtension->BusHandler == NULL\0".as_ptr() as *const c_char as *mut core::ffi::c_void,
            b"drivers/bus/pcix/pci/config.c\0".as_ptr() as *const c_char as *mut core::ffi::c_void,
            228u32,
            core::ptr::null_mut(),
        );
    }

    if (*FdoExtension).BusRootFdoExtension == FdoExtension {
        if !((*FdoExtension).PciBusInterface).is_null() {
            RtlAssert(
                b"FdoExtension->PciBusInterface == NULL\0".as_ptr() as *const c_char as *mut core::ffi::c_void,
                b"drivers/bus/pcix/pci/config.c\0".as_ptr() as *const c_char as *mut core::ffi::c_void,
                234u32,
                core::ptr::null_mut(),
            );
        }
        Status = PciQueryForPciBusInterface(FdoExtension);
        if Status < 0 {
            if PciAssignBusNumbers != 0 {
                RtlAssert(
                    b"!PciAssignBusNumbers\0".as_ptr() as *const c_char as *mut core::ffi::c_void,
                    b"drivers/bus/pcix/pci/config.c\0".as_ptr() as *const c_char as *mut core::ffi::c_void,
                    239u32,
                    core::ptr::null_mut(),
                );
            }
        } else {
            PciAssignBusNumbers = 1u8;

            /* The driver below owns _OSC and reports what the firmware granted */
            let PciInterface: PPCI_BUS_INTERFACE_STANDARD = (*FdoExtension).PciBusInterface;
            if (*PciInterface).Size as usize
                >= core::mem::offset_of!(_PCI_BUS_INTERFACE_STANDARD, RootBusCapability)
                    + core::mem::size_of::<PCI_ROOT_BUS_CAPABILITY>()
                && (*PciInterface).RootBusCapability.is_some()
            {
                if let Some(RootBusCapability) = (*PciInterface).RootBusCapability {
                    RootBusCapability(
                        (*PciInterface).Context,
                        core::ptr::addr_of_mut!((*FdoExtension).RootBusHardwareCapability),
                    );
                }

                DbgPrint(
                    b"(%s:%d) PCI - Root bus interface %d, _OSC support 0x%08lx, granted 0x%08lx\n\0".as_ptr() as PCSTR,
                    b"drivers/bus/pcix/pci/config.c\0".as_ptr() as PCSTR,
                    299,
                    (*FdoExtension).RootBusHardwareCapability.SecondaryInterface as i32,
                    (*FdoExtension).RootBusHardwareCapability.OscFeatureSupport.u.AsULONG,
                    (*FdoExtension).RootBusHardwareCapability.OscControlGranted.u.AsULONG,
                );
            }
        }
    } else {
        if !((*(*FdoExtension).BusRootFdoExtension).PciBusInterface).is_null() {
            return STATUS_SUCCESS;
        }
        Status = STATUS_NOT_SUPPORTED;
    }

    if Status < 0 {
        if Status != STATUS_NOT_SUPPORTED {
            RtlAssert(
                b"Status == STATUS_NOT_SUPPORTED\0".as_ptr() as *const c_char as *mut core::ffi::c_void,
                b"drivers/bus/pcix/pci/config.c\0".as_ptr() as *const c_char as *mut core::ffi::c_void,
                264u32,
                core::ptr::null_mut(),
            );
        }
        if PciAssignBusNumbers != 0 {
            RtlAssert(
                b"!PciAssignBusNumbers\0".as_ptr() as *const c_char as *mut core::ffi::c_void,
                b"drivers/bus/pcix/pci/config.c\0".as_ptr() as *const c_char as *mut core::ffi::c_void,
                265u32,
                core::ptr::null_mut(),
            );
        }
        BusHandler = HalDispatchTable
            .HalReferenceHandlerForBus
            .expect("non-null function pointer")(
            PCIBus, (*FdoExtension).BaseBus as ULONG,
        );
        (*FdoExtension).BusHandler = BusHandler as PBUS_HANDLER;
        if BusHandler.is_null() {
            return STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    STATUS_SUCCESS
}
