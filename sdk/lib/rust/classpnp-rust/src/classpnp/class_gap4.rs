//! ORPHAN-BY-DESIGN: a rustconv file-split gap artifact; its one export,
//! ClassShutdownFlush, is defined live in class_part_d.rs. See class_gap3.rs.
//!
use ntdll_sys::nt::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn ClassShutdownFlush(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    let isRemoved: ULONG;

    isRemoved = ClassAcquireRemoveLock(DeviceObject, Irp as PVOID);
    
    if isRemoved != 0 {
        ClassReleaseRemoveLock(DeviceObject, Irp as PVOID);

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;

        ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT as CCHAR);

        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    if !(*(*commonExtension).DevInfo).ClassShutdownFlush.is_null() {
        return (*(*commonExtension).DevInfo).ClassShutdownFlush.unwrap()(DeviceObject, Irp);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;

    ClassReleaseRemoveLock(DeviceObject, Irp as PVOID);
    ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT as CCHAR);

    return STATUS_INVALID_DEVICE_REQUEST;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "classp.h"
// #include "debug.h"
//
// NTSTATUS
// NTAPI 
// ClassShutdownFlush(
//       PDEVICE_OBJECT DeviceObject,
//       PIRP Irp
//     )
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//
//     ULONG isRemoved;
//
//     isRemoved = ClassAcquireRemoveLock(DeviceObject, Irp);
//     _Analysis_assume_(isRemoved);
//     if(isRemoved) {
//
//         ClassReleaseRemoveLock(DeviceObject, Irp);
//
//         Irp->IoStatus.Status = STATUS_DEVICE_DOES_NOT_EXIST;
//
//         ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
//
//         return STATUS_DEVICE_DOES_NOT_EXIST;
//     }
//
//     if (commonExtension->DevInfo->ClassShutdownFlush) {
//
//         //
//         // Call the device-specific driver's routine.
//         //
//
//         return commonExtension->DevInfo->ClassShutdownFlush(DeviceObject, Irp);
//     }
//
//     //
//     // Device-specific driver doesn't support this.
//     //
//
//     Irp->IoStatus.Status = STATUS_INVALID_DEVICE_REQUEST;
//
//     ClassReleaseRemoveLock(DeviceObject, Irp);
//     ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
//
//     return STATUS_INVALID_DEVICE_REQUEST;
// } // end ClassShutdownFlush()
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
