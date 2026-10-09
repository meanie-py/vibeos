#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, KEVENT, KeInitializeEvent, KeWaitForSingleObject, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PKEVENT, _EVENT_TYPE_SynchronizationEvent};
use ntkrnl_sys::inline::tier1::{IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoSetCompletionRoutine, IoSkipCurrentIrpStackLocation};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{PCSTR, RtlAssert, _IO_STACK_LOCATION, _IRP, _DEVICE_OBJECT, KeSetEvent};

// NOTE: the c2rust preamble extraction swallowed this fn (header + body);
// reconstructed faithfully from drivers/bus/pcix/dispatch.c PciSetEventCompletion.
#[no_mangle]
pub unsafe extern "system" fn PciSetEventCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let Event: PKEVENT = Context as PKEVENT;
    // ASSERT(Event)
    if Event.is_null() {
        RtlAssert(
            b"Event\0".as_ptr() as *const i8 as PVOID,
            b"dispatch.c\0".as_ptr() as *const i8 as PVOID,
            27 as ULONG,
            (core::ptr::null_mut::<u8>()).cast(),
        );
    }
    let _ = DeviceObject;
    let _ = Irp;
    // Set the event and return the appropriate status code
    KeSetEvent(Event, 0i32, 0u8);
    const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;
    STATUS_MORE_PROCESSING_REQUIRED
}

// NOTE: the c2rust preamble extraction swallowed this fn's header + the DPRINT1
// opening; reconstructed from drivers/bus/pcix/dispatch.c PciCallDownIrpStack.
#[no_mangle]
pub unsafe extern "system" fn PciCallDownIrpStack(
    DeviceExtension: *mut _PCI_FDO_EXTENSION,
    Irp: *mut _IRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Event: KEVENT = core::mem::zeroed();
    DbgPrint(b"PciCallDownIrpStack ...\n\0".as_ptr() as PCSTR);

    // ASSERT_FDO(DeviceExtension)
    if !((*DeviceExtension).ExtensionType == PciFdoExtensionType as u32) {
        RtlAssert(
            b"(DeviceExtension)->ExtensionType == PciFdoExtensionType\0".as_ptr() as *const i8
                as PVOID,
            b"dispatch.c\0".as_ptr() as *const i8 as PVOID,
            46 as ULONG,
            (core::ptr::null_mut::<u8>()).cast(),
        );
    }

    // Initialize the wait event
    KeInitializeEvent(&mut Event as PKEVENT, _EVENT_TYPE_SynchronizationEvent, 0u8);

    // Setup a completion routine
    IoCopyCurrentIrpStackLocationToNext(Irp);
    IoSetCompletionRoutine(
        Irp,
        Some(core::mem::transmute::<*const (), _>(PciSetEventCompletion as *const ())),
        &mut Event as *mut _ as PVOID,
        1u8,
        1u8,
        1u8,
    );

    // Call the attached device
    Status = IoCallDriver((*DeviceExtension).AttachedDeviceObject, Irp);
    if Status == STATUS_PENDING {
        // Wait for it to complete the request, and get its status
        KeWaitForSingleObject(
            &mut Event as *mut _ as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PciPassIrpFromFdoToPdo(
    DeviceExtension: *mut _PCI_FDO_EXTENSION,
    Irp: *mut _IRP,
) -> NTSTATUS {
    let mut IoStackLocation: *mut _IO_STACK_LOCATION;
    let mut Status: NTSTATUS;

    if DbgPrint(
        b"(%s:%d) Pci PassIrp ...\n\0".as_ptr() as PCHAR,
        b"drivers/bus/pcix/dispatch.c\0".as_ptr() as PCHAR,
        75i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCHAR,
            b"drivers/bus/pcix/dispatch.c\0".as_ptr() as PCHAR,
            75i32,
        );
    }

    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
    if (*IoStackLocation).MajorFunction as i32 == IRP_MJ_POWER as i32 {
        IoCopyCurrentIrpStackLocationToNext(Irp);
        PoStartNextPowerIrp(Irp);
        Status = PoCallDriver((*DeviceExtension).AttachedDeviceObject, Irp);
    } else {
        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*DeviceExtension).AttachedDeviceObject, Irp);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PciDispatchIrp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceExtension: PPCI_FDO_EXTENSION;
    let mut IoStackLocation: PIO_STACK_LOCATION;
    let mut IrpDispatchTable: PPCI_MJ_DISPATCH_TABLE;
    let mut PassToPdo: BOOLEAN;
    let mut Status: NTSTATUS;
    let mut TableArray: *mut _PCI_MN_DISPATCH_TABLE = core::ptr::null_mut::<_PCI_MN_DISPATCH_TABLE>();
    let mut Table: *mut _PCI_MN_DISPATCH_TABLE;
    let mut MaxMinor: USHORT;
    let mut DispatchStyle: PCI_DISPATCH_STYLE = 0; // 0 = IRP_COMPLETE
    let mut DispatchFunction: PCI_DISPATCH_FUNCTION = None;

    if DbgPrint(
        b"(%s:%d) PCI: Dispatch IRP\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/dispatch.c\0".as_ptr()
            as *const ::core::ffi::c_char,
        111 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/dispatch.c\0".as_ptr()
                as *const ::core::ffi::c_char,
            111 as ::core::ffi::c_int,
        );
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PPCI_FDO_EXTENSION;
    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
    if !((*DeviceExtension).ExtensionType
        == PciPdoExtensionType as ::core::ffi::c_ulong
        || (*DeviceExtension).ExtensionType == PciFdoExtensionType as ::core::ffi::c_ulong)
    {
        RtlAssert(
            b"(DeviceExtension->ExtensionType == PciPdoExtensionType) || (DeviceExtension->ExtensionType == PciFdoExtensionType)\0"
                .as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/dispatch.c\0"
                .as_ptr() as *const ::core::ffi::c_char as PVOID,
            117 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    if (*DeviceExtension).DeviceState as ::core::ffi::c_int == PciDeleted as ::core::ffi::c_int {
        Status = STATUS_NO_SUCH_DEVICE;
        PassToPdo = 0u8;
    } else {
        IrpDispatchTable = (*DeviceExtension).IrpDispatchTable as PPCI_MJ_DISPATCH_TABLE;
        match (*IoStackLocation).MajorFunction as u8 {
            IRP_MJ_POWER => {
                TableArray = (*IrpDispatchTable).PowerIrpDispatchTable;
                MaxMinor = (*IrpDispatchTable).PowerIrpMaximumMinorFunction as USHORT;
            }
            IRP_MJ_PNP => {
                TableArray = (*IrpDispatchTable).PnpIrpDispatchTable;
                MaxMinor = (*IrpDispatchTable).PnpIrpMaximumMinorFunction as USHORT;
            }
            IRP_MJ_SYSTEM_CONTROL => {
                DispatchFunction = (*IrpDispatchTable).SystemControlIrpDispatchFunction;
                DispatchStyle = (*IrpDispatchTable).SystemControlIrpDispatchStyle;
                MaxMinor = 0xffff as USHORT;
            }
            _ => {
                DispatchFunction = (*IrpDispatchTable).OtherIrpDispatchFunction;
                DispatchStyle = (*IrpDispatchTable).OtherIrpDispatchStyle;
                MaxMinor = 0xffff as USHORT;
            }
        }

        if MaxMinor as ::core::ffi::c_int != 0xffff as ::core::ffi::c_int {
            if (*IoStackLocation).MinorFunction as ::core::ffi::c_int
                > MaxMinor as ::core::ffi::c_int
            {
                Table = TableArray.offset(
                    (MaxMinor as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
                ) as PPCI_MN_DISPATCH_TABLE;
            } else {
                Table = TableArray.offset((*IoStackLocation).MinorFunction as isize)
                    as PPCI_MN_DISPATCH_TABLE;
            }
            DispatchStyle = (*Table).DispatchStyle;
            DispatchFunction = (*Table).DispatchFunction;
        }

        if PciDebugIrpDispatchDisplay(IoStackLocation, DeviceExtension, MaxMinor) != 0 {
            DbgBreakPoint();
        }

        if DispatchStyle as ::core::ffi::c_uint == 2u32 {
            // IRP_UPWARD = 2
            PciCallDownIrpStack(DeviceExtension, Irp);
        }

        Status = DispatchFunction.expect("non-null function pointer")(
            Irp,
            IoStackLocation,
            DeviceExtension as PVOID,
        );

        match DispatchStyle as ::core::ffi::c_uint {
            0 => {
                // IRP_COMPLETE
                PassToPdo = 0u8;
            }
            1 => {
                // IRP_DOWNWARD
                PassToPdo = 1u8;
            }
            2 => {
                // IRP_UPWARD
                PassToPdo = 0u8;
            }
            3 => {
                // IRP_DISPATCH
                return Status;
            }
            _ => {
                if 0 as ::core::ffi::c_int == 0 {
                    RtlAssert(
                        b"FALSE\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
                        b"drivers/bus/pcix/dispatch.c\0"
                            .as_ptr() as *const ::core::ffi::c_char as PVOID,
                        226 as ULONG,
                        core::ptr::null_mut::<CHAR>(),
                    );
                }
                return Status;
            }
        }
    }

    if Status == STATUS_PENDING {
        return Status;
    }

    if Status != STATUS_NOT_SUPPORTED {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    }

    if PassToPdo != 0 && (Status >= 0 || Status == STATUS_NOT_SUPPORTED) {
        Status = PciPassIrpFromFdoToPdo(DeviceExtension, Irp);
    } else {
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        if (*IoStackLocation).MajorFunction as ::core::ffi::c_int == IRP_MJ_POWER as ::core::ffi::c_int {
            PoStartNextPowerIrp(Irp);
        }
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn PciIrpNotSupported(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    let _ = (Irp, IoStackLocation, DeviceExtension);
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciIrpInvalidDeviceRequest(
    _Irp: PIRP,
    _IoStackLocation: PIO_STACK_LOCATION,
    _DeviceExtension: *mut core::ffi::c_void,
) -> NTSTATUS {
    STATUS_INVALID_DEVICE_REQUEST
}
