// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_0D1A236C5C8A: [u16; 5] = [0x0042u16, 0x0069, 0x006E, 0x0064, 0x0000];

use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePool, ExFreePool, KEY_VALUE_PARTIAL_INFORMATION, LIST_ENTRY, _OBJECT_ATTRIBUTES, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PKEY_VALUE_PARTIAL_INFORMATION, PSCATTER_GATHER_LIST, PSTRING, RtlCompareUnicodeString, RtlInitUnicodeString, RtlUpcaseUnicodeString, ZwClose, ZwOpenKey, ZwQueryValueKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const SERVICES_KEY: [::core::ffi::c_ushort; 53] = unsafe {
    ::core::mem::transmute::<
        [u8; 106],
        [::core::ffi::c_ushort; 53],
    >(
        *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0\0\0",
    )
};

pub const LINKAGE_KEY: [::core::ffi::c_ushort; 9] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_ushort; 9]>(
        *b"\\\0L\0i\0n\0k\0a\0g\0e\0\0\0",
    )
};

pub const PARAMETERS_KEY: [::core::ffi::c_ushort; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_ushort; 13]>(
        *b"\\\0P\0a\0r\0a\0m\0e\0t\0e\0r\0s\0\\\0\0\0",
    )
};

#[no_mangle]
pub static mut ProtocolListHead: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

#[no_mangle]
pub static mut ProtocolListLock: KSPIN_LOCK = 0;

pub const WORKER_TEST: i32 = 0;

#[repr(C)]
pub struct _DMA_CONTEXT {
    pub Adapter: PLOGICAL_ADAPTER,
    pub Packet: PNDIS_PACKET,
}
pub type DMA_CONTEXT = _DMA_CONTEXT;
pub type PDMA_CONTEXT = *mut _DMA_CONTEXT;
#[no_mangle]
pub unsafe extern "C" fn ProSetupPnPEvent(
    EventCode: NET_PNP_EVENT_CODE,
    EventBuffer: PVOID,
    EventBufferLength: ULONG,
) -> PNET_PNP_EVENT {
    let mut PnPEvent: PNET_PNP_EVENT;

    PnPEvent = ExAllocatePool(
        PagedPool,
        core::mem::size_of::<NET_PNP_EVENT>() as SIZE_T,
    ) as PNET_PNP_EVENT;
    if PnPEvent.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, "Insufficient resources\n");
        return core::ptr::null_mut();
    }

    core::ptr::write_bytes(
        PnPEvent as *mut u8,
        0,
        core::mem::size_of::<NET_PNP_EVENT>(),
    );

    (*PnPEvent).NetEvent = EventCode;

    if !EventBuffer.is_null() {
        (*PnPEvent).Buffer = ExAllocatePool(
            PagedPool,
            EventBufferLength as SIZE_T,
        );
        if (*PnPEvent).Buffer.is_null() {
            NDIS_DbgPrint!(MIN_TRACE, "Insufficient resources\n");
            ExFreePool(PnPEvent as PVOID);
            return core::ptr::null_mut();
        }

        (*PnPEvent).BufferLength = EventBufferLength;

        core::ptr::copy_nonoverlapping(
            EventBuffer as *const u8,
            (*PnPEvent).Buffer as *mut u8,
            (*PnPEvent).BufferLength as usize,
        );
    }

    return PnPEvent;
}
#[no_mangle]
pub unsafe extern "C" fn ProSendAndFreePnPEvent(
   Adapter: PLOGICAL_ADAPTER,
   PnPEvent: PNET_PNP_EVENT,
   Irp: PIRP) -> NDIS_STATUS {
  let mut CurrentEntry: PLIST_ENTRY;
  let mut Status: NDIS_STATUS;
  let mut AdapterBinding: *mut ADAPTER_BINDING;

  CurrentEntry = (*Adapter).ProtocolListHead.Flink;

  while CurrentEntry != core::ptr::addr_of_mut!((*Adapter).ProtocolListHead)
  {
     AdapterBinding = CONTAINING_RECORD!(CurrentEntry, ADAPTER_BINDING, AdapterListEntry);

     let PnPEventHandler = (*(*AdapterBinding).ProtocolBinding).Chars.__bindgen_anon_1.PnPEventHandler
         .unwrap_unchecked();
     Status = PnPEventHandler(
      (*AdapterBinding).NdisOpenBlock.ProtocolBindingContext,
      PnPEvent);

     if Status == NDIS_STATUS_PENDING
     {
         IoMarkIrpPending(Irp);
         /* Yes, I know this is stupid */
         (*PnPEvent).NdisReserved[0 as usize] = Irp as ULONG_PTR;
         (*PnPEvent).NdisReserved[1 as usize] = (*CurrentEntry).Flink as ULONG_PTR;
         return NDIS_STATUS_PENDING;
     }
     else if Status != NDIS_STATUS_SUCCESS
     {
         if !(*PnPEvent).Buffer.is_null() { ExFreePool((*PnPEvent).Buffer); }
         ExFreePool(PnPEvent as PVOID);
         return Status;
     }

     CurrentEntry = (*CurrentEntry).Flink;
  }

  if !(*PnPEvent).Buffer.is_null() { ExFreePool((*PnPEvent).Buffer); }
  ExFreePool(PnPEvent as PVOID);

  return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NdisIPwrSetPower(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Adapter: PLOGICAL_ADAPTER = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;
    let mut PnPEvent: PNET_PNP_EVENT = core::ptr::null_mut();
    let mut Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    PnPEvent = ProSetupPnPEvent(
        NetEventSetPower,
        core::ptr::addr_of_mut!((*Stack).Parameters.Power.State) as PVOID,
        core::mem::size_of::<NDIS_DEVICE_POWER_STATE>() as ULONG,
    );
    if PnPEvent.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            (b"Insufficient resources\n\0".as_ptr() as *const i8),
        );
        return NDIS_STATUS_RESOURCES as NTSTATUS;
    }

    return ProSendAndFreePnPEvent(Adapter, PnPEvent, Irp) as NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn NdisIPwrQueryPower(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Adapter: PLOGICAL_ADAPTER = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;
    let mut PnPEvent: PNET_PNP_EVENT = core::ptr::null_mut();
    let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    PnPEvent = ProSetupPnPEvent(
        NetEventQueryPower,
        core::ptr::addr_of_mut!((*Stack).Parameters.Power.State) as PVOID,
        core::mem::size_of::<NDIS_DEVICE_POWER_STATE>() as ULONG,
    );
    if PnPEvent.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources\n\0".as_ptr() as *const i8,
        );
        return NDIS_STATUS_RESOURCES;
    }

    return ProSendAndFreePnPEvent(Adapter, PnPEvent, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn NdisIPnPQueryStopDevice(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Adapter: PLOGICAL_ADAPTER = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;
    let mut PnPEvent: PNET_PNP_EVENT;

    PnPEvent = ProSetupPnPEvent(NetEventQueryRemoveDevice, core::ptr::null_mut(), 0);
    if PnPEvent.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources\n\0".as_ptr() as *const i8);
        return NDIS_STATUS_RESOURCES as NTSTATUS;
    }

    return ProSendAndFreePnPEvent(Adapter, PnPEvent, Irp) as NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn NdisIPnPCancelStopDevice(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Adapter: PLOGICAL_ADAPTER = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;
    let mut PnPEvent: PNET_PNP_EVENT;

    PnPEvent = ProSetupPnPEvent(NetEventCancelRemoveDevice, core::ptr::null_mut(), 0);
    if PnPEvent.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources\n\0".as_ptr() as PCSTR);
        return NDIS_STATUS_RESOURCES as NTSTATUS;
    }

    return ProSendAndFreePnPEvent(Adapter, PnPEvent, Irp) as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisCompleteBindAdapter(
    BindAdapterContext: NDIS_HANDLE,
    Status: NDIS_STATUS,
    OpenStatus: NDIS_STATUS,
) {
    let mut Protocol: *mut PROTOCOL_BINDING = BindAdapterContext as *mut PROTOCOL_BINDING;
    if !NT_SUCCESS!(Status) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Binding failed (%x)\n\0".as_ptr() as PCSTR,
            Status,
        );
        return;
    }

    /* Put protocol binding struct on global list */
    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!(ProtocolListHead),
        core::ptr::addr_of_mut!((*Protocol).ListEntry),
        core::ptr::addr_of_mut!(ProtocolListLock),
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisCompleteUnbindAdapter(
        UnbindAdapterContext: NDIS_HANDLE,
        Status: NDIS_STATUS) {
  let Protocol: *mut PROTOCOL_BINDING = UnbindAdapterContext as *mut PROTOCOL_BINDING;

  if !NT_SUCCESS!(Status) {
      NDIS_DbgPrint!(MIN_TRACE, b"Unbinding failed (%x)\n\0".as_ptr() as *const i8, Status);
      return;
  }

  ExInterlockedRemoveEntryList!(core::ptr::addr_of_mut!((*Protocol).ListEntry), core::ptr::addr_of_mut!(ProtocolListLock));
}
#[no_mangle]
pub unsafe extern "C" fn ProIndicatePacket(Adapter: PLOGICAL_ADAPTER, Packet: PNDIS_PACKET) -> NDIS_STATUS {
    let mut BufferedLength: UINT = 0;
    let mut PacketLength: UINT = 0;
    let mut OldIrql: KIRQL = 0;
    let mut LookaheadBuffer: PUCHAR = core::ptr::null_mut();

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    MiniDisplayPacket(Packet, b"INDICATE\0".as_ptr() as PCSTR);

    NdisQueryPacket(
        Packet,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &raw mut PacketLength,
    );

    LookaheadBuffer = ExAllocatePool(NonPagedPool, PacketLength as SIZE_T) as PUCHAR;
    if LookaheadBuffer.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources\n\0".as_ptr() as PCSTR);
        return NDIS_STATUS_RESOURCES;
    }

    NDIS_DbgPrint!(MAX_TRACE, b"acquiring miniport block lock\n\0".as_ptr() as PCSTR);
    KeAcquireSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, &raw mut OldIrql);
    {
        BufferedLength = CopyPacketToBuffer(LookaheadBuffer, Packet, 0, PacketLength);
        (*Adapter).NdisMiniportBlock.IndicatedPacket[KeGetCurrentProcessorNumber() as usize] = Packet;
    }
    KeReleaseSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, OldIrql);

    if BufferedLength > (*Adapter).MediumHeaderSize as UINT {
        /* XXX Change this to call SendPackets so we don't have to duplicate this wacky logic */
        MiniIndicateData(
            Adapter,
            core::ptr::null_mut(),
            LookaheadBuffer as PVOID,
            (*Adapter).MediumHeaderSize as UINT,
            LookaheadBuffer.add((*Adapter).MediumHeaderSize as usize) as PVOID,
            BufferedLength.wrapping_sub((*Adapter).MediumHeaderSize as UINT),
            PacketLength.wrapping_sub((*Adapter).MediumHeaderSize as UINT),
        );
    } else {
        MiniIndicateData(
            Adapter,
            core::ptr::null_mut(),
            LookaheadBuffer as PVOID,
            (*Adapter).MediumHeaderSize as UINT,
            core::ptr::null_mut(),
            0,
            0,
        );
    }

    ExFreePool(LookaheadBuffer as PVOID);

    KeAcquireSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, &raw mut OldIrql);
    {
        (*Adapter).NdisMiniportBlock.IndicatedPacket[KeGetCurrentProcessorNumber() as usize] =
            core::ptr::null_mut();
    }
    KeReleaseSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, OldIrql);

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn ProRequest(MacBindingHandle: NDIS_HANDLE, NdisRequest: PNDIS_REQUEST) -> NDIS_STATUS {
    let mut AdapterBinding: PADAPTER_BINDING;
    let mut Adapter: PLOGICAL_ADAPTER;
    let mut MacBlock: PNDIS_REQUEST_MAC_BLOCK =
        &raw mut (*NdisRequest).MacReserved as *mut UCHAR as PNDIS_REQUEST_MAC_BLOCK;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    AdapterBinding = GET_ADAPTER_BINDING!(MacBindingHandle);

    Adapter = (*AdapterBinding).Adapter;

    (*MacBlock).Binding = &raw mut (*AdapterBinding).NdisOpenBlock as PNDIS_OPEN_BLOCK;

    if false {
        MiniQueueWorkItem(Adapter, NdisWorkItemRequest, (NdisRequest).cast(), FALSE);
        return NDIS_STATUS_PENDING;
    }
    if MiniIsBusy(Adapter, NdisWorkItemRequest) != 0 {
        MiniQueueWorkItem(Adapter, NdisWorkItemRequest, (NdisRequest).cast(), FALSE);
        return NDIS_STATUS_PENDING;
    }

    return MiniDoRequest(Adapter, NdisRequest);
}
#[no_mangle]
pub unsafe extern "system" fn ProReset(MacBindingHandle: NDIS_HANDLE) -> NDIS_STATUS {
    let AdapterBinding: PADAPTER_BINDING = MacBindingHandle as PADAPTER_BINDING;

    return MiniReset((*AdapterBinding).Adapter);
}
#[no_mangle]
pub unsafe extern "system" fn ScatterGatherSendPacket(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, ScatterGather: PSCATTER_GATHER_LIST, Context: PVOID) {
    let DmaContext: PDMA_CONTEXT = Context as PDMA_CONTEXT;
    let Adapter: PLOGICAL_ADAPTER = (*DmaContext).Adapter;
    let Packet: PNDIS_PACKET = (*DmaContext).Packet;
    let mut Status: NDIS_STATUS;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    NDIS_PER_PACKET_INFO_FROM_PACKET!(Packet, ScatterGatherListPacketInfo) = ScatterGather as PVOID;

    Status = proSendPacketToMiniport(Adapter, Packet);

    if Status != NDIS_STATUS_PENDING {
        NDIS_DbgPrint!(MAX_TRACE, b"Completing packet.\n\0".as_ptr() as PCSTR);
        MiniSendComplete(Adapter as NDIS_HANDLE, Packet, Status);
    }

    ExFreePool(DmaContext as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn proSendPacketToMiniport(Adapter: PLOGICAL_ADAPTER, mut Packet: PNDIS_PACKET) -> NDIS_STATUS {
    if 0 != 0 {
        MiniQueueWorkItem(Adapter, NdisWorkItemSend, (Packet).cast(), FALSE);
        return NDIS_STATUS_PENDING;
    }
    let mut RaiseOldIrql: KIRQL = 0;
    let mut NdisStatus: NDIS_STATUS = 0;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    if MiniIsBusy(Adapter, NdisWorkItemSend) != 0 {
        NDIS_DbgPrint!(MID_TRACE, b"Busy: NdisWorkItemSend.\n\0".as_ptr() as PCSTR);

        MiniQueueWorkItem(Adapter, NdisWorkItemSend, (Packet).cast(), FALSE);
        return NDIS_STATUS_PENDING;
    }

    MiniDisplayPacket(Packet, b"SEND\0".as_ptr() as PCSTR);

    if (*(*Adapter).NdisMiniportBlock.DriverHandle).MiniportCharacteristics.SendPacketsHandler.is_some()
    {
        if ((*Adapter).NdisMiniportBlock.Flags & NDIS_ATTRIBUTE_DESERIALIZE) != 0
        {
            NDIS_DbgPrint!(MAX_TRACE, b"Calling miniport's SendPackets handler\n\0".as_ptr() as PCSTR);
            (*(*Adapter).NdisMiniportBlock.DriverHandle).MiniportCharacteristics.SendPacketsHandler.unwrap_unchecked()(
                (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                core::ptr::addr_of_mut!(Packet),
                1,
            );
            NdisStatus = NDIS_STATUS_PENDING;
        } else {
            /* SendPackets is called at DISPATCH_LEVEL for all serialized miniports */
            KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(RaiseOldIrql));
            {
                NDIS_DbgPrint!(MAX_TRACE, b"Calling miniport's SendPackets handler\n\0".as_ptr() as PCSTR);
                (*(*Adapter).NdisMiniportBlock.DriverHandle).MiniportCharacteristics.SendPacketsHandler.unwrap_unchecked()(
                    (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                    core::ptr::addr_of_mut!(Packet),
                    1,
                );
            }
            KeLowerIrql(RaiseOldIrql);

            NdisStatus = NDIS_GET_PACKET_STATUS!(Packet);
            if NdisStatus == NDIS_STATUS_RESOURCES {
                MiniQueueWorkItem(Adapter, NdisWorkItemSend, (Packet).cast(), TRUE);
                NdisStatus = NDIS_STATUS_PENDING;
            }
        }

        if NdisStatus != NDIS_STATUS_PENDING {
            MiniWorkItemComplete(Adapter, NdisWorkItemSend);
        }

        return NdisStatus;
    } else {
        if ((*Adapter).NdisMiniportBlock.Flags & NDIS_ATTRIBUTE_DESERIALIZE) != 0
        {
            NDIS_DbgPrint!(MAX_TRACE, b"Calling miniport's Send handler\n\0".as_ptr() as PCSTR);
            NdisStatus = (*(*Adapter).NdisMiniportBlock.DriverHandle).MiniportCharacteristics.SendHandler.unwrap_unchecked()(
                (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                Packet,
                (*Packet).Private.Flags as UINT,
            );
            NDIS_DbgPrint!(MAX_TRACE, b"back from miniport's send handler\n\0".as_ptr() as PCSTR);
        } else {
            /* Send is called at DISPATCH_LEVEL for all serialized miniports */
            KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(RaiseOldIrql));
            NDIS_DbgPrint!(MAX_TRACE, b"Calling miniport's Send handler\n\0".as_ptr() as PCSTR);
            NdisStatus = (*(*Adapter).NdisMiniportBlock.DriverHandle).MiniportCharacteristics.SendHandler.unwrap_unchecked()(
                (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                Packet,
                (*Packet).Private.Flags as UINT,
            );
            NDIS_DbgPrint!(MAX_TRACE, b"back from miniport's send handler\n\0".as_ptr() as PCSTR);
            KeLowerIrql(RaiseOldIrql);

            if NdisStatus == NDIS_STATUS_RESOURCES {
                MiniQueueWorkItem(Adapter, NdisWorkItemSend, (Packet).cast(), TRUE);
                NdisStatus = NDIS_STATUS_PENDING;
            }
        }

        if NdisStatus != NDIS_STATUS_PENDING {
            MiniWorkItemComplete(Adapter, NdisWorkItemSend);
        }

        return NdisStatus;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ProSend(MacBindingHandle: NDIS_HANDLE, Packet: PNDIS_PACKET) -> NDIS_STATUS {
    let mut AdapterBinding: PADAPTER_BINDING = core::ptr::null_mut();
    let mut Adapter: PLOGICAL_ADAPTER = core::ptr::null_mut();
    let mut NdisBuffer: PNDIS_BUFFER = core::ptr::null_mut();
    let mut Context: PDMA_CONTEXT = core::ptr::null_mut();
    let mut NdisStatus: NDIS_STATUS = 0;
    let mut PacketLength: UINT = 0;
    let mut OldIrql: KIRQL = 0;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    AdapterBinding = GET_ADAPTER_BINDING!(MacBindingHandle);

    Adapter = (*AdapterBinding).Adapter;

    /* if the following is not true, KeRaiseIrql() below will break */

    /* XXX what is this crazy black magic? */
    (*Packet).Reserved[1 as usize] = MacBindingHandle as ULONG_PTR;

    /*
     * Test the packet to see if it is a MAC loopback.
     *
     * We may have to loop this packet if miniport cannot.
     * If dest MAC address of packet == MAC address of adapter,
     * this is a loopback frame.
     */

    if (((*Adapter).NdisMiniportBlock.MacOptions & NDIS_MAC_OPTION_NO_LOOPBACK) != 0)
        && (MiniAdapterHasAddress(Adapter, Packet) != 0)
    {
        // WORKER_TEST (defined as 0 in the C source)
        if 0 != 0 {
            MiniQueueWorkItem(
                Adapter,
                NdisWorkItemSendLoopback,
                (Packet).cast(),
                FALSE,
            );
            return NDIS_STATUS_PENDING;
        } else {
            return ProIndicatePacket(Adapter, Packet);
        }
    } else {
        if (*Adapter).NdisMiniportBlock.ScatterGatherListSize != 0 {
            NDIS_DbgPrint!(
                MID_TRACE,
                b"Using Scatter/Gather DMA\n\0".as_ptr() as PCSTR,
            );

            NdisQueryPacket(
                Packet,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(NdisBuffer),
                core::ptr::addr_of_mut!(PacketLength),
            );

            Context = ExAllocatePool(
                NonPagedPool,
                core::mem::size_of::<DMA_CONTEXT>() as SIZE_T,
            ) as PDMA_CONTEXT;
            if Context.is_null() {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"Insufficient resources\n\0".as_ptr() as PCSTR,
                );
                return NDIS_STATUS_RESOURCES;
            }

            (*Context).Adapter = Adapter;
            (*Context).Packet = Packet;

            KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(OldIrql));

            KeFlushIoBuffers(
                NdisBuffer,
                FALSE,
                TRUE,
            );

            NdisStatus = (*(*(*Adapter).NdisMiniportBlock.SystemAdapterObject).DmaOperations)
                .GetScatterGatherList
                .unwrap_unchecked()(
                (*Adapter).NdisMiniportBlock.SystemAdapterObject,
                (*Adapter).NdisMiniportBlock.PhysicalDeviceObject,
                NdisBuffer as PMDL,
                MmGetMdlVirtualAddress!(NdisBuffer as PMDL),
                PacketLength as ULONG,
                Some(
                    ScatterGatherSendPacket,
                ),
                Context as PVOID,
                TRUE as BOOLEAN,
            ) as NDIS_STATUS;

            KeLowerIrql(OldIrql);

            if !NT_SUCCESS!(NdisStatus) {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"GetScatterGatherList failed! (%x)\n\0".as_ptr() as PCSTR,
                    NdisStatus,
                );
                return NdisStatus;
            }

            return NDIS_STATUS_PENDING;
        }

        return proSendPacketToMiniport(Adapter, Packet);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ProSendPackets(NdisBindingHandle: NDIS_HANDLE, PacketArray: PPNDIS_PACKET, NumberOfPackets: UINT) {
    let mut AdapterBinding: PADAPTER_BINDING = NdisBindingHandle as PADAPTER_BINDING;
    let mut Adapter: PLOGICAL_ADAPTER = (*AdapterBinding).Adapter;
    let mut RaiseOldIrql: KIRQL = 0;
    let mut NdisStatus: NDIS_STATUS = 0;
    let mut i: UINT = 0;

    if (*(*Adapter).NdisMiniportBlock.DriverHandle)
        .MiniportCharacteristics
        .SendPacketsHandler
        .is_some()
    {
        if (*Adapter).NdisMiniportBlock.Flags & NDIS_ATTRIBUTE_DESERIALIZE != 0 {
            ((*(*Adapter).NdisMiniportBlock.DriverHandle)
                .MiniportCharacteristics
                .SendPacketsHandler
                .unwrap_unchecked())(
                (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                PacketArray,
                NumberOfPackets,
            );
        } else {
            /* SendPackets is called at DISPATCH_LEVEL for all serialized miniports */
            KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(RaiseOldIrql));
            ((*(*Adapter).NdisMiniportBlock.DriverHandle)
                .MiniportCharacteristics
                .SendPacketsHandler
                .unwrap_unchecked())(
                (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                PacketArray,
                NumberOfPackets,
            );
            KeLowerIrql(RaiseOldIrql);
            i = 0;
            while i < NumberOfPackets {
                NdisStatus = NDIS_GET_PACKET_STATUS!(*PacketArray.offset(i as isize));
                if NdisStatus != NDIS_STATUS_PENDING {
                    MiniSendComplete((Adapter).cast(), *PacketArray.offset(i as isize), NdisStatus);
                }
                i += 1;
            }
        }
    } else {
        if (*Adapter).NdisMiniportBlock.Flags & NDIS_ATTRIBUTE_DESERIALIZE != 0 {
            i = 0;
            while i < NumberOfPackets {
                NdisStatus = ((*(*Adapter).NdisMiniportBlock.DriverHandle)
                    .MiniportCharacteristics
                    .SendHandler
                    .unwrap_unchecked())(
                    (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                    *PacketArray.offset(i as isize),
                    (**PacketArray.offset(i as isize)).Private.Flags,
                );
                if NdisStatus != NDIS_STATUS_PENDING {
                    MiniSendComplete((Adapter).cast(), *PacketArray.offset(i as isize), NdisStatus);
                }
                i += 1;
            }
        } else {
            /* Send is called at DISPATCH_LEVEL for all serialized miniports */
            KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(RaiseOldIrql));
            i = 0;
            while i < NumberOfPackets {
                NdisStatus = ((*(*Adapter).NdisMiniportBlock.DriverHandle)
                    .MiniportCharacteristics
                    .SendHandler
                    .unwrap_unchecked())(
                    (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                    *PacketArray.offset(i as isize),
                    (**PacketArray.offset(i as isize)).Private.Flags,
                );
                if NdisStatus != NDIS_STATUS_PENDING {
                    MiniSendComplete((Adapter).cast(), *PacketArray.offset(i as isize), NdisStatus);
                }
                i += 1;
            }
            KeLowerIrql(RaiseOldIrql);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn ProTransferData(MacBindingHandle: NDIS_HANDLE, MacReceiveContext: NDIS_HANDLE, ByteOffset: UINT, BytesToTransfer: UINT, Packet: PNDIS_PACKET, BytesTransferred: PUINT) -> NDIS_STATUS {
    let AdapterBinding: PADAPTER_BINDING = GET_ADAPTER_BINDING!(MacBindingHandle);
    let Adapter: PLOGICAL_ADAPTER = (*AdapterBinding).Adapter;
    let mut Status: NDIS_STATUS = 0;
    let mut OldIrql: KIRQL = 0;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    if !(*Adapter).NdisMiniportBlock.IndicatedPacket[KeGetCurrentProcessorNumber() as usize].is_null() {
        NDIS_DbgPrint!(MAX_TRACE, b"LoopPacket\n\0".as_ptr() as *const i8);
        NdisCopyFromPacketToPacket(
            Packet,
            (ByteOffset as ULONG).wrapping_add((*Adapter).MediumHeaderSize as ULONG) as UINT,
            (BytesToTransfer as ULONG).wrapping_add((*Adapter).MediumHeaderSize as ULONG) as UINT,
            (*Adapter).NdisMiniportBlock.IndicatedPacket[KeGetCurrentProcessorNumber() as usize],
            0,
            BytesTransferred,
        );
        return NDIS_STATUS_SUCCESS;
    }

    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(OldIrql));

    let TransferDataHandler = (*(*Adapter).NdisMiniportBlock.DriverHandle)
        .MiniportCharacteristics
        .TransferDataHandler
        .unwrap_unchecked();
    Status = TransferDataHandler(
        Packet,
        BytesTransferred,
        (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
        MacReceiveContext,
        ByteOffset,
        BytesToTransfer,
    );

    KeLowerIrql(OldIrql);

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn NdisCloseAdapter(Status: PNDIS_STATUS, NdisBindingHandle: NDIS_HANDLE) -> () {
    let AdapterBinding: PADAPTER_BINDING = GET_ADAPTER_BINDING!(NdisBindingHandle);

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    /* Remove from protocol's bound adapters list */
    ExInterlockedRemoveEntryList!(
        core::ptr::addr_of_mut!((*AdapterBinding).ProtocolListEntry),
        core::ptr::addr_of_mut!((*(*AdapterBinding).ProtocolBinding).Lock),
    );

    /* Remove protocol from adapter's bound protocols list */
    ExInterlockedRemoveEntryList!(
        core::ptr::addr_of_mut!((*AdapterBinding).AdapterListEntry),
        core::ptr::addr_of_mut!((*(*AdapterBinding).Adapter).NdisMiniportBlock.Lock),
    );

    ExFreePool(AdapterBinding as PVOID);

    *Status = NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisDeregisterProtocol(
    Status: PNDIS_STATUS,
    NdisProtocolHandle: NDIS_HANDLE,
) {
    let Protocol: PPROTOCOL_BINDING = GET_PROTOCOL_BINDING!(NdisProtocolHandle);

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    /* FIXME: Make sure no adapter bindings exist */

    /* Remove protocol from global list */
    ExInterlockedRemoveEntryList!(
        core::ptr::addr_of_mut!((*Protocol).ListEntry),
        core::ptr::addr_of_mut!(ProtocolListLock),
    );

    ExFreePool(Protocol as PVOID);

    *Status = NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisOpenAdapter(
    Status: PNDIS_STATUS,
    OpenErrorStatus: PNDIS_STATUS,
    NdisBindingHandle: PNDIS_HANDLE,
    SelectedMediumIndex: PUINT,
    MediumArray: PNDIS_MEDIUM,
    MediumArraySize: UINT,
    NdisProtocolHandle: NDIS_HANDLE,
    ProtocolBindingContext: NDIS_HANDLE,
    AdapterName: PNDIS_STRING,
    OpenOptions: UINT,
    AddressingInformation: PSTRING,
) {
    let mut i: UINT;
    let mut Found: BOOLEAN;
    let mut Adapter: PLOGICAL_ADAPTER;
    let mut AdapterBinding: PADAPTER_BINDING;
    let mut Protocol: PPROTOCOL_BINDING =
        GET_PROTOCOL_BINDING!(NdisProtocolHandle) as PPROTOCOL_BINDING;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0" as *const u8 as PCSTR);

    if NdisProtocolHandle.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"NdisProtocolHandle is NULL\n\0" as *const u8 as PCSTR,
        );
        *OpenErrorStatus = NDIS_STATUS_FAILURE;
        *Status = *OpenErrorStatus;
        return;
    }

    Adapter = MiniLocateDevice(AdapterName);
    if Adapter.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Adapter not found.\n\0" as *const u8 as PCSTR,
        );
        *Status = NDIS_STATUS_ADAPTER_NOT_FOUND;
        return;
    }

    /* Find the media type in the list provided by the protocol driver */
    Found = FALSE;
    i = 0;
    while i < MediumArraySize {
        if (*Adapter).NdisMiniportBlock.MediaType == *MediumArray.offset(i as isize) {
            *SelectedMediumIndex = i;
            Found = TRUE;
            break;
        }
        i += 1;
    }

    if Found == 0 {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Medium is not supported.\n\0" as *const u8 as PCSTR,
        );
        *Status = NDIS_STATUS_UNSUPPORTED_MEDIA;
        return;
    }

    /* Now that we have confirmed that the adapter can be opened, create a binding */

    AdapterBinding = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<ADAPTER_BINDING>() as SIZE_T,
    ) as PADAPTER_BINDING;
    if AdapterBinding.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources.\n\0" as *const u8 as PCSTR,
        );
        *Status = NDIS_STATUS_RESOURCES;
        return;
    }

    RtlZeroMemory!(
        AdapterBinding as PVOID,
        core::mem::size_of::<ADAPTER_BINDING>() as SIZE_T,
    );

    (*AdapterBinding).ProtocolBinding = Protocol;
    (*AdapterBinding).Adapter = Adapter;
    (*AdapterBinding).NdisOpenBlock.ProtocolBindingContext = ProtocolBindingContext;

    /* Set fields required by some NDIS macros */
    (*AdapterBinding).NdisOpenBlock.BindingHandle = AdapterBinding as NDIS_HANDLE;

    /* Set handlers (some NDIS macros require these) */
    (*AdapterBinding).NdisOpenBlock.RequestHandler =
        Some(ProRequest);
    (*AdapterBinding).NdisOpenBlock.ResetHandler =
        Some(ProReset);
    (*AdapterBinding).NdisOpenBlock.__bindgen_anon_1.SendHandler =
        Some(ProSend);
    (*AdapterBinding).NdisOpenBlock.SendPacketsHandler =
        Some(ProSendPackets);
    (*AdapterBinding).NdisOpenBlock.TransferDataHandler = Some(ProTransferData);

    (*AdapterBinding).NdisOpenBlock.RequestCompleteHandler =
        (*Protocol).Chars.__bindgen_anon_1.RequestCompleteHandler;

    /* Put on protocol's bound adapters list */
    ExInterlockedInsertTailList(
        &raw mut (*Protocol).AdapterListHead,
        &raw mut (*AdapterBinding).ProtocolListEntry,
        &raw mut (*Protocol).Lock,
    );

    /* Put protocol on adapter's bound protocols list */
    NDIS_DbgPrint!(
        MAX_TRACE,
        b"acquiring miniport block lock\n\0" as *const u8 as PCSTR,
    );
    ExInterlockedInsertTailList(
        &raw mut (*Adapter).ProtocolListHead,
        &raw mut (*AdapterBinding).AdapterListEntry,
        &raw mut (*Adapter).NdisMiniportBlock.Lock,
    );

    *NdisBindingHandle = AdapterBinding as NDIS_HANDLE;

    *Status = NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn LocateAdapterBindingByName(
    ProtocolBinding: PPROTOCOL_BINDING,
    AdapterName: PNDIS_STRING,
) -> PADAPTER_BINDING {
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut AdapterBinding: PADAPTER_BINDING = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*ProtocolBinding).Lock),
        core::ptr::addr_of_mut!(OldIrql),
    );

    CurrentEntry = (*ProtocolBinding).AdapterListHead.Flink;

    while CurrentEntry != core::ptr::addr_of_mut!((*ProtocolBinding).AdapterListHead) {
        AdapterBinding = ((CurrentEntry as usize).wrapping_sub(
            core::mem::offset_of!(ADAPTER_BINDING, ProtocolListEntry),
        )) as PADAPTER_BINDING;

        if RtlCompareUnicodeString(
            AdapterName as PCUNICODE_STRING,
            core::ptr::addr_of_mut!((*(*AdapterBinding).Adapter).NdisMiniportBlock.MiniportName)
                as PCUNICODE_STRING,
            TRUE as BOOLEAN,
        ) == 0
        {
            KeReleaseSpinLock(
                core::ptr::addr_of_mut!((*ProtocolBinding).Lock),
                OldIrql,
            );
            return AdapterBinding;
        }

        CurrentEntry = (*CurrentEntry).Flink;
    }

    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*ProtocolBinding).Lock),
        OldIrql,
    );

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn ndisBindMiniportsToProtocol(Status: PNDIS_STATUS, Protocol: PPROTOCOL_BINDING) {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut RegistryPath: _UNICODE_STRING = core::mem::zeroed();
    let mut RegistryPathStr: *mut WCHAR = core::ptr::null_mut();
    let mut DataPtr: *mut WCHAR = core::ptr::null_mut();
    let mut NtStatus: NTSTATUS = 0;
    let mut DriverKeyHandle: HANDLE = core::ptr::null_mut();
    let mut KeyInformation: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut ProtocolCharacteristics: PNDIS_PROTOCOL_CHARACTERISTICS = &raw mut (*Protocol).Chars;
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut ResultLength: ULONG = 0;
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();

    RegistryPathStr = ExAllocatePoolWithTag(
        PagedPool,
        (core::mem::size_of::<[u16; 53]>() as SIZE_T)
            + ((*ProtocolCharacteristics).__bindgen_anon_1.Name.Length as SIZE_T)
            + (core::mem::size_of_val(&LINKAGE_KEY) as SIZE_T),
        NDIS_TAG,
    ) as *mut WCHAR;
    if RegistryPathStr.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources.\n\0".as_ptr() as *const i8,
        );
        *Status = NDIS_STATUS_RESOURCES;
        return;
    }

    wcscpy(RegistryPathStr, (SERVICES_KEY).as_ptr());
    wcsncat(
        RegistryPathStr,
        (*ProtocolCharacteristics).__bindgen_anon_1.Name.Buffer as *mut WCHAR,
        ((*ProtocolCharacteristics).__bindgen_anon_1.Name.Length as usize) / core::mem::size_of::<WCHAR>(),
    );
    *RegistryPathStr.offset(
        (wcslen(SERVICES_KEY.as_ptr() as *const wchar_t) as usize
            + ((*ProtocolCharacteristics).__bindgen_anon_1.Name.Length as usize) / core::mem::size_of::<WCHAR>())
            as isize,
    ) = 0;
    wcscat(RegistryPathStr, (LINKAGE_KEY).as_ptr());

    RtlInitUnicodeString(&raw mut RegistryPath, RegistryPathStr);
    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Opening configuration key: %wZ\n\0".as_ptr() as *const i8,
        &raw mut RegistryPath,
    );

    // InitializeObjectAttributes expanded inline
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut RegistryPath;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    NtStatus = ZwOpenKey(
        &raw mut DriverKeyHandle,
        KEY_READ,
        &raw mut ObjectAttributes,
    );

    ExFreePoolWithTag((RegistryPathStr).cast(), NDIS_TAG);

    if NT_SUCCESS!(NtStatus) {
        NDIS_DbgPrint!(
            MAX_TRACE,
            b"Successfully opened the registry configuration\n\0".as_ptr() as *const i8,
        );

        RtlInitUnicodeString(&raw mut ValueName, RUSTCONV_WSTR_0D1A236C5C8A.as_ptr());

        NtStatus = ZwQueryValueKey(
            DriverKeyHandle,
            &raw mut ValueName,
            KeyValuePartialInformation,
            core::ptr::null_mut(),
            0,
            &raw mut ResultLength,
        );
        if NtStatus != STATUS_BUFFER_OVERFLOW
            && NtStatus != STATUS_BUFFER_TOO_SMALL
            && NtStatus != STATUS_SUCCESS
        {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Unable to query the Bind value for size\n\0".as_ptr() as *const i8,
            );
            ZwClose(DriverKeyHandle);
        } else {
            KeyInformation = ExAllocatePoolWithTag(
                PagedPool,
                (core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as SIZE_T)
                    + (ResultLength as SIZE_T),
                NDIS_TAG,
            ) as PKEY_VALUE_PARTIAL_INFORMATION;
            if KeyInformation.is_null() {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"Insufficient resources.\n\0".as_ptr() as *const i8,
                );
                ZwClose(DriverKeyHandle);
                NtStatus = STATUS_NO_MEMORY;
            } else {
                NtStatus = ZwQueryValueKey(
                    DriverKeyHandle,
                    &raw mut ValueName,
                    KeyValuePartialInformation,
                    KeyInformation as PVOID,
                    (core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG)
                        + ResultLength,
                    &raw mut ResultLength,
                );

                ZwClose(DriverKeyHandle);

                if !NT_SUCCESS!(NtStatus) {
                    NDIS_DbgPrint!(
                        MIN_TRACE,
                        b"Unable to query the Bind value\n\0".as_ptr() as *const i8,
                    );
                    ExFreePoolWithTag(KeyInformation as PVOID, NDIS_TAG);
                    KeyInformation = core::ptr::null_mut();
                }
            }
        }
    }

    if !NT_SUCCESS!(NtStatus) {
        NDIS_DbgPrint!(
            MID_TRACE,
            b"Performing global bind for protocol '%wZ'\n\0".as_ptr() as *const i8,
            &raw mut (*ProtocolCharacteristics).__bindgen_anon_1.Name,
        );

        CurrentEntry = AdapterListHead.Flink as PLIST_ENTRY;
    } else {
        NDIS_DbgPrint!(
            MID_TRACE,
            b"Performing standard bind for protocol '%wZ'\n\0".as_ptr() as *const i8,
            &raw mut (*ProtocolCharacteristics).__bindgen_anon_1.Name,
        );

        DataPtr = &raw mut (*KeyInformation).Data as *mut UCHAR as *mut WCHAR;
    }

    /* Assume success for now */
    *Status = NDIS_STATUS_SUCCESS;

    'bindloop: loop {
        /* BindContext is for tracking pending binding operations */
        let mut BindContext: *mut core::ffi::c_void =
            0 as *mut core::ffi::c_void;
        let mut DeviceName: NDIS_STRING = core::mem::zeroed();
        let mut RegistryPath: NDIS_STRING = core::mem::zeroed();
        let mut RegistryPathStr: *mut WCHAR = core::ptr::null_mut();
        let mut PathLength: ULONG = 0 as ULONG;
        let mut Adapter: PLOGICAL_ADAPTER = core::ptr::null_mut();

        if !KeyInformation.is_null() {
            /* Parse the REG_MULTI_SZ entry for device names */
            if *DataPtr == 0 {
                break 'bindloop;
            }

            RtlInitUnicodeString(&raw mut DeviceName, DataPtr);
        } else {
            /* Use the device name from the global adapter list */
            if CurrentEntry == &raw mut AdapterListHead {
                break 'bindloop;
            }

            Adapter = CONTAINING_RECORD!(CurrentEntry, LOGICAL_ADAPTER, ListEntry);

            DeviceName = (*Adapter).NdisMiniportBlock.MiniportName;
        }

        'next: {
            /* Make sure the adapter has started */
            if MiniLocateDevice(&raw mut DeviceName) == core::ptr::null_mut() {
                /* It wasn't in the global miniport list, so skip the bind entry */
                break 'next;
            }

            /* Make sure this device isn't already bound to this protocol */
            if LocateAdapterBindingByName(Protocol, &raw mut DeviceName) != core::ptr::null_mut() {
                /* It was already in this protocol's bound adapter list, so skip the bind entry */
                break 'next;
            }

            /*
             * RegistryPath should be:
             *     \Registry\Machine\System\CurrentControlSet\Services\Nic1\Parameters\Tcpip
             */
            PathLength = (core::mem::size_of::<[u16; 53]>() as ULONG)
                + (wcslen(
                    DeviceName.Buffer.offset(8 as isize) as *const wchar_t,
                ) as ULONG)
                    * (core::mem::size_of::<WCHAR>() as ULONG)
                + (core::mem::size_of::<[u16; 13]>() as ULONG)
                + ((*ProtocolCharacteristics).__bindgen_anon_1.Name.Length as ULONG)
                + (core::mem::size_of::<WCHAR>() as ULONG);

            RegistryPathStr = ExAllocatePool(PagedPool, PathLength as SIZE_T)
                as *mut WCHAR;
            if RegistryPathStr.is_null() {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"insufficient resources.\n\0".as_ptr() as *const i8,
                );
                *Status = NDIS_STATUS_RESOURCES;
                break 'bindloop;
            }

            wcscpy(RegistryPathStr, (SERVICES_KEY).as_ptr());
            wcscat(
                RegistryPathStr,
                DeviceName.Buffer.offset(8 as isize),
            );
            wcscat(RegistryPathStr, (PARAMETERS_KEY).as_ptr());
            wcsncat(
                RegistryPathStr,
                (*ProtocolCharacteristics).__bindgen_anon_1.Name.Buffer,
                ((*ProtocolCharacteristics).__bindgen_anon_1.Name.Length as usize) / core::mem::size_of::<WCHAR>(),
            );

            *RegistryPathStr.offset(
                ((PathLength as usize) / core::mem::size_of::<WCHAR>()) as isize
                    - 1 as isize,
            ) = 0 as WCHAR;

            RtlInitUnicodeString(&raw mut RegistryPath, RegistryPathStr);

            NDIS_DbgPrint!(
                MAX_TRACE,
                b"Calling protocol's BindAdapter handler with DeviceName %wZ and RegistryPath %wZ\n\0"
                    .as_ptr() as *const i8,
                &raw mut DeviceName,
                &raw mut RegistryPath,
            );

            {
                let mut BindHandler: BIND_HANDLER = (*ProtocolCharacteristics).__bindgen_anon_1.BindAdapterHandler;
                if BindHandler.is_some() {
                    (BindHandler.unwrap_unchecked())(
                        Status,
                        BindContext,
                        &raw mut DeviceName,
                        (&raw mut RegistryPath).cast(),
                        core::ptr::null_mut(),
                    );
                    NDIS_DbgPrint!(
                        MID_TRACE,
                        b"%wZ's BindAdapter handler returned 0x%x for %wZ\n\0".as_ptr() as *const i8,
                        &raw mut (*ProtocolCharacteristics).__bindgen_anon_1.Name,
                        *Status,
                        &raw mut DeviceName,
                    );
                } else {
                    NDIS_DbgPrint!(
                        MID_TRACE,
                        b"No protocol bind handler specified\n\0".as_ptr() as *const i8,
                    );
                }
            }
        }

        // next:
        if !KeyInformation.is_null() {
            /* Advance to the next adapter in the REG_MULTI_SZ */
            DataPtr = DataPtr.offset(
                ((DeviceName.Length as usize) / core::mem::size_of::<WCHAR>()
                    + 1 as usize) as isize,
            );
        } else {
            /* Advance to the next adapter in the global list */
            CurrentEntry = (*CurrentEntry).Flink as PLIST_ENTRY;
        }
    }

    if !KeyInformation.is_null() {
        ExFreePoolWithTag(KeyInformation as PVOID, NDIS_TAG);
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisRegisterProtocol(
    Status: PNDIS_STATUS,
    NdisProtocolHandle: PNDIS_HANDLE,
    ProtocolCharacteristics: PNDIS_PROTOCOL_CHARACTERISTICS,
    CharacteristicsLength: UINT,
) {
    let mut Protocol: PPROTOCOL_BINDING = core::ptr::null_mut();
    let mut NtStatus: NTSTATUS = 0;
    let mut MinSize: UINT = 0;
    let mut PnPEvent: PNET_PNP_EVENT = core::ptr::null_mut();

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    *NdisProtocolHandle = core::ptr::null_mut();

    /* first validate the PROTOCOL_CHARACTERISTICS */
    match (*ProtocolCharacteristics).__bindgen_anon_1.MajorNdisVersion {
        0x03 => {
            /* we don't really want to support ndis3 drivers - so we complain for now */
            NDIS_DbgPrint!(MID_TRACE, b"NDIS 3 protocol attempting to register\n\0".as_ptr() as *const i8);
            MinSize = core::mem::size_of::<NDIS30_PROTOCOL_CHARACTERISTICS>() as UINT;
        }
        0x04 => {
            MinSize = core::mem::size_of::<NDIS40_PROTOCOL_CHARACTERISTICS>() as UINT;
        }
        0x05 => {
            /* Identical for both NDIS50 and NDIS51 */
            MinSize = core::mem::size_of::<NDIS50_PROTOCOL_CHARACTERISTICS>() as UINT;
        }
        _ => {
            *Status = NDIS_STATUS_BAD_VERSION;
            NDIS_DbgPrint!(MIN_TRACE, b"Incorrect characteristics size\n\0".as_ptr() as *const i8);
            return;
        }
    }

    if CharacteristicsLength < MinSize {
        NDIS_DbgPrint!(MIN_TRACE, b"Bad protocol characteristics.\n\0".as_ptr() as *const i8);
        *Status = NDIS_STATUS_BAD_CHARACTERISTICS;
        return;
    }

    /* set up the protocol block */
    Protocol = ExAllocatePool(NonPagedPool, core::mem::size_of::<PROTOCOL_BINDING>() as SIZE_T) as PPROTOCOL_BINDING;
    if Protocol.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0".as_ptr() as *const i8);
        *Status = NDIS_STATUS_RESOURCES;
        return;
    }

    core::ptr::write_bytes(Protocol as *mut u8, 0, core::mem::size_of::<PROTOCOL_BINDING>());
    core::ptr::copy_nonoverlapping(
        ProtocolCharacteristics as *const u8,
        &raw mut (*Protocol).Chars as *mut u8,
        MinSize as usize,
    );

    NtStatus = RtlUpcaseUnicodeString(
        &raw mut (*Protocol).Chars.__bindgen_anon_1.Name,
        &raw mut (*ProtocolCharacteristics).__bindgen_anon_1.Name,
        TRUE as BOOLEAN,
    );
    if !(NtStatus >= 0 as NTSTATUS) {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0".as_ptr() as *const i8);
        ExFreePool(Protocol as PVOID);
        *Status = NDIS_STATUS_RESOURCES;
        return;
    }

    KeInitializeSpinLock(&raw mut (*Protocol).Lock);

    InitializeListHead(&raw mut (*Protocol).AdapterListHead);

    /* We must set this before the call to ndisBindMiniportsToProtocol because the protocol's
     * BindAdapter handler might need it */

    *NdisProtocolHandle = Protocol as PVOID;

    ndisBindMiniportsToProtocol(Status, Protocol);

    /* Should we only send this if ndisBindMiniportsToProtocol succeeds? */
    PnPEvent = ProSetupPnPEvent(NetEventBindsComplete, core::ptr::null_mut(), 0 as ULONG);
    if !PnPEvent.is_null() {
        if let Some(handler) = (*Protocol).Chars.__bindgen_anon_1.PnPEventHandler {
            /* We call this with a NULL binding context because it affects all bindings */
            NtStatus = handler(core::ptr::null_mut(), PnPEvent) as NTSTATUS;

            if NtStatus == 0x103 as NTSTATUS {
                RtlAssert(
                    b"NtStatus != NDIS_STATUS_PENDING\0".as_ptr() as PVOID,
                    b"drivers/network/ndis/ndis/protocol.c\0".as_ptr() as PVOID,
                    1219 as ULONG,
                    core::ptr::null_mut(),
                );
            }

            /* FIXME: We don't support this yet */
        }

        ExFreePool(PnPEvent as PVOID);
    }

    if *Status == NDIS_STATUS_SUCCESS {
        ExInterlockedInsertTailList(
            &raw mut ProtocolListHead,
            &raw mut (*Protocol).ListEntry,
            &raw mut ProtocolListLock,
        );
    } else {
        NDIS_DbgPrint!(MIN_TRACE, b"Binding failed (%x)\n\0".as_ptr() as *const i8, *Status);
        ExFreePool(Protocol as PVOID);
        *NdisProtocolHandle = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisRequest(Status: PNDIS_STATUS, NdisBindingHandle: NDIS_HANDLE, NdisRequest: PNDIS_REQUEST) {
    *Status = ProRequest(NdisBindingHandle, NdisRequest);
}
#[no_mangle]
pub unsafe extern "system" fn NdisReset(Status: *mut NDIS_STATUS, NdisBindingHandle: NDIS_HANDLE) {
    *Status = ProReset(NdisBindingHandle);
}
#[no_mangle]
pub unsafe extern "system" fn NdisSend(
    Status: *mut NDIS_STATUS,
    NdisBindingHandle: NDIS_HANDLE,
    Packet: PNDIS_PACKET,
) {
    *Status = ProSend(NdisBindingHandle, Packet);
}

#[no_mangle]
pub unsafe extern "system" fn NdisSendPackets(
    NdisBindingHandle: NDIS_HANDLE,
    PacketArray: PPNDIS_PACKET,
    NumberOfPackets: UINT,
) {
    ProSendPackets(NdisBindingHandle, PacketArray, NumberOfPackets);
}
#[no_mangle]
pub unsafe extern "system" fn NdisTransferData(
    Status: PNDIS_STATUS,
    NdisBindingHandle: NDIS_HANDLE,
    MacReceiveContext: NDIS_HANDLE,
    ByteOffset: UINT,
    BytesToTransfer: UINT,
    Packet: PNDIS_PACKET,
    BytesTransferred: PUINT,
) {
    *Status = ProTransferData(
        NdisBindingHandle,
        MacReceiveContext,
        ByteOffset,
        BytesToTransfer,
        Packet,
        BytesTransferred,
    );
}

#[no_mangle]
pub unsafe extern "system" fn NdisReEnumerateProtocolBindings(NdisProtocolHandle: NDIS_HANDLE) {
    let mut NdisStatus: NDIS_STATUS = 0;
    ndisBindMiniportsToProtocol(core::ptr::addr_of_mut!(NdisStatus), NdisProtocolHandle as PPROTOCOL_BINDING);
}
#[no_mangle]
pub unsafe extern "system" fn NdisGetDriverHandle(
    NdisBindingHandle: PNDIS_HANDLE,
    NdisDriverHandle: PNDIS_HANDLE,
) {
    let Binding: PADAPTER_BINDING = NdisBindingHandle as PADAPTER_BINDING;

    if Binding.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Bad binding handle\n\0".as_ptr() as *const i8);
        *NdisDriverHandle = core::ptr::null_mut();
        return;
    }

    *NdisDriverHandle = (*(*Binding).Adapter).NdisMiniportBlock.DriverHandle as PVOID;
}
