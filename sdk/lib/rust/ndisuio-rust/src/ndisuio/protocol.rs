use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePool, ExFreePool, _KWAIT_REASON_Executive, KeInitializeEvent, KeWaitForSingleObject, RtlFreeUnicodeString, _EVENT_TYPE_SynchronizationEvent};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn NduOpenAdapterComplete(ProtocolBindingContext: NDIS_HANDLE, Status: NDIS_STATUS, OpenStatus: NDIS_STATUS) {
    let AdapterContext: PNDISUIO_ADAPTER_CONTEXT =
        ProtocolBindingContext as PNDISUIO_ADAPTER_CONTEXT;

    DPRINT!("Asynchronous adapter open completed\n");

    /* Store the final status and signal the event */
    (*AdapterContext).AsyncStatus = Status;
    KeSetEvent(
        core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent),
        (IO_NO_INCREMENT) as KPRIORITY,
        FALSE,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NduCloseAdapterComplete(ProtocolBindingContext: NDIS_HANDLE, Status: NDIS_STATUS) {
    let AdapterContext: PNDISUIO_ADAPTER_CONTEXT = ProtocolBindingContext as PNDISUIO_ADAPTER_CONTEXT;

    DPRINT!("Asynchronous adapter close completed\n");

    /* Store the final status and signal the event */
    (*AdapterContext).AsyncStatus = Status;
    KeSetEvent(
        &raw mut (*AdapterContext).AsyncEvent,
        IO_NO_INCREMENT as KPRIORITY,
        FALSE as BOOLEAN,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NduNetPnPEvent(ProtocolBindingContext: NDIS_HANDLE, NetPnPEvent: PNET_PNP_EVENT) -> NDIS_STATUS {
    let mut PowerState: PNDIS_DEVICE_POWER_STATE = core::ptr::null_mut();

    DPRINT!("NetPnPEvent\n");

    match (*NetPnPEvent).NetEvent {
        NetEventQueryRemoveDevice => {
            /* Nothing to do */
            DPRINT1!("NetPnPEvent: QueryRemoveDevice\n");
            return NDIS_STATUS_SUCCESS;
        }

        NetEventSetPower => {
            ASSERT!((*NetPnPEvent).BufferLength as usize >= core::mem::size_of::<NDIS_DEVICE_POWER_STATE>());

            PowerState = (*NetPnPEvent).Buffer as PNDIS_DEVICE_POWER_STATE;
            match *PowerState {
                NdisDeviceStateD0 => {
                    DPRINT1!("NetPnPEvent: SetPower D0\n");
                    return NDIS_STATUS_SUCCESS;
                }

                _ => {
                    DPRINT1!("NetPnPEvent: SetPower state %d not supported\n", *PowerState);
                    return NDIS_STATUS_FAILURE;
                }
            }
        }

        NetEventQueryPower => {
            DPRINT1!("NetPnPEvent: QueryPower\n");
            return NDIS_STATUS_SUCCESS;
        }

        NetEventCancelRemoveDevice => {
            DPRINT1!("NetPnPEvent: CancelRemoveDevice\n");
            return NDIS_STATUS_SUCCESS;
        }

        NetEventReconfigure => {
            DPRINT1!("NetPnPEvent: Reconfigure\n");
            return NDIS_STATUS_SUCCESS;
        }

        NetEventBindList => {
            DPRINT1!("NetPnPEvent: BindList\n");
            return NDIS_STATUS_SUCCESS;
        }

        NetEventBindsComplete => {
            DPRINT!("NetPnPEvent: BindsComplete\n");
            return NDIS_STATUS_SUCCESS;
        }

        NetEventPnPCapabilities => {
            DPRINT1!("NetPnPEvent: PnPCapabilities\n");
            return NDIS_STATUS_SUCCESS;
        }

        _ => {
            DPRINT1!("NetPnPEvent unimplemented for net event 0x%x\n", (*NetPnPEvent).NetEvent);
            return NDIS_STATUS_FAILURE;
        }
    };
}
#[no_mangle]
pub unsafe extern "system" fn NduSendComplete(ProtocolBindingContext: NDIS_HANDLE, Packet: PNDIS_PACKET, Status: NDIS_STATUS) {
    let AdapterContext: PNDISUIO_ADAPTER_CONTEXT =
        ProtocolBindingContext as PNDISUIO_ADAPTER_CONTEXT;

    DPRINT!("Asynchronous adapter send completed\n");

    /* Store the final status and signal the event */
    (*AdapterContext).AsyncStatus = Status;
    KeSetEvent(
        core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent),
        IO_NO_INCREMENT as KPRIORITY,
        FALSE as BOOLEAN,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NduTransferDataComplete(ProtocolBindingContext: NDIS_HANDLE, Packet: PNDIS_PACKET, Status: NDIS_STATUS, BytesTransferred: UINT) {
    let mut AdapterContext: PNDISUIO_ADAPTER_CONTEXT =
        ProtocolBindingContext as PNDISUIO_ADAPTER_CONTEXT;

    DPRINT!("Asynchronous adapter transfer completed\n");

    /* Store the final status and signal the event */
    (*AdapterContext).AsyncStatus = Status;
    KeSetEvent(
        core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent),
        IO_NO_INCREMENT as KPRIORITY,
        FALSE as BOOLEAN,
    );
}

#[no_mangle]
pub unsafe extern "system" fn NduResetComplete(ProtocolBindingContext: NDIS_HANDLE, Status: NDIS_STATUS) {
    let mut AdapterContext: PNDISUIO_ADAPTER_CONTEXT = ProtocolBindingContext as PNDISUIO_ADAPTER_CONTEXT;

    DPRINT!("Asynchronous adapter reset completed\n");

    /* Store the final status and signal the event */
    (*AdapterContext).AsyncStatus = Status;
    KeSetEvent(
        core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent),
        IO_NO_INCREMENT as KPRIORITY,
        FALSE as BOOLEAN,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NduRequestComplete(ProtocolBindingContext: NDIS_HANDLE, NdisRequest: PNDIS_REQUEST, Status: NDIS_STATUS) {
    let AdapterContext: PNDISUIO_ADAPTER_CONTEXT = ProtocolBindingContext as PNDISUIO_ADAPTER_CONTEXT;

    DPRINT!("Asynchronous adapter request completed\n");

    /* Store the final status and signal the event */
    (*AdapterContext).AsyncStatus = Status;
    KeSetEvent(
        core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent),
        IO_NO_INCREMENT as KPRIORITY,
        FALSE as BOOLEAN,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NduReceive(ProtocolBindingContext: NDIS_HANDLE, MacReceiveContext: NDIS_HANDLE, HeaderBuffer: PVOID, HeaderBufferSize: UINT, LookAheadBuffer: PVOID, LookaheadBufferSize: UINT, PacketSize: UINT) -> NDIS_STATUS {
    let AdapterContext: PNDISUIO_ADAPTER_CONTEXT = ProtocolBindingContext as PNDISUIO_ADAPTER_CONTEXT;
    let mut PacketEntry: PNDISUIO_PACKET_ENTRY;
    let mut PacketBuffer: PVOID;
    let mut Packet: PNDIS_PACKET;
    let mut Status: NDIS_STATUS = 0;
    let mut BytesTransferred: UINT = 0;

    DPRINT!("Received a %d byte packet\n", PacketSize);

    /* Discard if nobody is waiting for it */
    if (*AdapterContext).OpenCount == 0 {
        return NDIS_STATUS_NOT_ACCEPTED;
    }

    /* Allocate a buffer to hold the packet data and header */
    PacketBuffer = ExAllocatePool(NonPagedPool, PacketSize.wrapping_add(HeaderBufferSize) as SIZE_T);
    if PacketBuffer.is_null() {
        return NDIS_STATUS_NOT_ACCEPTED;
    }

    /* Allocate the packet descriptor and buffer */
    Packet = CreatePacketFromPoolBuffer(
        AdapterContext,
        (PacketBuffer as PUCHAR).offset(HeaderBufferSize as isize) as PVOID,
        PacketSize,
    );
    if Packet.is_null() {
        ExFreePool(PacketBuffer);
        return NDIS_STATUS_NOT_ACCEPTED;
    }

    /* Transfer the packet data into our data buffer */
    if LookaheadBufferSize == PacketSize {
        NdisCopyLookaheadData!(
            (PacketBuffer as PUCHAR).offset(HeaderBufferSize as isize) as PVOID,
            LookAheadBuffer,
            PacketSize,
            (*AdapterContext).MacOptions,
        );
        BytesTransferred = PacketSize;
    } else {
        NdisTransferData(
            core::ptr::addr_of_mut!(Status),
            (*AdapterContext).BindingHandle,
            MacReceiveContext,
            0,
            PacketSize,
            Packet,
            core::ptr::addr_of_mut!(BytesTransferred),
        );
        if Status == NDIS_STATUS_PENDING {
            KeWaitForSingleObject(
                (core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent)).cast(),
                _KWAIT_REASON_Executive,
                (0i32) as KPROCESSOR_MODE,
                FALSE,
                core::ptr::null_mut(),
            );
            Status = (*AdapterContext).AsyncStatus;
        }
        if Status != NDIS_STATUS_SUCCESS {
            DPRINT1!("Failed to transfer data with status 0x%x\n", Status);
            CleanupAndFreePacket(Packet, FALSE);
            ExFreePool(PacketBuffer);
            return NDIS_STATUS_NOT_ACCEPTED;
        }
    }

    /* Copy the header data */
    RtlCopyMemory!(PacketBuffer, HeaderBuffer, HeaderBufferSize as SIZE_T);

    /* Free the packet descriptor and buffers
       but not the pool because we still need it */
    CleanupAndFreePacket(Packet, FALSE);

    /* Allocate a packet entry from pool */
    PacketEntry = ExAllocatePool(
        NonPagedPool,
        (core::mem::size_of::<NDISUIO_PACKET_ENTRY>() as SIZE_T)
            .wrapping_add(BytesTransferred as SIZE_T)
            .wrapping_add(HeaderBufferSize as SIZE_T)
            .wrapping_sub(1 as SIZE_T),
    ) as PNDISUIO_PACKET_ENTRY;
    if PacketEntry.is_null() {
        ExFreePool(PacketBuffer);
        return NDIS_STATUS_RESOURCES;
    }

    /* Initialize the packet entry and copy in packet data */
    (*PacketEntry).PacketLength = BytesTransferred.wrapping_add(HeaderBufferSize);
    RtlCopyMemory!(
        core::ptr::addr_of_mut!((*PacketEntry).PacketData) as PVOID,
        PacketBuffer,
        (*PacketEntry).PacketLength as SIZE_T,
    );

    /* Free the old buffer */
    ExFreePool(PacketBuffer);

    /* Insert the packet on the adapter's packet list */
    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!((*AdapterContext).PacketList),
        core::ptr::addr_of_mut!((*PacketEntry).ListEntry),
        core::ptr::addr_of_mut!((*AdapterContext).Spinlock),
    );

    /* Signal the read event */
    KeSetEvent(
        core::ptr::addr_of_mut!((*AdapterContext).PacketReadEvent),
        (IO_NETWORK_INCREMENT) as KPRIORITY,
        FALSE,
    );

    return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NduReceiveComplete(ProtocolBindingContext: NDIS_HANDLE) {}

#[no_mangle]
pub unsafe extern "system" fn NduStatus(ProtocolBindingContext: NDIS_HANDLE, GeneralStatus: NDIS_STATUS, StatusBuffer: PVOID, StatusBufferSize: UINT) {}

#[no_mangle]
pub unsafe extern "system" fn NduStatusComplete(ProtocolBindingContext: NDIS_HANDLE) {}

unsafe extern "C" fn UnbindAdapterByContext(AdapterContext: PNDISUIO_ADAPTER_CONTEXT) -> NDIS_STATUS {
    let mut OldIrql: KIRQL = 0;
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut OpenEntry: PNDISUIO_OPEN_ENTRY = core::ptr::null_mut();
    let mut PacketEntry: PNDISUIO_PACKET_ENTRY = core::ptr::null_mut();
    let mut Status: NDIS_STATUS = 0;

    DPRINT!(
        "Unbinding adapter %wZ\n",
        core::ptr::addr_of_mut!((*AdapterContext).DeviceName)
    );

    /* FIXME: We don't do anything with outstanding reads */

    /* Remove the adapter context from the global list */
    KeAcquireSpinLock(
        core::ptr::addr_of_mut!(GlobalAdapterListLock),
        core::ptr::addr_of_mut!(OldIrql),
    );
    RemoveEntryList(core::ptr::addr_of_mut!((*AdapterContext).ListEntry));
    KeReleaseSpinLock(core::ptr::addr_of_mut!(GlobalAdapterListLock), OldIrql);

    /* Free the device name string */
    RtlFreeUnicodeString(core::ptr::addr_of_mut!((*AdapterContext).DeviceName));

    /* Invalidate all handles to this adapter */
    CurrentEntry = (*AdapterContext).OpenEntryList.Flink as PLIST_ENTRY;
    while CurrentEntry != core::ptr::addr_of_mut!((*AdapterContext).OpenEntryList) {
        OpenEntry = (CurrentEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(NDISUIO_OPEN_ENTRY, ListEntry) as ULONG_PTR,
        ) as *mut NDISUIO_OPEN_ENTRY as PNDISUIO_OPEN_ENTRY;

        /* Make sure the entry is sane */
        ASSERT!(!(*OpenEntry).FileObject.is_null());

        /* Remove the adapter context pointer */
        ASSERT!(AdapterContext == (*(*OpenEntry).FileObject).FsContext as PNDISUIO_ADAPTER_CONTEXT);
        (*(*OpenEntry).FileObject).FsContext = core::ptr::null_mut();
        (*AdapterContext).OpenCount = (*AdapterContext).OpenCount.wrapping_sub(1);

        /* Remove the open entry pointer */
        ASSERT!(OpenEntry == (*(*OpenEntry).FileObject).FsContext2 as PNDISUIO_OPEN_ENTRY);
        (*(*OpenEntry).FileObject).FsContext2 = core::ptr::null_mut();

        /* Move to the next entry */
        CurrentEntry = (*CurrentEntry).Flink as PLIST_ENTRY;

        /* Free the open entry */
        ExFreePool(OpenEntry as PVOID);
    }

    /* If this fails, we have a refcount mismatch somewhere */
    ASSERT!((*AdapterContext).OpenCount == 0 as ULONG);

    /* Free all pending packet entries */
    CurrentEntry = (*AdapterContext).PacketList.Flink as PLIST_ENTRY;
    while CurrentEntry != core::ptr::addr_of_mut!((*AdapterContext).PacketList) {
        PacketEntry = (CurrentEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(NDISUIO_PACKET_ENTRY, ListEntry) as ULONG_PTR,
        ) as *mut NDISUIO_PACKET_ENTRY as PNDISUIO_PACKET_ENTRY;

        /* Move to the next entry */
        CurrentEntry = (*CurrentEntry).Flink as PLIST_ENTRY;

        /* Free the packet entry */
        ExFreePool(PacketEntry as PVOID);
    }

    /* Send the close request */
    NdisCloseAdapter(
        core::ptr::addr_of_mut!(Status),
        (*AdapterContext).BindingHandle,
    );

    /* Wait for a pending close */
    if Status == NDIS_STATUS_PENDING as NDIS_STATUS {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent) as PVOID,
            (_KWAIT_REASON_Executive as KPROCESSOR_MODE) as KWAIT_REASON,
            0i32 as KPROCESSOR_MODE,
            FALSE as BOOLEAN,
            core::ptr::null_mut(),
        );
        Status = (*AdapterContext).AsyncStatus;
    }

    /* Free the context */
    ExFreePool(AdapterContext as PVOID);

    return Status;
}

unsafe extern "C" fn BindAdapterByName(DeviceName: PNDIS_STRING) -> NDIS_STATUS {
    let mut OpenErrorStatus: NDIS_STATUS = 0;
    let mut AdapterContext: PNDISUIO_ADAPTER_CONTEXT = core::ptr::null_mut();
    let mut SupportedMedia: [NDIS_MEDIUM; 1] = [NdisMedium802_3];
    let mut SelectedMedium: UINT = 0;
    let mut Status: NDIS_STATUS = 0;
    let mut Request: NDIS_REQUEST = core::mem::zeroed();
    let mut NameLength: ULONG = 0;

    /* Allocate the adapter context */
    AdapterContext = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<_NDISUIO_ADAPTER_CONTEXT>() as SIZE_T,
    ) as PNDISUIO_ADAPTER_CONTEXT;
    if AdapterContext.is_null() {
        return NDIS_STATUS_RESOURCES;
    }

    /* Set up the adapter context */
    RtlZeroMemory!(
        AdapterContext as *mut core::ffi::c_void,
        core::mem::size_of::<_NDISUIO_ADAPTER_CONTEXT>() as size_t,
    );
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent),
        _EVENT_TYPE_SynchronizationEvent,
        FALSE,
    );
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*AdapterContext).PacketReadEvent),
        _EVENT_TYPE_SynchronizationEvent,
        FALSE,
    );
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*AdapterContext).Spinlock));
    InitializeListHead(core::ptr::addr_of_mut!((*AdapterContext).PacketList));
    InitializeListHead(core::ptr::addr_of_mut!((*AdapterContext).OpenEntryList));
    (*AdapterContext).OpenCount = 0;

    /* Store the name without a terminator, so an open matches
     * whether or not the caller passes one */
    NameLength = (*DeviceName).Length as ULONG;
    if NameLength as usize >= core::mem::size_of::<WCHAR>() as usize
        && *(*DeviceName).Buffer.offset(
            ((NameLength as usize) / core::mem::size_of::<WCHAR>() - 1 as usize)
                as isize,
        ) == UNICODE_NULL
    {
        NameLength = NameLength.wrapping_sub(core::mem::size_of::<WCHAR>() as ULONG);
    }

    (*AdapterContext).DeviceName.MaximumLength = NameLength as USHORT;
    (*AdapterContext).DeviceName.Length = (*AdapterContext).DeviceName.MaximumLength;
    (*AdapterContext).DeviceName.Buffer =
        ExAllocatePool(NonPagedPool, NameLength as SIZE_T) as PWCH;
    if (*AdapterContext).DeviceName.Buffer.is_null() {
        ExFreePool(AdapterContext as PVOID);
        return NDIS_STATUS_RESOURCES;
    }

    /* Copy the device name into the adapter context */
    RtlCopyMemory!(
        (*AdapterContext).DeviceName.Buffer as *mut core::ffi::c_void,
        (*DeviceName).Buffer as *const core::ffi::c_void,
        NameLength as usize,
    );

    DPRINT!(
        "Binding adapter %wZ\n",
        core::ptr::addr_of_mut!((*AdapterContext).DeviceName),
    );

    /* Create the buffer pool */
    NdisAllocateBufferPool(
        core::ptr::addr_of_mut!(Status),
        core::ptr::addr_of_mut!((*AdapterContext).BufferPoolHandle),
        50,
    );
    if Status != NDIS_STATUS_SUCCESS {
        DPRINT1!(
            "Failed to allocate buffer pool with status 0x%x\n",
            Status,
        );
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*AdapterContext).DeviceName));
        ExFreePool(AdapterContext as PVOID);
        return Status;
    }

    /* Create the packet pool */
    NdisAllocatePacketPool(
        core::ptr::addr_of_mut!(Status),
        core::ptr::addr_of_mut!((*AdapterContext).PacketPoolHandle),
        25,
        (PROTOCOL_RESERVED_SIZE_IN_PACKET) as UINT,
    );
    if Status != NDIS_STATUS_SUCCESS {
        DPRINT1!(
            "Failed to allocate packet pool with status 0x%x\n",
            Status,
        );
        NdisFreeBufferPool((*AdapterContext).BufferPoolHandle);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*AdapterContext).DeviceName));
        ExFreePool(AdapterContext as PVOID);
        return Status;
    }

    /* Send the open request */
    NdisOpenAdapter(
        core::ptr::addr_of_mut!(Status),
        core::ptr::addr_of_mut!(OpenErrorStatus),
        core::ptr::addr_of_mut!((*AdapterContext).BindingHandle),
        core::ptr::addr_of_mut!(SelectedMedium),
        addr_of_mut!(SupportedMedia) as PNDIS_MEDIUM,
        1,
        GlobalProtocolHandle,
        AdapterContext as NDIS_HANDLE,
        DeviceName,
        0,
        core::ptr::null_mut(),
    );

    /* Wait for a pending open */
    if Status == NDIS_STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent) as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
        Status = (*AdapterContext).AsyncStatus;
    }

    /* Check the final status */
    if Status != NDIS_STATUS_SUCCESS {
        DPRINT1!(
            "Failed to open adapter for bind with status 0x%x\n",
            Status,
        );
        NdisFreePacketPool((*AdapterContext).PacketPoolHandle);
        NdisFreeBufferPool((*AdapterContext).BufferPoolHandle);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*AdapterContext).DeviceName));
        ExFreePool(AdapterContext as PVOID);
        return Status;
    }

    /* Get the MAC options */
    Request.RequestType = NdisRequestQueryInformation;
    Request.DATA.QUERY_INFORMATION.Oid = OID_GEN_MAC_OPTIONS;
    Request.DATA.QUERY_INFORMATION.InformationBuffer =
        core::ptr::addr_of_mut!((*AdapterContext).MacOptions) as PVOID;
    Request.DATA.QUERY_INFORMATION.InformationBufferLength =
        core::mem::size_of::<ULONG>() as UINT;
    NdisRequest(
        core::ptr::addr_of_mut!(Status),
        (*AdapterContext).BindingHandle,
        core::ptr::addr_of_mut!(Request),
    );

    /* Wait for a pending request */
    if Status == NDIS_STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent) as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
        Status = (*AdapterContext).AsyncStatus;
    }

    /* Check the final status */
    if Status != NDIS_STATUS_SUCCESS {
        let mut CloseStatus: NDIS_STATUS = 0;

        DPRINT1!(
            "Failed to get MAC options with status 0x%x\n",
            Status,
        );

        NdisCloseAdapter(
            core::ptr::addr_of_mut!(CloseStatus),
            (*AdapterContext).BindingHandle,
        );
        if CloseStatus == NDIS_STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent) as PVOID,
                _KWAIT_REASON_Executive,
                (0i32) as KPROCESSOR_MODE,
                FALSE,
                core::ptr::null_mut(),
            );
        }

        NdisFreePacketPool((*AdapterContext).PacketPoolHandle);
        NdisFreeBufferPool((*AdapterContext).BufferPoolHandle);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*AdapterContext).DeviceName));
        ExFreePool(AdapterContext as PVOID);
        return Status;
    }

    /* Add the adapter context to the global list */
    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!(GlobalAdapterList),
        core::ptr::addr_of_mut!((*AdapterContext).ListEntry),
        core::ptr::addr_of_mut!(GlobalAdapterListLock),
    );

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NduBindAdapter(Status: PNDIS_STATUS, BindContext: NDIS_HANDLE, DeviceName: PNDIS_STRING, SystemSpecific1: PVOID, SystemSpecific2: PVOID) {
    *Status = BindAdapterByName(DeviceName);
}
#[no_mangle]
pub unsafe extern "system" fn NduUnbindAdapter(Status: PNDIS_STATUS, ProtocolBindingContext: NDIS_HANDLE, UnbindContext: NDIS_HANDLE) {
    *Status = UnbindAdapterByContext(ProtocolBindingContext as PNDISUIO_ADAPTER_CONTEXT);
}
