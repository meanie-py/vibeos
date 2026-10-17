use crate::prelude::*;
use ntkrnl_sys::ddk::{ExFreePool, RtlEqualUnicodeString};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn AllocateAndChainBuffer(
    AdapterContext: PNDISUIO_ADAPTER_CONTEXT,
    Packet: PNDIS_PACKET,
    Buffer: PVOID,
    BufferSize: ULONG,
    Front: BOOLEAN,
) -> NDIS_STATUS {
    let mut Status: NDIS_STATUS = 0;
    let mut NdisBuffer: PNDIS_BUFFER = core::ptr::null_mut();

    /* Allocate the NDIS buffer mapping the pool */
    NdisAllocateBuffer(
        core::ptr::addr_of_mut!(Status),
        core::ptr::addr_of_mut!(NdisBuffer),
        (*AdapterContext).BufferPoolHandle,
        Buffer,
        BufferSize as UINT,
    );
    if Status != NDIS_STATUS_SUCCESS {
        DPRINT1!("No free buffer descriptors\n");
        return Status;
    }

    if Front != 0 {
        /* Chain the buffer to front */
        NdisChainBufferAtFront!(Packet, NdisBuffer);
    } else {
        /* Chain the buffer to back */
        NdisChainBufferAtBack!(Packet, NdisBuffer);
    }

    /* Return success */
    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn CreatePacketFromPoolBuffer(
    AdapterContext: PNDISUIO_ADAPTER_CONTEXT,
    Buffer: PVOID,
    BufferSize: ULONG,
) -> PNDIS_PACKET {
    let mut Packet: PNDIS_PACKET = core::ptr::null_mut();
    let mut Status: NDIS_STATUS = 0;

    /* Allocate a packet descriptor */
    NdisAllocatePacket(
        core::ptr::addr_of_mut!(Status),
        core::ptr::addr_of_mut!(Packet),
        (*AdapterContext).PacketPoolHandle,
    );
    if Status != NDIS_STATUS_SUCCESS {
        DPRINT1!("No free packet descriptors\n");
        return core::ptr::null_mut();
    }

    /* Use the helper to chain the buffer */
    Status = AllocateAndChainBuffer(
        AdapterContext,
        Packet,
        Buffer,
        BufferSize,
        TRUE as BOOLEAN,
    );
    if Status != NDIS_STATUS_SUCCESS {
        NdisFreePacket(Packet);
        return core::ptr::null_mut();
    }

    /* Return the packet */
    return Packet;
}
#[no_mangle]
pub unsafe extern "C" fn CleanupAndFreePacket(Packet: PNDIS_PACKET, FreePool: BOOLEAN) {
    let mut Buffer: PNDIS_BUFFER = core::ptr::null_mut();
    let mut Data: PVOID = core::ptr::null_mut();
    let mut Length: ULONG = 0;

    /* Free each buffer and its backing pool memory */
    while (1 != 0) {
        /* Unchain each buffer */
        NdisUnchainBufferAtFront(Packet, core::ptr::addr_of_mut!(Buffer));
        if Buffer.is_null() {
            break;
        }

        /* Get the backing memory */
        NdisQueryBuffer!(Buffer, core::ptr::addr_of_mut!(Data), core::ptr::addr_of_mut!(Length));

        /* Free the buffer */
        NdisFreeBuffer(Buffer);

        if FreePool != 0 {
            /* Free the backing memory */
            ExFreePool(Data);
        }
    }

    /* Free the packet descriptor */
    NdisFreePacket(Packet);
}
#[no_mangle]
pub unsafe extern "C" fn FindAdapterContextByName(DeviceName: PNDIS_STRING) -> PNDISUIO_ADAPTER_CONTEXT {
    let mut OldIrql: KIRQL = 0;
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut AdapterContext: PNDISUIO_ADAPTER_CONTEXT = core::ptr::null_mut();

    KeAcquireSpinLock(core::ptr::addr_of_mut!(GlobalAdapterListLock), core::ptr::addr_of_mut!(OldIrql));
    CurrentEntry = (*core::ptr::addr_of_mut!(GlobalAdapterList)).Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(GlobalAdapterList) {
        AdapterContext = ((CurrentEntry as usize).wrapping_sub(
            core::mem::offset_of!(NDISUIO_ADAPTER_CONTEXT, ListEntry),
        )) as PNDISUIO_ADAPTER_CONTEXT;

        /* Check if the device name matches */
        if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!((*AdapterContext).DeviceName),
            DeviceName,
            TRUE,
        ) != 0
        {
            KeReleaseSpinLock(core::ptr::addr_of_mut!(GlobalAdapterListLock), OldIrql);
            return AdapterContext;
        }

        CurrentEntry = (*CurrentEntry).Flink;
    }
    KeReleaseSpinLock(core::ptr::addr_of_mut!(GlobalAdapterListLock), OldIrql);

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn ReferenceAdapterContext(AdapterContext: PNDISUIO_ADAPTER_CONTEXT) {
    (*AdapterContext).OpenCount = (*AdapterContext).OpenCount.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn DereferenceAdapterContextWithOpenEntry(
    mut AdapterContext: PNDISUIO_ADAPTER_CONTEXT,
    mut OpenEntry: PNDISUIO_OPEN_ENTRY,
) {
    let mut OldIrql: KIRQL = 0;

    /* Lock the adapter context */
    KeAcquireSpinLock(core::ptr::addr_of_mut!((*AdapterContext).Spinlock), core::ptr::addr_of_mut!(OldIrql));

    /* Decrement the open count */
    (*AdapterContext).OpenCount = (*AdapterContext).OpenCount.wrapping_sub(1);

    /* Cleanup the open entry if we were given one */
    if !OpenEntry.is_null() {
        /* Remove the open entry */
        RemoveEntryList(core::ptr::addr_of_mut!((*OpenEntry).ListEntry));

        /* Invalidate the FO */
        (*(*OpenEntry).FileObject).FsContext = core::ptr::null_mut() as PVOID;
        (*(*OpenEntry).FileObject).FsContext2 = core::ptr::null_mut() as PVOID;

        /* Free the open entry */
        ExFreePool(OpenEntry as PVOID);
    }

    /* Release the adapter context lock */
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*AdapterContext).Spinlock), OldIrql);
}
