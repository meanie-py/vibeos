use crate::prelude::*;
use ntkrnl_sys::ddk::{PKINTERRUPT, PULONG64, SCATTER_GATHER_LIST};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
const _: () = {
    assert!(core::mem::offset_of!(ATA_DEVICE_REQUEST, __bindgen_anon_2.Cdb) % core::mem::size_of::<ULONG64>() == 0);
    assert!(core::mem::offset_of!(AHCI_COMMAND_TABLE, AtapiCommand) % core::mem::size_of::<ULONG64>() == 0);
};

unsafe extern "C" fn AtaAhciBeginHostToDeviceFis(
    Request: *mut ATA_DEVICE_REQUEST,
    Fis: *mut AHCI_FIS_HOST_TO_DEVICE,
) {
    RtlZeroMemory!(Fis as *mut core::ffi::c_void, core::mem::size_of::<AHCI_FIS_HOST_TO_DEVICE>());

    (*Fis).Type = (AHCI_FIS_REGISTER_HOST_TO_DEVICE as u8);
    (*Fis).Flags = (UPDATE_COMMAND | DEV_NUMBER!((*Request).Device)) as u8;

    if (*Request).Flags & REQUEST_FLAG_SET_DEVICE_REGISTER != 0 {
        (*Fis).Device = (*Request).__bindgen_anon_2.TaskFile.DriveSelect;
    } else {
        (*Fis).Device = (IDE_DRIVE_SELECT as u8);
    }

    (*Fis).Control = (IDE_DC_ALWAYS as u8);
}

unsafe extern "C" fn AtaAhciBuildPacketCommandFis(
    mut Request: *mut ATA_DEVICE_REQUEST,
    mut Fis: *mut AHCI_FIS_HOST_TO_DEVICE,
) {
    (*Fis).Command = (IDE_COMMAND_ATAPI_PACKET as u8);
    if (*Request).Flags & REQUEST_FLAG_DMA != 0 {
        if (*(*Request).Device).TransportFlags & DEVICE_NEED_DMA_DIRECTION != 0
            && (*Request).Flags & REQUEST_FLAG_DATA_IN != 0
        {
            (*Fis).Features = (IDE_FEATURE_DMA | IDE_FEATURE_DMADIR) as u8;
        } else {
            (*Fis).Features = (IDE_FEATURE_DMA as u8);
        }
    } else {
        let mut ByteCount: USHORT = 0;
        ByteCount = min((*Request).DataTransferLength, ATAPI_MAX_DRQ_DATA_BLOCK) as USHORT;
        (*Fis).LbaMid = ByteCount as UCHAR;
        (*Fis).LbaHigh = (ByteCount >> 8) as UCHAR;
    };
}

unsafe extern "C" fn AtaAhciBuildAtaCommandFis(
    Request: *mut ATA_DEVICE_REQUEST,
    Fis: *mut AHCI_FIS_HOST_TO_DEVICE,
) {
    let TaskFile: PATA_TASKFILE = &raw mut (*Request).__bindgen_anon_2.TaskFile;

    (*Fis).Command = (*TaskFile).__bindgen_anon_1.Command;
    (*Fis).Features = (*TaskFile).__bindgen_anon_2.Feature;
    (*Fis).LbaLow = (*TaskFile).LowLba;
    (*Fis).LbaMid = (*TaskFile).MidLba;
    (*Fis).LbaHigh = (*TaskFile).HighLba;
    (*Fis).SectorCount = (*TaskFile).SectorCount;

    if (*Request).Flags & REQUEST_FLAG_NCQ != 0 {
        (*Fis).SectorCount =
            ((*Fis).SectorCount as u32 | (*Request).Slot << 3) as u8;
    }

    if (*Request).Flags & REQUEST_FLAG_LBA48 != 0 {
        (*Fis).LbaLowEx = (*TaskFile).LowLbaEx;
        (*Fis).LbaMidEx = (*TaskFile).MidLbaEx;
        (*Fis).LbaHighEx = (*TaskFile).HighLbaEx;
        (*Fis).FeaturesEx = (*TaskFile).FeatureEx;
        (*Fis).SectorCountEx = (*TaskFile).SectorCountEx;
    }

    if (*Request).Flags & REQUEST_FLAG_SET_ICC_FIELD != 0 {
        (*Fis).Icc = (*TaskFile).Icc;
    }

    if (*Request).Flags & REQUEST_FLAG_SET_AUXILIARY_FIELD != 0 {
        (*Fis).Auxiliary = (*TaskFile).Auxiliary;
    }
}

unsafe extern "C" fn AtaAhciTransferACMDRegion(
    mut Destination: *mut core::ffi::c_void,
    mut Source: *mut core::ffi::c_void,
    mut Device: *mut ATA_IO_CONTEXT_COMMON,
) {
    let mut Dest: PULONG64 = Destination as PULONG64;
    let mut Src: PULONG64 = Source as PULONG64;

    *Dest = *Src;
    Dest = Dest.offset(1);
    Src = Src.offset(1);

    if ((*Device).CdbSize as usize
        & (core::mem::size_of::<u64>() - 1))
        == 0
    {
        *Dest = *Src;
    } else {
        *(Dest as *mut u32) = *(Src as *mut u32);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciStartIo(ChannelContext: PVOID, Request: PATA_DEVICE_REQUEST) -> BOOLEAN {
    let ChanData = ChannelContext as *mut CHANNEL_DATA_AHCI;
    let IoBase = (*ChanData).IoBase;
    let mut IssueSlot: u32;

    IssueSlot = 1 << (*Request).Slot;

    (*ChanData).__bindgen_anon_1.ActiveSlotsBitmap |= IssueSlot;

    if (*Request).Flags & REQUEST_FLAG_NCQ != 0 {
        (*ChanData).__bindgen_anon_1.ActiveQueuedSlotsBitmap |= IssueSlot;
        AHCI_PORT_WRITE(IoBase, PxSataActive, IssueSlot);
    }

    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_FBS_ENABLED != 0
        && (*ChanData).LastFbsDeviceNumber as u32 != DEV_NUMBER!((*Request).Device)
    {
        let mut Control: u32;

        (*ChanData).LastFbsDeviceNumber = DEV_NUMBER!((*Request).Device) as u8;

        Control = AHCI_FBS_ENABLE;
        Control |= (DEV_NUMBER!((*Request).Device) as u32) << AHCI_FBS_ISSUE_SHIFT;
        AHCI_PORT_WRITE(IoBase, PxFisSwitchingControl, Control);
    }

    AHCI_PORT_WRITE(IoBase, PxCommandIssue, IssueSlot);
    return ((FALSE as i32)) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciPrepareIo(ChannelContext: PVOID, Request: PATA_DEVICE_REQUEST) {
    let ChanData = ChannelContext as *mut CHANNEL_DATA_AHCI;
    let CommandTable: *mut AHCI_COMMAND_TABLE;
    let CommandHeader: *mut AHCI_COMMAND_HEADER;
    let Fis: *mut AHCI_FIS_HOST_TO_DEVICE;
    let mut Control: u32;

    Control = (core::mem::size_of::<AHCI_FIS_HOST_TO_DEVICE>() / core::mem::size_of::<u32>()) as u32;
    Control |= DEV_NUMBER!((*Request).Device) << AHCI_COMMAND_HEADER_PMP_SHIFT;

    if (*Request).Flags & (REQUEST_FLAG_DATA_IN | REQUEST_FLAG_DATA_OUT) != 0 {
        ASSERT!((*Request).Flags & REQUEST_FLAG_HAS_SG_LIST);

        Control |= (*(*Request).SgList).NumberOfElements << AHCI_COMMAND_HEADER_PRDT_LENGTH_SHIFT;

        if (*Request).Flags & REQUEST_FLAG_DATA_OUT != 0 {
            Control |= AHCI_COMMAND_HEADER_WRITE;
        }
    }

    CommandTable = (*ChanData).CommandTable[(*Request).Slot as usize];

    Fis = &raw mut (*CommandTable).__bindgen_anon_1.HostToDeviceFis;

    AtaAhciBeginHostToDeviceFis(Request, Fis);

    if (*Request).Flags & REQUEST_FLAG_PACKET_COMMAND != 0 {
        AtaAhciBuildPacketCommandFis(Request, Fis);
        AtaAhciTransferACMDRegion(
            &raw mut (*CommandTable).AtapiCommand as *mut core::ffi::c_void,
            &raw mut (*Request).__bindgen_anon_2.Cdb as *mut core::ffi::c_void,
            (*Request).Device,
        );

        Control |= AHCI_COMMAND_HEADER_ATAPI;
    } else {
        AtaAhciBuildAtaCommandFis(Request, Fis);
    }

    CommandHeader = (&raw mut (*(*ChanData).CommandList).CommandHeader as *mut AHCI_COMMAND_HEADER)
        .offset((*Request).Slot as isize);
    (*CommandHeader).Control = Control;
    (*CommandHeader).PrdByteCount = 0;

    KeFlushIoBuffers(
        (*Request).Mdl,
        (((*Request).Flags & REQUEST_FLAG_DATA_IN != 0) as i32) as u8,
        TRUE,
    );
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciPreparePrdTable(ChannelContext: PVOID, Request: PATA_DEVICE_REQUEST, SgList: *mut SCATTER_GATHER_LIST) {
    let ChanData = ChannelContext as *mut CHANNEL_DATA_AHCI;
    let mut PrdTableEntry: *mut AHCI_PRD_TABLE_ENTRY;
    let mut i: u32;
    let Is64BitDma: u8 = ((*((*ChanData).__bindgen_anon_1.Controller)).AhciCapabilities & AHCI_CAP_S64A != 0) as u8;

    ASSUME!((*SgList).NumberOfElements > 0);
    ASSERT!((*SgList).NumberOfElements < AHCI_MAX_PRDT_ENTRIES);

    PrdTableEntry = (*(*(*ChanData).CommandTable.as_ptr().offset((*Request).Slot as isize))).PrdTable.as_ptr() as *mut AHCI_PRD_TABLE_ENTRY;

    i = 0;
    while i < (*SgList).NumberOfElements {
        ASSERT!((*(*SgList).Elements.as_ptr().offset(i as isize)).Length != 0);
        ASSERT!((*(*SgList).Elements.as_ptr().offset(i as isize)).Length < AHCI_MAX_PRD_LENGTH);
        ASSERT!(((*(*SgList).Elements.as_ptr().offset(i as isize)).Length & ATA_MIN_BUFFER_ALIGNMENT) == 0);
        ASSERT!(Is64BitDma != 0 || ((*(*SgList).Elements.as_ptr().offset(i as isize)).Address.__bindgen_anon_1.HighPart == 0));
        ASSERT!(i < (*ChanData).__bindgen_anon_1.MaximumPhysicalPages);

        (*PrdTableEntry).DataBaseLow = (*(*SgList).Elements.as_ptr().offset(i as isize)).Address.__bindgen_anon_1.LowPart;
        (*PrdTableEntry).DataBaseHigh = ((*(*SgList).Elements.as_ptr().offset(i as isize)).Address.__bindgen_anon_1.HighPart) as u32;
        (*PrdTableEntry).ByteCount = (*(*SgList).Elements.as_ptr().offset(i as isize)).Length - 1;

        PrdTableEntry = PrdTableEntry.offset(1);
        i = i.wrapping_add(1);
    }

    PrdTableEntry = PrdTableEntry.offset(-1);
    (*PrdTableEntry).ByteCount |= AHCI_PRD_INTERRUPT_ON_COMPLETION;
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciAllocateSlot(ChannelContext: PVOID, Request: PATA_DEVICE_REQUEST, Allocate: BOOLEAN) -> BOOLEAN {
    let ChanData: *mut CHANNEL_DATA_AHCI = ChannelContext as *mut CHANNEL_DATA_AHCI;

    if Allocate != 0 {
        if ((*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_IS_PMP) != 0
            && ((*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_FBS_ENABLED) == 0
        {
            if ((*ChanData).LastPmpDeviceNumber as i32 != 0xFF as i32)
                && ((*ChanData).LastPmpDeviceNumber as u32
                    != DEV_NUMBER!((*Request).Device) as u32)
            {
                return ((FALSE as i32)) as BOOLEAN;
            }

            (*ChanData).LastPmpDeviceNumber = DEV_NUMBER!((*Request).Device) as u8;

            (*Request).Flags |= REQUEST_FLAG_DEVICE_EXCLUSIVE_ACCESS;
        }
    } else {
        if ((*Request).Flags & REQUEST_FLAG_DEVICE_EXCLUSIVE_ACCESS) != 0 {
            (*ChanData).LastPmpDeviceNumber = 0xFF as u8;
        }
    }

    return ((TRUE as i32)) as BOOLEAN;
}

unsafe extern "C" fn AtaAhciPortCompleteCommands(
    mut ChanData: PCHANNEL_DATA_AHCI,
    mut CommandsCompleted: ULONG,
) {
    let mut Slot: ULONG = 0;
    while _BitScanForward(&mut Slot, CommandsCompleted) != 0 {
        let mut Request: PATA_DEVICE_REQUEST = core::ptr::null_mut();
        let mut SrbStatus: UCHAR = 0;

        CommandsCompleted &= !(1 << Slot);

        Request = *(*ChanData).__bindgen_anon_1.Slots.offset(Slot as isize);
        ASSERT!(Request);
        ASSERT!((*Request).Slot == Slot);

        SrbStatus = (SRB_STATUS_SUCCESS as u8);

        if (*Request).Flags & REQUEST_FLAG_NCQ == 0
            && (*Request).Flags & (REQUEST_FLAG_DATA_IN | REQUEST_FLAG_DATA_OUT) != 0
        {
            let mut CommandHeader: PAHCI_COMMAND_HEADER =
                (&mut (*(*ChanData).CommandList).CommandHeader as *mut AHCI_COMMAND_HEADER)
                    .offset(Slot as isize) as PAHCI_COMMAND_HEADER;

            if (*CommandHeader).PrdByteCount < (*Request).DataTransferLength {
                (*Request).DataTransferLength = (*CommandHeader).PrdByteCount;
                SrbStatus = (SRB_STATUS_DATA_OVERRUN as u8);
            }
        }

        (*Request).SrbStatus = SrbStatus;

        if (*Request).Flags & REQUEST_FLAG_SAVE_TASK_FILE != 0 {
            AtaAhciSaveTaskFile(ChanData, Request, FALSE);
        }
    }
}

unsafe extern "C" fn AtaAhciPortHandleInterrupt(mut ChanData: PCHANNEL_DATA_AHCI) {
    let mut InterruptStatus: ULONG = 0;
    let mut CommandsIssued: ULONG = 0;
    let mut CommandsCompleted: ULONG = 0;
    let mut CommandRegister: AHCI_PORT_REGISTER = PxCommandListBaseLow;

    InterruptStatus = AHCI_PORT_READ((*ChanData).IoBase, PxInterruptStatus);
    AHCI_PORT_WRITE((*ChanData).IoBase, PxInterruptStatus, InterruptStatus);

    AHCI_PORT_WRITE(
        (*ChanData).IoBase,
        PxSataError,
        AHCI_PORT_READ((*ChanData).IoBase, PxSataError),
    );

    if (*ChanData).__bindgen_anon_1.ActiveQueuedSlotsBitmap != 0 {
        CommandRegister = PxSataActive;
    } else {
        CommandRegister = PxCommandIssue;
    }
    CommandsIssued = AHCI_PORT_READ((*ChanData).IoBase, CommandRegister);
    CommandsCompleted = !CommandsIssued & (*ChanData).__bindgen_anon_1.ActiveSlotsBitmap;

    if CommandsCompleted != 0 {
        (*ChanData).__bindgen_anon_1.ActiveSlotsBitmap &= !CommandsCompleted;
        (*ChanData).__bindgen_anon_1.ActiveQueuedSlotsBitmap &= !CommandsCompleted;

        AtaAhciPortCompleteCommands(ChanData, CommandsCompleted);
        if let Some(notification) = (*ChanData).__bindgen_anon_1.PortNotification {
            notification(
                AtaRequestComplete,
                (*ChanData).__bindgen_anon_1.PortContext,
                CommandsCompleted,
            );
        }
    }

    if InterruptStatus & AHCI_PXIRQ_SDBS != 0 {
        let mut Message: ULONG = AHCI_PORT_READ((*ChanData).IoBase, PxSataNotification);

        if Message != 0 {
            AHCI_PORT_WRITE((*ChanData).IoBase, PxSataNotification, Message);

            WARN!(
                b"CH %lu: Notification %08lx arrived\n\0".as_ptr() as *const i8,
                (*ChanData).__bindgen_anon_1.Channel,
                Message,
            );

            if let Some(notification) = (*ChanData).__bindgen_anon_1.PortNotification {
                notification(
                    AtaAsyncNotificationDetected,
                    (*ChanData).__bindgen_anon_1.PortContext,
                    0x1,
                );
            }
        }
    }

    if InterruptStatus & AHCI_PXIRQ_PORT_STATUS != 0 {
        AtaAhciHandlePortStateChange(ChanData, InterruptStatus);
    }

    if InterruptStatus & AHCI_PXIRQ_FATAL_ERROR != 0 {
        AtaAhciHandleFatalError(ChanData);
    }
}

#[no_mangle]
pub unsafe extern "system" fn AtaAhciHbaIsr(Interrupt: *mut _KINTERRUPT, Context: PVOID) -> BOOLEAN {
    let Controller: *mut ATA_CONTROLLER = Context as *mut ATA_CONTROLLER;
    let mut Port: u32 = 0;
    let mut InterruptStatus: u32;
    let mut PortInterruptBitmap: u32;

    InterruptStatus = AHCI_HBA_READ(((*Controller).IoBase).cast(), HbaInterruptStatus);
    if InterruptStatus == 0 {
        return ((FALSE as i32)) as BOOLEAN;
    }

    PortInterruptBitmap = InterruptStatus & (*Controller).ChannelBitmap;
    while _BitScanForward(&mut Port, PortInterruptBitmap) != 0 {
        PortInterruptBitmap &= !(1 << Port);

        AtaAhciPortHandleInterrupt(((*Controller).Channels[Port as usize]).cast());
    }

    AHCI_HBA_WRITE(((*Controller).IoBase).cast(), HbaInterruptStatus, InterruptStatus);

    return ((TRUE as i32)) as BOOLEAN;
}
