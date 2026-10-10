use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_158 {
    pub DeviceID: USHORT,
    pub Flags: USHORT,
}

static mut SvwSataControllerList: [C2RustUnnamed_158; 7] = [
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_K2_SATA as USHORT,
        Flags: HW_FLAGS_NO_ATAPI_DMA as USHORT,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_BCM5770_SATA as USHORT,
        Flags: (HW_FLAGS_NO_ATAPI_DMA | HW_FLAGS_8_PORTS) as USHORT,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_BCM5770R_SATA as USHORT,
        Flags: HW_FLAGS_NO_ATAPI_DMA as USHORT,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HT1000_SATA as USHORT,
        Flags: HW_FLAGS_NO_ATAPI_DMA as USHORT,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HT1000_SATA_2 as USHORT,
        Flags: HW_FLAGS_NO_ATAPI_DMA as USHORT,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HT1100_SATA as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HT1100_SATA_2 as USHORT,
        Flags: 0 as USHORT,
    },
];

pub const PCI_DEV_K2_SATA: u16 = 0x0240;
pub const PCI_DEV_BCM5770_SATA: u16 = 0x0241;
pub const PCI_DEV_BCM5770R_SATA: u16 = 0x0242;
pub const PCI_DEV_HT1000_SATA: u16 = 0x024A;
pub const PCI_DEV_HT1000_SATA_2: u16 = 0x024B;
pub const PCI_DEV_HT1100_SATA: u16 = 0x0410;
pub const PCI_DEV_HT1100_SATA_2: u16 = 0x0411;
pub const SVW_SATA_PORT_BASE_OFFSET: u32 = 0x100;
pub const SVW_SATA_PORT_CONTROL_OFFSET: u32 = 0x20;
pub const SVW_SATA_PORT_DMA_OFFSET: u32 = 0x30;
pub const SVW_SATA_PORT_SCR_OFFSET: u32 = 0x40;
pub const SVW_SATA_PORT_TF_STRIDE: u32 = 4;
pub const HW_FLAGS_8_PORTS: u16 = 0x01;
pub const HW_FLAGS_NO_ATAPI_DMA: u16 = 0x02;

#[repr(C)]
pub struct SvwSataControllerEntry {
    pub DeviceID: u16,
    pub Flags: u16,
}

unsafe extern "C" fn SvwSataScrRead(ChanData: PCHANNEL_DATA_PATA, Register: SATA_SCR_REGISTER, PortNumber: ULONG, Result: PULONG) -> BOOLEAN {
    UNREFERENCED_PARAMETER!(PortNumber);

    if Register as u32 > ATA_SCONTROL {
        return FALSE;
    }

    *Result = READ_REGISTER_ULONG(
        (*ChanData).Regs.Scr.offset((Register as u32).wrapping_mul(4) as isize) as PULONG
    );
    return TRUE;
}

unsafe extern "C" fn SvwSataScrWrite(ChanData: PCHANNEL_DATA_PATA, Register: SATA_SCR_REGISTER, PortNumber: ULONG, Value: ULONG) -> BOOLEAN {
    UNREFERENCED_PARAMETER!(PortNumber);

    if Register as ::core::ffi::c_uint > ATA_SCONTROL as ::core::ffi::c_uint {
        return FALSE;
    }

    WRITE_REGISTER_ULONG(
        (*ChanData).Regs.Scr.offset(
            (Register as ::core::ffi::c_uint).wrapping_mul(4 as ::core::ffi::c_uint) as isize,
        ) as PULONG,
        Value,
    );
    return TRUE;
}

unsafe extern "C" fn SvwSataGetIoBarIndex(Controller: PATA_CONTROLLER) -> ULONG {
    return if (*Controller).Pci.DeviceID == PCI_DEV_HT1100_SATA {
        3
    } else {
        5
    };
}

unsafe extern "C" fn SvwSataCheckInterrupt(mut ChanData: PCHANNEL_DATA_PATA) -> BOOLEAN {
    let mut Controller: PATA_CONTROLLER = (*ChanData).__bindgen_anon_1.Controller;
    let mut IoBase: PUCHAR = ((*Controller).AccessRange[SvwSataGetIoBarIndex(Controller) as usize].IoBase).cast();
    let mut InterruptStatus: ULONG = 0;
    InterruptStatus = READ_REGISTER_ULONG((IoBase.add(0x1F80 as usize)) as PULONG); // nolint: magic-offset (ServerWorks MMIO register, as the C)
    return ((InterruptStatus & ((1 as ULONG) << (*ChanData).__bindgen_anon_1.Channel)) != 0) as BOOLEAN;
}

unsafe extern "C" fn SvwSataReadStatus(ChanData: PCHANNEL_DATA_PATA) -> UCHAR {
    // C: `Regs.Status` -- the model took the Error/Features slot (port base+1).
    return READ_REGISTER_ULONG((*ChanData).Regs.__bindgen_anon_5.Status as *mut ULONG) as UCHAR;
}

unsafe extern "C" fn SvwSataLoadTaskFile(ChanData: PCHANNEL_DATA_PATA, Request: PATA_DEVICE_REQUEST) {
    let mut Features: USHORT;
    let mut SectorCount: USHORT;
    let mut LbaLow: USHORT;
    let mut LbaMid: USHORT;
    let mut LbaHigh: USHORT;

    Features = ((*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_2.Feature) as _;
    SectorCount = ((*Request).__bindgen_anon_2.TaskFile.SectorCount) as _;
    LbaLow = ((*Request).__bindgen_anon_2.TaskFile.LowLba) as _;
    LbaMid = ((*Request).__bindgen_anon_2.TaskFile.MidLba) as _;
    LbaHigh = ((*Request).__bindgen_anon_2.TaskFile.HighLba) as _;

    if (*Request).Flags & REQUEST_FLAG_LBA48 != 0 {
        Features = (Features as i32
            | ((*Request).__bindgen_anon_2.TaskFile.FeatureEx as USHORT as i32) << 8) as USHORT;
        SectorCount = (SectorCount as i32
            | ((*Request).__bindgen_anon_2.TaskFile.SectorCountEx as USHORT as i32) << 8) as USHORT;
        LbaLow = (LbaLow as i32
            | ((*Request).__bindgen_anon_2.TaskFile.LowLbaEx as USHORT as i32) << 8) as USHORT;
        LbaMid = (LbaMid as i32
            | ((*Request).__bindgen_anon_2.TaskFile.MidLbaEx as USHORT as i32) << 8) as USHORT;
        LbaHigh = (LbaHigh as i32
            | ((*Request).__bindgen_anon_2.TaskFile.HighLbaEx as USHORT as i32) << 8) as USHORT;
    }

    WRITE_REGISTER_USHORT((*ChanData).Regs.__bindgen_anon_1.Features as *mut u16, Features);
    WRITE_REGISTER_USHORT((*ChanData).Regs.__bindgen_anon_2.SectorCount as *mut u16, SectorCount);
    WRITE_REGISTER_USHORT((*ChanData).Regs.LbaLow as *mut u16, LbaLow);
    WRITE_REGISTER_USHORT((*ChanData).Regs.__bindgen_anon_3.LbaMid as *mut u16, LbaMid);
    WRITE_REGISTER_USHORT((*ChanData).Regs.__bindgen_anon_4.LbaHigh as *mut u16, LbaHigh);

    if (*Request).Flags & REQUEST_FLAG_SET_DEVICE_REGISTER != 0 {
        WRITE_REGISTER_UCHAR((*ChanData).Regs.Device, (*Request).__bindgen_anon_2.TaskFile.DriveSelect);
    }
}

unsafe extern "C" fn SvwSataSaveTaskFile(ChanData: PCHANNEL_DATA_PATA, Request: PATA_DEVICE_REQUEST) {
    let TaskFile: PATA_TASKFILE = &raw mut (*Request).Output;
    let mut Error: USHORT;
    let mut SectorCount: USHORT;
    let mut LbaLow: USHORT;
    let mut LbaMid: USHORT;
    let mut LbaHigh: USHORT;

    (*TaskFile).DriveSelect = READ_REGISTER_UCHAR((*ChanData).Regs.Device);
    (*TaskFile).__bindgen_anon_1.Command = READ_REGISTER_UCHAR((*ChanData).Regs.__bindgen_anon_5.Command);

    Error = READ_REGISTER_USHORT((*ChanData).Regs.__bindgen_anon_1.Error as PUSHORT);
    SectorCount = READ_REGISTER_USHORT((*ChanData).Regs.__bindgen_anon_2.SectorCount as PUSHORT);
    LbaLow = READ_REGISTER_USHORT((*ChanData).Regs.LbaLow as PUSHORT);
    LbaMid = READ_REGISTER_USHORT((*ChanData).Regs.__bindgen_anon_3.LbaMid as PUSHORT);
    LbaHigh = READ_REGISTER_USHORT((*ChanData).Regs.__bindgen_anon_4.LbaHigh as PUSHORT);

    (*TaskFile).__bindgen_anon_2.Error = (Error & 0xFF) as UCHAR;
    (*TaskFile).SectorCount = (SectorCount & 0xFF) as UCHAR;
    (*TaskFile).LowLba = (LbaLow & 0xFF) as UCHAR;
    (*TaskFile).MidLba = (LbaMid & 0xFF) as UCHAR;
    (*TaskFile).HighLba = (LbaHigh & 0xFF) as UCHAR;

    if (*Request).Flags & REQUEST_FLAG_LBA48 != 0 {
        (*TaskFile).FeatureEx = (Error >> 8) as UCHAR;
        (*TaskFile).SectorCountEx = (SectorCount >> 8) as UCHAR;
        (*TaskFile).LowLbaEx = (LbaLow >> 8) as UCHAR;
        (*TaskFile).MidLbaEx = (LbaMid >> 8) as UCHAR;
        (*TaskFile).HighLbaEx = (LbaHigh >> 8) as UCHAR;
    }
}

unsafe extern "C" fn SvwSataParseResources(
    mut ChanData: PCHANNEL_DATA_PATA,
    mut IoBase: PUCHAR,
    mut PortIndex: ULONG,
) {
    let mut ChanBase: PUCHAR = IoBase.offset((PortIndex as usize).wrapping_mul((SVW_SATA_PORT_BASE_OFFSET) as _) as isize);

    PAGED_CODE!();

    (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_MRES_TF | CHANNEL_FLAG_MRES_CTRL | CHANNEL_FLAG_MRES_DMA;

    (*ChanData).Regs.Dma = ChanBase.offset(SVW_SATA_PORT_DMA_OFFSET as isize);
    (*ChanData).Regs.Scr = ChanBase.offset(SVW_SATA_PORT_SCR_OFFSET as isize);

    PciIdeInitTaskFileIoResources(
        ChanData,
        (ChanBase as ULONG_PTR) as _,
        ((ChanBase as ULONG_PTR).wrapping_add(SVW_SATA_PORT_CONTROL_OFFSET as ULONG_PTR)) as _,
        SVW_SATA_PORT_TF_STRIDE,
    );
}
#[no_mangle]
pub unsafe extern "C" fn SvwSataGetControllerProperties(
    Controller: PATA_CONTROLLER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut i: ULONG;
    let mut HwFlags: ULONG = core::mem::zeroed();
    let mut IoBase: PUCHAR;

    PAGED_CODE!();
    ASSERT!((*Controller).Pci.VendorID as u32 == PCI_VEN_SERVERWORKS);

    i = 0;
    while i < RTL_NUMBER_OF!(SvwSataControllerList) {
        HwFlags = SvwSataControllerList[i as usize].Flags as ULONG;

        if (*Controller).Pci.DeviceID == SvwSataControllerList[i as usize].DeviceID {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i == RTL_NUMBER_OF!(SvwSataControllerList) {
        return STATUS_NO_MATCH;
    }

    if HwFlags & (HW_FLAGS_8_PORTS as u32) != 0 {
        (*Controller).MaxChannels = 8;
    } else {
        (*Controller).MaxChannels = 4;
    }

    (*Controller).Flags |= CTRL_FLAG_MANUAL_RES;

    Status = PciIdeCreateChannelData(Controller, 0);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    IoBase = (AtaCtrlPciMapBar(Controller, SvwSataGetIoBarIndex(Controller), 0)).cast();
    if IoBase.is_null() {
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }

    WRITE_REGISTER_ULONG(
        IoBase.offset(0x80 as isize) as PULONG, // nolint: magic-offset (ServerWorks MMIO register, as the C)
        READ_REGISTER_ULONG(IoBase.offset(0x80 as isize) as PULONG) & !0x00040000, // nolint: magic-offset (ServerWorks MMIO register, as the C)
    );

    WRITE_REGISTER_ULONG(IoBase.offset(0x44 as isize) as PULONG, 0xFFFFFFFF); // nolint: magic-offset (ServerWorks MMIO register, as the C)
    WRITE_REGISTER_ULONG(IoBase.offset(0x88 as isize) as PULONG, 0); // nolint: magic-offset (ServerWorks MMIO register, as the C)

    i = 0;
    while i < (*Controller).MaxChannels {
        let ChanData: PCHANNEL_DATA_PATA = ((*Controller).Channels[i as usize]).cast();

        (*ChanData).__bindgen_anon_1.TransferModeSupported = SATA_ALL;
        (*ChanData).__bindgen_anon_1.SetTransferMode = Some(SataSetTransferMode);

        if HwFlags & (HW_FLAGS_NO_ATAPI_DMA as u32) != 0 {
            (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_NO_ATAPI_DMA;
        }

        (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_DMA_BEFORE_CMD;

        (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_NO_SLAVE;
        (*ChanData).ReadStatus = Some(SvwSataReadStatus);
        (*ChanData).LoadTaskFile = Some(SvwSataLoadTaskFile);
        (*ChanData).SaveTaskFile = Some(SvwSataSaveTaskFile);
        (*ChanData).CheckInterrupt = Some(SvwSataCheckInterrupt);
        (*ChanData).ScrRead = Some(SvwSataScrRead);
        (*ChanData).ScrWrite = Some(SvwSataScrWrite);

        SvwSataParseResources(ChanData, IoBase, i);
        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}
