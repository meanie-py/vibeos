#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoAttachDeviceToDeviceStack, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoOpenDeviceRegistryKey, KEY_VALUE_PARTIAL_INFORMATION, KeInitializeEvent, PCM_RESOURCE_LIST, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, PKEY_VALUE_PARTIAL_INFORMATION, RtlInitUnicodeString, _EVENT_TYPE_SynchronizationEvent, ZwClose, ZwQueryValueKey};
use ntkrnl_sys::ddk::{
    _DEVICE_RELATION_TYPE_BusRelations as BusRelations,
    DEVICE_REGISTRY_PROPERTY_DevicePropertyBootConfiguration as DevicePropertyBootConfiguration,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as KeyValuePartialInformation,
    _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0,
    _SYSTEM_POWER_STATE_PowerSystemWorking as PowerSystemWorking,
};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{ACCESS_MASK, DEVICE_POWER_STATE, PCSTR, _SINGLE_LIST_ENTRY, _UNICODE_STRING};

extern "system" {
    fn DbgBreakPoint();
    fn RtlAssert(pvFailedAssertion: PVOID, pvFileName: PVOID, ulLineNumber: ULONG, pszMessage: *mut ::core::ffi::c_char);
}
extern "C" {
    fn DbgPrint(format: *const u8, ...) -> ULONG;
}
unsafe extern "system" {
    fn PciSendIoctl( DeviceObject: PDEVICE_OBJECT, Ioctl: ULONG, InputBuffer: PVOID, InputBufferLength: ULONG, OutputBuffer: PVOID, OutputBufferLength: ULONG, ) -> NTSTATUS;
}

// ---- ACPI ioctl private types (from sdk/include/ddk/acpiioct.h) ----
// Not bound by gen_driver_types (acpiioct.h not in pcix allowlist); defined here
// faithfully so the c2rust references resolve. Layout matches the C exactly.
#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed_172 {
    pub MethodName: [UCHAR; 4usize],
    pub MethodNameAsUlong: ULONG,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _ACPI_EVAL_INPUT_BUFFER {
    pub Signature: ULONG,
    pub c2rust_unnamed: C2RustUnnamed_172,
}
pub type ACPI_EVAL_INPUT_BUFFER = _ACPI_EVAL_INPUT_BUFFER;
pub type PACPI_EVAL_INPUT_BUFFER = *mut _ACPI_EVAL_INPUT_BUFFER;

#[repr(C)]
#[derive(Copy, Clone)]
pub union _ACPI_METHOD_ARGUMENT_u {
    pub Argument: ULONG,
    pub Data: [UCHAR; 1usize],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _ACPI_METHOD_ARGUMENT {
    pub Type: USHORT,
    pub DataLength: USHORT,
    pub c2rust_unnamed: _ACPI_METHOD_ARGUMENT_u,
}
pub type ACPI_METHOD_ARGUMENT = _ACPI_METHOD_ARGUMENT;
pub type PACPI_METHOD_ARGUMENT = *mut _ACPI_METHOD_ARGUMENT;

#[repr(C)]
pub struct _ACPI_EVAL_OUTPUT_BUFFER {
    pub Signature: ULONG,
    pub Length: ULONG,
    pub Count: ULONG,
    pub Argument: [ACPI_METHOD_ARGUMENT; 1usize],
}
pub type ACPI_EVAL_OUTPUT_BUFFER = _ACPI_EVAL_OUTPUT_BUFFER;
pub type PACPI_EVAL_OUTPUT_BUFFER = *mut _ACPI_EVAL_OUTPUT_BUFFER;

// 'BieA' little-endian ULONG
pub const ACPI_EVAL_INPUT_BUFFER_SIGNATURE: ULONG = 0x42696541; // 'BieA' (acpiioct.h:7)
// CTL_CODE(FILE_DEVICE_ACPI=0x32, 1, METHOD_BUFFERED=0, FILE_READ_ACCESS|FILE_WRITE_ACCESS=3)
pub const IOCTL_ACPI_EVAL_METHOD: ULONG = 0x0032C004;

// pci.h macro: #define PCI_IS_ROOT_FDO(x) ((x)->BusRootFdoExtension == x)
#[inline]
unsafe fn PCI_IS_ROOT_FDO(x: PPCI_FDO_EXTENSION) -> bool {
    (*x).BusRootFdoExtension == x
}
// bugcodes.mc: MessageId=0xA1 SymbolicName=PCI_BUS_DRIVER_INTERNAL
pub const PCI_BUS_DRIVER_INTERNAL: ULONG = 0xA1;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut PciFdoExtensionListHead: SINGLE_LIST_ENTRY = SINGLE_LIST_ENTRY {
    Next: (::core::ptr::null::<_SINGLE_LIST_ENTRY>() as *mut _SINGLE_LIST_ENTRY).cast(),
};

pub static mut PciBreakOnDefault: BOOLEAN = 0;

pub static mut PciFdoDispatchPowerTable: [PCI_MN_DISPATCH_TABLE; 5] = unsafe {
    [
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DISPATCH,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoWaitWake as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoSetPowerState as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpQueryPower as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
    ]
};

pub static mut PciFdoDispatchPnpTable: [PCI_MN_DISPATCH_TABLE; 26] = unsafe {
    [
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_UPWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpStartDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpQueryRemoveDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DISPATCH,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpRemoveDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpCancelRemoveDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpStopDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpQueryStopDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpCancelStopDevice as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpQueryDeviceRelations as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DISPATCH,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpQueryInterface as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_UPWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpQueryCapabilities as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_UPWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpDeviceUsageNotification as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpSurpriseRemoval as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciFdoIrpQueryLegacyBusInformation as *const ()),
        },
        _PCI_MN_DISPATCH_TABLE {
            DispatchStyle: IRP_DOWNWARD,
            DispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        },
    ]
};

pub static mut PciFdoDispatchTable: PCI_MJ_DISPATCH_TABLE = unsafe {
    _PCI_MJ_DISPATCH_TABLE {
        PnpIrpMaximumMinorFunction: IRP_MN_QUERY_LEGACY_BUS_INFORMATION as ULONG,
        PnpIrpDispatchTable: &raw const PciFdoDispatchPnpTable as PPCI_MN_DISPATCH_TABLE,
        PowerIrpMaximumMinorFunction: IRP_MN_QUERY_POWER as ULONG,
        PowerIrpDispatchTable: &raw const PciFdoDispatchPowerTable as PPCI_MN_DISPATCH_TABLE,
        SystemControlIrpDispatchStyle: IRP_DOWNWARD,
        SystemControlIrpDispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
        OtherIrpDispatchStyle: IRP_DOWNWARD,
        OtherIrpDispatchFunction: ::core::mem::transmute::<*const (), PCI_DISPATCH_FUNCTION>(PciIrpNotSupported as *const ()),
    }
};
#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpStartDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let Resources: PCM_RESOURCE_LIST;

    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8).cast(),
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/bus/pcix/fdo.c\0".as_ptr() as *const i8 as PVOID,
            82 as ULONG,
            core::ptr::null_mut::<i8>(),
        );
    }

    if !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
        return STATUS_NOT_SUPPORTED;
    }

    Status = PciBeginStateTransition(DeviceExtension, PciStarted);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Resources = (*IoStackLocation).Parameters.StartDevice.AllocatedResources;
    if !Resources.is_null() && !((*DeviceExtension).BusRootFdoExtension == DeviceExtension) {
        ntkrnl_sys::ASSERT!((*Resources).Count == 1);
        DbgPrint(
            (b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8).cast(),
            b"PciFdoIrpStartDevice\0".as_ptr() as *const i8,
            b"drivers/bus/pcix/fdo.c\0".as_ptr() as *const i8,
            97 as i32,
        );
        DbgPrint((b"\0".as_ptr() as *const i8).cast());
        DbgBreakPoint();
    }

    Status = PciInitializeArbiterRanges(DeviceExtension, Resources);
    if !NT_SUCCESS(Status) {
        PciCancelStateTransition(DeviceExtension, PciStarted);
        return Status;
    }

    if !Resources.is_null() && !((*DeviceExtension).BusRootFdoExtension == DeviceExtension) {
        ntkrnl_sys::ASSERT!((*Resources).Count == 1);
        DbgPrint(
            (b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8).cast(),
            b"PciFdoIrpStartDevice\0".as_ptr() as *const i8,
            b"drivers/bus/pcix/fdo.c\0".as_ptr() as *const i8,
            114 as i32,
        );
        DbgPrint((b"\0".as_ptr() as *const i8).cast());
        DbgBreakPoint();
    }

    PciCommitStateTransition(DeviceExtension, PciStarted);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpQueryRemoveDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {

    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const u8,
        b"PciFdoIrpQueryRemoveDevice\0".as_ptr() as *const i8,
        b"drivers/bus/pcix/fdo.c\0".as_ptr() as *const i8,
        132i32,
    );
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpRemoveDevice(
    _Irp: PIRP,
    _IoStackLocation: PIO_STACK_LOCATION,
    _DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        (b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const ::core::ffi::c_char).cast(),
        b"PciFdoIrpRemoveDevice\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/fdo.c\0".as_ptr()
            as *const ::core::ffi::c_char,
        146 as ::core::ffi::c_int,
    );
    DbgPrint((b"\0".as_ptr() as *const ::core::ffi::c_char).cast());
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpCancelRemoveDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        (b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const ::core::ffi::c_char).cast(),
        b"PciFdoIrpCancelRemoveDevice\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/fdo.c\0".as_ptr()
            as *const ::core::ffi::c_char,
        160i32,
    );
    DbgPrint((b"\0".as_ptr() as *const ::core::ffi::c_char).cast());
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpStopDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    let _ = (Irp, IoStackLocation, DeviceExtension);
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr(),
        b"PciFdoIrpStopDevice\0".as_ptr(),
        b"drivers/bus/pcix/fdo.c\0".as_ptr(),
        174i32,
    );
    DbgPrint(b"\0".as_ptr());
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpQueryStopDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    let _ = (Irp, IoStackLocation, DeviceExtension);
    DbgPrint(
        (b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const core::ffi::c_char).cast(),
        b"PciFdoIrpQueryStopDevice\0" as *const u8 as *const core::ffi::c_char,
        b"drivers/bus/pcix/fdo.c\0" as *const u8 as *const core::ffi::c_char,
        188 as core::ffi::c_int,
    );
    DbgPrint((b"\0" as *const u8 as *const core::ffi::c_char).cast());
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpCancelStopDevice(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        (b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR).cast(),
        b"PciFdoIrpCancelStopDevice\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/fdo.c\0" as *const u8
            as *const ::core::ffi::c_char,
        202 as ::core::ffi::c_int,
    );
    DbgPrint((b"\0" as *const u8 as PCSTR).cast());
    DbgBreakPoint();
    return STATUS_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpQueryDeviceRelations(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8 as *const i8).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/bus/pcix/fdo.c\0".as_ptr()
                    as *const i8 as PVOID,
                213u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    if (*IoStackLocation).Parameters.QueryDeviceRelations.Type as u32 != BusRelations as u32 {
        Status = STATUS_NOT_SUPPORTED;
    } else {
        Status = PciQueryDeviceRelations(
            DeviceExtension,
            core::ptr::addr_of_mut!((*Irp).IoStatus.Information) as *mut PDEVICE_RELATIONS,
        );
    }
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpQueryInterface(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    // PAGED_CODE()
    if KeGetCurrentIrql() as u32 > APC_LEVEL as u32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8).cast(),
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *mut core::ffi::c_void,
            b"drivers/bus/pcix/fdo.c\0".as_ptr()
                as *mut core::ffi::c_void,
            240u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    if !((*DeviceExtension).ExtensionType == PciFdoExtensionType) {
        RtlAssert(
            b"DeviceExtension->ExtensionType == PciFdoExtensionType\0".as_ptr()
                as *mut core::ffi::c_void,
            b"drivers/bus/pcix/fdo.c\0".as_ptr()
                as *mut core::ffi::c_void,
            241u32,
            core::ptr::null_mut::<i8>(),
        );
    }

    if (*DeviceExtension).DeviceState as i32 == PciDeleted as i32 {
        return PciPassIrpFromFdoToPdo(DeviceExtension, Irp);
    }

    let mut Status: NTSTATUS = PciQueryInterface(
        DeviceExtension,
        (*IoStackLocation).Parameters.QueryInterface.InterfaceType,
        (*IoStackLocation).Parameters.QueryInterface.Size as u32,
        (*IoStackLocation).Parameters.QueryInterface.Version as u32,
        (*IoStackLocation)
            .Parameters
            .QueryInterface
            .InterfaceSpecificData,
        (*IoStackLocation).Parameters.QueryInterface.Interface,
        0u8, // FALSE
    );

    if Status >= 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        return PciPassIrpFromFdoToPdo(DeviceExtension, Irp);
    } else if Status == STATUS_NOT_SUPPORTED {
        Status = PciCallDownIrpStack(DeviceExtension, Irp);
        if Status == STATUS_NOT_SUPPORTED {
            Status = PciQueryInterface(
                DeviceExtension,
                (*IoStackLocation).Parameters.QueryInterface.InterfaceType,
                (*IoStackLocation).Parameters.QueryInterface.Size as u32,
                (*IoStackLocation).Parameters.QueryInterface.Version as u32,
                (*IoStackLocation)
                    .Parameters
                    .QueryInterface
                    .InterfaceSpecificData,
                (*IoStackLocation).Parameters.QueryInterface.Interface,
                1u8, // TRUE
            );
        }
    }

    if Status == STATUS_NOT_SUPPORTED {
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    } else {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    }

    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpQueryCapabilities(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    // PAGED_CODE()
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR).cast(),
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/fdo.c\0".as_ptr() as PVOID,
            315 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    // ASSERT_FDO(DeviceExtension)
    if !((*DeviceExtension).ExtensionType == (PciFdoExtensionType as ::core::ffi::c_int as ::core::ffi::c_ulong)) {
        RtlAssert(
            b"(DeviceExtension)->ExtensionType == PciFdoExtensionType\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/fdo.c\0".as_ptr() as PVOID,
            316 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    UNREFERENCED_PARAMETER!(Irp);

    let Capabilities: PDEVICE_CAPABILITIES = (*IoStackLocation).Parameters.DeviceCapabilities.Capabilities;

    (*DeviceExtension).PowerState.SystemWakeLevel = (*Capabilities).SystemWake;
    (*DeviceExtension).PowerState.DeviceWakeLevel = (*Capabilities).DeviceWake;

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*Capabilities).DeviceState) as *const DEVICE_POWER_STATE as *const u8,
        core::ptr::addr_of_mut!((*DeviceExtension).PowerState.SystemStateMapping) as *mut DEVICE_POWER_STATE as *mut u8,
        core::mem::size_of::<[DEVICE_POWER_STATE; 7]>(),
    );

    PciDebugDumpQueryCapabilities(Capabilities);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpDeviceUsageNotification(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const u8,
        b"PciFdoIrpDeviceUsageNotification\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/fdo.c\0".as_ptr() as *const ::core::ffi::c_char,
        345i32,
    );
    DbgPrint(b"\0".as_ptr() as *const u8);
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpSurpriseRemoval(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    // UNREFERENCED_PARAMETER macros are no-ops; parameters ignored.

    DbgPrint(
        (b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char).cast(),
        b"PciFdoIrpSurpriseRemoval\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
        359i32,
    );
    DbgPrint(
        (b"\0" as *const u8 as *const ::core::ffi::c_char).cast(),
    );
    DbgBreakPoint();

    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpQueryLegacyBusInformation(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        (b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8).cast(),
        b"PciFdoIrpQueryLegacyBusInformation\0" as *const u8 as *const i8,
        b"drivers/bus/pcix/fdo.c\0" as *const u8 as *const i8,
        373 as i32,
    );
    DbgPrint((b"\0" as *const u8 as *const i8).cast());
    DbgBreakPoint();
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciGetHotPlugParameters(FdoExtension: PPCI_FDO_EXTENSION) {

    // PAGED_CODE
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR).cast(),
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/fdo.c\0".as_ptr() as PVOID,
            385 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    let mut InputBuffer: ACPI_EVAL_INPUT_BUFFER = ACPI_EVAL_INPUT_BUFFER {
        Signature: 0,
        c2rust_unnamed: C2RustUnnamed_172 { MethodName: [0; 4] },
    };
    let mut OutputBuffer: PACPI_EVAL_OUTPUT_BUFFER;
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    Length = (core::mem::size_of::<ACPI_EVAL_OUTPUT_BUFFER>()
        .wrapping_add(4_usize.wrapping_mul(core::mem::size_of::<ACPI_METHOD_ARGUMENT>())))
        as ULONG;

    OutputBuffer =
        ExAllocatePoolWithTag(PagedPool, Length as SIZE_T, PCI_POOL_TAG as ULONG)
            as PACPI_EVAL_OUTPUT_BUFFER;
    if OutputBuffer.is_null() {
        return;
    }

    RtlZeroMemory(OutputBuffer as PVOID, Length as usize);

    * (core::ptr::addr_of_mut!(InputBuffer.c2rust_unnamed.MethodName) as *mut UCHAR as PULONG) =
        1347438687u32 as ULONG;
    InputBuffer.Signature = ACPI_EVAL_INPUT_BUFFER_SIGNATURE as ULONG;

    Status = PciSendIoctl(
        (*FdoExtension).PhysicalDeviceObject,
        IOCTL_ACPI_EVAL_METHOD as ULONG,
        core::ptr::addr_of_mut!(InputBuffer) as PVOID,
        core::mem::size_of::<ACPI_EVAL_INPUT_BUFFER>() as ULONG,
        OutputBuffer as PVOID,
        Length,
    );

    if !NT_SUCCESS(Status) {
        if !PCI_IS_ROOT_FDO(FdoExtension) {
            // C struct-copy (bitwise); field type is non-Copy so read it explicitly.
            (*FdoExtension).HotPlugParameters = core::ptr::read(core::ptr::addr_of!(
                (*(*FdoExtension).ParentFdoExtension).HotPlugParameters
            ));
        }
    } else if (*OutputBuffer).Count == 4u32 as u32 {
        'hpp: {
            /* Every one of them has to be an integer for the set to mean anything */
            let Argument: PACPI_METHOD_ARGUMENT =
                core::ptr::addr_of_mut!((*OutputBuffer).Argument) as PACPI_METHOD_ARGUMENT;
            let mut i: ULONG = 0;
            while i < 4 {
                if (*Argument.add(i as usize)).Type as u32 != ACPI_METHOD_ARGUMENT_INTEGER {
                    break;
                }
                i += 1;
            }
            if i != 4 {
                DbgPrint(
                    (b"(%s:%d) PCI - _HPP returned something other than four integers\n\0".as_ptr() as PCSTR).cast(),
                    b"drivers/bus/pcix/fdo.c\0".as_ptr() as PCSTR,
                    433 as i32,
                );
                break 'hpp;
            }

            /* Cache line size and latency timer are byte registers, SERR and PERR are flags */
            if (*Argument.add(0)).c2rust_unnamed.Argument > MAXUCHAR
                || (*Argument.add(1)).c2rust_unnamed.Argument > MAXUCHAR
                || (*Argument.add(2)).c2rust_unnamed.Argument > 1
                || (*Argument.add(3)).c2rust_unnamed.Argument > 1
            {
                DbgPrint(
                    (b"(%s:%d) PCI - _HPP returned values out of range, ignoring them\n\0".as_ptr() as PCSTR).cast(),
                    b"drivers/bus/pcix/fdo.c\0".as_ptr() as PCSTR,
                    444 as i32,
                );
                break 'hpp;
            }

            /* Settings for devices that later appear in a hot plug slot on this bus */
            (*FdoExtension).HotPlugParameters.CacheLineSize =
                (*Argument.add(0)).c2rust_unnamed.Argument as UCHAR;
            (*FdoExtension).HotPlugParameters.LatencyTimer =
                (*Argument.add(1)).c2rust_unnamed.Argument as UCHAR;
            (*FdoExtension).HotPlugParameters.EnableSERR =
                (*Argument.add(2)).c2rust_unnamed.Argument as BOOLEAN;
            (*FdoExtension).HotPlugParameters.EnablePERR =
                (*Argument.add(3)).c2rust_unnamed.Argument as BOOLEAN;
            (*FdoExtension).HotPlugParameters.Acquired = 1u8;

            DbgPrint(
                (b"(%s:%d) PCI - _HPP for FDO ext 0x%p: cache line %u, latency %u, SERR %u, PERR %u\n\0".as_ptr() as PCSTR).cast(),
                b"drivers/bus/pcix/fdo.c\0".as_ptr() as PCSTR,
                454 as i32,
                FdoExtension,
                (*FdoExtension).HotPlugParameters.CacheLineSize as u32,
                (*FdoExtension).HotPlugParameters.LatencyTimer as u32,
                (*FdoExtension).HotPlugParameters.EnableSERR as u32,
                (*FdoExtension).HotPlugParameters.EnablePERR as u32,
            );
        }
    }

    ExFreePoolWithTag(OutputBuffer as PVOID, 0 as ULONG);
}

#[no_mangle]
pub unsafe extern "system" fn PciInitializeFdoExtensionCommonFields(
    FdoExtension: PPCI_FDO_EXTENSION,
    DeviceObject: PDEVICE_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) {
    core::ptr::write_bytes(
        FdoExtension as *mut u8,
        0u8,
        core::mem::size_of::<PCI_FDO_EXTENSION>(),
    );
    (*FdoExtension).PhysicalDeviceObject = PhysicalDeviceObject;
    (*FdoExtension).FunctionalDeviceObject = DeviceObject;
    (*FdoExtension).ExtensionType = PciFdoExtensionType as i32 as u32;
    (*FdoExtension).PowerState.CurrentSystemState = PowerSystemWorking;
    (*FdoExtension).PowerState.CurrentDeviceState = PowerDeviceD0;
    (*FdoExtension).IrpDispatchTable =
        core::ptr::addr_of_mut!(PciFdoDispatchTable) as *mut PCI_MJ_DISPATCH_TABLE;
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*FdoExtension).SecondaryExtLock),
        _EVENT_TYPE_SynchronizationEvent,
        1u8,
    );
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*FdoExtension).ChildListLock),
        _EVENT_TYPE_SynchronizationEvent,
        1u8,
    );
    PciInitializeState(FdoExtension);
}

#[no_mangle]
pub unsafe extern "system" fn PciAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Descriptor: PCM_RESOURCE_LIST = core::ptr::null_mut();
    let mut AttachedTo: PDEVICE_OBJECT;
    let mut FdoExtension: PPCI_FDO_EXTENSION;
    let mut ParentExtension: PPCI_FDO_EXTENSION;
    let mut PdoExtension: PPCI_PDO_EXTENSION;
    let mut DeviceObject: PDEVICE_OBJECT ;
    let mut Buffer: [u8; 20] = [0u8; 20];
    let mut ValueInfo: PKEY_VALUE_PARTIAL_INFORMATION = &raw mut Buffer as *mut u8 as PKEY_VALUE_PARTIAL_INFORMATION;
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ResultLength: ULONG = 0;

    // PAGED_CODE();
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/bus/pcix/fdo.c\0".as_ptr()
                    as *const i8 as *mut core::ffi::c_void,
                473 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    // DPRINT1 for entry
    if DbgPrint(
        (b"(%s:%d) PCI - AddDevice (a new bus). PDO: %p (Driver: %wZ)\n\0".as_ptr() as PCSTR).cast(),
        b"drivers/bus/pcix/fdo.c\0".as_ptr() as *const i8,
        475i32,
        PhysicalDeviceObject,
        &raw mut (*(*PhysicalDeviceObject).DriverObject).DriverName,
    ) != 0
    {
        DbgPrint(
            (b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR).cast(),
            b"drivers/bus/pcix/fdo.c\0".as_ptr()
                as *const i8,
            475i32,
        );
    }

    AttachedTo = core::ptr::null_mut();
    FdoExtension = core::ptr::null_mut();
    PdoExtension = core::ptr::null_mut();
    DeviceObject = core::ptr::null_mut();

    let mut success = false;
    let mut fail_status = STATUS_SUCCESS;

    loop {
        ParentExtension =
            PciFindParentPciFdoExtension(PhysicalDeviceObject, &raw mut PciGlobalLock);
        if !ParentExtension.is_null() {
            PdoExtension = (*PhysicalDeviceObject).DeviceExtension as PPCI_PDO_EXTENSION;
            // ASSERT_PDO
            if !((*PdoExtension).ExtensionType == PciPdoExtensionType as u32) {
                RtlAssert(
                    b"(PdoExtension)->ExtensionType == PciPdoExtensionType\0".as_ptr() as *const i8
                        as *mut core::ffi::c_void,
                    b"drivers/bus/pcix/fdo.c\0".as_ptr()
                        as *const i8 as *mut core::ffi::c_void,
                    492 as ULONG,
                    core::ptr::null_mut::<i8>(),
                );
            }
            if (*PdoExtension).BaseClass as i32 != PCI_CLASS_BRIDGE_DEV as i32
                || (*PdoExtension).SubClass as i32 != PCI_SUBCLASS_BR_PCI_TO_PCI as i32
            {
                // DPRINT1 for non-bridge
                if DbgPrint(
                    (b"(%s:%d) PCI - PciAddDevice for Non-Root/Non-PCI-PCI bridge,\n      Class %02x, SubClass %02x, will not add.\n\0"
                        .as_ptr() as PCSTR).cast(),
                    b"drivers/bus/pcix/fdo.c\0".as_ptr()
                        as *const i8,
                    502i32,
                    (*PdoExtension).BaseClass as i32,
                    (*PdoExtension).SubClass as i32,
                ) != 0
                {
                    DbgPrint(
                        (b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR).cast(),
                        b"drivers/bus/pcix/fdo.c\0"
                            .as_ptr() as *const i8,
                        502i32,
                    );
                }
                // ASSERT
                if !((*PdoExtension).BaseClass as i32 == 0x6i32
                    && (*PdoExtension).SubClass as i32 == 0x4i32)
                {
                    RtlAssert(
                        b"(PdoExtension->BaseClass == PCI_CLASS_BRIDGE_DEV) && (PdoExtension->SubClass == PCI_SUBCLASS_BR_PCI_TO_PCI)\0"
                            .as_ptr() as *const i8 as *mut core::ffi::c_void,
                        b"drivers/bus/pcix/fdo.c\0"
                            .as_ptr() as *const i8 as *mut core::ffi::c_void,
                        504 as ULONG,
                        core::ptr::null_mut::<i8>(),
                    );
                }
                Status = STATUS_INVALID_DEVICE_REQUEST;
                break;
            }
            // DPRINT1 for child bus
            if DbgPrint(
                (b"(%s:%d) PCI - AddDevice (new bus is child of bus 0x%x).\n\0".as_ptr() as PCSTR).cast(),
                b"drivers/bus/pcix/fdo.c\0".as_ptr()
                    as *const i8,
                513i32,
                (*ParentExtension).BaseBus as i32,
            ) != 0
            {
                DbgPrint(
                    (b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR).cast(),
                    b"drivers/bus/pcix/fdo.c\0".as_ptr()
                        as *const i8,
                    513i32,
                );
            }
            if PciAreBusNumbersConfigured(PdoExtension) == 0 {
                // DPRINT1 for unconfigured
                if DbgPrint(
                    (b"(%s:%d) PCI - Bus numbers not configured for bridge (0x%x.0x%x.0x%x)\n\0"
                        .as_ptr() as PCSTR).cast(),
                    b"drivers/bus/pcix/fdo.c\0".as_ptr()
                        as *const i8,
                    522i32,
                    (*ParentExtension).BaseBus as i32,
                    (*PdoExtension).Slot.u.bits.DeviceNumber() as i32,
                    (*PdoExtension).Slot.u.bits.FunctionNumber() as i32,
                ) != 0
                {
                    DbgPrint(
                        (b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR).cast(),
                        b"drivers/bus/pcix/fdo.c\0"
                            .as_ptr() as *const i8,
                        522i32,
                    );
                }
                Status = STATUS_INVALID_DEVICE_REQUEST;
                break;
            }
        }

        // IoCreateDevice
        Status = IoCreateDevice(
            DriverObject,
            core::mem::size_of::<PCI_FDO_EXTENSION>() as ULONG,
            core::ptr::null_mut(),
            FILE_DEVICE_BUS_EXTENDER as ULONG,
            0,
            0 as BOOLEAN,
            &raw mut DeviceObject,
        );
        if Status >= 0i32 {
            FdoExtension = (*DeviceObject).DeviceExtension as PPCI_FDO_EXTENSION;
            PciInitializeFdoExtensionCommonFields(
                (*DeviceObject).DeviceExtension as PPCI_FDO_EXTENSION,
                DeviceObject,
                PhysicalDeviceObject,
            );
            Status = STATUS_NO_SUCH_DEVICE;
            AttachedTo = IoAttachDeviceToDeviceStack(DeviceObject, PhysicalDeviceObject);
            if AttachedTo.is_null() {
                RtlAssert(
                    b"AttachedTo != NULL\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                    b"drivers/bus/pcix/fdo.c\0".as_ptr()
                        as *const i8 as *mut core::ffi::c_void,
                    550 as ULONG,
                    core::ptr::null_mut::<i8>(),
                );
            }
            if !AttachedTo.is_null() {
                (*FdoExtension).AttachedDeviceObject = AttachedTo;
                if !ParentExtension.is_null() {
                    (*FdoExtension).BaseBus =
                        (*PdoExtension).Dependent.type1.as_ref().SecondaryBus as BOOLEAN;
                    (*FdoExtension).BusRootFdoExtension = (*ParentExtension).BusRootFdoExtension;
                    (*PdoExtension).BridgeFdoExtension = FdoExtension as *mut PCI_FDO_EXTENSION;
                    (*FdoExtension).ParentFdoExtension = ParentExtension as *mut PCI_FDO_EXTENSION;
                } else {
                    Status = PciGetDeviceProperty(
                        PhysicalDeviceObject,
                        DevicePropertyBootConfiguration,
                        &raw mut Descriptor as *mut PVOID,
                    );
                    if !(Status >= 0i32) {
                        Descriptor = core::ptr::null_mut();
                    } else {
                        DbgPrint(
                            (b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR).cast(),
                            b"PciAddDevice\0".as_ptr() as *const i8,
                            b"drivers/bus/pcix/fdo.c\0"
                                .as_ptr() as *const i8,
                            577i32,
                        );
                        DbgPrint((b"Encountered during setup\n\0".as_ptr() as PCSTR).cast());
                        DbgBreakPoint();
                        Descriptor = core::ptr::null_mut();
                    }
                    if !Descriptor.is_null() {
                        DbgPrint(
                            (b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR).cast(),
                            b"PciAddDevice\0".as_ptr() as *const i8,
                            b"drivers/bus/pcix/fdo.c\0"
                                .as_ptr() as *const i8,
                            584i32,
                        );
                        DbgPrint((b"\0".as_ptr() as PCSTR).cast());
                        DbgBreakPoint();
                    } else {
                        if PciBreakOnDefault != 0u8 {
                            KeBugCheckEx(
                                PCI_BUS_DRIVER_INTERNAL,
                                0xdead0010usize as ULONG_PTR,
                                DeviceObject as ULONG_PTR,
                                0usize as ULONG_PTR,
                                0usize as ULONG_PTR,
                            );
                        }
                        if DbgPrint(
                            (b"(%s:%d) PCI   Will use default configuration.\n\0".as_ptr() as PCSTR).cast(),
                            b"drivers/bus/pcix/fdo.c\0"
                                .as_ptr() as *const i8,
                            600i32,
                        ) != 0
                        {
                            DbgPrint(
                                (b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR).cast(),
                                b"drivers/bus/pcix/fdo.c\0"
                                    .as_ptr() as *const i8,
                                600i32,
                            );
                        }
                        PciBreakOnDefault = 1u8;
                        (*FdoExtension).BaseBus = 0u8;
                    }
                    (*FdoExtension).BusRootFdoExtension =
                        FdoExtension as *mut PCI_FDO_EXTENSION;
                }
                Status = PciGetConfigHandlers(FdoExtension);
                if Status >= 0i32 {
                    /* Map this bus for extended configuration space access */
                    PciInitializeEcam(FdoExtension);

                    Status = PciInitializeArbiters(FdoExtension);
                    if Status >= 0i32 {
                        (*FdoExtension).Fake = 0u8;
                        PciInsertEntryAtTail(
                            (&raw mut PciFdoExtensionListHead).cast(),
                            FdoExtension,
                            &raw mut PciGlobalLock,
                        );
                        IoOpenDeviceRegistryKey(
                            DeviceObject,
                            PLUGPLAY_REGKEY_DEVICE as ULONG,
                            KEY_ALL_ACCESS as ACCESS_MASK,
                            &raw mut KeyHandle,
                        );
                        let HackFlagsValueName: [core::ffi::c_ushort; 10] =
                            ::core::mem::transmute::<[u8; 20], [core::ffi::c_ushort; 10]>(
                                *b"H\0a\0c\0k\0F\0l\0a\0g\0s\0\0\0",
                            );
                        RtlInitUnicodeString(
                            &raw mut ValueName,
                            HackFlagsValueName.as_ptr() as PCWSTR,
                        );
                        Status = ZwQueryValueKey(
                            KeyHandle,
                            &raw mut ValueName,
                            KeyValuePartialInformation,
                            ValueInfo as PVOID,
                            core::mem::size_of::<[u8; 20]>() as ULONG,
                            &raw mut ResultLength,
                        );
                        ZwClose(KeyHandle);
                        if Status >= 0i32 {
                            if (*ValueInfo).Type == REG_DWORD as u32
                                && (*ValueInfo).DataLength as usize
                                    == core::mem::size_of::<ULONG>() as usize
                            {
                                (*FdoExtension).BusHackFlags =
                                    *(&raw mut (*ValueInfo).Data as *mut ULONG) as LONG;
                            }
                        }
                        PciGetHotPlugParameters(FdoExtension);
                        (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;
                        // return STATUS_SUCCESS
                        success = true;
                        break;
                    }
                }
            }
        }
        break; // break if any failure inside the inner block
    }

    // Failure path
    if !success {
        if NT_SUCCESS(Status) {
            RtlAssert(
                b"!NT_SUCCESS(Status)\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/bus/pcix/fdo.c\0".as_ptr()
                    as *const i8 as *mut core::ffi::c_void,
                658 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
        if !FdoExtension.is_null() {
            if DbgPrint(
                (b"(%s:%d) Should destroy secondaries\n\0".as_ptr() as PCSTR).cast(),
                b"drivers/bus/pcix/fdo.c\0".as_ptr()
                    as *const i8,
                661i32,
            ) != 0
            {
                DbgPrint(
                    (b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR).cast(),
                    b"drivers/bus/pcix/fdo.c\0".as_ptr()
                        as *const i8,
                    661i32,
                );
            }
        }
        if !AttachedTo.is_null() {
            IoDetachDevice(AttachedTo);
        }
        if !DeviceObject.is_null() {
            IoDeleteDevice(DeviceObject);
        }
        return Status;
    }

    // Success return
    STATUS_SUCCESS
}
