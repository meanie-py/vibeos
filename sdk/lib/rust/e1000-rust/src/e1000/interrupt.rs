use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn MiniportISR(InterruptRecognized: PBOOLEAN, QueueMiniportHandleInterrupt: PBOOLEAN, MiniportAdapterContext: NDIS_HANDLE) {
    let mut Value: ULONG = 0;
    let mut Adapter: PE1000_ADAPTER = MiniportAdapterContext as PE1000_ADAPTER;

    /* Reading the interrupt acknowledges them */
    E1000ReadUlong(Adapter, E1000_REG_ICR, core::ptr::addr_of_mut!(Value));

    Value &= (*Adapter).InterruptMask as ULONG;
    _InterlockedOr(core::ptr::addr_of_mut!((*Adapter).InterruptPending), Value as core::ffi::c_long);

    if Value != 0 {
        *InterruptRecognized = TRUE;
        /* Mark the events pending service */
        *QueueMiniportHandleInterrupt = TRUE;
    } else {
        /* This is not ours. */
        *InterruptRecognized = FALSE;
        *QueueMiniportHandleInterrupt = FALSE;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiniportHandleInterrupt(MiniportAdapterContext: NDIS_HANDLE) {
    let mut InterruptPending: ULONG;
    let mut Adapter: PE1000_ADAPTER = MiniportAdapterContext as PE1000_ADAPTER;
    let mut TransmitDescriptor: PE1000_TRANSMIT_DESCRIPTOR = core::ptr::null_mut();

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    InterruptPending = (_InterlockedExchange(
        core::ptr::addr_of_mut!((*Adapter).InterruptPending),
        (0 as LONG),
    ) as ULONG);

    /* Link State Changed */
    if InterruptPending & (E1000_IMS_LSC as ULONG) != 0 {
        let mut Status: ULONG = 0;

        InterruptPending &= !(E1000_IMS_LSC as ULONG);
        NDIS_DbgPrint!(MAX_TRACE, b"Link status changed!.\n\0".as_ptr() as *const i8);

        NICUpdateLinkStatus(Adapter);

        Status = if (*Adapter).MediaState == (NdisMediaStateConnected as ULONG) {
            (NDIS_STATUS_MEDIA_CONNECT as ULONG)
        } else {
            (NDIS_STATUS_MEDIA_DISCONNECT as ULONG)
        };

        NdisMIndicateStatus!(
            (*Adapter).AdapterHandle,
            Status as NDIS_STATUS,
            core::ptr::null_mut(),
            (0 as UINT),
        );
        NdisMIndicateStatusComplete!((*Adapter).AdapterHandle);
    }

    /* Handling receive interrupts */
    if InterruptPending & ((E1000_IMS_RXDMT0 as ULONG) | (E1000_IMS_RXT0 as ULONG)) != 0 {
        let mut ReceiveDescriptor: PE1000_RECEIVE_DESCRIPTOR = core::ptr::null_mut();
        let mut EthHeader: PETH_HEADER = core::ptr::null_mut();
        let mut BufferOffset: ULONG = 0;
        let mut bGotAny: BOOLEAN = (FALSE as BOOLEAN);
        let mut RxDescHead: ULONG = 0;
        let mut RxDescTail: ULONG = 0;
        let mut CurrRxDesc: ULONG = 0;

        /* Clear out these interrupts */
        InterruptPending &= !((E1000_IMS_RXDMT0 as ULONG) | (E1000_IMS_RXT0 as ULONG));

        E1000ReadUlong(
            Adapter,
            (E1000_REG_RDH as ULONG),
            core::ptr::addr_of_mut!(RxDescHead),
        );
        E1000ReadUlong(
            Adapter,
            (E1000_REG_RDT as ULONG),
            core::ptr::addr_of_mut!(RxDescTail),
        );

        while (RxDescTail.wrapping_add(1 as ULONG))
            .wrapping_rem(NUM_RECEIVE_DESCRIPTORS as ULONG)
            != RxDescHead
        {
            CurrRxDesc = (RxDescTail.wrapping_add(1 as ULONG))
                .wrapping_rem(NUM_RECEIVE_DESCRIPTORS as ULONG);
            BufferOffset = CurrRxDesc.wrapping_mul((*Adapter).ReceiveBufferEntrySize);
            core::ptr::write_volatile(
                core::ptr::addr_of_mut!(ReceiveDescriptor),
                (*Adapter).ReceiveDescriptors.offset(CurrRxDesc as isize),
            );

            /* Check if the hardware have released this descriptor (DD - Descriptor Done) */
            if ((*ReceiveDescriptor).Status as ULONG) & (E1000_RDESC_STATUS_DD as ULONG) == 0 {
                /* No need to check descriptors after the first unfinished one */
                break;
            }

            /* Ignoring these flags for now */
            (*ReceiveDescriptor).Status = (((*ReceiveDescriptor).Status as ULONG)
                & !((E1000_RDESC_STATUS_IXSM as ULONG) | (E1000_RDESC_STATUS_PIF as ULONG)))
                as UCHAR;

            if (*ReceiveDescriptor).Status as ULONG
                != ((E1000_RDESC_STATUS_EOP as ULONG) | (E1000_RDESC_STATUS_DD as ULONG))
            {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"Unrecognized ReceiveDescriptor status flag: %u\n\0".as_ptr() as *const i8,
                    (*ReceiveDescriptor).Status as ULONG,
                );
            }

            'next: {
                /* Make sure the receive indications are enabled */
                if (*Adapter).PacketFilter == 0 {
                    break 'next;
                }

                if ((*ReceiveDescriptor).Length as ULONG) != (0 as ULONG)
                    && (*ReceiveDescriptor).Address != (0 as UINT64)
                {
                    EthHeader = (*Adapter).ReceiveBuffer.offset(BufferOffset as isize) as PETH_HEADER;

                    NdisMEthIndicateReceive!(
                        (*Adapter).AdapterHandle,
                        core::ptr::null_mut(),
                        EthHeader as PCHAR,
                        (core::mem::size_of::<ETH_HEADER>() as usize) as UINT,
                        EthHeader.offset(1 as isize) as PCHAR,
                        ((*ReceiveDescriptor).Length as usize)
                            .wrapping_sub((core::mem::size_of::<ETH_HEADER>() as usize)) as UINT,
                        ((*ReceiveDescriptor).Length as usize)
                            .wrapping_sub((core::mem::size_of::<ETH_HEADER>() as usize)) as UINT,
                    );

                    bGotAny = (TRUE as BOOLEAN);
                } else {
                    NDIS_DbgPrint!(
                        MIN_TRACE,
                        b"Got a NULL descriptor\0".as_ptr() as *const i8,
                    );
                }
            }

            /* Give the descriptor back */
            (*ReceiveDescriptor).Status = (0 as UCHAR);

            RxDescTail = CurrRxDesc;
        }

        if bGotAny != 0 {
            /* Write back new tail value */
            E1000WriteUlong(Adapter, (E1000_REG_RDT as ULONG), RxDescTail);

            NDIS_DbgPrint!(
                MAX_TRACE,
                b"Rx done (RDH: %u, RDT: %u)\n\0".as_ptr() as *const i8,
                RxDescHead,
                RxDescTail,
            );

            NdisMEthIndicateReceiveComplete!((*Adapter).AdapterHandle);
        }
    }

    /* Handling transmit interrupts */
    if InterruptPending
        & ((E1000_IMS_TXD_LOW as ULONG)
            | (E1000_IMS_TXDW as ULONG)
            | (E1000_IMS_TXQE as ULONG))
        != 0
    {
        let mut AckPackets: [PNDIS_PACKET; 40 as usize] =
            [core::ptr::null_mut::<_NDIS_PACKET>(); 40 as usize];
        let mut NumPackets: ULONG = 0 as ULONG;
        let mut i: ULONG;

        /* Clear out these interrupts */
        InterruptPending &= !((E1000_IMS_TXD_LOW as ULONG)
            | (E1000_IMS_TXDW as ULONG)
            | (E1000_IMS_TXQE as ULONG));

        while ((*Adapter).TxFull != 0 || (*Adapter).LastTxDesc != (*Adapter).CurrentTxDesc)
            && NumPackets < ARRAYSIZE!(AckPackets) as ULONG
        {
            core::ptr::write_volatile(
                core::ptr::addr_of_mut!(TransmitDescriptor),
                (*Adapter).TransmitDescriptors.offset((*Adapter).LastTxDesc as isize),
            );

            if (*TransmitDescriptor).Status as ULONG & (E1000_TDESC_STATUS_DD as ULONG) != 0 {
                if !(*Adapter).TransmitPackets[(*Adapter).LastTxDesc as usize].is_null() {
                    let fresh = NumPackets;
                    NumPackets = NumPackets.wrapping_add(1);
                    AckPackets[fresh as usize] =
                        (*Adapter).TransmitPackets[(*Adapter).LastTxDesc as usize];
                    (*Adapter).TransmitPackets[(*Adapter).LastTxDesc as usize] =
                        core::ptr::null_mut::<_NDIS_PACKET>();
                    (*TransmitDescriptor).Status = (0 as UCHAR);
                }

                (*Adapter).LastTxDesc = (*Adapter)
                    .LastTxDesc
                    .wrapping_add(1 as ULONG)
                    .wrapping_rem(NUM_TRANSMIT_DESCRIPTORS as ULONG);
                (*Adapter).TxFull = (FALSE as BOOLEAN);
            } else {
                break;
            }
        }

        if NumPackets != 0 {
            NDIS_DbgPrint!(
                MAX_TRACE,
                b"Tx: (TDH: %u, TDT: %u)\n\0".as_ptr() as *const i8,
                (*Adapter).CurrentTxDesc,
                (*Adapter).LastTxDesc,
            );
            NDIS_DbgPrint!(
                MAX_TRACE,
                b"Tx Done: %u packets to ack\n\0".as_ptr() as *const i8,
                NumPackets,
            );

            i = 0 as ULONG;
            while i < NumPackets {
                NdisMSendComplete!(
                    (*Adapter).AdapterHandle,
                    AckPackets[i as usize],
                    (NDIS_STATUS_SUCCESS as NDIS_STATUS),
                );
                i = i.wrapping_add(1);
            }
        }
    }

    ASSERT!(InterruptPending == (0 as ULONG));
}
