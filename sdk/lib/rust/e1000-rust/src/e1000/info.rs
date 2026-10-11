use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut SupportedOidList: [NDIS_OID; 32] = [
    OID_GEN_SUPPORTED_LIST as NDIS_OID,
    OID_GEN_CURRENT_PACKET_FILTER as NDIS_OID,
    OID_GEN_HARDWARE_STATUS as NDIS_OID,
    OID_GEN_MEDIA_SUPPORTED as NDIS_OID,
    OID_GEN_MEDIA_IN_USE as NDIS_OID,
    OID_GEN_MAXIMUM_LOOKAHEAD as NDIS_OID,
    OID_GEN_MAXIMUM_FRAME_SIZE as NDIS_OID,
    OID_GEN_MAXIMUM_SEND_PACKETS as NDIS_OID,
    OID_GEN_LINK_SPEED as NDIS_OID,
    OID_GEN_TRANSMIT_BUFFER_SPACE as NDIS_OID,
    OID_GEN_RECEIVE_BUFFER_SPACE as NDIS_OID,
    OID_GEN_RECEIVE_BLOCK_SIZE as NDIS_OID,
    OID_GEN_TRANSMIT_BLOCK_SIZE as NDIS_OID,
    OID_GEN_VENDOR_ID as NDIS_OID,
    OID_GEN_VENDOR_DESCRIPTION as NDIS_OID,
    OID_GEN_VENDOR_DRIVER_VERSION as NDIS_OID,
    OID_GEN_CURRENT_LOOKAHEAD as NDIS_OID,
    OID_802_3_MULTICAST_LIST as NDIS_OID,
    OID_GEN_DRIVER_VERSION as NDIS_OID,
    OID_GEN_MAXIMUM_TOTAL_SIZE as NDIS_OID,
    OID_GEN_MAC_OPTIONS as NDIS_OID,
    OID_GEN_MEDIA_CONNECT_STATUS as NDIS_OID,
    OID_802_3_PERMANENT_ADDRESS as NDIS_OID,
    OID_802_3_CURRENT_ADDRESS as NDIS_OID,
    OID_802_3_MAXIMUM_LIST_SIZE as NDIS_OID,
    OID_GEN_PHYSICAL_MEDIUM as NDIS_OID,
    OID_GEN_XMIT_OK as NDIS_OID,
    OID_GEN_RCV_OK as NDIS_OID,
    OID_GEN_XMIT_ERROR as NDIS_OID,
    OID_GEN_RCV_ERROR as NDIS_OID,
    OID_GEN_RCV_NO_BUFFER as NDIS_OID,
    OID_PNP_CAPABILITIES as NDIS_OID,
];

// Values inlined from ntddndis.h; the name of each OID is kept as a trailing comment.

unsafe extern "C" fn NICQueryStatisticCounter(
    Adapter: PE1000_ADAPTER,
    Oid: NDIS_OID,
) -> ULONG64 {
    0
}

unsafe extern "C" fn NICFillPowerManagementCapabilities(
    Adapter: PE1000_ADAPTER,
    Capabilities: PNDIS_PNP_CAPABILITIES,
) -> NDIS_STATUS {
    NDIS_STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn MiniportQueryInformation(MiniportAdapterContext: NDIS_HANDLE, Oid: NDIS_OID, InformationBuffer: PVOID, InformationBufferLength: ULONG, BytesWritten: PULONG, BytesNeeded: PULONG) -> NDIS_STATUS {
    let Adapter: PE1000_ADAPTER = MiniportAdapterContext as PE1000_ADAPTER;
    let mut copyLength: ULONG = 0;
    let mut copySource: PVOID = core::ptr::null_mut();
    let mut status: NDIS_STATUS;
    let mut GenericInfo: _GENERIC_INFORMATION = core::mem::zeroed();

    status = NDIS_STATUS_SUCCESS;
    copySource = &raw mut GenericInfo as PVOID;
    copyLength = core::mem::size_of::<ULONG>() as ULONG;

    match Oid {
        OID_GEN_SUPPORTED_LIST => {
            copySource = &raw mut SupportedOidList as PVOID;
            copyLength = core::mem::size_of_val(&SupportedOidList) as ULONG;
        }

        OID_GEN_CURRENT_PACKET_FILTER => {
            GenericInfo.Ulong = (*Adapter).PacketFilter;
        }

        OID_GEN_HARDWARE_STATUS => {
            UNIMPLEMENTED_DBGBREAK!();
            GenericInfo.Ulong = NdisHardwareStatusReady as ULONG; //FIXME
        }

        OID_GEN_MEDIA_SUPPORTED | OID_GEN_MEDIA_IN_USE => {
            GenericInfo.Medium = NdisMedium802_3;
            copyLength = core::mem::size_of::<NDIS_MEDIUM>() as ULONG;
        }

        OID_GEN_RECEIVE_BLOCK_SIZE
        | OID_GEN_TRANSMIT_BLOCK_SIZE
        | OID_GEN_CURRENT_LOOKAHEAD
        | OID_GEN_MAXIMUM_LOOKAHEAD
        | OID_GEN_MAXIMUM_FRAME_SIZE => {
            GenericInfo.Ulong = (MAXIMUM_FRAME_SIZE as usize)
                .wrapping_sub(core::mem::size_of::<ETH_HEADER>())
                as ULONG;
        }

        OID_802_3_MULTICAST_LIST => {
            copySource = &raw mut (*Adapter).MulticastList as PVOID;
            copyLength = (*Adapter)
                .MulticastListSize
                .wrapping_mul(IEEE_802_ADDR_LENGTH as ULONG);
        }

        OID_802_3_MAXIMUM_LIST_SIZE => {
            GenericInfo.Ulong = MAXIMUM_MULTICAST_ADDRESSES as ULONG;
        }

        OID_GEN_LINK_SPEED => {
            GenericInfo.Ulong = (*Adapter)
                .LinkSpeedMbps
                .wrapping_mul(10000 as ULONG);
        }

        OID_GEN_TRANSMIT_BUFFER_SPACE => {
            GenericInfo.Ulong = MAXIMUM_FRAME_SIZE as ULONG;
        }

        OID_GEN_RECEIVE_BUFFER_SPACE => {
            GenericInfo.Ulong = RECEIVE_BUFFER_SIZE as ULONG;
        }

        OID_GEN_VENDOR_ID => {
            /* The 3 bytes of the MAC address is the vendor ID */
            GenericInfo.Ulong = 0 as ULONG;
            GenericInfo.Ulong |= ((*Adapter).PermanentMacAddress[0 as usize]
                as ULONG)
                << 16;
            GenericInfo.Ulong |= ((*Adapter).PermanentMacAddress[1 as usize]
                as ULONG)
                << 8;
            GenericInfo.Ulong |= ((*Adapter).PermanentMacAddress[2 as usize]
                as ULONG
                & 0xFF as ULONG);
        }

        OID_GEN_VENDOR_DESCRIPTION => {
            static mut vendorDesc: [UCHAR; 13] = *b"ReactOS Team\0";
            copySource = &raw mut vendorDesc as *mut UCHAR as PVOID;
            copyLength = core::mem::size_of::<[UCHAR; 13]>() as ULONG;
        }

        OID_GEN_VENDOR_DRIVER_VERSION => {
            GenericInfo.Ulong = DRIVER_VERSION as ULONG;
        }

        OID_GEN_DRIVER_VERSION => {
            copyLength = core::mem::size_of::<USHORT>() as ULONG;
            GenericInfo.Ushort = ((NDIS_MINIPORT_MAJOR_VERSION << 8)
                + NDIS_MINIPORT_MINOR_VERSION) as USHORT;
        }

        OID_GEN_MAXIMUM_TOTAL_SIZE => {
            GenericInfo.Ulong = MAXIMUM_FRAME_SIZE as ULONG;
        }

        OID_GEN_MAXIMUM_SEND_PACKETS => {
            GenericInfo.Ulong = 1 as ULONG;
        }

        OID_GEN_MAC_OPTIONS => {
            GenericInfo.Ulong = (NDIS_MAC_OPTION_RECEIVE_SERIALIZED
                | NDIS_MAC_OPTION_COPY_LOOKAHEAD_DATA
                | NDIS_MAC_OPTION_TRANSFERS_NOT_PEND
                | NDIS_MAC_OPTION_NO_LOOPBACK) as ULONG;
        }

        OID_GEN_MEDIA_CONNECT_STATUS => {
            GenericInfo.Ulong = (*Adapter).MediaState;
        }

        OID_802_3_CURRENT_ADDRESS => {
            copySource = &raw mut (*Adapter).MulticastList[0 as usize]
                .MacAddress as *mut UCHAR as PVOID;
            copyLength = IEEE_802_ADDR_LENGTH as ULONG;
        }

        OID_802_3_PERMANENT_ADDRESS => {
            copySource = &raw mut (*Adapter).PermanentMacAddress as *mut UCHAR as PVOID;
            copyLength = IEEE_802_ADDR_LENGTH as ULONG;
        }

        OID_GEN_XMIT_OK
        | OID_GEN_RCV_OK
        | OID_GEN_XMIT_ERROR
        | OID_GEN_RCV_ERROR
        | OID_GEN_RCV_NO_BUFFER => {
            GenericInfo.Ulong64 = NICQueryStatisticCounter(Adapter, Oid);

            *BytesNeeded = core::mem::size_of::<ULONG64>() as ULONG;
            if InformationBufferLength as usize >= core::mem::size_of::<ULONG64>()
            {
                *BytesWritten = core::mem::size_of::<ULONG64>() as ULONG;
                NdisMoveMemory!(
                    InformationBuffer,
                    copySource,
                    core::mem::size_of::<ULONG64>(),
                );
            } else if InformationBufferLength as usize >= core::mem::size_of::<ULONG>() {
                *BytesWritten = core::mem::size_of::<ULONG>() as ULONG;
                NdisMoveMemory!(
                    InformationBuffer,
                    copySource,
                    core::mem::size_of::<ULONG>(),
                );
            } else {
                *BytesWritten = 0 as ULONG;
                return NDIS_STATUS_BUFFER_TOO_SHORT;
            }
            return NDIS_STATUS_SUCCESS;
        }

        OID_PNP_CAPABILITIES => {
            copyLength = core::mem::size_of::<NDIS_PNP_CAPABILITIES>() as ULONG;

            status = NICFillPowerManagementCapabilities(Adapter, &raw mut GenericInfo.PmCapabilities);
        }

        OID_GEN_PHYSICAL_MEDIUM => {
            GenericInfo.Ulong = NdisPhysicalMedium802_3 as ULONG;
        }

        _ => {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Unknown OID 0x%x(%s)\n\0".as_ptr() as PCSTR,
                Oid,
                Oid2Str(Oid),
            );
            status = NDIS_STATUS_NOT_SUPPORTED;
        }
    }

    if status == NDIS_STATUS_SUCCESS {
        if copyLength > InformationBufferLength {
            *BytesNeeded = copyLength;
            *BytesWritten = 0 as ULONG;
            status = NDIS_STATUS_BUFFER_TOO_SHORT;
        } else {
            NdisMoveMemory!(InformationBuffer, copySource, copyLength as usize);
            *BytesWritten = copyLength;
            *BytesNeeded = copyLength;
        }
    } else {
        *BytesWritten = 0 as ULONG;
        *BytesNeeded = 0 as ULONG;
    }

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Query OID 0x%x(%s): Completed with status 0x%x (%d, %d)\n\0".as_ptr() as PCSTR,
        Oid,
        Oid2Str(Oid),
        status,
        *BytesWritten,
        *BytesNeeded,
    );
    return status;
}
#[no_mangle]
pub unsafe extern "system" fn MiniportSetInformation(mut MiniportAdapterContext: NDIS_HANDLE, mut Oid: NDIS_OID, mut InformationBuffer: PVOID, mut InformationBufferLength: ULONG, mut BytesRead: PULONG, mut BytesNeeded: PULONG) -> NDIS_STATUS {
    let mut Adapter: PE1000_ADAPTER = MiniportAdapterContext as PE1000_ADAPTER;
    let mut genericUlong: ULONG = 0;
    let mut status: NDIS_STATUS;

    status = NDIS_STATUS_SUCCESS;

    match Oid {
        OID_GEN_CURRENT_PACKET_FILTER => {
            if (InformationBufferLength as usize) < core::mem::size_of::<ULONG>() {
                *BytesRead = 0;
                *BytesNeeded = core::mem::size_of::<ULONG>() as ULONG;
                status = NDIS_STATUS_INVALID_LENGTH;
                // break
            } else {
                NdisMoveMemory!(
                    &raw mut genericUlong as *mut core::ffi::c_void,
                    InformationBuffer as *const core::ffi::c_void,
                    core::mem::size_of::<ULONG>(),
                );

                if (genericUlong
                    & !((NDIS_PACKET_TYPE_DIRECTED
                        | NDIS_PACKET_TYPE_MULTICAST
                        | NDIS_PACKET_TYPE_ALL_MULTICAST
                        | NDIS_PACKET_TYPE_BROADCAST
                        | NDIS_PACKET_TYPE_PROMISCUOUS
                        | NDIS_PACKET_TYPE_MAC_FRAME) as ULONG))
                    != 0
                {
                    *BytesRead = core::mem::size_of::<ULONG>() as ULONG;
                    *BytesNeeded = core::mem::size_of::<ULONG>() as ULONG;
                    status = NDIS_STATUS_NOT_SUPPORTED;
                    // break
                } else if (*Adapter).PacketFilter == genericUlong {
                    // break
                } else {
                    (*Adapter).PacketFilter = genericUlong;

                    status = NICApplyPacketFilter(Adapter);
                    if status != NDIS_STATUS_SUCCESS {
                        NDIS_DbgPrint!(
                            MIN_TRACE,
                            b"Failed to apply new packet filter (0x%x)\n\0".as_ptr()
                                as *const core::ffi::c_char,
                            status,
                        );
                    }
                    // break
                }
            }
        }

        OID_GEN_CURRENT_LOOKAHEAD => {
            if (InformationBufferLength as usize) < core::mem::size_of::<ULONG>() {
                *BytesRead = 0;
                *BytesNeeded = core::mem::size_of::<ULONG>() as ULONG;
                status = NDIS_STATUS_INVALID_LENGTH;
                // break
            } else {
                NdisMoveMemory!(
                    &raw mut genericUlong as *mut core::ffi::c_void,
                    InformationBuffer as *const core::ffi::c_void,
                    core::mem::size_of::<ULONG>(),
                );

                if (genericUlong as usize)
                    > (MAXIMUM_FRAME_SIZE as usize)
                        .wrapping_sub(core::mem::size_of::<ETH_HEADER>())
                {
                    status = NDIS_STATUS_INVALID_DATA;
                } else {
                    // Ignore this...
                }
                // break
            }
        }

        OID_802_3_MULTICAST_LIST => {
            if InformationBufferLength % IEEE_802_ADDR_LENGTH != 0 {
                *BytesRead = 0;
                *BytesNeeded = InformationBufferLength
                    + (InformationBufferLength % IEEE_802_ADDR_LENGTH);
                status = NDIS_STATUS_INVALID_LENGTH;
                // break
            } else if (InformationBufferLength as usize)
                > core::mem::size_of::<[C2RustUnnamed_123; 16]>()
            {
                *BytesNeeded = core::mem::size_of::<[C2RustUnnamed_123; 16]>() as ULONG;
                *BytesRead = 0;
                status = NDIS_STATUS_MULTICAST_FULL;
                // break
            } else {
                NdisMoveMemory!(
                    &raw mut (*Adapter).MulticastList as *mut core::ffi::c_void,
                    InformationBuffer as *const core::ffi::c_void,
                    InformationBufferLength as usize,
                );

                (*Adapter).MulticastListSize =
                    InformationBufferLength / IEEE_802_ADDR_LENGTH;

                NICUpdateMulticastList(Adapter);
                // break
            }
        }

        _ => {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Unknown OID 0x%x(%s)\n\0".as_ptr() as *const core::ffi::c_char,
                Oid,
                Oid2Str(Oid),
            );
            status = NDIS_STATUS_NOT_SUPPORTED;
            *BytesRead = 0;
            *BytesNeeded = 0;
            // break
        }
    }

    if status == NDIS_STATUS_SUCCESS {
        *BytesRead = InformationBufferLength;
        *BytesNeeded = 0;
    }

    return status;
}
