use crate::prelude::*;
use ntkrnl_sys::ddk::{PHYSICAL_ADDRESS, PSCATTER_GATHER_LIST};
use core::ffi::c_void;

unsafe extern "C" fn NICTransmitPacket(
    Adapter: PE1000_ADAPTER,
    PhysicalAddress: PHYSICAL_ADDRESS,
    Length: ULONG,
) -> NDIS_STATUS {
    let mut TransmitDescriptor: PE1000_TRANSMIT_DESCRIPTOR = core::ptr::null_mut();

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    TransmitDescriptor = (*Adapter)
        .TransmitDescriptors
        .add((*Adapter).CurrentTxDesc as usize);
    (*TransmitDescriptor).Address = PhysicalAddress.QuadPart as UINT64;
    (*TransmitDescriptor).Length = Length as USHORT;
    (*TransmitDescriptor).ChecksumOffset = 0;
    (*TransmitDescriptor).Command = (E1000_TDESC_CMD_RS | E1000_TDESC_CMD_IFCS
        | E1000_TDESC_CMD_EOP | E1000_TDESC_CMD_IDE) as u8;
    (*TransmitDescriptor).Status = 0;
    (*TransmitDescriptor).ChecksumStartField = 0;
    (*TransmitDescriptor).Special = 0;

    (*Adapter).CurrentTxDesc = (*Adapter)
        .CurrentTxDesc
        .wrapping_add(1)
        % NUM_TRANSMIT_DESCRIPTORS;

    E1000WriteUlong(Adapter, E1000_REG_TDT, (*Adapter).CurrentTxDesc);

    if (*Adapter).CurrentTxDesc == (*Adapter).LastTxDesc {
        NDIS_DbgPrint!(
            MID_TRACE,
            b"All TX descriptors are full now\n\0".as_ptr() as *const i8,
        );
        (*Adapter).TxFull = TRUE;
    }

    return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn MiniportSend(MiniportAdapterContext: NDIS_HANDLE, Packet: PNDIS_PACKET, Flags: UINT) -> NDIS_STATUS {
    let Adapter: PE1000_ADAPTER = MiniportAdapterContext as PE1000_ADAPTER;
    let mut sgList: PSCATTER_GATHER_LIST;
    let mut TransmitLength: ULONG;
    let mut TransmitBuffer: PHYSICAL_ADDRESS;
    let mut Status: NDIS_STATUS;

    sgList = NDIS_PER_PACKET_INFO_FROM_PACKET!(Packet, ScatterGatherListPacketInfo) as PSCATTER_GATHER_LIST;

    ASSERT!(!sgList.is_null());
    ASSERT!((*sgList).NumberOfElements == 1);
    ASSERT!(
        ((*(&raw mut (*sgList).Elements as *mut SCATTER_GATHER_ELEMENT)
            .offset(0 as isize))
        .Address
        
        .__bindgen_anon_1.LowPart
            & 3)
            == 0
    );
    ASSERT!(
        (*(&raw mut (*sgList).Elements as *mut SCATTER_GATHER_ELEMENT)
            .offset(0 as isize))
        .Length
            <= MAXIMUM_FRAME_SIZE
    );

    if (*Adapter).TxFull != 0 {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"All TX descriptors are full\n\0".as_ptr() as PCSTR,
        );
        return NDIS_STATUS_RESOURCES;
    }

    TransmitLength = (*(&raw mut (*sgList).Elements as *mut SCATTER_GATHER_ELEMENT)
        .offset(0 as isize))
    .Length;
    TransmitBuffer = (*(&raw mut (*sgList).Elements as *mut SCATTER_GATHER_ELEMENT)
        .offset(0 as isize))
    .Address;
    (*Adapter).TransmitPackets[(*Adapter).CurrentTxDesc as usize] = Packet;

    Status = NICTransmitPacket(Adapter, TransmitBuffer, TransmitLength);
    if Status != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Transmit packet failed\n\0".as_ptr() as PCSTR,
        );
        return Status;
    }

    return NDIS_STATUS_PENDING;
}
