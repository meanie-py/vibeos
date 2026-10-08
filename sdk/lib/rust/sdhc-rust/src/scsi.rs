//! The card as a SCSI direct-access device, which is what disk.sys and
//! classpnp speak: SRBs carrying CDBs on IRP_MJ_SCSI, and the storage
//! property/address IOCTLs, answered the way usbstor answers them for a USB
//! stick (drivers/usb/usbstor/disk.c), since that is the path proven to give a
//! drive in Explorer on this board.
//!
//! With `pnp::WRITE_ENABLED` off, MODE SENSE reports write protection (fastfat
//! then mounts read-only) and a WRITE is refused with DATA PROTECT.

use core::ffi::c_void;
use core::mem::size_of;

use ntkrnl_sys::ddk::{NTSTATUS, PIRP};

use crate::host::{Error, Host, SECTOR};
use crate::print::sdprint;

/// srb.h SCSI_REQUEST_BLOCK. ntkrnl-sys has it only as an opaque type.
#[repr(C)]
pub struct Srb {
    pub Length: u16,
    pub Function: u8,
    pub SrbStatus: u8,
    pub ScsiStatus: u8,
    pub PathId: u8,
    pub TargetId: u8,
    pub Lun: u8,
    pub QueueTag: u8,
    pub QueueAction: u8,
    pub CdbLength: u8,
    pub SenseInfoBufferLength: u8,
    pub SrbFlags: u32,
    pub DataTransferLength: u32,
    pub TimeOutValue: u32,
    pub DataBuffer: *mut c_void,
    pub SenseInfoBuffer: *mut c_void,
    pub NextSrb: *mut Srb,
    pub OriginalRequest: *mut c_void,
    pub SrbExtension: *mut c_void,
    pub QueueSortKey: u32,
    #[cfg(target_pointer_width = "64")]
    pub Reserved: u32,
    pub Cdb: [u8; 16],
}
const _: () = assert!(size_of::<Srb>() == if cfg!(target_pointer_width = "64") { 0x58 } else { 0x40 });

pub const SRB_FUNCTION_EXECUTE_SCSI: u8 = 0x00;
pub const SRB_FUNCTION_CLAIM_DEVICE: u8 = 0x01;
pub const SRB_FUNCTION_RELEASE_QUEUE: u8 = 0x04;
pub const SRB_FUNCTION_RELEASE_DEVICE: u8 = 0x06;
pub const SRB_FUNCTION_SHUTDOWN: u8 = 0x07;
pub const SRB_FUNCTION_FLUSH: u8 = 0x08;
pub const SRB_FUNCTION_FLUSH_QUEUE: u8 = 0x15;
pub const SRB_FUNCTION_LOCK_QUEUE: u8 = 0x18;
pub const SRB_FUNCTION_UNLOCK_QUEUE: u8 = 0x19;

pub const SRB_STATUS_PENDING: u8 = 0x00;
pub const SRB_STATUS_SUCCESS: u8 = 0x01;
pub const SRB_STATUS_BUSY: u8 = 0x05;
pub const SRB_STATUS_ERROR: u8 = 0x04;
pub const SRB_STATUS_INVALID_REQUEST: u8 = 0x06;
const SRB_STATUS_AUTOSENSE_VALID: u8 = 0x80;
const SRB_FLAGS_DISABLE_AUTOSENSE: u32 = 0x20;
const SCSISTAT_GOOD: u8 = 0x00;
const SCSISTAT_CHECK_CONDITION: u8 = 0x02;

const SCSIOP_TEST_UNIT_READY: u8 = 0x00;
const SCSIOP_REQUEST_SENSE: u8 = 0x03;
const SCSIOP_READ6: u8 = 0x08;
const SCSIOP_WRITE6: u8 = 0x0A;
const SCSIOP_INQUIRY: u8 = 0x12;
const SCSIOP_MODE_SENSE: u8 = 0x1A;
const SCSIOP_START_STOP_UNIT: u8 = 0x1B;
const SCSIOP_MEDIUM_REMOVAL: u8 = 0x1E;
const SCSIOP_READ_CAPACITY: u8 = 0x25;
const SCSIOP_READ: u8 = 0x28;
const SCSIOP_WRITE: u8 = 0x2A;
const SCSIOP_VERIFY: u8 = 0x2F;
const SCSIOP_SYNCHRONIZE_CACHE: u8 = 0x35;
const SCSIOP_MODE_SENSE10: u8 = 0x5A;
const SCSIOP_READ16: u8 = 0x88;
const SCSIOP_WRITE16: u8 = 0x8A;
const SCSIOP_VERIFY16: u8 = 0x8F;
const SCSIOP_SYNCHRONIZE_CACHE16: u8 = 0x91;
const SCSIOP_SERVICE_ACTION_IN16: u8 = 0x9E;
const SERVICE_ACTION_READ_CAPACITY16: u8 = 0x10;
const SCSIOP_READ12: u8 = 0xA8;
const SCSIOP_WRITE12: u8 = 0xAA;

const SCSI_SENSE_MEDIUM_ERROR: u8 = 0x03;
const SCSI_SENSE_ILLEGAL_REQUEST: u8 = 0x05;
const SCSI_SENSE_DATA_PROTECT: u8 = 0x07;
const SCSI_ADSENSE_WRITE_ERROR: u8 = 0x0C;
const SCSI_ADSENSE_UNRECOVERED_ERROR: u8 = 0x11;
const SCSI_ADSENSE_ILLEGAL_COMMAND: u8 = 0x20;
const SCSI_ADSENSE_ILLEGAL_BLOCK: u8 = 0x21;
const SCSI_ADSENSE_INVALID_CDB: u8 = 0x24;
const SCSI_ADSENSE_WRITE_PROTECT: u8 = 0x27;
/// Mode parameter header, device-specific byte: WP.
const MODE_DSP_WRITE_PROTECT: u8 = 0x80;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xC000_0185u32 as NTSTATUS;
pub const STATUS_NOT_SUPPORTED: NTSTATUS = 0xC000_00BBu32 as NTSTATUS;
const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC000_0023u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000_000Du32 as NTSTATUS;

/// The most one SRB moves, as the adapter descriptor tells classpnp.
pub const MAX_TRANSFER: u32 = 64 * 1024;
const MAX_TRANSFER_SECTORS: u64 = (MAX_TRANSFER as usize / SECTOR) as u64;

/// What the card is called, as the SCSI strings and the PnP IDs show it.
pub struct Ident {
    /// INQUIRY product: the CID product name, printable ASCII.
    pub product: [u8; 5],
    /// INQUIRY revision: the CID product revision as "n.m".
    pub revision: [u8; 3],
    pub serial: u32,
}

impl Ident {
    pub fn of(host: &Host) -> Self {
        let c = host.card;
        let mut product = c.name;
        for b in product.iter_mut() {
            if !b.is_ascii_alphanumeric() {
                *b = b'_';
            }
        }
        let digit = |n: u8| if n < 10 { b'0' + n } else { b'A' + n - 10 };
        Ident { product, revision: [digit(c.revision >> 4), b'.', digit(c.revision & 0xF)], serial: c.serial }
    }
}

struct Sense(u8, u8);

fn reply(data: *mut u8, len: usize, bytes: &[u8]) -> usize {
    let n = bytes.len().min(len);
    if n > 0 {
        unsafe { core::ptr::copy_nonoverlapping(bytes.as_ptr(), data, n) };
    }
    n
}

fn be(b: &[u8]) -> u64 {
    b.iter().fold(0, |a, &x| a << 8 | x as u64)
}

/// The (LBA, sector count) a read or write CDB addresses.
fn extent(cdb: &[u8; 16]) -> (u64, u64) {
    match cdb[0] {
        SCSIOP_READ6 | SCSIOP_WRITE6 => {
            let n = cdb[4] as u64;
            (be(&cdb[1..4]) & 0x1F_FFFF, if n == 0 { 256 } else { n })
        }
        SCSIOP_READ12 | SCSIOP_WRITE12 => (be(&cdb[2..6]), be(&cdb[6..10])),
        SCSIOP_READ16 | SCSIOP_WRITE16 => (be(&cdb[2..10]), be(&cdb[10..14])),
        _ => (be(&cdb[2..6]), be(&cdb[7..9])),
    }
}

/// Run one EXECUTE_SCSI SRB against the card. `data` is the system address of
/// the SRB's data buffer (null when it moves none). Returns the IRP status.
///
/// # Safety
/// `srb` must be a live SRB and `data` valid for its DataTransferLength.
pub unsafe fn execute(host: &Host, ident: &Ident, write_enabled: bool, srb: *mut Srb, data: *mut u8) -> NTSTATUS {
    let cdb = (*srb).Cdb;
    let len = if data.is_null() { 0 } else { (*srb).DataTransferLength as usize };
    let blocks = host.card.blocks;
    let result: Result<usize, Sense> = match cdb[0] {
        SCSIOP_TEST_UNIT_READY
        | SCSIOP_START_STOP_UNIT
        | SCSIOP_MEDIUM_REMOVAL
        | SCSIOP_SYNCHRONIZE_CACHE
        | SCSIOP_SYNCHRONIZE_CACHE16
        | SCSIOP_VERIFY
        | SCSIOP_VERIFY16 => Ok(0),
        SCSIOP_INQUIRY if cdb[1] & 1 != 0 => Err(Sense(SCSI_SENSE_ILLEGAL_REQUEST, SCSI_ADSENSE_INVALID_CDB)),
        SCSIOP_INQUIRY => {
            let mut q = [b' '; 36];
            q[0] = 0x00; // direct access block device
            q[1] = 0x80; // removable medium
            q[2] = 0x02; // SCSI-2
            q[3] = 0x02; // response data format
            q[4] = 31; // additional length
            q[5..8].fill(0);
            q[8..10].copy_from_slice(b"SD");
            q[16..21].copy_from_slice(&ident.product);
            q[32..35].copy_from_slice(&ident.revision);
            Ok(reply(data, len, &q))
        }
        SCSIOP_REQUEST_SENSE => {
            let mut s = [0u8; 18];
            s[0] = 0x70;
            s[7] = 10;
            Ok(reply(data, len, &s))
        }
        SCSIOP_READ_CAPACITY => {
            let mut c = [0u8; 8];
            c[..4].copy_from_slice(&(blocks.saturating_sub(1).min(0xFFFF_FFFF) as u32).to_be_bytes());
            c[4..].copy_from_slice(&(SECTOR as u32).to_be_bytes());
            Ok(reply(data, len, &c))
        }
        SCSIOP_SERVICE_ACTION_IN16 if cdb[1] & 0x1F == SERVICE_ACTION_READ_CAPACITY16 => {
            let mut c = [0u8; 32];
            c[..8].copy_from_slice(&blocks.saturating_sub(1).to_be_bytes());
            c[8..12].copy_from_slice(&(SECTOR as u32).to_be_bytes());
            Ok(reply(data, len, &c))
        }
        // A header and no pages: classpnp finds no caching page and moves on.
        SCSIOP_MODE_SENSE => {
            let wp = if write_enabled { 0 } else { MODE_DSP_WRITE_PROTECT };
            Ok(reply(data, len, &[3, 0, wp, 0]))
        }
        SCSIOP_MODE_SENSE10 => {
            let wp = if write_enabled { 0 } else { MODE_DSP_WRITE_PROTECT };
            Ok(reply(data, len, &[0, 6, 0, wp, 0, 0, 0, 0]))
        }
        SCSIOP_READ6 | SCSIOP_READ | SCSIOP_READ12 | SCSIOP_READ16 => transfer(host, &cdb, blocks, data, len, false),
        SCSIOP_WRITE6 | SCSIOP_WRITE | SCSIOP_WRITE12 | SCSIOP_WRITE16 if !write_enabled => {
            Err(Sense(SCSI_SENSE_DATA_PROTECT, SCSI_ADSENSE_WRITE_PROTECT))
        }
        SCSIOP_WRITE6 | SCSIOP_WRITE | SCSIOP_WRITE12 | SCSIOP_WRITE16 => transfer(host, &cdb, blocks, data, len, true),
        _ => Err(Sense(SCSI_SENSE_ILLEGAL_REQUEST, SCSI_ADSENSE_ILLEGAL_COMMAND)),
    };
    match result {
        Ok(n) => {
            (*srb).ScsiStatus = SCSISTAT_GOOD;
            // A short reply (MODE SENSE with no pages) is a success with the
            // shorter length. SRB_STATUS_DATA_OVERRUN with a successful IRP is
            // what classpnp calls "SRB and IRP status don't match", and fails.
            (*srb).DataTransferLength = n as u32;
            (*srb).SrbStatus = SRB_STATUS_SUCCESS;
            STATUS_SUCCESS
        }
        Err(Sense(key, asc)) => {
            (*srb).ScsiStatus = SCSISTAT_CHECK_CONDITION;
            (*srb).SrbStatus = SRB_STATUS_ERROR;
            (*srb).DataTransferLength = 0;
            let sense = (*srb).SenseInfoBuffer as *mut u8;
            let room = (*srb).SenseInfoBufferLength as usize;
            if !sense.is_null() && room > 0 && (*srb).SrbFlags & SRB_FLAGS_DISABLE_AUTOSENSE == 0 {
                let mut s = [0u8; 18];
                s[0] = 0x70; // current error, fixed format
                s[2] = key;
                s[7] = 10;
                s[12] = asc;
                core::ptr::copy_nonoverlapping(s.as_ptr(), sense, room.min(s.len()));
                (*srb).SrbStatus |= SRB_STATUS_AUTOSENSE_VALID;
            }
            STATUS_IO_DEVICE_ERROR
        }
    }
}

unsafe fn transfer(host: &Host, cdb: &[u8; 16], blocks: u64, data: *mut u8, len: usize, write: bool) -> Result<usize, Sense> {
    let (lba, count) = extent(cdb);
    if count == 0 {
        return Ok(0);
    }
    if lba.checked_add(count).is_none_or(|end| end > blocks) {
        return Err(Sense(SCSI_SENSE_ILLEGAL_REQUEST, SCSI_ADSENSE_ILLEGAL_BLOCK));
    }
    if count * SECTOR as u64 > len as u64 {
        return Err(Sense(SCSI_SENSE_ILLEGAL_REQUEST, SCSI_ADSENSE_INVALID_CDB));
    }
    let mut done = 0;
    while done < count {
        let n = (count - done).min(MAX_TRANSFER_SECTORS);
        let buf = data.add((done * SECTOR as u64) as usize);
        let r: Result<(), Error> =
            if write { host.write(lba + done, n as u16, buf) } else { host.read(lba + done, n as u16, buf) };
        if let Err(e) = r {
            sdprint!("{} of {} sectors at LBA {} failed: {:?}\n", if write { "write" } else { "read" }, n, lba + done, e);
            let asc = if write { SCSI_ADSENSE_WRITE_ERROR } else { SCSI_ADSENSE_UNRECOVERED_ERROR };
            return Err(Sense(SCSI_SENSE_MEDIUM_ERROR, asc));
        }
        done += n;
    }
    Ok((count * SECTOR as u64) as usize)
}

// ---------------------------------------------------------------------------
// IRP_MJ_DEVICE_CONTROL on the disk PDO
// ---------------------------------------------------------------------------

const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D_1400;
const IOCTL_SCSI_GET_CAPABILITIES: u32 = 0x0004_1010;
const IOCTL_SCSI_GET_ADDRESS: u32 = 0x0004_1018;
const STORAGE_DEVICE_PROPERTY: u32 = 0;
const STORAGE_ADAPTER_PROPERTY: u32 = 1;
const PROPERTY_STANDARD_QUERY: u32 = 0;
const PROPERTY_EXISTS_QUERY: u32 = 1;
const BUS_TYPE_SD: u8 = 0x0C;
/// STORAGE_ADAPTER_DESCRIPTOR_WIN8, through AddressType.
const ADAPTER_DESCRIPTOR_SIZE: usize = 32;
/// STORAGE_DEVICE_DESCRIPTOR up to RawDeviceProperties, rounded; the strings follow.
const DEVICE_DESCRIPTOR_SIZE: usize = 40;
const SCSI_CAPABILITIES_SIZE: usize = 24;

fn put32(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

/// Answer a storage IOCTL in the IRP's system buffer. Returns (status, bytes).
pub unsafe fn device_control(ident: &Ident, irp: PIRP, code: u32, in_len: usize, out_len: usize) -> (NTSTATUS, usize) {
    let buf = (*irp).AssociatedIrp.SystemBuffer as *mut u8;
    match code {
        IOCTL_STORAGE_QUERY_PROPERTY => {
            if in_len < 8 || buf.is_null() {
                return (STATUS_INVALID_PARAMETER, 0);
            }
            let q = core::slice::from_raw_parts(buf, 8);
            let (id, kind) = (le32(&q[0..4]), le32(&q[4..8]));
            if id != STORAGE_DEVICE_PROPERTY && id != STORAGE_ADAPTER_PROPERTY {
                return (STATUS_NOT_SUPPORTED, 0);
            }
            match kind {
                PROPERTY_EXISTS_QUERY => return (STATUS_SUCCESS, 0),
                PROPERTY_STANDARD_QUERY => {}
                _ => return (STATUS_NOT_SUPPORTED, 0),
            }
            let mut d = [0u8; 96];
            let total = if id == STORAGE_ADAPTER_PROPERTY {
                put32(&mut d, 0, ADAPTER_DESCRIPTOR_SIZE as u32);
                put32(&mut d, 4, ADAPTER_DESCRIPTOR_SIZE as u32);
                put32(&mut d, 8, MAX_TRANSFER);
                put32(&mut d, 12, MAX_TRANSFER / 4096 + 1); // MaximumPhysicalPages
                put32(&mut d, 16, 0); // AlignmentMask: PIO copies any alignment
                d[20] = 1; // AdapterUsesPio
                d[24] = BUS_TYPE_SD;
                d[26] = 3; // BusMajorVersion: SD 3.0
                // SrbType 0: SCSI_REQUEST_BLOCK, not STORAGE_REQUEST_BLOCK.
                ADAPTER_DESCRIPTOR_SIZE
            } else {
                // Vendor, product, revision and serial strings after the header.
                let mut at = DEVICE_DESCRIPTOR_SIZE;
                let mut string = |d: &mut [u8; 96], s: &[u8], field: usize| {
                    put32(d, field, at as u32);
                    d[at..at + s.len()].copy_from_slice(s);
                    at += s.len() + 1;
                };
                string(&mut d, b"SD", 12);
                string(&mut d, &ident.product, 16);
                string(&mut d, &ident.revision, 20);
                let mut serial = [0u8; 8];
                for (i, c) in serial.iter_mut().enumerate() {
                    *c = b"0123456789ABCDEF"[(ident.serial >> (28 - 4 * i)) as usize & 0xF];
                }
                string(&mut d, &serial, 24);
                put32(&mut d, 0, DEVICE_DESCRIPTOR_SIZE as u32); // Version
                put32(&mut d, 4, at as u32); // Size
                d[8] = 0x00; // direct access
                d[10] = 1; // RemovableMedia
                put32(&mut d, 28, BUS_TYPE_SD as u32);
                put32(&mut d, 32, (at - DEVICE_DESCRIPTOR_SIZE) as u32); // RawPropertiesLength
                at
            };
            if out_len < total {
                // Too small for the whole descriptor: report its size in the header.
                if out_len < 8 {
                    return (STATUS_BUFFER_TOO_SMALL, 0);
                }
                put32(&mut d, 0, total as u32);
                put32(&mut d, 4, total as u32);
                core::ptr::copy_nonoverlapping(d.as_ptr(), buf, 8);
                return (STATUS_SUCCESS, 8);
            }
            core::ptr::copy_nonoverlapping(d.as_ptr(), buf, total);
            (STATUS_SUCCESS, total)
        }
        IOCTL_SCSI_GET_ADDRESS => {
            if out_len < 8 || buf.is_null() {
                return (STATUS_BUFFER_TOO_SMALL, 0);
            }
            // Length, then port/path/target/LUN 0.
            core::ptr::copy_nonoverlapping([8u8, 0, 0, 0, 0, 0, 0, 0].as_ptr(), buf, 8);
            (STATUS_SUCCESS, 8)
        }
        IOCTL_SCSI_GET_CAPABILITIES => {
            if out_len < SCSI_CAPABILITIES_SIZE || buf.is_null() {
                return (STATUS_BUFFER_TOO_SMALL, 0);
            }
            let mut c = [0u8; SCSI_CAPABILITIES_SIZE];
            put32(&mut c, 0, SCSI_CAPABILITIES_SIZE as u32);
            put32(&mut c, 4, MAX_TRANSFER);
            put32(&mut c, 8, MAX_TRANSFER / 4096 + 1);
            c[22] = 1; // AdapterUsesPio
            core::ptr::copy_nonoverlapping(c.as_ptr(), buf, c.len());
            (STATUS_SUCCESS, c.len())
        }
        _ => (STATUS_NOT_SUPPORTED, 0),
    }
}

fn le32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}
