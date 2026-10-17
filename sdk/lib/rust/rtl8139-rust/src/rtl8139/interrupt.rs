use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn MiniportISR(InterruptRecognized: PBOOLEAN, QueueMiniportHandleInterrupt: PBOOLEAN, MiniportAdapterContext: NDIS_HANDLE) {
    let adapter: PRTL_ADAPTER = MiniportAdapterContext as PRTL_ADAPTER;
    let mut csConfig: ULONG = 0;

    (*adapter).InterruptPending |= NICInterruptRecognized(adapter, InterruptRecognized) as USHORT;
    if !(*InterruptRecognized == 0) {
    } else {
        *QueueMiniportHandleInterrupt = FALSE as BOOLEAN;
        return;
    }

    if (*adapter).InterruptPending as ULONG & R_I_RXUNDRUN as ULONG != 0 {
        NdisRawReadPortUlong!(
            (*adapter).IoBase.offset(R_CSCFG as isize) as ULONG,
            core::ptr::addr_of_mut!(csConfig),
        );
        if csConfig & R_CSCR_LINKCHNG as ULONG != 0 {
            (*adapter).LinkChange = TRUE as BOOLEAN;
            NICUpdateLinkStatus(adapter);
        }
    }

    NICAcknowledgeInterrupts(adapter);
    *QueueMiniportHandleInterrupt = TRUE as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn MiniportHandleInterrupt(MiniportAdapterContext: NDIS_HANDLE) {
    let adapter: PRTL_ADAPTER = MiniportAdapterContext as PRTL_ADAPTER;
    let mut txStatus: ULONG = 0;
    let mut command: UCHAR = 0;
    let mut nicHeader: PPACKET_HEADER = core::ptr::null_mut();
    let mut ethHeader: PETH_HEADER = core::ptr::null_mut();

    NdisDprAcquireSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Interrupts pending: 0x%x\n\0".as_ptr() as *const i8,
        (*adapter).InterruptPending as ::core::ffi::c_int,
    );

    //
    // Handle a link change
    //
    if (*adapter).LinkChange != 0 {
        NdisDprReleaseSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));
        NdisMIndicateStatus!(
            (*adapter).MiniportAdapterHandle,
            if (*adapter).MediaState == NdisMediaStateConnected {
                NDIS_STATUS_MEDIA_CONNECT
            } else {
                NDIS_STATUS_MEDIA_DISCONNECT
            },
            core::ptr::null_mut(),
            0,
        );
        NdisMIndicateStatusComplete!((*adapter).MiniportAdapterHandle);
        NdisDprAcquireSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));
        (*adapter).LinkChange = FALSE;
    }

    //
    // Handle a TX interrupt
    //
    if ((*adapter).InterruptPending as ULONG & (R_I_TXOK | R_I_TXERR) as ULONG) != 0 {
        while (*adapter).TxFull != 0
            || (*adapter).DirtyTxDesc != (*adapter).CurrentTxDesc
        {
            NdisRawReadPortUlong!(
                (*adapter)
                    .IoBase
                    .offset(R_TXSTS0 as isize)
                    .offset(((*adapter).DirtyTxDesc as usize * core::mem::size_of::<ULONG>()) as isize),
                core::ptr::addr_of_mut!(txStatus),
            );

            if (txStatus & (R_TXS_STATOK | R_TXS_UNDERRUN | R_TXS_ABORTED) as ULONG) == 0 {
                //
                // Not sent yet
                //
                break;
            }

            NDIS_DbgPrint!(
                MAX_TRACE,
                b"Transmission for desc %d complete: 0x%x\n\0".as_ptr() as *const i8,
                (*adapter).DirtyTxDesc as ::core::ffi::c_int,
                txStatus,
            );

            if (txStatus & R_TXS_STATOK as ULONG) != 0 {
                (*adapter).TransmitOk = (*adapter).TransmitOk.wrapping_add(1);
            } else {
                (*adapter).TransmitError = (*adapter).TransmitError.wrapping_add(1);
            }

            (*adapter).DirtyTxDesc = (*adapter).DirtyTxDesc.wrapping_add(1);
            (*adapter).DirtyTxDesc =
                ((*adapter).DirtyTxDesc as ULONG % TX_DESC_COUNT as ULONG) as UCHAR;
            (*adapter).InterruptPending = ((*adapter).InterruptPending as ULONG
                & !((R_I_TXOK | R_I_TXERR) as ULONG)) as USHORT;
            (*adapter).TxFull = FALSE;
        }
    }

    //
    // Handle a good RX interrupt
    //
    if ((*adapter).InterruptPending as ULONG & (R_I_RXOK | R_I_RXERR) as ULONG) != 0 {
        loop {
            NdisRawReadPortUchar!(
                (*adapter).IoBase.offset(R_CMD as isize),
                core::ptr::addr_of_mut!(command),
            );
            if (command as ULONG & R_CMD_RXEMPTY as ULONG) != 0 {
                //
                // The buffer is empty
                //
                (*adapter).InterruptPending = ((*adapter).InterruptPending as ULONG
                    & !((R_I_RXOK | R_I_RXERR) as ULONG)) as USHORT;
                break;
            }

            (*adapter).ReceiveOffset =
                ((*adapter).ReceiveOffset as ULONG % RECEIVE_BUFFER_SIZE as ULONG) as USHORT;

            NDIS_DbgPrint!(
                MAX_TRACE,
                b"Looking for a packet at offset 0x%x\n\0".as_ptr() as *const i8,
                (*adapter).ReceiveOffset as ::core::ffi::c_int,
            );
            nicHeader = (*adapter)
                .ReceiveBuffer
                .offset((*adapter).ReceiveOffset as isize) as PPACKET_HEADER;

            //
            // NextPacket is the shared tail; the labeled block is the jump scope
            //
            'next_packet: {
                if ((*nicHeader).Status as ULONG & RSR_ROK as ULONG) == 0 {
                    //
                    // Receive failed
                    //
                    NDIS_DbgPrint!(
                        MIN_TRACE,
                        b"Receive failed: 0x%x\n\0".as_ptr() as *const i8,
                        (*nicHeader).Status as ::core::ffi::c_int,
                    );

                    if ((*nicHeader).Status as ULONG & RSR_FAE as ULONG) != 0 {
                        (*adapter).ReceiveAlignmentError =
                            (*adapter).ReceiveAlignmentError.wrapping_add(1);
                    } else if ((*nicHeader).Status as ULONG & RSR_CRC as ULONG) != 0 {
                        (*adapter).ReceiveCrcError = (*adapter).ReceiveCrcError.wrapping_add(1);
                    }
                    (*adapter).ReceiveError = (*adapter).ReceiveError.wrapping_add(1);

                    break 'next_packet;
                }

                NDIS_DbgPrint!(
                    MAX_TRACE,
                    b"Indicating %d byte packet to NDIS\n\0".as_ptr() as *const i8,
                    (*nicHeader).PacketLength as ::core::ffi::c_int
                        - RECV_CRC_LENGTH as ::core::ffi::c_int,
                );

                ethHeader = nicHeader.offset(1 as isize) as PETH_HEADER;
                NdisMEthIndicateReceive!(
                    (*adapter).MiniportAdapterHandle,
                    core::ptr::null_mut(),
                    ethHeader as PVOID,
                    core::mem::size_of::<ETH_HEADER>() as UINT,
                    ethHeader.offset(1 as isize) as PVOID,
                    ((*nicHeader).PacketLength as usize)
                        .wrapping_sub(core::mem::size_of::<ETH_HEADER>())
                        .wrapping_sub(RECV_CRC_LENGTH as usize) as UINT,
                    ((*nicHeader).PacketLength as usize)
                        .wrapping_sub(core::mem::size_of::<ETH_HEADER>())
                        .wrapping_sub(RECV_CRC_LENGTH as usize) as UINT,
                );
                (*adapter).ReceiveOk = (*adapter).ReceiveOk.wrapping_add(1);
            }

            // NextPacket:
            (*adapter).ReceiveOffset = ((*adapter).ReceiveOffset as ULONG)
                .wrapping_add((*nicHeader).PacketLength as ULONG)
                .wrapping_add(core::mem::size_of::<PACKET_HEADER>() as ULONG)
                as USHORT;
            (*adapter).ReceiveOffset = (((*adapter).ReceiveOffset as ULONG)
                .wrapping_add(3 as ULONG)
                & !(3 as ULONG)) as USHORT;
            NdisRawWritePortUshort!(
                (*adapter).IoBase.offset(R_CAPR as isize),
                ((*adapter).ReceiveOffset as ULONG).wrapping_sub(0x10 as ULONG)
                    as USHORT,
            );

            if ((*adapter).InterruptPending as ULONG
                & (R_I_RXOVRFLW | R_I_FIFOOVR) as ULONG)
                != 0
            {
                //
                // We can only clear these interrupts once CAPR has been reset
                //
                NdisRawWritePortUshort!(
                    (*adapter).IoBase.offset(R_IS as isize),
                    (R_I_RXOVRFLW | R_I_FIFOOVR) as USHORT,
                );
                (*adapter).InterruptPending = ((*adapter).InterruptPending as ULONG
                    & !((R_I_RXOVRFLW | R_I_FIFOOVR) as ULONG))
                    as USHORT;
            }
        }

        NdisMEthIndicateReceiveComplete!((*adapter).MiniportAdapterHandle);
    }

    NdisDprReleaseSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));
}
