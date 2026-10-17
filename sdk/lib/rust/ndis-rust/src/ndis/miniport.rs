use crate::ndis::buffer::NdisQueryBuffer;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_25B56DF68A17: [u16; 10] = [0x0042u16, 0x0075, 0x0073, 0x004E, 0x0075, 0x006D, 0x0062, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_853B46C50805: [u16; 7] = [0x0045u16, 0x0078, 0x0070, 0x006F, 0x0072, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_E56EFEA51CCC: [u16; 11] = [
    0x0053u16, 0x006C, 0x006F, 0x0074, 0x004E, 0x0075, 0x006D, 0x0062, 0x0065, 0x0072,
    0x0000,
];
pub static RUSTCONV_WSTR_FB83D8E92731: [u16; 8] = [0x0042u16, 0x0075, 0x0073, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000];

use crate::prelude::*;
use ntkrnl_sys::ddk::{CM_RESOURCE_LIST, ExAllocatePool, ExFreePool, _KWAIT_REASON_Executive, IoAllocateDriverObjectExtension, IoAllocateWorkItem, IoAttachDeviceToDeviceStack, IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IoDetachDevice, IoFreeWorkItem, IoGetDeviceProperty, IoGetDriverObjectExtension, IoOpenDeviceRegistryKey, IoQueueWorkItem, IoRegisterDeviceInterface, IoRegisterShutdownNotification, IoSetDeviceInterfaceState, IoUnregisterShutdownNotification, KBUGCHECK_CALLBACK_RECORD, KEVENT, KTIMER, KeCancelTimer, KeDeregisterBugCheckCallback, KeInitializeDpc, KeInitializeEvent, KeInitializeTimer, KeRegisterBugCheckCallback, KeSetTimer, KeSetTimerEx, KeSynchronizeExecution, KeWaitForSingleObject, LARGE_INTEGER, LIST_ENTRY, _EVENT_TYPE_NotificationEvent, PCI_SLOT_NUMBER, PCM_RESOURCE_LIST, PCSTR, PDEVICE_OBJECT, PDRIVER_DISPATCH, PDRIVER_OBJECT, PDRIVER_UNLOAD, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP, PKEVENT, PKSYNCHRONIZE_ROUTINE, PSCATTER_GATHER_LIST, PUNICODE_STRING, RTL_QUERY_REGISTRY_TABLE, RtlCompareMemory, RtlCompareUnicodeString, RtlCopyUnicodeString, RtlFreeUnicodeString, RtlInitUnicodeString, RtlQueryRegistryValues, ZwClose};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut GUID_DEVINTERFACE_NET: GUID = _GUID {
    Data1: 0xcac88484 as ::core::ffi::c_uint,
    Data2: 0x7515 as ::core::ffi::c_ushort,
    Data3: 0x4c03 as ::core::ffi::c_ushort,
    Data4: [
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x61 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut MediaArray: [NDIS_MEDIUM; 15] = [
    NdisMedium802_3,
    NdisMedium802_5,
    NdisMediumFddi,
    NdisMediumWan,
    NdisMediumLocalTalk,
    NdisMediumDix,
    NdisMediumArcnetRaw,
    NdisMediumArcnet878_2,
    NdisMediumAtm,
    NdisMediumWirelessWan,
    NdisMediumIrda,
    NdisMediumBpc,
    NdisMediumCoWan,
    NdisMedium1394,
    NdisMediumMax,
];

#[no_mangle]
pub static mut MiniportListHead: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

#[no_mangle]
pub static mut MiniportListLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut AdapterListHead: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

#[no_mangle]
pub static mut AdapterListLock: KSPIN_LOCK = 0;

pub const BREAK_ON_MINIPORT_INIT: i32 = 0;

pub const ROUTE_DATA_SIZE: u32 = 256;

pub const MEDIA_ARRAY_SIZE: usize = 15;

#[no_mangle]
pub unsafe extern "C" fn MiniDisplayPacket(mut Packet: PNDIS_PACKET, mut Reason: PCSTR) {
    let mut i: ULONG = 0;
    let mut Length: ULONG = 0;
    let mut Buffer: [UCHAR; 64] = [0; 64];
    if (DebugTraceLevel & DEBUG_PACKET as ULONG) > 0 as ULONG {
        Length = CopyPacketToBuffer(
            core::ptr::addr_of_mut!(Buffer) as PUCHAR,
            Packet,
            0 as UINT,
            64 as UINT,
        ) as ULONG;

        DbgPrint(
            b"*** %s PACKET START (%p) ***\n\0".as_ptr() as *const u8 as PCSTR,
            Reason,
            Packet,
        );

        i = 0;
        while i < Length {
            if i % 16 as ULONG == 0 as ULONG {
                DbgPrint(b"\n%04X \0".as_ptr() as *const u8 as PCSTR, i);
            }
            DbgPrint(
                b"%02X \0".as_ptr() as *const u8 as PCSTR,
                Buffer[i as usize] as ::core::ffi::c_int,
            );
            i = i.wrapping_add(1);
        }

        DbgPrint(
            b"\n*** %s PACKET STOP ***\n\0".as_ptr() as *const u8 as PCSTR,
            Reason,
        );
    }
}

unsafe extern "C" fn MiniDisplayPacket2(
    mut HeaderBuffer: PVOID,
    mut HeaderBufferSize: UINT,
    mut LookaheadBuffer: PVOID,
    mut LookaheadBufferSize: UINT,
) {
    if (DebugTraceLevel & DEBUG_PACKET as ULONG) > 0 as ULONG {
        let mut i: ULONG = 0;
        let mut Length: ULONG = 0;
        let mut p: PUCHAR = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
        DbgPrint(b"*** RECEIVE PACKET START ***\n\0" as *const u8 as PCSTR);
        DbgPrint(b"HEADER:\0" as *const u8 as PCSTR);
        p = HeaderBuffer as PUCHAR;
        i = 0 as ULONG;
        while i < HeaderBufferSize as ULONG {
            if i.wrapping_rem(16 as ULONG) == 0 as ULONG {
                DbgPrint(b"\n%04X \0" as *const u8 as PCSTR, i);
            }
            let fresh20 = p;
            p = p.offset(1);
            DbgPrint(
                b"%02X \0" as *const u8 as PCSTR,
                *fresh20 as ::core::ffi::c_int,
            );
            i = i.wrapping_add(1);
        }
        DbgPrint(b"\nFRAME:\0" as *const u8 as PCSTR);
        p = LookaheadBuffer as PUCHAR;
        Length = (if LookaheadBufferSize < 64 as UINT {
            LookaheadBufferSize
        } else {
            64 as UINT
        }) as ULONG;
        i = 0 as ULONG;
        while i < Length {
            if i.wrapping_rem(16 as ULONG) == 0 as ULONG {
                DbgPrint(b"\n%04X \0" as *const u8 as PCSTR, i);
            }
            let fresh21 = p;
            p = p.offset(1);
            DbgPrint(
                b"%02X \0" as *const u8 as PCSTR,
                *fresh21 as ::core::ffi::c_int,
            );
            i = i.wrapping_add(1);
        }
        DbgPrint(b"\n*** RECEIVE PACKET STOP ***\n\0" as *const u8 as PCSTR);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MiniGetFirstWorkItem(
    Adapter: PLOGICAL_ADAPTER,
    Type: NDIS_WORK_ITEM_TYPE,
) -> PNDIS_MINIPORT_WORK_ITEM {
    let mut CurrentEntry: PNDIS_MINIPORT_WORK_ITEM = (*Adapter).WorkQueueHead;

    while !CurrentEntry.is_null() {
        if ((*CurrentEntry).WorkItemType as ::core::ffi::c_uint) == (Type as ::core::ffi::c_uint)
            || ((Type as ::core::ffi::c_uint) == (NdisMaxWorkItems as ::core::ffi::c_uint))
        {
            return CurrentEntry;
        }

        CurrentEntry = (*CurrentEntry).Link.Next as PNDIS_MINIPORT_WORK_ITEM;
    }

    return ::core::ptr::null_mut::<_NDIS_MINIPORT_WORK_ITEM>();
}
#[no_mangle]
pub unsafe extern "C" fn MiniIsBusy(
    Adapter: PLOGICAL_ADAPTER,
    Type: NDIS_WORK_ITEM_TYPE,
) -> BOOLEAN {
    let mut Busy: BOOLEAN = FALSE;
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock), core::ptr::addr_of_mut!(OldIrql));

    if !MiniGetFirstWorkItem(Adapter, Type).is_null() {
        Busy = TRUE;
    } else if Type == NdisWorkItemRequest && !(*Adapter).NdisMiniportBlock.PendingRequest.is_null() {
        Busy = TRUE;
    } else if Type == NdisWorkItemSend && !(*Adapter).NdisMiniportBlock.FirstPendingPacket.is_null() {
        Busy = TRUE;
    } else if Type == NdisWorkItemResetRequested
        && (*Adapter).NdisMiniportBlock.ResetStatus == NDIS_STATUS_PENDING
    {
        Busy = TRUE;
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock), OldIrql);

    return Busy;
}
#[no_mangle]
pub unsafe extern "C" fn MiniIndicateData(
    Adapter: PLOGICAL_ADAPTER,
    MacReceiveContext: NDIS_HANDLE,
    HeaderBuffer: PVOID,
    HeaderBufferSize: UINT,
    LookaheadBuffer: PVOID,
    LookaheadBufferSize: UINT,
    PacketSize: UINT,
) {
    let mut OldIrql: KIRQL = 0;
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut AdapterBinding: PADAPTER_BINDING = core::ptr::null_mut();

    NDIS_DbgPrint!(
        DEBUG_MINIPORT,
        b"Called. Adapter (0x%X)  HeaderBuffer (0x%X)  HeaderBufferSize (0x%X)  LookaheadBuffer (0x%X)  LookaheadBufferSize (0x%X).\n\0".as_ptr() as *const i8,
        Adapter,
        HeaderBuffer,
        HeaderBufferSize,
        LookaheadBuffer,
        LookaheadBufferSize,
    );

    MiniDisplayPacket2(
        HeaderBuffer,
        HeaderBufferSize,
        LookaheadBuffer,
        LookaheadBufferSize,
    );

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"acquiring miniport block lock\n\0".as_ptr() as *const i8,
    );
    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock),
        core::ptr::addr_of_mut!(OldIrql),
    );
    {
        CurrentEntry = (*Adapter).ProtocolListHead.Flink as PLIST_ENTRY;
        NDIS_DbgPrint!(
            DEBUG_MINIPORT,
            b"CurrentEntry = %x\n\0".as_ptr() as *const i8,
            CurrentEntry,
        );

        if CurrentEntry == core::ptr::addr_of_mut!((*Adapter).ProtocolListHead) {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"WARNING: No upper protocol layer.\n\0".as_ptr() as *const i8,
            );
        }

        while CurrentEntry != core::ptr::addr_of_mut!((*Adapter).ProtocolListHead) {
            // CONTAINING_RECORD(CurrentEntry, COMMENT_TOKEN_ECHO____, AdapterListEntry), expanded inline
            AdapterBinding = ((CurrentEntry as ULONG_PTR).wrapping_sub(
                core::mem::offset_of!(ADAPTER_BINDING, AdapterListEntry) as ULONG_PTR,
            )) as *mut ADAPTER_BINDING as PADAPTER_BINDING;
            NDIS_DbgPrint!(
                DEBUG_MINIPORT,
                b"AdapterBinding = %x\n\0".as_ptr() as *const i8,
                AdapterBinding,
            );

            NDIS_DbgPrint!(
                MID_TRACE,
                b"XXX (%x) %x %x %x %x %x %x %x XXX\n\0".as_ptr() as *const i8,
                (*(*AdapterBinding).ProtocolBinding).Chars.__bindgen_anon_1.__bindgen_anon_4.ReceiveHandler,
                (*AdapterBinding).NdisOpenBlock.ProtocolBindingContext,
                MacReceiveContext,
                HeaderBuffer,
                HeaderBufferSize,
                LookaheadBuffer,
                LookaheadBufferSize,
                PacketSize,
            );

            /* call the receive handler */
            (*(*AdapterBinding).ProtocolBinding)
                .Chars
                .__bindgen_anon_1.__bindgen_anon_4.ReceiveHandler
                .unwrap_unchecked()(
                (*AdapterBinding).NdisOpenBlock.ProtocolBindingContext,
                MacReceiveContext,
                HeaderBuffer,
                HeaderBufferSize,
                LookaheadBuffer,
                LookaheadBufferSize,
                PacketSize,
            );

            CurrentEntry = (*CurrentEntry).Flink as PLIST_ENTRY;
        }
    }
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock),
        OldIrql,
    );

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Leaving.\n\0".as_ptr() as *const i8,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisReturnPackets(PacketsToReturn: *mut PNDIS_PACKET, NumberOfPackets: UINT) {
    let mut i: UINT;
    let mut Adapter: PLOGICAL_ADAPTER;
    let mut OldIrql: KIRQL = core::mem::zeroed();

    NDIS_DbgPrint!(
        MID_TRACE,
        b"Returning %d packets\n\0".as_ptr() as PCSTR,
        NumberOfPackets,
    );

    i = 0;
    while i < NumberOfPackets {
        let WrapperReservedPtr = core::ptr::addr_of_mut!(
            (**PacketsToReturn.offset(i as isize)).__bindgen_anon_1.__bindgen_anon_1.WrapperReserved[0]
        );
        *WrapperReservedPtr = (*WrapperReservedPtr).wrapping_sub(1);
        if ((**PacketsToReturn.offset(i as isize)).__bindgen_anon_1.__bindgen_anon_1.WrapperReserved[0] as i32)
            == 0 as i32
        {
            Adapter = (**PacketsToReturn.offset(i as isize)).Reserved
                [1 as usize] as PVOID
                as PLOGICAL_ADAPTER;

            NDIS_DbgPrint!(
                MAX_TRACE,
                b"Freeing packet %d (adapter = 0x%p)\n\0".as_ptr() as PCSTR,
                i,
                Adapter,
            );

            KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(OldIrql));
            (*(*Adapter).NdisMiniportBlock.DriverHandle)
                .MiniportCharacteristics
                .ReturnPacketHandler
                .unwrap_unchecked()(
                (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                *PacketsToReturn.offset(i as isize),
            );
            KeLowerIrql(OldIrql);
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "system" fn MiniIndicateReceivePacket(MiniportAdapterHandle: NDIS_HANDLE, PacketArray: PPNDIS_PACKET, NumberOfPackets: UINT) {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut AdapterBinding: PADAPTER_BINDING = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;
    let mut i: UINT = 0;

    KeAcquireSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, &raw mut OldIrql);

    CurrentEntry = (*Adapter).ProtocolListHead.Flink;

    while CurrentEntry != &raw mut (*Adapter).ProtocolListHead {
        AdapterBinding = CONTAINING_RECORD!(CurrentEntry, ADAPTER_BINDING, AdapterListEntry);

        i = 0 as UINT;
        while i < NumberOfPackets {
            let pkt: PNDIS_PACKET = *PacketArray.offset(i as isize);
            /* Store the indicating miniport in the packet */
            (*pkt).Reserved[1 as usize] = Adapter as ULONG_PTR;

            if (*(*AdapterBinding).ProtocolBinding).Chars.__bindgen_anon_1.ReceivePacketHandler.is_some()
                && NDIS_GET_PACKET_STATUS!(pkt) != NDIS_STATUS_RESOURCES as NDIS_STATUS
            {
                NDIS_DbgPrint!(
                    MID_TRACE,
                    b"Indicating packet to protocol's ReceivePacket handler\n\0".as_ptr() as PCSTR,
                );
                let ret = ((*(*AdapterBinding).ProtocolBinding)
                    .Chars
                    .__bindgen_anon_1.ReceivePacketHandler
                    .unwrap_unchecked())(
                    (*AdapterBinding).NdisOpenBlock.ProtocolBindingContext,
                    pkt,
                );
                (*pkt).__bindgen_anon_1.__bindgen_anon_1.WrapperReserved[0 as usize] =
                    (*pkt).__bindgen_anon_1.__bindgen_anon_1.WrapperReserved[0 as usize]
                        .wrapping_add(ret as UCHAR);
                NDIS_DbgPrint!(
                    MID_TRACE,
                    b"Protocol is holding %d references to the packet\n\0".as_ptr() as PCSTR,
                    (*pkt).__bindgen_anon_1.__bindgen_anon_1.WrapperReserved[0 as usize] as ::core::ffi::c_int,
                );
            } else {
                let mut FirstBufferLength: UINT = 0;
                let mut TotalBufferLength: UINT = 0;
                let mut LookAheadSize: UINT = 0;
                let mut HeaderSize: UINT = 0;
                let mut NdisBuffer: PNDIS_BUFFER = core::ptr::null_mut();
                let mut NdisBufferVA: PVOID = core::ptr::null_mut();
                let mut LookAheadBuffer: PVOID = core::ptr::null_mut();

                NdisGetFirstBufferFromPacket(
                    pkt,
                    &raw mut NdisBuffer,
                    &raw mut NdisBufferVA,
                    &raw mut FirstBufferLength,
                    &raw mut TotalBufferLength,
                );

                HeaderSize = NDIS_GET_PACKET_HEADER_SIZE!(pkt);

                LookAheadSize = TotalBufferLength.wrapping_sub(HeaderSize);

                LookAheadBuffer = ExAllocatePool(
                    NonPagedPool,
                    LookAheadSize as SIZE_T,
                );
                if LookAheadBuffer.is_null() {
                    NDIS_DbgPrint!(
                        MIN_TRACE,
                        b"Failed to allocate lookahead buffer!\n\0".as_ptr() as PCSTR,
                    );
                    KeReleaseSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, OldIrql);
                    return;
                }

                CopyBufferChainToBuffer(
                    LookAheadBuffer as PUCHAR,
                    NdisBuffer,
                    HeaderSize,
                    LookAheadSize,
                );

                NDIS_DbgPrint!(
                    MID_TRACE,
                    b"Indicating packet to protocol's legacy Receive handler\n\0".as_ptr() as PCSTR,
                );
                ((*(*AdapterBinding).ProtocolBinding)
                    .Chars
                    .__bindgen_anon_1.__bindgen_anon_4.ReceiveHandler
                    .unwrap_unchecked())(
                    (*AdapterBinding).NdisOpenBlock.ProtocolBindingContext,
                    (*AdapterBinding).NdisOpenBlock.MacHandle as NDIS_HANDLE,
                    NdisBufferVA,
                    HeaderSize,
                    LookAheadBuffer,
                    LookAheadSize,
                    TotalBufferLength.wrapping_sub(HeaderSize),
                );

                ExFreePool(LookAheadBuffer);
            }
            i = i.wrapping_add(1);
        }

        CurrentEntry = (*CurrentEntry).Flink;
    }

    /* Loop the packet array to get everything
     * set up for return the packets to the miniport */
    i = 0 as UINT;
    while i < NumberOfPackets {
        let pkt: PNDIS_PACKET = *PacketArray.offset(i as isize);
        /* First, check the initial packet status */
        if NDIS_GET_PACKET_STATUS!(pkt) == NDIS_STATUS_RESOURCES as NDIS_STATUS {
            /* The miniport driver gets it back immediately so nothing to do here */
            NDIS_DbgPrint!(
                MID_TRACE,
                b"Miniport needs the packet back immediately\n\0".as_ptr() as PCSTR,
            );
            i = i.wrapping_add(1);
            continue;
        }

        /* Different behavior depending on whether it's serialized or not */
        if ((*Adapter).NdisMiniportBlock.Flags & (NDIS_ATTRIBUTE_DESERIALIZE as ULONG)) != 0 {
            /* We need to check the reference count */
            if ((*pkt).__bindgen_anon_1.__bindgen_anon_1.WrapperReserved[0 as usize] as ::core::ffi::c_int)
                == 0
            {
                /* NOTE: Unlike serialized miniports, this is REQUIRED to be called for each
                 * packet received that can be reused immediately, it is not implied! */
                ((*(*Adapter).NdisMiniportBlock.DriverHandle)
                    .MiniportCharacteristics
                    .ReturnPacketHandler
                    .unwrap_unchecked())(
                    (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                    pkt,
                );
                NDIS_DbgPrint!(
                    MID_TRACE,
                    b"Packet has been returned to miniport (Deserialized)\n\0".as_ptr() as PCSTR,
                );
            } else {
                /* Packet will be returned by the protocol's call to NdisReturnPackets */
                NDIS_DbgPrint!(
                    MID_TRACE,
                    b"Packet will be returned to miniport later (Deserialized)\n\0".as_ptr()
                        as PCSTR,
                );
            }
        } else {
            /* Check the reference count */
            if ((*pkt).__bindgen_anon_1.__bindgen_anon_1.WrapperReserved[0 as usize] as ::core::ffi::c_int)
                == 0
            {
                /* NDIS_STATUS_SUCCESS means the miniport can have the packet back immediately */
                NDIS_SET_PACKET_STATUS!(pkt, NDIS_STATUS_SUCCESS as NDIS_STATUS);

                NDIS_DbgPrint!(
                    MID_TRACE,
                    b"Packet has been returned to miniport (Serialized)\n\0".as_ptr() as PCSTR,
                );
            } else {
                /* NDIS_STATUS_PENDING means the miniport needs to wait for MiniportReturnPacket */
                NDIS_SET_PACKET_STATUS!(pkt, NDIS_STATUS_PENDING as NDIS_STATUS);

                NDIS_DbgPrint!(
                    MID_TRACE,
                    b"Packet will be returned to miniport later (Serialized)\n\0".as_ptr()
                        as PCSTR,
                );
            }
        }
        i = i.wrapping_add(1);
    }

    KeReleaseSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, OldIrql);
}
#[no_mangle]
pub unsafe extern "system" fn MiniResetComplete(MiniportAdapterHandle: NDIS_HANDLE, Status: NDIS_STATUS, AddressingReset: BOOLEAN) {
    let Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut CurrentEntry: PLIST_ENTRY;
    let mut AdapterBinding: PADAPTER_BINDING;
    let mut OldIrql: KIRQL = 0;

    if AddressingReset != 0 {
        MiniDoAddressingReset(Adapter);
    }

    NdisMIndicateStatus(
        Adapter as NDIS_HANDLE,
        NDIS_STATUS_RESET_END as NDIS_STATUS,
        core::ptr::null_mut(),
        0 as UINT,
    );
    NdisMIndicateStatusComplete(Adapter as NDIS_HANDLE);

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock), core::ptr::addr_of_mut!(OldIrql));

    if (*Adapter).NdisMiniportBlock.ResetStatus != NDIS_STATUS_PENDING as NDIS_STATUS {
        KeBugCheckEx(
            BUGCODE_ID_DRIVER as ULONG,
            MiniportAdapterHandle as ULONG_PTR,
            Status as ULONG_PTR,
            AddressingReset as ULONG_PTR,
            0 as ULONG_PTR,
        );
    }

    (*Adapter).NdisMiniportBlock.ResetStatus = Status;

    CurrentEntry = (*Adapter).ProtocolListHead.Flink;

    while CurrentEntry != core::ptr::addr_of_mut!((*Adapter).ProtocolListHead) {
        AdapterBinding = ((CurrentEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(ADAPTER_BINDING, AdapterListEntry) as ULONG_PTR,
        )) as PADAPTER_BINDING;

        (*(*AdapterBinding).ProtocolBinding)
            .Chars
            .__bindgen_anon_1.ResetCompleteHandler
            .unwrap_unchecked()(
            (*AdapterBinding).NdisOpenBlock.ProtocolBindingContext,
            Status,
        );

        CurrentEntry = (*CurrentEntry).Flink;
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock), OldIrql);
}
#[no_mangle]
pub unsafe extern "system" fn MiniRequestComplete(MiniportAdapterHandle: NDIS_HANDLE, Status: NDIS_STATUS) {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut Request: PNDIS_REQUEST = core::ptr::null_mut();
    let mut MacBlock: PNDIS_REQUEST_MAC_BLOCK = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;

    NDIS_DbgPrint!(DEBUG_MINIPORT, ("Called.\n"));

    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(OldIrql));

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock));
    Request = (*Adapter).NdisMiniportBlock.PendingRequest;
    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock));

    MacBlock = core::ptr::addr_of_mut!((*Request).MacReserved) as PNDIS_REQUEST_MAC_BLOCK;

    /* We may or may not be doing this request on behalf of an adapter binding */
    if !(*MacBlock).Binding.is_null() {
        /* We are, so invoke its request complete handler */
        if let Some(handler) = (*(*MacBlock).Binding).RequestCompleteHandler {
            handler(
                (*(*MacBlock).Binding).ProtocolBindingContext,
                Request,
                Status,
            );
        }
    } else {
        /* We are doing this internally, so we'll signal this event we've stashed in the MacBlock */
        if (*MacBlock).Unknown1.is_null() {
            RtlAssert(
                b"MacBlock->Unknown1 != NULL\0" as *const u8 as *const core::ffi::c_char as PVOID,
                b"miniport.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
                515 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
        if !(*MacBlock).Unknown3.is_null() {
            RtlAssert(
                b"MacBlock->Unknown3 == NULL\0" as *const u8 as *const core::ffi::c_char as PVOID,
                b"miniport.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
                516 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
        (*MacBlock).Unknown3 = UlongToPtr!(Status);
        KeSetEvent(((*MacBlock).Unknown1).cast(), (IO_NO_INCREMENT) as KPRIORITY, FALSE);
    }

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock));
    (*Adapter).NdisMiniportBlock.PendingRequest = core::ptr::null_mut();
    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock));
    KeLowerIrql(OldIrql);

    MiniWorkItemComplete(Adapter, NdisWorkItemRequest);
}
#[no_mangle]
pub unsafe extern "system" fn MiniSendComplete(MiniportAdapterHandle: NDIS_HANDLE, Packet: PNDIS_PACKET, Status: NDIS_STATUS) {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut AdapterBinding: PADAPTER_BINDING;
    let mut OldIrql: KIRQL = core::mem::zeroed();
    let mut SGList: PSCATTER_GATHER_LIST;

    NDIS_DbgPrint!(DEBUG_MINIPORT, b"Called.\n\0".as_ptr() as PCSTR);

    AdapterBinding = (*Packet).Reserved[1 as usize] as PADAPTER_BINDING;

    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(OldIrql));

    if (*Adapter).NdisMiniportBlock.ScatterGatherListSize != 0 {
        NDIS_DbgPrint!(MAX_TRACE, b"Freeing Scatter/Gather list\n\0".as_ptr() as PCSTR);

        SGList = NDIS_PER_PACKET_INFO_FROM_PACKET!(Packet, ScatterGatherListPacketInfo) as PSCATTER_GATHER_LIST;

        (*(*(*Adapter).NdisMiniportBlock.SystemAdapterObject).DmaOperations)
            .PutScatterGatherList
            .unwrap_unchecked()(
            (*Adapter).NdisMiniportBlock.SystemAdapterObject,
            SGList,
            TRUE,
        );

        NDIS_PER_PACKET_INFO_FROM_PACKET!(Packet, ScatterGatherListPacketInfo) = core::ptr::null_mut();
    }

    (*(*AdapterBinding).ProtocolBinding)
        .Chars
        
        
        .__bindgen_anon_1.__bindgen_anon_2.SendCompleteHandler
        .unwrap_unchecked()(
        (*AdapterBinding).NdisOpenBlock.ProtocolBindingContext,
        Packet,
        Status,
    );

    KeLowerIrql(OldIrql);

    MiniWorkItemComplete(Adapter, NdisWorkItemSend);
}

#[no_mangle]
pub unsafe extern "system" fn MiniSendResourcesAvailable(MiniportAdapterHandle: NDIS_HANDLE) {
    /* Run the work if anything is waiting */
    MiniWorkItemComplete(MiniportAdapterHandle as PLOGICAL_ADAPTER, NdisWorkItemSend);
}
#[no_mangle]
pub unsafe extern "system" fn MiniTransferDataComplete(MiniportAdapterHandle: NDIS_HANDLE, Packet: PNDIS_PACKET, Status: NDIS_STATUS, BytesTransferred: UINT) {
    let mut AdapterBinding: PADAPTER_BINDING = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;

    NDIS_DbgPrint!(DEBUG_MINIPORT, b"Called.\n\0".as_ptr() as *const i8);

    AdapterBinding = (*Packet).Reserved[1 as usize] as PADAPTER_BINDING;

    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(OldIrql));
    (*(*AdapterBinding).ProtocolBinding)
        .Chars
        .__bindgen_anon_1.__bindgen_anon_3.TransferDataCompleteHandler
        .unwrap_unchecked()(
        (*AdapterBinding).NdisOpenBlock.ProtocolBindingContext,
        Packet,
        Status,
        BytesTransferred,
    );
    KeLowerIrql(OldIrql);
}
#[no_mangle]
pub unsafe extern "C" fn MiniAdapterHasAddress(
    Adapter: PLOGICAL_ADAPTER,
    Packet: PNDIS_PACKET,
) -> BOOLEAN {
    let mut Length: UINT = 0;
    let mut PacketAddress: PUCHAR = core::ptr::null_mut();
    let mut AdapterAddress: PUCHAR = core::ptr::null_mut();
    let mut NdisBuffer: PNDIS_BUFFER = core::ptr::null_mut();
    let mut BufferLength: UINT = 0;

    NDIS_DbgPrint!(DEBUG_MINIPORT, b"Called.\n\0".as_ptr() as PCSTR);

    if Adapter.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Adapter object was null\n\0".as_ptr() as PCSTR);
        return FALSE as BOOLEAN;
    }

    if Packet.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Packet was null\n\0".as_ptr() as PCSTR);
        return FALSE as BOOLEAN;
    }

    NdisQueryPacket(
        Packet,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(NdisBuffer),
        core::ptr::null_mut(),
    );

    if NdisBuffer.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Packet contains no buffers.\n\0".as_ptr() as PCSTR);
        return FALSE as BOOLEAN;
    }

    NdisQueryBuffer(
        NdisBuffer,
        (core::ptr::addr_of_mut!(PacketAddress) as PVOID).cast(),
        core::ptr::addr_of_mut!(BufferLength),
    );

    /* FIXME: Should handle fragmented packets */

    match (*Adapter).NdisMiniportBlock.MediaType {
        NdisMedium802_3 => {
            Length = ETH_LENGTH_OF_ADDRESS;
            /* Destination address is the first field */
        }
        _ => {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Adapter has unsupported media type (0x%X).\n\0".as_ptr() as PCSTR,
                (*Adapter).NdisMiniportBlock.MediaType as UINT,
            );
            return FALSE as BOOLEAN;
        }
    }

    if BufferLength < Length {
        NDIS_DbgPrint!(MIN_TRACE, b"Buffer is too small.\n\0".as_ptr() as PCSTR);
        return FALSE as BOOLEAN;
    }

    AdapterAddress = core::ptr::addr_of_mut!((*Adapter).Address) as PUCHAR;
    NDIS_DbgPrint!(
        MAX_TRACE,
        b"packet address: %x:%x:%x:%x:%x:%x adapter address: %x:%x:%x:%x:%x:%x\n\0".as_ptr() as PCSTR,
        *PacketAddress as UINT,
        *PacketAddress.add(1) as UINT,
        *PacketAddress.add(2) as UINT,
        *PacketAddress.add(3) as UINT,
        *PacketAddress.add(4) as UINT,
        *PacketAddress.add(5) as UINT,
        *AdapterAddress as UINT,
        *AdapterAddress.add(1) as UINT,
        *AdapterAddress.add(2) as UINT,
        *AdapterAddress.add(3) as UINT,
        *AdapterAddress.add(4) as UINT,
        *AdapterAddress.add(5) as UINT,
    );

    return (RtlCompareMemory(
        PacketAddress as *const core::ffi::c_void,
        AdapterAddress as *const core::ffi::c_void,
        Length as SIZE_T,
    ) == Length as SIZE_T) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "C" fn MiniLocateDevice(AdapterName: PNDIS_STRING) -> PLOGICAL_ADAPTER {
    let mut OldIrql: KIRQL = 0;
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut Adapter: PLOGICAL_ADAPTER = core::ptr::null_mut();

    NDIS_DbgPrint!(DEBUG_MINIPORT, b"Called.\n\0".as_ptr() as PCSTR);

    if IsListEmpty(core::ptr::addr_of_mut!(AdapterListHead)) != 0 {
        NDIS_DbgPrint!(MIN_TRACE, b"No registered miniports for protocol to bind to\n\0".as_ptr() as PCSTR);
        return core::ptr::null_mut();
    }

    NDIS_DbgPrint!(DEBUG_MINIPORT, b"AdapterName = %wZ\n\0".as_ptr() as PCSTR, AdapterName);

    KeAcquireSpinLock(core::ptr::addr_of_mut!(AdapterListLock), core::ptr::addr_of_mut!(OldIrql));
    {
        CurrentEntry = (*core::ptr::addr_of_mut!(AdapterListHead)).Flink;

        while CurrentEntry != core::ptr::addr_of_mut!(AdapterListHead) {
            Adapter = CONTAINING_RECORD!(CurrentEntry, LOGICAL_ADAPTER, ListEntry);

            NDIS_DbgPrint!(DEBUG_MINIPORT, b"Examining adapter 0x%lx\n\0".as_ptr() as PCSTR, Adapter);

            /* We're technically not allowed to call this above PASSIVE_LEVEL, but it doesn't break
             * right now and I'd rather use a working API than reimplement it here */
            if RtlCompareUnicodeString(
                AdapterName as PCUNICODE_STRING,
                core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.MiniportName) as PCUNICODE_STRING,
                TRUE,
            ) == 0
            {
                break;
            }

            Adapter = core::ptr::null_mut();
            CurrentEntry = (*CurrentEntry).Flink;
        }
    }
    KeReleaseSpinLock(core::ptr::addr_of_mut!(AdapterListLock), OldIrql);

    if !Adapter.is_null() {
        NDIS_DbgPrint!(DEBUG_MINIPORT, b"Leaving. Adapter found at 0x%x\n\0".as_ptr() as PCSTR, Adapter);
    } else {
        NDIS_DbgPrint!(MIN_TRACE, b"Leaving (adapter not found for %wZ).\n\0".as_ptr() as PCSTR, AdapterName);
    }

    return Adapter;
}
#[no_mangle]
pub unsafe extern "C" fn MiniSetInformation(
    Adapter: PLOGICAL_ADAPTER,
    Oid: NDIS_OID,
    Size: ULONG,
    Buffer: PVOID,
    BytesRead: PULONG,
) -> NDIS_STATUS {
    let mut NdisStatus: NDIS_STATUS = 0;
    let mut NdisRequest: PNDIS_REQUEST = core::ptr::null_mut();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut MacBlock: PNDIS_REQUEST_MAC_BLOCK = core::ptr::null_mut();

    NDIS_DbgPrint!(DEBUG_MINIPORT, b"Called.\n\0".as_ptr() as PCSTR);

    NdisRequest = ExAllocatePool(NonPagedPool, core::mem::size_of::<NDIS_REQUEST>() as SIZE_T) as PNDIS_REQUEST;
    if NdisRequest.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources\n\0".as_ptr() as PCSTR);
        return NDIS_STATUS_RESOURCES;
    }

    RtlZeroMemory!(NdisRequest as *mut core::ffi::c_void, core::mem::size_of::<NDIS_REQUEST>() as SIZE_T);

    (*NdisRequest).RequestType = NdisRequestSetInformation;
    (*NdisRequest).DATA.SET_INFORMATION.Oid = Oid;
    (*NdisRequest).DATA.SET_INFORMATION.InformationBuffer = Buffer;
    (*NdisRequest).DATA.SET_INFORMATION.InformationBufferLength = Size;

    KeInitializeEvent(&raw mut Event, _EVENT_TYPE_NotificationEvent, FALSE);
    MacBlock = &raw mut (*NdisRequest).MacReserved as *mut UCHAR as PNDIS_REQUEST_MAC_BLOCK;
    (*MacBlock).Unknown1 = &raw mut Event as PVOID;

    NdisStatus = MiniDoRequest(Adapter, NdisRequest);

    if NdisStatus == NDIS_STATUS_PENDING {
        KeWaitForSingleObject(
            &raw mut Event as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
        NdisStatus = PtrToUlong!((*MacBlock).Unknown3 as *const core::ffi::c_void) as NDIS_STATUS;
    }

    *BytesRead = (*NdisRequest).DATA.SET_INFORMATION.BytesRead as ULONG;

    ExFreePool((NdisRequest).cast());

    return NdisStatus;
}
#[no_mangle]
pub unsafe extern "C" fn MiniQueryInformation(
    mut Adapter: PLOGICAL_ADAPTER,
    mut Oid: NDIS_OID,
    mut Size: ULONG,
    mut Buffer: PVOID,
    mut BytesWritten: PULONG,
) -> NDIS_STATUS {
    let mut NdisStatus: NDIS_STATUS = 0;
    let mut NdisRequest: PNDIS_REQUEST = ::core::ptr::null_mut::<_NDIS_REQUEST>();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut MacBlock: PNDIS_REQUEST_MAC_BLOCK = ::core::ptr::null_mut::<_NDIS_REQUEST_MAC_BLOCK>();
    if if (DEBUG_MINIPORT) as ::core::ffi::c_int > (NORMAL_MASK as i32) {
        (DebugTraceLevel & (DEBUG_MINIPORT) as ULONG > NORMAL_MASK as ULONG) as ::core::ffi::c_int
    } else {
        (DebugTraceLevel & NORMAL_MASK as ULONG >= (DEBUG_MINIPORT) as ULONG) as ::core::ffi::c_int
    } != 0
    {
        DbgPrint(
            b"(%s:%d)(%s) \0" as *const u8 as PCSTR,
            b"drivers/network/ndis/ndis/miniport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            828 as ::core::ffi::c_int,
            b"MiniQueryInformation\0" as *const u8 as *const ::core::ffi::c_char,
        );
        DbgPrint(b"Called.\n\0" as *const u8 as PCSTR);
    }
    NdisRequest = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<NDIS_REQUEST>() as SIZE_T,
    ) as PNDIS_REQUEST;
    if NdisRequest.is_null() {
        if if (MIN_TRACE) as ::core::ffi::c_int > (NORMAL_MASK as i32) {
            (DebugTraceLevel & (MIN_TRACE) as ULONG > NORMAL_MASK as ULONG) as ::core::ffi::c_int
        } else {
            (DebugTraceLevel & NORMAL_MASK as ULONG >= (MIN_TRACE) as ULONG) as ::core::ffi::c_int
        } != 0
        {
            DbgPrint(
                b"(%s:%d)(%s) \0" as *const u8 as PCSTR,
                b"drivers/network/ndis/ndis/miniport.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                832 as ::core::ffi::c_int,
                b"MiniQueryInformation\0" as *const u8 as *const ::core::ffi::c_char,
            );
            DbgPrint(b"Insufficient resources\n\0" as *const u8 as PCSTR);
        }
        return NDIS_STATUS_RESOURCES;
    }
    RtlZeroMemory!(
        NdisRequest as *mut ::core::ffi::c_void,
        core::mem::size_of::<NDIS_REQUEST>() as size_t,
    );
    (*NdisRequest).RequestType = NdisRequestQueryInformation;
    (*NdisRequest).DATA.QUERY_INFORMATION.Oid = Oid;
    (*NdisRequest).DATA.QUERY_INFORMATION.InformationBuffer = Buffer;
    (*NdisRequest).DATA.QUERY_INFORMATION.InformationBufferLength = Size as UINT;
    KeInitializeEvent(&raw mut Event, _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);
    MacBlock = &raw mut (*NdisRequest).MacReserved as *mut UCHAR as PNDIS_REQUEST_MAC_BLOCK;
    (*MacBlock).Unknown1 = &raw mut Event as PVOID;
    NdisStatus = MiniDoRequest(Adapter, NdisRequest);
    if NdisStatus == NDIS_STATUS_PENDING {
        KeWaitForSingleObject(
            &raw mut Event as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as ::core::ffi::c_int as KPROCESSOR_MODE,
            FALSE as BOOLEAN,
            ::core::ptr::null_mut::<_LARGE_INTEGER>(),
        );
        NdisStatus = PtrToUlong!((*MacBlock).Unknown3 as *const ::core::ffi::c_void) as NDIS_STATUS;
    }
    *BytesWritten = (*NdisRequest).DATA.QUERY_INFORMATION.BytesWritten as ::core::ffi::c_ulong;
    ExFreePool(NdisRequest as PVOID);
    return NdisStatus;
}
#[no_mangle]
pub unsafe extern "C" fn MiniCheckForHang(Adapter: PLOGICAL_ADAPTER) -> BOOLEAN {
    let mut Ret: BOOLEAN = FALSE;
    let mut OldIrql: KIRQL = 0;
    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(OldIrql));
    if let Some(CheckForHangHandler) = (*(*Adapter).NdisMiniportBlock.DriverHandle)
        .MiniportCharacteristics
        .CheckForHangHandler
    {
        Ret = CheckForHangHandler((*Adapter).NdisMiniportBlock.MiniportAdapterContext);
    }
    KeLowerIrql(OldIrql);
    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn MiniDoAddressingReset(Adapter: PLOGICAL_ADAPTER) {
    let mut BytesRead: ULONG = 0;

    MiniSetInformation(
        Adapter,
        OID_GEN_CURRENT_LOOKAHEAD as NDIS_OID,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.CurrentLookahead) as PVOID,
        core::ptr::addr_of_mut!(BytesRead),
    );

    /* FIXME: Set more stuff */
}
#[no_mangle]
pub unsafe extern "C" fn MiniReset(Adapter: PLOGICAL_ADAPTER) -> NDIS_STATUS {
    let mut Status: NDIS_STATUS = 0;
    let mut OldIrql: KIRQL = 0;
    let mut AddressingReset: BOOLEAN = TRUE as BOOLEAN;

    if MiniIsBusy(Adapter, NdisWorkItemResetRequested) != 0 {
        MiniQueueWorkItem(Adapter, NdisWorkItemResetRequested, core::ptr::null_mut(), FALSE);
        return NDIS_STATUS_PENDING;
    }

    NdisMIndicateStatus(Adapter as NDIS_HANDLE, NDIS_STATUS_RESET_START, core::ptr::null_mut(), 0);
    NdisMIndicateStatusComplete(Adapter as NDIS_HANDLE);

    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(OldIrql));
    Status = (((*(*Adapter).NdisMiniportBlock.DriverHandle).MiniportCharacteristics.ResetHandler).unwrap_unchecked())(
        core::ptr::addr_of_mut!(AddressingReset),
        (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
    );

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock));
    (*Adapter).NdisMiniportBlock.ResetStatus = Status;
    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock));

    KeLowerIrql(OldIrql);

    if Status != NDIS_STATUS_PENDING {
        if AddressingReset != 0 {
            MiniDoAddressingReset(Adapter);
        }

        NdisMIndicateStatus(Adapter as NDIS_HANDLE, NDIS_STATUS_RESET_END, core::ptr::null_mut(), 0);
        NdisMIndicateStatusComplete(Adapter as NDIS_HANDLE);

        MiniWorkItemComplete(Adapter, NdisWorkItemResetRequested);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn MiniportHangDpc(Dpc: PKDPC, DeferredContext: PVOID, SystemArgument1: PVOID, SystemArgument2: PVOID) {
    let mut Adapter: PLOGICAL_ADAPTER = DeferredContext as PLOGICAL_ADAPTER;
    if MiniCheckForHang(Adapter) != 0 {
        NDIS_DbgPrint!(MIN_TRACE, b"Miniport detected adapter hang\n\0".as_ptr() as PCSTR);
        MiniReset(Adapter);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MiniWorkItemComplete(
    mut Adapter: PLOGICAL_ADAPTER,
    mut WorkItemType: NDIS_WORK_ITEM_TYPE,
) {
    let mut IoWorkItem: PIO_WORKITEM = core::ptr::null_mut();
    /* Check if there's anything queued to run after this work item */
    if MiniIsBusy(Adapter, WorkItemType) == 0 {
        return;
    }
    /* There is, so fire the worker */
    IoWorkItem = IoAllocateWorkItem((*Adapter).NdisMiniportBlock.DeviceObject);
    if !IoWorkItem.is_null() {
        IoQueueWorkItem(
            IoWorkItem,
            Some(MiniportWorker),
            DelayedWorkQueue,
            IoWorkItem as PVOID,
        );
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MiniQueueWorkItem(
    mut Adapter: PLOGICAL_ADAPTER,
    mut WorkItemType: NDIS_WORK_ITEM_TYPE,
    mut WorkItemContext: PVOID,
    mut Top: BOOLEAN,
) {
    let mut MiniportWorkItem: PNDIS_MINIPORT_WORK_ITEM = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    KeAcquireSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, &raw mut OldIrql);
    if Top != 0 {
        if WorkItemType == NdisWorkItemSend {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Requeuing failed packet (%x).\n\0".as_ptr() as PCSTR,
                WorkItemContext,
            );
            (*Adapter).NdisMiniportBlock.FirstPendingPacket = WorkItemContext as PNDIS_PACKET;
        } else {
            //This should never happen
        }
    } else {
        MiniportWorkItem = ExAllocatePool(
            NonPagedPool,
            core::mem::size_of::<NDIS_MINIPORT_WORK_ITEM>() as SIZE_T,
        ) as PNDIS_MINIPORT_WORK_ITEM;
        if MiniportWorkItem.is_null() {
            KeReleaseSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, OldIrql);
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Insufficient resources.\n\0".as_ptr() as PCSTR,
            );
            return;
        }

        (*MiniportWorkItem).WorkItemType = WorkItemType;
        (*MiniportWorkItem).WorkItemContext = WorkItemContext;

        /* safe due to adapter lock held */
        (*MiniportWorkItem).Link.Next = core::ptr::null_mut();
        if (*Adapter).WorkQueueHead.is_null() {
            (*Adapter).WorkQueueHead = MiniportWorkItem;
            (*Adapter).WorkQueueTail = MiniportWorkItem;
        } else {
            (*(*Adapter).WorkQueueTail).Link.Next = MiniportWorkItem as PSINGLE_LIST_ENTRY as *mut _SINGLE_LIST_ENTRY;
            (*Adapter).WorkQueueTail = MiniportWorkItem;
        }
    }

    KeReleaseSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, OldIrql);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MiniQueueWorkItem(
    mut Adapter: PLOGICAL_ADAPTER,
    mut WorkItemType: NDIS_WORK_ITEM_TYPE,
    mut WorkItemContext: PVOID,
    mut Top: BOOLEAN,
) {
    let mut MiniportWorkItem: PNDIS_MINIPORT_WORK_ITEM = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    KeAcquireSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, &raw mut OldIrql);
    if Top != 0 {
        if WorkItemType == NdisWorkItemSend {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Requeuing failed packet (%x).\n\0".as_ptr() as PCSTR,
                WorkItemContext,
            );
            (*Adapter).NdisMiniportBlock.FirstPendingPacket = WorkItemContext as PNDIS_PACKET;
        } else {
            //This should never happen
        }
    } else {
        MiniportWorkItem = ExAllocatePool(
            NonPagedPool,
            core::mem::size_of::<NDIS_MINIPORT_WORK_ITEM>() as SIZE_T,
        ) as PNDIS_MINIPORT_WORK_ITEM;
        if MiniportWorkItem.is_null() {
            KeReleaseSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, OldIrql);
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Insufficient resources.\n\0".as_ptr() as PCSTR,
            );
            return;
        }

        (*MiniportWorkItem).WorkItemType = WorkItemType;
        (*MiniportWorkItem).WorkItemContext = WorkItemContext;

        /* safe due to adapter lock held */
        (*MiniportWorkItem).Link.Next = core::ptr::null_mut();
        if (*Adapter).WorkQueueHead.is_null() {
            (*Adapter).WorkQueueHead = MiniportWorkItem;
            (*Adapter).WorkQueueTail = MiniportWorkItem;
        } else {
            (*(*Adapter).WorkQueueTail).Link.Next = MiniportWorkItem as PSINGLE_LIST_ENTRY as *mut _SINGLE_LIST_ENTRY;
            (*Adapter).WorkQueueTail = MiniportWorkItem;
        }
    }

    KeReleaseSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, OldIrql);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MiniDequeueWorkItem(
    Adapter: PLOGICAL_ADAPTER,
    WorkItemType: *mut NDIS_WORK_ITEM_TYPE,
    WorkItemContext: *mut PVOID,
) -> NDIS_STATUS {
    let mut MiniportWorkItem: PNDIS_MINIPORT_WORK_ITEM = core::ptr::null_mut();
    let mut Packet: PNDIS_PACKET = core::ptr::null_mut();

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    MiniportWorkItem = (*Adapter).WorkQueueHead;

    Packet = (*Adapter).NdisMiniportBlock.FirstPendingPacket;
    if !Packet.is_null() {
        (*Adapter).NdisMiniportBlock.FirstPendingPacket = core::ptr::null_mut();

        *WorkItemType = NdisWorkItemSend;
        *WorkItemContext = Packet as PVOID;

        return NDIS_STATUS_SUCCESS;
    } else if !MiniportWorkItem.is_null() {
        /* safe due to adapter lock held */
        (*Adapter).WorkQueueHead = (*MiniportWorkItem).Link.Next as PNDIS_MINIPORT_WORK_ITEM;

        if MiniportWorkItem == (*Adapter).WorkQueueTail {
            (*Adapter).WorkQueueTail = core::ptr::null_mut();
        }

        *WorkItemType = (*MiniportWorkItem).WorkItemType;
        *WorkItemContext = (*MiniportWorkItem).WorkItemContext;

        ExFreePool(MiniportWorkItem as PVOID);

        return NDIS_STATUS_SUCCESS;
    } else {
        NDIS_DbgPrint!(MIN_TRACE, b"No work item to dequeue\n\0".as_ptr() as PCSTR);

        return NDIS_STATUS_FAILURE;
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MiniDequeueWorkItem(
    Adapter: PLOGICAL_ADAPTER,
    WorkItemType: *mut NDIS_WORK_ITEM_TYPE,
    WorkItemContext: *mut PVOID,
) -> NDIS_STATUS {
    let mut MiniportWorkItem: PNDIS_MINIPORT_WORK_ITEM = core::ptr::null_mut();
    let mut Packet: PNDIS_PACKET = core::ptr::null_mut();

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    MiniportWorkItem = (*Adapter).WorkQueueHead;

    Packet = (*Adapter).NdisMiniportBlock.FirstPendingPacket;
    if !Packet.is_null() {
        (*Adapter).NdisMiniportBlock.FirstPendingPacket = core::ptr::null_mut();

        *WorkItemType = NdisWorkItemSend;
        *WorkItemContext = Packet as PVOID;

        return NDIS_STATUS_SUCCESS;
    } else if !MiniportWorkItem.is_null() {
        /* safe due to adapter lock held */
        (*Adapter).WorkQueueHead = (*MiniportWorkItem).Link.Next as PNDIS_MINIPORT_WORK_ITEM;

        if MiniportWorkItem == (*Adapter).WorkQueueTail {
            (*Adapter).WorkQueueTail = core::ptr::null_mut();
        }

        *WorkItemType = (*MiniportWorkItem).WorkItemType;
        *WorkItemContext = (*MiniportWorkItem).WorkItemContext;

        ExFreePool(MiniportWorkItem as PVOID);

        return NDIS_STATUS_SUCCESS;
    } else {
        NDIS_DbgPrint!(MIN_TRACE, b"No work item to dequeue\n\0".as_ptr() as PCSTR);

        return NDIS_STATUS_FAILURE;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MiniDoRequest(
    Adapter: PLOGICAL_ADAPTER,
    NdisRequest: PNDIS_REQUEST,
) -> NDIS_STATUS {
    let mut Status: NDIS_STATUS;
    let mut OldIrql: KIRQL = core::mem::zeroed();
    NDIS_DbgPrint!(DEBUG_MINIPORT, b"Called.\n\0".as_ptr() as *const i8);

    // An NDIS 6 miniport's OID handler expects PASSIVE_LEVEL, so it is called without the
    // raise below; a pended request completes through NdisMOidRequestComplete.
    let a6 = crate::ndis::ndis6::adapter_of(Adapter);
    if !a6.is_null() {
        (*Adapter).NdisMiniportBlock.PendingRequest = NdisRequest;
        Status = crate::ndis::ndis6::do_request(a6, NdisRequest);
        if Status != NDIS_STATUS_PENDING {
            (*Adapter).NdisMiniportBlock.PendingRequest = core::ptr::null_mut();
            MiniWorkItemComplete(Adapter, NdisWorkItemRequest);
        }
        return Status;
    }

    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(OldIrql));

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock));
    (*Adapter).NdisMiniportBlock.PendingRequest = NdisRequest;
    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock));

    if (*(*Adapter).NdisMiniportBlock.DriverHandle)
        .MiniportCharacteristics
        .CoRequestHandler
        .is_none()
    {
        match (*NdisRequest).RequestType {
            NdisRequestQueryInformation => {
                let f = (*(*Adapter).NdisMiniportBlock.DriverHandle)
                    .MiniportCharacteristics
                    .QueryInformationHandler
                    .unwrap_unchecked();
                Status = f(
                    (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                    (*NdisRequest).DATA.QUERY_INFORMATION.Oid,
                    (*NdisRequest).DATA.QUERY_INFORMATION.InformationBuffer,
                    (*NdisRequest).DATA.QUERY_INFORMATION.InformationBufferLength,
                    core::ptr::addr_of_mut!((*NdisRequest).DATA.QUERY_INFORMATION.BytesWritten) as PULONG,
                    core::ptr::addr_of_mut!((*NdisRequest).DATA.QUERY_INFORMATION.BytesNeeded) as PULONG,
                );
            }
            NdisRequestSetInformation => {
                let f = (*(*Adapter).NdisMiniportBlock.DriverHandle)
                    .MiniportCharacteristics
                    .SetInformationHandler
                    .unwrap_unchecked();
                Status = f(
                    (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                    (*NdisRequest).DATA.SET_INFORMATION.Oid,
                    (*NdisRequest).DATA.SET_INFORMATION.InformationBuffer,
                    (*NdisRequest).DATA.SET_INFORMATION.InformationBufferLength,
                    core::ptr::addr_of_mut!((*NdisRequest).DATA.SET_INFORMATION.BytesRead) as PULONG,
                    core::ptr::addr_of_mut!((*NdisRequest).DATA.SET_INFORMATION.BytesNeeded) as PULONG,
                );
            }
            _ => {
                NDIS_DbgPrint!(MIN_TRACE, b"Bad request type\n\0".as_ptr() as *const i8);
                Status = NDIS_STATUS_FAILURE;
            }
        }
    } else {
        let f = (*(*Adapter).NdisMiniportBlock.DriverHandle)
            .MiniportCharacteristics
            .CoRequestHandler
            .unwrap_unchecked();
        Status = f(
            (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
            core::ptr::null_mut(), /* FIXME */
            NdisRequest,
        );
    }

    if Status != NDIS_STATUS_PENDING {
        KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock));
        (*Adapter).NdisMiniportBlock.PendingRequest = core::ptr::null_mut();
        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock));
    }

    KeLowerIrql(OldIrql);

    if Status != NDIS_STATUS_PENDING {
        MiniWorkItemComplete(Adapter, NdisWorkItemRequest);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMSetInformationComplete(
    MiniportAdapterHandle: NDIS_HANDLE,
    Status: NDIS_STATUS,
) {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut OldIrql: KIRQL = 0;
    if Adapter.is_null() {
        RtlAssert(
            b"Adapter\0".as_ptr() as *const i8 as PVOID,
            b"drivers/network/ndis/ndis/miniport.c\0".as_ptr() as *const i8 as PVOID,
            1194 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    KeRaiseIrql(DISPATCH_LEVEL as KIRQL, core::ptr::addr_of_mut!(OldIrql));
    if (*Adapter).NdisMiniportBlock.SetCompleteHandler.is_some() {
        (*Adapter)
            .NdisMiniportBlock
            .SetCompleteHandler
            .unwrap_unchecked()(MiniportAdapterHandle, Status);
    }
    KeLowerIrql(OldIrql);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMQueryInformationComplete(
    MiniportAdapterHandle: NDIS_HANDLE,
    Status: NDIS_STATUS,
) {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut OldIrql: KIRQL = 0;
    if Adapter.is_null() {
        RtlAssert(
            b"Adapter\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/network/ndis/ndis/miniport.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            1214 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(OldIrql));
    if (*Adapter).NdisMiniportBlock.QueryCompleteHandler.is_some() {
        (*Adapter)
            .NdisMiniportBlock
            .QueryCompleteHandler
            .unwrap_unchecked()(MiniportAdapterHandle, Status);
    }
    KeLowerIrql(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn MiniportWorker(DeviceObject: PDEVICE_OBJECT, Context: PVOID) {
    let Adapter: PLOGICAL_ADAPTER = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;
    let mut OldIrql: KIRQL = 0;
    let mut RaiseOldIrql: KIRQL = 0;
    let mut NdisStatus: NDIS_STATUS = 0;
    let mut WorkItemContext: PVOID = core::ptr::null_mut();
    let mut WorkItemType: NDIS_WORK_ITEM_TYPE = NdisWorkItemRequest;
    let mut AddressingReset: BOOLEAN = 0;

    IoFreeWorkItem(Context as PIO_WORKITEM);

    KeAcquireSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, &raw mut OldIrql);

    NdisStatus = MiniDequeueWorkItem(Adapter, &raw mut WorkItemType, &raw mut WorkItemContext);

    KeReleaseSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, OldIrql);

    if NdisStatus == NDIS_STATUS_SUCCESS {
        'sw: {
            match WorkItemType as u32 {
                NdisWorkItemSend => {
                    /*
                     * called by ProSend when protocols want to send packets to the miniport
                     */

                    if (*(*Adapter).NdisMiniportBlock.DriverHandle)
                        .MiniportCharacteristics
                        .SendPacketsHandler
                        .is_some()
                    {
                        if ((*Adapter).NdisMiniportBlock.Flags & NDIS_ATTRIBUTE_DESERIALIZE) != 0 {
                            NDIS_DbgPrint!(
                                MAX_TRACE,
                                b"Calling miniport's SendPackets handler\n\0".as_ptr() as PCSTR,
                            );
                            (*(*Adapter).NdisMiniportBlock.DriverHandle)
                                .MiniportCharacteristics
                                .SendPacketsHandler
                                .unwrap_unchecked()(
                                (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                                &raw mut WorkItemContext as PPNDIS_PACKET,
                                1,
                            );
                            NdisStatus = NDIS_STATUS_PENDING;
                        } else {
                            /* SendPackets is called at DISPATCH_LEVEL for all serialized miniports */
                            KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, &raw mut RaiseOldIrql);
                            {
                                NDIS_DbgPrint!(
                                    MAX_TRACE,
                                    b"Calling miniport's SendPackets handler\n\0".as_ptr() as PCSTR,
                                );
                                (*(*Adapter).NdisMiniportBlock.DriverHandle)
                                    .MiniportCharacteristics
                                    .SendPacketsHandler
                                    .unwrap_unchecked()(
                                    (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                                    &raw mut WorkItemContext as PPNDIS_PACKET,
                                    1,
                                );
                            }
                            KeLowerIrql(RaiseOldIrql);

                            NdisStatus = NDIS_GET_PACKET_STATUS!(WorkItemContext as PNDIS_PACKET);
                            if NdisStatus == NDIS_STATUS_RESOURCES {
                                MiniQueueWorkItem(
                                    Adapter,
                                    WorkItemType,
                                    WorkItemContext,
                                    TRUE,
                                );
                                break 'sw;
                            }
                        }
                    } else {
                        if ((*Adapter).NdisMiniportBlock.Flags & NDIS_ATTRIBUTE_DESERIALIZE) != 0 {
                            NDIS_DbgPrint!(
                                MAX_TRACE,
                                b"Calling miniport's Send handler\n\0".as_ptr() as PCSTR,
                            );
                            NdisStatus = (*(*Adapter).NdisMiniportBlock.DriverHandle)
                                .MiniportCharacteristics
                                .SendHandler
                                .unwrap_unchecked()(
                                (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                                WorkItemContext as PNDIS_PACKET,
                                (*(WorkItemContext as PNDIS_PACKET)).Private.Flags,
                            );
                            NDIS_DbgPrint!(
                                MAX_TRACE,
                                b"back from miniport's send handler\n\0".as_ptr() as PCSTR,
                            );
                        } else {
                            /* Send is called at DISPATCH_LEVEL for all serialized miniports */
                            KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, &raw mut RaiseOldIrql);
                            NDIS_DbgPrint!(
                                MAX_TRACE,
                                b"Calling miniport's Send handler\n\0".as_ptr() as PCSTR,
                            );
                            NdisStatus = (*(*Adapter).NdisMiniportBlock.DriverHandle)
                                .MiniportCharacteristics
                                .SendHandler
                                .unwrap_unchecked()(
                                (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                                WorkItemContext as PNDIS_PACKET,
                                (*(WorkItemContext as PNDIS_PACKET)).Private.Flags,
                            );
                            NDIS_DbgPrint!(
                                MAX_TRACE,
                                b"back from miniport's send handler\n\0".as_ptr() as PCSTR,
                            );
                            KeLowerIrql(RaiseOldIrql);
                            if NdisStatus == NDIS_STATUS_RESOURCES {
                                MiniQueueWorkItem(
                                    Adapter,
                                    WorkItemType,
                                    WorkItemContext,
                                    TRUE,
                                );
                                break 'sw;
                            }
                        }
                    }

                    if NdisStatus != NDIS_STATUS_PENDING {
                        MiniSendComplete(
                            (Adapter).cast(),
                            WorkItemContext as PNDIS_PACKET,
                            NdisStatus,
                        );
                    }
                }

                NdisWorkItemSendLoopback => {
                    /*
                     * called by ProSend when protocols want to send loopback packets
                     */
                    /* XXX atm ProIndicatePacket sends a packet up via the loopback adapter only */
                    NdisStatus = ProIndicatePacket(Adapter, WorkItemContext as PNDIS_PACKET);

                    if NdisStatus != NDIS_STATUS_PENDING {
                        MiniSendComplete(
                            Adapter as NDIS_HANDLE,
                            WorkItemContext as PNDIS_PACKET,
                            NdisStatus,
                        );
                    }
                }

                NdisWorkItemReturnPackets => {}

                NdisWorkItemResetRequested => {
                    NdisMIndicateStatus(
                        (Adapter).cast(),
                        NDIS_STATUS_RESET_START,
                        core::ptr::null_mut(),
                        0,
                    );
                    NdisMIndicateStatusComplete((Adapter).cast());

                    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, &raw mut OldIrql);
                    NdisStatus = (*(*Adapter).NdisMiniportBlock.DriverHandle)
                        .MiniportCharacteristics
                        .ResetHandler
                        .unwrap_unchecked()(
                        &raw mut AddressingReset,
                        (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
                    );

                    KeAcquireSpinLockAtDpcLevel(&raw mut (*Adapter).NdisMiniportBlock.Lock);
                    (*Adapter).NdisMiniportBlock.ResetStatus = NdisStatus;
                    KeReleaseSpinLockFromDpcLevel(&raw mut (*Adapter).NdisMiniportBlock.Lock);

                    KeLowerIrql(OldIrql);

                    if NdisStatus != NDIS_STATUS_PENDING {
                        MiniResetComplete((Adapter).cast(), NdisStatus, AddressingReset);
                    }
                }

                NdisWorkItemResetInProgress => {}

                NdisWorkItemMiniportCallback => {}

                NdisWorkItemRequest => {
                    NdisStatus = MiniDoRequest(Adapter, WorkItemContext as PNDIS_REQUEST);

                    if NdisStatus == NDIS_STATUS_PENDING {
                        break 'sw;
                    }

                    (*Adapter).NdisMiniportBlock.PendingRequest = WorkItemContext as PNDIS_REQUEST;
                    match (*(WorkItemContext as PNDIS_REQUEST)).RequestType as u32 {
                        NdisRequestQueryInformation => {
                            NdisMQueryInformationComplete(Adapter as NDIS_HANDLE, NdisStatus);
                        }

                        NdisRequestSetInformation => {
                            NdisMSetInformationComplete(Adapter as NDIS_HANDLE, NdisStatus);
                        }

                        _ => {
                            NDIS_DbgPrint!(
                                MIN_TRACE,
                                b"Unknown NDIS request type.\n\0".as_ptr() as PCSTR,
                            );
                        }
                    }
                    (*Adapter).NdisMiniportBlock.PendingRequest = core::ptr::null_mut();
                }

                _ => {
                    NDIS_DbgPrint!(
                        MIN_TRACE,
                        b"Unknown NDIS work item type (%d).\n\0".as_ptr() as PCSTR,
                        WorkItemType as u32,
                    );
                }
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn MiniStatus(MiniportHandle: NDIS_HANDLE, GeneralStatus: NDIS_STATUS, StatusBuffer: PVOID, StatusBufferSize: UINT) {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportHandle as PLOGICAL_ADAPTER;
    let mut CurrentEntry: PLIST_ENTRY;
    let mut AdapterBinding: PADAPTER_BINDING;
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock),
        &mut OldIrql,
    );

    CurrentEntry = (*Adapter).ProtocolListHead.Flink;

    while CurrentEntry != core::ptr::addr_of_mut!((*Adapter).ProtocolListHead) {
        AdapterBinding = ((CurrentEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(ADAPTER_BINDING, AdapterListEntry) as ULONG_PTR,
        )) as PADAPTER_BINDING;

        let StatusHandler = (*(*AdapterBinding).ProtocolBinding)
            .Chars
            .__bindgen_anon_1.StatusHandler
            .unwrap_unchecked();
        StatusHandler(
            (*AdapterBinding).NdisOpenBlock.ProtocolBindingContext,
            GeneralStatus,
            StatusBuffer,
            StatusBufferSize,
        );

        CurrentEntry = (*CurrentEntry).Flink;
    }

    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock),
        OldIrql,
    );
}
#[no_mangle]
pub unsafe extern "system" fn MiniStatusComplete(MiniportAdapterHandle: NDIS_HANDLE) {
    let Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut CurrentEntry: PLIST_ENTRY;
    let mut AdapterBinding: PADAPTER_BINDING;
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock), &mut OldIrql);

    CurrentEntry = (*Adapter).ProtocolListHead.Flink;

    while CurrentEntry != core::ptr::addr_of_mut!((*Adapter).ProtocolListHead) {
        AdapterBinding = ((CurrentEntry as *mut u8)
            .sub(core::mem::offset_of!(ADAPTER_BINDING, AdapterListEntry))
            as *mut ADAPTER_BINDING) as PADAPTER_BINDING;

        ((*(*AdapterBinding).ProtocolBinding).Chars.__bindgen_anon_1.StatusCompleteHandler
            .unwrap_unchecked())(
            (*AdapterBinding).NdisOpenBlock.ProtocolBindingContext,
        );

        CurrentEntry = (*CurrentEntry).Flink;
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock), OldIrql);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMCloseLog(LogHandle: NDIS_HANDLE) {
    let Log: PNDIS_LOG = LogHandle as PNDIS_LOG;
    let Miniport: PNDIS_MINIPORT_BLOCK = (*Log).Miniport;
    let mut OldIrql: KIRQL = 0;

    NDIS_DbgPrint!(MAX_TRACE, b"called: LogHandle 0x%x\n\0".as_ptr() as PCSTR, LogHandle);

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*Miniport).Lock), core::ptr::addr_of_mut!(OldIrql));
    (*Miniport).Log = core::ptr::null_mut::<_NDIS_LOG>();
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*Miniport).Lock), OldIrql);

    ExFreePool(Log as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMCreateLog(
    mut MiniportAdapterHandle: NDIS_HANDLE,
    mut Size: UINT,
    mut LogHandle: PNDIS_HANDLE,
) -> NDIS_STATUS {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut Log: PNDIS_LOG = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"called: MiniportAdapterHandle 0x%x, Size %ld\n\0".as_ptr() as PCSTR,
        MiniportAdapterHandle,
        Size,
    );

    KeAcquireSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, &raw mut OldIrql);

    // Diverges from the C, which returns from both failure paths still holding the adapter
    // lock at DISPATCH_LEVEL: the caller resumes at raised IRQL and the next acquire of the
    // lock spins forever. Both paths release it here.
    if !(*Adapter).NdisMiniportBlock.Log.is_null() {
        *LogHandle = core::ptr::null_mut();
        KeReleaseSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, OldIrql);
        return NDIS_STATUS_FAILURE;
    }

    Log = ExAllocatePool(
        NonPagedPool,
        (Size as SIZE_T).wrapping_add(core::mem::size_of::<NDIS_LOG>() as SIZE_T),
    ) as PNDIS_LOG;
    if Log.is_null() {
        *LogHandle = core::ptr::null_mut();
        KeReleaseSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, OldIrql);
        return NDIS_STATUS_RESOURCES;
    }

    (*Adapter).NdisMiniportBlock.Log = Log;

    KeInitializeSpinLock(&raw mut (*Log).LogLock);

    (*Log).Miniport = &raw mut (*Adapter).NdisMiniportBlock as PNDIS_MINIPORT_BLOCK;
    (*Log).TotalSize = Size;
    (*Log).CurrentSize = 0 as UINT;
    (*Log).OutPtr = 0 as UINT;
    (*Log).InPtr = 0 as UINT;
    (*Log).Irp = core::ptr::null_mut();

    *LogHandle = Log as PVOID;

    KeReleaseSpinLock(&raw mut (*Adapter).NdisMiniportBlock.Lock, OldIrql);

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMDeregisterAdapterShutdownHandler(MiniportHandle: NDIS_HANDLE) {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportHandle as PLOGICAL_ADAPTER;
    NDIS_DbgPrint!(DEBUG_MINIPORT, b"Called.\n\0".as_ptr() as PCSTR);
    if (*(*Adapter).BugcheckContext).ShutdownHandler.is_some() {
        KeDeregisterBugCheckCallback((*(*Adapter).BugcheckContext).CallbackRecord);
        IoUnregisterShutdownNotification((*Adapter).NdisMiniportBlock.DeviceObject);
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisMFlushLog(LogHandle: NDIS_HANDLE) {
    let Log: PNDIS_LOG = LogHandle as PNDIS_LOG;
    let mut OldIrql: KIRQL = 0;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"called: LogHandle 0x%x\n\0".as_ptr() as PCSTR,
        LogHandle,
    );

    /* Lock object */
    KeAcquireSpinLock(&raw mut (*Log).LogLock, &raw mut OldIrql);

    /* Set buffers size */
    (*Log).CurrentSize = 0 as UINT;
    (*Log).OutPtr = 0 as UINT;
    (*Log).InPtr = 0 as UINT;

    /* Unlock object */
    KeReleaseSpinLock(&raw mut (*Log).LogLock, OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn NdisMIndicateStatus(
    MiniportAdapterHandle: NDIS_HANDLE,
    GeneralStatus: NDIS_STATUS,
    StatusBuffer: PVOID,
    StatusBufferSize: UINT,
) {
    MiniStatus(MiniportAdapterHandle, GeneralStatus, StatusBuffer, StatusBufferSize);
}

#[no_mangle]
pub unsafe extern "system" fn NdisMIndicateStatusComplete(MiniportAdapterHandle: NDIS_HANDLE) {
    MiniStatusComplete(MiniportAdapterHandle);
}
#[no_mangle]
pub unsafe extern "system" fn NdisInitializeWrapper(
    NdisWrapperHandle: PNDIS_HANDLE,
    SystemSpecific1: PVOID,
    SystemSpecific2: PVOID,
    SystemSpecific3: PVOID,
) {
    let mut Miniport: PNDIS_M_DRIVER_BLOCK;
    let mut RegistryPath: PUNICODE_STRING;
    let mut RegistryBuffer: *mut WCHAR;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    *NdisWrapperHandle = core::ptr::null_mut();

    if 0 != 0 {
        DbgBreakPoint();
    }

    Miniport = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<NDIS_M_DRIVER_BLOCK>() as SIZE_T,
    ) as PNDIS_M_DRIVER_BLOCK;

    if Miniport.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0".as_ptr() as *const i8);
        return;
    }

    RtlZeroMemory!(
        Miniport as PVOID,
        core::mem::size_of::<NDIS_M_DRIVER_BLOCK>() as SIZE_T,
    );

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Miniport).Lock));

    (*Miniport).DriverObject = SystemSpecific1 as PDRIVER_OBJECT;

    /* set the miniport's driver registry path */
    RegistryPath = ExAllocatePool(
        PagedPool,
        core::mem::size_of::<_UNICODE_STRING>() as SIZE_T,
    ) as PUNICODE_STRING;
    if RegistryPath.is_null() {
        ExFreePool(Miniport as PVOID);
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0".as_ptr() as *const i8);
        return;
    }

    (*RegistryPath).Length = (*(SystemSpecific2 as PUNICODE_STRING)).Length;
    (*RegistryPath).MaximumLength = ((*RegistryPath).Length as usize)
        .wrapping_add(core::mem::size_of::<WCHAR>() as usize) as USHORT;

    RegistryBuffer = ExAllocatePool(
        PagedPool,
        (*RegistryPath).MaximumLength as SIZE_T,
    ) as *mut WCHAR;
    if RegistryBuffer.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0".as_ptr() as *const i8);
        ExFreePool(Miniport as PVOID);
        ExFreePool(RegistryPath as PVOID);
        return;
    }

    RtlCopyMemory!(
        RegistryBuffer as PVOID,
        (*(SystemSpecific2 as PUNICODE_STRING)).Buffer as PVOID,
        (*RegistryPath).Length as SIZE_T,
    );
    *RegistryBuffer.offset(
        ((*RegistryPath).Length as usize).wrapping_div(core::mem::size_of::<WCHAR>() as usize)
            as isize,
    ) = 0;

    (*RegistryPath).Buffer = RegistryBuffer as PWCH;
    (*Miniport).RegistryPath = RegistryPath;

    InitializeListHead(core::ptr::addr_of_mut!((*Miniport).DeviceList));

    /* Put miniport in global miniport list */
    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!(MiniportListHead),
        core::ptr::addr_of_mut!((*Miniport).ListEntry),
        core::ptr::addr_of_mut!(MiniportListLock),
    );

    *NdisWrapperHandle = Miniport as PVOID;
}
#[no_mangle]
pub unsafe extern "system" fn NdisIBugcheckCallback(Buffer: PVOID, Length: ULONG) {
    let Context: PMINIPORT_BUGCHECK_CONTEXT = Buffer as PMINIPORT_BUGCHECK_CONTEXT;
    let sh: ADAPTER_SHUTDOWN_HANDLER = (*Context).ShutdownHandler;

    NDIS_DbgPrint!(DEBUG_MINIPORT, b"Called.\n\0".as_ptr() as PCSTR);

    if let Some(sh) = sh {
        sh((*Context).DriverContext);
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisMRegisterAdapterShutdownHandler(
    MiniportHandle: NDIS_HANDLE,
    ShutdownContext: PVOID,
    ShutdownHandler: ADAPTER_SHUTDOWN_HANDLER,
) {
    let Adapter: PLOGICAL_ADAPTER = MiniportHandle as PLOGICAL_ADAPTER;
    let mut BugcheckContext: PMINIPORT_BUGCHECK_CONTEXT = core::ptr::null_mut();

    NDIS_DbgPrint!(DEBUG_MINIPORT, b"Called.\n\0".as_ptr() as PCSTR);

    if !(*Adapter).BugcheckContext.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Attempted to register again for a shutdown callback\n\0".as_ptr() as PCSTR,
        );
        return;
    }

    BugcheckContext = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<MINIPORT_BUGCHECK_CONTEXT>() as SIZE_T,
    ) as PMINIPORT_BUGCHECK_CONTEXT;
    if BugcheckContext.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources.\n\0".as_ptr() as PCSTR,
        );
        return;
    }

    (*BugcheckContext).ShutdownHandler = ShutdownHandler;
    (*BugcheckContext).DriverContext = ShutdownContext;

    (*BugcheckContext).CallbackRecord = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<KBUGCHECK_CALLBACK_RECORD>() as SIZE_T,
    ) as PKBUGCHECK_CALLBACK_RECORD;
    if (*BugcheckContext).CallbackRecord.is_null() {
        ExFreePool(BugcheckContext as PVOID);
        return;
    }

    (*Adapter).BugcheckContext = BugcheckContext;

    KeInitializeCallbackRecord!((*BugcheckContext).CallbackRecord);

    KeRegisterBugCheckCallback(
        (*BugcheckContext).CallbackRecord,
        Some(NdisIBugcheckCallback),
        BugcheckContext as PVOID,
        core::mem::size_of::<MINIPORT_BUGCHECK_CONTEXT>() as ULONG,
        b"Ndis Miniport\0".as_ptr() as PUCHAR,
    );

    IoRegisterShutdownNotification((*Adapter).NdisMiniportBlock.DeviceObject);
}
#[no_mangle]
pub unsafe extern "C" fn DoQueries(
    mut Adapter: PLOGICAL_ADAPTER,
    mut AddressOID: NDIS_OID,
) -> NDIS_STATUS {
    let mut BytesWritten: ULONG = 0;
    let mut NdisStatus: NDIS_STATUS = 0;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    /* Get MAC options for adapter */
    NdisStatus = MiniQueryInformation(
        Adapter,
        OID_GEN_MAC_OPTIONS,
        core::mem::size_of::<UINT>() as ULONG,
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.MacOptions) as PVOID,
        core::ptr::addr_of_mut!(BytesWritten),
    );

    if NdisStatus != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"OID_GEN_MAC_OPTIONS failed. NdisStatus (0x%X).\n\0".as_ptr() as PCSTR,
            NdisStatus,
        );
        return NdisStatus;
    }

    NDIS_DbgPrint!(
        DEBUG_MINIPORT,
        b"MacOptions (0x%X).\n\0".as_ptr() as PCSTR,
        (*Adapter).NdisMiniportBlock.MacOptions,
    );

    /* Get current hardware address of adapter */
    NdisStatus = MiniQueryInformation(
        Adapter,
        AddressOID,
        (*Adapter).AddressLength,
        core::ptr::addr_of_mut!((*Adapter).Address) as PVOID,
        core::ptr::addr_of_mut!(BytesWritten),
    );

    if NdisStatus != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Address OID (0x%X) failed. NdisStatus (0x%X).\n\0".as_ptr() as PCSTR,
            AddressOID,
            NdisStatus,
        );
        return NdisStatus;
    }

    {
        /* 802.3 only */

        let mut A: PUCHAR = core::ptr::addr_of_mut!((*Adapter).Address.Type.Medium802_3) as PUCHAR;

        NDIS_DbgPrint!(
            MAX_TRACE,
            b"Adapter address is (%02X %02X %02X %02X %02X %02X).\n\0".as_ptr() as PCSTR,
            *A.offset(0 as isize) as ::core::ffi::c_int,
            *A.offset(1 as isize) as ::core::ffi::c_int,
            *A.offset(2 as isize) as ::core::ffi::c_int,
            *A.offset(3 as isize) as ::core::ffi::c_int,
            *A.offset(4 as isize) as ::core::ffi::c_int,
            *A.offset(5 as isize) as ::core::ffi::c_int,
        );
    }

    /* Get maximum lookahead buffer size of adapter */
    NdisStatus = MiniQueryInformation(
        Adapter,
        OID_GEN_MAXIMUM_LOOKAHEAD,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.MaximumLookahead) as PVOID,
        core::ptr::addr_of_mut!(BytesWritten),
    );

    if NdisStatus != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"OID_GEN_MAXIMUM_LOOKAHEAD failed. NdisStatus (0x%X).\n\0".as_ptr() as PCSTR,
            NdisStatus,
        );
        return NdisStatus;
    }

    NDIS_DbgPrint!(
        DEBUG_MINIPORT,
        b"MaxLookaheadLength (0x%X).\n\0".as_ptr() as PCSTR,
        (*Adapter).NdisMiniportBlock.MaximumLookahead,
    );

    /* Get current lookahead buffer size of adapter */
    NdisStatus = MiniQueryInformation(
        Adapter,
        OID_GEN_CURRENT_LOOKAHEAD,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.CurrentLookahead) as PVOID,
        core::ptr::addr_of_mut!(BytesWritten),
    );

    if NdisStatus != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"OID_GEN_CURRENT_LOOKAHEAD failed. NdisStatus (0x%X).\n\0".as_ptr() as PCSTR,
            NdisStatus,
        );
        return NdisStatus;
    }

    NdisStatus = MiniQueryInformation(
        Adapter,
        OID_GEN_MAXIMUM_SEND_PACKETS,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.MaxSendPackets) as PVOID,
        core::ptr::addr_of_mut!(BytesWritten),
    );

    if NdisStatus != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"OID_GEN_MAXIMUM_SEND_PACKETS failed. NdisStatus (0x%X).\n\0".as_ptr() as PCSTR,
            NdisStatus,
        );

        /* Set it to 1 if it fails because some drivers don't support this (?)*/
        (*Adapter).NdisMiniportBlock.MaxSendPackets = 1 as USHORT;
    }

    NDIS_DbgPrint!(
        DEBUG_MINIPORT,
        b"CurLookaheadLength (0x%X).\n\0".as_ptr() as PCSTR,
        (*Adapter).NdisMiniportBlock.CurrentLookahead,
    );

    return STATUS_SUCCESS as NDIS_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisIForwardIrpAndWaitCompletionRoutine(mut Fdo: PDEVICE_OBJECT, mut Irp: PIRP, mut Context: PVOID) -> NTSTATUS {
    let mut Event: PKEVENT = Context as PKEVENT;
    if (*Irp).PendingReturned != 0 {
        KeSetEvent(
            Event as PRKEVENT,
            IO_NO_INCREMENT as KPRIORITY,
            FALSE as BOOLEAN,
        );
    }
    return STATUS_MORE_PROCESSING_REQUIRED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisIForwardIrpAndWait(Adapter: PLOGICAL_ADAPTER, Irp: PIRP) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, FALSE);
    IoCopyCurrentIrpStackLocationToNext(Irp);
    IoSetCompletionRoutine(
        Irp,
        Some(NdisIForwardIrpAndWaitCompletionRoutine),
        core::ptr::addr_of_mut!(Event) as PVOID,
        TRUE,
        TRUE,
        TRUE,
    );
    Status = IoCallDriver((*Adapter).NdisMiniportBlock.NextDeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn NdisICreateClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;

    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NdisIPnPStartDevice(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let Adapter: PLOGICAL_ADAPTER = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;
    let mut WrapperContext: NDIS_WRAPPER_CONTEXT = core::mem::zeroed();
    let mut NdisStatus: NDIS_STATUS = 0;
    let mut OpenErrorStatus: NDIS_STATUS = 0;
    let mut Status: NTSTATUS = 0;
    let mut SelectedMediumIndex: UINT = 0;
    let mut AddressOID: NDIS_OID = 0;
    let mut Success: BOOLEAN = FALSE;
    let mut ResourceCount: ULONG = 0;
    let mut ResourceListSize: ULONG = 0;
    let mut ParamName: _UNICODE_STRING = core::mem::zeroed();
    let mut ConfigParam: PNDIS_CONFIGURATION_PARAMETER = core::ptr::null_mut();
    let mut ConfigHandle: NDIS_HANDLE = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut MaxMulticastAddresses: UINT = 0;
    let mut BytesWritten: ULONG = 0;
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut ProtocolBinding: PPROTOCOL_BINDING = core::ptr::null_mut();

    /*
     * Prepare wrapper context used by HW and configuration routines.
     */

    NDIS_DbgPrint!(
        DEBUG_MINIPORT,
        b"Start Device %wZ\n\0".as_ptr() as PCSTR,
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.MiniportName),
    );

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Inserting adapter 0x%x into adapter list\n\0".as_ptr() as PCSTR,
        Adapter,
    );

    /* Put adapter in global adapter list */
    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!(AdapterListHead),
        core::ptr::addr_of_mut!((*Adapter).ListEntry),
        core::ptr::addr_of_mut!(AdapterListLock),
    );

    Status = IoOpenDeviceRegistryKey(
        (*Adapter).NdisMiniportBlock.PhysicalDeviceObject,
        PLUGPLAY_REGKEY_DRIVER,
        KEY_ALL_ACCESS,
        core::ptr::addr_of_mut!(WrapperContext.RegistryHandle),
    );
    if !NT_SUCCESS!(Status) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"failed to open adapter-specific reg key\n\0".as_ptr() as PCSTR,
        );
        ExInterlockedRemoveEntryList!(
            core::ptr::addr_of_mut!((*Adapter).ListEntry),
            core::ptr::addr_of_mut!(AdapterListLock),
        );
        return Status;
    }

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"opened device reg key\n\0".as_ptr() as PCSTR,
    );

    WrapperContext.DeviceObject = (*Adapter).NdisMiniportBlock.DeviceObject;

    /*
     * Store the adapter resources used by HW routines such as
     * NdisMQueryAdapterResources.
     */

    if !(*Stack).Parameters.StartDevice.AllocatedResources.is_null() {
        ResourceCount = (*(*Stack).Parameters.StartDevice.AllocatedResources).List
            [0 as usize]
            .PartialResourceList
            .Count;
        ResourceListSize = ((FIELD_OFFSET!(CM_RESOURCE_LIST, List[0 as usize].PartialResourceList.PartialDescriptors) as usize) + ResourceCount as usize
            * core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>())
            as ULONG;

        (*Adapter).NdisMiniportBlock.AllocatedResources =
            ExAllocatePool(PagedPool, ResourceListSize as SIZE_T)
                as PCM_RESOURCE_LIST;
        if (*Adapter).NdisMiniportBlock.AllocatedResources.is_null() {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Insufficient resources\n\0".as_ptr() as PCSTR,
            );
            ExInterlockedRemoveEntryList!(
                core::ptr::addr_of_mut!((*Adapter).ListEntry),
                core::ptr::addr_of_mut!(AdapterListLock),
            );
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*Adapter).NdisMiniportBlock.Resources =
            ExAllocatePool(PagedPool, ResourceListSize as SIZE_T)
                as PCM_RESOURCE_LIST;
        if (*Adapter).NdisMiniportBlock.Resources.is_null() {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Insufficient resources\n\0".as_ptr() as PCSTR,
            );
            ExFreePool(
                (*Adapter).NdisMiniportBlock.AllocatedResources as PVOID,
            );
            ExInterlockedRemoveEntryList!(
                core::ptr::addr_of_mut!((*Adapter).ListEntry),
                core::ptr::addr_of_mut!(AdapterListLock),
            );
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::copy_nonoverlapping(
            (*Stack).Parameters.StartDevice.AllocatedResources as *const u8,
            (*Adapter).NdisMiniportBlock.Resources as *mut u8,
            ResourceListSize as usize,
        );

        core::ptr::copy_nonoverlapping(
            (*Stack).Parameters.StartDevice.AllocatedResources as *const u8,
            (*Adapter).NdisMiniportBlock.AllocatedResources as *mut u8,
            ResourceListSize as usize,
        );
    }

    if !(*Stack).Parameters.StartDevice.AllocatedResourcesTranslated.is_null() {
        ResourceCount = (*(*Stack).Parameters.StartDevice.AllocatedResourcesTranslated).List
            [0 as usize]
            .PartialResourceList
            .Count;
        ResourceListSize = ((FIELD_OFFSET!(CM_RESOURCE_LIST, List[0 as usize].PartialResourceList.PartialDescriptors) as usize) + ResourceCount as usize
            * core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>())
            as ULONG;

        (*Adapter).NdisMiniportBlock.AllocatedResourcesTranslated =
            ExAllocatePool(PagedPool, ResourceListSize as SIZE_T)
                as PCM_RESOURCE_LIST;
        if (*Adapter).NdisMiniportBlock.AllocatedResourcesTranslated.is_null() {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Insufficient resources\n\0".as_ptr() as PCSTR,
            );
            ExInterlockedRemoveEntryList!(
                core::ptr::addr_of_mut!((*Adapter).ListEntry),
                core::ptr::addr_of_mut!(AdapterListLock),
            );
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::copy_nonoverlapping(
            (*Stack).Parameters.StartDevice.AllocatedResourcesTranslated as *const u8,
            (*Adapter).NdisMiniportBlock.AllocatedResourcesTranslated as *mut u8,
            ResourceListSize as usize,
        );
    }

    /*
     * Store the Bus Type, Bus Number and Slot information. It's used by
     * the hardware routines then.
     */

    NdisOpenConfiguration(
        core::ptr::addr_of_mut!(NdisStatus),
        core::ptr::addr_of_mut!(ConfigHandle),
        core::ptr::addr_of_mut!(WrapperContext) as NDIS_HANDLE,
    );
    if NdisStatus != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Failed to open configuration key\n\0".as_ptr() as PCSTR,
        );
        ExInterlockedRemoveEntryList!(
            core::ptr::addr_of_mut!((*Adapter).ListEntry),
            core::ptr::addr_of_mut!(AdapterListLock),
        );
        return NdisStatus as NTSTATUS;
    }

    Size = core::mem::size_of::<ULONG>() as ULONG;
    Status = IoGetDeviceProperty(
        (*Adapter).NdisMiniportBlock.PhysicalDeviceObject,
        DevicePropertyLegacyBusType,
        Size,
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.BusType) as PVOID,
        core::ptr::addr_of_mut!(Size),
    );
    if !NT_SUCCESS!(Status)
        || ((*Adapter).NdisMiniportBlock.BusType as INTERFACE_TYPE)
            == InterfaceTypeUndefined
    {
        NdisInitUnicodeString(
            core::ptr::addr_of_mut!(ParamName),
            RUSTCONV_WSTR_FB83D8E92731.as_ptr(),
        );
        NdisReadConfiguration(
            core::ptr::addr_of_mut!(NdisStatus),
            core::ptr::addr_of_mut!(ConfigParam),
            ConfigHandle,
            core::ptr::addr_of_mut!(ParamName),
            NdisParameterInteger,
        );
        if NdisStatus == NDIS_STATUS_SUCCESS {
            (*Adapter).NdisMiniportBlock.BusType =
                (*ConfigParam).ParameterData.IntegerData as NDIS_INTERFACE_TYPE;
        } else {
            (*Adapter).NdisMiniportBlock.BusType = NdisInterfaceIsa;
        }
    }

    Status = IoGetDeviceProperty(
        (*Adapter).NdisMiniportBlock.PhysicalDeviceObject,
        DevicePropertyBusNumber,
        Size,
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.BusNumber) as PVOID,
        core::ptr::addr_of_mut!(Size),
    );
    if !NT_SUCCESS!(Status) || (*Adapter).NdisMiniportBlock.BusNumber == 0xFFFFFFF0 {
        NdisInitUnicodeString(
            core::ptr::addr_of_mut!(ParamName),
            RUSTCONV_WSTR_25B56DF68A17.as_ptr(),
        );
        NdisReadConfiguration(
            core::ptr::addr_of_mut!(NdisStatus),
            core::ptr::addr_of_mut!(ConfigParam),
            ConfigHandle,
            core::ptr::addr_of_mut!(ParamName),
            NdisParameterInteger,
        );
        if NdisStatus == NDIS_STATUS_SUCCESS {
            (*Adapter).NdisMiniportBlock.BusNumber =
                (*ConfigParam).ParameterData.IntegerData;
        } else {
            (*Adapter).NdisMiniportBlock.BusNumber = 0;
        }
    }
    WrapperContext.BusNumber = (*Adapter).NdisMiniportBlock.BusNumber;

    Status = IoGetDeviceProperty(
        (*Adapter).NdisMiniportBlock.PhysicalDeviceObject,
        DevicePropertyAddress,
        Size,
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.SlotNumber) as PVOID,
        core::ptr::addr_of_mut!(Size),
    );
    if !NT_SUCCESS!(Status)
        || (*Adapter).NdisMiniportBlock.SlotNumber
            == (-(1 as i64)) as ULONG
    {
        NdisInitUnicodeString(
            core::ptr::addr_of_mut!(ParamName),
            RUSTCONV_WSTR_E56EFEA51CCC.as_ptr(),
        );
        NdisReadConfiguration(
            core::ptr::addr_of_mut!(NdisStatus),
            core::ptr::addr_of_mut!(ConfigParam),
            ConfigHandle,
            core::ptr::addr_of_mut!(ParamName),
            NdisParameterInteger,
        );
        if NdisStatus == NDIS_STATUS_SUCCESS {
            (*Adapter).NdisMiniportBlock.SlotNumber =
                (*ConfigParam).ParameterData.IntegerData;
        } else {
            (*Adapter).NdisMiniportBlock.SlotNumber = 0;
        }
    } else {
        /* Convert slotnumber to PCI_SLOT_NUMBER */
        let PciSlotNumber: ULONG = (*Adapter).NdisMiniportBlock.SlotNumber;
        let mut SlotNumber: PCI_SLOT_NUMBER = core::mem::zeroed();

        SlotNumber.u.AsULONG = 0;
        SlotNumber.u.bits.set_DeviceNumber(
            ((PciSlotNumber >> 16) & 0xFFFF) as ULONG,
        );
        SlotNumber.u.bits.set_FunctionNumber(
            (PciSlotNumber & 0xFFFF) as ULONG,
        );

        (*Adapter).NdisMiniportBlock.SlotNumber = SlotNumber.u.AsULONG;
    }
    WrapperContext.SlotNumber = (*Adapter).NdisMiniportBlock.SlotNumber;

    if (*Adapter).NdisMiniportBlock.BusType as ::core::ffi::c_uint
        == NdisInterfacePci as ::core::ffi::c_uint
    {
        Status = NdisQueryPciBusInterface(Adapter);
    }

    NdisCloseConfiguration(ConfigHandle);

    /* Set handlers (some NDIS macros require these) */
    (*Adapter).NdisMiniportBlock.EthRxCompleteHandler = Some(EthFilterDprIndicateReceiveComplete);
    (*Adapter).NdisMiniportBlock.EthRxIndicateHandler = Some(EthFilterDprIndicateReceive);
    (*Adapter).NdisMiniportBlock.SendCompleteHandler = Some(MiniSendComplete);
    (*Adapter).NdisMiniportBlock.SendResourcesHandler = Some(MiniSendResourcesAvailable);
    (*Adapter).NdisMiniportBlock.ResetCompleteHandler = Some(MiniResetComplete);
    (*Adapter).NdisMiniportBlock.TDCompleteHandler = Some(MiniTransferDataComplete);
    (*Adapter).NdisMiniportBlock.PacketIndicateHandler = Some(MiniIndicateReceivePacket);
    (*Adapter).NdisMiniportBlock.StatusHandler = Some(MiniStatus);
    (*Adapter).NdisMiniportBlock.StatusCompleteHandler = Some(MiniStatusComplete);
    (*Adapter).NdisMiniportBlock.SendPacketsHandler = Some(ProSendPackets);
    (*Adapter).NdisMiniportBlock.QueryCompleteHandler = Some(MiniRequestComplete);
    (*Adapter).NdisMiniportBlock.SetCompleteHandler = Some(MiniRequestComplete);

    /*
     * Call MiniportInitialize.
     */

    NDIS_DbgPrint!(
        MID_TRACE,
        b"calling MiniportInitialize\n\0".as_ptr() as PCSTR,
    );
    NdisStatus = ((*(*Adapter).NdisMiniportBlock.DriverHandle)
        .MiniportCharacteristics
        .InitializeHandler
        .unwrap_unchecked())(
        core::ptr::addr_of_mut!(OpenErrorStatus),
        core::ptr::addr_of_mut!(SelectedMediumIndex),
        (core::ptr::addr_of_mut!(MediaArray) as *mut NDIS_MEDIUM)
            .offset(0 as isize) as PNDIS_MEDIUM,
        MEDIA_ARRAY_SIZE as UINT,
        Adapter as NDIS_HANDLE,
        core::ptr::addr_of_mut!(WrapperContext) as NDIS_HANDLE,
    );

    ZwClose(WrapperContext.RegistryHandle);

    if NdisStatus != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"MiniportInitialize() failed for an adapter (%lx).\n\0".as_ptr() as PCSTR,
            NdisStatus,
        );
        ExInterlockedRemoveEntryList!(
            core::ptr::addr_of_mut!((*Adapter).ListEntry),
            core::ptr::addr_of_mut!(AdapterListLock),
        );
        if !(*Adapter).NdisMiniportBlock.Interrupt.is_null() {
            KeBugCheckEx(
                BUGCODE_ID_DRIVER,
                Adapter as ULONG_PTR,
                (*Adapter).NdisMiniportBlock.Interrupt as ULONG_PTR,
                (*Adapter).NdisMiniportBlock.TimerQueue as ULONG_PTR,
                1 as ULONG_PTR,
            );
        }
        if !(*Adapter).NdisMiniportBlock.TimerQueue.is_null() {
            KeBugCheckEx(
                BUGCODE_ID_DRIVER,
                Adapter as ULONG_PTR,
                (*Adapter).NdisMiniportBlock.Interrupt as ULONG_PTR,
                (*Adapter).NdisMiniportBlock.TimerQueue as ULONG_PTR,
                1 as ULONG_PTR,
            );
        }
        return NdisStatus as NTSTATUS;
    }

    if SelectedMediumIndex >= (MEDIA_ARRAY_SIZE as u32) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"MiniportInitialize() selected a bad index\n\0".as_ptr() as PCSTR,
        );
        ExInterlockedRemoveEntryList!(
            core::ptr::addr_of_mut!((*Adapter).ListEntry),
            core::ptr::addr_of_mut!(AdapterListLock),
        );
        return NDIS_STATUS_UNSUPPORTED_MEDIA;
    }

    (*Adapter).NdisMiniportBlock.MediaType = MediaArray[SelectedMediumIndex as usize];

    match (*Adapter).NdisMiniportBlock.MediaType as ::core::ffi::c_uint {
        NdisMedium802_3 => {
            (*Adapter).MediumHeaderSize = 14;
            AddressOID = OID_802_3_CURRENT_ADDRESS;
            (*Adapter).AddressLength = ETH_LENGTH_OF_ADDRESS;
            NdisStatus = DoQueries(Adapter, AddressOID);
            if NdisStatus == NDIS_STATUS_SUCCESS {
                NdisStatus = MiniQueryInformation(
                    Adapter,
                    OID_802_3_MAXIMUM_LIST_SIZE,
                    core::mem::size_of::<UINT>() as ULONG,
                    core::ptr::addr_of_mut!(MaxMulticastAddresses) as PVOID,
                    core::ptr::addr_of_mut!(BytesWritten),
                );

                if NdisStatus != NDIS_STATUS_SUCCESS {
                    ExInterlockedRemoveEntryList!(
                        core::ptr::addr_of_mut!((*Adapter).ListEntry),
                        core::ptr::addr_of_mut!(AdapterListLock),
                    );
                    NDIS_DbgPrint!(
                        MIN_TRACE,
                        b"MiniQueryInformation failed (%x)\n\0".as_ptr() as PCSTR,
                        NdisStatus,
                    );
                    return NdisStatus as NTSTATUS;
                }

                Success = EthCreateFilter(
                    MaxMulticastAddresses,
                    core::ptr::addr_of_mut!((*Adapter).Address.Type.Medium802_3) as PUCHAR,
                    core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.__bindgen_anon_1.EthDB),
                );
                if Success != 0 {
                    (*((*Adapter).NdisMiniportBlock.__bindgen_anon_1.EthDB as PETHI_FILTER))
                        .Miniport = Adapter as PNDIS_MINIPORT_BLOCK as *mut _NDIS_MINIPORT_BLOCK;
                } else {
                    NdisStatus = NDIS_STATUS_RESOURCES;
                }
            }
        }
        _ => {
            /* FIXME: Support other types of media */
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"error: unsupported media\n\0".as_ptr() as PCSTR,
            );

            ExInterlockedRemoveEntryList!(
                core::ptr::addr_of_mut!((*Adapter).ListEntry),
                core::ptr::addr_of_mut!(AdapterListLock),
            );
            return STATUS_UNSUCCESSFUL;
        }
    }

    if NdisStatus != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"couldn't create filter (%x)\n\0".as_ptr() as PCSTR,
            NdisStatus,
        );
        return NdisStatus as NTSTATUS;
    }

    /* Check for a hang every two seconds if it wasn't set in MiniportInitialize */
    if (*Adapter).NdisMiniportBlock.CheckForHangSeconds == 0 {
        (*Adapter).NdisMiniportBlock.CheckForHangSeconds = 2;
    }

    (*Adapter).NdisMiniportBlock.OldPnPDeviceState = (*Adapter).NdisMiniportBlock.PnPDeviceState;
    (*Adapter).NdisMiniportBlock.PnPDeviceState = NdisPnPDeviceStarted;

    IoSetDeviceInterfaceState(
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.SymbolicLinkName),
        TRUE,
    );

    Timeout.QuadPart = Int32x32To64!(
        (*Adapter).NdisMiniportBlock.CheckForHangSeconds as i32,
        -(1000000 as i32),
    ) as LONGLONG;
    KeSetTimerEx(
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.WakeUpDpcTimer.Timer),
        Timeout,
        ((*Adapter).NdisMiniportBlock.CheckForHangSeconds * 1000 as ULONG)
            as LONG,
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.WakeUpDpcTimer.Dpc),
    );

    /* Put adapter in adapter list for this miniport */
    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!((*(*Adapter).NdisMiniportBlock.DriverHandle).DeviceList),
        core::ptr::addr_of_mut!((*Adapter).MiniportListEntry),
        core::ptr::addr_of_mut!((*(*Adapter).NdisMiniportBlock.DriverHandle).Lock),
    );

    /* Refresh bindings for all protocols */
    CurrentEntry = ProtocolListHead.Flink as PLIST_ENTRY;
    while CurrentEntry != core::ptr::addr_of_mut!(ProtocolListHead) {
        ProtocolBinding = (CurrentEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(PROTOCOL_BINDING, ListEntry) as ULONG_PTR,
        ) as *mut PROTOCOL_BINDING as PPROTOCOL_BINDING;

        ndisBindMiniportsToProtocol(
            core::ptr::addr_of_mut!(NdisStatus),
            ProtocolBinding,
        );

        CurrentEntry = (*CurrentEntry).Flink as PLIST_ENTRY;
    }

    return STATUS_SUCCESS as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisIPnPStopDevice(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Adapter: PLOGICAL_ADAPTER = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;

    /* Remove adapter from adapter list for this miniport */
    ExInterlockedRemoveEntryList!(
        core::ptr::addr_of_mut!((*Adapter).MiniportListEntry),
        core::ptr::addr_of_mut!((*(*Adapter).NdisMiniportBlock.DriverHandle).Lock),
    );

    /* Remove adapter from global adapter list */
    ExInterlockedRemoveEntryList!(
        core::ptr::addr_of_mut!((*Adapter).ListEntry),
        core::ptr::addr_of_mut!(AdapterListLock),
    );

    KeCancelTimer(core::ptr::addr_of_mut!(
        (*Adapter).NdisMiniportBlock.WakeUpDpcTimer.Timer
    ));

    /* Set this here so MiniportISR will be forced to run for interrupts generated in MiniportHalt */
    (*Adapter).NdisMiniportBlock.OldPnPDeviceState = (*Adapter).NdisMiniportBlock.PnPDeviceState;
    (*Adapter).NdisMiniportBlock.PnPDeviceState = NdisPnPDeviceStopped;

    ((*(*Adapter).NdisMiniportBlock.DriverHandle)
        .MiniportCharacteristics
        .HaltHandler
        .unwrap_unchecked())(Adapter as NDIS_HANDLE);

    IoSetDeviceInterfaceState(
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.SymbolicLinkName),
        FALSE,
    );

    if !(*Adapter).NdisMiniportBlock.AllocatedResources.is_null() {
        ExFreePool((*Adapter).NdisMiniportBlock.AllocatedResources as PVOID);
        (*Adapter).NdisMiniportBlock.AllocatedResources =
            core::ptr::null_mut::<_CM_RESOURCE_LIST>();
    }
    if !(*Adapter).NdisMiniportBlock.AllocatedResourcesTranslated.is_null() {
        ExFreePool((*Adapter).NdisMiniportBlock.AllocatedResourcesTranslated as PVOID);
        (*Adapter).NdisMiniportBlock.AllocatedResourcesTranslated =
            core::ptr::null_mut::<_CM_RESOURCE_LIST>();
    }

    if (*Adapter).BusInterface.SetBusData.is_some() {
        if (*Adapter).BusInterface.InterfaceDereference.is_some() {
            ((*Adapter).BusInterface.InterfaceDereference
                .unwrap_unchecked())((*Adapter).BusInterface.Context);
        }
        (*Adapter).BusInterface.SetBusData = None;
    }

    if !(*Adapter).NdisMiniportBlock.Resources.is_null() {
        ExFreePool((*Adapter).NdisMiniportBlock.Resources as PVOID);
        (*Adapter).NdisMiniportBlock.Resources = core::ptr::null_mut::<_CM_RESOURCE_LIST>();
    }

    if !(*Adapter).NdisMiniportBlock.__bindgen_anon_1.EthDB.is_null() {
        EthDeleteFilter((*Adapter).NdisMiniportBlock.__bindgen_anon_1.EthDB);
        (*Adapter).NdisMiniportBlock.__bindgen_anon_1.EthDB = core::ptr::null_mut::<_X_FILTER>();
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisIShutdown(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Adapter: PLOGICAL_ADAPTER = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;
    let Context: PMINIPORT_BUGCHECK_CONTEXT = (*Adapter).BugcheckContext;
    let ShutdownHandler: ADAPTER_SHUTDOWN_HANDLER = (*Context).ShutdownHandler;

    (ShutdownHandler.unwrap_unchecked())((*Context).DriverContext);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;

    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NdisIDeviceIoControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Adapter: PLOGICAL_ADAPTER = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;
    let mut Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Status: NDIS_STATUS = STATUS_NOT_SUPPORTED;
    let mut ControlCode: ULONG;
    let mut Written: ULONG = core::mem::zeroed();

    (*Irp).IoStatus.Information = 0;

    ControlCode = (*Stack).Parameters.DeviceIoControl.IoControlCode;
    match ControlCode {
        IOCTL_NDIS_QUERY_GLOBAL_STATS => {
            Status = MiniQueryInformation(
                Adapter,
                *((*Irp).AssociatedIrp.SystemBuffer as PNDIS_OID),
                (*Stack).Parameters.DeviceIoControl.OutputBufferLength,
                MmGetSystemAddressForMdl!((*Irp).MdlAddress),
                core::ptr::addr_of_mut!(Written),
            );
            (*Irp).IoStatus.Information = Written as ULONG_PTR;
        }
        IOCTL_NDIS_RESERVED7 => {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"NdisIDeviceIoControl: IOCTL_NDIS_RESERVED7 UNIMPLEMENTED (CORE-13831)\n\0"
                    .as_ptr() as PCSTR,
            );
            Status = STATUS_NOT_IMPLEMENTED;
        }
        _ => {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"NdisIDeviceIoControl: unsupported control code 0x%lx\n\0".as_ptr() as PCSTR,
                ControlCode,
            );
        }
    }

    if Status != NDIS_STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    } else {
        IoMarkIrpPending(Irp);
    }

    return Status;
}

unsafe extern "C" fn NdisIPnPRemoveDevice(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Adapter: PLOGICAL_ADAPTER = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;

    if !(*Adapter).NdisMiniportBlock.SymbolicLinkName.Buffer.is_null() {
        IoSetDeviceInterfaceState(
            &raw mut (*Adapter).NdisMiniportBlock.SymbolicLinkName,
            FALSE as BOOLEAN,
        );
        RtlFreeUnicodeString(&raw mut (*Adapter).NdisMiniportBlock.SymbolicLinkName);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoSkipCurrentIrpStackLocation(Irp);
    Status = IoCallDriver((*Adapter).NdisMiniportBlock.NextDeviceObject, Irp);

    IoDetachDevice((*Adapter).NdisMiniportBlock.NextDeviceObject);

    RtlFreeUnicodeString(&raw mut (*Adapter).NdisMiniportBlock.MiniportName);

    if (*Adapter).NdisMiniportBlock.PnPDeviceState as ::core::ffi::c_uint
        == NdisPnPDeviceStarted as ::core::ffi::c_uint
    {
        /* Remove adapter from adapter list for this miniport */
        ExInterlockedRemoveEntryList!(
            &raw mut (*Adapter).MiniportListEntry,
            &raw mut (*(*Adapter).NdisMiniportBlock.DriverHandle).Lock,
        );

        /* Remove adapter from global adapter list */
        ExInterlockedRemoveEntryList!(&raw mut (*Adapter).ListEntry, &raw mut AdapterListLock);

        KeCancelTimer(&raw mut (*Adapter).NdisMiniportBlock.WakeUpDpcTimer.Timer);

        (*(*Adapter).NdisMiniportBlock.DriverHandle)
            .MiniportCharacteristics
            .HaltHandler
            .unwrap_unchecked()(Adapter as NDIS_HANDLE);
    }

    if !(*Adapter).NdisMiniportBlock.__bindgen_anon_1.EthDB.is_null() {
        EthDeleteFilter((*Adapter).NdisMiniportBlock.__bindgen_anon_1.EthDB);
        (*Adapter).NdisMiniportBlock.__bindgen_anon_1.EthDB = ::core::ptr::null_mut::<_X_FILTER>();
    }

    if !(*Adapter).NdisMiniportBlock.Resources.is_null() {
        ExFreePool((*Adapter).NdisMiniportBlock.Resources as PVOID);
        (*Adapter).NdisMiniportBlock.Resources = ::core::ptr::null_mut::<_CM_RESOURCE_LIST>();
    }

    if !(*Adapter).NdisMiniportBlock.AllocatedResources.is_null() {
        ExFreePool((*Adapter).NdisMiniportBlock.AllocatedResources as PVOID);
        (*Adapter).NdisMiniportBlock.AllocatedResources =
            ::core::ptr::null_mut::<_CM_RESOURCE_LIST>();
    }

    if !(*Adapter).NdisMiniportBlock.AllocatedResourcesTranslated.is_null() {
        ExFreePool(
            (*Adapter).NdisMiniportBlock.AllocatedResourcesTranslated as PVOID,
        );
        (*Adapter).NdisMiniportBlock.AllocatedResourcesTranslated =
            ::core::ptr::null_mut::<_CM_RESOURCE_LIST>();
    }

    IoDeleteDevice(DeviceObject);

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn NdisIDispatchPnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let Adapter: PLOGICAL_ADAPTER = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;
    let mut Status: NTSTATUS = 0;

    match (*Stack).MinorFunction {
        IRP_MN_START_DEVICE => {
            Status = NdisIForwardIrpAndWait(Adapter, Irp);
            if NT_SUCCESS!(Status) && NT_SUCCESS!((*Irp).IoStatus.__bindgen_anon_1.Status) {
                Status = NdisIPnPStartDevice(DeviceObject, Irp);
            } else {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"Lower driver failed device start\n\0".as_ptr() as PCSTR,
                );
            }
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
            return Status;
        }

        IRP_MN_STOP_DEVICE => {
            Status = NdisIPnPStopDevice(DeviceObject, Irp);
            if !NT_SUCCESS!(Status) {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"WARNING: Ignoring halt device failure! Passing the IRP down anyway\n\0".as_ptr() as PCSTR,
                );
            }
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        }

        IRP_MN_QUERY_REMOVE_DEVICE | IRP_MN_QUERY_STOP_DEVICE => {
            Status = NdisIPnPQueryStopDevice(DeviceObject, Irp);
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            if Status != STATUS_SUCCESS {
                IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"Failing miniport halt request\n\0".as_ptr() as PCSTR,
                );
                return Status;
            }
        }

        IRP_MN_CANCEL_REMOVE_DEVICE | IRP_MN_CANCEL_STOP_DEVICE => {
            Status = NdisIForwardIrpAndWait(Adapter, Irp);
            if NT_SUCCESS!(Status) && NT_SUCCESS!((*Irp).IoStatus.__bindgen_anon_1.Status) {
                Status = NdisIPnPCancelStopDevice(DeviceObject, Irp);
            } else {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"Lower driver failed cancel stop/remove request\n\0".as_ptr() as PCSTR,
                );
            }
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
            return Status;
        }

        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            Status = NDIS_STATUS_SUCCESS;
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            (*Irp).IoStatus.Information |= (*Adapter).NdisMiniportBlock.PnPFlags as ULONG_PTR;
        }

        IRP_MN_REMOVE_DEVICE => {
            return NdisIPnPRemoveDevice(DeviceObject, Irp);
        }

        _ => {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Unhandled minor function: 0x%X\n\0".as_ptr() as PCSTR,
                (*Stack).MinorFunction as u32,
            );
        }
    }

    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver((*Adapter).NdisMiniportBlock.NextDeviceObject, Irp);
}
#[no_mangle]
pub unsafe extern "system" fn NdisIPower(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut Adapter: PLOGICAL_ADAPTER = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;

    PoStartNextPowerIrp(Irp);
    IoSkipCurrentIrpStackLocation(Irp);
    return PoCallDriver(
        (*Adapter).NdisMiniportBlock.NextDeviceObject as PDEVICE_OBJECT,
        Irp,
    );
}

#[no_mangle]
pub unsafe extern "system" fn NdisIAddDevice(DriverObject: PDRIVER_OBJECT, PhysicalDeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    static ClassKeyName: [WCHAR; 6] = [0x0043u16, 0x006C, 0x0061, 0x0073, 0x0073, 0x005C];
    static LinkageKeyName: [WCHAR; 9] = [0x005Cu16, 0x004C, 0x0069, 0x006E, 0x006B, 0x0061, 0x0067, 0x0065, 0x0000];
    let mut Miniport: PNDIS_M_DRIVER_BLOCK = core::ptr::null_mut();
    let mut MiniportPtr: *mut PNDIS_M_DRIVER_BLOCK = core::ptr::null_mut();
    let mut LinkageKeyBuffer: *mut WCHAR = core::ptr::null_mut();
    let mut DriverKeyLength: ULONG = 0;
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
    let mut ExportName: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Adapter: PLOGICAL_ADAPTER = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    /*
     * Gain the access to the miniport data structure first.
     */

    MiniportPtr = IoGetDriverObjectExtension(DriverObject, (core::mem::size_of::<PVOID>() as PVOID))
        as *mut PNDIS_M_DRIVER_BLOCK;
    if MiniportPtr.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Can't get driver object extension.\n\0".as_ptr() as PCSTR,
        );
        return NDIS_STATUS_FAILURE;
    }
    Miniport = *MiniportPtr;

    /*
     * Get name of the Linkage registry key for our adapter.
     */

    Status = IoGetDeviceProperty(
        PhysicalDeviceObject,
        DevicePropertyDriverKeyName,
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(DriverKeyLength),
    );
    if Status != STATUS_BUFFER_TOO_SMALL && Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Can't get miniport driver key length.\n\0".as_ptr() as PCSTR,
        );
        return Status;
    }

    LinkageKeyBuffer = ExAllocatePool(
        PagedPool,
        (DriverKeyLength
            + core::mem::size_of::<[WCHAR; 6]>() as ULONG
            + core::mem::size_of::<[WCHAR; 9]>() as ULONG) as SIZE_T,
    ) as *mut WCHAR;
    if LinkageKeyBuffer.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Can't allocate memory for driver key name.\n\0".as_ptr() as PCSTR,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = IoGetDeviceProperty(
        PhysicalDeviceObject,
        DevicePropertyDriverKeyName,
        DriverKeyLength,
        LinkageKeyBuffer.add(core::mem::size_of::<[WCHAR; 6]>() / core::mem::size_of::<WCHAR>())
            as PVOID,
        core::ptr::addr_of_mut!(DriverKeyLength),
    );
    if !NT_SUCCESS!(Status) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Can't get miniport driver key.\n\0".as_ptr() as PCSTR,
        );
        ExFreePool(LinkageKeyBuffer as PVOID);
        return Status;
    }

    /* Compose the linkage key name. */
    RtlCopyMemory!(
        LinkageKeyBuffer as PVOID,
        core::ptr::addr_of!(ClassKeyName) as *const WCHAR as PVOID,
        core::mem::size_of::<[WCHAR; 6]>() as SIZE_T,
    );
    RtlCopyMemory!(
        LinkageKeyBuffer
            .add((core::mem::size_of::<[WCHAR; 6]>() + DriverKeyLength as usize) / core::mem::size_of::<WCHAR>())
            .sub(1 as usize) as PVOID,
        core::ptr::addr_of!(LinkageKeyName) as *const WCHAR as PVOID,
        core::mem::size_of::<[WCHAR; 9]>() as SIZE_T,
    );

    NDIS_DbgPrint!(
        DEBUG_MINIPORT,
        b"LinkageKey: %S.\n\0".as_ptr() as PCSTR,
        LinkageKeyBuffer,
    );

    /*
     * Now open the linkage key and read the "Export" and "RootDevice" values
     * which contains device name and root service respectively.
     */

    RtlZeroMemory!(
        core::ptr::addr_of_mut!(QueryTable) as PVOID,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>() as SIZE_T,
    );
    RtlInitUnicodeString(core::ptr::addr_of_mut!(ExportName), core::ptr::null());
    QueryTable[0].Flags =
        (RTL_QUERY_REGISTRY_REQUIRED | RTL_QUERY_REGISTRY_DIRECT) as ULONG;
    QueryTable[0].Name = RUSTCONV_WSTR_853B46C50805.as_ptr();
    QueryTable[0].EntryContext =
        core::ptr::addr_of_mut!(ExportName) as PVOID;

    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_CONTROL,
        LinkageKeyBuffer as PCWSTR,
        core::ptr::addr_of_mut!(QueryTable) as PRTL_QUERY_REGISTRY_TABLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    ExFreePool(LinkageKeyBuffer as PVOID);
    if !NT_SUCCESS!(Status) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Can't get miniport device name. (%x)\n\0".as_ptr() as PCSTR,
            Status,
        );
        return Status;
    }

    /*
     * Create the device object.
     */

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"creating device %wZ\n\0".as_ptr() as PCSTR,
        core::ptr::addr_of_mut!(ExportName),
    );

    Status = IoCreateDevice(
        (*Miniport).DriverObject,
        core::mem::size_of::<LOGICAL_ADAPTER>() as ULONG,
        core::ptr::addr_of_mut!(ExportName),
        FILE_DEVICE_PHYSICAL_NETCARD as ULONG,
        0 as ULONG,
        FALSE as BOOLEAN,
        core::ptr::addr_of_mut!(DeviceObject),
    );
    if !NT_SUCCESS!(Status) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Could not create device object.\n\0".as_ptr() as PCSTR,
        );
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(ExportName));
        return Status;
    }

    /*
     * Initialize the adapter structure.
     */

    Adapter = (*DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.Lock));
    InitializeListHead(core::ptr::addr_of_mut!((*Adapter).ProtocolListHead));

    Status = IoRegisterDeviceInterface(
        PhysicalDeviceObject,
        core::ptr::addr_of!(GUID_DEVINTERFACE_NET),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.SymbolicLinkName),
    );

    if !NT_SUCCESS!(Status) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Could not create device interface.\n\0".as_ptr() as PCSTR,
        );
        IoDeleteDevice(DeviceObject);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(ExportName));
        return Status;
    }

    (*Adapter).NdisMiniportBlock.DriverHandle = Miniport;
    (*Adapter).NdisMiniportBlock.MiniportName = ExportName;
    (*Adapter).NdisMiniportBlock.DeviceObject = DeviceObject;
    (*Adapter).NdisMiniportBlock.PhysicalDeviceObject = PhysicalDeviceObject;
    (*Adapter).NdisMiniportBlock.NextDeviceObject = IoAttachDeviceToDeviceStack(
        (*Adapter).NdisMiniportBlock.DeviceObject,
        PhysicalDeviceObject,
    );

    (*Adapter).NdisMiniportBlock.OldPnPDeviceState = 0;
    (*Adapter).NdisMiniportBlock.PnPDeviceState = NdisPnPDeviceAdded;

    KeInitializeTimer(core::ptr::addr_of_mut!(
        (*Adapter).NdisMiniportBlock.WakeUpDpcTimer.Timer
    ));
    KeInitializeDpc(
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.WakeUpDpcTimer.Dpc),
        Some(MiniportHangDpc),
        Adapter as PVOID,
    );

    (*DeviceObject).Flags &= !(DO_DEVICE_INITIALIZING as ULONG);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisGenericIrpHandler(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    /* Use the characteristics to classify the device */
    if (*DeviceObject).DeviceType == FILE_DEVICE_PHYSICAL_NETCARD {
        if (*IrpSp).MajorFunction == (IRP_MJ_CREATE as u8)
            || (*IrpSp).MajorFunction == (IRP_MJ_CLOSE as u8)
            || (*IrpSp).MajorFunction == (IRP_MJ_CLEANUP as u8)
        {
            return NdisICreateClose(DeviceObject, Irp);
        } else if (*IrpSp).MajorFunction == (IRP_MJ_PNP as u8) {
            return NdisIDispatchPnp(DeviceObject, Irp);
        } else if (*IrpSp).MajorFunction == (IRP_MJ_SHUTDOWN as u8) {
            return NdisIShutdown(DeviceObject, Irp);
        } else if (*IrpSp).MajorFunction == (IRP_MJ_DEVICE_CONTROL as u8) {
            return NdisIDeviceIoControl(DeviceObject, Irp);
        } else if (*IrpSp).MajorFunction == (IRP_MJ_POWER as u8) {
            return NdisIPower(DeviceObject, Irp);
        }
        NDIS_DbgPrint!(
            MIN_TRACE,
            (
                b"Unexpected IRP MajorFunction 0x%x\n\0".as_ptr() as PCSTR,
                (*IrpSp).MajorFunction as ULONG,
            ),
        );
        if 0 == 0 {
            RtlAssert(
                b"FALSE\0".as_ptr() as *const core::ffi::c_char as PVOID,
                b"drivers/network/ndis/ndis/miniport.c\0".as_ptr() as *const core::ffi::c_char as PVOID,
                2691 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    } else if (*DeviceObject).DeviceType == FILE_DEVICE_NETWORK {
        let DeviceBlock: PNDIS_M_DEVICE_BLOCK = (*DeviceObject).DeviceExtension as PNDIS_M_DEVICE_BLOCK;

        if !((*DeviceBlock).DeviceObject == DeviceObject) {
            RtlAssert(
                b"DeviceBlock->DeviceObject == DeviceObject\0".as_ptr() as *const core::ffi::c_char as PVOID,
                b"drivers/network/ndis/ndis/miniport.c\0".as_ptr() as *const core::ffi::c_char as PVOID,
                2697 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }

        if (*DeviceBlock).MajorFunction[(*IrpSp).MajorFunction as usize].is_some() {
            return (*DeviceBlock).MajorFunction[(*IrpSp).MajorFunction as usize]
                .unwrap_unchecked()(DeviceObject, Irp);
        }
    } else {
        if 0 == 0 {
            RtlAssert(
                b"FALSE\0".as_ptr() as *const core::ffi::c_char as PVOID,
                b"drivers/network/ndis/ndis/miniport.c\0".as_ptr() as *const core::ffi::c_char as PVOID,
                2706 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
    (*Irp).IoStatus.Information = 0;

    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    return STATUS_INVALID_DEVICE_REQUEST;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMRegisterMiniport(
    NdisWrapperHandle: NDIS_HANDLE,
    MiniportCharacteristics: PNDIS_MINIPORT_CHARACTERISTICS,
    CharacteristicsLength: UINT,
) -> NDIS_STATUS {
    let mut MinSize: UINT = 0;
    let mut Miniport: PNDIS_M_DRIVER_BLOCK = GET_MINIPORT_DRIVER!(NdisWrapperHandle);
    let mut MiniportPtr: *mut PNDIS_M_DRIVER_BLOCK = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut i: ULONG;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    match (*MiniportCharacteristics).MajorNdisVersion as ::core::ffi::c_int {
        0x03 => {
            MinSize = core::mem::size_of::<NDIS30_MINIPORT_CHARACTERISTICS>() as UINT;
        }

        0x04 => {
            MinSize = core::mem::size_of::<NDIS40_MINIPORT_CHARACTERISTICS>() as UINT;
        }

        0x05 => {
            match (*MiniportCharacteristics).MinorNdisVersion as ::core::ffi::c_int {
                0x00 => {
                    MinSize = core::mem::size_of::<NDIS50_MINIPORT_CHARACTERISTICS>() as UINT;
                }

                0x01 => {
                    MinSize = core::mem::size_of::<NDIS51_MINIPORT_CHARACTERISTICS>() as UINT;
                }

                _ => {
                    NDIS_DbgPrint!(
                        MIN_TRACE,
                        b"Bad 5.x minor characteristics version.\n\0".as_ptr() as PCSTR,
                    );
                    return NDIS_STATUS_BAD_VERSION;
                }
            }
        }

        _ => {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Bad miniport characteristics version.\n\0".as_ptr() as PCSTR,
            );
            return NDIS_STATUS_BAD_VERSION;
        }
    }

    NDIS_DbgPrint!(
        MID_TRACE,
        b"Initializing an NDIS %u.%u miniport\n\0".as_ptr() as PCSTR,
        (*MiniportCharacteristics).MajorNdisVersion as ::core::ffi::c_int,
        (*MiniportCharacteristics).MinorNdisVersion as ::core::ffi::c_int,
    );

    if CharacteristicsLength < MinSize {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Bad miniport characteristics length.\n\0".as_ptr() as PCSTR,
        );
        return NDIS_STATUS_BAD_CHARACTERISTICS;
    }

    /* Check if mandatory MiniportXxx functions are specified */
    if (*MiniportCharacteristics).HaltHandler.is_none()
        || (*MiniportCharacteristics).InitializeHandler.is_none()
        || (*MiniportCharacteristics).ResetHandler.is_none()
    {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Bad miniport characteristics.\n\0".as_ptr() as PCSTR,
        );
        return NDIS_STATUS_BAD_CHARACTERISTICS;
    }

    if ((*MiniportCharacteristics).MajorNdisVersion as ::core::ffi::c_int)
        < 0x05 as ::core::ffi::c_int
    {
        if (*MiniportCharacteristics).QueryInformationHandler.is_none()
            || (*MiniportCharacteristics).SetInformationHandler.is_none()
        {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Bad miniport characteristics. (Set/Query)\n\0".as_ptr() as PCSTR,
            );
            return NDIS_STATUS_BAD_CHARACTERISTICS;
        }
    } else {
        if ((*MiniportCharacteristics).QueryInformationHandler.is_none()
            || (*MiniportCharacteristics).SetInformationHandler.is_none())
            && (*MiniportCharacteristics).CoRequestHandler.is_none()
        {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Bad miniport characteristics. (Set/Query)\n\0".as_ptr() as PCSTR,
            );
            return NDIS_STATUS_BAD_CHARACTERISTICS;
        }
    }

    if (*MiniportCharacteristics).MajorNdisVersion as ::core::ffi::c_int
        == 0x03 as ::core::ffi::c_int
    {
        if (*MiniportCharacteristics).SendHandler.is_none() {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Bad miniport characteristics. (NDIS 3.0)\n\0".as_ptr() as PCSTR,
            );
            return NDIS_STATUS_BAD_CHARACTERISTICS;
        }
    } else if (*MiniportCharacteristics).MajorNdisVersion as ::core::ffi::c_int
        == 0x04 as ::core::ffi::c_int
    {
        /* NDIS 4.0 */
        if (*MiniportCharacteristics).SendHandler.is_none()
            && (*MiniportCharacteristics).SendPacketsHandler.is_none()
        {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Bad miniport characteristics. (NDIS 4.0)\n\0".as_ptr() as PCSTR,
            );
            return NDIS_STATUS_BAD_CHARACTERISTICS;
        }
    } else if (*MiniportCharacteristics).MajorNdisVersion as ::core::ffi::c_int
        == 0x05 as ::core::ffi::c_int
    {
        /* TODO: Add more checks here */

        if (*MiniportCharacteristics).SendHandler.is_none()
            && (*MiniportCharacteristics).SendPacketsHandler.is_none()
            && (*MiniportCharacteristics).CoSendPacketsHandler.is_none()
        {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Bad miniport characteristics. (NDIS 5.0)\n\0".as_ptr() as PCSTR,
            );
            return NDIS_STATUS_BAD_CHARACTERISTICS;
        }
    }

    RtlCopyMemory!(
        core::ptr::addr_of_mut!((*Miniport).MiniportCharacteristics) as *mut ::core::ffi::c_void,
        MiniportCharacteristics as *const ::core::ffi::c_void,
        MinSize as usize,
    );

    /*
     * NOTE: This is VERY unoptimal! Should we store the NDIS_M_DRIVER_BLOCK
     * structure in the driver extension or what?
     */

    Status = IoAllocateDriverObjectExtension(
        (*Miniport).DriverObject,
        core::mem::size_of::<PVOID>() as PVOID,
        core::mem::size_of::<PNDIS_M_DRIVER_BLOCK>() as ULONG,
        core::ptr::addr_of_mut!(MiniportPtr) as *mut PVOID,
    );
    if !NT_SUCCESS!(Status) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Can't allocate driver object extension.\n\0".as_ptr() as PCSTR,
        );
        return NDIS_STATUS_RESOURCES;
    }

    *MiniportPtr = Miniport;

    /* We have to register for all of these so handler registered in NdisMRegisterDevice work */
    i = 0 as ULONG;
    while i <= IRP_MJ_MAXIMUM_FUNCTION as ULONG {
        (*(*Miniport).DriverObject).MajorFunction[i as usize] =
            Some(NdisGenericIrpHandler);
        i = i.wrapping_add(1);
    }

    (*(*(*Miniport).DriverObject).DriverExtension).AddDevice =
        Some(NdisIAddDevice);

    return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NdisMResetComplete(
    MiniportAdapterHandle: NDIS_HANDLE,
    Status: NDIS_STATUS,
    AddressingReset: BOOLEAN,
) {
    MiniResetComplete(MiniportAdapterHandle, Status, AddressingReset);
}

#[no_mangle]
pub unsafe extern "system" fn NdisMSendComplete(
    MiniportAdapterHandle: NDIS_HANDLE,
    Packet: PNDIS_PACKET,
    Status: NDIS_STATUS,
) {
    MiniSendComplete(MiniportAdapterHandle, Packet, Status);
}

#[no_mangle]
pub unsafe extern "system" fn NdisMSendResourcesAvailable(MiniportAdapterHandle: NDIS_HANDLE) {
    MiniSendResourcesAvailable(MiniportAdapterHandle);
}

#[no_mangle]
pub unsafe extern "system" fn NdisMTransferDataComplete(
    MiniportAdapterHandle: NDIS_HANDLE,
    Packet: PNDIS_PACKET,
    Status: NDIS_STATUS,
    BytesTransferred: UINT,
) {
    MiniTransferDataComplete(MiniportAdapterHandle, Packet, Status, BytesTransferred);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMSetAttributes(
    MiniportAdapterHandle: NDIS_HANDLE,
    MiniportAdapterContext: NDIS_HANDLE,
    BusMaster: BOOLEAN,
    AdapterType: NDIS_INTERFACE_TYPE,
) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);
    NdisMSetAttributesEx(
        MiniportAdapterHandle,
        MiniportAdapterContext,
        0,
        if BusMaster != 0 { NDIS_ATTRIBUTE_BUS_MASTER } else { 0 },
        AdapterType,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisMSetAttributesEx(
    MiniportAdapterHandle: NDIS_HANDLE,
    MiniportAdapterContext: NDIS_HANDLE,
    CheckForHangTimeInSeconds: UINT,
    AttributeFlags: ULONG,
    AdapterType: NDIS_INTERFACE_TYPE,
) {
    let mut Adapter: PLOGICAL_ADAPTER = GET_LOGICAL_ADAPTER!(MiniportAdapterHandle) as PLOGICAL_ADAPTER;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    (*Adapter).NdisMiniportBlock.MiniportAdapterContext = MiniportAdapterContext;
    (*Adapter).NdisMiniportBlock.Flags = AttributeFlags;
    (*Adapter).NdisMiniportBlock.AdapterType = AdapterType;
    if CheckForHangTimeInSeconds > 0 {
        (*Adapter).NdisMiniportBlock.CheckForHangSeconds = CheckForHangTimeInSeconds;
    }
    if (AttributeFlags & NDIS_ATTRIBUTE_INTERMEDIATE_DRIVER) != 0 {
        NDIS_DbgPrint!(MIN_TRACE, b"Intermediate drivers not supported yet.\n\0".as_ptr() as *const i8);
    }

    NDIS_DbgPrint!(MID_TRACE, b"Miniport attribute flags: 0x%x\n\0".as_ptr() as *const i8, AttributeFlags);

    if (*(*Adapter).NdisMiniportBlock.DriverHandle)
        .MiniportCharacteristics
        .AdapterShutdownHandler
        .is_some()
    {
        NDIS_DbgPrint!(MAX_TRACE, b"Miniport set AdapterShutdownHandler in MiniportCharacteristics\n\0".as_ptr() as *const i8);
        NdisMRegisterAdapterShutdownHandler(
            Adapter as NDIS_HANDLE,
            (*Adapter).NdisMiniportBlock.MiniportAdapterContext as PVOID,
            (*(*Adapter).NdisMiniportBlock.DriverHandle)
                .MiniportCharacteristics
                .AdapterShutdownHandler as ADAPTER_SHUTDOWN_HANDLER,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisMSleep(MicrosecondsToSleep: ULONG) {
    let mut Timer: KTIMER = core::mem::zeroed();
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();

    // PAGED_CODE();
    if KeGetCurrentIrql() as ::core::ffi::c_int > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        RtlAssert(
            b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/network/ndis/ndis/miniport.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            3026 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    // A relative due time: negative, in 100 ns units. The C computed `(-1) * 10 * Microseconds`
    // in 32-bit unsigned arithmetic, so it widened to a large POSITIVE value, an absolute time
    // long past, and the timer fired at once: NdisMSleep never slept. Drivers that poll their
    // hardware with it (Realtek's WDI Wi-Fi driver while it starts the chip) then time out at
    // once, and succeed only when something else happens to slow them down.
    DueTime.QuadPart = -(MicrosecondsToSleep as LONGLONG) * 10;

    KeInitializeTimer(&mut Timer);
    KeSetTimer(&mut Timer, DueTime, core::ptr::null_mut());
    KeWaitForSingleObject(
        core::ptr::addr_of_mut!(Timer) as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as KPROCESSOR_MODE,
        FALSE as BOOLEAN,
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisMSynchronizeWithInterrupt(
    Interrupt: PNDIS_MINIPORT_INTERRUPT,
    SynchronizeFunction: PVOID,
    SynchronizeContext: PVOID,
) -> BOOLEAN {
    return KeSynchronizeExecution(
        (*Interrupt).InterruptObject,
        core::mem::transmute::<PVOID, PKSYNCHRONIZE_ROUTINE>(SynchronizeFunction),
        SynchronizeContext,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisMWriteLogData(
    LogHandle: NDIS_HANDLE,
    LogBuffer: PVOID,
    LogBufferSize: UINT,
) -> NDIS_STATUS {
    let mut Buffer: PUCHAR = LogBuffer as PUCHAR;
    let mut i: UINT;
    let mut j: UINT;
    let mut idx: UINT;

    UNIMPLEMENTED!();
    i = 0;
    while i < LogBufferSize {
        DbgPrint(b"%08x |\0".as_ptr() as PCSTR, i);
        j = 0;
        while j < 16 {
            idx = i.wrapping_add(j);
            if idx < LogBufferSize {
                DbgPrint(
                    b" %02x\0".as_ptr() as PCSTR,
                    *Buffer.offset(idx as isize) as core::ffi::c_int,
                );
            } else {
                DbgPrint(b"   \0".as_ptr() as PCSTR);
            }
            j = j.wrapping_add(1);
        }
        DbgPrint(b" | \0".as_ptr() as PCSTR);
        j = 0;
        while j < 16 {
            idx = i.wrapping_add(j);
            if idx == LogBufferSize {
                break;
            }
            if *Buffer.offset(idx as isize) as core::ffi::c_int >= ' ' as i32 {
                DbgPrint(
                    b"%c\0".as_ptr() as PCSTR,
                    *Buffer.offset(idx as isize) as core::ffi::c_int,
                );
            } else {
                DbgPrint(b".\0".as_ptr() as PCSTR);
            }
            j = j.wrapping_add(1);
        }
        DbgPrint(b"\n\0".as_ptr() as PCSTR);
        i = i.wrapping_add(16);
    }

    return NDIS_STATUS_FAILURE;
}
#[no_mangle]
pub unsafe extern "system" fn NdisTerminateWrapper(
    NdisWrapperHandle: NDIS_HANDLE,
    SystemSpecific: PVOID,
) {
    let Miniport: PNDIS_M_DRIVER_BLOCK = GET_MINIPORT_DRIVER!(NdisWrapperHandle);

    NDIS_DbgPrint!(MAX_TRACE, (b"Called.\n\0".as_ptr() as PCSTR));

    ExFreePool((*(*Miniport).RegistryPath).Buffer as PVOID);
    ExFreePool((*Miniport).RegistryPath as PVOID);
    ExInterlockedRemoveEntryList!(
        core::ptr::addr_of_mut!((*Miniport).ListEntry),
        core::ptr::addr_of_mut!(MiniportListLock),
    );
    ExFreePool(Miniport as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMQueryAdapterInstanceName(
    AdapterInstanceName: PNDIS_STRING,
    MiniportAdapterHandle: NDIS_HANDLE,
) -> NDIS_STATUS {
    let Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut AdapterName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    AdapterName.Length = 0;
    AdapterName.MaximumLength = (*Adapter).NdisMiniportBlock.MiniportName.MaximumLength;
    AdapterName.Buffer = ExAllocatePool(
        PagedPool,
        AdapterName.MaximumLength as SIZE_T,
    ) as PWCH;
    if AdapterName.Buffer.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources\n\0".as_ptr() as PCSTR,
        );
        return NDIS_STATUS_RESOURCES;
    }

    RtlCopyUnicodeString(
        core::ptr::addr_of_mut!(AdapterName),
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock.MiniportName) as PCUNICODE_STRING,
    );

    *AdapterInstanceName = AdapterName;

    return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NdisDeregisterAdapterShutdownHandler(NdisAdapterHandle: NDIS_HANDLE) {
    NdisMDeregisterAdapterShutdownHandler(NdisAdapterHandle);
}

#[no_mangle]
pub unsafe extern "system" fn NdisRegisterAdapterShutdownHandler(
    NdisAdapterHandle: NDIS_HANDLE,
    ShutdownContext: PVOID,
    ShutdownHandler: ADAPTER_SHUTDOWN_HANDLER,
) {
    NdisMRegisterAdapterShutdownHandler(NdisAdapterHandle, ShutdownContext, ShutdownHandler);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMGetDeviceProperty(
    MiniportAdapterHandle: NDIS_HANDLE,
    PhysicalDeviceObject: *mut PDEVICE_OBJECT,
    FunctionalDeviceObject: *mut PDEVICE_OBJECT,
    NextDeviceObject: *mut PDEVICE_OBJECT,
    AllocatedResources: *mut PCM_RESOURCE_LIST,
    AllocatedResourcesTranslated: *mut PCM_RESOURCE_LIST,
) {
    let Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;

    NDIS_DbgPrint!(MAX_TRACE, b"Called\n\0" as *const u8 as PCSTR);

    if !PhysicalDeviceObject.is_null() {
        *PhysicalDeviceObject = (*Adapter).NdisMiniportBlock.PhysicalDeviceObject;
    }

    if !FunctionalDeviceObject.is_null() {
        *FunctionalDeviceObject = (*Adapter).NdisMiniportBlock.DeviceObject;
    }

    if !NextDeviceObject.is_null() {
        *NextDeviceObject = (*Adapter).NdisMiniportBlock.NextDeviceObject;
    }

    if !AllocatedResources.is_null() {
        *AllocatedResources = (*Adapter).NdisMiniportBlock.AllocatedResources;
    }

    if !AllocatedResourcesTranslated.is_null() {
        *AllocatedResourcesTranslated = (*Adapter).NdisMiniportBlock.AllocatedResourcesTranslated;
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisMRegisterUnloadHandler(NdisWrapperHandle: NDIS_HANDLE, UnloadHandler: PDRIVER_UNLOAD) {
    let DriverBlock: PNDIS_M_DRIVER_BLOCK = NdisWrapperHandle as PNDIS_M_DRIVER_BLOCK;
    NDIS_DbgPrint!(MAX_TRACE, b"Miniport registered unload handler\n\0".as_ptr() as *const i8);
    (*(*DriverBlock).DriverObject).DriverUnload = UnloadHandler;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMRegisterDevice(NdisWrapperHandle: NDIS_HANDLE, DeviceName: PNDIS_STRING, SymbolicName: PNDIS_STRING, MajorFunctions: *mut PDRIVER_DISPATCH, pDeviceObject: *mut PDEVICE_OBJECT, NdisDeviceHandle: *mut NDIS_HANDLE) -> NDIS_STATUS {
    let DriverBlock: PNDIS_M_DRIVER_BLOCK = NdisWrapperHandle as PNDIS_M_DRIVER_BLOCK;
    let mut DeviceBlock: PNDIS_M_DEVICE_BLOCK = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Status: NDIS_STATUS = 0;
    let mut i: UINT = 0;

    NDIS_DbgPrint!(MAX_TRACE, b"Called\n\0".as_ptr() as PCSTR);

    Status = IoCreateDevice(
        (*DriverBlock).DriverObject,
        core::mem::size_of::<NDIS_M_DEVICE_BLOCK>() as ULONG,
        DeviceName as PUNICODE_STRING,
        FILE_DEVICE_NETWORK as ULONG,
        0 as ULONG,
        FALSE as BOOLEAN,
        core::ptr::addr_of_mut!(DeviceObject),
    ) as NDIS_STATUS;

    if !NT_SUCCESS!(Status as NTSTATUS) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"IoCreateDevice failed (%x)\n\0".as_ptr() as PCSTR,
            Status,
        );
        return Status;
    }

    Status = IoCreateSymbolicLink(
        SymbolicName as PUNICODE_STRING,
        DeviceName as PUNICODE_STRING,
    ) as NDIS_STATUS;

    if !NT_SUCCESS!(Status as NTSTATUS) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"IoCreateSymbolicLink failed (%x)\n\0".as_ptr() as PCSTR,
            Status,
        );
        IoDeleteDevice(DeviceObject);
        return Status;
    }

    DeviceBlock = (*DeviceObject).DeviceExtension as PNDIS_M_DEVICE_BLOCK;

    if DeviceBlock.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources\n\0".as_ptr() as PCSTR,
        );
        IoDeleteDevice(DeviceObject);
        IoDeleteSymbolicLink(SymbolicName as PUNICODE_STRING);
        return NDIS_STATUS_RESOURCES as NDIS_STATUS;
    }

    i = 0 as UINT;
    while i <= IRP_MJ_MAXIMUM_FUNCTION as UINT {
        (*DeviceBlock).MajorFunction[i as usize] = *MajorFunctions.offset(i as isize);
        i = i.wrapping_add(1);
    }

    (*DeviceBlock).DeviceObject = DeviceObject;
    (*DeviceBlock).SymbolicName = SymbolicName;

    *pDeviceObject = DeviceObject;
    *NdisDeviceHandle = DeviceBlock as NDIS_HANDLE;

    return NDIS_STATUS_SUCCESS as NDIS_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMDeregisterDevice(NdisDeviceHandle: NDIS_HANDLE) -> NDIS_STATUS {
    let mut DeviceBlock: PNDIS_M_DEVICE_BLOCK = NdisDeviceHandle as PNDIS_M_DEVICE_BLOCK;

    IoDeleteDevice((*DeviceBlock).DeviceObject);

    IoDeleteSymbolicLink((*DeviceBlock).SymbolicName as PUNICODE_STRING);

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisQueryAdapterInstanceName(
    AdapterInstanceName: PNDIS_STRING,
    NdisBindingHandle: NDIS_HANDLE,
) -> NDIS_STATUS {
    let AdapterBinding: PADAPTER_BINDING = NdisBindingHandle as PADAPTER_BINDING;
    let Adapter: PLOGICAL_ADAPTER = (*AdapterBinding).Adapter;

    return NdisMQueryAdapterInstanceName(AdapterInstanceName, Adapter as NDIS_HANDLE);
}
#[no_mangle]
pub unsafe extern "system" fn NdisCompletePnPEvent(
    Status: NDIS_STATUS,
    NdisBindingHandle: NDIS_HANDLE,
    NetPnPEvent: PNET_PNP_EVENT,
) {
    let mut Irp: PIRP = (*NetPnPEvent).NdisReserved[0 as usize] as PIRP;
    let mut CurrentEntry: PLIST_ENTRY =
        (*NetPnPEvent).NdisReserved[1 as usize] as PLIST_ENTRY;
    let mut AdapterBinding: PADAPTER_BINDING = NdisBindingHandle as PADAPTER_BINDING;
    let mut Adapter: PLOGICAL_ADAPTER = (*AdapterBinding).Adapter;
    let mut NdisStatus: NDIS_STATUS;

    if Status != NDIS_STATUS_SUCCESS {
        if !(*NetPnPEvent).Buffer.is_null() {
            ExFreePool((*NetPnPEvent).Buffer);
        }
        ExFreePool(NetPnPEvent as PVOID);
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status as NTSTATUS;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return;
    }

    while CurrentEntry != &raw mut (*Adapter).ProtocolListHead {
        AdapterBinding = CONTAINING_RECORD!(CurrentEntry, ADAPTER_BINDING, AdapterListEntry);

        let PnPHandler = (*(*AdapterBinding).ProtocolBinding)
            .Chars
            .__bindgen_anon_1.PnPEventHandler
            .unwrap_unchecked();
        NdisStatus = PnPHandler(
            (*AdapterBinding).NdisOpenBlock.ProtocolBindingContext,
            NetPnPEvent,
        );

        if NdisStatus == NDIS_STATUS_PENDING {
            (*NetPnPEvent).NdisReserved[1 as usize] =
                (*CurrentEntry).Flink as ULONG_PTR;
            return;
        } else if NdisStatus != NDIS_STATUS_SUCCESS {
            if !(*NetPnPEvent).Buffer.is_null() {
                ExFreePool((*NetPnPEvent).Buffer);
            }
            ExFreePool(NetPnPEvent as PVOID);
            (*Irp).IoStatus.__bindgen_anon_1.Status = NdisStatus as NTSTATUS;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return;
        }

        CurrentEntry = (*CurrentEntry).Flink as PLIST_ENTRY;
    }

    if !(*NetPnPEvent).Buffer.is_null() {
        ExFreePool((*NetPnPEvent).Buffer);
    }
    ExFreePool(NetPnPEvent as PVOID);

    (*Irp).IoStatus.__bindgen_anon_1.Status = NDIS_STATUS_SUCCESS as NTSTATUS;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
}
#[no_mangle]
pub unsafe extern "system" fn NdisCancelSendPackets(
    NdisBindingHandle: NDIS_HANDLE,
    CancelId: PVOID,
) {
    let AdapterBinding: PADAPTER_BINDING = NdisBindingHandle as PADAPTER_BINDING;
    let Adapter: PLOGICAL_ADAPTER = (*AdapterBinding).Adapter;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Called for ID %x.\n\0".as_ptr() as *const i8,
        CancelId,
    );

    if (*(*Adapter).NdisMiniportBlock.DriverHandle)
        .MiniportCharacteristics
        .CancelSendPacketsHandler
        .is_some()
    {
        Some(
            (*(*Adapter).NdisMiniportBlock.DriverHandle)
                .MiniportCharacteristics
                .CancelSendPacketsHandler
                .unwrap_unchecked(),
        )
        .unwrap_unchecked()(
            (*Adapter).NdisMiniportBlock.MiniportAdapterContext,
            CancelId,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisIMGetBindingContext(NdisBindingHandle: NDIS_HANDLE) -> NDIS_HANDLE {
    let mut AdapterBinding: PADAPTER_BINDING = NdisBindingHandle as PADAPTER_BINDING;
    let mut Adapter: PLOGICAL_ADAPTER = (*AdapterBinding).Adapter;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    return (*Adapter).NdisMiniportBlock.DeviceContext;
}
#[no_mangle]
pub unsafe extern "system" fn NdisIMGetDeviceContext(
    MiniportAdapterHandle: NDIS_HANDLE,
) -> NDIS_HANDLE {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    return (*Adapter).NdisMiniportBlock.DeviceContext;
}
