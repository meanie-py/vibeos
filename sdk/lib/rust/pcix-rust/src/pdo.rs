#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ddk::{IoCreateDevice, KeInitializeEvent, PCI_SLOT_NUMBER, PCM_RESOURCE_LIST, PDEVICE_OBJECT, PDEVICE_RELATIONS, PIO_RESOURCE_REQUIREMENTS_LIST, PIO_STACK_LOCATION, PIRP, POWER_STATE, PPNP_BUS_INFORMATION, RtlInitUnicodeString, _EVENT_TYPE_SynchronizationEvent};
use ntkrnl_sys::ddk::{
    _POWER_STATE_TYPE_DevicePowerState as DevicePowerState,
    _DEVICE_RELATION_TYPE_EjectionRelations as EjectionRelations,
    _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0,
    _SYSTEM_POWER_STATE_PowerSystemWorking as PowerSystemWorking,
    _DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation,
};
use core::ffi::c_void;
use core::ffi::{c_char, c_int, c_uint};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{PCSTR, RtlAssert, _DEVICE_OBJECT, _UNICODE_STRING};

extern "system" {
    fn DbgBreakPoint();
}
extern "C" {
    fn DbgPrint(format: *const i8, ...) -> NTSTATUS;
}

extern "system" {
    fn PoSetPowerState(
        DeviceObject: PDEVICE_OBJECT,
        Type: ntkrnl_sys::ddk::_POWER_STATE_TYPE,
        State: POWER_STATE,
    ) -> POWER_STATE;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut PciPdoSequenceNumber: LONG = 0;

pub static mut PciPdoDispatchPowerTable: [PCI_MN_DISPATCH_TABLE; 5] = unsafe {
    [
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DISPATCH,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoWaitWake as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoSetPowerState as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryPower as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
    ]
};

pub static mut PciPdoDispatchPnpTable: [PCI_MN_DISPATCH_TABLE; 26] = unsafe {
    [
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpStartDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryRemoveDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpRemoveDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpCancelRemoveDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpStopDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryStopDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpCancelStopDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryDeviceRelations as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryInterface as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryCapabilities as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryResources as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryResourceRequirements as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryDeviceText as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpReadConfig as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpWriteConfig as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryId as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryDeviceState as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryBusInformation as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpDeviceUsageNotification as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpSurpriseRemoval as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciPdoIrpQueryLegacyBusInformation as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_COMPLETE,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
    ]
};

pub static mut PciPdoDispatchTable: PCI_MJ_DISPATCH_TABLE = unsafe {
    _PCI_MJ_DISPATCH_TABLE {
        PnpIrpMaximumMinorFunction: IRP_MN_QUERY_LEGACY_BUS_INFORMATION as ULONG,
        PnpIrpDispatchTable: &raw const PciPdoDispatchPnpTable as PPCI_MN_DISPATCH_TABLE,
        PowerIrpMaximumMinorFunction: IRP_MN_QUERY_POWER as ULONG,
        PowerIrpDispatchTable: &raw const PciPdoDispatchPowerTable as PPCI_MN_DISPATCH_TABLE,
        SystemControlIrpDispatchStyle: IRP_COMPLETE,
        SystemControlIrpDispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        OtherIrpDispatchStyle: IRP_COMPLETE,
        OtherIrpDispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpInvalidDeviceRequest as *const ()),
    }
};
#[no_mangle]
pub unsafe extern "system" fn PciPdoWaitWake(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    let _ = Irp;
    let _ = IoStackLocation;
    let _ = DeviceExtension;
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"PciPdoWaitWake\0".as_ptr() as *const i8,
        b"drivers/bus/pcix/pdo.c\0".as_ptr() as *const i8,
        87i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoSetPowerState(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    let _ = Irp;
    let _ = IoStackLocation;
    let _ = DeviceExtension;

    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
        b"PciPdoSetPowerState\0".as_ptr() as *const core::ffi::c_char,
        b"drivers/bus/pcix/pdo.c\0".as_ptr() as *const core::ffi::c_char,
        101 as core::ffi::c_int,
    );

    return STATUS_NOT_SUPPORTED;
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryPower(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"PciPdoIrpQueryPower\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/pdo.c\0".as_ptr()
            as *const ::core::ffi::c_char,
        115 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpStartDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as c_uint,
        );
        if false {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/pdo.c\0".as_ptr()
                    as PVOID,
                128u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    let mut Status: NTSTATUS;
    let Changed: BOOLEAN;
    let mut DoReset: BOOLEAN;

    DoReset = 0u8;
    Status = PciBeginStateTransition(
        DeviceExtension as PVOID as PPCI_FDO_EXTENSION,
        PciStarted,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if ((*DeviceExtension).BaseClass as u32 == PCI_CLASS_PRE_20
        && (*DeviceExtension).SubClass as u32 == PCI_SUBCLASS_PRE_20_VGA)
        || ((*DeviceExtension).BaseClass as u32 == PCI_CLASS_DISPLAY_CTLR
            && (*DeviceExtension).SubClass as u32 == PCI_SUBCLASS_VID_VGA_CTLR)
    {
        (*DeviceExtension).CommandEnables =
            ((*DeviceExtension).CommandEnables as u32 | (PCI_ENABLE_IO_SPACE | PCI_ENABLE_MEMORY_SPACE)) as u16;
    }

    if (*DeviceExtension).IoSpaceUnderNativeIdeControl != 0 {
        (*DeviceExtension).CommandEnables =
            ((*DeviceExtension).CommandEnables as u32 & !PCI_ENABLE_IO_SPACE) as u16;
    }

    (*DeviceExtension).CommandEnables =
        ((*DeviceExtension).CommandEnables as u32 | PCI_ENABLE_BUS_MASTER) as u16;

    Changed = PciComputeNewCurrentSettings(
        DeviceExtension,
        (*IoStackLocation).Parameters.StartDevice.AllocatedResources,
    );
    if Changed != 0 {
        (*DeviceExtension).MovedDevice = 1u8;
    } else if DbgPrint(
        b"(%s:%d) PCI - START not changing resource settings.\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/pdo.c\0".as_ptr()
            as *const i8,
        171i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/pdo.c\0".as_ptr()
                as *const i8,
            171i32,
        );
    }

    if (*DeviceExtension).PowerState.CurrentDeviceState as u32
        != PowerDeviceD0 as u32
    {
        Status = PciSetPowerManagedDevicePowerState(
            DeviceExtension,
            PowerDeviceD0,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            PciCancelStateTransition(
                DeviceExtension as PVOID as PPCI_FDO_EXTENSION,
                PciStarted,
            );
            return STATUS_DEVICE_POWER_FAILURE;
        }

        let mut PowerState: POWER_STATE = core::mem::zeroed();
        PowerState.DeviceState = PowerDeviceD0;
        PoSetPowerState(
            (*DeviceExtension).PhysicalDeviceObject as *mut _DEVICE_OBJECT,
            DevicePowerState,
            PowerState,
        );

        (*DeviceExtension).PowerState.CurrentDeviceState = PowerDeviceD0;
        DoReset = 1u8;
    }

    Status = PciSetResources(DeviceExtension, DoReset, 1u8);
    if !NT_SUCCESS(Status) {
        PciCancelStateTransition(
            DeviceExtension as PVOID as PPCI_FDO_EXTENSION,
            PciStarted,
        );
    } else {
        PciCommitStateTransition(
            DeviceExtension as PVOID as PPCI_FDO_EXTENSION,
            PciStarted,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryRemoveDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"PciPdoIrpQueryRemoveDevice\0".as_ptr() as *const i8,
        b"drivers/bus/pcix/pdo.c\0".as_ptr()
            as *const i8,
        228 as i32,
    );
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpRemoveDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"PciPdoIrpRemoveDevice\0" as *const u8 as *const core::ffi::c_char,
        b"drivers/bus/pcix/pdo.c\0" as *const u8
            as *const core::ffi::c_char,
        242 as core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as PCSTR);
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpCancelRemoveDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    let _ = Irp;
    let _ = IoStackLocation;
    let _ = DeviceExtension;

    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"PciPdoIrpCancelRemoveDevice\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/pdo.c\0".as_ptr()
            as *const ::core::ffi::c_char,
        256 as ::core::ffi::c_int,
    );
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpStopDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    let _ = Irp;
    let _ = IoStackLocation;
    let _ = DeviceExtension;

    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"PciPdoIrpStopDevice\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/pdo.c\0".as_ptr()
            as *const ::core::ffi::c_char,
        270 as ::core::ffi::c_int,
    );
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryStopDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"PciPdoIrpQueryStopDevice\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/pdo.c\0".as_ptr() as *const ::core::ffi::c_char,
        284 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpCancelStopDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"PciPdoIrpCancelStopDevice\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/pdo.c\0".as_ptr() as *const ::core::ffi::c_char,
        298 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryInterface(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    let _ = Irp;
    let _ = IoStackLocation;
    let _ = DeviceExtension;

    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"PciPdoIrpQueryInterface\0".as_ptr() as *const c_char,
        b"drivers/bus/pcix/pdo.c\0".as_ptr() as *const c_char,
        312 as c_int,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
    return STATUS_NOT_SUPPORTED;
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryDeviceRelations(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    // PAGED_CODE() implementation
    let irql = KeGetCurrentIrql();
    if irql > APC_LEVEL as u8 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            irql as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/pdo.c\0".as_ptr() as PVOID,
            323u32,
            core::ptr::null_mut::<i8>(),
        );
    }

    let status: NTSTATUS;
    if (*IoStackLocation).Parameters.QueryDeviceRelations.Type as u32
        == EjectionRelations as u32
    {
        status = PciQueryEjectionRelations(
            DeviceExtension,
            core::ptr::addr_of_mut!((*Irp).IoStatus.Information) as *mut PDEVICE_RELATIONS,
        );
    } else if (*IoStackLocation).Parameters.QueryDeviceRelations.Type as u32
        == TargetDeviceRelation as u32
    {
        status = PciQueryTargetDeviceRelations(
            DeviceExtension,
            core::ptr::addr_of_mut!((*Irp).IoStatus.Information) as *mut PDEVICE_RELATIONS,
        );
    } else {
        status = STATUS_NOT_SUPPORTED;
    }
    status
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryCapabilities(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        let _ = DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/pdo.c\0".as_ptr() as PVOID,
                356u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    PciQueryCapabilities(
        DeviceExtension,
        (*IoStackLocation).Parameters.DeviceCapabilities.Capabilities,
    )
}

#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryResources(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    // PAGED_CODE()
    if (KeGetCurrentIrql() as i32) > APC_LEVEL as i32 {
        DbgPrint(
            c"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if true {
            RtlAssert(
                core::ptr::null_mut::<u8>() as PVOID,
                b"drivers/bus/pcix/pdo.c\0".as_ptr() as PVOID,
                372 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    let info_ptr = core::ptr::addr_of_mut!((*Irp).IoStatus.Information) as *mut PCM_RESOURCE_LIST;
    PciQueryResources(DeviceExtension, info_ptr)
}

#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryResourceRequirements(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const CHAR as PVOID,
                b"drivers/bus/pcix/pdo.c\0" as *const u8 as *const CHAR as PVOID,
                387u32,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    let _ = IoStackLocation;

    PciQueryRequirements(
        DeviceExtension,
        core::ptr::addr_of_mut!((*Irp).IoStatus.Information) as *mut PIO_RESOURCE_REQUIREMENTS_LIST,
    )
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryDeviceText(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/pdo.c\0".as_ptr() as PVOID,
                403 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    PciQueryDeviceText(
        DeviceExtension,
        (*IoStackLocation).Parameters.QueryDeviceText.DeviceTextType,
        (*IoStackLocation).Parameters.QueryDeviceText.LocaleId as ULONG,
        core::ptr::addr_of_mut!((*Irp).IoStatus.Information) as *mut PWCHAR,
    )
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryId(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    if (KeGetCurrentIrql() as i32) > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                b"drivers/bus/pcix/pdo.c\0" as *const u8
                    as *const i8 as *mut core::ffi::c_void,
                420 as u32,
                core::ptr::null_mut(),
            );
        }
    }
    PciQueryId(
        DeviceExtension,
        (*IoStackLocation).Parameters.QueryId.IdType,
        core::ptr::addr_of_mut!((*Irp).IoStatus.Information) as *mut PWCHAR,
    )
}

#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryBusInformation(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    if (KeGetCurrentIrql() as i32) > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if (0 as i32) == 0 {
            RtlAssert(
                b"0\0" as *const u8 as PVOID,
                b"drivers/bus/pcix/pdo.c\0" as *const u8
                    as PVOID,
                434 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
    }
    let _ = IoStackLocation;
    PciQueryBusInformation(
        DeviceExtension,
        &raw mut (*Irp).IoStatus.Information as *mut PPNP_BUS_INFORMATION,
    )
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpReadConfig(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"PciPdoIrpReadConfig\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/pdo.c\0".as_ptr() as *const ::core::ffi::c_char,
        454 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpWriteConfig(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"PciPdoIrpWriteConfig\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/pdo.c\0".as_ptr() as PCSTR,
        468i32,
    );
    DbgPrint(b"\0".as_ptr() as PCSTR);
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryDeviceState(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    // UNREFERENCED_PARAMETER macros
    let _ = Irp;
    let _ = IoStackLocation;
    let _ = DeviceExtension;

    // UNIMPLEMENTED macro expansion
    DbgPrint(
        (b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const u8).cast(),
        b"PciPdoIrpQueryDeviceState\0".as_ptr() as *const u8,
        b"drivers/bus/pcix/pdo.c\0".as_ptr() as *const u8,
        482 as ::core::ffi::c_int,
    );

    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpDeviceUsageNotification(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"PciPdoIrpDeviceUsageNotification\0" as *const u8 as PCSTR,
        b"drivers/bus/pcix/pdo.c\0" as *const u8 as PCSTR,
        496i32,
    );
    DbgPrint(b"\0" as *const u8 as PCSTR);
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpSurpriseRemoval(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    UNREFERENCED_PARAMETER(Irp);
    UNREFERENCED_PARAMETER(IoStackLocation);
    UNREFERENCED_PARAMETER(DeviceExtension);
    UNIMPLEMENTED_DBGBREAK();
    return STATUS_NOT_SUPPORTED;
}
#[no_mangle]
pub unsafe extern "system" fn PciPdoIrpQueryLegacyBusInformation(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_PDO_EXTENSION,
) -> NTSTATUS {
    let error_msg = b"ERROR:  %s at %s:%d is unimplemented!()\n\0";
    let func_name = b"PciPdoIrpQueryLegacyBusInformation\0";
    let file_name = b"drivers/bus/pcix/pdo.c\0";
    let line: i32 = 524;
    DbgPrint((error_msg.as_ptr() as *const u8).cast(),
             func_name.as_ptr() as *const i8,
             file_name.as_ptr() as *const i8,
             line);
    DbgPrint((b"\0".as_ptr() as *const u8).cast());
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn PciPdoCreate(
    DeviceExtension: PPCI_FDO_EXTENSION,
    Slot: PCI_SLOT_NUMBER,
    PdoDeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut DeviceName: [u16; 32] = [0u16; 32];
    let mut DeviceString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut PdoExtension: PPCI_PDO_EXTENSION;
    let SequenceNumber: ULONG;

    if KeGetCurrentIrql() as u32 > APC_LEVEL as u32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/bus/pcix/pdo.c\0" as *const u8 as *const i8 as PVOID,
                540 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    SequenceNumber = InterlockedIncrement(&raw mut PciPdoSequenceNumber as *mut i32) as ULONG;

    let format_str: [u16; 22] = core::mem::transmute::<[u8; 44], [u16; 22]>(
        *b"\\\0D\0e\0v\0i\0c\0e\0\\\0N\0T\0P\0N\0P\0_\0P\0C\0I\0%\x000\x004\0d\0\0\0",
    );
    swprintf(
        DeviceName.as_mut_ptr(),
        format_str.as_ptr(),
        SequenceNumber,
    );

    RtlInitUnicodeString(
        &raw mut DeviceString,
        DeviceName.as_mut_ptr() as PCWSTR,
    );

    let Status = IoCreateDevice(
        (*(*DeviceExtension).FunctionalDeviceObject).DriverObject,
        core::mem::size_of::<PCI_PDO_EXTENSION>() as ULONG,
        &raw mut DeviceString,
        FILE_DEVICE_BUS_EXTENDER as ULONG,
        0 as ULONG,
        0 as BOOLEAN,
        &raw mut DeviceObject,
    );
    if !(Status >= 0) {
        RtlAssert(
            b"NT_SUCCESS(Status)\0" as *const u8 as *const i8 as PVOID,
            b"drivers/bus/pcix/pdo.c\0" as *const u8 as *const i8 as PVOID,
            557 as ULONG,
            core::ptr::null_mut::<i8>(),
        );
    }

    PdoExtension = (*DeviceObject).DeviceExtension as PPCI_PDO_EXTENSION;

    if DbgPrint(
        b"(%s:%d) PCI: New PDO (b=0x%x, d=0x%x, f=0x%x) @ %p, ext @ %p\n\0" as *const u8 as PCSTR,
        b"drivers/bus/pcix/pdo.c\0" as *const u8 as *const i8,
        566 as i32,
        (*DeviceExtension).BaseBus as i32,
        Slot.u.bits.DeviceNumber() as i32,
        Slot.u.bits.FunctionNumber() as i32,
        DeviceObject,
        (*DeviceObject).DeviceExtension,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/bus/pcix/pdo.c\0" as *const u8 as *const i8,
            566 as i32,
        );
    }

    (*PdoExtension).ExtensionType = PciPdoExtensionType as ULONG;
    (*PdoExtension).IrpDispatchTable = &raw mut PciPdoDispatchTable as *mut PCI_MJ_DISPATCH_TABLE;
    (*PdoExtension).PhysicalDeviceObject = DeviceObject;
    (*PdoExtension).Slot = Slot;
    (*PdoExtension).PowerState.CurrentSystemState = PowerSystemWorking;
    (*PdoExtension).PowerState.CurrentDeviceState = PowerDeviceD0;
    (*PdoExtension).ParentFdoExtension = DeviceExtension;

    KeInitializeEvent(
        &raw mut (*PdoExtension).SecondaryExtLock,
        _EVENT_TYPE_SynchronizationEvent,
        1u8,
    );

    PciInitializeState(PdoExtension as PPCI_FDO_EXTENSION);

    (*PdoExtension).Next = core::ptr::null_mut::<core::ffi::c_void>();
    PciInsertEntryAtTail(
        (&raw mut (*DeviceExtension).ChildPdoList as PSINGLE_LIST_ENTRY).cast(),
        PdoExtension as PPCI_FDO_EXTENSION,
        &raw mut (*DeviceExtension).ChildListLock,
    );

    *PdoDeviceObject = DeviceObject;
    STATUS_SUCCESS
}
