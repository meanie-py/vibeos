#[allow(unused_imports)] use crate::{ERR, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_DISK; // hwide.c: DBG_DEFAULT_CHANNEL(DISK)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PULONG64};
// freeldr_ddk_imports dropped (not in ddk): RtlUlongByteSwap, RtlUlonglongByteSwap, RtlUshortByteSwap
// freeldr_ddk_imports dropped (not in ddk): DeviceNumber, Spare
use core::ffi::c_void;

// Cross-module C symbols (real exported FreeLDR symbols).
// RtlU*ByteSwap are RTL intrinsics; declared file-local (freeldr_ddk_imports.py
// catalog misclassifies them as "not in ddk" though they exist in bindgen).
extern "C" {
    fn StallExecutionProcessor(Microseconds: ULONG);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RtlUshortByteSwap(Source: USHORT) -> USHORT;
    fn RtlUlongByteSwap(Source: ULONG) -> ULONG;
    fn RtlUlonglongByteSwap(Source: ULONGLONG) -> ULONGLONG;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RtlUshortByteSwap(Source: USHORT) -> USHORT;
    fn RtlUlongByteSwap(Source: ULONG) -> ULONG;
    fn RtlUlonglongByteSwap(Source: ULONGLONG) -> ULONGLONG;
}

// ===========================================================================
// Dropped non-fn content faithfully restored (rustconv drops headers/macros).
//   hwidep.h (private ATA/ATAPI driver header), <ata.h>, <scsi.h>, hwide.h.
// ===========================================================================

// ---- hwidep.h constants ----
pub const TAG_ATA_DEVICE: ULONG = u32::from_le_bytes(*b"dATA"); // 'dATA'

pub const ATA_STATUS_SUCCESS: UCHAR = 0;
pub const ATA_STATUS_PENDING: UCHAR = 1;
pub const ATA_STATUS_ERROR: UCHAR = 2;
pub const ATA_STATUS_RESET: UCHAR = 3;
pub const ATA_STATUS_RETRY: UCHAR = 4;

pub const CHANNEL_MAX_DEVICES: usize = 2;

pub type IDE_REG = USHORT; // x86 I/O space

pub const ATA_MAX_LBA_28: ULONG64 = 0x0FFFFFFF;
pub const ATA_MAX_LBA_48: ULONG64 = 1u64 << 48;

pub const IDE_FEATURE_PIO: UCHAR = 0x00;
pub const IDE_DC_ALWAYS: UCHAR = 0x08;
pub const IDE_DRIVE_SELECT: UCHAR = 0xA0;

pub const ATAPI_INT_REASON_COD: UCHAR = 0x01;
pub const ATAPI_INT_REASON_IO: UCHAR = 0x02;
pub const ATAPI_INT_REASON_MASK: UCHAR = ATAPI_INT_REASON_IO | ATAPI_INT_REASON_COD;
// ADDED 2026-07-05: these four were unresolved in the AtapProcessAtapiRequest
// match — Rust silently made the first arm a bind-all catch-all (bind-all
// match-arm bug class). Values per boot/freeldr/freeldr/arch/drivers/hwidep.h.
pub const ATAPI_INT_REASON_STATUS_NEC: UCHAR = 0x00;
pub const ATAPI_INT_REASON_STATUS: UCHAR = ATAPI_INT_REASON_IO | ATAPI_INT_REASON_COD;
pub const ATAPI_INT_REASON_AWAIT_CDB: UCHAR = IDE_STATUS_DRQ | ATAPI_INT_REASON_COD;
pub const ATAPI_INT_REASON_DATA_IN: UCHAR = ATAPI_INT_REASON_IO | IDE_STATUS_DRQ;
// Same class, AtapAtapiHandleError match; values per sdk/include/ddk/scsi.h.
pub const SCSI_SENSEQ_BECOMING_READY: UCHAR = 0x01;
pub const SCSI_SENSEQ_INIT_COMMAND_REQUIRED: UCHAR = 0x02;

pub const MAXIMUM_CDROM_SIZE: ULONG = 804;

pub const ATA_TIME_BUSY_SELECT: ULONG = 2000;
pub const ATA_TIME_BUSY_POLL: ULONG = 500000;
pub const ATA_TIME_BUSY_ENUM: ULONG = 100;
pub const ATA_TIME_BUSY_RESET: ULONG = 1000000;
pub const ATA_TIME_RESET_SELECT: ULONG = 200000;
pub const ATA_TIME_DRQ_CLEAR: ULONG = 1000;
pub const ATA_TIME_PHASE_CHANGE: ULONG = 100;
pub const ATA_TIME_DRQ_ASSERT: ULONG = 15;

// ---- <ata.h> constants ----
pub const IDE_LBA_MODE: UCHAR = 1 << 6;

pub const IDE_DC_DISABLE_INTERRUPTS: UCHAR = 0x02;
pub const IDE_DC_RESET_CONTROLLER: UCHAR = 0x04;

pub const IDE_STATUS_ERROR: UCHAR = 0x01;
pub const IDE_STATUS_DRQ: UCHAR = 0x08;
pub const IDE_STATUS_DEVICE_FAULT: UCHAR = 0x20;
pub const IDE_STATUS_BUSY: UCHAR = 0x80;

pub const IDE_COMMAND_READ: UCHAR = 0x20;
pub const IDE_COMMAND_READ_EXT: UCHAR = 0x24;
pub const IDE_COMMAND_READ_MULTIPLE_EXT: UCHAR = 0x29;
pub const IDE_COMMAND_ATAPI_PACKET: UCHAR = 0xA0;
pub const IDE_COMMAND_ATAPI_IDENTIFY: UCHAR = 0xA1;
pub const IDE_COMMAND_READ_MULTIPLE: UCHAR = 0xC4;
pub const IDE_COMMAND_SET_MULTIPLE: UCHAR = 0xC6;
pub const IDE_COMMAND_IDENTIFY: UCHAR = 0xEC;

// ---- <scsi.h> constants ----
pub const READ_TOC_FORMAT_SESSION: UCHAR = 0x01;
pub const SCSIOP_TEST_UNIT_READY: UCHAR = 0x00;
pub const SCSIOP_REQUEST_SENSE: UCHAR = 0x03;
pub const SCSIOP_READ_CAPACITY: UCHAR = 0x25;
pub const SCSIOP_READ: UCHAR = 0x28;
pub const SCSIOP_READ_TOC: UCHAR = 0x43;
pub const SCSIOP_READ16: UCHAR = 0x88;
pub const SCSIOP_SERVICE_ACTION_IN16: UCHAR = 0x9E;
pub const SERVICE_ACTION_READ_CAPACITY16: UCHAR = 0x10;
pub const SCSI_SENSE_NOT_READY: UCHAR = 0x02;
pub const SCSI_SENSE_UNIT_ATTENTION: UCHAR = 0x06;
pub const SCSI_ADSENSE_LUN_NOT_READY: UCHAR = 0x04;
pub const SCSI_ADSENSE_BUS_RESET: UCHAR = 0x29;
pub const SCSI_ADSENSE_NO_MEDIA_IN_DEVICE: UCHAR = 0x3a;

pub const MAXULONG: ULONG = 0xFFFFFFFF;

// ---- hwide.h: DEVICE_UNIT + flags ----
pub const ATA_DEVICE_ATAPI: ULONG = 0x00000001;
pub const ATA_DEVICE_LBA: ULONG = 0x00000002;
pub const ATA_DEVICE_LBA48: ULONG = 0x00000004;
pub const ATA_DEVICE_IS_NEC_CDR260: ULONG = 0x00000008;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DEVICE_UNIT {
    pub Cylinders: ULONG,
    pub Heads: ULONG,
    pub SectorsPerTrack: ULONG,
    pub SectorSize: ULONG,
    pub TotalSectors: ULONG64,
    pub Flags: ULONG,
}
pub type PDEVICE_UNIT = *mut DEVICE_UNIT;

// ---- <ata.h> ATA_DEVICE_CLASS ----
pub type ATA_DEVICE_CLASS = ::core::ffi::c_uint;
pub const DEV_ATA: ATA_DEVICE_CLASS = 0;
pub const DEV_ATAPI: ATA_DEVICE_CLASS = 1;
pub const DEV_NONE: ATA_DEVICE_CLASS = 2;

// ---- hwidep.h: IDE_REGISTERS (anonymous unions; all members are IDE_REG) ----
#[repr(C)]
#[derive(Clone, Copy)]
pub union IdeRegFeatErr {
    pub Features: IDE_REG,
    pub Error: IDE_REG,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union IdeRegScIr {
    pub SectorCount: IDE_REG,
    pub InterruptReason: IDE_REG,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union IdeRegLbaMid {
    pub LbaMid: IDE_REG,
    pub ByteCountLow: IDE_REG,
    pub SignatureLow: IDE_REG,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union IdeRegLbaHigh {
    pub LbaHigh: IDE_REG,
    pub ByteCountHigh: IDE_REG,
    pub SignatureHigh: IDE_REG,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union IdeRegCmdStatus {
    pub Command: IDE_REG,
    pub Status: IDE_REG,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union IdeRegControl {
    pub Control: IDE_REG,
    pub AlternateStatus: IDE_REG,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IDE_REGISTERS {
    pub Data: IDE_REG,
    pub feat_err: IdeRegFeatErr,
    pub sc_ir: IdeRegScIr,
    pub LbaLow: IDE_REG,
    pub lm: IdeRegLbaMid,
    pub lh: IdeRegLbaHigh,
    pub Device: IDE_REG,
    pub __bindgen_anon_1: IdeRegCmdStatus,
    pub ctl: IdeRegControl,
}
pub type PIDE_REGISTERS = *mut IDE_REGISTERS;

// ---- hwidep.h: ATA_TASKFILE ----
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ATA_TASKFILE_DATA {
    pub Feature: UCHAR,
    pub SectorCount: UCHAR,
    pub LowLba: UCHAR,
    pub MidLba: UCHAR,
    pub HighLba: UCHAR,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ATA_TASKFILE {
    pub DriveSelect: UCHAR,
    pub Command: UCHAR,
    pub Data: [ATA_TASKFILE_DATA; 2],
}
pub type PATA_TASKFILE = *mut ATA_TASKFILE;

// hwidep.h: ATA_DEVICE_REQUEST flags
pub const REQUEST_FLAG_LBA48: ULONG = 0x00000001;
pub const REQUEST_FLAG_READ_WRITE_MULTIPLE: ULONG = 0x00000002;
pub const REQUEST_FLAG_PACKET_COMMAND: ULONG = 0x00000004;
pub const REQUEST_FLAG_SET_DEVICE_REGISTER: ULONG = 0x00000008;
pub const REQUEST_FLAG_AWAIT_CDB: ULONG = 0x00000010;
pub const REQUEST_FLAG_READ_COMMAND: ULONG = 0x00000020;
pub const REQUEST_FLAG_IDENTIFY_COMMAND: ULONG = 0x00000040;

#[repr(C)]
#[derive(Clone, Copy)]
pub union AtaReqCdbTaskFile {
    pub Cdb: [UCHAR; 16],
    pub TaskFile: ATA_TASKFILE,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ATA_DEVICE_REQUEST {
    pub u: AtaReqCdbTaskFile,
    pub DataBuffer: PVOID,
    pub Flags: ULONG,
    pub DataTransferLength: ULONG,
}
pub type PATA_DEVICE_REQUEST = *mut ATA_DEVICE_REQUEST;

// ---- <ata.h>: IDENTIFY_DEVICE_DATA / IDENTIFY_PACKET_DATA ----
// 512-byte device-filled buffer; only spec word offsets are read (by the
// FORCEINLINE helpers below), so model as a raw word array — faithful to the
// ATA wire layout and avoids a fragile 370-line bitfield transliteration.
// Exact ATA byte layout for the few named fields hwide.c touches; the
// FORCEINLINE helpers read remaining spec words via the w(idx) accessor.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IDENTIFY_DEVICE_DATA {
    pub GeneralConfiguration: u16,    // word 0
    pub NumCylinders: u16,            // word 1
    pub SpecificConfiguration: u16,   // word 2
    pub NumHeads: u16,                // word 3
    _pad8: [u8; 4],                   // words 4-5 Retired1[2]
    pub NumSectorsPerTrack: u16,      // word 6
    _pad14: [u8; 6],                  // words 7-9 VendorUnique1[3]
    pub SerialNumber: [UCHAR; 20],    // words 10-19
    _pad40: [u8; 6],                  // words 20-22 Retired2[2]+Obsolete1
    pub FirmwareRevision: [UCHAR; 8], // words 23-26
    pub ModelNumber: [UCHAR; 40],     // words 27-46
    _rest: [u8; 512 - 94],            // words 47-255 (read via w())
}
pub type PIDENTIFY_DEVICE_DATA = *mut IDENTIFY_DEVICE_DATA;
impl IDENTIFY_DEVICE_DATA {
    /// ATA IDENTIFY spec word `idx` (0..256).
    #[inline]
    pub unsafe fn w(&self, idx: usize) -> u16 {
        *(self as *const Self as *const u16).add(idx)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IDENTIFY_PACKET_DATA {
    pub w: [u16; 256],
}
pub type PIDENTIFY_PACKET_DATA = *mut IDENTIFY_PACKET_DATA;

// ---- <scsi.h>: SENSE_DATA (18 bytes; bitfield byte 2 = SenseKey:4|...) ----
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SENSE_DATA {
    pub b0: UCHAR, // ErrorCode:7 | Valid:1
    pub SegmentNumber: UCHAR,
    pub b2: UCHAR, // SenseKey:4 | Reserved:1 | IncorrectLength:1 | EndOfMedia:1 | FileMark:1
    pub Information: [UCHAR; 4],
    pub AdditionalSenseLength: UCHAR,
    pub CommandSpecificInformation: [UCHAR; 4],
    pub AdditionalSenseCode: UCHAR,
    pub AdditionalSenseCodeQualifier: UCHAR,
    pub FieldReplaceableUnitCode: UCHAR,
    pub SenseKeySpecific: [UCHAR; 3],
}
pub type PSENSE_DATA = *mut SENSE_DATA;
impl SENSE_DATA {
    #[inline]
    pub fn SenseKey(&self) -> UCHAR { self.b2 & 0x0F }
}

// ---- <scsi.h>: READ_CAPACITY_DATA / READ_CAPACITY16_DATA ----
#[repr(C)]
#[derive(Clone, Copy)]
pub struct READ_CAPACITY_DATA {
    pub LogicalBlockAddress: ULONG,
    pub BytesPerBlock: ULONG,
}
pub type PREAD_CAPACITY_DATA = *mut READ_CAPACITY_DATA;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct READ_CAPACITY16_DATA {
    pub LogicalBlockAddress: LARGE_INTEGER,
    pub BytesPerBlock: ULONG,
    pub b12: UCHAR, // ProtectionEnable:1 | ProtectionType:3 | RcBasis:2 | Reserved:2
    pub b13: UCHAR, // LogicalPerPhysicalExponent:4 | ProtectionInfoExponent:4
    pub b14: UCHAR, // LowestAlignedBlock_MSB:6 | LBPRZ:1 | LBPME:1
    pub LowestAlignedBlock_LSB: UCHAR,
    pub Reserved3: [UCHAR; 16],
}
pub type PREAD_CAPACITY16_DATA = *mut READ_CAPACITY16_DATA;

// ---- hwidep.h: HW_DEVICE_UNIT ----
#[repr(C)]
#[derive(Clone, Copy)]
pub union HwIdentify {
    pub IdentifyDeviceData: IDENTIFY_DEVICE_DATA,
    pub IdentifyPacketData: IDENTIFY_PACKET_DATA,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HW_DEVICE_UNIT {
    pub P: DEVICE_UNIT,
    pub Registers: IDE_REGISTERS,
    pub BytesToTransfer: ULONG,
    pub DataBuffer: PUCHAR,
    pub DrqByteCount: ULONG,
    pub MaximumTransferLength: ULONG,
    pub DeviceNumber: UCHAR,
    pub DeviceSelect: UCHAR,
    pub CdbSize: UCHAR,
    pub MultiSectorTransfer: UCHAR,
    pub u: HwIdentify,
}
pub type PHW_DEVICE_UNIT = *mut HW_DEVICE_UNIT;

// hwide.c file-local globals.
// static const IDE_REG AtapChannelBaseArray[] = { 0x1F0, 0x170, 0x1E8, 0x168 };
static AtapChannelBaseArray: [IDE_REG; 4] = [0x1F0, 0x170, 0x1E8, 0x168];
pub const CHANNEL_MAX_CHANNELS: usize = 4;
// static PHW_DEVICE_UNIT AtapUnits[CHANNEL_MAX_CHANNELS * CHANNEL_MAX_DEVICES];
pub type ATA_UNIT = PHW_DEVICE_UNIT;
static mut AtapUnits: [PHW_DEVICE_UNIT; CHANNEL_MAX_CHANNELS * CHANNEL_MAX_DEVICES] =
    [core::ptr::null_mut(); CHANNEL_MAX_CHANNELS * CHANNEL_MAX_DEVICES];

// ---- hwidep.h port-I/O macros → faithful i386 inline-asm primitives ----
// arm64 is UEFI-only with no x86 port-I/O hardware; the hwide IDE driver is
// never exercised on the arm64 uefildr boot path. Gate the x86 asm bodies
// per-arch and give aarch64 unreachable stubs so the crate compiles for
// aarch64 without touching the i386/amd64 code paths.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
unsafe fn ATA_WRITE(Port: IDE_REG, Value: UCHAR) {
    core::arch::asm!("out dx, al", in("dx") Port, in("al") Value,
        options(nomem, nostack, preserves_flags));
}
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn ATA_WRITE(_Port: IDE_REG, _Value: UCHAR) {
    unreachable!("hwide IDE port-I/O is not used on the arm64 UEFI boot path");
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
unsafe fn ATA_READ(Port: IDE_REG) -> UCHAR {
    let v: UCHAR;
    core::arch::asm!("in al, dx", out("al") v, in("dx") Port,
        options(nomem, nostack, preserves_flags));
    v
}
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn ATA_READ(_Port: IDE_REG) -> UCHAR {
    unreachable!("hwide IDE port-I/O is not used on the arm64 UEFI boot path");
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
unsafe fn READ_PORT_USHORT(Port: PUSHORT) -> USHORT {
    let v: USHORT;
    core::arch::asm!("in ax, dx", out("ax") v, in("dx") Port as USHORT,
        options(nomem, nostack, preserves_flags));
    v
}
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn READ_PORT_USHORT(_Port: PUSHORT) -> USHORT {
    unreachable!("hwide IDE port-I/O is not used on the arm64 UEFI boot path");
}
// rep ins/outs string ops: the pointer register is edi/esi on i386 but
// rdi/rsi on amd64. i386 bodies kept byte-identical; amd64 bodies use the
// 64-bit pointer regs directly (LLVM does NOT reserve rsi/rdi on amd64, so
// no esi save/restore xchg is needed there — and a 32-bit reg in the
// template would truncate the 64-bit Buffer pointer / mismatch operands).
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn ATA_READ_BLOCK_16(Port: IDE_REG, Buffer: PUSHORT, Count: ULONG) {
    core::arch::asm!("rep insw",
        in("dx") Port, inout("edi") Buffer => _, inout("ecx") Count => _,
        options(nostack, preserves_flags));
}
#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn ATA_READ_BLOCK_16(Port: IDE_REG, Buffer: PUSHORT, Count: ULONG) {
    core::arch::asm!("rep insw",
        in("dx") Port, inout("rdi") Buffer => _, inout("ecx") Count => _,
        options(nostack, preserves_flags));
}
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn ATA_READ_BLOCK_16(_Port: IDE_REG, _Buffer: PUSHORT, _Count: ULONG) {
    unreachable!("hwide IDE port-I/O is not used on the arm64 UEFI boot path");
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn ATA_WRITE_BLOCK_16(Port: IDE_REG, Buffer: PUSHORT, Count: ULONG) {
    // LLVM reserves esi; xchg it in around the rep (esi saved/restored).
    core::arch::asm!(
        "xchg esi, {buf}",
        "rep outsw",
        "xchg esi, {buf}",
        buf = inout(reg) Buffer => _,
        in("dx") Port, inout("ecx") Count => _,
        options(nostack, preserves_flags));
}
#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn ATA_WRITE_BLOCK_16(Port: IDE_REG, Buffer: PUSHORT, Count: ULONG) {
    core::arch::asm!("rep outsw",
        in("dx") Port, inout("rsi") Buffer => _, inout("ecx") Count => _,
        options(nostack, preserves_flags));
}
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn ATA_WRITE_BLOCK_16(_Port: IDE_REG, _Buffer: PUSHORT, _Count: ULONG) {
    unreachable!("hwide IDE port-I/O is not used on the arm64 UEFI boot path");
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn ATA_READ_BLOCK_32(Port: IDE_REG, Buffer: PULONG, Count: ULONG) {
    core::arch::asm!("rep insd",
        in("dx") Port, inout("edi") Buffer => _, inout("ecx") Count => _,
        options(nostack, preserves_flags));
}
#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn ATA_READ_BLOCK_32(Port: IDE_REG, Buffer: PULONG, Count: ULONG) {
    core::arch::asm!("rep insd",
        in("dx") Port, inout("rdi") Buffer => _, inout("ecx") Count => _,
        options(nostack, preserves_flags));
}
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn ATA_READ_BLOCK_32(_Port: IDE_REG, _Buffer: PULONG, _Count: ULONG) {
    unreachable!("hwide IDE port-I/O is not used on the arm64 UEFI boot path");
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn ATA_WRITE_BLOCK_32(Port: IDE_REG, Buffer: PULONG, Count: ULONG) {
    // LLVM reserves esi; xchg it in around the rep (esi saved/restored).
    core::arch::asm!(
        "xchg esi, {buf}",
        "rep outsd",
        "xchg esi, {buf}",
        buf = inout(reg) Buffer => _,
        in("dx") Port, inout("ecx") Count => _,
        options(nostack, preserves_flags));
}
#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn ATA_WRITE_BLOCK_32(Port: IDE_REG, Buffer: PULONG, Count: ULONG) {
    core::arch::asm!("rep outsd",
        in("dx") Port, inout("rsi") Buffer => _, inout("ecx") Count => _,
        options(nostack, preserves_flags));
}
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn ATA_WRITE_BLOCK_32(_Port: IDE_REG, _Buffer: PULONG, _Count: ULONG) {
    unreachable!("hwide IDE port-I/O is not used on the arm64 UEFI boot path");
}
#[inline]
unsafe fn ATA_IO_WAIT() { StallExecutionProcessor(1); }
#[inline]
fn DEV_SLAVE(DeviceNumber: UCHAR) -> UCHAR { DeviceNumber }

// ---- hwidep.h FORCEINLINE helpers (no exported symbol; real Rust fns) ----
#[inline]
unsafe fn AtaDevIsIdentifyDataValid(IdentifyData: PIDENTIFY_DEVICE_DATA) -> BOOLEAN {
    // Bits 0:8 of word 255
    if ((*IdentifyData).w(255) & 0xFF) != 0xA5 {
        return TRUE;
    }
    let p = IdentifyData as *const u8;
    let mut crc: u8 = 0;
    let mut i = 0usize;
    while i < core::mem::size_of::<IDENTIFY_DEVICE_DATA>() {
        crc = crc.wrapping_add(*p.add(i));
        i += 1;
    }
    if crc == 0 { TRUE } else { FALSE }
}
#[inline]
unsafe fn AtaDevCdbSizeInWords(IdentifyPacketData: PIDENTIFY_PACKET_DATA) -> ULONG {
    // Bits 0:2 of word 0 (GeneralConfiguration.PacketType, bits 0-1)
    if ((*IdentifyPacketData).w[0] & 0x3) != 0 { 8 } else { 6 }
}
#[inline]
unsafe fn AtaDevHasLbaTranslation(IdentifyData: *const c_void) -> BOOLEAN {
    let id = IdentifyData as *const IDENTIFY_DEVICE_DATA;
    // Bit 9 of word 49
    if ((*id).w(49) >> 9) & 1 != 0 { TRUE } else { FALSE }
}
#[inline]
unsafe fn AtaDevUserAddressableSectors28Bit(IdentifyData: *const c_void) -> ULONG64 {
    // Words 60-61
    let id = IdentifyData as *const IDENTIFY_DEVICE_DATA;
    ((*id).w(60) as ULONG64) | (((*id).w(61) as ULONG64) << 16)
}
#[inline]
unsafe fn AtaDevUserAddressableSectors48Bit(IdentifyData: *const c_void) -> ULONG64 {
    // Words 100-103: Max48BitLBA is ULONG[2]
    let id = IdentifyData as *const IDENTIFY_DEVICE_DATA;
    let lo = (*id).w(100) as ULONG64 | (((*id).w(101) as ULONG64) << 16);
    let hi = (*id).w(102) as ULONG64 | (((*id).w(103) as ULONG64) << 16);
    (hi << 32) | lo
}
#[inline]
unsafe fn AtaDevHas48BitAddressFeature(IdentifyData: *const c_void) -> BOOLEAN {
    // Word 83: bits 14:15 == WordValid83 (==1 valid); bit 10 == BigLba
    let id = IdentifyData as *const IDENTIFY_DEVICE_DATA;
    if (((*id).w(83) >> 14) & 0x3) == 1 {
        if ((*id).w(83) >> 10) & 1 != 0 { return TRUE; }
        return FALSE;
    }
    FALSE
}
#[inline]
unsafe fn AtaDevIsCurrentGeometryValid(IdentifyData: *const c_void) -> BOOLEAN {
    let id = IdentifyData as *const IDENTIFY_DEVICE_DATA;
    let tfv = (*id).w(53) & 1;
    let cyl = (*id).w(54);
    let heads = (*id).w(55);
    let spt = (*id).w(56);
    if tfv != 0 && cyl != 0 && cyl <= 63 && heads != 0 && heads <= 16 && spt != 0 {
        TRUE
    } else {
        FALSE
    }
}
#[inline]
unsafe fn AtaDevDefaultChsTranslation(
    IdentifyData: *const c_void,
    Cylinders: *mut USHORT,
    Heads: *mut USHORT,
    SectorsPerTrack: *mut USHORT,
) {
    let id = IdentifyData as *const IDENTIFY_DEVICE_DATA;
    *Cylinders = (*id).w(1); // Word 1
    *Heads = (*id).w(3); // Word 3
    *SectorsPerTrack = (*id).w(6); // Word 6
}
#[inline]
unsafe fn AtaDevCurrentChsTranslation(
    IdentifyData: *const c_void,
    Cylinders: *mut USHORT,
    Heads: *mut USHORT,
    SectorsPerTrack: *mut USHORT,
) {
    let id = IdentifyData as *const IDENTIFY_DEVICE_DATA;
    *Cylinders = (*id).w(54); // NumberOfCurrentCylinders
    *Heads = (*id).w(55); // NumberOfCurrentHeads
    *SectorsPerTrack = (*id).w(56); // CurrentSectorsPerTrack
}
#[inline]
unsafe fn AtaDevCurrentSectorsPerDrq(IdentifyData: *const c_void) -> UCHAR {
    let id = IdentifyData as *const IDENTIFY_DEVICE_DATA;
    // Bit 8 of word 59 = MultiSectorSettingValid
    if ((*id).w(59) >> 8) & 1 == 0 {
        return 0;
    }
    let m = ((*id).w(59) & 0xFF) as UCHAR; // CurrentMultiSectorSetting
    if m > 0 && (m & (m - 1)) == 0 { m } else { 0 }
}
#[inline]
unsafe fn AtaDevMaximumSectorsPerDrq(IdentifyData: *const c_void) -> UCHAR {
    let id = IdentifyData as *const IDENTIFY_DEVICE_DATA;
    // MaximumBlockTransfer = bits 0:7 of word 47
    let m = ((*id).w(47) & 0xFF) as UCHAR;
    if m > 0 && (m & (m - 1)) == 0 { m } else { 0 }
}
#[inline]
unsafe fn AtaDevBytesPerLogicalSector(IdentifyData: *const c_void) -> ULONG {
    let id = IdentifyData as *const IDENTIFY_DEVICE_DATA;
    // Word 106: bit14 == Reserved1 (==1) && bit12 == LogicalSectorLongerThan256Words
    let word_count: ULONG;
    if (((*id).w(106) >> 14) & 1) == 1 && (((*id).w(106) >> 12) & 1) != 0 {
        // Words 116-117 (WordsPerLogicalSector ULONG)
        word_count = (*id).w(116) as ULONG | (((*id).w(117) as ULONG) << 16);
    } else {
        word_count = 256;
    }
    word_count * core::mem::size_of::<USHORT>() as ULONG
}
#[inline]
fn AtaCommandUseLba48(SectorNumber: ULONG64, SectorCount: ULONG) -> BOOLEAN {
    if (SectorNumber + SectorCount as ULONG64) >= ATA_MAX_LBA_28 || SectorCount > 0x100 {
        TRUE
    } else {
        FALSE
    }
}

// ---- hwide.c static fns dropped by rustconv (faithful translations) ----
unsafe fn AtapSelectDevice(Registers: PIDE_REGISTERS, DeviceNumber: UCHAR) {
    ATA_WRITE((*Registers).Device, (DEV_SLAVE(DeviceNumber) << 4) | IDE_DRIVE_SELECT);
    ATA_IO_WAIT();
}

unsafe fn AtapWait(
    Registers: PIDE_REGISTERS,
    Timeout: ULONG,
    Mask: UCHAR,
    Value: UCHAR,
) -> UCHAR {
    let mut IdeStatus: UCHAR = 0xFF;
    let mut i: ULONG = 0;
    while i < Timeout {
        IdeStatus = ATA_READ((*Registers).__bindgen_anon_1.Status);
        if (IdeStatus & Mask) == Value {
            break;
        }
        if IdeStatus == 0xFF {
            break;
        }
        StallExecutionProcessor(10);
        i += 1;
    }
    IdeStatus
}

unsafe fn AtapSendCdb(
    DeviceUnit: PHW_DEVICE_UNIT,
    Request: PATA_DEVICE_REQUEST)
{

    #[cfg(feature = "ATA_SUPPORT_32_BIT_IO")]
    {
        ATA_WRITE_BLOCK_32((*DeviceUnit).Registers.Data as PULONG,
                          (*Request).u.Cdb as PULONG,
                          (*DeviceUnit).CdbSize / core::mem::size_of::<USHORT>() as ULONG);
    }
    #[cfg(not(feature = "ATA_SUPPORT_32_BIT_IO"))]
    {
        ATA_WRITE_BLOCK_16((*DeviceUnit).Registers.Data,
                          (*Request).u.Cdb.as_mut_ptr() as PUSHORT,
                          (*DeviceUnit).CdbSize as ULONG);
    }
}

unsafe fn AtapPioDataIn(
    DeviceUnit: PHW_DEVICE_UNIT,
    ByteCount: ULONG,
) {
    let ByteCount = core::cmp::min(ByteCount, (*DeviceUnit).BytesToTransfer);

    #[cfg(feature = "ATA_SUPPORT_32_BIT_IO")]
    {
        if (ByteCount & (core::mem::size_of::<ULONG>() as u32 - 1)) == 0 {
            ATA_READ_BLOCK_32(
                (*DeviceUnit).Registers.Data,
                (*DeviceUnit).DataBuffer as PULONG,
                ByteCount / core::mem::size_of::<ULONG>() as u32,
            );
        } else {
            ATA_READ_BLOCK_16(
                (*DeviceUnit).Registers.Data,
                (*DeviceUnit).DataBuffer as PUSHORT,
                ByteCount / core::mem::size_of::<USHORT>() as u32,
            );
        }
    }
    #[cfg(not(feature = "ATA_SUPPORT_32_BIT_IO"))]
    {
        ATA_READ_BLOCK_16(
            (*DeviceUnit).Registers.Data,
            (*DeviceUnit).DataBuffer as PUSHORT,
            ByteCount / core::mem::size_of::<USHORT>() as u32,
        );
    }

    (*DeviceUnit).DataBuffer = (*DeviceUnit).DataBuffer.add(ByteCount as usize);
    (*DeviceUnit).BytesToTransfer -= ByteCount;
}

unsafe fn AtapWaitForRegisterAccess(
    Registers: *mut IDE_REGISTERS,
    DeviceNumber: UCHAR,
) -> BOOLEAN {
    let mut i: ULONG;

    i = 0;
    while i < ATA_TIME_RESET_SELECT {
        AtapSelectDevice(Registers, DeviceNumber);

        ATA_WRITE((*Registers).lm.ByteCountLow, 0xAA);
        ATA_WRITE((*Registers).lm.ByteCountLow, 0x55);
        ATA_WRITE((*Registers).lm.ByteCountLow, 0xAA);
        if ATA_READ((*Registers).lm.ByteCountLow) == 0xAA {
            return 1u8;
        }

        StallExecutionProcessor(10);

        i += 1;
    }

    0u8
}

unsafe fn AtapSoftwareReset(Registers: PIDE_REGISTERS) {
    
    ATA_WRITE((*Registers).ctl.Control, IDE_DC_RESET_CONTROLLER | IDE_DC_ALWAYS);
    StallExecutionProcessor(20);
    ATA_WRITE((*Registers).ctl.Control, IDE_DC_DISABLE_INTERRUPTS | IDE_DC_ALWAYS);
    StallExecutionProcessor(20);
}

unsafe fn AtapDrainDeviceBuffer(
    Registers: PIDE_REGISTERS)
{
    let mut IdeStatus: UCHAR;
    let _i: ULONG;

    for _i in 0..(0x10000 / core::mem::size_of::<USHORT>() as ULONG) {
        IdeStatus = ATA_READ((*Registers).__bindgen_anon_1.Status);
        if (IdeStatus & IDE_STATUS_DRQ) == 0 {
            break;
        }

        READ_PORT_USHORT((*Registers).Data as ULONG_PTR as PUSHORT);
    }
}

unsafe fn AtapPerformSoftwareReset(
    DeviceUnit: PHW_DEVICE_UNIT) -> BOOLEAN
{

    let Registers: PIDE_REGISTERS = core::ptr::addr_of_mut!((*DeviceUnit).Registers);
    let IdeStatus: UCHAR;

    ERR!("Reset device at %X:%u\n", ((*Registers).Data) as usize, ((*DeviceUnit).DeviceNumber) as usize);

    AtapDrainDeviceBuffer(Registers);

    AtapSoftwareReset(Registers);

    if DEV_SLAVE((*DeviceUnit).DeviceNumber) != 0 {
        if AtapWaitForRegisterAccess(Registers, (*DeviceUnit).DeviceNumber) == 0 {
            return 0u8;
        }
    }

    IdeStatus = AtapWait(Registers, ATA_TIME_BUSY_RESET, IDE_STATUS_BUSY, 0);
    if (IdeStatus & IDE_STATUS_BUSY) != 0 {
        return 0u8;
    }

    return 1u8;
}

unsafe fn AtapProcessAtapiRequest(
    DeviceUnit: PHW_DEVICE_UNIT,
    Request: PATA_DEVICE_REQUEST,
    IdeStatus: UCHAR,
) -> UCHAR {

    let mut InterruptReason: UCHAR;

    InterruptReason = ATA_READ((*DeviceUnit).Registers.sc_ir.InterruptReason);
    InterruptReason &= ATAPI_INT_REASON_MASK;
    InterruptReason |= IdeStatus & IDE_STATUS_DRQ;

    if (InterruptReason == ATAPI_INT_REASON_COD) || (InterruptReason == ATAPI_INT_REASON_IO) {
        let IdeStatus = AtapWait(
            core::ptr::addr_of_mut!((*DeviceUnit).Registers),
            ATA_TIME_DRQ_ASSERT,
            IDE_STATUS_DRQ,
            IDE_STATUS_DRQ,
        );

        InterruptReason = ATA_READ((*DeviceUnit).Registers.sc_ir.InterruptReason);
        InterruptReason &= ATAPI_INT_REASON_MASK;
        InterruptReason |= IdeStatus & IDE_STATUS_DRQ;
    }

    match InterruptReason {
        ATAPI_INT_REASON_AWAIT_CDB => {
            if ((*Request).Flags & REQUEST_FLAG_AWAIT_CDB) == 0 {
                return ATA_STATUS_RESET;
            }

            (*Request).Flags &= !REQUEST_FLAG_AWAIT_CDB;

            AtapSendCdb(DeviceUnit, Request);
            return ATA_STATUS_PENDING;
        }

        ATAPI_INT_REASON_DATA_IN => {
            if (*Request).DataBuffer.is_null() || ((*Request).Flags & REQUEST_FLAG_AWAIT_CDB) != 0 {
                return ATA_STATUS_RESET;
            }

            let mut ByteCount: ULONG = ATA_READ((*DeviceUnit).Registers.lm.ByteCountLow) as ULONG;
            ByteCount |= (ATA_READ((*DeviceUnit).Registers.lh.ByteCountHigh) as ULONG) << 8;

            AtapPioDataIn(DeviceUnit, ByteCount);
            return ATA_STATUS_PENDING;
        }

        ATAPI_INT_REASON_STATUS_NEC => {
            if ((*DeviceUnit).P.Flags & ATA_DEVICE_IS_NEC_CDR260) == 0 {
                return ATA_STATUS_RESET;
            }

            if (IdeStatus & (IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT)) != 0 {
                return ATA_STATUS_ERROR;
            }
        }

        ATAPI_INT_REASON_STATUS => {
            if (IdeStatus & (IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT)) != 0 {
                return ATA_STATUS_ERROR;
            }
        }

        _ => {
            return ATA_STATUS_RESET;
        }
    }

    ATA_STATUS_SUCCESS
}

unsafe fn AtapProcessAtaRequest(
    DeviceUnit: PHW_DEVICE_UNIT,
    Request: PATA_DEVICE_REQUEST,
    mut IdeStatus: UCHAR,
) -> UCHAR {
    if (IdeStatus & (IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT)) != 0 {
        if (IdeStatus & IDE_STATUS_DRQ) != 0 {
            return ATA_STATUS_RESET;
        } else {
            return ATA_STATUS_ERROR;
        }
    }

    if !(*Request).DataBuffer.is_null() {
        if (IdeStatus & IDE_STATUS_DRQ) == 0 {
            IdeStatus = AtapWait(
                core::ptr::addr_of_mut!((*DeviceUnit).Registers),
                ATA_TIME_DRQ_ASSERT,
                IDE_STATUS_DRQ,
                IDE_STATUS_DRQ,
            );
            if (IdeStatus & IDE_STATUS_DRQ) == 0 {
                return ATA_STATUS_RESET;
            }
        }

        AtapPioDataIn(DeviceUnit, (*DeviceUnit).DrqByteCount);

        if (*DeviceUnit).BytesToTransfer != 0 {
            return ATA_STATUS_PENDING;
        }

        IdeStatus = AtapWait(
            core::ptr::addr_of_mut!((*DeviceUnit).Registers),
            ATA_TIME_DRQ_CLEAR,
            IDE_STATUS_BUSY | IDE_STATUS_DRQ,
            0,
        );
        if (IdeStatus & (IDE_STATUS_BUSY | IDE_STATUS_DRQ)) != 0 {
            return ATA_STATUS_RESET;
        }

        if (IdeStatus & (IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT)) != 0 {
            return ATA_STATUS_ERROR;
        }
    }

    return ATA_STATUS_SUCCESS;
}

unsafe fn AtapProcessRequest(
    DeviceUnit: PHW_DEVICE_UNIT,
    Request: PATA_DEVICE_REQUEST,
    IdeStatus: UCHAR,
) -> UCHAR {

    let mut AtaStatus: UCHAR;

    if ((*Request).Flags & REQUEST_FLAG_PACKET_COMMAND) == 0 {
        return AtapProcessAtaRequest(DeviceUnit, Request, IdeStatus);
    }

    AtaStatus = AtapProcessAtapiRequest(DeviceUnit, Request, IdeStatus);

    if ((*DeviceUnit).P.Flags & ATA_DEVICE_IS_NEC_CDR260) != 0 && AtaStatus == ATA_STATUS_PENDING {
        let OldInterruptReason: UCHAR = ATA_READ((*DeviceUnit).Registers.sc_ir.InterruptReason);
        let mut NewInterruptReason: UCHAR;
        let mut i: ULONG = ATA_TIME_BUSY_POLL;

        while i > 0 {
            NewInterruptReason = ATA_READ((*DeviceUnit).Registers.sc_ir.InterruptReason);
            if NewInterruptReason != OldInterruptReason {
                break;
            }

            StallExecutionProcessor(10);
            i -= 1;
        }

        if i == 0 {
            AtaStatus = ATA_STATUS_RESET;
        }
    }

    AtaStatus
}

unsafe fn AtapIssuePacketCommand(
    DeviceUnit: PHW_DEVICE_UNIT,
    Request: PATA_DEVICE_REQUEST)
{
    let ByteCount: USHORT;

    (*Request).Flags |= REQUEST_FLAG_AWAIT_CDB;

    ByteCount = min(((*Request).DataTransferLength) as u16 , 0xFFFE);

    ATA_WRITE((*DeviceUnit).Registers.lm.ByteCountLow, (ByteCount >> 0) as UCHAR);
    ATA_WRITE((*DeviceUnit).Registers.lh.ByteCountHigh, (ByteCount >> 8) as UCHAR);
    ATA_WRITE((*DeviceUnit).Registers.feat_err.Features, IDE_FEATURE_PIO);
    ATA_WRITE((*DeviceUnit).Registers.__bindgen_anon_1.Command, IDE_COMMAND_ATAPI_PACKET);
}

unsafe fn AtapLoadTaskFile(
    DeviceUnit: PHW_DEVICE_UNIT,
    Request: PATA_DEVICE_REQUEST)
{
    let TaskFile: PATA_TASKFILE = core::ptr::addr_of_mut!((*Request).u.TaskFile);
    let mut i: ULONG;
    let BlockCount: ULONG;

    i = if (*Request).Flags & REQUEST_FLAG_LBA48 != 0 { 2 } else { 1 };

    while i > 0 {
        i -= 1;
        ATA_WRITE((*DeviceUnit).Registers.feat_err.Features, (*TaskFile).Data[i as usize].Feature);
        ATA_WRITE((*DeviceUnit).Registers.sc_ir.SectorCount, (*TaskFile).Data[i as usize].SectorCount);
        ATA_WRITE((*DeviceUnit).Registers.LbaLow, (*TaskFile).Data[i as usize].LowLba);
        ATA_WRITE((*DeviceUnit).Registers.lm.LbaMid, (*TaskFile).Data[i as usize].MidLba);
        ATA_WRITE((*DeviceUnit).Registers.lh.LbaHigh, (*TaskFile).Data[i as usize].HighLba);
    }
    if (*Request).Flags & REQUEST_FLAG_SET_DEVICE_REGISTER != 0 {
        ATA_WRITE((*DeviceUnit).Registers.Device, (*TaskFile).DriveSelect);
    }
    ATA_WRITE((*DeviceUnit).Registers.__bindgen_anon_1.Command, (*TaskFile).Command);

    if (*Request).Flags & REQUEST_FLAG_READ_WRITE_MULTIPLE != 0 {
        BlockCount = ((*DeviceUnit).MultiSectorTransfer) as u32 ;
    } else {
        BlockCount = 1;
    }
    (*DeviceUnit).DrqByteCount = BlockCount * (*DeviceUnit).P.SectorSize;
}

unsafe fn AtapSendCommand(
    DeviceUnit: PHW_DEVICE_UNIT,
    Request: PATA_DEVICE_REQUEST,
) -> UCHAR {

    let mut IdeStatus: UCHAR;
    let _AtaStatus: UCHAR;

    (*DeviceUnit).BytesToTransfer = (*Request).DataTransferLength;
    (*DeviceUnit).DataBuffer = (*Request).DataBuffer as PUCHAR;

    AtapSelectDevice(core::ptr::addr_of_mut!((*DeviceUnit).Registers), (*DeviceUnit).DeviceNumber);
    IdeStatus = AtapWait(core::ptr::addr_of_mut!((*DeviceUnit).Registers), ATA_TIME_BUSY_SELECT, IDE_STATUS_BUSY, 0);
    if IdeStatus & IDE_STATUS_BUSY != 0 {
        return ATA_STATUS_RETRY;
    }

    ATA_WRITE((*DeviceUnit).Registers.ctl.Control, IDE_DC_DISABLE_INTERRUPTS | IDE_DC_ALWAYS);

    if (*Request).Flags & REQUEST_FLAG_PACKET_COMMAND != 0 {
        AtapIssuePacketCommand(DeviceUnit, Request);
    } else {
        AtapLoadTaskFile(DeviceUnit, Request);
    }

    loop {
        ATA_IO_WAIT();

        IdeStatus = AtapWait(core::ptr::addr_of_mut!((*DeviceUnit).Registers), ATA_TIME_BUSY_POLL, IDE_STATUS_BUSY, 0);
        if IdeStatus & IDE_STATUS_BUSY != 0 {
            return ATA_STATUS_RESET;
        }

        let AtaStatus = AtapProcessRequest(DeviceUnit, Request, IdeStatus);
        if AtaStatus != ATA_STATUS_PENDING {
            return AtaStatus;
        }
    }
}

unsafe fn AtapAtapiBuildRequestSense(
    Request: PATA_DEVICE_REQUEST,
    SenseData: PSENSE_DATA)
{
    (*Request).Flags = REQUEST_FLAG_PACKET_COMMAND;
    (*Request).DataBuffer = SenseData as PVOID;
    (*Request).DataTransferLength = core::mem::size_of::<SENSE_DATA>() as ULONG;
    (*Request).u.Cdb[0] = SCSIOP_REQUEST_SENSE;
    (*Request).u.Cdb[4] = (*Request).DataTransferLength as UCHAR;
}

unsafe fn AtapAtapiHandleError(
    DeviceUnit: PHW_DEVICE_UNIT) -> BOOLEAN
{

    let mut Request: ATA_DEVICE_REQUEST = core::mem::zeroed();
    let mut SenseData: SENSE_DATA = core::mem::zeroed();

    AtapAtapiBuildRequestSense(&mut Request, &mut SenseData);
    if AtapSendCommand(DeviceUnit, &mut Request) != ATA_STATUS_SUCCESS {
        return 0u8;
    }

    ERR!("SK %02X, ASC %02X, ASCQ %02X\n", (SenseData.SenseKey()) as usize, (SenseData.AdditionalSenseCode) as usize, (SenseData.AdditionalSenseCodeQualifier) as usize);

    return 1u8;
}

unsafe fn AtapIssueCommand(
    DeviceUnit: PHW_DEVICE_UNIT,
    Request: PATA_DEVICE_REQUEST) -> BOOLEAN
{

    let mut RetryCount: ULONG;

    RetryCount = 0;
    while RetryCount < 3 {
        let AtaStatus: UCHAR;

        AtaStatus = AtapSendCommand(DeviceUnit, Request);

        if (AtaStatus != ATA_STATUS_SUCCESS) &&
            ((*Request).Flags & REQUEST_FLAG_IDENTIFY_COMMAND) == 0 {
            ERR!("ATA%s command %02X failed %u %02X:%02X at %X:%u\n", (if ((*Request).Flags & REQUEST_FLAG_PACKET_COMMAND) != 0 { b"PI\0".as_ptr() as PCHAR } else { b"\0".as_ptr() as PCHAR }) as usize, (if ((*Request).Flags & REQUEST_FLAG_PACKET_COMMAND) != 0 {
                    (*Request).u.Cdb[0]
                } else {
                    (*Request).u.TaskFile.Command
                }) as usize, (AtaStatus) as usize, (ATA_READ((*DeviceUnit).Registers.__bindgen_anon_1.Status)) as usize, (ATA_READ((*DeviceUnit).Registers.feat_err.Error)) as usize, ((*DeviceUnit).Registers.Data) as usize, ((*DeviceUnit).DeviceNumber) as usize);
        }

        match AtaStatus {
            ATA_STATUS_SUCCESS => {
                return 1u8;
            }

            ATA_STATUS_RETRY => {
            }

            ATA_STATUS_RESET => {
                if ((*Request).Flags & REQUEST_FLAG_IDENTIFY_COMMAND) != 0 {
                    if ATA_READ((*DeviceUnit).Registers.__bindgen_anon_1.Status) == 0 {
                        return 0u8;
                    }
                }

                if AtapPerformSoftwareReset(DeviceUnit) == 0 {
                    return 0u8;
                }
            }

            _ => {
                if ((*Request).Flags & REQUEST_FLAG_PACKET_COMMAND) != 0 {
                    if AtapAtapiHandleError(DeviceUnit) == 0 {
                        return 0u8;
                    }
                }

                if ((*Request).Flags & REQUEST_FLAG_READ_COMMAND) == 0 {
                    return 0u8;
                }
            }
        }

        RetryCount += 1;
    }

    return 0u8;
}

unsafe fn AtapGetReadCommand(
    Request: PATA_DEVICE_REQUEST
) -> UCHAR
{
    const AtapReadCommandMap: [[UCHAR; 2]; 2] = [
        [IDE_COMMAND_READ, IDE_COMMAND_READ_EXT],
        [IDE_COMMAND_READ_MULTIPLE, IDE_COMMAND_READ_MULTIPLE_EXT],
    ];

    AtapReadCommandMap[if ((*Request).Flags & REQUEST_FLAG_READ_WRITE_MULTIPLE) != 0 { 1 } else { 0 }]
                      [if ((*Request).Flags & REQUEST_FLAG_LBA48) != 0 { 1 } else { 0 }]
}

unsafe fn AtapBuildReadTaskFile(
    DeviceUnit: PHW_DEVICE_UNIT,
    Request: PATA_DEVICE_REQUEST,
    Lba: ULONG64,
    SectorCount: ULONG)
{
    let TaskFile: PATA_TASKFILE = core::ptr::addr_of_mut!((*Request).u.TaskFile);
    let mut DriveSelect: UCHAR;

    (*Request).Flags = REQUEST_FLAG_READ_COMMAND | REQUEST_FLAG_SET_DEVICE_REGISTER;

    if (*DeviceUnit).MultiSectorTransfer != 0
    {
        (*Request).Flags |= REQUEST_FLAG_READ_WRITE_MULTIPLE;
    }

    if ((*DeviceUnit).P.Flags & ATA_DEVICE_LBA) != 0
    {
        DriveSelect = IDE_LBA_MODE;

        (*TaskFile).Data[0].SectorCount = SectorCount as UCHAR;
        (*TaskFile).Data[0].LowLba = Lba as UCHAR;
        (*TaskFile).Data[0].MidLba = (Lba >> 8) as UCHAR;
        (*TaskFile).Data[0].HighLba = (Lba >> 16) as UCHAR;

        if (((*DeviceUnit).P.Flags & ATA_DEVICE_LBA48) != 0) && (AtaCommandUseLba48(Lba, SectorCount) != 0)
        {
            (*TaskFile).Data[1].SectorCount = (SectorCount >> 8) as UCHAR;
            (*TaskFile).Data[1].LowLba = (Lba >> 24) as UCHAR;
            (*TaskFile).Data[1].MidLba = (Lba >> 32) as UCHAR;
            (*TaskFile).Data[1].HighLba = (Lba >> 40) as UCHAR;

            (*Request).Flags |= REQUEST_FLAG_LBA48;
        }
        else
        {
            DriveSelect |= ((Lba >> 24) & 0x0F) as UCHAR;
        }
    }
    else
    {
        let ChsTemp: ULONG = (Lba as ULONG) / (*DeviceUnit).P.SectorsPerTrack;
        let Cylinder: ULONG = ChsTemp / (*DeviceUnit).P.Heads;
        let Head: ULONG = ChsTemp % (*DeviceUnit).P.Heads;
        let Sector: ULONG = ((Lba as ULONG) % (*DeviceUnit).P.SectorsPerTrack) + 1;

        (*TaskFile).Data[0].SectorCount = SectorCount as UCHAR;
        (*TaskFile).Data[0].LowLba = Sector as UCHAR;
        (*TaskFile).Data[0].MidLba = Cylinder as UCHAR;
        (*TaskFile).Data[0].HighLba = (Cylinder >> 8) as UCHAR;

        DriveSelect = Head as UCHAR;
    }
    (*TaskFile).DriveSelect = (*DeviceUnit).DeviceSelect | DriveSelect;
    (*TaskFile).Command = AtapGetReadCommand(Request);
}

unsafe fn AtapBuildReadPacketCommand(
    Request: PATA_DEVICE_REQUEST,
    Lba: u64,
    SectorCount: u32)
{
    (*Request).Flags = REQUEST_FLAG_READ_COMMAND | REQUEST_FLAG_PACKET_COMMAND;

    RtlZeroMemory(
        (*Request).u.Cdb.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of_val(&(*Request).u.Cdb)
    );

    if Lba > u32::MAX as u64
    {
        (*Request).u.Cdb[0] = SCSIOP_READ16;
        (*Request).u.Cdb[2] = (Lba >> 56) as u8;
        (*Request).u.Cdb[3] = (Lba >> 48) as u8;
        (*Request).u.Cdb[4] = (Lba >> 40) as u8;
        (*Request).u.Cdb[5] = (Lba >> 32) as u8;
        (*Request).u.Cdb[6] = (Lba >> 24) as u8;
        (*Request).u.Cdb[7] = (Lba >> 16) as u8;
        (*Request).u.Cdb[8] = (Lba >> 8) as u8;
        (*Request).u.Cdb[9] = (Lba >> 0) as u8;
        (*Request).u.Cdb[10] = (SectorCount >> 24) as u8;
        (*Request).u.Cdb[11] = (SectorCount >> 16) as u8;
        (*Request).u.Cdb[12] = (SectorCount >> 8) as u8;
        (*Request).u.Cdb[13] = (SectorCount >> 0) as u8;
    }
    else
    {
        (*Request).u.Cdb[0] = SCSIOP_READ;
        (*Request).u.Cdb[2] = (Lba >> 24) as u8;
        (*Request).u.Cdb[3] = (Lba >> 16) as u8;
        (*Request).u.Cdb[4] = (Lba >> 8) as u8;
        (*Request).u.Cdb[5] = (Lba >> 0) as u8;
        (*Request).u.Cdb[7] = (SectorCount >> 8) as u8;
        (*Request).u.Cdb[8] = (SectorCount >> 0) as u8;
    }
}

unsafe fn AtapIsDevicePresent(
    DeviceUnit: PHW_DEVICE_UNIT) -> BOOLEAN
{

    let Registers: PIDE_REGISTERS = core::ptr::addr_of_mut!((*DeviceUnit).Registers);
    let mut IdeStatus: UCHAR;

    AtapSelectDevice(Registers, (*DeviceUnit).DeviceNumber);

    IdeStatus = ATA_READ((*Registers).__bindgen_anon_1.Status);
    if IdeStatus == 0xFF || IdeStatus == 0x7F {
        return 0u8;
    }

    ATA_WRITE((*Registers).lm.ByteCountLow, 0x55);
    ATA_WRITE((*Registers).lm.ByteCountLow, 0xAA);
    ATA_WRITE((*Registers).lm.ByteCountLow, 0x55);
    if ATA_READ((*Registers).lm.ByteCountLow) != 0x55 {
        return 0u8;
    }
    ATA_WRITE((*Registers).lh.ByteCountHigh, 0xAA);
    ATA_WRITE((*Registers).lh.ByteCountHigh, 0x55);
    ATA_WRITE((*Registers).lh.ByteCountHigh, 0xAA);
    if ATA_READ((*Registers).lh.ByteCountHigh) != 0xAA {
        return 0u8;
    }

    IdeStatus = AtapWait(core::ptr::addr_of_mut!((*DeviceUnit).Registers), ATA_TIME_BUSY_ENUM, IDE_STATUS_BUSY, 0);
    if (IdeStatus & (IDE_STATUS_BUSY | IDE_STATUS_DRQ)) != 0
    {
        ERR!("Device %X:%u is busy %02x\n", ((*Registers).Data) as usize, ((*DeviceUnit).DeviceNumber) as usize, (IdeStatus) as usize);

        if AtapPerformSoftwareReset(DeviceUnit) == 0 {
            return 0u8;
        }
    }

    return 1u8;
}

unsafe fn AtapReadIdentifyData(
    DeviceUnit: PHW_DEVICE_UNIT,
    Command: UCHAR) -> BOOLEAN
{
    let mut Request: ATA_DEVICE_REQUEST = core::mem::zeroed();
    let Id: PIDENTIFY_DEVICE_DATA = core::ptr::addr_of_mut!((*DeviceUnit).u.IdentifyDeviceData);

    Request.Flags = REQUEST_FLAG_IDENTIFY_COMMAND;
    Request.DataBuffer = Id as PVOID;
    Request.DataTransferLength = core::mem::size_of::<IDENTIFY_DEVICE_DATA>() as ULONG;
    Request.u.TaskFile.Command = Command;

    AtapIssueCommand(DeviceUnit, &mut Request)
}

unsafe fn AtapIdentifyDevice(
    DeviceUnit: PHW_DEVICE_UNIT) -> ATA_DEVICE_CLASS
{
    if AtapIsDevicePresent(DeviceUnit) == 0 {
        return DEV_NONE;
    }

    if AtapReadIdentifyData(DeviceUnit, IDE_COMMAND_IDENTIFY) != 0 {
        return DEV_ATA;
    }

    if AtapReadIdentifyData(DeviceUnit, IDE_COMMAND_ATAPI_IDENTIFY) != 0 {
        return DEV_ATAPI;
    }

    return DEV_NONE;
}

unsafe fn AtapAtapiReadCapacity16(
    DeviceUnit: PHW_DEVICE_UNIT,
    CapacityData: PREAD_CAPACITY16_DATA) -> BOOLEAN
{
    let mut Request: ATA_DEVICE_REQUEST = core::mem::zeroed();

    Request.Flags = REQUEST_FLAG_PACKET_COMMAND;
    Request.DataBuffer = CapacityData as PVOID;
    Request.DataTransferLength = core::mem::size_of::<READ_CAPACITY16_DATA>() as ULONG;
    Request.u.Cdb[0] = SCSIOP_SERVICE_ACTION_IN16;
    Request.u.Cdb[1] = SERVICE_ACTION_READ_CAPACITY16;
    Request.u.Cdb[13] = core::mem::size_of::<READ_CAPACITY16_DATA>() as UCHAR;

    AtapIssueCommand(DeviceUnit, &mut Request)
}

unsafe fn AtapAtapiReadCapacity10(
    DeviceUnit: PHW_DEVICE_UNIT,
    CapacityData: PREAD_CAPACITY_DATA) -> BOOLEAN
{
    let mut Request: ATA_DEVICE_REQUEST = core::mem::zeroed();

    Request.Flags = REQUEST_FLAG_PACKET_COMMAND;
    Request.DataBuffer = CapacityData as PVOID;
    Request.DataTransferLength = core::mem::size_of::<READ_CAPACITY_DATA>() as ULONG;
    Request.u.Cdb[0] = SCSIOP_READ_CAPACITY;

    AtapIssueCommand(DeviceUnit, &mut Request)
}

unsafe fn AtapAtapiDetectCapacity(
    DeviceUnit: PHW_DEVICE_UNIT,
    TotalSectors: PULONG64,
    SectorSize: PULONG)
{
    #[repr(C)]
    union CapacityDataUnion {
        Cmd10: READ_CAPACITY_DATA,
        Cmd16: READ_CAPACITY16_DATA,
    }
    let mut CapacityData: CapacityDataUnion = core::mem::zeroed();
    let LastLba: ULONG;

    *TotalSectors = 0;
    *SectorSize = 0;

    if AtapAtapiReadCapacity10(DeviceUnit, &mut CapacityData.Cmd10) == 0 {
        return;
    }

    LastLba = RtlUlongByteSwap(CapacityData.Cmd10.LogicalBlockAddress);
    if LastLba == MAXULONG {
        if AtapAtapiReadCapacity16(DeviceUnit, &mut CapacityData.Cmd16) == 0 {
            return;
        }

        *TotalSectors = RtlUlonglongByteSwap(CapacityData.Cmd16.LogicalBlockAddress.QuadPart as u64) + 1;
        *SectorSize = RtlUlongByteSwap(CapacityData.Cmd16.BytesPerBlock);
    } else {
        *TotalSectors = LastLba as ULONG64 + 1;
        *SectorSize = RtlUlongByteSwap(CapacityData.Cmd10.BytesPerBlock);
    }

    if *SectorSize != 0 {
        *SectorSize = 2048;
    }
}

unsafe fn AtapAtapiTestUnitReady(
    DeviceUnit: PHW_DEVICE_UNIT) -> BOOLEAN
{
    let mut Request: ATA_DEVICE_REQUEST = core::mem::zeroed();

    Request.Flags = REQUEST_FLAG_PACKET_COMMAND;
    Request.u.Cdb[0] = SCSIOP_TEST_UNIT_READY;

    AtapIssueCommand(DeviceUnit, &mut Request)
}

unsafe fn AtapAtapiRequestSense(
    DeviceUnit: PHW_DEVICE_UNIT,
    SenseData: PSENSE_DATA) -> BOOLEAN
{
    let mut Request: ATA_DEVICE_REQUEST = core::mem::zeroed();

    AtapAtapiBuildRequestSense(&mut Request, SenseData);
    AtapIssueCommand(DeviceUnit, &mut Request)
}

unsafe fn AtapAtapiReadToc(
    DeviceUnit: PHW_DEVICE_UNIT) -> BOOLEAN
{
    let mut Request: ATA_DEVICE_REQUEST = core::mem::zeroed();
    let mut DummyData: [UCHAR; MAXIMUM_CDROM_SIZE as usize] = [0; MAXIMUM_CDROM_SIZE as usize];

    Request.Flags = REQUEST_FLAG_PACKET_COMMAND;
    Request.DataBuffer = DummyData.as_mut_ptr() as PVOID;
    Request.DataTransferLength = core::mem::size_of_val(&DummyData) as ULONG;
    Request.u.Cdb[0] = SCSIOP_READ_TOC;
    Request.u.Cdb[7] = ((MAXIMUM_CDROM_SIZE >> 8) & 0xFF) as UCHAR;
    Request.u.Cdb[8] = (MAXIMUM_CDROM_SIZE & 0xFF) as UCHAR;
    Request.u.Cdb[9] = (READ_TOC_FORMAT_SESSION << 6) as UCHAR;

    AtapIssueCommand(DeviceUnit, &mut Request)
}

unsafe fn AtapAtapiReadyCheck(
    DeviceUnit: PHW_DEVICE_UNIT) -> BOOLEAN
{
    let mut SenseData: SENSE_DATA = core::mem::zeroed();

    if AtapAtapiTestUnitReady(DeviceUnit) == 0 {
        return 0u8;
    }

    if AtapAtapiRequestSense(DeviceUnit, &mut SenseData) == 0 {
        return 0u8;
    }

    if SenseData.SenseKey() == SCSI_SENSE_NOT_READY {
        if SenseData.AdditionalSenseCode == SCSI_ADSENSE_NO_MEDIA_IN_DEVICE {
            return 0u8;
        }

        if SenseData.AdditionalSenseCode == SCSI_ADSENSE_LUN_NOT_READY {
            match SenseData.AdditionalSenseCodeQualifier {
                SCSI_SENSEQ_BECOMING_READY => {
                    StallExecutionProcessor(2000000);
                    return 0u8;
                }

                SCSI_SENSEQ_INIT_COMMAND_REQUIRED => {
                    AtapAtapiReadToc(DeviceUnit);
                    return 0u8;
                }

                _ => {
                    return 0u8;
                }
            }
        }
    }

    1u8
}

unsafe fn AtapAtapiClearUnitAttention(
    DeviceUnit: PHW_DEVICE_UNIT)
{
    let mut SenseData: SENSE_DATA = core::mem::zeroed();
    let mut i: ULONG;

    i = 0;
    while i < 5 {
        if AtapAtapiRequestSense(DeviceUnit, &mut SenseData) == 0 {
            i += 1;
            continue;
        }

        if (SenseData.SenseKey() != SCSI_SENSE_UNIT_ATTENTION) &&
            (SenseData.AdditionalSenseCode != SCSI_ADSENSE_BUS_RESET)
        {
            break;
        }
        i += 1;
    }
}

unsafe fn AtapAtapiInitDevice(
    DeviceUnit: PHW_DEVICE_UNIT) -> BOOLEAN
{
    let IdentifyPacketData: PIDENTIFY_PACKET_DATA = &mut (*DeviceUnit).u.IdentifyPacketData;
    let mut i: ULONG;

    (*DeviceUnit).CdbSize = (AtaDevCdbSizeInWords(IdentifyPacketData)) as u8 ;

    AtapAtapiClearUnitAttention(DeviceUnit);

    i = 4;
    while i > 0 {
        if AtapAtapiReadyCheck(DeviceUnit) != 0 {
            break;
        }
        i -= 1;
    }
    if i == 0 {
        ERR!("Device not ready\n");
        return 0u8;
    }

    AtapAtapiDetectCapacity(DeviceUnit,
                            &mut (*DeviceUnit).P.TotalSectors,
                            &mut (*DeviceUnit).P.SectorSize);
    if (*DeviceUnit).P.SectorSize == 0 || (*DeviceUnit).P.TotalSectors == 0 {
        TRACE!("No media found\n");
        return 0u8;
    }

    (*DeviceUnit).P.Cylinders = MAXULONG;
    (*DeviceUnit).P.Heads = MAXULONG;
    (*DeviceUnit).P.SectorsPerTrack = MAXULONG;

    (*DeviceUnit).MaximumTransferLength = 0xFFFF;

    1u8
}

unsafe fn AtapAtaSetMultipleMode(DeviceUnit: PHW_DEVICE_UNIT)
{

    let mut Request: ATA_DEVICE_REQUEST = core::mem::zeroed();

    #[cfg(not(ATA_ENABLE_MULTIPLE_MODE))]
    {
        (*DeviceUnit).MultiSectorTransfer = AtaDevCurrentSectorsPerDrq(
            core::ptr::addr_of!((*DeviceUnit).u.IdentifyDeviceData) as *const core::ffi::c_void
        );
        if (*DeviceUnit).MultiSectorTransfer != 0 {
            return;
        }
    }

    (*DeviceUnit).MultiSectorTransfer = AtaDevMaximumSectorsPerDrq(
        core::ptr::addr_of!((*DeviceUnit).u.IdentifyDeviceData) as *const core::ffi::c_void
    );
    if (*DeviceUnit).MultiSectorTransfer == 0 {
        return;
    }

    Request.u.TaskFile.Command = IDE_COMMAND_SET_MULTIPLE;
    Request.u.TaskFile.Data[0].SectorCount = (*DeviceUnit).MultiSectorTransfer;
    if AtapIssueCommand(DeviceUnit, &mut Request) == 0 {
        (*DeviceUnit).MultiSectorTransfer = 0;
    }
}

unsafe fn AtapAtaInitDevice(
    DeviceUnit: PHW_DEVICE_UNIT
) -> BOOLEAN
{

    let IdentifyData: PIDENTIFY_DEVICE_DATA = core::ptr::addr_of_mut!((*DeviceUnit).u.IdentifyDeviceData);
    let TotalSectors: ULONG64;
    let mut Cylinders: USHORT = 0;
    let mut Heads: USHORT = 0;
    let mut SectorsPerTrack: USHORT = 0;

    (*DeviceUnit).MaximumTransferLength = 0xFF;

    if AtaDevIsCurrentGeometryValid(IdentifyData as *const core::ffi::c_void) != 0 {
        AtaDevCurrentChsTranslation(IdentifyData as *const core::ffi::c_void, core::ptr::addr_of_mut!(Cylinders), core::ptr::addr_of_mut!(Heads), core::ptr::addr_of_mut!(SectorsPerTrack));
    } else {
        AtaDevDefaultChsTranslation(IdentifyData as *const core::ffi::c_void, core::ptr::addr_of_mut!(Cylinders), core::ptr::addr_of_mut!(Heads), core::ptr::addr_of_mut!(SectorsPerTrack));
    }

    if AtaDevHasLbaTranslation(IdentifyData as *const core::ffi::c_void) != 0 {
        (*DeviceUnit).P.Flags |= ATA_DEVICE_LBA;

        if AtaDevHas48BitAddressFeature(IdentifyData as *const core::ffi::c_void) != 0 {
            TotalSectors = AtaDevUserAddressableSectors48Bit(IdentifyData as *const core::ffi::c_void);

            (*DeviceUnit).P.Flags |= ATA_DEVICE_LBA48;
            (*DeviceUnit).MaximumTransferLength = 0x10000;
        } else {
            TotalSectors = AtaDevUserAddressableSectors28Bit(IdentifyData as *const core::ffi::c_void);
        }
    } else {
        TotalSectors = (Cylinders as ULONG64) * (Heads as ULONG64) * (SectorsPerTrack as ULONG64);
    }

    if TotalSectors == 0 {
        ERR!("Unknown geometry\n");
        return 0u8;
    }

    (*DeviceUnit).P.TotalSectors = TotalSectors;
    (*DeviceUnit).P.Cylinders = (Cylinders) as u32 ;
    (*DeviceUnit).P.Heads = (Heads) as u32 ;
    (*DeviceUnit).P.SectorsPerTrack = (SectorsPerTrack) as u32 ;
    (*DeviceUnit).P.SectorSize = AtaDevBytesPerLogicalSector(IdentifyData as *const core::ffi::c_void);
    (*DeviceUnit).P.SectorSize = max((*DeviceUnit).P.SectorSize, 512);

    AtapAtaSetMultipleMode(DeviceUnit);

    TRACE!("Multiple sector setting %u\n", ((*DeviceUnit).MultiSectorTransfer) as usize);

    1u8
}

unsafe fn AtapAnalyzeIdentifyData(
    DeviceUnit: PHW_DEVICE_UNIT,
    DeviceClass: ATA_DEVICE_CLASS,
) -> BOOLEAN {
    let Id: PIDENTIFY_DEVICE_DATA = core::ptr::addr_of_mut!((*DeviceUnit).u.IdentifyDeviceData);
    let mut i: ULONG;

    if AtaDevIsIdentifyDataValid(Id) == 0 {
        ERR!("Identify data CRC error\n");
        return 0u8;
    }

    if DeviceClass == DEV_ATAPI {
        (*DeviceUnit).P.Flags |= ATA_DEVICE_ATAPI | ATA_DEVICE_LBA;

        if (*Id).ModelNumber[0] == b'N' &&
           (*Id).ModelNumber[1] == b'E' &&
           (*Id).ModelNumber[2] == b'C' &&
           (*Id).ModelNumber[3] == b' ' {
            (*DeviceUnit).P.Flags |= ATA_DEVICE_IS_NEC_CDR260;
        }
    }

    i = 0;
    while i < (core::mem::size_of_val(&(*Id).SerialNumber) / 2) as ULONG {
        ((*Id).SerialNumber.as_mut_ptr() as PUSHORT).add(i as usize).write(
            RtlUshortByteSwap(((*Id).SerialNumber.as_ptr() as PUSHORT).add(i as usize).read())
        );
        i += 1;
    }

    i = 0;
    while i < (core::mem::size_of_val(&(*Id).FirmwareRevision) / 2) as ULONG {
        ((*Id).FirmwareRevision.as_mut_ptr() as PUSHORT).add(i as usize).write(
            RtlUshortByteSwap(((*Id).FirmwareRevision.as_ptr() as PUSHORT).add(i as usize).read())
        );
        i += 1;
    }

    i = 0;
    while i < (core::mem::size_of_val(&(*Id).ModelNumber) / 2) as ULONG {
        ((*Id).ModelNumber.as_mut_ptr() as PUSHORT).add(i as usize).write(
            RtlUshortByteSwap(((*Id).ModelNumber.as_ptr() as PUSHORT).add(i as usize).read())
        );
        i += 1;
    }

    TRACE!("MN  '%.*s'\n", (core::mem::size_of_val(&(*Id).ModelNumber) as i32) as usize, ((*Id).ModelNumber.as_ptr()) as usize);
    TRACE!("FR  '%.*s'\n", (core::mem::size_of_val(&(*Id).FirmwareRevision) as i32) as usize, ((*Id).FirmwareRevision.as_ptr()) as usize);
    TRACE!("S/N '%.*s'\n", (core::mem::size_of_val(&(*Id).SerialNumber) as i32) as usize, ((*Id).SerialNumber.as_ptr()) as usize);

    1u8
}

unsafe fn AtapInitDevice(
    DeviceUnit: PHW_DEVICE_UNIT,
    DeviceClass: ATA_DEVICE_CLASS,
) -> BOOLEAN {
    if AtapAnalyzeIdentifyData(DeviceUnit, DeviceClass) == 0 {
        return 0u8;
    }

    if DeviceClass == DEV_ATAPI {
        return AtapAtapiInitDevice(DeviceUnit);
    } else {
        return AtapAtaInitDevice(DeviceUnit);
    }
}

unsafe fn AtapIdentifyChannel(
    ChannelNumber: ULONG,
    Registers: PIDE_REGISTERS,
) -> BOOLEAN {
    let IoBase: IDE_REG = AtapChannelBaseArray[ChannelNumber as usize];
    let Spare: ULONG;

    Spare = 1;
    (*Registers).ctl.Control = IoBase + 0x206;
    (*Registers).Data = IoBase + ((0 * Spare) as u16);
    (*Registers).feat_err.Error = IoBase + ((1 * Spare) as u16);
    (*Registers).sc_ir.SectorCount = IoBase + ((2 * Spare) as u16);
    (*Registers).LbaLow = IoBase + ((3 * Spare) as u16);
    (*Registers).lm.LbaMid = IoBase + ((4 * Spare) as u16);
    (*Registers).lh.LbaHigh = IoBase + ((5 * Spare) as u16);
    (*Registers).Device = IoBase + ((6 * Spare) as u16);
    (*Registers).__bindgen_anon_1.Status = IoBase + ((7 * Spare) as u16);

    1u8
}

unsafe fn AtaReadLogicalSectors(
    DeviceUnit: PDEVICE_UNIT,
    SectorNumber: ULONG64,
    SectorCount: ULONG,
    Buffer: PVOID) -> BOOLEAN
{

    let Unit: PHW_DEVICE_UNIT = DeviceUnit as PHW_DEVICE_UNIT;
    let mut Request: ATA_DEVICE_REQUEST = core::mem::zeroed();

    let mut SectorNumber = SectorNumber;
    let mut SectorCount = SectorCount;
    let mut Buffer = Buffer;

    while SectorCount > 0
    {
        let BlockCount: ULONG;

        BlockCount = min(SectorCount, (*Unit).MaximumTransferLength);

        Request.DataBuffer = Buffer;
        Request.DataTransferLength = BlockCount * (*Unit).P.SectorSize;

        if ((*Unit).P.Flags & ATA_DEVICE_ATAPI) != 0 {
            AtapBuildReadPacketCommand(core::ptr::addr_of_mut!(Request), SectorNumber, BlockCount);
        } else {
            AtapBuildReadTaskFile(Unit, core::ptr::addr_of_mut!(Request), SectorNumber, BlockCount);
        }

        if AtapIssueCommand(Unit, core::ptr::addr_of_mut!(Request)) == 0 {
            return 0u8;
        }

        SectorNumber += BlockCount as ULONG64;
        SectorCount -= BlockCount;

        Buffer = (Buffer as ULONG_PTR + (*Unit).P.SectorSize as ULONG_PTR) as PVOID;
    }

    1u8
}

unsafe fn AtaGetDevice(UnitNumber: UCHAR) -> PDEVICE_UNIT {
    if (UnitNumber as usize) < (core::mem::size_of_val(&AtapUnits) / core::mem::size_of::<ATA_UNIT>()) {
        return core::ptr::addr_of_mut!(AtapUnits[UnitNumber as usize]) as PDEVICE_UNIT;
    }
    
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn AtaInit(
    DetectedCount: PUCHAR,
) -> BOOLEAN
{
    let mut ChannelNumber: ULONG;

    *DetectedCount = 0;

    ChannelNumber = 0;
    while ChannelNumber < ((CHANNEL_MAX_CHANNELS) as u32) {
        let mut DeviceNumber: UCHAR;
        let mut Registers: IDE_REGISTERS = core::mem::zeroed();

        if AtapIdentifyChannel(ChannelNumber, &mut Registers) == 0 {
            ChannelNumber += 1;
            continue;
        }

        DeviceNumber = 0;
        while DeviceNumber < ((CHANNEL_MAX_DEVICES) as u8) {
            let DeviceUnit: PHW_DEVICE_UNIT;
            let DeviceClass: ATA_DEVICE_CLASS;

            DeviceUnit = FrLdrTempAlloc(core::mem::size_of::<HW_DEVICE_UNIT>(), TAG_ATA_DEVICE) as PHW_DEVICE_UNIT;
            if DeviceUnit.is_null() {
                ERR!("Failed to allocate device unit!\n");
                DeviceNumber += 1;
                continue;
            }
            core::ptr::write_bytes(DeviceUnit, 0, 1);

            core::ptr::copy_nonoverlapping(
                &Registers as *const IDE_REGISTERS as *const u8,
                core::ptr::addr_of_mut!((*DeviceUnit).Registers) as *mut u8,
                core::mem::size_of::<IDE_REGISTERS>()
            );
            (*DeviceUnit).DeviceNumber = DeviceNumber;
            (*DeviceUnit).P.SectorSize = 512;
            (*DeviceUnit).DeviceSelect = (DEV_SLAVE(DeviceNumber) << 4) | IDE_DRIVE_SELECT;

            DeviceClass = AtapIdentifyDevice(DeviceUnit);
            if DeviceClass == DEV_NONE {
                FrLdrTempFree(DeviceUnit as PVOID, TAG_ATA_DEVICE);
                DeviceNumber += 1;
                continue;
            }

            TRACE!("Found %lu device at %X:%u\n", (DeviceClass) as usize, (Registers.Data) as usize, (DeviceNumber) as usize);

            if AtapInitDevice(DeviceUnit, DeviceClass) == 0 {
                FrLdrTempFree(DeviceUnit as PVOID, TAG_ATA_DEVICE);
                DeviceNumber += 1;
                continue;
            }

            TRACE!("Total sectors %I64u of size %lu, CHS %lu:%lu:%lu, %lx\n", ((*DeviceUnit).P.TotalSectors) as usize, ((*DeviceUnit).P.SectorSize) as usize, ((*DeviceUnit).P.Cylinders) as usize, ((*DeviceUnit).P.Heads) as usize, ((*DeviceUnit).P.SectorsPerTrack) as usize, ((*DeviceUnit).P.Flags) as usize);

            AtapUnits[*DetectedCount as usize] = DeviceUnit;
            *DetectedCount += 1;

            DeviceNumber += 1;
        }

        ChannelNumber += 1;
    }

    if *DetectedCount > 0 { 1u8 } else { 0u8 }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     ATA/ATAPI programmed I/O driver.
//  * COPYRIGHT:   Copyright 2019-2026 Dmitry Borisov <di.sean@protonmail.com>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <freeldr.h>
//
// /* DDK */
// #include <ata.h>
// #include <scsi.h>
//
// #include <hwide.h>
// #include "hwidep.h"
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(DISK);
//
// /* GLOBALS ********************************************************************/
//
// /** IDE Channels base - Primary, Secondary, Tertiary, Quaternary */
// static const IDE_REG AtapChannelBaseArray[] =
// {
//     0x1F0, 0x170, 0x1E8, 0x168
// };
// #define CHANNEL_MAX_CHANNELS    RTL_NUMBER_OF(AtapChannelBaseArray)
//
// static PHW_DEVICE_UNIT AtapUnits[CHANNEL_MAX_CHANNELS * CHANNEL_MAX_DEVICES];
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// static
// VOID
// AtapSelectDevice(
//     _In_ PIDE_REGISTERS Registers,
//     _In_range_(0, 3) UCHAR DeviceNumber)
// {
//
//     ATA_WRITE(Registers->Device, (DEV_SLAVE(DeviceNumber) << 4) | IDE_DRIVE_SELECT);
//     ATA_IO_WAIT();
// }
//
// static
// UCHAR
// AtapWait(
//     _In_ PIDE_REGISTERS Registers,
//     _In_range_(>, 0) ULONG Timeout,
//     _In_ UCHAR Mask,
//     _In_ UCHAR Value)
// {
//     UCHAR IdeStatus;
//     ULONG i;
//
//     ASSERT(Timeout != 0);
//
//     for (i = 0; i < Timeout; ++i)
//     {
//         IdeStatus = ATA_READ(Registers->Status);
//         if ((IdeStatus & Mask) == Value)
//             break;
//
//         if (IdeStatus == 0xFF)
//             break;
//
//         StallExecutionProcessor(10);
//     }
//
//     return IdeStatus;
// }
//
// static
// VOID
// AtapSendCdb(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ PATA_DEVICE_REQUEST Request)
// {
// #if defined(ATA_SUPPORT_32_BIT_IO)
//     ATA_WRITE_BLOCK_32(DeviceUnit->Registers.Data,
//                        Request->Cdb,
//                        DeviceUnit->CdbSize / sizeof(USHORT));
// #else
//     ATA_WRITE_BLOCK_16(DeviceUnit->Registers.Data,
//                        Request->Cdb,
//                        DeviceUnit->CdbSize);
// #endif
// }
//
// static
// VOID
// AtapPioDataIn(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ ULONG ByteCount)
// {
//     ByteCount = min(ByteCount, DeviceUnit->BytesToTransfer);
//
// #if defined(ATA_SUPPORT_32_BIT_IO)
//     if (!(ByteCount & (sizeof(ULONG) - 1)))
//     {
//         ATA_READ_BLOCK_32(DeviceUnit->Registers.Data,
//                           (PULONG)DeviceUnit->DataBuffer,
//                           ByteCount / sizeof(ULONG));
//     }
//     else
// #endif
//     {
//         ATA_READ_BLOCK_16(DeviceUnit->Registers.Data,
//                           (PUSHORT)DeviceUnit->DataBuffer,
//                           ByteCount / sizeof(USHORT));
//     }
//
//     DeviceUnit->DataBuffer += ByteCount;
//     DeviceUnit->BytesToTransfer -= ByteCount;
// }
//
// static
// BOOLEAN
// AtapWaitForRegisterAccess(
//     _In_ PIDE_REGISTERS Registers,
//     _In_range_(0, 3) UCHAR DeviceNumber)
// {
//     ULONG i;
//
//     for (i = 0; i < ATA_TIME_RESET_SELECT; ++i)
//     {
//         /* Select the device again */
//         AtapSelectDevice(Registers, DeviceNumber);
//
//         /* Check whether the selection was successful */
//         ATA_WRITE(Registers->ByteCountLow, 0xAA);
//         ATA_WRITE(Registers->ByteCountLow, 0x55);
//         ATA_WRITE(Registers->ByteCountLow, 0xAA);
//         if (ATA_READ(Registers->ByteCountLow) == 0xAA)
//             return TRUE;
//
//         StallExecutionProcessor(10);
//     }
//
//     return FALSE;
// }
//
// static
// VOID
// AtapSoftwareReset(
//     _In_ PIDE_REGISTERS Registers)
// {
//     ATA_WRITE(Registers->Control, IDE_DC_RESET_CONTROLLER | IDE_DC_ALWAYS);
//     StallExecutionProcessor(20);
//     ATA_WRITE(Registers->Control, IDE_DC_DISABLE_INTERRUPTS | IDE_DC_ALWAYS);
//     StallExecutionProcessor(20);
// }
//
// static
// VOID
// AtapDrainDeviceBuffer(
//     _In_ PIDE_REGISTERS Registers)
// {
//     UCHAR IdeStatus;
//     ULONG i;
//
//     /* Try to clear the DRQ indication */
//     for (i = 0; i < 0x10000 / sizeof(USHORT); ++i)
//     {
//         IdeStatus = ATA_READ(Registers->Status);
//         if (!(IdeStatus & IDE_STATUS_DRQ))
//             break;
//
//         READ_PORT_USHORT((PUSHORT)(ULONG_PTR)Registers->Data);
//     }
// }
//
// static
// BOOLEAN
// AtapPerformSoftwareReset(
//     _In_ PHW_DEVICE_UNIT DeviceUnit)
// {
//     PIDE_REGISTERS Registers = &DeviceUnit->Registers;
//     UCHAR IdeStatus;
//
//     ERR("Reset device at %X:%u\n", (Registers->Data) as usize, (DeviceUnit->DeviceNumber) as usize);
//
//     AtapDrainDeviceBuffer(Registers);
//
//     /* Perform a software reset */
//     AtapSoftwareReset(Registers);
//
//     /* The reset will cause the master device to be selected */
//     if (DEV_SLAVE(DeviceUnit->DeviceNumber))
//     {
//         if (!AtapWaitForRegisterAccess(Registers, DeviceUnit->DeviceNumber))
//             return FALSE;
//     }
//
//     /* Now wait for busy to clear */
//     IdeStatus = AtapWait(Registers, ATA_TIME_BUSY_RESET, IDE_STATUS_BUSY, 0);
//     if (IdeStatus & IDE_STATUS_BUSY)
//         return FALSE;
//
//     return TRUE;
// }
//
// static
// UCHAR
// AtapProcessAtapiRequest(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ PATA_DEVICE_REQUEST Request,
//     _In_ UCHAR IdeStatus)
// {
//     UCHAR InterruptReason;
//
//     InterruptReason = ATA_READ(DeviceUnit->Registers.sc_ir.InterruptReason);
//     InterruptReason &= ATAPI_INT_REASON_MASK;
//     InterruptReason |= IdeStatus & IDE_STATUS_DRQ;
//
//     /*
//      * The NEC CDR-C251 drive is not fully ATAPI-compliant
//      * and clears BSY before raising the DRQ bit and updating the interrupt reason register.
//      * As a workaround, we will wait a bit more in the case the valid IR is not quite there yet.
//      */
//     if ((InterruptReason == ATAPI_INT_REASON_COD) || (InterruptReason == ATAPI_INT_REASON_IO))
//     {
//         IdeStatus = AtapWait(&DeviceUnit->Registers,
//                              ATA_TIME_DRQ_ASSERT,
//                              IDE_STATUS_DRQ,
//                              IDE_STATUS_DRQ);
//
//         InterruptReason = ATA_READ(DeviceUnit->Registers.sc_ir.InterruptReason);
//         InterruptReason &= ATAPI_INT_REASON_MASK;
//         InterruptReason |= IdeStatus & IDE_STATUS_DRQ;
//     }
//
//     switch (InterruptReason)
//     {
//         case ATAPI_INT_REASON_AWAIT_CDB:
//         {
//             if (!(Request->Flags & REQUEST_FLAG_AWAIT_CDB))
//                 return ATA_STATUS_RESET;
//
//             Request->Flags &= ~REQUEST_FLAG_AWAIT_CDB;
//
//             AtapSendCdb(DeviceUnit, Request);
//             return ATA_STATUS_PENDING;
//         }
//
//         case ATAPI_INT_REASON_DATA_IN:
//         {
//             ULONG ByteCount;
//
//             if (!Request->DataBuffer || (Request->Flags & REQUEST_FLAG_AWAIT_CDB))
//                 return ATA_STATUS_RESET;
//
//             ByteCount = ATA_READ(DeviceUnit->Registers.lm.ByteCountLow);
//             ByteCount |= ATA_READ(DeviceUnit->Registers.lh.ByteCountHigh) << 8;
//
//             AtapPioDataIn(DeviceUnit, ByteCount);
//             return ATA_STATUS_PENDING;
//         }
//
//         case ATAPI_INT_REASON_STATUS_NEC:
//         {
//             /* The NEC CDR-260 drive always clears CoD and IO on command completion */
//             if (!(DeviceUnit->P.Flags & ATA_DEVICE_IS_NEC_CDR260))
//                 return ATA_STATUS_RESET;
//
//             __fallthrough;
//         }
//         case ATAPI_INT_REASON_STATUS:
//         {
//             if (IdeStatus & (IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT))
//                 return ATA_STATUS_ERROR;
//
//             break;
//         }
//
//         default:
//             return ATA_STATUS_RESET;
//     }
//
//     return ATA_STATUS_SUCCESS;
// }
//
// static
// UCHAR
// AtapProcessAtaRequest(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ PATA_DEVICE_REQUEST Request,
//     _In_ UCHAR IdeStatus)
// {
//     /* Check for errors */
//     if (IdeStatus & (IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT))
//     {
//         if (IdeStatus & IDE_STATUS_DRQ)
//             return ATA_STATUS_RESET;
//         else
//             return ATA_STATUS_ERROR;
//     }
//
//     /* Read command */
//     if (Request->DataBuffer)
//     {
//         /*
//          * The NEC CDR-C251 drive clears BSY before raising the DRQ bit
//          * while processing the ATAPI identify command.
//          * Give the device a chance to assert that bit.
//          */
//         if (!(IdeStatus & IDE_STATUS_DRQ))
//         {
//             IdeStatus = AtapWait(&DeviceUnit->Registers,
//                                  ATA_TIME_DRQ_ASSERT,
//                                  IDE_STATUS_DRQ,
//                                  IDE_STATUS_DRQ);
//             if (!(IdeStatus & IDE_STATUS_DRQ))
//                 return ATA_STATUS_RESET;
//         }
//
//         /* Read the next data block */
//         AtapPioDataIn(DeviceUnit, DeviceUnit->DrqByteCount);
//
//         if (DeviceUnit->BytesToTransfer != 0)
//             return ATA_STATUS_PENDING;
//
//         /* All data has been transferred, wait for DRQ to clear */
//         IdeStatus = AtapWait(&DeviceUnit->Registers,
//                              ATA_TIME_DRQ_CLEAR,
//                              IDE_STATUS_BUSY | IDE_STATUS_DRQ,
//                              0);
//         if (IdeStatus & (IDE_STATUS_BUSY | IDE_STATUS_DRQ))
//             return ATA_STATUS_RESET;
//
//         if (IdeStatus & (IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT))
//             return ATA_STATUS_ERROR;
//     }
//
//     /* Status phase or non-data ATA command */
//     return ATA_STATUS_SUCCESS;
// }
//
// static
// UCHAR
// AtapProcessRequest(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ PATA_DEVICE_REQUEST Request,
//     _In_ UCHAR IdeStatus)
// {
//     UCHAR AtaStatus;
//
//     if (!(Request->Flags & REQUEST_FLAG_PACKET_COMMAND))
//         return AtapProcessAtaRequest(DeviceUnit, Request, IdeStatus);
//
//     AtaStatus = AtapProcessAtapiRequest(DeviceUnit, Request, IdeStatus);
//
//     /*
//      * In polled mode (interrupts disabled), the NEC CDR-260 drive behaves quite differently
//      * that other devices. This drive does not raise BSY immediately
//      * in response to a CDB or buffer write. The status and interrupt reason registers
//      * remain invalid and unchanged for the time of media access.
//      * As a workaround, we will wait for the phase change.
//      */
//     if ((DeviceUnit->P.Flags & ATA_DEVICE_IS_NEC_CDR260) && (AtaStatus == ATA_STATUS_PENDING))
//     {
//         UCHAR OldInterruptReason, NewInterruptReason;
//         ULONG i;
//
//         OldInterruptReason = ATA_READ(DeviceUnit->Registers.sc_ir.InterruptReason);
//
//         /* Set a long timeout on purpose */
//         for (i = ATA_TIME_BUSY_POLL; i > 0; i--)
//         {
//             NewInterruptReason = ATA_READ(DeviceUnit->Registers.sc_ir.InterruptReason);
//             if (NewInterruptReason != OldInterruptReason)
//                 break;
//
//             StallExecutionProcessor(10);
//         }
//         if (i == 0)
//         {
//             AtaStatus = ATA_STATUS_RESET;
//         }
//     }
//
//     return AtaStatus;
// }
//
// static
// VOID
// AtapIssuePacketCommand(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ PATA_DEVICE_REQUEST Request)
// {
//     USHORT ByteCount;
//
//     Request->Flags |= REQUEST_FLAG_AWAIT_CDB;
//
//     /*
//      * If a larger transfer is attempted, the 16-bit ByteCount register might overflow.
//      * In this case we round down the length to the closest multiple of 2.
//      */
//     ByteCount = min(Request->DataTransferLength, 0xFFFE);
//
//     /* Prepare to transfer a device command */
//     ATA_WRITE(DeviceUnit->Registers.lm.ByteCountLow, (UCHAR)(ByteCount >> 0));
//     ATA_WRITE(DeviceUnit->Registers.lh.ByteCountHigh, (UCHAR)(ByteCount >> 8));
//     ATA_WRITE(DeviceUnit->Registers.feat_err.Features, IDE_FEATURE_PIO);
//     ATA_WRITE(DeviceUnit->Registers.__bindgen_anon_1.Command, IDE_COMMAND_ATAPI_PACKET);
// }
//
// static
// VOID
// AtapLoadTaskFile(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ PATA_DEVICE_REQUEST Request)
// {
//     PATA_TASKFILE TaskFile = &Request->TaskFile;
//     ULONG i, BlockCount;
//
//     /* Store the extra information in the second byte of FIFO for 48-bit commands */
//     i = (Request->Flags & REQUEST_FLAG_LBA48) ? 2 : 1;
//
//     while (i--)
//     {
//         ATA_WRITE(DeviceUnit->Registers.feat_err.Features, TaskFile->Data[i].Feature);
//         ATA_WRITE(DeviceUnit->Registers.sc_ir.SectorCount, TaskFile->Data[i].SectorCount);
//         ATA_WRITE(DeviceUnit->Registers.LbaLow, TaskFile->Data[i].LowLba);
//         ATA_WRITE(DeviceUnit->Registers.lm.LbaMid, TaskFile->Data[i].MidLba);
//         ATA_WRITE(DeviceUnit->Registers.lh.LbaHigh, TaskFile->Data[i].HighLba);
//     }
//     if (Request->Flags & REQUEST_FLAG_SET_DEVICE_REGISTER)
//     {
//         ATA_WRITE(DeviceUnit->Registers.Device, TaskFile->DriveSelect);
//     }
//     ATA_WRITE(DeviceUnit->Registers.__bindgen_anon_1.Command, TaskFile->Command);
//
//     /* Set the byte count per DRQ data block */
//     if (Request->Flags & REQUEST_FLAG_READ_WRITE_MULTIPLE)
//         BlockCount = DeviceUnit->MultiSectorTransfer;
//     else
//         BlockCount = 1;
//     DeviceUnit->DrqByteCount = BlockCount * DeviceUnit->P.SectorSize;
// }
//
// static
// UCHAR
// AtapSendCommand(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ PATA_DEVICE_REQUEST Request)
// {
//     UCHAR IdeStatus, AtaStatus;
//
//     DeviceUnit->BytesToTransfer = Request->DataTransferLength;
//     DeviceUnit->DataBuffer = Request->DataBuffer;
//
//     /* Select the device */
//     AtapSelectDevice(&DeviceUnit->Registers, DeviceUnit->DeviceNumber);
//     IdeStatus = AtapWait(&DeviceUnit->Registers, ATA_TIME_BUSY_SELECT, IDE_STATUS_BUSY, 0);
//     if (IdeStatus & IDE_STATUS_BUSY)
//         return ATA_STATUS_RETRY;
//
//     /* Always disable interrupts, otherwise it may lead to problems in underlying BIOS firmware */
//     ATA_WRITE(DeviceUnit->Registers.ctl.Control, IDE_DC_DISABLE_INTERRUPTS | IDE_DC_ALWAYS);
//
//     if (Request->Flags & REQUEST_FLAG_PACKET_COMMAND)
//         AtapIssuePacketCommand(DeviceUnit, Request);
//     else
//         AtapLoadTaskFile(DeviceUnit, Request);
//
//     while (TRUE)
//     {
//         ATA_IO_WAIT();
//
//         IdeStatus = AtapWait(&DeviceUnit->Registers, ATA_TIME_BUSY_POLL, IDE_STATUS_BUSY, 0);
//         if (IdeStatus & IDE_STATUS_BUSY)
//             return ATA_STATUS_RESET;
//
//         AtaStatus = AtapProcessRequest(DeviceUnit, Request, IdeStatus);
//         if (AtaStatus != ATA_STATUS_PENDING)
//             break;
//     }
//
//     return AtaStatus;
// }
//
// static
// VOID
// AtapAtapiBuildRequestSense(
//     _In_ PATA_DEVICE_REQUEST Request,
//     _Out_ PSENSE_DATA SenseData)
// {
//     Request->Flags = REQUEST_FLAG_PACKET_COMMAND;
//     Request->DataBuffer = SenseData;
//     Request->DataTransferLength = sizeof(*SenseData);
//     Request->Cdb[0] = SCSIOP_REQUEST_SENSE;
//     Request->Cdb[4] = (UCHAR)Request->DataTransferLength;
// }
//
// static
// BOOLEAN
// AtapAtapiHandleError(
//     _In_ PHW_DEVICE_UNIT DeviceUnit)
// {
//     ATA_DEVICE_REQUEST Request = { 0 };
//     SENSE_DATA SenseData;
//
//     AtapAtapiBuildRequestSense(&Request, &SenseData);
//     if (AtapSendCommand(DeviceUnit, &Request) != ATA_STATUS_SUCCESS)
//         return FALSE;
//
//     ERR("SK %02X, ASC %02X, ASCQ %02X\n", (//         SenseData.SenseKey()) as usize, (//         SenseData.AdditionalSenseCode) as usize, (//         SenseData.AdditionalSenseCodeQualifier) as usize);
//
//     return TRUE;
// }
//
// static
// BOOLEAN
// AtapIssueCommand(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ PATA_DEVICE_REQUEST Request)
// {
//     ULONG RetryCount;
//
//     for (RetryCount = 0; RetryCount < 3; ++RetryCount)
//     {
//         UCHAR AtaStatus;
//
//         AtaStatus = AtapSendCommand(DeviceUnit, Request);
//
//         if ((AtaStatus != ATA_STATUS_SUCCESS) &&
//             !(Request->Flags & REQUEST_FLAG_IDENTIFY_COMMAND))
//         {
//             ERR("ATA%s command %02X failed %u %02X:%02X at %X:%u\n", (//                 (Request->Flags & REQUEST_FLAG_PACKET_COMMAND) ? "PI" : "") as usize, (//                 (Request->Flags & REQUEST_FLAG_PACKET_COMMAND) ?
//                 Request->Cdb[0] : Request->TaskFile.Command) as usize, (//                 AtaStatus) as usize, (//                 ATA_READ(DeviceUnit->Registers.__bindgen_anon_1.Status)) as usize, (//                 ATA_READ(DeviceUnit->Registers.feat_err.Error)) as usize, (//                 DeviceUnit->Registers.Data) as usize, (//                 DeviceUnit->DeviceNumber) as usize);
//         }
//
//         switch (AtaStatus)
//         {
//             case ATA_STATUS_SUCCESS:
//                 return TRUE;
//
//             case ATA_STATUS_RETRY:
//                 break;
//
//             case ATA_STATUS_RESET:
//             {
//                 if (Request->Flags & REQUEST_FLAG_IDENTIFY_COMMAND)
//                 {
//                     /*
//                      * Some controllers indicate status 0x00
//                      * when the selected device does not exist,
//                      * no point in going further.
//                      */
//                     if (ATA_READ(DeviceUnit->Registers.__bindgen_anon_1.Status) == 0)
//                         return FALSE;
//                 }
//
//                 if (!AtapPerformSoftwareReset(DeviceUnit))
//                     return FALSE;
//
//                 break;
//             }
//
//             default:
//             {
//                 if (Request->Flags & REQUEST_FLAG_PACKET_COMMAND)
//                 {
//                     if (!AtapAtapiHandleError(DeviceUnit))
//                         return FALSE;
//                 }
//
//                 /* Only retry failed read commands */
//                 if (!(Request->Flags & REQUEST_FLAG_READ_COMMAND))
//                     return FALSE;
//
//                 break;
//             }
//         }
//     }
//
//     return FALSE;
// }
//
// static
// UCHAR
// AtapGetReadCommand(
//     _In_ PATA_DEVICE_REQUEST Request)
// {
//     static const UCHAR AtapReadCommandMap[2][2] =
//     {
//         /* Read                      Read EXT */
//         { IDE_COMMAND_READ,          IDE_COMMAND_READ_EXT          }, // PIO single
//         { IDE_COMMAND_READ_MULTIPLE, IDE_COMMAND_READ_MULTIPLE_EXT }, // PIO multiple
//     };
//
//     return AtapReadCommandMap[(Request->Flags & REQUEST_FLAG_READ_WRITE_MULTIPLE) ? 1 : 0]
//                              [(Request->Flags & REQUEST_FLAG_LBA48) ? 1 : 0];
// }
//
// static
// VOID
// AtapBuildReadTaskFile(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ PATA_DEVICE_REQUEST Request,
//     _In_ ULONG64 Lba,
//     _In_ ULONG SectorCount)
// {
//     PATA_TASKFILE TaskFile = &Request->TaskFile;
//     UCHAR DriveSelect;
//
//     Request->Flags = REQUEST_FLAG_READ_COMMAND | REQUEST_FLAG_SET_DEVICE_REGISTER;
//
//     if (DeviceUnit->MultiSectorTransfer != 0)
//     {
//         Request->Flags |= REQUEST_FLAG_READ_WRITE_MULTIPLE;
//     }
//
//     if (DeviceUnit->P.Flags & ATA_DEVICE_LBA)
//     {
//         DriveSelect = IDE_LBA_MODE;
//
//         TaskFile->Data[0].SectorCount = (UCHAR)SectorCount;
//         TaskFile->Data[0].LowLba = (UCHAR)Lba;              // LBA bits 0-7
//         TaskFile->Data[0].MidLba = (UCHAR)(Lba >> 8);       // LBA bits 8-15
//         TaskFile->Data[0].HighLba = (UCHAR)(Lba >> 16);     // LBA bits 16-23
//
//         if ((DeviceUnit->P.Flags & ATA_DEVICE_LBA48) && (AtaCommandUseLba48(Lba, SectorCount)))
//         {
//             ASSERT((Lba + SectorCount) <= ATA_MAX_LBA_48);
//
//             /* 48-bit command */
//             TaskFile->Data[1].SectorCount = (UCHAR)(SectorCount >> 8);
//             TaskFile->Data[1].LowLba = (UCHAR)(Lba >> 24);  // LBA bits 24-31
//             TaskFile->Data[1].MidLba = (UCHAR)(Lba >> 32);  // LBA bits 32-39
//             TaskFile->Data[1].HighLba = (UCHAR)(Lba >> 40); // LBA bits 40-47
//
//             Request->Flags |= REQUEST_FLAG_LBA48;
//         }
//         else
//         {
//             ASSERT((Lba + SectorCount) <= ATA_MAX_LBA_28);
//
//             /* 28-bit command */
//             DriveSelect |= ((Lba >> 24) & 0x0F);      // LBA bits 24-27
//         }
//     }
//     else
//     {
//         ULONG ChsTemp, Cylinder, Head, Sector;
//
//         ChsTemp = (ULONG)Lba / DeviceUnit->P.SectorsPerTrack;
//
//         /* Legacy CHS translation */
//         Cylinder = ChsTemp / DeviceUnit->P.Heads;
//         Head = ChsTemp % DeviceUnit->P.Heads;
//         Sector = ((ULONG)Lba % DeviceUnit->P.SectorsPerTrack) + 1;
//
//         ASSERT(Cylinder <= 65535 && Head <= 15 && Sector <= 255);
//
//         TaskFile->Data[0].SectorCount = (UCHAR)SectorCount;
//         TaskFile->Data[0].LowLba = (UCHAR)Sector;
//         TaskFile->Data[0].MidLba = (UCHAR)Cylinder;
//         TaskFile->Data[0].HighLba = (UCHAR)(Cylinder >> 8);
//
//         DriveSelect = Head;
//     }
//     TaskFile->DriveSelect = DeviceUnit->DeviceSelect | DriveSelect;
//     TaskFile->Command = AtapGetReadCommand(Request);
// }
//
// static
// VOID
// AtapBuildReadPacketCommand(
//     _In_ PATA_DEVICE_REQUEST Request,
//     _In_ ULONG64 Lba,
//     _In_ ULONG SectorCount)
// {
//     Request->Flags = REQUEST_FLAG_READ_COMMAND | REQUEST_FLAG_PACKET_COMMAND;
//
//     RtlZeroMemory(Request->Cdb, sizeof(Request->Cdb));
//
//     if (Lba > MAXULONG)
//     {
//         /* READ (16) */
//         Request->Cdb[0] = SCSIOP_READ16;
//         Request->Cdb[2] = (UCHAR)(Lba >> 56);
//         Request->Cdb[3] = (UCHAR)(Lba >> 48);
//         Request->Cdb[4] = (UCHAR)(Lba >> 40);
//         Request->Cdb[5] = (UCHAR)(Lba >> 32);
//         Request->Cdb[6] = (UCHAR)(Lba >> 24);
//         Request->Cdb[7] = (UCHAR)(Lba >> 16);
//         Request->Cdb[8] = (UCHAR)(Lba >> 8);
//         Request->Cdb[9] = (UCHAR)(Lba >> 0);
//         Request->Cdb[10] = (UCHAR)(SectorCount >> 24);
//         Request->Cdb[11] = (UCHAR)(SectorCount >> 16);
//         Request->Cdb[12] = (UCHAR)(SectorCount >> 8);
//         Request->Cdb[13] = (UCHAR)(SectorCount >> 0);
//     }
//     else
//     {
//         /* READ (10) */
//         Request->Cdb[0] = SCSIOP_READ;
//         Request->Cdb[2] = (UCHAR)(Lba >> 24);
//         Request->Cdb[3] = (UCHAR)(Lba >> 16);
//         Request->Cdb[4] = (UCHAR)(Lba >> 8);
//         Request->Cdb[5] = (UCHAR)(Lba >> 0);
//         Request->Cdb[7] = (UCHAR)(SectorCount >> 8);
//         Request->Cdb[8] = (UCHAR)(SectorCount >> 0);
//     }
// }
//
// static
// BOOLEAN
// AtapIsDevicePresent(
//     _In_ PHW_DEVICE_UNIT DeviceUnit)
// {
//     PIDE_REGISTERS Registers = &DeviceUnit->Registers;
//     UCHAR IdeStatus;
//
//     AtapSelectDevice(Registers, DeviceUnit->DeviceNumber);
//
//     IdeStatus = ATA_READ(Registers->Status);
//     if (IdeStatus == 0xFF || IdeStatus == 0x7F)
//         return FALSE;
//
//     ATA_WRITE(Registers->ByteCountLow, 0x55);
//     ATA_WRITE(Registers->ByteCountLow, 0xAA);
//     ATA_WRITE(Registers->ByteCountLow, 0x55);
//     if (ATA_READ(Registers->ByteCountLow) != 0x55)
//         return FALSE;
//     ATA_WRITE(Registers->ByteCountHigh, 0xAA);
//     ATA_WRITE(Registers->ByteCountHigh, 0x55);
//     ATA_WRITE(Registers->ByteCountHigh, 0xAA);
//     if (ATA_READ(Registers->ByteCountHigh) != 0xAA)
//         return FALSE;
//
//     IdeStatus = AtapWait(&DeviceUnit->Registers, ATA_TIME_BUSY_ENUM, IDE_STATUS_BUSY, 0);
//     if (IdeStatus & (IDE_STATUS_BUSY | IDE_STATUS_DRQ))
//     {
//         ERR("Device %X:%u is busy %02x\n", (Registers->Data) as usize, (DeviceUnit->DeviceNumber) as usize, (IdeStatus) as usize);
//
//         /* Bring the device into a known state */
//         if (!AtapPerformSoftwareReset(DeviceUnit))
//             return FALSE;
//     }
//
//     return TRUE;
// }
//
// static
// BOOLEAN
// AtapReadIdentifyData(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ UCHAR Command)
// {
//     ATA_DEVICE_REQUEST Request = { 0 };
//     PIDENTIFY_DEVICE_DATA Id = &DeviceUnit->IdentifyDeviceData;
//
//     /* Send the identify command */
//     Request.Flags = REQUEST_FLAG_IDENTIFY_COMMAND;
//     Request.DataBuffer = Id;
//     Request.DataTransferLength = sizeof(*Id);
//     Request.u.TaskFile.Command = Command;
//
//     return AtapIssueCommand(DeviceUnit, &Request);
// }
//
// static
// ATA_DEVICE_CLASS
// AtapIdentifyDevice(
//     _In_ PHW_DEVICE_UNIT DeviceUnit)
// {
//     if (!AtapIsDevicePresent(DeviceUnit))
//         return DEV_NONE;
//
//     /*
//      * We don't check the device signature here,
//      * because the NEC CDR-260 drive reports an ATA signature.
//      */
//
//     /* Check for ATA */
//     if (AtapReadIdentifyData(DeviceUnit, IDE_COMMAND_IDENTIFY))
//         return DEV_ATA;
//
//     /* Check for ATAPI */
//     if (AtapReadIdentifyData(DeviceUnit, IDE_COMMAND_ATAPI_IDENTIFY))
//         return DEV_ATAPI;
//
//     return DEV_NONE;
// }
//
// static
// BOOLEAN
// AtapAtapiReadCapacity16(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _Out_ PREAD_CAPACITY16_DATA CapacityData)
// {
//     ATA_DEVICE_REQUEST Request = { 0 };
//
//     /* Send the SCSI READ CAPACITY(16) command */
//     Request.Flags = REQUEST_FLAG_PACKET_COMMAND;
//     Request.DataBuffer = CapacityData;
//     Request.DataTransferLength = sizeof(*CapacityData);
//     Request.u.Cdb[0] = SCSIOP_SERVICE_ACTION_IN16;
//     Request.u.Cdb[1] = SERVICE_ACTION_READ_CAPACITY16;
//     Request.u.Cdb[13] = sizeof(*CapacityData);
//
//     return AtapIssueCommand(DeviceUnit, &Request);
// }
//
// static
// BOOLEAN
// AtapAtapiReadCapacity10(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _Out_ PREAD_CAPACITY_DATA CapacityData)
// {
//     ATA_DEVICE_REQUEST Request = { 0 };
//
//     /* Send the SCSI READ CAPACITY(10) command */
//     Request.Flags = REQUEST_FLAG_PACKET_COMMAND;
//     Request.DataBuffer = CapacityData;
//     Request.DataTransferLength = sizeof(*CapacityData);
//     Request.u.Cdb[0] = SCSIOP_READ_CAPACITY;
//
//     return AtapIssueCommand(DeviceUnit, &Request);
// }
//
// static
// VOID
// AtapAtapiDetectCapacity(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _Out_ PULONG64 TotalSectors,
//     _Out_ PULONG SectorSize)
// {
//     union
//     {
//         READ_CAPACITY_DATA Cmd10;
//         READ_CAPACITY16_DATA Cmd16;
//     } CapacityData;
//     ULONG LastLba;
//
//     *TotalSectors = 0;
//     *SectorSize = 0;
//
//     if (!AtapAtapiReadCapacity10(DeviceUnit, &CapacityData.Cmd10))
//         return;
//
//     LastLba = RtlUlongByteSwap(CapacityData.Cmd10.LogicalBlockAddress);
//     if (LastLba == MAXULONG)
//     {
//         if (!AtapAtapiReadCapacity16(DeviceUnit, &CapacityData.Cmd16))
//             return;
//
//         *TotalSectors = RtlUlonglongByteSwap(CapacityData.Cmd16.LogicalBlockAddress.QuadPart) + 1;
//         *SectorSize = RtlUlongByteSwap(CapacityData.Cmd16.BytesPerBlock);
//     }
//     else
//     {
//         *TotalSectors = LastLba + 1;
//         *SectorSize = RtlUlongByteSwap(CapacityData.Cmd10.BytesPerBlock);
//     }
//
//     /*
//      * If device reports a non-zero block length, reset to defaults
//      * (we use a READ command instead of READ CD).
//      */
//     if (*SectorSize != 0)
//         *SectorSize = 2048;
// }
//
// static
// BOOLEAN
// AtapAtapiTestUnitReady(
//     _In_ PHW_DEVICE_UNIT DeviceUnit)
// {
//     ATA_DEVICE_REQUEST Request = { 0 };
//
//     /* Send the SCSI TEST UNIT READY command */
//     Request.Flags = REQUEST_FLAG_PACKET_COMMAND;
//     Request.u.Cdb[0] = SCSIOP_TEST_UNIT_READY;
//
//     return AtapIssueCommand(DeviceUnit, &Request);
// }
//
// static
// BOOLEAN
// AtapAtapiRequestSense(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _Out_ PSENSE_DATA SenseData)
// {
//     ATA_DEVICE_REQUEST Request = { 0 };
//
//     AtapAtapiBuildRequestSense(&Request, SenseData);
//     return AtapIssueCommand(DeviceUnit, &Request);
// }
//
// static
// BOOLEAN
// AtapAtapiReadToc(
//     _In_ PHW_DEVICE_UNIT DeviceUnit)
// {
//     ATA_DEVICE_REQUEST Request = { 0 };
//     UCHAR DummyData[MAXIMUM_CDROM_SIZE];
//
//     /* Send the SCSI READ TOC command */
//     Request.Flags = REQUEST_FLAG_PACKET_COMMAND;
//     Request.DataBuffer = DummyData;
//     Request.DataTransferLength = sizeof(DummyData);
//     Request.u.Cdb[0] = SCSIOP_READ_TOC;
//     Request.u.Cdb[7] = (MAXIMUM_CDROM_SIZE >> 8) & 0xFF;
//     Request.u.Cdb[8] = MAXIMUM_CDROM_SIZE & 0xFF;
//     Request.u.Cdb[9] = READ_TOC_FORMAT_SESSION << 6;
//
//     return AtapIssueCommand(DeviceUnit, &Request);
// }
//
// static
// BOOLEAN
// AtapAtapiReadyCheck(
//     _In_ PHW_DEVICE_UNIT DeviceUnit)
// {
//     SENSE_DATA SenseData;
//
//     if (!AtapAtapiTestUnitReady(DeviceUnit))
//         return FALSE;
//
//     if (!AtapAtapiRequestSense(DeviceUnit, &SenseData))
//         return FALSE;
//
//     if (SenseData.SenseKey() == SCSI_SENSE_NOT_READY)
//     {
//         if (SenseData.AdditionalSenseCode == SCSI_ADSENSE_NO_MEDIA_IN_DEVICE)
//             return FALSE;
//
//         if (SenseData.AdditionalSenseCode == SCSI_ADSENSE_LUN_NOT_READY)
//         {
//             switch (SenseData.AdditionalSenseCodeQualifier)
//             {
//                 case SCSI_SENSEQ_BECOMING_READY:
//                     /* Wait until the CD is spun up */
//                     StallExecutionProcessor(2e6);
//                     return FALSE;
//
//                 case SCSI_SENSEQ_INIT_COMMAND_REQUIRED:
//                     /* The drive needs to be spun up */
//                     AtapAtapiReadToc(DeviceUnit);
//                     return FALSE;
//
//                 default:
//                     return FALSE;
//             }
//         }
//     }
//
//     return TRUE;
// }
//
// static
// VOID
// AtapAtapiClearUnitAttention(
//     _In_ PHW_DEVICE_UNIT DeviceUnit)
// {
//     SENSE_DATA SenseData;
//     ULONG i;
//
//     for (i = 0; i < 5; ++i)
//     {
//         if (!AtapAtapiRequestSense(DeviceUnit, &SenseData))
//             continue;
//
//         if ((SenseData.SenseKey() != SCSI_SENSE_UNIT_ATTENTION) &&
//             (SenseData.AdditionalSenseCode != SCSI_ADSENSE_BUS_RESET))
//         {
//             break;
//         }
//     }
// }
//
// static
// BOOLEAN
// AtapAtapiInitDevice(
//     _In_ PHW_DEVICE_UNIT DeviceUnit)
// {
//     PIDENTIFY_PACKET_DATA IdentifyPacketData = &DeviceUnit->IdentifyPacketData;
//     ULONG i;
//
//     DeviceUnit->CdbSize = AtaDevCdbSizeInWords(IdentifyPacketData);
//
//     /* Clear the ATAPI 'Bus reset' indication */
//     AtapAtapiClearUnitAttention(DeviceUnit);
//
//     /* Make the device ready */
//     for (i = 4; i > 0; i--)
//     {
//         if (AtapAtapiReadyCheck(DeviceUnit))
//             break;
//     }
//     if (i == 0)
//     {
//         ERR("Device not ready\n");
//         return FALSE;
//     }
//
//     /* Detect a medium's capacity */
//     AtapAtapiDetectCapacity(DeviceUnit,
//                             &DeviceUnit->P.TotalSectors,
//                             &DeviceUnit->P.SectorSize);
//     if (DeviceUnit->P.SectorSize == 0 || DeviceUnit->P.TotalSectors == 0)
//     {
//         TRACE("No media found\n");
//         return FALSE;
//     }
//
//     DeviceUnit->P.Cylinders = MAXULONG;
//     DeviceUnit->P.Heads = MAXULONG;
//     DeviceUnit->P.SectorsPerTrack = MAXULONG;
//
//     DeviceUnit->MaximumTransferLength = 0xFFFF;
//
//     return TRUE;
// }
//
// static
// VOID
// AtapAtaSetMultipleMode(
//     _In_ PHW_DEVICE_UNIT DeviceUnit)
// {
//     ATA_DEVICE_REQUEST Request = { 0 };
//
// #if !defined(ATA_ENABLE_MULTIPLE_MODE)
//     /* Inherit multiple mode from the state the BIOS firmware left the device in during boot */
//     DeviceUnit->MultiSectorTransfer = AtaDevCurrentSectorsPerDrq(&DeviceUnit->IdentifyDeviceData);
//     if (DeviceUnit->MultiSectorTransfer != 0)
//         return;
// #endif
//
//     /* Use the maximum possible value */
//     DeviceUnit->MultiSectorTransfer = AtaDevMaximumSectorsPerDrq(&DeviceUnit->IdentifyDeviceData);
//     if (DeviceUnit->MultiSectorTransfer == 0)
//         return;
//
//     Request.u.TaskFile.Command = IDE_COMMAND_SET_MULTIPLE;
//     Request.u.TaskFile.Data[0].SectorCount = DeviceUnit->MultiSectorTransfer;
//     if (!AtapIssueCommand(DeviceUnit, &Request))
//     {
//         DeviceUnit->MultiSectorTransfer = 0;
//     }
// }
//
// static
// BOOLEAN
// AtapAtaInitDevice(
//     _In_ PHW_DEVICE_UNIT DeviceUnit)
// {
//     PIDENTIFY_DEVICE_DATA IdentifyData = &DeviceUnit->IdentifyDeviceData;
//     ULONG64 TotalSectors;
//     USHORT Cylinders, Heads, SectorsPerTrack;
//
//     DeviceUnit->MaximumTransferLength = 0xFF;
//
//     if (AtaDevIsCurrentGeometryValid(IdentifyData))
//         AtaDevCurrentChsTranslation(IdentifyData, &Cylinders, &Heads, &SectorsPerTrack);
//     else
//         AtaDevDefaultChsTranslation(IdentifyData, &Cylinders, &Heads, &SectorsPerTrack);
//
//     /* Using LBA addressing mode */
//     if (AtaDevHasLbaTranslation(IdentifyData))
//     {
//         DeviceUnit->P.Flags |= ATA_DEVICE_LBA;
//
//         if (AtaDevHas48BitAddressFeature(IdentifyData))
//         {
//             /* Using LBA48 addressing mode */
//             TotalSectors = AtaDevUserAddressableSectors48Bit(IdentifyData);
//             ASSERT(TotalSectors <= ATA_MAX_LBA_48);
//
//             DeviceUnit->P.Flags |= ATA_DEVICE_LBA48;
//             DeviceUnit->MaximumTransferLength = 0x10000;
//         }
//         else
//         {
//             /* Using LBA28 addressing mode */
//             TotalSectors = AtaDevUserAddressableSectors28Bit(IdentifyData);
//             ASSERT(TotalSectors <= ATA_MAX_LBA_28);
//         }
//     }
//     else
//     {
//         /* Using CHS addressing mode */
//         TotalSectors = (ULONG64)Cylinders * Heads * SectorsPerTrack;
//     }
//
//     if (TotalSectors == 0)
//     {
//         ERR("Unknown geometry\n");
//         return FALSE;
//     }
//
//     DeviceUnit->P.TotalSectors = TotalSectors;
//     DeviceUnit->P.Cylinders = Cylinders;
//     DeviceUnit->P.Heads = Heads;
//     DeviceUnit->P.SectorsPerTrack = SectorsPerTrack;
//     DeviceUnit->P.SectorSize = AtaDevBytesPerLogicalSector(IdentifyData);
//     ASSERT(DeviceUnit->P.SectorSize >= 512);
//     DeviceUnit->P.SectorSize = max(DeviceUnit->P.SectorSize, 512);
//
//     AtapAtaSetMultipleMode(DeviceUnit);
//
//     TRACE("Multiple sector setting %u\n", (DeviceUnit->MultiSectorTransfer) as usize);
//
//     return TRUE;
// }
//
// static
// BOOLEAN
// AtapAnalyzeIdentifyData(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ ATA_DEVICE_CLASS DeviceClass)
// {
//     PIDENTIFY_DEVICE_DATA Id = &DeviceUnit->IdentifyDeviceData;
//     ULONG i;
//
//     /* Verify the checksum */
//     if (!AtaDevIsIdentifyDataValid(Id))
//     {
//         ERR("Identify data CRC error\n");
//         return FALSE;
//     }
//
//     if (DeviceClass == DEV_ATAPI)
//     {
//         DeviceUnit->P.Flags |= ATA_DEVICE_ATAPI | ATA_DEVICE_LBA;
//
//         /* The returned string is not byteswapped */
//         if (Id->ModelNumber[0] == 'N' &&
//             Id->ModelNumber[1] == 'E' &&
//             Id->ModelNumber[2] == 'C' &&
//             Id->ModelNumber[3] == ' ')
//         {
//             DeviceUnit->P.Flags |= ATA_DEVICE_IS_NEC_CDR260;
//         }
//     }
//
//     /* Swap byte order of the ASCII data */
//     for (i = 0; i < sizeof(Id->SerialNumber) / 2; ++i)
//         ((PUSHORT)Id->SerialNumber)[i] = RtlUshortByteSwap(((PUSHORT)Id->SerialNumber)[i]);
//
//     for (i = 0; i < sizeof(Id->FirmwareRevision) / 2; ++i)
//         ((PUSHORT)Id->FirmwareRevision)[i] = RtlUshortByteSwap(((PUSHORT)Id->FirmwareRevision)[i]);
//
//     for (i = 0; i < sizeof(Id->ModelNumber) / 2; ++i)
//         ((PUSHORT)Id->ModelNumber)[i] = RtlUshortByteSwap(((PUSHORT)Id->ModelNumber)[i]);
//
//     TRACE("MN  '%.*s'\n", (sizeof(Id->ModelNumber)) as usize, (Id->ModelNumber) as usize);
//     TRACE("FR  '%.*s'\n", (sizeof(Id->FirmwareRevision)) as usize, (Id->FirmwareRevision) as usize);
//     TRACE("S/N '%.*s'\n", (sizeof(Id->SerialNumber)) as usize, (Id->SerialNumber) as usize);
//
//     return TRUE;
// }
//
// static
// BOOLEAN
// AtapInitDevice(
//     _In_ PHW_DEVICE_UNIT DeviceUnit,
//     _In_ ATA_DEVICE_CLASS DeviceClass)
// {
//     if (!AtapAnalyzeIdentifyData(DeviceUnit, DeviceClass))
//         return FALSE;
//
//     if (DeviceClass == DEV_ATAPI)
//         return AtapAtapiInitDevice(DeviceUnit);
//     else
//         return AtapAtaInitDevice(DeviceUnit);
// }
//
// static
// BOOLEAN
// AtapIdentifyChannel(
//     _In_ ULONG ChannelNumber,
//     _Out_ PIDE_REGISTERS Registers)
// {
//     const IDE_REG IoBase = AtapChannelBaseArray[ChannelNumber];
//     ULONG Spare;
//
//
//     Spare = 1;
//     Registers->Control     = IoBase + 0x206;
//     Registers->Data        = IoBase + 0 * Spare;
//     Registers->Error       = IoBase + 1 * Spare;
//     Registers->SectorCount = IoBase + 2 * Spare;
//     Registers->LbaLow      = IoBase + 3 * Spare;
//     Registers->LbaMid      = IoBase + 4 * Spare;
//     Registers->LbaHigh     = IoBase + 5 * Spare;
//     Registers->Device      = IoBase + 6 * Spare;
//     Registers->Status      = IoBase + 7 * Spare;
//
//     return TRUE;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// BOOLEAN
// AtaReadLogicalSectors(
//     _In_ PDEVICE_UNIT DeviceUnit,
//     _In_ ULONG64 SectorNumber,
//     _In_ ULONG SectorCount,
//     _Out_writes_bytes_all_(SectorCount * DeviceUnit->SectorSize) PVOID Buffer)
// {
//     PHW_DEVICE_UNIT Unit = (PHW_DEVICE_UNIT)DeviceUnit;
//     ATA_DEVICE_REQUEST Request = { 0 };
//
//     ASSERT(Unit);
//     ASSERT((SectorNumber + SectorCount) <= Unit->P.TotalSectors);
//     ASSERT(SectorCount != 0);
//
//     while (SectorCount > 0)
//     {
//         ULONG BlockCount;
//
//         BlockCount = min(SectorCount, Unit->MaximumTransferLength);
//
//         Request.DataBuffer = Buffer;
//         Request.DataTransferLength = BlockCount * Unit->P.SectorSize;
//
//         if (Unit->P.Flags & ATA_DEVICE_ATAPI)
//             AtapBuildReadPacketCommand(&Request, SectorNumber, BlockCount);
//         else
//             AtapBuildReadTaskFile(Unit, &Request, SectorNumber, BlockCount);
//
//         if (!AtapIssueCommand(Unit, &Request))
//             return FALSE;
//
//         SectorNumber += BlockCount;
//         SectorCount -= BlockCount;
//
//         Buffer = (PVOID)((ULONG_PTR)Buffer + Unit->P.SectorSize);
//     }
//
//     return TRUE;
// }
//
// PDEVICE_UNIT
// AtaGetDevice(
//     _In_ UCHAR UnitNumber)
// {
//     if (UnitNumber < RTL_NUMBER_OF(AtapUnits))
//         return (PDEVICE_UNIT)AtapUnits[UnitNumber];
//
//     return NULL;
// }
//
// BOOLEAN
// AtaInit(
//     _Out_ PUCHAR DetectedCount)
// {
//     ULONG ChannelNumber;
//
//     *DetectedCount = 0;
//
//     /* Enumerate IDE channels */
//     for (ChannelNumber = 0; ChannelNumber < CHANNEL_MAX_CHANNELS; ++ChannelNumber)
//     {
//         UCHAR DeviceNumber;
//         IDE_REGISTERS Registers;
//
//         if (!AtapIdentifyChannel(ChannelNumber, &Registers))
//             continue;
//
//         /* Check for devices attached to the bus */
//         for (DeviceNumber = 0; DeviceNumber < CHANNEL_MAX_DEVICES; ++DeviceNumber)
//         {
//             PHW_DEVICE_UNIT DeviceUnit;
//             ATA_DEVICE_CLASS DeviceClass;
//
//             /* Allocate a new device unit structure */
//             DeviceUnit = FrLdrTempAlloc(sizeof(*DeviceUnit), TAG_ATA_DEVICE);
//             if (!DeviceUnit)
//             {
//                 ERR("Failed to allocate device unit!\n");
//                 continue;
//             }
//             RtlZeroMemory(DeviceUnit, sizeof(*DeviceUnit));
//
//             /* Perform a minimal initialization */
//             RtlCopyMemory(&DeviceUnit->Registers, &Registers, sizeof(Registers));
//             DeviceUnit->DeviceNumber = DeviceNumber;
//             DeviceUnit->P.SectorSize = 512;
//             DeviceUnit->DeviceSelect = (DEV_SLAVE(DeviceNumber) << 4) | IDE_DRIVE_SELECT;
//
//             /* Let's see what kind of device this is */
//             DeviceClass = AtapIdentifyDevice(DeviceUnit);
//             if (DeviceClass == DEV_NONE)
//                 goto NextDevice;
//
//             TRACE("Found %lu device at %X:%u\n", (DeviceClass) as usize, (Registers.Data) as usize, (DeviceNumber) as usize);
//
//             if (!AtapInitDevice(DeviceUnit, DeviceClass))
//                 goto NextDevice;
//
//             TRACE("Total sectors %I64u of size %lu, CHS %lu:%lu:%lu, %lx\n", (//                   DeviceUnit->P.TotalSectors) as usize, (//                   DeviceUnit->P.SectorSize) as usize, (//                   DeviceUnit->P.Cylinders) as usize, (//                   DeviceUnit->P.Heads) as usize, (//                   DeviceUnit->P.SectorsPerTrack) as usize, (//                   DeviceUnit->P.Flags) as usize);
//
//             AtapUnits[(*DetectedCount)++] = DeviceUnit;
//             continue;
//
// NextDevice:
//             FrLdrTempFree(DeviceUnit, TAG_ATA_DEVICE);
//         }
//     }
//
//     return (*DetectedCount > 0);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
