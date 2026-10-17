use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn NICPowerOn(Adapter: PRTL_ADAPTER) -> NDIS_STATUS {
    NdisRawWritePortUchar!((*Adapter).IoBase.add(R_CFG1 as usize), 0x00 as UCHAR);
    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NICSoftReset(Adapter: PRTL_ADAPTER) -> NDIS_STATUS {
    let mut commandReg: UCHAR = core::mem::zeroed();
    let mut resetAttempts: UINT;
    NdisRawWritePortUchar!(((*Adapter).IoBase).wrapping_add((R_CMD) as usize), B_CMD_RST);
    resetAttempts = 0;
    while resetAttempts < MAX_RESET_ATTEMPTS {
        NdisRawReadPortUchar!(((*Adapter).IoBase).wrapping_add((R_CMD) as usize), &mut commandReg);
        if (commandReg & (B_CMD_RST as u8)) == 0 {
            return NDIS_STATUS_SUCCESS;
        }
        NdisMSleep(100);
        resetAttempts = resetAttempts.wrapping_add(1);
    }
    return NDIS_STATUS_FAILURE;
}
#[no_mangle]
pub unsafe extern "system" fn NICRegisterReceiveBuffer(Adapter: PRTL_ADAPTER) -> NDIS_STATUS {
    ASSERT!(NdisGetPhysicalAddressHigh!((*Adapter).ReceiveBufferPa) == 0);

    NdisRawWritePortUlong!(
        ((*Adapter).IoBase as ULONG_PTR).wrapping_add(R_RXSA as ULONG_PTR),
        (*Adapter).ReceiveBufferPa.__bindgen_anon_1.LowPart,
    );

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NICRemoveReceiveBuffer(Adapter: PRTL_ADAPTER) -> NDIS_STATUS {
    NdisRawWritePortUlong!(
        (*Adapter).IoBase.offset(R_RXSA as isize) as PULONG,
        0 as ULONG,
    );
    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NICEnableTxRx(mut Adapter: PRTL_ADAPTER) -> NDIS_STATUS {
    NdisRawWritePortUchar!(
        (*Adapter).IoBase.offset(R_CMD as isize),
        (B_CMD_TXE | B_CMD_RXE) as UCHAR,
    );

    //
    // TX and RX must be enabled before setting these
    //
    NdisRawWritePortUlong!(
        (*Adapter).IoBase.offset(R_RC as isize) as PULONG,
        RC_VAL as ULONG,
    );
    NdisRawWritePortUlong!(
        (*Adapter).IoBase.offset(R_TC as isize) as PULONG,
        TC_VAL as ULONG,
    );
    return NDIS_STATUS_SUCCESS as NDIS_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn NICGetPermanentMacAddress(
    mut Adapter: PRTL_ADAPTER,
    mut MacAddress: PUCHAR,
) -> NDIS_STATUS {
    let mut i: UINT;

    i = 0;
    while i < IEEE_802_ADDR_LENGTH {
        NdisRawReadPortUchar!(
            ((*Adapter).IoBase).wrapping_add((R_MAC) as usize).wrapping_add(i as usize),
            MacAddress.offset(i as isize),
        );
        i = i.wrapping_add(1);
    }

    return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NICApplyInterruptMask(Adapter: PRTL_ADAPTER) -> NDIS_STATUS {
    NdisRawWritePortUshort!(
        (*Adapter).IoBase.offset(R_IM as isize) as PUSHORT,
        (*Adapter).InterruptMask,
    );
    return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NICDisableInterrupts(mut Adapter: PRTL_ADAPTER) -> NDIS_STATUS {
    NdisRawWritePortUshort!(
        (*Adapter).IoBase.offset(R_IM as isize) as PUSHORT,
        0 as USHORT,
    );
    return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NICInterruptRecognized(
    Adapter: PRTL_ADAPTER,
    InterruptRecognized: PBOOLEAN,
) -> USHORT {
    let mut interruptStatus: USHORT = 0;

    NdisRawReadPortUshort!(
        (*Adapter).IoBase.offset(R_IS as isize) as PUSHORT,
        core::ptr::addr_of_mut!(interruptStatus),
    );

    *InterruptRecognized = ((interruptStatus & (*Adapter).InterruptMask) != 0) as BOOLEAN;

    return (interruptStatus & (*Adapter).InterruptMask);
}
#[no_mangle]
pub unsafe extern "system" fn NICAcknowledgeInterrupts(Adapter: PRTL_ADAPTER) {
    NdisRawWritePortUshort!(
        (*Adapter).IoBase as usize + R_IS as usize,
        (*Adapter).InterruptPending,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NICUpdateLinkStatus(Adapter: PRTL_ADAPTER) {
    let mut mediaState: UCHAR = 0;

    NdisRawReadPortUchar!(
        ((*Adapter).IoBase as ULONG_PTR).wrapping_add((R_MS) as ULONG_PTR),
        core::ptr::addr_of_mut!(mediaState),
    );
    (*Adapter).MediaState = (if (mediaState as ::core::ffi::c_int & (R_MS_LINKDWN) as ::core::ffi::c_int) != 0 {
        (NdisMediaStateDisconnected) as ::core::ffi::c_int
    } else {
        (NdisMediaStateConnected) as ::core::ffi::c_int
    }) as ULONG;
    (*Adapter).LinkSpeedMbps = (if (mediaState as ::core::ffi::c_int & (R_MS_SPEED_10) as ::core::ffi::c_int) != 0 {
        (10) as ::core::ffi::c_int
    } else {
        (100) as ::core::ffi::c_int
    }) as ULONG;
}
#[no_mangle]
pub unsafe extern "system" fn NICApplyPacketFilter(Adapter: PRTL_ADAPTER) -> NDIS_STATUS {
    let mut filterMask: ULONG;

    filterMask = RC_VAL as ULONG;

    if ((*Adapter).PacketFilter & (NDIS_PACKET_TYPE_DIRECTED as ULONG)) != 0 {
        filterMask |= B_RC_APM as ULONG;
    }

    if ((*Adapter).PacketFilter & (NDIS_PACKET_TYPE_MULTICAST as ULONG)) != 0 {
        filterMask |= B_RC_AM as ULONG;
    }

    if ((*Adapter).PacketFilter & (NDIS_PACKET_TYPE_BROADCAST as ULONG)) != 0 {
        filterMask |= B_RC_AB as ULONG;
    }

    if ((*Adapter).PacketFilter & (NDIS_PACKET_TYPE_PROMISCUOUS as ULONG)) != 0 {
        filterMask |= B_RC_AAP as ULONG;
    }

    NdisRawWritePortUlong!(
        ((*Adapter).IoBase as ULONG_PTR).wrapping_add(R_RC as ULONG_PTR),
        filterMask,
    );

    return NDIS_STATUS_SUCCESS as NDIS_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn NICTransmitPacket(
    Adapter: PRTL_ADAPTER,
    TxDesc: UCHAR,
    PhysicalAddress: ULONG,
    Length: ULONG,
) -> NDIS_STATUS {
    NdisRawWritePortUlong!(
        (*Adapter).IoBase.offset(R_TXSAD0 as isize)
            .offset((TxDesc as usize).wrapping_mul(core::mem::size_of::<ULONG>()) as isize)
            as PULONG,
        PhysicalAddress,
    );
    NdisRawWritePortUlong!(
        (*Adapter).IoBase.offset(R_TXSTS0 as isize)
            .offset((TxDesc as usize).wrapping_mul(core::mem::size_of::<ULONG>()) as isize)
            as PULONG,
        Length,
    );
    return NDIS_STATUS_SUCCESS;
}
