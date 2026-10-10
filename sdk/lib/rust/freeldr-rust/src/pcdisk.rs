#[allow(unused_imports)] use crate::{ERR, TRACE, WARN};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_DISK; // pcdisk.c: DBG_DEFAULT_CHANNEL(DISK)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PCSTR};
// freeldr_ddk_imports dropped (not in ddk): ErrorCode
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_CdromController as CdromController;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_DiskPeripheral as DiskPeripheral;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_FloppyDiskPeripheral as FloppyDiskPeripheral;

// pcdisk.c dropped content (rustconv drops non-fn). pshpack1 packets +
// PC_DISK_DRIVE (CACHE_MULTI_DRIVES is OFF → single-drive variant).
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct I386_DISK_ADDRESS_PACKET {
    pub PacketSize: UCHAR,
    pub Reserved: UCHAR,
    pub LBABlockCount: USHORT,
    pub TransferBufferOffset: USHORT,
    pub TransferBufferSegment: USHORT,
    pub LBAStartBlock: ULONGLONG,
}
pub type PI386_DISK_ADDRESS_PACKET = *mut I386_DISK_ADDRESS_PACKET;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct I386_CDROM_SPEC_PACKET {
    pub PacketSize: UCHAR,
    pub MediaType: UCHAR,
    pub DriveNumber: UCHAR,
    pub Controller: UCHAR,
    pub LBAImage: ULONG,
    pub DeviceSpec: USHORT,
    pub Buffer: USHORT,
    pub LoadSeg: USHORT,
    pub SectorCount: USHORT,
    pub CHSGeometry: [UCHAR; 3],
    pub Reserved: UCHAR,
}
pub type PI386_CDROM_SPEC_PACKET = *mut I386_CDROM_SPEC_PACKET;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct EXTENDED_GEOMETRY {
    pub Size: USHORT,
    pub Flags: USHORT,
    pub Cylinders: ULONG,
    pub Heads: ULONG,
    pub SectorsPerTrack: ULONG,
    pub Sectors: ULONGLONG,
    pub BytesPerSector: USHORT,
    pub PDPTE: ULONG,
}
pub type PEXTENDED_GEOMETRY = *mut EXTENDED_GEOMETRY;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PC_DISK_DRIVE {
    pub Geometry: _GEOMETRY,
    pub ExtGeometry: EXTENDED_GEOMETRY,
    pub Int13ExtensionsSupported: BOOLEAN,
    pub IsRemovable: BOOLEAN,
}
pub type PPC_DISK_DRIVE = *mut PC_DISK_DRIVE;

static mut LastDriveNumber: USHORT = 0xFFFF;
static mut PcDiskDrive: PC_DISK_DRIVE = PC_DISK_DRIVE {
    Geometry: _GEOMETRY { Cylinders: 0, Heads: 0, SectorsPerTrack: 0, BytesPerSector: 0, Sectors: 0 },
    ExtGeometry: EXTENDED_GEOMETRY { Size: 0, Flags: 0, Cylinders: 0, Heads: 0, SectorsPerTrack: 0, Sectors: 0, BytesPerSector: 0, PDPTE: 0 },
    Int13ExtensionsSupported: 0,
    IsRemovable: 0,
};

// rustconv emitted underscore-prefixed names for these pcdisk.c structs.
pub type _I386_DISK_ADDRESS_PACKET = I386_DISK_ADDRESS_PACKET;
pub type _PC_DISK_DRIVE = PC_DISK_DRIVE;

// pcdisk.c file-local error-report counter (dropped by rustconv).
static mut lReportError: LONG = 0;

extern "C" {
    // uefidisk.c external freeldr globals (machpc.h declarations).
    static FrldrBootDrive: UCHAR;
    static FrldrBootPartition: ULONG;
}

extern "C" {
    fn WRITE_PORT_UCHAR(Port: *mut u8, Value: u8);
    fn sprintf(s: *mut i8, format: PCSTR, ...) -> i32;
    fn UiMessageBox(message: PCSTR);
    fn INT386_SUCCESS(regs: REGS) -> u8;
    fn Int386(interrupt: i32, in_regs: *mut REGS, out_regs: *mut REGS);
}

#[no_mangle]
pub unsafe extern "C" fn DiskReportError(bShowError: BOOLEAN) -> LONG {
    if bShowError != 0 {
        lReportError += 1;
    } else {
        lReportError -= 1;
    }
    lReportError
}

unsafe fn DiskGetErrorCodeString(ErrorCode: u32) -> PCSTR {
    match ErrorCode {
        0x00 => b"no error\0".as_ptr() as PCSTR,
        0x01 => b"bad command passed to driver\0".as_ptr() as PCSTR,
        0x02 => b"address mark not found or bad sector\0".as_ptr() as PCSTR,
        0x03 => b"diskette write protect error\0".as_ptr() as PCSTR,
        0x04 => b"sector not found\0".as_ptr() as PCSTR,
        0x05 => b"fixed disk reset failed\0".as_ptr() as PCSTR,
        0x06 => b"diskette changed or removed\0".as_ptr() as PCSTR,
        0x07 => b"bad fixed disk parameter table\0".as_ptr() as PCSTR,
        0x08 => b"DMA overrun\0".as_ptr() as PCSTR,
        0x09 => b"DMA access across 64k boundary\0".as_ptr() as PCSTR,
        0x0A => b"bad fixed disk sector flag\0".as_ptr() as PCSTR,
        0x0B => b"bad fixed disk cylinder\0".as_ptr() as PCSTR,
        0x0C => b"unsupported track/invalid media\0".as_ptr() as PCSTR,
        0x0D => b"invalid number of sectors on fixed disk format\0".as_ptr() as PCSTR,
        0x0E => b"fixed disk controlled data address mark detected\0".as_ptr() as PCSTR,
        0x0F => b"fixed disk DMA arbitration level out of range\0".as_ptr() as PCSTR,
        0x10 => b"ECC/CRC error on disk read\0".as_ptr() as PCSTR,
        0x11 => b"recoverable fixed disk data error, data fixed by ECC\0".as_ptr() as PCSTR,
        0x20 => b"controller error (NEC for floppies)\0".as_ptr() as PCSTR,
        0x40 => b"seek failure\0".as_ptr() as PCSTR,
        0x80 => b"time out, drive not ready\0".as_ptr() as PCSTR,
        0xAA => b"fixed disk drive not ready\0".as_ptr() as PCSTR,
        0xBB => b"fixed disk undefined error\0".as_ptr() as PCSTR,
        0xCC => b"fixed disk write fault on selected drive\0".as_ptr() as PCSTR,
        0xE0 => b"fixed disk status error/Error reg = 0\0".as_ptr() as PCSTR,
        0xFF => b"sense operation failed\0".as_ptr() as PCSTR,
        _ => b"unknown error code\0".as_ptr() as PCSTR,
    }
}

unsafe fn DiskError(ErrorString: PCSTR, ErrorCode: u32) {
    let mut ErrorCodeString: [i8; 200] = [0; 200];

    if lReportError < 0 {
        return;
    }

    sprintf(
        ErrorCodeString.as_mut_ptr(),
        b"%s\n\nError Code: 0x%lx\nError: %s\0".as_ptr() as PCSTR,
        ErrorString,
        ErrorCode,
        DiskGetErrorCodeString(ErrorCode)
    );

    ERR!("%s\n", (ErrorCodeString.as_ptr()) as usize);
    UiMessageBox(ErrorCodeString.as_ptr() as PCSTR);
}

#[no_mangle]
pub unsafe extern "C" fn DiskResetController(DriveNumber: u8) -> u8 {
    let mut RegsIn: REGS = core::mem::zeroed();
    let mut RegsOut: REGS = core::mem::zeroed();

    WARN!("DiskResetController(0x%x) DISK OPERATION FAILED -- RESETTING CONTROLLER\n", (DriveNumber) as usize);

    RegsIn.b.ah = 0x00;
    RegsIn.b.dl = DriveNumber;

    Int386(0x13, &mut RegsIn, &mut RegsOut);

    INT386_SUCCESS(RegsOut)
}

unsafe fn DiskIsDriveRemovable(DriveNumber: u8) -> u8 {
    if (DriveNumber >= 0x80) || (DriveNumber == 0x49) {
        return 0u8;
    }
    1u8
}

unsafe fn DiskInt13ExtensionsSupported(DriveNumber: UCHAR) -> BOOLEAN {
    let mut RegsIn: REGS = core::mem::zeroed();
    let mut RegsOut: REGS = core::mem::zeroed();

    if DriveNumber >= 0x8A {
        return 1u8;
    }

    RegsIn.b.ah = 0x41;
    RegsIn.w.bx = 0x55AA;
    RegsIn.b.dl = DriveNumber;

    Int386(0x13, &mut RegsIn, &mut RegsOut);
    if INT386_SUCCESS(RegsOut) == 0 {
        return 0u8;
    }

    if RegsOut.w.bx != 0xAA55 {
        return 0u8;
    }

    #[cfg(debug_assertions)]
    TRACE!("Drive 0x%x: INT 13h Extended version: 0x%02x, API bitmap: 0x%04x\n", (DriveNumber) as usize, (RegsOut.b.ah) as usize, (RegsOut.w.cx) as usize);

    if (RegsOut.w.cx & 0x0001) == 0 {
        WARN!("Drive 0x%x: Suspicious API subset support bitmap 0x%04x\n", (DriveNumber) as usize, (RegsOut.w.cx) as usize);
        return 0u8;
    }

    1u8
}

unsafe fn DiskGetExtendedDriveParameters(
    DriveNumber: UCHAR,
    DiskDrive: PPC_DISK_DRIVE,
    Buffer: PVOID,
    BufferSize: USHORT,
) -> BOOLEAN {
    let Ptr = BIOSCALLBUFFER as PEXTENDED_GEOMETRY;
    let mut RegsIn: REGS = core::mem::zeroed();
    let mut RegsOut: REGS = core::mem::zeroed();

    TRACE!("DiskGetExtendedDriveParameters(0x%x)\n", (DriveNumber) as usize);

    if (*DiskDrive).Int13ExtensionsSupported == 0 || (BufferSize < core::mem::size_of::<EXTENDED_GEOMETRY>() as USHORT) {
        return 0u8;
    }

    core::ptr::write_bytes(Ptr as *mut u8, 0, core::mem::size_of::<EXTENDED_GEOMETRY>());
    (*Ptr).Size = BufferSize;

    RegsIn.b.ah = 0x48;
    RegsIn.b.dl = DriveNumber;
    RegsIn.x.ds = (BIOSCALLBUFSEGMENT) as u16 ;
    RegsIn.w.si = (BIOSCALLBUFOFFSET) as u16 ;

    Int386(0x13, &mut RegsIn, &mut RegsOut);
    if INT386_SUCCESS(RegsOut) == 0 {
        return 0u8;
    }

    core::ptr::copy_nonoverlapping(Ptr as *const u8, Buffer as *mut u8, BufferSize as usize);

    #[cfg(debug_assertions)]
    {
        TRACE!("Size of buffer:                          0x%x\n", ((*Ptr).Size) as usize);
        TRACE!("Information flags:                       0x%x\n", ((*Ptr).Flags) as usize);
        TRACE!("Number of physical cylinders on drive:   %u\n", ((*Ptr).Cylinders) as usize);
        TRACE!("Number of physical heads on drive:       %u\n", ((*Ptr).Heads) as usize);
        TRACE!("Number of physical sectors per track:    %u\n", ((*Ptr).SectorsPerTrack) as usize);
        TRACE!("Total number of sectors on drive:        %I64u\n", ((*Ptr).Sectors) as usize);
        TRACE!("Bytes per sector:                        %u\n", ((*Ptr).BytesPerSector) as usize);
        if (*Ptr).Size >= 0x1E {
            let Off = ((*Ptr).PDPTE & 0xFFFF) as USHORT;
            let Seg = (((*Ptr).PDPTE >> 16) & 0xFFFF) as USHORT;
            TRACE!("EDD configuration parameters (DPTE):     %x:%x\n", (Seg) as usize, (Off) as usize);

            if (*Ptr).PDPTE != 0xFFFFFFFF && (*Ptr).PDPTE != 0 {
                let SpecPtr = (((Seg as ULONG_PTR) << 4) + Off as ULONG_PTR) as PUCHAR;
                TRACE!("SpecPtr:                                 0x%x\n", (SpecPtr) as usize);
                TRACE!("Physical I/O port base address:          0x%x\n", (*(SpecPtr.offset(0) as *const USHORT)) as usize);
                TRACE!("Disk-drive control port address:         0x%x\n", (*(SpecPtr.offset(2) as *const USHORT)) as usize);
                TRACE!("Head register upper nibble:              0x%x\n", (*SpecPtr.offset(4)) as usize);
                TRACE!("BIOS Vendor-specific:                    0x%x\n", (*SpecPtr.offset(5)) as usize);
                TRACE!("IRQ for drive:                           %u\n", (*SpecPtr.offset(6)) as usize);
                TRACE!("Sector count for multi-sector transfers: %u\n", (*SpecPtr.offset(7)) as usize);
                TRACE!("DMA control:                             0x%x\n", (*SpecPtr.offset(8)) as usize);
                TRACE!("Programmed I/O control:                  0x%x\n", (*SpecPtr.offset(9)) as usize);
                TRACE!("Drive options:                           0x%x\n", (*(SpecPtr.offset(10) as *const USHORT)) as usize);
            }
        }
        if (*Ptr).Size >= 0x42 {
            TRACE!("Signature:                             0x%x\n", (*((Ptr as *const USHORT).offset(15))) as usize);
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn DiskGetConfigType(DriveNumber: u8) -> CONFIGURATION_TYPE {
    if (DriveNumber == FrldrBootDrive) && (FrldrBootPartition == 0xFF) {
        return CdromController;
    } else if DiskIsDriveRemovable(DriveNumber) != 0 {
        return FloppyDiskPeripheral;
    } else {
        return DiskPeripheral;
    }
}

unsafe fn InitDriveGeometry(
    DriveNumber: UCHAR,
    DiskDrive: PPC_DISK_DRIVE,
) -> BOOLEAN {
    let mut Success: BOOLEAN;
    let mut RegsIn: REGS = core::mem::zeroed();
    let mut RegsOut: REGS = core::mem::zeroed();
    let mut Cylinders: ULONG;

    RtlZeroMemory(
        core::ptr::addr_of_mut!((*DiskDrive).ExtGeometry) as PVOID,
        core::mem::size_of_val(&(*DiskDrive).ExtGeometry) as SIZE_T,
    );
    Success = DiskGetExtendedDriveParameters(
        DriveNumber,
        DiskDrive,
        core::ptr::addr_of_mut!((*DiskDrive).ExtGeometry) as PVOID,
        core::mem::size_of_val(&(*DiskDrive).ExtGeometry) as USHORT,
    );
    if Success != 0 {
        TRACE!("DiskGetExtendedDriveParameters(0x%x) returned:\nCylinders  : 0x%x\nHeads      : 0x%x\nSects/Track: 0x%x\nTotal Sects: 0x%llx\nBytes/Sect : 0x%x\n", (DriveNumber) as usize, ((*DiskDrive).ExtGeometry.Cylinders) as usize, ((*DiskDrive).ExtGeometry.Heads) as usize, ((*DiskDrive).ExtGeometry.SectorsPerTrack) as usize, ((*DiskDrive).ExtGeometry.Sectors) as usize, ((*DiskDrive).ExtGeometry.BytesPerSector) as usize);
    }

    RtlZeroMemory(
        core::ptr::addr_of_mut!((*DiskDrive).Geometry) as PVOID,
        core::mem::size_of_val(&(*DiskDrive).Geometry) as SIZE_T,
    );

    RegsIn.b.ah = 0x08;
    RegsIn.b.dl = DriveNumber;
    RegsIn.w.es = 0x0000;
    RegsIn.w.di = 0x0000;

    Int386(0x13, &mut RegsIn, &mut RegsOut);
    if INT386_SUCCESS(RegsOut) == 0 {
        return Success;
    }
    Success |= 1u8;

    Cylinders = ((RegsOut.b.cl & 0xC0) as ULONG) << 2;
    Cylinders += RegsOut.b.ch as ULONG;
    Cylinders += 1;
    (*DiskDrive).Geometry.Cylinders = Cylinders;
    (*DiskDrive).Geometry.Heads = (RegsOut.b.dh as ULONG) + 1;
    (*DiskDrive).Geometry.SectorsPerTrack = (RegsOut.b.cl & 0x3F) as ULONG;
    (*DiskDrive).Geometry.BytesPerSector = 512;

    (*DiskDrive).Geometry.Sectors = ((*DiskDrive).Geometry.Cylinders as ULONGLONG)
        * ((*DiskDrive).Geometry.Heads as ULONGLONG)
        * ((*DiskDrive).Geometry.SectorsPerTrack as ULONGLONG);

    TRACE!("Regular Int13h(0x%x) returned:\nCylinders  : 0x%x\nHeads      : 0x%x\nSects/Track: 0x%x\nTotal Sects: 0x%llx\nBytes/Sect : 0x%x\n", (DriveNumber) as usize, ((*DiskDrive).Geometry.Cylinders) as usize, ((*DiskDrive).Geometry.Heads) as usize, ((*DiskDrive).Geometry.SectorsPerTrack) as usize, ((*DiskDrive).Geometry.Sectors) as usize, ((*DiskDrive).Geometry.BytesPerSector) as usize);

    Success
}

unsafe fn PcDiskDriveInit(
    DriveNumber: UCHAR,
    DiskDrive: PPC_DISK_DRIVE,
) -> BOOLEAN {
    (*DiskDrive).IsRemovable = DiskIsDriveRemovable(DriveNumber);

    (*DiskDrive).Int13ExtensionsSupported = DiskInt13ExtensionsSupported(DriveNumber);

    if InitDriveGeometry(DriveNumber, DiskDrive) == 0 {
        return 0u8;
    }

    TRACE!("\nDriveNumber: 0x%x\nIsRemovable              = %s\nInt13ExtensionsSupported = %s\n", (DriveNumber) as usize, (if (*DiskDrive).IsRemovable != 0 { b"TRUE\0".as_ptr() } else { b"FALSE\0".as_ptr() }) as usize, (if (*DiskDrive).Int13ExtensionsSupported != 0 { b"TRUE\0".as_ptr() } else { b"FALSE\0".as_ptr() }) as usize);

    1u8
}

unsafe fn PcDiskDriveNumberToDrive(DriveNumber: UCHAR) -> PPC_DISK_DRIVE {
#[cfg(feature = "CACHE_MULTI_DRIVES")]
    {
        let DiskDrive: PPC_DISK_DRIVE;

        DiskDrive = &mut PcDiskDrive[DriveNumber as usize];

        if (*DiskDrive).Initialized == 0 {
            if PcDiskDriveInit(DriveNumber, DiskDrive) == 0 {
                (*DiskDrive).Initialized |= 0x80;
                return core::ptr::null_mut();
            }
            (*DiskDrive).Initialized = 1u8;
        } else if (*DiskDrive).Initialized & 0x80 != 0 {
            (*DiskDrive).Initialized = 0u8;
            return core::ptr::null_mut();
        }

        return DiskDrive;
    }

#[cfg(not(feature = "CACHE_MULTI_DRIVES"))]
    {
        static mut NewDiskDrive: _PC_DISK_DRIVE = unsafe { core::mem::zeroed() };

        if DriveNumber as USHORT != LastDriveNumber {
            core::ptr::write_bytes(&mut NewDiskDrive as *mut _, 0u8, 1);
            if PcDiskDriveInit(DriveNumber, &mut NewDiskDrive) == 0 {
                return core::ptr::null_mut();
            }
            PcDiskDrive = NewDiskDrive;
            LastDriveNumber = DriveNumber as USHORT;
        }

        return &mut PcDiskDrive;
    }
}

unsafe fn PcDiskReadLogicalSectorsLBA(
    DriveNumber: u8,
    SectorNumber: u64,
    SectorCount: u32,
    Buffer: *mut core::ffi::c_void,
) -> u8 {

    let Packet: *mut _I386_DISK_ADDRESS_PACKET = BIOSCALLBUFFER as *mut _I386_DISK_ADDRESS_PACKET;
    let mut RegsIn: REGS = core::mem::zeroed();
    let mut RegsOut: REGS = core::mem::zeroed();
    let mut RetryCount: u32;

    RtlZeroMemory(Packet as *mut core::ffi::c_void, core::mem::size_of::<_I386_DISK_ADDRESS_PACKET>());
    (*Packet).PacketSize = core::mem::size_of::<_I386_DISK_ADDRESS_PACKET>() as u8;
    (*Packet).Reserved = 0;
    (*Packet).TransferBufferOffset = (Buffer as usize & 0x0F) as u16;
    (*Packet).TransferBufferSegment = ((Buffer as usize) >> 4) as u16;
    (*Packet).LBAStartBlock = SectorNumber;

    RegsIn.b.ah = 0x42;
    RegsIn.b.dl = DriveNumber;
    RegsIn.x.ds = (BIOSCALLBUFSEGMENT) as u16 ;
    RegsIn.w.si = (BIOSCALLBUFOFFSET) as u16 ;

    RetryCount = 0;
    while RetryCount < 3 {
        (*Packet).LBABlockCount = SectorCount as u16;

        Int386(0x13, &mut RegsIn, &mut RegsOut);

        if INT386_SUCCESS(RegsOut) != 0 || RegsOut.b.ah == 0x11 {
            return 1u8;
        }

        DiskResetController(DriveNumber);
        RetryCount += 1;
    }

    DiskError(b"Disk Read Failed in LBA mode\0".as_ptr() as PCSTR, RegsOut.b.ah as u32);
    ERR!("Disk Read Failed in LBA mode: %x (%s) (DriveNumber: 0x%x SectorNumber: %I64u SectorCount: %u)\n", (RegsOut.b.ah as i32) as usize, (DiskGetErrorCodeString(RegsOut.b.ah as u32)) as usize, (DriveNumber as i32) as usize, (SectorNumber) as usize, (SectorCount) as usize);

    0u8
}

unsafe fn PcDiskReadLogicalSectorsCHS(
    DriveNumber: u8,
    DiskDrive: *mut PC_DISK_DRIVE,
    mut SectorNumber: u64,
    mut SectorCount: u32,
    mut Buffer: *mut core::ffi::c_void,
) -> u8 {
    let mut PhysicalSector: u8;
    let mut PhysicalHead: u8;
    let mut PhysicalTrack: u32;
    let DriveGeometry: GEOMETRY;
    let mut NumberOfSectorsToRead: u32;
    let mut RegsIn: REGS = core::mem::zeroed();
    let mut RegsOut: REGS = core::mem::zeroed();
    let mut RetryCount: u32;

    DriveGeometry = (*DiskDrive).Geometry;
    if DriveGeometry.SectorsPerTrack == 0 || DriveGeometry.Heads == 0 {
        return 0u8;
    }

    while SectorCount > 0 {
        PhysicalSector = 1 + (SectorNumber % DriveGeometry.SectorsPerTrack as u64) as u8;
        PhysicalHead = ((SectorNumber / DriveGeometry.SectorsPerTrack as u64) % DriveGeometry.Heads as u64) as u8;
        PhysicalTrack = ((SectorNumber / DriveGeometry.SectorsPerTrack as u64) / DriveGeometry.Heads as u64) as u32;

        if PhysicalSector > 1 {
            NumberOfSectorsToRead = min(SectorCount, DriveGeometry.SectorsPerTrack - (PhysicalSector as u32 - 1));
        } else {
            NumberOfSectorsToRead = min(SectorCount, DriveGeometry.SectorsPerTrack);
        }

        if (PhysicalHead >= DriveGeometry.Heads as u8) ||
           (PhysicalTrack >= DriveGeometry.Cylinders) ||
           ((NumberOfSectorsToRead + PhysicalSector as u32) > (DriveGeometry.SectorsPerTrack + 1)) ||
           (PhysicalSector > DriveGeometry.SectorsPerTrack as u8) {
            DiskError(b"Disk read exceeds drive geometry limits.\0".as_ptr() as PCSTR, 0);
            return 0u8;
        }

        RegsIn.b.ah = 0x02;
        RegsIn.b.al = NumberOfSectorsToRead as u8;
        RegsIn.b.ch = (PhysicalTrack & 0xFF) as u8;
        RegsIn.b.cl = (PhysicalSector + ((PhysicalTrack & 0x300) >> 2) as u8) as u8;
        RegsIn.b.dh = PhysicalHead;
        RegsIn.b.dl = DriveNumber;
        RegsIn.w.es = ((Buffer as usize) >> 4) as u16;
        RegsIn.w.bx = ((Buffer as usize) & 0x0F) as u16;

        RetryCount = 0;
        while RetryCount < 3 {
            Int386(0x13, &mut RegsIn, &mut RegsOut);

            if INT386_SUCCESS(RegsOut) != 0 || RegsOut.b.ah == 0x11 {
                break;
            }

            DiskResetController(DriveNumber);
            RetryCount += 1;
        }

        if RetryCount >= 3 {
            DiskError(b"Disk Read Failed in CHS mode, after retrying 3 times\0".as_ptr() as PCSTR, RegsOut.b.ah as u32);
            ERR!("Disk Read Failed in CHS mode, after retrying 3 times: %x (%s) (DriveNumber: 0x%x SectorNumber: %I64u SectorCount: %u)\n", (RegsOut.b.ah) as usize, (DiskGetErrorCodeString(RegsOut.b.ah as u32)) as usize, (DriveNumber) as usize, (SectorNumber) as usize, (SectorCount) as usize);
            return 0u8;
        }

        Buffer = (Buffer as usize + (NumberOfSectorsToRead * DriveGeometry.BytesPerSector) as usize) as *mut core::ffi::c_void;
        SectorCount -= NumberOfSectorsToRead;
        SectorNumber += NumberOfSectorsToRead as u64;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn PcDiskReadLogicalSectors(
    DriveNumber: UCHAR,
    SectorNumber: ULONGLONG,
    SectorCount: ULONG,
    Buffer: PVOID,
) -> BOOLEAN {

    let DiskDrive: PPC_DISK_DRIVE;

    TRACE!("PcDiskReadLogicalSectors() DriveNumber: 0x%x SectorNumber: %I64u SectorCount: %u Buffer: 0x%x\n", (DriveNumber) as usize, (SectorNumber) as usize, (SectorCount) as usize, (Buffer) as usize);

    DiskDrive = PcDiskDriveNumberToDrive(DriveNumber);
    if DiskDrive.is_null() {
        return 0u8;
    }

    if (DriveNumber >= 0x80) && (*DiskDrive).Int13ExtensionsSupported != 0 {
        TRACE!("--> Using LBA\n");
        return PcDiskReadLogicalSectorsLBA(DriveNumber, SectorNumber, SectorCount, Buffer);
    } else {
        TRACE!("--> Using CHS\n");
        return PcDiskReadLogicalSectorsCHS(DriveNumber, DiskDrive, SectorNumber, SectorCount, Buffer);
    }
}

#[no_mangle]
pub unsafe extern "C" fn DiskStopFloppyMotor() {
    WRITE_PORT_UCHAR(0x3F2 as *mut u8, 0);
}

#[no_mangle]
pub unsafe extern "C" fn PcDiskGetDriveGeometry(DriveNumber: UCHAR, Geometry: PGEOMETRY) -> BOOLEAN {
    let DiskDrive: PPC_DISK_DRIVE;

    TRACE!("PcDiskGetDriveGeometry(0x%x)\n", (DriveNumber) as usize);

    DiskDrive = PcDiskDriveNumberToDrive(DriveNumber);
    if DiskDrive.is_null() {
        return 0u8;
    }

    if (*DiskDrive).ExtGeometry.Size == core::mem::size_of_val(&(*DiskDrive).ExtGeometry) as u16 {
        (*Geometry).Cylinders = (*DiskDrive).ExtGeometry.Cylinders;
        (*Geometry).Heads = (*DiskDrive).ExtGeometry.Heads;
        (*Geometry).SectorsPerTrack = (*DiskDrive).ExtGeometry.SectorsPerTrack;
        (*Geometry).BytesPerSector = ((*DiskDrive).ExtGeometry.BytesPerSector) as u32 ;
        (*Geometry).Sectors = (*DiskDrive).ExtGeometry.Sectors;
    } else {
        *Geometry = (*DiskDrive).Geometry;
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn PcDiskGetCacheableBlockCount(DriveNumber: UCHAR) -> ULONG {
    let DiskDrive: PPC_DISK_DRIVE;

    DiskDrive = PcDiskDriveNumberToDrive(DriveNumber);
    if DiskDrive.is_null() {
        return 1;
    }

    if (*DiskDrive).Int13ExtensionsSupported != 0 {
        return 64;
    } else {
        return (*DiskDrive).Geometry.SectorsPerTrack;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 1998-2003  Brian Palmer  <brianp@sginet.com>
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// // #if defined(__i386__) || defined(_M_AMD64)
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(DISK);
//
// /* Enable this line if you want to support multi-drive caching (increases FreeLdr size!) */
// // #define CACHE_MULTI_DRIVES
//
// #include <pshpack1.h>
//
// typedef struct
// {
//     UCHAR       PacketSize;             // 00h - Size of packet (10h or 18h)
//     UCHAR       Reserved;               // 01h - Reserved (0)
//     USHORT      LBABlockCount;          // 02h - Number of blocks to transfer (max 007Fh for Phoenix EDD)
//     USHORT      TransferBufferOffset;   // 04h - Transfer buffer offset (seg:off)
//     USHORT      TransferBufferSegment;  //       Transfer buffer segment (seg:off)
//     ULONGLONG   LBAStartBlock;          // 08h - Starting absolute block number
// //  ULONGLONG   TransferBuffer64;       // 10h - (EDD-3.0, optional) 64-bit flat address of transfer buffer
//                                         //       used if DWORD at 04h is FFFFh:FFFFh
//                                         //       Commented since some earlier BIOSes refuse to work with
//                                         //       such extended structure
// } I386_DISK_ADDRESS_PACKET, *PI386_DISK_ADDRESS_PACKET;
//
// typedef struct
// {
//     UCHAR   PacketSize;     // 00h - Size of packet in bytes (13h)
//     UCHAR   MediaType;      // 01h - Boot media type (see #00282)
//     UCHAR   DriveNumber;    /* 02h - Drive number:
//                              *   00h Floppy image
//                              *   80h Bootable hard disk
//                              *   81h-FFh Nonbootable or no emulation
//                              */
//     UCHAR   Controller;     // 03h - CD-ROM controller number
//     ULONG   LBAImage;       // 04h - Logical Block Address of disk image to emulate
//     USHORT  DeviceSpec;     /* 08h - Device specification (see also #00282)
//                              * (IDE) Bit 0:
//                              *     Drive is slave instead of master
//                              * (SCSI) Bits 7-0:
//                              *     LUN and PUN
//                              * Bits 15-8:
//                              *     Bus number
//                              */
//     USHORT  Buffer;         // 0Ah - Segment of 3K buffer for caching CD-ROM reads
//     USHORT  LoadSeg;        // 0Ch - Load segment for initial boot image.
//                             //       If 0000h, load at segment 07C0h.
//     USHORT  SectorCount;    // 0Eh - Number of 512-byte virtual sectors to load
//                             //       (only valid for AH=4Ch).
//     UCHAR   CHSGeometry[3]; /* 10h - Low byte of cylinder count (for INT 13/AH=08h)
//                              * 11h - Sector count, high bits of cylinder count (for INT 13/AH=08h)
//                              * 12h - Head count (for INT 13/AH=08h)
//                              */
//     UCHAR   Reserved;
// } I386_CDROM_SPEC_PACKET, *PI386_CDROM_SPEC_PACKET;
//
// /*
//  * Extended disk geometry (Int13 / AH=48h)
//  * See also ntdddisk.h DISK_EX_INT13_INFO
//  */
// typedef struct _EXTENDED_GEOMETRY
// {
//     USHORT      Size;
//     USHORT      Flags;
//     ULONG       Cylinders;
//     ULONG       Heads;
//     ULONG       SectorsPerTrack;
//     ULONGLONG   Sectors;
//     USHORT      BytesPerSector;
//     ULONG       PDPTE;
// } EXTENDED_GEOMETRY, *PEXTENDED_GEOMETRY;
//
// #include <poppack.h>
//
// typedef struct _PC_DISK_DRIVE
// {
//     /* Disk geometry (legacy BIOS and INT13 extended) */
//     GEOMETRY Geometry;
//     EXTENDED_GEOMETRY ExtGeometry;
//
//     /* TRUE when INT 13h extensions are supported */
//     BOOLEAN Int13ExtensionsSupported;
//
//     /*
//      * 'IsRemovable' flag: TRUE when the drive is removable (e.g. floppy, CD-ROM...).
//      * In that case some of the cached information might need to be refreshed regularly.
//      */
//     BOOLEAN IsRemovable;
//
// #ifdef CACHE_MULTI_DRIVES
//     /*
//      * 'Initialized' flag: if TRUE then the drive has been initialized;
//      * if FALSE then it needs to be initialized; if its high bit is set
//      * then there has been an error; don't try to use it.
//      */
//     BOOLEAN Initialized;
// #endif
// } PC_DISK_DRIVE, *PPC_DISK_DRIVE;
//
// #ifdef CACHE_MULTI_DRIVES
// /* Cache of all possible PC disk drives */
// // Maximum number of disks is 0x100, indexed from 0x00 to 0xFF.
// static PC_DISK_DRIVE PcDiskDrive[0x100];
// #else
// /* Cached data for the last-accessed PC disk drive */
// // We use a USHORT so that we can initialize it with a drive number that cannot exist
// // on the system (they are <= 0xFF), therefore forcing drive caching on first access.
// static USHORT LastDriveNumber = 0xFFFF;
// static PC_DISK_DRIVE PcDiskDrive;
// #endif /* CACHE_MULTI_DRIVES */
//
// /* DISK IO ERROR SUPPORT *****************************************************/
//
// static LONG lReportError = 0; // >= 0: display errors; < 0: hide errors.
//
// LONG DiskReportError(BOOLEAN bShowError)
// {
//     /* Set the reference count */
//     if (bShowError) ++lReportError;
//     else            --lReportError;
//     return lReportError;
// }
//
// static PCSTR DiskGetErrorCodeString(ULONG ErrorCode)
// {
//     switch (ErrorCode)
//     {
//     case 0x00:  return "no error";
//     case 0x01:  return "bad command passed to driver";
//     case 0x02:  return "address mark not found or bad sector";
//     case 0x03:  return "diskette write protect error";
//     case 0x04:  return "sector not found";
//     case 0x05:  return "fixed disk reset failed";
//     case 0x06:  return "diskette changed or removed";
//     case 0x07:  return "bad fixed disk parameter table";
//     case 0x08:  return "DMA overrun";
//     case 0x09:  return "DMA access across 64k boundary";
//     case 0x0A:  return "bad fixed disk sector flag";
//     case 0x0B:  return "bad fixed disk cylinder";
//     case 0x0C:  return "unsupported track/invalid media";
//     case 0x0D:  return "invalid number of sectors on fixed disk format";
//     case 0x0E:  return "fixed disk controlled data address mark detected";
//     case 0x0F:  return "fixed disk DMA arbitration level out of range";
//     case 0x10:  return "ECC/CRC error on disk read";
//     case 0x11:  return "recoverable fixed disk data error, data fixed by ECC";
//     case 0x20:  return "controller error (NEC for floppies)";
//     case 0x40:  return "seek failure";
//     case 0x80:  return "time out, drive not ready";
//     case 0xAA:  return "fixed disk drive not ready";
//     case 0xBB:  return "fixed disk undefined error";
//     case 0xCC:  return "fixed disk write fault on selected drive";
//     case 0xE0:  return "fixed disk status error/Error reg = 0";
//     case 0xFF:  return "sense operation failed";
//
//     default:    return "unknown error code";
//     }
// }
//
// static VOID DiskError(PCSTR ErrorString, ULONG ErrorCode)
// {
//     CHAR ErrorCodeString[200];
//
//     if (lReportError < 0)
//         return;
//
//     sprintf(ErrorCodeString, "%s\n\nError Code: 0x%lx\nError: %s",
//             ErrorString, ErrorCode, DiskGetErrorCodeString(ErrorCode));
//
//     ERR("%s\n", (ErrorCodeString) as usize);
//     UiMessageBox(ErrorCodeString);
// }
//
// /* FUNCTIONS *****************************************************************/
//
// BOOLEAN DiskResetController(UCHAR DriveNumber)
// {
//     REGS RegsIn, RegsOut;
//
//     WARN("DiskResetController(0x%x) DISK OPERATION FAILED -- RESETTING CONTROLLER\n", (DriveNumber) as usize);
//
//     /*
//      * BIOS Int 13h, function 0 - Reset disk system
//      * AH = 00h
//      * DL = drive (if bit 7 is set both hard disks and floppy disks reset)
//      * Return:
//      * AH = status
//      * CF clear if successful
//      * CF set on error
//      */
//     RegsIn.b.ah = 0x00;
//     RegsIn.b.dl = DriveNumber;
//
//     /* Reset the disk controller */
//     Int386(0x13, &RegsIn, &RegsOut);
//
//     return INT386_SUCCESS(RegsOut);
// }
//
// static BOOLEAN
// DiskIsDriveRemovable(UCHAR DriveNumber)
// {
//     /*
//      * Hard disks use drive numbers >= 0x80 . So if the drive number
//      * indicates a hard disk then return FALSE.
//      * 0x49 is our magic ramdisk drive, so return FALSE for that too.
//      */
//     if ((DriveNumber >= 0x80) || (DriveNumber == 0x49))
//         return FALSE;
//
//     /* The drive is a floppy diskette so return TRUE */
//     return TRUE;
// }
//
// static BOOLEAN
// DiskInt13ExtensionsSupported(IN UCHAR DriveNumber)
// {
//     REGS RegsIn, RegsOut;
//
//     /*
//      * Some BIOSes report that extended disk access functions are not supported
//      * when booting from a CD (e.g. Phoenix BIOS v6.00PG and Insyde BIOS shipping
//      * with Intel Macs). Therefore we just return TRUE if we're booting from a CD
//      * - we can assume that all El Torito capable BIOSes support INT 13 extensions.
//      * We simply detect whether we're booting from CD by checking whether the drive
//      * number is >= 0x8A. It's 0x90 on the Insyde BIOS, and 0x9F on most other BIOSes.
//      */
//     if (DriveNumber >= 0x8A)
//         return TRUE;
//
//     /*
//      * IBM/MS INT 13 Extensions - INSTALLATION CHECK
//      * AH = 41h
//      * BX = 55AAh
//      * DL = drive (80h-FFh)
//      * Return:
//      * CF set on error (extensions not supported)
//      * AH = 01h (invalid function)
//      * CF clear if successful
//      * BX = AA55h if installed
//      * AH = major version of extensions
//      * 01h = 1.x
//      * 20h = 2.0 / EDD-1.0
//      * 21h = 2.1 / EDD-1.1
//      * 30h = EDD-3.0
//      * AL = internal use
//      * CX = API subset support bitmap
//      * DH = extension version (v2.0+ ??? -- not present in 1.x)
//      *
//      * Bitfields for IBM/MS INT 13 Extensions API support bitmap
//      * Bit 0, extended disk access functions (AH=42h-44h,47h,48h) supported
//      * Bit 1, removable drive controller functions (AH=45h,46h,48h,49h,INT 15/AH=52h) supported
//      * Bit 2, enhanced disk drive (EDD) functions (AH=48h,AH=4Eh) supported
//      *        extended drive parameter table is valid
//      * Bits 3-15 reserved
//      */
//     RegsIn.b.ah = 0x41;
//     RegsIn.w.bx = 0x55AA;
//     RegsIn.b.dl = DriveNumber;
//
//     /* Reset the disk controller */
//     Int386(0x13, &RegsIn, &RegsOut);
//     if (!INT386_SUCCESS(RegsOut))
//     {
//         /* CF set on error (extensions not supported) */
//         return FALSE;
//     }
//
//     if (RegsOut.w.bx != 0xAA55)
//     {
//         /* BX = AA55h if installed */
//         return FALSE;
//     }
//
// #if DBG
//     TRACE("Drive 0x%x: INT 13h Extended version: 0x%02x, API bitmap: 0x%04x\n", (//           DriveNumber) as usize, (RegsOut.b.ah) as usize, (RegsOut.w.cx) as usize);
// #endif
//     if (!(RegsOut.w.cx & 0x0001))
//     {
//         /*
//          * CX = API subset support bitmap.
//          * Bit 0, extended disk access functions (AH=42h-44h,47h,48h) supported.
//          */
//         WARN("Drive 0x%x: Suspicious API subset support bitmap 0x%04x\n", (//              DriveNumber) as usize, (RegsOut.w.cx) as usize);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// static BOOLEAN
// DiskGetExtendedDriveParameters(
//     _In_ UCHAR DriveNumber,
//     _In_ PPC_DISK_DRIVE DiskDrive,
//     _Out_ PVOID Buffer,
//     _In_ USHORT BufferSize)
// {
//     PEXTENDED_GEOMETRY Ptr = (PEXTENDED_GEOMETRY)BIOSCALLBUFFER;
//     REGS RegsIn, RegsOut;
//
//     TRACE("DiskGetExtendedDriveParameters(0x%x)\n", (DriveNumber) as usize);
//
//     if (!DiskDrive->Int13ExtensionsSupported || (BufferSize < sizeof(*Ptr)))
//         return FALSE;
//
//     /*
//      * Initialize the transfer buffer.
//      * NOTE: Zeroing out the buffer also helps avoiding the bug where Dell
//      * machines using PhoenixBIOS 4.0 Release 6.0 fail to correctly handle
//      * this function if Ptr->Flags is not 0 on entry.
//      */
//     RtlZeroMemory(Ptr, sizeof(*Ptr)); // BufferSize;
//     Ptr->Size = BufferSize;
//
//     /*
//      * BIOS Int 13h, function 48h - Get drive parameters
//      * AH = 48h
//      * DL = drive (bit 7 set for hard disk)
//      * DS:SI = result buffer
//      * Return:
//      * CF set on error
//      * AH = status (07h)
//      * CF clear if successful
//      * AH = 00h
//      * DS:SI -> result buffer
//      */
//     RegsIn.b.ah = 0x48;
//     RegsIn.b.dl = DriveNumber;
//     RegsIn.x.ds = BIOSCALLBUFSEGMENT;
//     RegsIn.w.si = BIOSCALLBUFOFFSET;
//
//     /* Get drive parameters */
//     Int386(0x13, &RegsIn, &RegsOut);
//     if (!INT386_SUCCESS(RegsOut))
//         return FALSE;
//
//     RtlCopyMemory(Buffer, Ptr, BufferSize);
//
// #if DBG
//     TRACE("Size of buffer:                          0x%x\n", (Ptr->Size) as usize);
//     TRACE("Information flags:                       0x%x\n", (Ptr->Flags) as usize);
//     TRACE("Number of physical cylinders on drive:   %u\n", (Ptr->Cylinders) as usize);
//     TRACE("Number of physical heads on drive:       %u\n", (Ptr->Heads) as usize);
//     TRACE("Number of physical sectors per track:    %u\n", (Ptr->SectorsPerTrack) as usize);
//     TRACE("Total number of sectors on drive:        %I64u\n", (Ptr->Sectors) as usize);
//     TRACE("Bytes per sector:                        %u\n", (Ptr->BytesPerSector) as usize);
//     if (Ptr->Size >= 0x1E)
//     {
//         // LOWORD(Ptr->PDPTE): offset, HIWORD(Ptr->PDPTE): segment
//         USHORT Off = (USHORT)(Ptr->PDPTE & 0xFFFF);         // ((PUSHORT)&Ptr->PDPTE)[0];
//         USHORT Seg = (USHORT)((Ptr->PDPTE >> 16) & 0xFFFF); // ((PUSHORT)&Ptr->PDPTE)[1];
//         TRACE("EDD configuration parameters (DPTE):     %x:%x\n", (Seg) as usize, (Off) as usize);
//
//         /* The DPTE pointer is valid if it's != FFFF:FFFF (per the Enhanced Disk
//          * Drive Specification), but also, when it's != 0000:0000 (broken BIOSes) */
//         if (Ptr->PDPTE != 0xFFFFFFFF && Ptr->PDPTE != 0)
//         {
//             PUCHAR SpecPtr = (PUCHAR)(ULONG_PTR)((Seg << 4) + Off);
//             TRACE("SpecPtr:                                 0x%x\n", (SpecPtr) as usize);
//             TRACE("Physical I/O port base address:          0x%x\n", (*(PUSHORT)&SpecPtr[0]) as usize);
//             TRACE("Disk-drive control port address:         0x%x\n", (*(PUSHORT)&SpecPtr[2]) as usize);
//             TRACE("Head register upper nibble:              0x%x\n", (SpecPtr[4]) as usize);
//             TRACE("BIOS Vendor-specific:                    0x%x\n", (SpecPtr[5]) as usize);
//             TRACE("IRQ for drive:                           %u\n", (SpecPtr[6]) as usize);
//             TRACE("Sector count for multi-sector transfers: %u\n", (SpecPtr[7]) as usize);
//             TRACE("DMA control:                             0x%x\n", (SpecPtr[8]) as usize);
//             TRACE("Programmed I/O control:                  0x%x\n", (SpecPtr[9]) as usize);
//             TRACE("Drive options:                           0x%x\n", (*(PUSHORT)&SpecPtr[10]) as usize);
//         }
//     }
//     if (Ptr->Size >= 0x42)
//     {
//         TRACE("Signature:                             0x%x\n", (((PUSHORT)Ptr)[15]) as usize);
//     }
// #endif // DBG
//
//     return TRUE;
// }
//
// CONFIGURATION_TYPE
// DiskGetConfigType(
//     _In_ UCHAR DriveNumber)
// {
//     if ((DriveNumber == FrldrBootDrive)/* && DiskIsDriveRemovable(DriveNumber) */ && (FrldrBootPartition == 0xFF))
//         return CdromController; /* This is our El Torito boot CD-ROM */
//     else if (DiskIsDriveRemovable(DriveNumber))
//         return FloppyDiskPeripheral;
//     else
//         return DiskPeripheral;
// }
//
// static BOOLEAN
// InitDriveGeometry(
//     IN UCHAR DriveNumber,
//     IN PPC_DISK_DRIVE DiskDrive)
// {
//     BOOLEAN Success;
//     REGS RegsIn, RegsOut;
//     ULONG Cylinders;
//
//     /* Get the extended geometry first */
//     RtlZeroMemory(&DiskDrive->ExtGeometry, sizeof(DiskDrive->ExtGeometry));
//     Success = DiskGetExtendedDriveParameters(DriveNumber, DiskDrive,
//                                              &DiskDrive->ExtGeometry,
//                                              sizeof(DiskDrive->ExtGeometry));
//     if (Success)
//     {
//         TRACE("DiskGetExtendedDriveParameters(0x%x) returned:\n"
//               "Cylinders  : 0x%x\n"
//               "Heads      : 0x%x\n"
//               "Sects/Track: 0x%x\n"
//               "Total Sects: 0x%llx\n"
//               "Bytes/Sect : 0x%x\n", (//               DriveNumber) as usize, (//               DiskDrive->ExtGeometry.Cylinders) as usize, (//               DiskDrive->ExtGeometry.Heads) as usize, (//               DiskDrive->ExtGeometry.SectorsPerTrack) as usize, (//               DiskDrive->ExtGeometry.Sectors) as usize, (//               DiskDrive->ExtGeometry.BytesPerSector) as usize);
//     }
//
//     /* Now try the legacy geometry */
//     RtlZeroMemory(&DiskDrive->Geometry, sizeof(DiskDrive->Geometry));
//
//     /*
//      * BIOS Int 13h, function 08h - Get drive parameters
//      * AH = 08h
//      * DL = drive (bit 7 set for hard disk)
//      * ES:DI = 0000h:0000h to guard against BIOS bugs
//      * Return:
//      * CF set on error
//      * AH = status (07h)
//      * CF clear if successful
//      * AH = 00h
//      * AL = 00h on at least some BIOSes
//      * BL = drive type (AT/PS2 floppies only)
//      * CH = low eight bits of maximum cylinder number
//      * CL = maximum sector number (bits 5-0)
//      *      high two bits of maximum cylinder number (bits 7-6)
//      * DH = maximum head number
//      * DL = number of drives
//      * ES:DI -> drive parameter table (floppies only)
//      */
//     RegsIn.b.ah = 0x08;
//     RegsIn.b.dl = DriveNumber;
//     RegsIn.w.es = 0x0000;
//     RegsIn.w.di = 0x0000;
//
//     /* Get drive parameters */
//     Int386(0x13, &RegsIn, &RegsOut);
//     if (!INT386_SUCCESS(RegsOut))
//     {
//         /* We failed, return the result of the previous call (extended geometry) */
//         return Success;
//     }
//     /* OR it with the old result, so that we return TRUE whenever either call succeeded */
//     Success |= TRUE;
//
//     Cylinders = (RegsOut.b.cl & 0xC0) << 2;
//     Cylinders += RegsOut.b.ch;
//     Cylinders++;
//     DiskDrive->Geometry.Cylinders = Cylinders;
//     DiskDrive->Geometry.Heads = RegsOut.b.dh + 1;
//     DiskDrive->Geometry.SectorsPerTrack = RegsOut.b.cl & 0x3F;
//     DiskDrive->Geometry.BytesPerSector = 512; /* Just assume 512 bytes per sector */
//
//     DiskDrive->Geometry.Sectors = (ULONGLONG)DiskDrive->Geometry.Cylinders *
//                                              DiskDrive->Geometry.Heads *
//                                              DiskDrive->Geometry.SectorsPerTrack;
//
//     TRACE("Regular Int13h(0x%x) returned:\n"
//           "Cylinders  : 0x%x\n"
//           "Heads      : 0x%x\n"
//           "Sects/Track: 0x%x\n"
//           "Total Sects: 0x%llx\n"
//           "Bytes/Sect : 0x%x\n", (//           DriveNumber) as usize, (//           DiskDrive->Geometry.Cylinders) as usize, (//           DiskDrive->Geometry.Heads) as usize, (//           DiskDrive->Geometry.SectorsPerTrack) as usize, (//           DiskDrive->Geometry.Sectors) as usize, (//           DiskDrive->Geometry.BytesPerSector) as usize);
//
//     return Success;
// }
//
// static BOOLEAN
// PcDiskDriveInit(
//     IN UCHAR DriveNumber,
//     IN OUT PPC_DISK_DRIVE DiskDrive)
// {
//     DiskDrive->IsRemovable = DiskIsDriveRemovable(DriveNumber);
//
//     /*
//      * Check to see if it is a fixed disk drive.
//      * If so then check to see if INT 13h extensions work.
//      * If they do then use them, otherwise default back to BIOS calls.
//      */
//     DiskDrive->Int13ExtensionsSupported = DiskInt13ExtensionsSupported(DriveNumber);
//
//     if (!InitDriveGeometry(DriveNumber, DiskDrive))
//         return FALSE;
//
//     TRACE("\n"
//           "DriveNumber: 0x%x\n"
//           "IsRemovable              = %s\n"
//           "Int13ExtensionsSupported = %s\n", (//           DriveNumber) as usize, (//           DiskDrive->IsRemovable ? "TRUE" : "FALSE") as usize, (//           DiskDrive->Int13ExtensionsSupported ? "TRUE" : "FALSE") as usize);
//
//     return TRUE;
// }
//
// static inline
// PPC_DISK_DRIVE
// PcDiskDriveNumberToDrive(IN UCHAR DriveNumber)
// {
// #ifdef CACHE_MULTI_DRIVES
//     PPC_DISK_DRIVE DiskDrive;
//
//     ASSERT((0 <= DriveNumber) && (DriveNumber < RTL_NUMBER_OF(PcDiskDrive)));
//
//     /* Retrieve a slot */
//     DiskDrive = &PcDiskDrive[DriveNumber];
//
//     /* If the drive has not been initialized before... */
//     if (!DiskDrive->Initialized)
//     {
//         /* ... try to initialize it now. */
//         if (!PcDiskDriveInit(DriveNumber, DiskDrive))
//         {
//             /*
//              * If we failed, there is no drive at this number
//              * and flag it as such (set its high bit).
//              */
//             DiskDrive->Initialized |= 0x80;
//             return NULL;
//         }
//         DiskDrive->Initialized = TRUE;
//     }
//     else if (DiskDrive->Initialized & 0x80)
//     {
//         /*
//          * The disk failed to be initialized previously, reset its flag to give
//          * it chance to be initialized again later, but just fail for the moment.
//          */
//         DiskDrive->Initialized = FALSE;
//         return NULL;
//     }
//
//     return DiskDrive;
// #else
//     static PC_DISK_DRIVE NewDiskDrive;
//
//     ASSERT((0 <= DriveNumber) && (DriveNumber <= 0xFF));
//
//     /* Update cached information */
//
//     /* If the drive has not been accessed last before... */
//     if ((USHORT)DriveNumber != LastDriveNumber)
//     {
//         /* ... try to (re-)initialize and cache it now. */
//         RtlZeroMemory(&NewDiskDrive, sizeof(NewDiskDrive));
//         if (!PcDiskDriveInit(DriveNumber, &NewDiskDrive))
//         {
//             /*
//              * If we failed, there is no drive at this number.
//              * Keep the last-accessed valid drive cached.
//              */
//             return NULL;
//         }
//         /* We succeeded, cache the drive data */
//         PcDiskDrive = NewDiskDrive;
//         LastDriveNumber = (USHORT)DriveNumber;
//     }
//
//     return &PcDiskDrive;
// #endif /* CACHE_MULTI_DRIVES */
// }
//
// static BOOLEAN
// PcDiskReadLogicalSectorsLBA(
//     IN UCHAR DriveNumber,
//     IN ULONGLONG SectorNumber,
//     IN ULONG SectorCount,
//     OUT PVOID Buffer)
// {
//     PI386_DISK_ADDRESS_PACKET Packet = (PI386_DISK_ADDRESS_PACKET)BIOSCALLBUFFER;
//     REGS RegsIn, RegsOut;
//     ULONG RetryCount;
//
//     /* Setup disk address packet */
//     RtlZeroMemory(Packet, sizeof(*Packet));
//     Packet->PacketSize = sizeof(*Packet);
//     Packet->Reserved = 0;
//     // Packet->LBABlockCount set in the loop.
//     Packet->TransferBufferOffset = ((ULONG_PTR)Buffer) & 0x0F;
//     Packet->TransferBufferSegment = (USHORT)(((ULONG_PTR)Buffer) >> 4);
//     Packet->LBAStartBlock = SectorNumber;
//
//     /*
//      * BIOS Int 13h, function 42h - IBM/MS INT 13 Extensions - EXTENDED READ
//      * Return:
//      * CF clear if successful
//      * AH = 00h
//      * CF set on error
//      * AH = error code
//      * Disk address packet's block count field set to the
//      * number of blocks successfully transferred.
//      */
//     RegsIn.b.ah = 0x42;
//     RegsIn.b.dl = DriveNumber;          // Drive number in DL (0 - floppy, 0x80 - harddisk)
//     RegsIn.x.ds = BIOSCALLBUFSEGMENT;   // DS:SI -> disk address packet
//     RegsIn.w.si = BIOSCALLBUFOFFSET;
//
//     /* Retry 3 times */
//     for (RetryCount = 0; RetryCount < 3; ++RetryCount)
//     {
//         /* Restore the number of blocks to transfer, since it gets reset
//          * on failure with the number of blocks that were successfully
//          * transferred (and which could be zero). */
//         Packet->LBABlockCount = (USHORT)SectorCount;
//         ASSERT(Packet->LBABlockCount == SectorCount);
//
//         Int386(0x13, &RegsIn, &RegsOut);
//
//         /* If it worked, or if it was a corrected ECC error
//          * and the data is still good, return success */
//         if (INT386_SUCCESS(RegsOut) || (RegsOut.b.ah == 0x11))
//             return TRUE;
//
//         /* It failed, do the next retry */
//         DiskResetController(DriveNumber);
//     }
//
//     /* If we get here then the read failed */
//     DiskError("Disk Read Failed in LBA mode", RegsOut.b.ah);
//     ERR("Disk Read Failed in LBA mode: %x (%s) (DriveNumber: 0x%x SectorNumber: %I64u SectorCount: %u)\n", (//         RegsOut.b.ah) as usize, (DiskGetErrorCodeString(RegsOut.b.ah)) as usize, (//         DriveNumber) as usize, (SectorNumber) as usize, (SectorCount) as usize);
//
//     return FALSE;
// }
//
// static BOOLEAN
// PcDiskReadLogicalSectorsCHS(
//     IN UCHAR DriveNumber,
//     IN PPC_DISK_DRIVE DiskDrive,
//     IN ULONGLONG SectorNumber,
//     IN ULONG SectorCount,
//     OUT PVOID Buffer)
// {
//     UCHAR PhysicalSector;
//     UCHAR PhysicalHead;
//     ULONG PhysicalTrack;
//     GEOMETRY DriveGeometry;
//     ULONG NumberOfSectorsToRead;
//     REGS RegsIn, RegsOut;
//     ULONG RetryCount;
//
//     DriveGeometry = DiskDrive->Geometry;
//     if (DriveGeometry.SectorsPerTrack == 0 || DriveGeometry.Heads == 0)
//         return FALSE;
//
//     while (SectorCount > 0)
//     {
//         /*
//          * Calculate the physical disk offsets.
//          * Note: DriveGeometry.SectorsPerTrack < 64
//          */
//         PhysicalSector = 1 + (UCHAR)(SectorNumber % DriveGeometry.SectorsPerTrack);
//         PhysicalHead = (UCHAR)((SectorNumber / DriveGeometry.SectorsPerTrack) % DriveGeometry.Heads);
//         PhysicalTrack = (ULONG)((SectorNumber / DriveGeometry.SectorsPerTrack) / DriveGeometry.Heads);
//
//         /* Calculate how many sectors we need to read this round */
//         if (PhysicalSector > 1)
//         {
//             NumberOfSectorsToRead = min(SectorCount,
//                                         (DriveGeometry.SectorsPerTrack - (PhysicalSector - 1)));
//         }
//         else
//         {
//             NumberOfSectorsToRead = min(SectorCount, DriveGeometry.SectorsPerTrack);
//         }
//
//         /* Make sure the read is within the geometry boundaries */
//         if ((PhysicalHead >= DriveGeometry.Heads) ||
//             (PhysicalTrack >= DriveGeometry.Cylinders) ||
//             ((NumberOfSectorsToRead + PhysicalSector) > (DriveGeometry.SectorsPerTrack + 1)) ||
//             (PhysicalSector > DriveGeometry.SectorsPerTrack))
//         {
//             DiskError("Disk read exceeds drive geometry limits.", 0);
//             return FALSE;
//         }
//
//         /*
//          * BIOS Int 13h, function 2 - Read Disk Sectors
//          * AH = 02h
//          * AL = number of sectors to read (must be nonzero)
//          * CH = low eight bits of cylinder number
//          * CL = sector number 1-63 (bits 0-5)
//          *      high two bits of cylinder (bits 6-7, hard disk only)
//          * DH = head number
//          * DL = drive number (bit 7 set for hard disk)
//          * ES:BX -> data buffer
//          * Return:
//          * CF set on error
//          * if AH = 11h (corrected ECC error), AL = burst length
//          * CF clear if successful
//          * AH = status
//          * AL = number of sectors transferred
//          *  (only valid if CF set for some BIOSes)
//          */
//         RegsIn.b.ah = 0x02;
//         RegsIn.b.al = (UCHAR)NumberOfSectorsToRead;
//         RegsIn.b.ch = (PhysicalTrack & 0xFF);
//         RegsIn.b.cl = (UCHAR)(PhysicalSector + ((PhysicalTrack & 0x300) >> 2));
//         RegsIn.b.dh = PhysicalHead;
//         RegsIn.b.dl = DriveNumber;
//         RegsIn.w.es = (USHORT)(((ULONG_PTR)Buffer) >> 4);
//         RegsIn.w.bx = ((ULONG_PTR)Buffer) & 0x0F;
//
//         /* Perform the read. Retry 3 times. */
//         for (RetryCount = 0; RetryCount < 3; ++RetryCount)
//         {
//             Int386(0x13, &RegsIn, &RegsOut);
//
//             /* If it worked, or if it was a corrected ECC error
//              * and the data is still good, break out */
//             if (INT386_SUCCESS(RegsOut) || (RegsOut.b.ah == 0x11))
//                 break;
//
//             /* It failed, do the next retry */
//             DiskResetController(DriveNumber);
//         }
//
//         /* If we retried 3 times then fail */
//         if (RetryCount >= 3)
//         {
//             DiskError("Disk Read Failed in CHS mode, after retrying 3 times", RegsOut.b.ah);
//             ERR("Disk Read Failed in CHS mode, after retrying 3 times: %x (%s) (DriveNumber: 0x%x SectorNumber: %I64u SectorCount: %u)\n", (//                 RegsOut.b.ah) as usize, (DiskGetErrorCodeString(RegsOut.b.ah)) as usize, (//                 DriveNumber) as usize, (SectorNumber) as usize, (SectorCount) as usize);
//             return FALSE;
//         }
//
//         /*
//          * I have learned that not all BIOSes return
//          * the sector read count in the AL register (at least mine doesn't)
//          * even if the sectors were read correctly. So instead
//          * of checking the sector read count we will rely solely
//          * on the carry flag being set on error.
//          */
//
//         Buffer = (PVOID)((ULONG_PTR)Buffer + (NumberOfSectorsToRead * DriveGeometry.BytesPerSector));
//         SectorCount -= NumberOfSectorsToRead;
//         SectorNumber += NumberOfSectorsToRead;
//     }
//
//     return TRUE;
// }
//
// BOOLEAN
// PcDiskReadLogicalSectors(
//     IN UCHAR DriveNumber,
//     IN ULONGLONG SectorNumber,
//     IN ULONG SectorCount,
//     OUT PVOID Buffer)
// {
//     PPC_DISK_DRIVE DiskDrive;
//
//     TRACE("PcDiskReadLogicalSectors() DriveNumber: 0x%x SectorNumber: %I64u SectorCount: %u Buffer: 0x%x\n", (//           DriveNumber) as usize, (SectorNumber) as usize, (SectorCount) as usize, (Buffer) as usize);
//
//     /* 16-bit BIOS addressing limitation */
//     ASSERT(((ULONG_PTR)Buffer) <= 0xFFFFF);
//
//     DiskDrive = PcDiskDriveNumberToDrive(DriveNumber);
//     if (!DiskDrive)
//         return FALSE;
//
//     if ((DriveNumber >= 0x80) && DiskDrive->Int13ExtensionsSupported)
//     {
//         /* LBA is easy, nothing to calculate. Just do the read. */
//         TRACE("--> Using LBA\n");
//         return PcDiskReadLogicalSectorsLBA(DriveNumber, SectorNumber, SectorCount, Buffer);
//     }
//     else
//     {
//         /* LBA is not supported, default to CHS */
//         TRACE("--> Using CHS\n");
//         return PcDiskReadLogicalSectorsCHS(DriveNumber, DiskDrive, SectorNumber, SectorCount, Buffer);
//     }
// }
//
// #if defined(__i386__) || defined(_M_AMD64)
// VOID __cdecl DiskStopFloppyMotor(VOID)
// {
//     WRITE_PORT_UCHAR((PUCHAR)0x3F2, 0); // DOR_FDC_ENABLE | DOR_DMA_IO_INTERFACE_ENABLE 0x0C // we changed 0x0C->0 to workaround CORE-16469
// }
// #endif // defined __i386__ || defined(_M_AMD64)
//
// BOOLEAN
// PcDiskGetDriveGeometry(UCHAR DriveNumber, PGEOMETRY Geometry)
// {
//     PPC_DISK_DRIVE DiskDrive;
//
//     TRACE("PcDiskGetDriveGeometry(0x%x)\n", (DriveNumber) as usize);
//
//     DiskDrive = PcDiskDriveNumberToDrive(DriveNumber);
//     if (!DiskDrive)
//         return FALSE;
//
//     /* Try to get the extended geometry first */
//     if (DiskDrive->ExtGeometry.Size == sizeof(DiskDrive->ExtGeometry))
//     {
//         /* Extended geometry has been initialized, return it */
//         Geometry->Cylinders = DiskDrive->ExtGeometry.Cylinders;
//         Geometry->Heads = DiskDrive->ExtGeometry.Heads;
//         Geometry->SectorsPerTrack = DiskDrive->ExtGeometry.SectorsPerTrack;
//         Geometry->BytesPerSector = DiskDrive->ExtGeometry.BytesPerSector;
//         Geometry->Sectors = DiskDrive->ExtGeometry.Sectors;
//     }
//     else
//     /* Fall back to legacy BIOS geometry */
//     {
//         *Geometry = DiskDrive->Geometry;
//     }
//
//     return TRUE;
// }
//
// ULONG
// PcDiskGetCacheableBlockCount(UCHAR DriveNumber)
// {
//     PPC_DISK_DRIVE DiskDrive;
//
//     DiskDrive = PcDiskDriveNumberToDrive(DriveNumber);
//     if (!DiskDrive)
//         return 1; // Unknown count.
//
//     /*
//      * If LBA is supported then the block size will be 64 sectors (32k).
//      * If not then the block size is the size of one track.
//      */
//     if (DiskDrive->Int13ExtensionsSupported)
//         return 64;
//     else
//         return DiskDrive->Geometry.SectorsPerTrack;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
