#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoAssignResources, KeEnterCriticalRegion, KeLeaveCriticalRegion, KeWaitForSingleObject, PCI_COMMON_HEADER, PCI_SLOT_NUMBER, PCM_RESOURCE_LIST, PDEVICE_OBJECT, PDRIVER_OBJECT, PHYSICAL_ADDRESS, PIO_RESOURCE_REQUIREMENTS_LIST, PPHYSICAL_ADDRESS, PUNICODE_STRING, pHalAssignSlotResources, pHalTranslateBusAddress};
use ntkrnl_sys::ddk::{_INTERFACE_TYPE_PCIBus as PCIBus, HAL_PRIVATE_DISPATCH};

// PCI_COMMON_HDR_LENGTH = FIELD_OFFSET(PCI_COMMON_CONFIG, DeviceSpecific) = 0x40
pub const PCI_COMMON_HDR_LENGTH: usize = 0x40;

// HAL global dispatch table (ndk/haltypes.h). value type; code takes &addr.
// ntoskrnl data export (cross-DLL data import → raw-dylib).
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    pub static mut HalPrivateDispatchTable: HAL_PRIVATE_DISPATCH;
}
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{INTERFACE_TYPE, KPRIORITY, KPROCESSOR_MODE, PCSTR, PHAL_PRIVATE_DISPATCH, RtlAssert, _CM_RESOURCE_LIST, _DEVICE_OBJECT, _IO_RESOURCE_REQUIREMENTS_LIST, _LARGE_INTEGER, _PCI_COMMON_HEADER};
use ntkrnl_sys::driver_ddk::{DbgPrint};

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut PcipSavedTranslateBusAddress: pHalTranslateBusAddress = None;

pub static mut PcipSavedAssignSlotResources: pHalAssignSlotResources = None;
#[no_mangle]
pub unsafe extern "system" fn PciTranslateBusAddress(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
    BusAddress: PHYSICAL_ADDRESS,
    AddressSpace: PULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    let _ = InterfaceType;
    let _ = BusNumber;
    let _ = AddressSpace;

    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"PciTranslateBusAddress\0".as_ptr() as *const i8,
        b"drivers/bus/pcix/hookhal.c\0".as_ptr() as *const i8,
        36i32,
    );

    (*TranslatedAddress).QuadPart = BusAddress.QuadPart;
    return 1u8;
}
#[no_mangle]
pub unsafe extern "system" fn PciFindPdoByLocation(
    BusNumber: ULONG,
    SlotNumber: ULONG,
) -> PPCI_PDO_EXTENSION {
    let mut DeviceExtension: PPCI_FDO_EXTENSION;
    let mut PdoExtension: PPCI_PDO_EXTENSION;
    let mut PciSlot: PCI_SLOT_NUMBER = core::mem::zeroed();
    PciSlot.u.AsULONG = SlotNumber;

    KeEnterCriticalRegion();
    KeWaitForSingleObject(
        &raw mut PciGlobalLock as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as KPROCESSOR_MODE,
        0u8,
        core::ptr::null_mut(),
    );

    DeviceExtension = PciFdoExtensionListHead.Next as PPCI_FDO_EXTENSION;
    while !DeviceExtension.is_null() {
        if (*DeviceExtension).BaseBus as ULONG == BusNumber {
            break;
        }
        DeviceExtension = (*DeviceExtension).List.Next as PPCI_FDO_EXTENSION;
    }

    KeSetEvent(
        (&raw mut PciGlobalLock).cast(),
        IO_NO_INCREMENT as KPRIORITY,
        0u8,
    );
    KeLeaveCriticalRegion();

    if DeviceExtension.is_null() {
        if DbgPrint(
            b"(%s:%d) Pci: Could not find PCI bus FDO. Bus Number = 0x%x\n\0" as *const u8 as PCSTR,
            b"drivers/bus/pcix/hookhal.c\0" as *const u8
                as *const core::ffi::c_char,
            74 as core::ffi::c_int,
            BusNumber,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"drivers/bus/pcix/hookhal.c\0"
                    as *const u8 as *const core::ffi::c_char,
                74 as core::ffi::c_int,
            );
        }
        return core::ptr::null_mut();
    }

    KeEnterCriticalRegion();
    KeWaitForSingleObject(
        &raw mut (*DeviceExtension).ChildListLock as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as KPROCESSOR_MODE,
        0u8,
        core::ptr::null_mut(),
    );

    PdoExtension = (*DeviceExtension).ChildPdoList as PPCI_PDO_EXTENSION;
    while !PdoExtension.is_null() {
        if (*PdoExtension).Slot.u.bits.FunctionNumber() as core::ffi::c_int
            == PciSlot.u.bits.FunctionNumber() as core::ffi::c_int
            && (*PdoExtension).Slot.u.bits.DeviceNumber() as core::ffi::c_int
                == PciSlot.u.bits.DeviceNumber() as core::ffi::c_int
        {
            if !((*PdoExtension).Slot.u.AsULONG == PciSlot.u.AsULONG) {
                RtlAssert(
                    b"PdoExtension->Slot.u.AsULONG == PciSlot.u.AsULONG\0" as *const u8
                        as *const core::ffi::c_char as PVOID,
                    b"drivers/bus/pcix/hookhal.c\0"
                        as *const u8 as *const core::ffi::c_char as PVOID,
                    96 as ULONG,
                    core::ptr::null_mut::<core::ffi::c_char>(),
                );
            }
            break;
        }
        PdoExtension = (*PdoExtension).Next as PPCI_PDO_EXTENSION;
    }

    KeSetEvent(
        (&raw mut (*DeviceExtension).ChildListLock).cast(),
        IO_NO_INCREMENT as KPRIORITY,
        0u8,
    );
    KeLeaveCriticalRegion();

    if PdoExtension.is_null() {
        if DbgPrint(
            b"(%s:%d) Pci: Could not find PDO for device @ %x.%x.%x\n\0" as *const u8 as PCSTR,
            b"drivers/bus/pcix/hookhal.c\0" as *const u8
                as *const core::ffi::c_char,
            112 as core::ffi::c_int,
            BusNumber,
            PciSlot.u.bits.DeviceNumber() as core::ffi::c_int,
            PciSlot.u.bits.FunctionNumber() as core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"drivers/bus/pcix/hookhal.c\0"
                    as *const u8 as *const core::ffi::c_char,
                112 as core::ffi::c_int,
            );
        }
    }

    PdoExtension
}

#[no_mangle]
pub unsafe extern "system" fn PciAssignSlotResources(
    RegistryPath: PUNICODE_STRING,
    DriverClassName: PUNICODE_STRING,
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    BusType: INTERFACE_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    AllocatedResources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/bus/pcix/hookhal.c\0".as_ptr()
                    as *const i8 as PVOID,
                136 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }
    if (*&raw const PcipSavedAssignSlotResources).is_none() {
        RtlAssert(
            b"PcipSavedAssignSlotResources\0".as_ptr() as *const i8 as PVOID,
            b"drivers/bus/pcix/hookhal.c\0".as_ptr()
                as *const i8 as PVOID,
            137 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    if !(BusType as i32 == PCIBus as i32) {
        RtlAssert(
            b"BusType == PCIBus\0".as_ptr() as *const i8 as PVOID,
            b"drivers/bus/pcix/hookhal.c\0".as_ptr()
                as *const i8 as PVOID,
            138 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    let mut RequirementsList: PIO_RESOURCE_REQUIREMENTS_LIST =
        core::ptr::null_mut::<_IO_RESOURCE_REQUIREMENTS_LIST>();
    let mut Resources: PCM_RESOURCE_LIST = core::ptr::null_mut::<_CM_RESOURCE_LIST>();
    let mut PciData: _PCI_COMMON_HEADER = core::mem::zeroed();
    let mut ExistingDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut::<_DEVICE_OBJECT>();
    let mut Status: NTSTATUS;

    *AllocatedResources = core::ptr::null_mut::<_CM_RESOURCE_LIST>();

    let PdoExtension: PPCI_PDO_EXTENSION = PciFindPdoByLocation(BusNumber, SlotNumber);
    if PdoExtension.is_null() {
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }
    if (*PdoExtension).DeviceState as i32 == PciNotStarted as i32 {
        return STATUS_INVALID_OWNER;
    }

    KeEnterCriticalRegion();
    KeWaitForSingleObject(
        core::ptr::addr_of_mut!(PciGlobalLock) as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as i32 as KPROCESSOR_MODE,
        FALSE as BOOLEAN,
        core::ptr::null_mut::<_LARGE_INTEGER>(),
    );

    'resource_assign_loop: loop {
        if !(DeviceObject != (*PdoExtension).PhysicalDeviceObject) {
            RtlAssert(
                b"DeviceObject != PdoExtension->PhysicalDeviceObject\0".as_ptr() as *const i8
                    as PVOID,
                b"drivers/bus/pcix/hookhal.c\0".as_ptr()
                    as *const i8 as PVOID,
                154 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }

        PciReadDeviceConfig(
            PdoExtension,
            core::ptr::addr_of_mut!(PciData) as PVOID,
            0 as ULONG,
            PCI_COMMON_HDR_LENGTH as ULONG,
        );

        Status = PciCacheLegacyDeviceRouting(
            DeviceObject,
            BusNumber,
            SlotNumber,
            PciData.u.type0.InterruptLine,
            PciData.u.type0.InterruptPin,
            PciData.BaseClass,
            PciData.SubClass,
            (*(*PdoExtension).ParentFdoExtension).PhysicalDeviceObject,
            PdoExtension,
            core::ptr::addr_of_mut!(ExistingDeviceObject),
        );

        if !NT_SUCCESS(Status as i32) {
            break;
        }

        Status = PciBuildRequirementsList(
            PdoExtension,
            core::ptr::addr_of_mut!(PciData),
            core::ptr::addr_of_mut!(RequirementsList),
        );
        if !NT_SUCCESS(Status as i32) {
            PciCacheLegacyDeviceRouting(
                ExistingDeviceObject,
                BusNumber,
                SlotNumber,
                PciData.u.type0.InterruptLine,
                PciData.u.type0.InterruptPin,
                PciData.BaseClass,
                PciData.SubClass,
                (*(*PdoExtension).ParentFdoExtension).PhysicalDeviceObject,
                PdoExtension,
                core::ptr::null_mut::<PDEVICE_OBJECT>(),
            );
            break;
        }

        (*PdoExtension).LegacyDriver = TRUE as BOOLEAN;

        Status = IoAssignResources(
            RegistryPath,
            DriverClassName,
            DriverObject,
            DeviceObject,
            RequirementsList,
            core::ptr::addr_of_mut!(Resources),
        );

        if NT_SUCCESS(Status as i32) {
            let ce = (*PdoExtension).CommandEnables as i32;
            (*PdoExtension).CommandEnables = (ce
                | ((PCI_ENABLE_IO_SPACE | PCI_ENABLE_MEMORY_SPACE | PCI_ENABLE_BUS_MASTER) as i32))
                as USHORT;

            PciComputeNewCurrentSettings(PdoExtension, Resources);

            Status = PciSetResources(PdoExtension, TRUE as BOOLEAN, TRUE as BOOLEAN);
            if NT_SUCCESS(Status as i32) {
                if !((*Resources).Count == 1) {
                    RtlAssert(
                        b"Resources->Count == 1\0".as_ptr() as *const i8 as PVOID,
                        b"drivers/bus/pcix/hookhal.c\0"
                            .as_ptr() as *const i8 as PVOID,
                        200 as ULONG,
                        core::ptr::null_mut::<CHAR>(),
                    );
                }
                DbgPrint(
                    b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const u8 as PCSTR,
                    b"PciAssignSlotResources\0".as_ptr() as *const u8,
                    b"drivers/bus/pcix/hookhal.c\0"
                        .as_ptr() as *const u8,
                    203 as i32,
                );

                *AllocatedResources = Resources;
                Resources = core::ptr::null_mut::<_CM_RESOURCE_LIST>();
                Status = STATUS_SUCCESS;
            }
        } else {
            if !Resources.is_null() {
                RtlAssert(
                    b"Resources == NULL\0".as_ptr() as *const i8 as PVOID,
                    b"drivers/bus/pcix/hookhal.c\0"
                        .as_ptr() as *const i8 as PVOID,
                    214 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            }
        }

        if NT_SUCCESS(Status as i32) {
            break;
        }

        PciCacheLegacyDeviceRouting(
            ExistingDeviceObject,
            BusNumber,
            SlotNumber,
            PciData.u.type0.InterruptLine,
            PciData.u.type0.InterruptPin,
            PciData.BaseClass,
            PciData.SubClass,
            (*(*PdoExtension).ParentFdoExtension).PhysicalDeviceObject,
            PdoExtension,
            core::ptr::null_mut::<PDEVICE_OBJECT>(),
        );

        break;
    }

    KeSetEvent(
        (core::ptr::addr_of_mut!(PciGlobalLock)).cast(),
        0 as KPRIORITY,
        0 as BOOLEAN,
    );
    KeLeaveCriticalRegion();

    if !RequirementsList.is_null() {
        ExFreePoolWithTag(RequirementsList as PVOID, 0 as ULONG);
    }
    if !Resources.is_null() {
        ExFreePoolWithTag(Resources as PVOID, 0 as ULONG);
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn PciHookHal() {
    if (*&raw const PcipSavedAssignSlotResources).is_some() {
        RtlAssert(
            b"PcipSavedAssignSlotResources == NULL\0".as_ptr() as *const i8 as PVOID,
            b"hookhal.c\0".as_ptr() as *const i8 as PVOID,
            251,
            core::ptr::null_mut::<i8>(),
        );
    }
    if (*&raw const PcipSavedTranslateBusAddress).is_some() {
        RtlAssert(
            b"PcipSavedTranslateBusAddress == NULL\0".as_ptr() as *const i8 as PVOID,
            b"hookhal.c\0".as_ptr() as *const i8 as PVOID,
            252,
            core::ptr::null_mut::<i8>(),
        );
    }
    PcipSavedAssignSlotResources =
        (*(core::ptr::addr_of_mut!(HalPrivateDispatchTable) as PHAL_PRIVATE_DISPATCH))
            .HalPciAssignSlotResources;
    PcipSavedTranslateBusAddress =
        (*(core::ptr::addr_of_mut!(HalPrivateDispatchTable) as PHAL_PRIVATE_DISPATCH))
            .HalPciTranslateBusAddress;
    (*(core::ptr::addr_of_mut!(HalPrivateDispatchTable) as PHAL_PRIVATE_DISPATCH))
        .HalPciTranslateBusAddress = core::mem::transmute::<_, pHalTranslateBusAddress>(Some(
        PciTranslateBusAddress
            as unsafe extern "system" fn(
                INTERFACE_TYPE,
                ULONG,
                PHYSICAL_ADDRESS,
                PULONG,
                PPHYSICAL_ADDRESS,
            ) -> BOOLEAN,
    ));
}

/// Undo PciHookHal. Not in the C, whose PciDriverUnload is an UNIMPLEMENTED stub: there an
/// unloaded pcix (its root AddDevice failed on amd64) left HalPciTranslateBusAddress
/// pointing into the freed image, and the next translation double-faulted the kernel.
/// Restores only what pcix installed, so a HAL entry someone else owns is left alone.
#[no_mangle]
pub unsafe extern "system" fn PciUnhookHal() {
    let table = core::ptr::addr_of_mut!(HalPrivateDispatchTable) as PHAL_PRIVATE_DISPATCH;
    let ours = PciTranslateBusAddress as usize;
    if (*table).HalPciTranslateBusAddress.map(|f| f as usize) == Some(ours) {
        (*table).HalPciTranslateBusAddress = PcipSavedTranslateBusAddress;
    }
    PcipSavedTranslateBusAddress = None;
    PcipSavedAssignSlotResources = None;
}
