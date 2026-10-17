use crate::prelude::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn NduDispatchCreate(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    ASSERT!(DeviceObject == GlobalDeviceObject);

    DPRINT!("Created file object 0x%x\n", (*IrpSp).FileObject);

    /* This is associated with an adapter during IOCTL_NDISUIO_OPEN_(WRITE_)DEVICE */
    (*(*IrpSp).FileObject).FsContext = core::ptr::null_mut();
    (*(*IrpSp).FileObject).FsContext2 = core::ptr::null_mut();

    /* Completed successfully */
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = FILE_OPENED as ULONG_PTR;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    /* Return success */
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NduDispatchClose(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let AdapterContext: PNDISUIO_ADAPTER_CONTEXT =
        (*(*IrpSp).FileObject).FsContext as PNDISUIO_ADAPTER_CONTEXT;
    let OpenEntry: PNDISUIO_OPEN_ENTRY =
        (*(*IrpSp).FileObject).FsContext2 as PNDISUIO_OPEN_ENTRY;

    ASSERT!(DeviceObject == GlobalDeviceObject);

    DPRINT!("Closing file object 0x%x\n", (*IrpSp).FileObject);

    /* Check if this handle was ever associated with an adapter */
    if !AdapterContext.is_null() {
        ASSERT!(!OpenEntry.is_null());

        DPRINT!(
            "Removing binding to adapter %wZ\n",
            core::ptr::addr_of_mut!((*AdapterContext).DeviceName)
        );

        /* Call the our helper */
        DereferenceAdapterContextWithOpenEntry(AdapterContext, OpenEntry);
    }

    /* Completed */
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    /* Return success */
    return STATUS_SUCCESS;
}
