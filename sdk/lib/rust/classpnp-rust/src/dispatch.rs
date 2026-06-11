use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PDRIVER_DISPATCH};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{IRP_MJ_SCSI};
use crate::types::{ClassCreateClose, ClassDeviceControlDispatch, ClassDispatchPnp, ClassDispatchPower, ClassInternalIoControl, ClassReadWrite, ClassShutdownFlush, ClassSystemControl, PCOMMON_DEVICE_EXTENSION, _CLASS_DRIVER_EXTENSION};
use ntkrnl_sys::ddk::{PCSTR, _DEVICE_OBJECT, _IRP};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{KeGetCurrentIrql};

// ---- file-scope preamble (translated) ----
// #[inline(never)]: standalone this body is fold-free, but inlining it into ClassInitialize lets
// LLVM combine the driverExtension dispatch-table writes with the caller's memcpy/dereferenceable
// facts into a captures(none) UB fold (=> ud2 after RtlCopyUnicodeString). Keep it out-of-line.
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn ClassInitializeDispatchTables(DriverExtension: *mut _CLASS_DRIVER_EXTENSION) {
    // PAGED_CODE() IRQL assertion omitted (debug-only).
    // Raw-pointer writes (NOT `&mut (*DriverExtension).DeviceMajorFunctionTable`): taking a
    // reference to the field inserts a `panic_misaligned_pointer_dereference` check and rustc then
    // wrongly infers `readnone captures(none)` on DriverExtension — release-opt folds the whole body
    // to `unreachable` (→ ud2, BUGCHECK 0x1E at boot). write_volatile keeps the stores observable.
    let table = core::ptr::addr_of_mut!((*DriverExtension).DeviceMajorFunctionTable) as *mut PDRIVER_DISPATCH;

    let mut idx: usize = 0;
    while idx <= IRP_MJ_MAXIMUM_FUNCTION as usize {
        core::ptr::write_volatile(table.add(idx), Some(abi_cast!(ClassDispatchUnimplemented)));
        idx = idx.wrapping_add(1);
    }

    core::ptr::write_volatile(table.add(IRP_MJ_CREATE as usize), Some(ClassCreateClose));
    core::ptr::write_volatile(table.add(IRP_MJ_CLOSE as usize), Some(ClassCreateClose));
    core::ptr::write_volatile(table.add(IRP_MJ_READ as usize), Some(ClassReadWrite));
    core::ptr::write_volatile(table.add(IRP_MJ_WRITE as usize), Some(ClassReadWrite));
    core::ptr::write_volatile(table.add(IRP_MJ_DEVICE_CONTROL as usize), Some(ClassDeviceControlDispatch));
    core::ptr::write_volatile(table.add(IRP_MJ_SCSI as usize), Some(ClassInternalIoControl));
    core::ptr::write_volatile(table.add(IRP_MJ_SHUTDOWN as usize), Some(ClassShutdownFlush));
    core::ptr::write_volatile(table.add(IRP_MJ_FLUSH_BUFFERS as usize), Some(ClassShutdownFlush));
    core::ptr::write_volatile(table.add(IRP_MJ_PNP as usize), Some(ClassDispatchPnp));
    core::ptr::write_volatile(table.add(IRP_MJ_POWER as usize), Some(ClassDispatchPower));
    core::ptr::write_volatile(table.add(IRP_MJ_SYSTEM_CONTROL as usize), Some(ClassSystemControl));
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
pub unsafe extern "system" fn ClassDispatchUnimplemented(
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
