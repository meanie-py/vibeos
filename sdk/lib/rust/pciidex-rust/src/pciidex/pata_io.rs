use crate::prelude::*;
use ntkrnl_sys::ddk::{KeAcquireInterruptSpinLock, KeInsertQueueDpc, KeReleaseInterruptSpinLock, PKINTERRUPT, SCATTER_GATHER_LIST};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn AtaWriteBlock16(
    Port: *mut u16,
    Buffer: *mut u16,
    Count: u32,
    MmioFlags: u32,
) {
    if MmioFlags != 0 {
        WRITE_REGISTER_BUFFER_USHORT(Port, Buffer, Count);
    } else {
        WRITE_PORT_BUFFER_USHORT(Port, Buffer, Count);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaReadBlock16(
    Port: *mut u16,
    Buffer: *mut u16,
    Count: u32,
    MmioFlags: u32,
) {
    if MmioFlags != 0 {
        READ_REGISTER_BUFFER_USHORT(Port, Buffer, Count);
    } else {
        READ_PORT_BUFFER_USHORT(Port, Buffer, Count);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaWriteBlock32(
    Port: *mut ULONG,
    Buffer: *mut ULONG,
    Count: ULONG,
    MmioFlags: ULONG,
) {
    if MmioFlags != 0 {
        WRITE_REGISTER_BUFFER_ULONG(Port, Buffer, Count);
    } else {
        WRITE_PORT_BUFFER_ULONG(Port, Buffer, Count);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaReadBlock32(
    Port: *mut ULONG,
    Buffer: *mut ULONG,
    Count: ULONG,
    MmioFlags: ULONG,
) {
    if MmioFlags != 0 {
        READ_REGISTER_BUFFER_ULONG(Port, Buffer, Count);
    } else {
        READ_PORT_BUFFER_ULONG(Port, Buffer, Count);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaReadPortUchar(
    Port: *mut u8,
    MmioFlags: u32,
) -> UCHAR {
    if MmioFlags != 0 {
        return READ_REGISTER_UCHAR(Port);
    } else {
        return READ_PORT_UCHAR(Port);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaWritePortUchar(
    Port: *mut u8,
    Value: u8,
    MmioFlags: u32,
) {
    if MmioFlags != 0 {
        WRITE_REGISTER_UCHAR(Port, Value);
    } else {
        WRITE_PORT_UCHAR(Port, Value);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaWritePortUlong(
    Port: *mut ULONG,
    Value: ULONG,
    MmioFlags: ULONG,
) {
    if MmioFlags != 0 {
        WRITE_REGISTER_ULONG(Port, Value);
    } else {
        WRITE_PORT_ULONG(Port, Value);
    }
}
#[no_mangle]
pub unsafe extern "C" fn PataSaveTaskFile(ChanData: PCHANNEL_DATA_PATA, Request: PATA_DEVICE_REQUEST) {
    let TaskFile: PATA_TASKFILE = &raw mut (*Request).Output;

    (*TaskFile).__bindgen_anon_2.Error = ATA_READ!((*ChanData).Regs.__bindgen_anon_1.Error, ChanData, MRES_TF);
    (*TaskFile).SectorCount = ATA_READ!((*ChanData).Regs.__bindgen_anon_2.SectorCount, ChanData, MRES_TF);
    (*TaskFile).LowLba = ATA_READ!((*ChanData).Regs.LbaLow, ChanData, MRES_TF);
    (*TaskFile).MidLba = ATA_READ!((*ChanData).Regs.__bindgen_anon_3.LbaMid, ChanData, MRES_TF);
    (*TaskFile).HighLba = ATA_READ!((*ChanData).Regs.__bindgen_anon_4.LbaHigh, ChanData, MRES_TF);
    (*TaskFile).DriveSelect = ATA_READ!((*ChanData).Regs.Device, ChanData, MRES_TF);
    (*TaskFile).__bindgen_anon_1.Command = ATA_READ!((*ChanData).Regs.__bindgen_anon_5.Command, ChanData, MRES_TF);

    if (*Request).Flags & REQUEST_FLAG_LBA48 != 0 {
        let Control: u8 = ATA_READ!((*ChanData).Regs.__bindgen_anon_6.Control, ChanData, MRES_CTRL) | (IDE_DC_ALWAYS as u8);

        ATA_WRITE!((*ChanData).Regs.__bindgen_anon_6.Control, Control | (IDE_HIGH_ORDER_BYTE as u8), ChanData, MRES_CTRL);

        (*TaskFile).FeatureEx = ATA_READ!((*ChanData).Regs.__bindgen_anon_1.Features, ChanData, MRES_TF);
        (*TaskFile).SectorCountEx = ATA_READ!((*ChanData).Regs.__bindgen_anon_2.SectorCount, ChanData, MRES_TF);
        (*TaskFile).LowLbaEx = ATA_READ!((*ChanData).Regs.LbaLow, ChanData, MRES_TF);
        (*TaskFile).MidLbaEx = ATA_READ!((*ChanData).Regs.__bindgen_anon_3.LbaMid, ChanData, MRES_TF);
        (*TaskFile).HighLbaEx = ATA_READ!((*ChanData).Regs.__bindgen_anon_4.LbaHigh, ChanData, MRES_TF);

        ATA_WRITE!((*ChanData).Regs.__bindgen_anon_6.Control, Control, ChanData, MRES_CTRL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn PataReadStatus(ChanData: PCHANNEL_DATA_PATA) -> UCHAR {
    return ATA_READ!((*ChanData).Regs.__bindgen_anon_1.Error, ChanData, MRES_TF);
}

unsafe extern "C" fn PataCompleteCommand(
    ChanData: PCHANNEL_DATA_PATA,
    IdeStatus: UCHAR,
    SrbStatus: UCHAR,
) {
    let Request: PATA_DEVICE_REQUEST = *(*ChanData).__bindgen_anon_1.Slots.offset(PATA_CHANNEL_SLOT as isize);

    (*Request).SrbStatus = SrbStatus;

    if SrbStatus as i32 != (SRB_STATUS_SUCCESS as i32) && SrbStatus as i32 != (SRB_STATUS_DATA_OVERRUN as i32) {
        if (*(*Request).Device).TransportFlags & DEVICE_IS_ATAPI == 0
            || (*Request).Flags & REQUEST_FLAG_SAVE_TASK_FILE != 0
        {
            (*ChanData).SaveTaskFile.unwrap_unchecked()(ChanData, Request);
            (*Request).Flags |= REQUEST_FLAG_HAS_TASK_FILE;
        } else {
            (*Request).Output.__bindgen_anon_1.Status = IdeStatus;
            (*Request).Output.__bindgen_anon_2.Error = ATA_READ!(
                (*ChanData).Regs.__bindgen_anon_1.Error,
                ChanData,
                MRES_TF,
            );
        }

        (*ChanData)
            .__bindgen_anon_1.PortNotification
            .unwrap_unchecked()(
            AtaRequestFailed,
            (*ChanData).__bindgen_anon_1.PortContext,
            Request,
        );
        return;
    }

    (*ChanData).__bindgen_anon_1.ActiveSlotsBitmap &= !(1 << PATA_CHANNEL_SLOT);

    if (*Request).Flags & REQUEST_FLAG_SAVE_TASK_FILE != 0 {
        (*ChanData).SaveTaskFile.unwrap_unchecked()(ChanData, Request);
        (*Request).Flags |= REQUEST_FLAG_HAS_TASK_FILE;
    }

    (*ChanData)
        .__bindgen_anon_1.PortNotification
        .unwrap_unchecked()(
        AtaRequestComplete,
        (*ChanData).__bindgen_anon_1.PortContext,
        1 << PATA_CHANNEL_SLOT,
    );
}

unsafe extern "C" fn PciIdeDmaReadStatus(ChanData: PCHANNEL_DATA_PATA) -> UCHAR {
    return ATA_READ!(
        (*ChanData).Regs.Dma.add((PCIIDE_DMA_STATUS as usize)),
        ChanData,
        MRES_DMA,
    );
}

unsafe extern "C" fn PciIdeDmaPrepare(mut ChanData: PCHANNEL_DATA_PATA) {
    let mut IoBase: PUCHAR = (*ChanData).Regs.Dma;

    ATA_WRITE!(
        IoBase.offset(PCIIDE_DMA_COMMAND as isize),
        (PCIIDE_DMA_COMMAND_STOP as u8),
        ChanData,
        MRES_DMA,
    );
    ATA_WRITE!(
        IoBase.offset(PCIIDE_DMA_STATUS as isize),
        (PCIIDE_DMA_STATUS_INTERRUPT | PCIIDE_DMA_STATUS_ERROR) as u8,
        ChanData,
        MRES_DMA,
    );

    ATA_WRITE_ULONG!(
        IoBase.offset(PCIIDE_DMA_PRDT_PHYSICAL_ADDRESS as isize) as PULONG,
        (*ChanData).PrdTablePhysicalAddress,
        ChanData,
        MRES_DMA,
    );
}

unsafe extern "C" fn PciIdeDmaStart(
    mut ChanData: PCHANNEL_DATA_PATA,
    mut Request: PATA_DEVICE_REQUEST,
) {
    let mut Command: UCHAR = 0;

    /* Transfer direction */
    if (*Request).Flags & REQUEST_FLAG_DATA_IN != 0 {
        Command = (PCIIDE_DMA_COMMAND_WRITE_TO_SYSTEM_MEMORY as u8);
    } else {
        Command = (PCIIDE_DMA_COMMAND_READ_FROM_SYSTEM_MEMORY as u8);
    }
    Command = (Command as ::core::ffi::c_int | (PCIIDE_DMA_COMMAND_START as i32)) as UCHAR;

    /* Begin transaction */
    ATA_WRITE!(
        (*ChanData).Regs.Dma.offset(PCIIDE_DMA_COMMAND as isize),
        Command,
        ChanData,
        MRES_DMA,
    );
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeDmaStop(ChanData: *mut CHANNEL_DATA_PATA) {
    let IoBase: *mut u8 = (*ChanData).Regs.Dma;

    ATA_READ!(
        IoBase.add((PCIIDE_DMA_STATUS as usize)),
        ChanData,
        MRES_DMA,
    );

    ATA_WRITE!(
        IoBase.add((PCIIDE_DMA_COMMAND as usize)),
        (PCIIDE_DMA_COMMAND_STOP as u8),
        ChanData,
        MRES_DMA,
    );
    ATA_WRITE!(
        IoBase.add((PCIIDE_DMA_STATUS as usize)),
        (PCIIDE_DMA_STATUS_INTERRUPT as u8),
        ChanData,
        MRES_DMA,
    );
}

unsafe extern "C" fn PciIdeDmaClearInterrupt(
    ChanData: PCHANNEL_DATA_PATA,
    DmaStatus: UCHAR,
) {
    ATA_WRITE!(
        (*ChanData).Regs.Dma.offset(PCIIDE_DMA_STATUS as isize),
        DmaStatus,
        ChanData,
        MRES_DMA,
    );
}

unsafe extern "C" fn PataChangeInterruptMode(
    ChanData: PCHANNEL_DATA_PATA,
    Device: PATA_IO_CONTEXT_COMMON,
    Request: PATA_DEVICE_REQUEST,
) {
    let mut Control: UCHAR;

    if (*Request).Flags & REQUEST_FLAG_POLL != 0 {
        (*ChanData).IsPollingActive = TRUE;
        Control = (IDE_DC_DISABLE_INTERRUPTS as u8);
    } else {
        (*ChanData).IsPollingActive = FALSE;
        Control = 0;
    }

    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_6.Control,
        Control | (IDE_DC_ALWAYS as u8),
        ChanData,
        MRES_CTRL,
    );

    ATA_IO_WAIT!();

    if (*(*ChanData).__bindgen_anon_1.Controller).Pci.VendorID == (PCI_VEN_VIA as u16) {
        ATA_SELECT_DEVICE(ChanData, (DEV_NUMBER!(Device)) as UCHAR, (*Device).DeviceSelect);
        ATA_WAIT(
            ChanData,
            ATA_TIME_BUSY_SELECT,
            (IDE_STATUS_BUSY) as UCHAR,
            0,
        );
    }
}

unsafe extern "C" fn PataSendCdb(
    ChanData: PCHANNEL_DATA_PATA,
    Request: PATA_DEVICE_REQUEST,
) {
    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_IO32 != 0 {
        ATA_WRITE_BLOCK_32!(
            (*ChanData).Regs.Data as *mut u32,
            ((*Request).__bindgen_anon_2.Cdb).as_mut_ptr() as *mut u32,
            ((*(*Request).Device).CdbSize as usize)
                .wrapping_div(core::mem::size_of::<USHORT>()) as u32,
            ChanData,
            MRES_TF,
        );
    } else {
        ATA_WRITE_BLOCK_16!(
            (*ChanData).Regs.Data as *mut u16,
            ((*Request).__bindgen_anon_2.Cdb).as_mut_ptr() as *mut u16,
            (*(*Request).Device).CdbSize as u32,
            ChanData,
            MRES_TF,
        );
    }
}

unsafe extern "C" fn PataPioDataIn(mut ChanData: PCHANNEL_DATA_PATA, mut ByteCount: ULONG) {
    ByteCount = min(ByteCount, (*ChanData).BytesToTransfer);

    if ByteCount & (core::mem::size_of::<u32>() as u32).wrapping_sub(1 as u32) == 0
        && ((*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_IO32) != 0
    {
        ATA_READ_BLOCK_32!(
            (*ChanData).Regs.Data as *mut u32,
            (*ChanData).DataBuffer as *mut u32,
            ByteCount / core::mem::size_of::<u32>() as u32,
            ChanData,
            MRES_TF,
        );
    } else {
        ATA_READ_BLOCK_16!(
            (*ChanData).Regs.Data as *mut u16,
            (*ChanData).DataBuffer as *mut u16,
            ByteCount / core::mem::size_of::<USHORT>() as u32,
            ChanData,
            MRES_TF,
        );

        if ByteCount & (core::mem::size_of::<u16>() as u32).wrapping_sub(1 as u32) != 0 {
            // DECLSPEC_ALIGN(2) UCHAR LocalBuffer[2]: the 16-bit port I/O below needs the C's
            // alignment, so the bytes live in a u16 and are viewed through it.
            let mut LocalBufferStorage: [u16; 1] = [0];
            let LocalBuffer: &mut [u8; 2] = &mut *(LocalBufferStorage.as_mut_ptr() as *mut [u8; 2]);
            let mut BufferEnd: *mut u8 = core::ptr::null_mut();

            ATA_READ_BLOCK_16!(
                (*ChanData).Regs.Data as *mut u16,
                LocalBuffer.as_mut_ptr() as *mut u16,
                1 as u32,
                ChanData,
                MRES_TF,
            );

            BufferEnd = (*ChanData).DataBuffer.add(ByteCount as usize).sub(1 as usize);
            *BufferEnd = LocalBuffer[0 as usize];
        }
    }

    (*ChanData).DataBuffer = (*ChanData).DataBuffer.add(ByteCount as usize);
    (*ChanData).BytesToTransfer -= ByteCount;
}

unsafe extern "C" fn PataPioDataOut(
    mut ChanData: PCHANNEL_DATA_PATA,
    mut ByteCount: ULONG,
) {
    ByteCount = min(ByteCount, (*ChanData).BytesToTransfer);

    if (ByteCount as usize & (core::mem::size_of::<ULONG>() - 1)) == 0
        && ((*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_IO32) != 0
    {
        ATA_WRITE_BLOCK_32!(
            (*ChanData).Regs.Data as PULONG,
            (*ChanData).DataBuffer as PULONG,
            (ByteCount as usize / core::mem::size_of::<ULONG>()) as ULONG,
            ChanData,
            MRES_TF,
        );
    } else {
        ATA_WRITE_BLOCK_16!(
            (*ChanData).Regs.Data as PUSHORT,
            (*ChanData).DataBuffer as PUSHORT,
            (ByteCount as usize / core::mem::size_of::<USHORT>()) as ULONG,
            ChanData,
            MRES_TF,
        );

        if (ByteCount as usize & (core::mem::size_of::<USHORT>() - 1)) != 0 {
            // DECLSPEC_ALIGN(2) UCHAR LocalBuffer[2]: the 16-bit port I/O below needs the C's
            // alignment, so the bytes live in a u16 and are viewed through it.
            let mut LocalBufferStorage: [u16; 1] = [0];
            let LocalBuffer: &mut [UCHAR; 2] = &mut *(LocalBufferStorage.as_mut_ptr() as *mut [UCHAR; 2]);
            let mut BufferEnd: PUCHAR;

            BufferEnd = (*ChanData)
                .DataBuffer
                .offset((ByteCount as isize) - (1 as isize));
            LocalBuffer[0] = *BufferEnd;
            LocalBuffer[1] = 0;

            ATA_WRITE_BLOCK_16!(
                (*ChanData).Regs.Data as PUSHORT,
                LocalBuffer.as_mut_ptr() as PUSHORT,
                1,
                ChanData,
                MRES_TF,
            );
        }
    }

    (*ChanData).DataBuffer = (*ChanData).DataBuffer.offset(ByteCount as isize);
    (*ChanData).BytesToTransfer = (*ChanData).BytesToTransfer.wrapping_sub(ByteCount);
}

unsafe extern "C" fn PataProcessAtapiRequest(
    mut ChanData: PCHANNEL_DATA_PATA,
    mut Request: PATA_DEVICE_REQUEST,
    mut IdeStatus: UCHAR,
) -> UCHAR {
    let mut SrbStatus: UCHAR = 0;
    let mut InterruptReason: UCHAR = 0;
    let mut ByteCount: ULONG = 0;

    InterruptReason = ATA_READ!(
        (*ChanData).Regs.__bindgen_anon_2.InterruptReason,
        ChanData,
        MRES_TF,
    );
    InterruptReason &= (ATAPI_INT_REASON_MASK as u8);
    InterruptReason |= IdeStatus & (IDE_STATUS_DRQ as u8);

    if InterruptReason == ATAPI_INT_REASON_COD
        || InterruptReason == ATAPI_INT_REASON_IO
    {
        static mut WarningsGiven: SIZE_T = 0;
        let fresh = WarningsGiven;
        WarningsGiven = WarningsGiven.wrapping_add(1);
        if fresh < 2 {
            WARN!(
                b"Not fully compliant ATAPI device %02x %02x\n\0".as_ptr() as *const i8,
                IdeStatus as i32,
                InterruptReason as i32,
            );
        }
        IdeStatus = ATA_WAIT(
            ChanData,
            ATA_TIME_DRQ_ASSERT,
            (IDE_STATUS_DRQ) as UCHAR,
            (IDE_STATUS_DRQ) as UCHAR,
        );

        InterruptReason = ATA_READ!(
            (*ChanData).Regs.__bindgen_anon_2.InterruptReason,
            ChanData,
            MRES_TF,
        );
        InterruptReason &= (ATAPI_INT_REASON_MASK as u8);
        InterruptReason |= IdeStatus & (IDE_STATUS_DRQ as u8);
    }

    match (InterruptReason) as u32 {
        ATAPI_INT_REASON_AWAIT_CDB => {
            if (*ChanData).CommandFlags & CMD_FLAG_AWAIT_CDB == 0 {
                SrbStatus = (SRB_STATUS_BUS_RESET as u8);
            } else {
                (*ChanData).CommandFlags &= !CMD_FLAG_AWAIT_CDB;
                PataSendCdb(ChanData, Request);
                if (*Request).Flags & REQUEST_FLAG_DMA != 0 {
                    (*ChanData).CommandFlags |= CMD_FLAG_DMA_TRANSFER;
                    PciIdeDmaStart(ChanData, Request);
                }
                SrbStatus = (SRB_STATUS_PENDING as u8);
            }
        }
        ATAPI_INT_REASON_DATA_IN => {
            if (*ChanData).CommandFlags
                & (CMD_FLAG_DATA_OUT | CMD_FLAG_AWAIT_CDB)
                != 0
            {
                SrbStatus = (SRB_STATUS_BUS_RESET as u8);
            } else {
                ByteCount = ATA_READ!(
                    (*ChanData).Regs.__bindgen_anon_3.ByteCountLow,
                    ChanData,
                    MRES_TF,
                ) as ULONG;
                ByteCount |= (ATA_READ!(
                    (*ChanData).Regs.__bindgen_anon_4.ByteCountHigh,
                    ChanData,
                    MRES_TF,
                ) as ULONG)
                    << 8;
                PataPioDataIn(ChanData, ByteCount);
                SrbStatus = (SRB_STATUS_PENDING as u8);
            }
        }
        ATAPI_INT_REASON_DATA_OUT => {
            if (*ChanData).CommandFlags
                & (CMD_FLAG_DATA_IN | CMD_FLAG_AWAIT_CDB)
                != 0
            {
                SrbStatus = (SRB_STATUS_BUS_RESET as u8);
            } else {
                ByteCount = ATA_READ!(
                    (*ChanData).Regs.__bindgen_anon_3.ByteCountLow,
                    ChanData,
                    MRES_TF,
                ) as ULONG;
                ByteCount |= (ATA_READ!(
                    (*ChanData).Regs.__bindgen_anon_4.ByteCountHigh,
                    ChanData,
                    MRES_TF,
                ) as ULONG)
                    << 8;
                PataPioDataOut(ChanData, ByteCount);
                SrbStatus = (SRB_STATUS_PENDING as u8);
            }
        }
        ATAPI_INT_REASON_STATUS_NEC => {
            if (*(*Request).Device).TransportFlags & DEVICE_IS_NEC_CDR260 == 0 {
                SrbStatus = (SRB_STATUS_BUS_RESET as u8);
            } else {
                if IdeStatus & ((IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT) as u8) != 0 {
                    SrbStatus = (SRB_STATUS_ERROR as u8);
                } else if (*ChanData).BytesToTransfer != 0 {
                    (*Request).DataTransferLength =
                        (*Request).DataTransferLength.wrapping_sub((*ChanData).BytesToTransfer);
                    SrbStatus = (SRB_STATUS_DATA_OVERRUN as u8);
                } else {
                    SrbStatus = (SRB_STATUS_SUCCESS as u8);
                }
            }
        }
        ATAPI_INT_REASON_STATUS => {
            if IdeStatus & ((IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT) as u8) != 0 {
                SrbStatus = (SRB_STATUS_ERROR as u8);
            } else if (*ChanData).BytesToTransfer != 0 {
                (*Request).DataTransferLength =
                    (*Request).DataTransferLength.wrapping_sub((*ChanData).BytesToTransfer);
                SrbStatus = (SRB_STATUS_DATA_OVERRUN as u8);
            } else {
                SrbStatus = (SRB_STATUS_SUCCESS as u8);
            }
        }
        _ => {
            SrbStatus = (SRB_STATUS_BUS_RESET as u8);
        }
    }

    SrbStatus
}

unsafe extern "C" fn PataProcessAtaRequest(
    mut ChanData: PCHANNEL_DATA_PATA,
    mut Request: PATA_DEVICE_REQUEST,
    mut IdeStatus: UCHAR,
) -> UCHAR {
    if IdeStatus as ::core::ffi::c_int & (IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT) as i32 != 0 {
        if IdeStatus as ::core::ffi::c_int & (IDE_STATUS_DRQ as i32) != 0 {
            return (SRB_STATUS_BUS_RESET) as UCHAR;
        } else {
            return (SRB_STATUS_ERROR) as UCHAR;
        }
    }
    if (*ChanData).CommandFlags & (CMD_FLAG_DATA_IN | CMD_FLAG_DATA_OUT) as ULONG == 0 {
        if ((IsNEC_98) != 0) {
            IdeStatus &= (!IDE_STATUS_DRQ) as u8;
        }
        if IdeStatus as ::core::ffi::c_int & (IDE_STATUS_DRQ as i32) != 0 {
            return (SRB_STATUS_BUS_RESET) as UCHAR;
        } else {
            return (SRB_STATUS_SUCCESS) as UCHAR;
        }
    }
    if (*ChanData).CommandFlags & CMD_FLAG_DATA_IN as ULONG != 0 {
        if IdeStatus as ::core::ffi::c_int & (IDE_STATUS_DRQ as i32) == 0 {
            if IdeStatus as ::core::ffi::c_int == 0
                && ((*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command as ::core::ffi::c_int == (IDE_COMMAND_ATAPI_IDENTIFY as i32)
                    || (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command as ::core::ffi::c_int == (IDE_COMMAND_IDENTIFY as i32))
            {
                return (SRB_STATUS_ERROR) as UCHAR;
            } else {
                static mut WarningsGiven: SIZE_T = 0;
                let fresh = WarningsGiven;
                WarningsGiven = WarningsGiven.wrapping_add(1);
                if fresh < 2 {
                    WARN!(
                        b"Not fully compliant ATA device %02x\n\0" as *const u8 as PCSTR,
                        IdeStatus as ::core::ffi::c_int,
                    );
                }
                IdeStatus = ATA_WAIT(
                    ChanData,
                    ATA_TIME_DRQ_ASSERT,
                    (IDE_STATUS_DRQ) as UCHAR,
                    (IDE_STATUS_DRQ) as UCHAR,
                );
                if IdeStatus as ::core::ffi::c_int & (IDE_STATUS_DRQ as i32) == 0 {
                    return (SRB_STATUS_BUS_RESET) as UCHAR;
                }
            }
        }
        PataPioDataIn(ChanData, (*ChanData).DrqByteCount);
        if (*ChanData).BytesToTransfer != 0 {
            return (SRB_STATUS_PENDING) as UCHAR;
        }
        IdeStatus = ATA_WAIT(
            ChanData,
            ATA_TIME_DRQ_CLEAR,
            (IDE_STATUS_BUSY | IDE_STATUS_DRQ) as UCHAR,
            0,
        );
        if IdeStatus as ::core::ffi::c_int & (IDE_STATUS_BUSY | IDE_STATUS_DRQ) as i32 != 0 {
            return (SRB_STATUS_BUS_RESET) as UCHAR;
        }
        if IdeStatus as ::core::ffi::c_int & (IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT) as i32 != 0 {
            return (SRB_STATUS_ERROR) as UCHAR;
        }
        return (SRB_STATUS_SUCCESS) as UCHAR;
    }
    if (*ChanData).BytesToTransfer == 0 {
        if IdeStatus as ::core::ffi::c_int & (IDE_STATUS_DRQ as i32) != 0 {
            return (SRB_STATUS_BUS_RESET) as UCHAR;
        } else {
            return (SRB_STATUS_SUCCESS) as UCHAR;
        }
    }
    if IdeStatus as ::core::ffi::c_int & (IDE_STATUS_DRQ as i32) == 0 {
        return (SRB_STATUS_BUS_RESET) as UCHAR;
    }
    PataPioDataOut(ChanData, (*ChanData).DrqByteCount);
    return (SRB_STATUS_PENDING) as UCHAR;
}

unsafe extern "C" fn PataProcessRequest(
    ChanData: PCHANNEL_DATA_PATA,
    IdeStatus: UCHAR,
    DmaStatus: UCHAR,
) -> BOOLEAN {
    let Request: PATA_DEVICE_REQUEST = *(*ChanData).__bindgen_anon_1.Slots.add((PATA_CHANNEL_SLOT as usize));
    let mut SrbStatus: UCHAR;

    match (*ChanData).CommandFlags & CMD_FLAG_TRANSFER_MASK {
        CMD_FLAG_DMA_TRANSFER => {
            if (DmaStatus & (PCIIDE_DMA_STATUS_ERROR as u8)) != 0
                || (IdeStatus & ((IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT) as u8)) != 0
            {
                SrbStatus = (SRB_STATUS_ERROR as u8);
            } else {
                SrbStatus = (SRB_STATUS_SUCCESS as u8);
            }
        }
        CMD_FLAG_ATA_PIO_TRANSFER => {
            SrbStatus = PataProcessAtaRequest(ChanData, Request, IdeStatus);
        }
        CMD_FLAG_ATAPI_PIO_TRANSFER => {
            SrbStatus = PataProcessAtapiRequest(ChanData, Request, IdeStatus);
        }
        _ => {
            UNREACHABLE!();
        }
    }

    if SrbStatus == (SRB_STATUS_PENDING as u8) {
        return FALSE;
    }

    PataCompleteCommand(ChanData, IdeStatus, SrbStatus);
    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn PataLoadTaskFile(ChanData: PCHANNEL_DATA_PATA, Request: PATA_DEVICE_REQUEST) {
    if (*Request).Flags & REQUEST_FLAG_LBA48 != 0 {
        ATA_WRITE!(
            (*ChanData).Regs.__bindgen_anon_1.Features,
            (*Request).__bindgen_anon_2.TaskFile.FeatureEx,
            ChanData,
            MRES_TF,
        );
        ATA_WRITE!(
            (*ChanData).Regs.__bindgen_anon_2.SectorCount,
            (*Request).__bindgen_anon_2.TaskFile.SectorCountEx,
            ChanData,
            MRES_TF,
        );
        ATA_WRITE!(
            (*ChanData).Regs.LbaLow,
            (*Request).__bindgen_anon_2.TaskFile.LowLbaEx,
            ChanData,
            MRES_TF,
        );
        ATA_WRITE!(
            (*ChanData).Regs.__bindgen_anon_3.LbaMid,
            (*Request).__bindgen_anon_2.TaskFile.MidLbaEx,
            ChanData,
            MRES_TF,
        );
        ATA_WRITE!(
            (*ChanData).Regs.__bindgen_anon_4.LbaHigh,
            (*Request).__bindgen_anon_2.TaskFile.HighLbaEx,
            ChanData,
            MRES_TF,
        );
    }
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_1.Features,
        (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_2.Feature,
        ChanData,
        MRES_TF,
    );
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_2.SectorCount,
        (*Request).__bindgen_anon_2.TaskFile.SectorCount,
        ChanData,
        MRES_TF,
    );
    ATA_WRITE!(
        (*ChanData).Regs.LbaLow,
        (*Request).__bindgen_anon_2.TaskFile.LowLba,
        ChanData,
        MRES_TF,
    );
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_3.LbaMid,
        (*Request).__bindgen_anon_2.TaskFile.MidLba,
        ChanData,
        MRES_TF,
    );
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_4.LbaHigh,
        (*Request).__bindgen_anon_2.TaskFile.HighLba,
        ChanData,
        MRES_TF,
    );
    if (*Request).Flags & REQUEST_FLAG_SET_DEVICE_REGISTER != 0 {
        ATA_WRITE!(
            (*ChanData).Regs.Device,
            (*Request).__bindgen_anon_2.TaskFile.DriveSelect,
            ChanData,
            MRES_TF,
        );
    }
}

unsafe extern "C" fn PataExecutePacketCommand(
    mut ChanData: PCHANNEL_DATA_PATA,
    mut Device: PATA_IO_CONTEXT_COMMON,
    mut Request: PATA_DEVICE_REQUEST,
) -> BOOLEAN {
    let mut CommandFlags: ULONG = 0;
    let mut ByteCount: USHORT = 0;
    let mut Features: UCHAR = 0;

    CommandFlags = (*Request).Flags & (REQUEST_FLAG_DATA_IN | REQUEST_FLAG_DATA_OUT);
    CommandFlags |= !(*Request).Flags & REQUEST_FLAG_POLL;
    CommandFlags |= CMD_FLAG_ATAPI_PIO_TRANSFER | CMD_FLAG_AWAIT_CDB;
    (*ChanData).CommandFlags = CommandFlags;

    if (*Request).Flags & REQUEST_FLAG_DMA != 0 {
        ByteCount = 0;
        Features = (IDE_FEATURE_DMA as u8);
    } else {
        ByteCount = min((*ChanData).BytesToTransfer, ATAPI_MAX_DRQ_DATA_BLOCK) as USHORT;
        Features = (IDE_FEATURE_PIO as u8);
    }

    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_3.ByteCountLow,
        (ByteCount >> 0) as UCHAR,
        ChanData,
        MRES_TF,
    );
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_4.ByteCountHigh,
        (ByteCount >> 8) as UCHAR,
        ChanData,
        MRES_TF,
    );
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_1.Features,
        Features,
        ChanData,
        MRES_TF,
    );
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_5.Command,
        (IDE_COMMAND_ATAPI_PACKET as u8),
        ChanData,
        MRES_TF,
    );

    if (*Device).TransportFlags & DEVICE_HAS_CDB_INTERRUPT != 0 {
        return FALSE;
    }

    return TRUE;
}

unsafe extern "C" fn PataExecuteAtaCommand(
    ChanData: PCHANNEL_DATA_PATA,
    Device: PATA_IO_CONTEXT_COMMON,
    Request: PATA_DEVICE_REQUEST,
) -> BOOLEAN {
    let mut CommandFlags: ULONG;
    let mut BlockCount: ULONG;

    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_DMA_BEFORE_CMD != 0
        && (*Request).Flags & REQUEST_FLAG_DMA != 0
    {
        PciIdeDmaStart(ChanData, Request);
    }

    (*ChanData).LoadTaskFile.unwrap_unchecked()(ChanData, Request);

    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_5.Command,
        (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command,
        ChanData,
        MRES_TF,
    );

    CommandFlags = (*Request).Flags & (REQUEST_FLAG_DATA_IN | REQUEST_FLAG_DATA_OUT);
    CommandFlags |= !(*Request).Flags & REQUEST_FLAG_POLL;

    if (*Request).Flags & REQUEST_FLAG_DMA != 0 {
        (*ChanData).CommandFlags = CommandFlags | CMD_FLAG_DMA_TRANSFER;

        PciIdeDmaStart(ChanData, Request);
        return FALSE;
    }

    (*ChanData).CommandFlags = CommandFlags | CMD_FLAG_ATA_PIO_TRANSFER;

    if (*Request).Flags & REQUEST_FLAG_READ_WRITE_MULTIPLE != 0 {
        BlockCount = ((*Device).MultiSectorCount) as _;
    } else {
        BlockCount = 1;
    }
    (*ChanData).DrqByteCount = BlockCount.wrapping_mul((*Device).SectorSize);

    if (*Request).Flags & REQUEST_FLAG_DATA_OUT != 0 {
        return TRUE;
    }

    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn PataStartIo(ChannelContext: PVOID, Request: PATA_DEVICE_REQUEST) -> BOOLEAN {
    let ChanData: *mut CHANNEL_DATA_PATA = ChannelContext as *mut CHANNEL_DATA_PATA;
    let Device: *mut ATA_IO_CONTEXT_COMMON = (*Request).Device;
    let mut IdeStatus: u8;
    let mut IsPollingNeeded: i32;

    (*ChanData).__bindgen_anon_1.ActiveSlotsBitmap |= 1 << PATA_CHANNEL_SLOT;

    ATA_SELECT_DEVICE(ChanData, (DEV_NUMBER!(Device)) as UCHAR, (*Device).DeviceSelect);

    IdeStatus = ATA_WAIT(ChanData, ATA_TIME_BUSY_SELECT, (IDE_STATUS_BUSY) as UCHAR, 0);
    if IdeStatus & (IDE_STATUS_BUSY as u8) != 0 {
        PataCompleteCommand(ChanData, IdeStatus, (SRB_STATUS_SELECTION_TIMEOUT) as UCHAR);
        return ((TRUE as i32)) as BOOLEAN;
    }

    if ((*Request).Flags & REQUEST_FLAG_POLL != 0) as i32 != (*ChanData).IsPollingActive as i32 {
        PataChangeInterruptMode(ChanData, Device, Request);
    }

    if (*Request).Flags & REQUEST_FLAG_PACKET_COMMAND != 0 {
        IsPollingNeeded = (PataExecutePacketCommand(ChanData, Device, Request)) as _;
    } else {
        IsPollingNeeded = (PataExecuteAtaCommand(ChanData, Device, Request)) as _;
    }

    if (*Request).Flags & REQUEST_FLAG_POLL != 0 {
        KeInsertQueueDpc(&raw mut (*ChanData).PollingTimerDpc, core::ptr::null_mut(), core::ptr::null_mut());
        return ((FALSE as i32)) as BOOLEAN;
    }

    (*ChanData).CommandFlags |= CMD_FLAG_AWAIT_INTERRUPT;

    if IsPollingNeeded == 0 {
        return ((FALSE as i32)) as BOOLEAN;
    }

    ATA_IO_WAIT!();

    IdeStatus = ATA_WAIT(ChanData, ATA_TIME_BUSY_NORMAL, (IDE_STATUS_BUSY) as UCHAR, 0);
    if IdeStatus & (IDE_STATUS_BUSY as u8) != 0 {
        PataCompleteCommand(ChanData, IdeStatus, (SRB_STATUS_BUS_RESET) as UCHAR);
        return ((TRUE as i32)) as BOOLEAN;
    }

    return (PataProcessRequest(ChanData, IdeStatus, 0)) as _;
}
#[no_mangle]
pub unsafe extern "C" fn PataPrepareIo(ChannelContext: PVOID, Request: PATA_DEVICE_REQUEST) {
    let ChanData = ChannelContext as *mut CHANNEL_DATA_PATA;

    if IsNEC_98 != 0 {
        (*Request).Flags |= REQUEST_FLAG_POLL;
    }

    (*ChanData).BytesToTransfer = (*Request).DataTransferLength;
    (*ChanData).DataBuffer = ((*Request).DataBuffer).cast();
}
#[no_mangle]
pub unsafe extern "C" fn PciIdePreparePrdTable(ChannelContext: PVOID, Request: PATA_DEVICE_REQUEST, SgList: *mut SCATTER_GATHER_LIST) {
    let ChanData: *mut CHANNEL_DATA_PATA = ChannelContext as *mut CHANNEL_DATA_PATA;
    let mut PrdTableEntry: *mut PCIIDE_PRD_TABLE_ENTRY;
    let mut i: u32;
    let mut DescriptorNumber: u32 = 0;

    DBG_UNREFERENCED_LOCAL_VARIABLE!(&mut DescriptorNumber);

    ASSUME!((*SgList).NumberOfElements > 0);

    PrdTableEntry = (*ChanData).PrdTable;

    i = 0;
    while i < (*SgList).NumberOfElements {
        let mut Address: u32 = (*core::ptr::addr_of!((*SgList).Elements[0]).add(i as usize)).Address.__bindgen_anon_1.LowPart;
        let mut Length: u32 = (*core::ptr::addr_of!((*SgList).Elements[0]).add(i as usize)).Length;

        ASSUME!(Length != 0);

        while Length > 0 {
            let mut TransferLength: u32;

            TransferLength = PCIIDE_PRD_LIMIT - (Address & (PCIIDE_PRD_LIMIT - 1));
            TransferLength = if TransferLength < Length { TransferLength } else { Length };

            DescriptorNumber += 1;

            (*PrdTableEntry).Address = Address;
            (*PrdTableEntry).Length = TransferLength & PCIIDE_PRD_LENGTH_MASK;

            PrdTableEntry = PrdTableEntry.offset(1);

            Address += TransferLength;
            Length -= TransferLength;
        }

        i += 1;
    }

    PrdTableEntry = PrdTableEntry.offset(-1);
    (*PrdTableEntry).Length |= PCIIDE_PRD_END_OF_TABLE;

    KeFlushIoBuffers(
        (*Request).Mdl,
        (((*Request).Flags & REQUEST_FLAG_DATA_IN) != 0) as u8,
        TRUE,
    );

    PciIdeDmaPrepare(ChanData);
}
#[no_mangle]
pub unsafe extern "C" fn PataAllocateSlot(ChannelContext: PVOID, Request: PATA_DEVICE_REQUEST, Allocate: BOOLEAN) -> BOOLEAN {
    UNREFERENCED_PARAMETER!(ChannelContext);
    UNREFERENCED_PARAMETER!(Request);
    UNREFERENCED_PARAMETER!(Allocate);

    return ((TRUE as i32)) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeChannelIsr(Interrupt: *mut _KINTERRUPT, Context: PVOID) -> BOOLEAN {
    let ChanData: *mut CHANNEL_DATA_PATA = Context as *mut CHANNEL_DATA_PATA;
    let IdeStatus: u8;
    let DmaStatus: u8;
    let WasCleared: i32;

    UNREFERENCED_PARAMETER!(Interrupt);

    if !(*ChanData).CheckInterrupt.is_none()
        && ((*ChanData).CheckInterrupt.unwrap_unchecked())(ChanData) == 0
    {
        return ((FALSE as i32)) as BOOLEAN;
    }

    DmaStatus = PciIdeDmaReadStatus(ChanData);
    if DmaStatus as i32 & (PCIIDE_DMA_STATUS_INTERRUPT as i32) != 0 {
        if (*ChanData).CommandFlags & CMD_FLAG_TRANSFER_MASK == CMD_FLAG_DMA_TRANSFER {
            PciIdeDmaStop(ChanData);
        } else {
            PciIdeDmaClearInterrupt(ChanData, DmaStatus);
        }
        WasCleared = (TRUE as i32);
    } else {
        if (*ChanData).CommandFlags & CMD_FLAG_TRANSFER_MASK == CMD_FLAG_DMA_TRANSFER {
            return ((FALSE as i32)) as BOOLEAN;
        }
        WasCleared = (FALSE as i32);
    }

    IdeStatus = ((*ChanData).ReadStatus.unwrap_unchecked())(ChanData);

    if (*ChanData).__bindgen_anon_1.ActiveSlotsBitmap & (1 << PATA_CHANNEL_SLOT) == 0
        || (*ChanData).CommandFlags & CMD_FLAG_AWAIT_INTERRUPT == 0
        || IdeStatus as i32 & (IDE_STATUS_BUSY as i32) != 0
    {
        if WasCleared != 0 {
        }
        return (WasCleared) as BOOLEAN;
    }

    PataProcessRequest(ChanData, IdeStatus, DmaStatus);
    return ((TRUE as i32)) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn PataChannelIsr(Interrupt: *mut _KINTERRUPT, Context: PVOID) -> BOOLEAN {
    let ChanData: PCHANNEL_DATA_PATA = Context as PCHANNEL_DATA_PATA;
    let mut IdeStatus: u8;

    UNREFERENCED_PARAMETER!(Interrupt);

    if (*ChanData).CheckInterrupt.is_some()
        && !((*ChanData).CheckInterrupt.unwrap_unchecked()(ChanData) != 0)
    {
        return ((FALSE as i32)) as BOOLEAN;
    }

    IdeStatus = (*ChanData).ReadStatus.unwrap_unchecked()(ChanData);

    if (*ChanData).__bindgen_anon_1.ActiveSlotsBitmap & ((1 as u32) << PATA_CHANNEL_SLOT) == 0
        || (*ChanData).CommandFlags & CMD_FLAG_AWAIT_INTERRUPT == 0
        || (IdeStatus as i32 & (IDE_STATUS_BUSY as i32)) != 0
    {
        return ((FALSE as i32)) as BOOLEAN;
    }

    PataProcessRequest(ChanData, IdeStatus, 0);
    return ((TRUE as i32)) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeCheckInterrupt(ChanData: PCHANNEL_DATA_PATA) -> BOOLEAN {
    let DmaStatus: UCHAR = PciIdeDmaReadStatus(ChanData);

    return ((DmaStatus as i32 & (PCIIDE_DMA_STATUS_INTERRUPT as i32) != 0)) as BOOLEAN;
}

unsafe extern "C" fn PataPoll(mut ChanData: PCHANNEL_DATA_PATA) -> BOOLEAN {
    let mut IsRequestCompleted: BOOLEAN = FALSE;
    let mut IdeStatus: UCHAR = 0;
    loop {
        ATA_IO_WAIT!();
        IdeStatus = ATA_WAIT(
            ChanData,
            ATA_TIME_BUSY_POLL,
            (IDE_STATUS_BUSY) as UCHAR,
            0,
        );
        if IdeStatus as ::core::ffi::c_int & (IDE_STATUS_BUSY as i32) != 0 {
            break;
        }
        IsRequestCompleted = PataProcessRequest(ChanData, IdeStatus, 0);
        if IsRequestCompleted != 0 {
            break;
        }
    }
    return IsRequestCompleted;
}

#[no_mangle]
pub unsafe extern "system" fn PataPollingTimerDpc(Dpc: *mut _KDPC, DeferredContext: PVOID, SystemArgument1: PVOID, SystemArgument2: PVOID) {
    let ChanData: PCHANNEL_DATA_PATA = DeferredContext as PCHANNEL_DATA_PATA;
    let mut OldIrql: KIRQL;
    let mut IsRequestCompleted: i32;

    UNREFERENCED_PARAMETER!(Dpc);
    UNREFERENCED_PARAMETER!(SystemArgument1);
    UNREFERENCED_PARAMETER!(SystemArgument2);

    OldIrql = KeAcquireInterruptSpinLock((*ChanData).__bindgen_anon_1.InterruptObject);

    if !((*ChanData).__bindgen_anon_1.ActiveSlotsBitmap & (1 << PATA_CHANNEL_SLOT) != 0)
        || ((*ChanData).CommandFlags & CMD_FLAG_AWAIT_INTERRUPT) != 0
    {
        IsRequestCompleted = (FALSE as i32);
    } else {
        IsRequestCompleted = (PataPoll(ChanData)) as _;
    }

    KeReleaseInterruptSpinLock((*ChanData).__bindgen_anon_1.InterruptObject, OldIrql);

    if IsRequestCompleted == 0 {
        KeInsertQueueDpc(&raw mut (*ChanData).PollingTimerDpc, core::ptr::null_mut(), core::ptr::null_mut());
    }
}
