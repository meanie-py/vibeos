use crate::prelude::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;

unsafe extern "C" fn PciIdeXPdoDispatchPower(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let IoStack: PIO_STACK_LOCATION;

    UNREFERENCED_PARAMETER!(PdoExtension);

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStack).MinorFunction {
        IRP_MN_SET_POWER | IRP_MN_QUERY_POWER => {
            Status = STATUS_SUCCESS;
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        }
        _ => {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    PoStartNextPowerIrp(Irp);
    IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);
    return Status;
}

unsafe extern "C" fn PciIdeXFdoDispatchPower(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    PoStartNextPowerIrp(Irp);
    IoSkipCurrentIrpStackLocation(Irp);
    return PoCallDriver((*FdoExtension).Common.__bindgen_anon_1.LowerDeviceObject, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXDispatchPower(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let CommonExt = (*DeviceObject).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;

    Status = IoAcquireRemoveLock(&mut (*CommonExt).RemoveLock, (Irp).cast());
    if !NT_SUCCESS!(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

        return Status;
    }

    if IS_FDO!(CommonExt) {
        Status = PciIdeXFdoDispatchPower((CommonExt as *mut core::ffi::c_void).cast(), Irp);
    } else {
        Status = PciIdeXPdoDispatchPower((CommonExt as *mut core::ffi::c_void).cast(), Irp);
    }

    IoReleaseRemoveLock(&mut (*CommonExt).RemoveLock, (Irp).cast());

    return Status;
}
