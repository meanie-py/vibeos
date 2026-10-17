use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePool, ExFreePool, _KWAIT_REASON_Executive, IoAcquireCancelSpinLock, IoReleaseCancelSpinLock, KeWaitForSingleObject, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, _KWAIT_REASON_UserRequest};
use core::ffi::c_void;

unsafe extern "system" fn ReadIrpCancel(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    let mut IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut AdapterContext: PNDISUIO_ADAPTER_CONTEXT =
        (*(*IrpSp).FileObject).FsContext as PNDISUIO_ADAPTER_CONTEXT;
    let mut PacketEntry: PNDISUIO_PACKET_ENTRY = core::ptr::null_mut();

    /* Release the cancel spin lock */
    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    /* Indicate a 0-byte packet on the queue to cancel the read */
    PacketEntry = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<NDISUIO_PACKET_ENTRY>() as SIZE_T,
    ) as PNDISUIO_PACKET_ENTRY;
    if !PacketEntry.is_null() {
        (*PacketEntry).PacketLength = 0 as ULONG;

        ExInterlockedInsertHeadList(
            core::ptr::addr_of_mut!((*AdapterContext).PacketList),
            core::ptr::addr_of_mut!((*PacketEntry).ListEntry),
            core::ptr::addr_of_mut!((*AdapterContext).Spinlock),
        );

        KeSetEvent(
            core::ptr::addr_of_mut!((*AdapterContext).PacketReadEvent),
            IO_NO_INCREMENT as KPRIORITY,
            FALSE as BOOLEAN,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn NduDispatchRead(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let AdapterContext: PNDISUIO_ADAPTER_CONTEXT = (*(*IrpSp).FileObject).FsContext as PNDISUIO_ADAPTER_CONTEXT;
    let OpenEntry: PNDISUIO_OPEN_ENTRY = (*(*IrpSp).FileObject).FsContext2 as PNDISUIO_OPEN_ENTRY;
    let mut OldIrql: KIRQL = 0;
    let mut OldCancelIrql: KIRQL = 0;
    let mut Status: NTSTATUS = 0;
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut PacketEntry: PNDISUIO_PACKET_ENTRY = core::ptr::null_mut();
    let mut BytesCopied: ULONG = 0;

    ASSERT!(DeviceObject == GlobalDeviceObject);

    if (*OpenEntry).WriteOnly != 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

        return STATUS_INVALID_PARAMETER;
    }

    /* Make the read cancellable */
    IoAcquireCancelSpinLock(&mut OldCancelIrql);
    IoSetCancelRoutine(Irp, Some(ReadIrpCancel));
    if (*Irp).Cancel != 0 {
        IoReleaseCancelSpinLock(OldCancelIrql);

        /* Indicate a 0 byte read */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

        return STATUS_SUCCESS;
    }
    IoReleaseCancelSpinLock(OldCancelIrql);

    loop {
        KeAcquireSpinLock(core::ptr::addr_of_mut!((*AdapterContext).Spinlock), &mut OldIrql);

        /* Check if we have a packet */
        if IsListEmpty(core::ptr::addr_of_mut!((*AdapterContext).PacketList)) != 0 {
            KeReleaseSpinLock(core::ptr::addr_of_mut!((*AdapterContext).Spinlock), OldIrql);

            /* Wait for a packet (in the context of the calling user thread) */
            Status = KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*AdapterContext).PacketReadEvent) as PVOID,
                _KWAIT_REASON_UserRequest,
                (1i32) as KPROCESSOR_MODE,
                TRUE,
                core::ptr::null_mut(),
            );
            if Status != STATUS_SUCCESS {
                /* Remove the cancel routine */
                IoAcquireCancelSpinLock(&mut OldCancelIrql);
                IoSetCancelRoutine(Irp, None);
                IoReleaseCancelSpinLock(OldCancelIrql);

                break;
            }
        } else {
            /* Remove the cancel routine */
            IoAcquireCancelSpinLock(&mut OldCancelIrql);
            IoSetCancelRoutine(Irp, None);
            IoReleaseCancelSpinLock(OldCancelIrql);

            /* Remove the first packet in the list */
            ListEntry = RemoveHeadList(core::ptr::addr_of_mut!((*AdapterContext).PacketList));
            PacketEntry = CONTAINING_RECORD!(ListEntry, NDISUIO_PACKET_ENTRY, ListEntry);

            /* Release the adapter lock */
            KeReleaseSpinLock(core::ptr::addr_of_mut!((*AdapterContext).Spinlock), OldIrql);

            /* And we're done with this loop */
            Status = STATUS_SUCCESS;
            break;
        }
    }

    /* Check if we got a packet */
    if !PacketEntry.is_null() {
        /* Find the right amount of bytes to copy */
        BytesCopied = (*PacketEntry).PacketLength;
        if BytesCopied > (*IrpSp).Parameters.Read.Length {
            BytesCopied = (*IrpSp).Parameters.Read.Length;
        }

        /* Copy the packet */
        RtlCopyMemory!(
            (*Irp).AssociatedIrp.SystemBuffer,
            core::ptr::addr_of!((*(*PacketEntry).PacketData.as_ptr().add(0 as usize))) as *const core::ffi::c_void,
            BytesCopied,
        );

        /* Free the packet entry */
        ExFreePool((PacketEntry).cast());
    } else {
        /* Something failed */
        BytesCopied = 0;
    }

    /* Complete the IRP */
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = BytesCopied as ULONG_PTR;
    IoCompleteRequest(Irp, (IO_NETWORK_INCREMENT as i8));

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn NduDispatchWrite(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let AdapterContext: PNDISUIO_ADAPTER_CONTEXT =
        (*(*IrpSp).FileObject).FsContext as PNDISUIO_ADAPTER_CONTEXT;
    let mut Packet: PNDIS_PACKET = core::ptr::null_mut();
    let mut Status: NDIS_STATUS = 0;
    let mut BytesCopied: ULONG = 0;

    ASSERT!(DeviceObject == GlobalDeviceObject);

    /* Create a packet and buffer descriptor for this user buffer */
    Packet = CreatePacketFromPoolBuffer(
        AdapterContext,
        (*Irp).AssociatedIrp.SystemBuffer,
        (*IrpSp).Parameters.Write.Length,
    );
    if !Packet.is_null() {
        /* Send it via NDIS */
        NdisSend(
            core::ptr::addr_of_mut!(Status),
            (*AdapterContext).BindingHandle,
            Packet,
        );

        /* Wait for the send */
        if Status == NDIS_STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent) as PVOID,
                _KWAIT_REASON_Executive,
                0i32 as KPROCESSOR_MODE,
                FALSE as BOOLEAN,
                core::ptr::null_mut(),
            );
            Status = (*AdapterContext).AsyncStatus;
        }

        /* Check if it succeeded */
        if Status == NDIS_STATUS_SUCCESS {
            BytesCopied = (*IrpSp).Parameters.Write.Length;
        }

        CleanupAndFreePacket(Packet, FALSE as BOOLEAN);
    } else {
        /* No memory */
        Status = STATUS_NO_MEMORY as NDIS_STATUS;
    }

    /* Complete the IRP */
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status as NTSTATUS;
    (*Irp).IoStatus.Information = BytesCopied as ULONG_PTR;
    IoCompleteRequest(Irp, IO_NETWORK_INCREMENT as CCHAR);

    return Status as NTSTATUS;
}
