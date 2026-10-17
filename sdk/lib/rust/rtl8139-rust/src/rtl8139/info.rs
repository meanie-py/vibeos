use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut SupportedOidList: [ULONG; 37] = [
    OID_GEN_SUPPORTED_LIST as ULONG,
    OID_GEN_HARDWARE_STATUS as ULONG,
    OID_GEN_MEDIA_SUPPORTED as ULONG,
    OID_GEN_MEDIA_IN_USE as ULONG,
    OID_GEN_MAXIMUM_LOOKAHEAD as ULONG,
    OID_GEN_MAXIMUM_FRAME_SIZE as ULONG,
    OID_GEN_LINK_SPEED as ULONG,
    OID_GEN_TRANSMIT_BUFFER_SPACE as ULONG,
    OID_GEN_RECEIVE_BUFFER_SPACE as ULONG,
    OID_GEN_RECEIVE_BLOCK_SIZE as ULONG,
    OID_GEN_TRANSMIT_BLOCK_SIZE as ULONG,
    OID_GEN_VENDOR_ID as ULONG,
    OID_GEN_VENDOR_DESCRIPTION as ULONG,
    OID_GEN_VENDOR_DRIVER_VERSION as ULONG,
    OID_GEN_CURRENT_PACKET_FILTER as ULONG,
    OID_GEN_CURRENT_LOOKAHEAD as ULONG,
    OID_GEN_DRIVER_VERSION as ULONG,
    OID_GEN_MAXIMUM_TOTAL_SIZE as ULONG,
    OID_GEN_PROTOCOL_OPTIONS as ULONG,
    OID_GEN_MAC_OPTIONS as ULONG,
    OID_GEN_MEDIA_CONNECT_STATUS as ULONG,
    OID_GEN_MAXIMUM_SEND_PACKETS as ULONG,
    OID_GEN_XMIT_OK as ULONG,
    OID_GEN_RCV_OK as ULONG,
    OID_GEN_XMIT_ERROR as ULONG,
    OID_GEN_RCV_ERROR as ULONG,
    OID_GEN_RCV_NO_BUFFER as ULONG,
    OID_GEN_RCV_CRC_ERROR as ULONG,
    OID_802_3_PERMANENT_ADDRESS as ULONG,
    OID_802_3_CURRENT_ADDRESS as ULONG,
    OID_802_3_MULTICAST_LIST as ULONG,
    OID_802_3_MAXIMUM_LIST_SIZE as ULONG,
    OID_802_3_MAC_OPTIONS as ULONG,
    OID_802_3_RCV_ERROR_ALIGNMENT as ULONG,
    OID_802_3_XMIT_ONE_COLLISION as ULONG,
    OID_802_3_XMIT_MORE_COLLISIONS as ULONG,
    OID_GEN_PHYSICAL_MEDIUM as ULONG,
];

#[no_mangle]
pub unsafe extern "system" fn MiniportQueryInformation(MiniportAdapterContext: NDIS_HANDLE, Oid: NDIS_OID, InformationBuffer: PVOID, InformationBufferLength: ULONG, BytesWritten: PULONG, BytesNeeded: PULONG) -> NDIS_STATUS {
    let adapter: PRTL_ADAPTER = MiniportAdapterContext as PRTL_ADAPTER;
    let mut genericUlong: ULONG = 0;
    let mut copyLength: ULONG = 0;
    let mut copySource: PVOID = core::ptr::null_mut();
    let mut status: NDIS_STATUS = 0;

    status = NDIS_STATUS_SUCCESS;
    copySource = core::ptr::addr_of_mut!(genericUlong) as PVOID;
    copyLength = core::mem::size_of::<ULONG>() as ULONG;

    NdisAcquireSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));

    match Oid {
        OID_GEN_SUPPORTED_LIST => {
            copySource = core::ptr::addr_of_mut!(SupportedOidList) as PVOID;
            copyLength = core::mem::size_of::<[ULONG; 37]>() as ULONG;
        }
        OID_GEN_CURRENT_PACKET_FILTER => {
            genericUlong = (*adapter).PacketFilter;
        }
        OID_GEN_HARDWARE_STATUS => {
            genericUlong = NdisHardwareStatusReady as ULONG; //FIXME
        }
        OID_GEN_MEDIA_SUPPORTED | OID_GEN_MEDIA_IN_USE => {
            static mut medium: NDIS_MEDIUM = NdisMedium802_3;
            copySource = core::ptr::addr_of!(medium) as PVOID;
            copyLength = core::mem::size_of::<NDIS_MEDIUM>() as ULONG;
        }
        OID_GEN_RECEIVE_BLOCK_SIZE | OID_GEN_TRANSMIT_BLOCK_SIZE | OID_GEN_CURRENT_LOOKAHEAD | OID_GEN_MAXIMUM_LOOKAHEAD | OID_GEN_MAXIMUM_FRAME_SIZE => {
            genericUlong = (MAXIMUM_FRAME_SIZE as usize)
                .wrapping_sub(core::mem::size_of::<ETH_HEADER>()) as ULONG;
        }
        OID_GEN_LINK_SPEED => {
            genericUlong = (*adapter).LinkSpeedMbps.wrapping_mul(1000 as ULONG);
        }
        OID_GEN_TRANSMIT_BUFFER_SPACE => {
            genericUlong = MAXIMUM_FRAME_SIZE as ULONG;
        }
        OID_GEN_RECEIVE_BUFFER_SPACE => {
            genericUlong = RECEIVE_BUFFER_SIZE as ULONG;
        }
        OID_GEN_VENDOR_ID => {
            // The 3 bytes of the MAC address is the vendor ID
            genericUlong = 0 as ULONG;
            genericUlong |= ((*adapter).PermanentMacAddress[0 as usize] as ULONG) << 16;
            genericUlong |= ((*adapter).PermanentMacAddress[1 as usize] as ULONG) << 8;
            genericUlong |= ((*adapter).PermanentMacAddress[2 as usize] as ULONG & 0xFF);
        }
        OID_GEN_VENDOR_DESCRIPTION => {
            static mut vendorDesc: [UCHAR; 13] = *b"ReactOS Team\0";
            copySource = core::ptr::addr_of_mut!(vendorDesc) as *mut UCHAR as PVOID;
            copyLength = core::mem::size_of::<[UCHAR; 13]>() as ULONG;
        }
        OID_GEN_VENDOR_DRIVER_VERSION => {
            genericUlong = DRIVER_VERSION as ULONG;
        }
        OID_GEN_DRIVER_VERSION => {
            static mut driverVersion: USHORT =
                ((NDIS_MINIPORT_MAJOR_VERSION << 8) + NDIS_MINIPORT_MINOR_VERSION) as USHORT;
            copySource = core::ptr::addr_of!(driverVersion) as PVOID;
            copyLength = core::mem::size_of::<USHORT>() as ULONG;
        }
        OID_GEN_MAXIMUM_TOTAL_SIZE => {
            genericUlong = MAXIMUM_FRAME_SIZE as ULONG;
        }
        OID_GEN_PROTOCOL_OPTIONS => {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"OID_GEN_PROTOCOL_OPTIONS is unimplemented\n\0".as_ptr() as *const i8,
            );
            status = NDIS_STATUS_NOT_SUPPORTED;
        }
        OID_GEN_MAC_OPTIONS => {
            genericUlong = (NDIS_MAC_OPTION_RECEIVE_SERIALIZED
                | NDIS_MAC_OPTION_COPY_LOOKAHEAD_DATA
                | NDIS_MAC_OPTION_TRANSFERS_NOT_PEND
                | NDIS_MAC_OPTION_NO_LOOPBACK) as ULONG;
        }
        OID_GEN_MEDIA_CONNECT_STATUS => {
            genericUlong = (*adapter).MediaState;
        }
        OID_GEN_MAXIMUM_SEND_PACKETS => {
            genericUlong = 1 as ULONG;
        }
        OID_802_3_CURRENT_ADDRESS => {
            copySource = core::ptr::addr_of_mut!((*adapter).CurrentMacAddress) as *mut UCHAR as PVOID;
            copyLength = IEEE_802_ADDR_LENGTH as ULONG;
        }
        OID_802_3_PERMANENT_ADDRESS => {
            copySource = core::ptr::addr_of_mut!((*adapter).PermanentMacAddress) as *mut UCHAR as PVOID;
            copyLength = IEEE_802_ADDR_LENGTH as ULONG;
        }
        OID_802_3_MAXIMUM_LIST_SIZE => {
            genericUlong = MAXIMUM_MULTICAST_ADDRESSES as ULONG;
        }
        OID_GEN_PHYSICAL_MEDIUM => {
            genericUlong = NdisPhysicalMedium802_3 as ULONG;
        }
        OID_GEN_XMIT_OK => {
            genericUlong = (*adapter).TransmitOk;
        }
        OID_GEN_RCV_OK => {
            genericUlong = (*adapter).ReceiveOk;
        }
        OID_GEN_XMIT_ERROR => {
            genericUlong = (*adapter).TransmitError;
        }
        OID_GEN_RCV_ERROR => {
            genericUlong = (*adapter).ReceiveError;
        }
        OID_GEN_RCV_NO_BUFFER => {
            genericUlong = (*adapter).ReceiveNoBufferSpace;
        }
        OID_GEN_RCV_CRC_ERROR => {
            genericUlong = (*adapter).ReceiveCrcError;
        }
        OID_802_3_RCV_ERROR_ALIGNMENT => {
            genericUlong = (*adapter).ReceiveAlignmentError;
        }
        OID_802_3_XMIT_ONE_COLLISION => {
            genericUlong = (*adapter).TransmitOneCollision;
        }
        OID_802_3_XMIT_MORE_COLLISIONS => {
            genericUlong = (*adapter).TransmitMoreCollisions;
        }
        _ => {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Unknown OID\n\0".as_ptr() as *const i8,
            );
            status = NDIS_STATUS_NOT_SUPPORTED;
        }
    }

    if status == NDIS_STATUS_SUCCESS {
        if copyLength > InformationBufferLength {
            *BytesNeeded = copyLength;
            *BytesWritten = 0 as ULONG;
            status = NDIS_STATUS_INVALID_LENGTH;
        } else {
            NdisMoveMemory!(InformationBuffer, copySource, copyLength);
            *BytesWritten = copyLength;
            *BytesNeeded = copyLength;
        }
    } else {
        *BytesWritten = 0 as ULONG;
        *BytesNeeded = 0 as ULONG;
    }

    NdisReleaseSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Query OID 0x%x: Completed with status 0x%x (%d, %d)\n\0".as_ptr() as *const i8,
        Oid,
        status,
        *BytesWritten,
        *BytesNeeded,
    );

    return status;
}
#[no_mangle]
pub unsafe extern "system" fn MiniportSetInformation(MiniportAdapterContext: NDIS_HANDLE, Oid: NDIS_OID, InformationBuffer: PVOID, InformationBufferLength: ULONG, BytesRead: PULONG, BytesNeeded: PULONG) -> NDIS_STATUS {
    let adapter: PRTL_ADAPTER = MiniportAdapterContext as PRTL_ADAPTER;
    let mut genericUlong: ULONG = 0;
    let mut status: NDIS_STATUS = NDIS_STATUS_SUCCESS;

    NdisAcquireSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));

    'sw: {
        match Oid {
            OID_GEN_CURRENT_PACKET_FILTER => {
                if InformationBufferLength < core::mem::size_of::<ULONG>() as ULONG {
                    *BytesRead = 0;
                    *BytesNeeded = core::mem::size_of::<ULONG>() as ULONG;
                    status = NDIS_STATUS_INVALID_LENGTH;
                    break 'sw;
                }

                NdisMoveMemory!(
                    core::ptr::addr_of_mut!(genericUlong) as PVOID,
                    InformationBuffer,
                    core::mem::size_of::<ULONG>() as usize,
                );

                if (genericUlong
                    & (NDIS_PACKET_TYPE_ALL_FUNCTIONAL
                        | NDIS_PACKET_TYPE_FUNCTIONAL
                        | NDIS_PACKET_TYPE_GROUP
                        | NDIS_PACKET_TYPE_MAC_FRAME
                        | NDIS_PACKET_TYPE_SMT
                        | NDIS_PACKET_TYPE_SOURCE_ROUTING))
                    != 0
                {
                    *BytesRead = core::mem::size_of::<ULONG>() as ULONG;
                    *BytesNeeded = core::mem::size_of::<ULONG>() as ULONG;
                    status = NDIS_STATUS_NOT_SUPPORTED;
                    break 'sw;
                }

                (*adapter).PacketFilter = genericUlong;

                status = NICApplyPacketFilter(adapter);
                if status != NDIS_STATUS_SUCCESS {
                    NDIS_DbgPrint!(
                        MIN_TRACE,
                        b"Failed to apply new packet filter\n\0".as_ptr() as PCSTR,
                    );
                    break 'sw;
                }

                break 'sw;
            }

            OID_GEN_CURRENT_LOOKAHEAD => {
                if InformationBufferLength < core::mem::size_of::<ULONG>() as ULONG {
                    *BytesRead = 0;
                    *BytesNeeded = core::mem::size_of::<ULONG>() as ULONG;
                    status = NDIS_STATUS_INVALID_LENGTH;
                    break 'sw;
                }

                NdisMoveMemory!(
                    core::ptr::addr_of_mut!(genericUlong) as PVOID,
                    InformationBuffer,
                    core::mem::size_of::<ULONG>() as usize,
                );

                if (genericUlong as usize)
                    > (MAXIMUM_FRAME_SIZE as usize)
                        .wrapping_sub(core::mem::size_of::<ETH_HEADER>() as usize)
                {
                    status = NDIS_STATUS_INVALID_DATA;
                } else {
                    // Ignore this...
                }

                break 'sw;
            }

            OID_802_3_MULTICAST_LIST => {
                if InformationBufferLength % IEEE_802_ADDR_LENGTH != 0 {
                    *BytesRead = 0;
                    *BytesNeeded = InformationBufferLength
                        + (InformationBufferLength % IEEE_802_ADDR_LENGTH);
                    status = NDIS_STATUS_INVALID_LENGTH;
                    break 'sw;
                }

                if InformationBufferLength / 6 > MAXIMUM_MULTICAST_ADDRESSES {
                    *BytesNeeded = MAXIMUM_MULTICAST_ADDRESSES * IEEE_802_ADDR_LENGTH;
                    *BytesRead = 0;
                    status = NDIS_STATUS_INVALID_LENGTH;
                    break 'sw;
                }

                NdisMoveMemory!(
                    core::ptr::addr_of_mut!((*adapter).MulticastList) as PVOID,
                    InformationBuffer,
                    InformationBufferLength as usize,
                );

                // FIXME: Write to device

                break 'sw;
            }

            _ => {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"Unknown OID\n\0".as_ptr() as PCSTR,
                );
                status = NDIS_STATUS_NOT_SUPPORTED;
                *BytesRead = 0;
                *BytesNeeded = 0;
                break 'sw;
            }
        }
    }

    if status == NDIS_STATUS_SUCCESS {
        *BytesRead = InformationBufferLength;
        *BytesNeeded = 0;
    }

    NdisReleaseSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));

    return status;
}
