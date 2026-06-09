use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{IRP_MJ_SCSI};
use crate::types::{ClassCreateClose, ClassDeviceControlDispatch, ClassDispatchPnp, ClassDispatchPower, ClassInternalIoControl, ClassReadWrite, ClassShutdownFlush, ClassSystemControl, PCOMMON_DEVICE_EXTENSION, _CLASS_DRIVER_EXTENSION};
use ntkrnl_sys::ddk::{PCSTR, _DEVICE_OBJECT, _IRP};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{KeGetCurrentIrql};

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "stdcall" fn ClassInitializeDispatchTables(DriverExtension: *mut _CLASS_DRIVER_EXTENSION) {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"/home/curi/Projects/os-playground/vibeos/drivers/storage/class/classpnp/dispatch.c\0"
                    .as_ptr() as *const i8,
                44i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    let table = &mut (*DriverExtension).DeviceMajorFunctionTable;

    let mut idx: u32 = 0;
    while idx <= IRP_MJ_MAXIMUM_FUNCTION as u32 {
        table[idx as usize] =
            Some(ClassDispatchUnimplemented as unsafe extern "stdcall" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS);
        idx = idx.wrapping_add(1);
    }

    table[IRP_MJ_CREATE as usize] =
        Some(ClassCreateClose as unsafe extern "stdcall" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS);
    table[IRP_MJ_CLOSE as usize] =
        Some(ClassCreateClose as unsafe extern "stdcall" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS);
    table[IRP_MJ_READ as usize] =
        Some(ClassReadWrite as unsafe extern "stdcall" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS);
    table[IRP_MJ_WRITE as usize] =
        Some(ClassReadWrite as unsafe extern "stdcall" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS);
    table[IRP_MJ_DEVICE_CONTROL as usize] =
        Some(ClassDeviceControlDispatch as unsafe extern "stdcall" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS);
    table[IRP_MJ_SCSI as usize] =
        Some(ClassInternalIoControl as unsafe extern "stdcall" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS);
    table[IRP_MJ_SHUTDOWN as usize] =
        Some(ClassShutdownFlush as unsafe extern "stdcall" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS);
    table[IRP_MJ_FLUSH_BUFFERS as usize] =
        Some(ClassShutdownFlush as unsafe extern "stdcall" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS);
    table[IRP_MJ_PNP as usize] =
        Some(ClassDispatchPnp as unsafe extern "stdcall" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS);
    table[IRP_MJ_POWER as usize] =
        Some(ClassDispatchPower as unsafe extern "stdcall" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS);
    table[IRP_MJ_SYSTEM_CONTROL as usize] =
        Some(ClassSystemControl as unsafe extern "stdcall" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS);
}
#[no_mangle]
pub unsafe extern "system" fn ClassGlobalDispatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let common_extension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let irp_stack = IoGetCurrentIrpStackLocation(Irp);
    let dispatch_fn = *(*common_extension).DispatchTable.offset((*irp_stack).MajorFunction as isize);
    dispatch_fn.expect("non-null function pointer")(DeviceObject, Irp)
}
#[no_mangle]
pub unsafe extern "stdcall" fn ClassDispatchUnimplemented(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let _ = DeviceObject;
    if (*IoGetCurrentIrpStackLocation(Irp)).MajorFunction as i32 == IRP_MJ_POWER as i32 {
        PoStartNextPowerIrp(Irp);
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_INVALID_DEVICE_REQUEST
}
