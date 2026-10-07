//! A block device as a SCSI direct-access disk: what disk.sys and classpnp
//! send (SRBs carrying CDBs on IRP_MJ_SCSI, and storage IOCTLs) and the
//! answers they expect, the way usbstor gives them for a USB stick
//! (drivers/usb/usbstor/disk.c), since that is the path proven to put a drive
//! in Explorer.
//!
//! The driver keeps the device: [`decode`] answers everything that needs no
//! media access and turns the rest into a [`Request`] (read, write, flush) for
//! the driver to carry out; [`finish`] then writes the outcome into the SRB.
//! Nothing here touches the kernel, so the whole layer runs under
//! `cargo test` on the host.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

use core::ffi::c_void;
use core::mem::size_of;

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
pub const SRB_STATUS_ERROR: u8 = 0x04;
pub const SRB_STATUS_BUSY: u8 = 0x05;
pub const SRB_STATUS_INVALID_REQUEST: u8 = 0x06;
const SRB_STATUS_DATA_OVERRUN: u8 = 0x12;
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

pub const SCSI_SENSE_MEDIUM_ERROR: u8 = 0x03;
pub const SCSI_SENSE_HARDWARE_ERROR: u8 = 0x04;
pub const SCSI_SENSE_ILLEGAL_REQUEST: u8 = 0x05;
pub const SCSI_SENSE_DATA_PROTECT: u8 = 0x07;
pub const SCSI_ADSENSE_WRITE_ERROR: u8 = 0x0C;
pub const SCSI_ADSENSE_UNRECOVERED_ERROR: u8 = 0x11;
pub const SCSI_ADSENSE_ILLEGAL_COMMAND: u8 = 0x20;
pub const SCSI_ADSENSE_ILLEGAL_BLOCK: u8 = 0x21;
pub const SCSI_ADSENSE_INVALID_CDB: u8 = 0x24;
pub const SCSI_ADSENSE_WRITE_PROTECT: u8 = 0x27;
/// Mode parameter header, device-specific byte: WP.
const MODE_DSP_WRITE_PROTECT: u8 = 0x80;

pub const STATUS_SUCCESS: i32 = 0;
pub const STATUS_IO_DEVICE_ERROR: i32 = 0xC000_0185u32 as i32;
pub const STATUS_NOT_SUPPORTED: i32 = 0xC000_00BBu32 as i32;
pub const STATUS_BUFFER_TOO_SMALL: i32 = 0xC000_0023u32 as i32;
pub const STATUS_INVALID_PARAMETER: i32 = 0xC000_000Du32 as i32;

/// STORAGE_BUS_TYPE values the drivers report.
pub const BUS_TYPE_SCSI: u8 = 0x01;
pub const BUS_TYPE_SD: u8 = 0x0C;
pub const BUS_TYPE_NVME: u8 = 0x11;

/// The disk as INQUIRY, READ CAPACITY and the storage descriptors show it.
#[derive(Clone, Copy)]
pub struct Disk {
    /// INQUIRY vendor, product and revision: printable ASCII, space-padded.
    pub vendor: [u8; 8],
    pub product: [u8; 16],
    pub revision: [u8; 4],
    /// The device's serial number, space-padded (trailing spaces are dropped).
    pub serial: [u8; 20],
    pub block_size: u32,
    pub blocks: u64,
    pub removable: bool,
    /// Off: MODE SENSE reports write protection and WRITEs fail DATA PROTECT.
    pub writable: bool,
    pub bus_type: u8,
    pub bus_version: (u16, u16),
    /// The most one SRB may move, as the adapter descriptor tells classpnp.
    pub max_transfer: u32,
    /// IOCTL_SCSI_GET_ADDRESS: port, path, target, LUN.
    pub address: [u8; 4],
}

/// Copy `src` into the space-padded field `dst`, keeping printable ASCII.
pub fn ascii_field(dst: &mut [u8], src: &[u8]) {
    dst.fill(b' ');
    for (d, &s) in dst.iter_mut().zip(src) {
        *d = if s.is_ascii_graphic() || s == b' ' { s } else { b'_' };
    }
}

/// `field` with its trailing spaces (and NULs) removed.
pub fn trimmed(field: &[u8]) -> &[u8] {
    let end = field.iter().rposition(|&b| b != b' ' && b != 0).map_or(0, |i| i + 1);
    &field[..end]
}

/// A CHECK CONDITION: sense key and additional sense code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sense(pub u8, pub u8);

/// What the driver must do to complete an EXECUTE_SCSI SRB.
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    /// Finished: the reply's length in the data buffer, or the failure.
    Done(Result<usize, Sense>),
    /// Read `blocks` blocks at `lba` into the start of the data buffer.
    Read { lba: u64, blocks: u64 },
    /// Write `blocks` blocks from the start of the data buffer to `lba`.
    Write { lba: u64, blocks: u64 },
    /// Make every completed write durable.
    Flush,
}

fn be(b: &[u8]) -> u64 {
    b.iter().fold(0, |a, &x| a << 8 | x as u64)
}

fn reply(data: &mut [u8], bytes: &[u8]) -> Request {
    let n = bytes.len().min(data.len());
    data[..n].copy_from_slice(&bytes[..n]);
    Request::Done(Ok(n))
}

/// The (LBA, block count) a read or write CDB addresses.
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

/// Decode one EXECUTE_SCSI CDB. `data` is the SRB's data buffer (empty when
/// it moves none); replies are written into it.
pub fn decode(disk: &Disk, cdb: &[u8; 16], data: &mut [u8]) -> Request {
    match cdb[0] {
        SCSIOP_TEST_UNIT_READY
        | SCSIOP_START_STOP_UNIT
        | SCSIOP_MEDIUM_REMOVAL
        | SCSIOP_VERIFY
        | SCSIOP_VERIFY16 => Request::Done(Ok(0)),
        SCSIOP_SYNCHRONIZE_CACHE | SCSIOP_SYNCHRONIZE_CACHE16 => Request::Flush,
        // No vital product data pages: classpnp takes the standard data.
        SCSIOP_INQUIRY if cdb[1] & 1 != 0 => Request::Done(Err(Sense(SCSI_SENSE_ILLEGAL_REQUEST, SCSI_ADSENSE_INVALID_CDB))),
        SCSIOP_INQUIRY => {
            let mut q = [b' '; 36];
            q[0] = 0x00; // direct access block device
            q[1] = if disk.removable { 0x80 } else { 0 };
            q[2] = 0x05; // SPC-3
            q[3] = 0x02; // response data format
            q[4] = 31; // additional length
            q[5..8].fill(0);
            q[8..16].copy_from_slice(&disk.vendor);
            q[16..32].copy_from_slice(&disk.product);
            q[32..36].copy_from_slice(&disk.revision);
            reply(data, &q)
        }
        SCSIOP_REQUEST_SENSE => {
            let mut s = [0u8; 18];
            s[0] = 0x70;
            s[7] = 10;
            reply(data, &s)
        }
        SCSIOP_READ_CAPACITY => {
            let mut c = [0u8; 8];
            c[..4].copy_from_slice(&(disk.blocks.saturating_sub(1).min(0xFFFF_FFFF) as u32).to_be_bytes());
            c[4..].copy_from_slice(&disk.block_size.to_be_bytes());
            reply(data, &c)
        }
        SCSIOP_SERVICE_ACTION_IN16 if cdb[1] & 0x1F == SERVICE_ACTION_READ_CAPACITY16 => {
            let mut c = [0u8; 32];
            c[..8].copy_from_slice(&disk.blocks.saturating_sub(1).to_be_bytes());
            c[8..12].copy_from_slice(&disk.block_size.to_be_bytes());
            reply(data, &c)
        }
        // A header and no pages: classpnp finds no caching page and moves on.
        SCSIOP_MODE_SENSE => {
            let wp = if disk.writable { 0 } else { MODE_DSP_WRITE_PROTECT };
            reply(data, &[3, 0, wp, 0])
        }
        SCSIOP_MODE_SENSE10 => {
            let wp = if disk.writable { 0 } else { MODE_DSP_WRITE_PROTECT };
            reply(data, &[0, 6, 0, wp, 0, 0, 0, 0])
        }
        SCSIOP_READ6 | SCSIOP_READ | SCSIOP_READ12 | SCSIOP_READ16 => transfer(disk, cdb, data.len(), false),
        SCSIOP_WRITE6 | SCSIOP_WRITE | SCSIOP_WRITE12 | SCSIOP_WRITE16 if !disk.writable => {
            Request::Done(Err(Sense(SCSI_SENSE_DATA_PROTECT, SCSI_ADSENSE_WRITE_PROTECT)))
        }
        SCSIOP_WRITE6 | SCSIOP_WRITE | SCSIOP_WRITE12 | SCSIOP_WRITE16 => transfer(disk, cdb, data.len(), true),
        _ => Request::Done(Err(Sense(SCSI_SENSE_ILLEGAL_REQUEST, SCSI_ADSENSE_ILLEGAL_COMMAND))),
    }
}

fn transfer(disk: &Disk, cdb: &[u8; 16], len: usize, write: bool) -> Request {
    let (lba, blocks) = extent(cdb);
    if blocks == 0 {
        return Request::Done(Ok(0));
    }
    if lba.checked_add(blocks).is_none_or(|end| end > disk.blocks) {
        return Request::Done(Err(Sense(SCSI_SENSE_ILLEGAL_REQUEST, SCSI_ADSENSE_ILLEGAL_BLOCK)));
    }
    if blocks.saturating_mul(disk.block_size as u64) > len as u64 {
        return Request::Done(Err(Sense(SCSI_SENSE_ILLEGAL_REQUEST, SCSI_ADSENSE_INVALID_CDB)));
    }
    if write {
        Request::Write { lba, blocks }
    } else {
        Request::Read { lba, blocks }
    }
}

/// The sense a failed read or write reports.
pub fn media_error(write: bool) -> Sense {
    Sense(SCSI_SENSE_MEDIUM_ERROR, if write { SCSI_ADSENSE_WRITE_ERROR } else { SCSI_ADSENSE_UNRECOVERED_ERROR })
}

/// Record `result` in the SRB (status, length, autosense). Returns the IRP status.
///
/// # Safety
/// `srb` must be a live SRB whose SenseInfoBuffer, when set, holds
/// SenseInfoBufferLength bytes.
pub unsafe fn finish(srb: *mut Srb, result: Result<usize, Sense>) -> i32 {
    let srb = &mut *srb;
    match result {
        Ok(n) => {
            srb.ScsiStatus = SCSISTAT_GOOD;
            srb.SrbStatus = if n < srb.DataTransferLength as usize {
                srb.DataTransferLength = n as u32;
                SRB_STATUS_DATA_OVERRUN // an underrun: classpnp takes the shorter length
            } else {
                SRB_STATUS_SUCCESS
            };
            STATUS_SUCCESS
        }
        Err(Sense(key, asc)) => {
            srb.ScsiStatus = SCSISTAT_CHECK_CONDITION;
            srb.SrbStatus = SRB_STATUS_ERROR;
            srb.DataTransferLength = 0;
            let sense = srb.SenseInfoBuffer as *mut u8;
            let room = srb.SenseInfoBufferLength as usize;
            if !sense.is_null() && room > 0 && srb.SrbFlags & SRB_FLAGS_DISABLE_AUTOSENSE == 0 {
                let mut s = [0u8; 18];
                s[0] = 0x70; // current error, fixed format
                s[2] = key;
                s[7] = 10;
                s[12] = asc;
                core::ptr::copy_nonoverlapping(s.as_ptr(), sense, room.min(s.len()));
                srb.SrbStatus |= SRB_STATUS_AUTOSENSE_VALID;
            }
            STATUS_IO_DEVICE_ERROR
        }
    }
}

// ---------------------------------------------------------------------------
// IRP_MJ_DEVICE_CONTROL on the disk PDO
// ---------------------------------------------------------------------------

pub const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D_1400;
pub const IOCTL_SCSI_GET_CAPABILITIES: u32 = 0x0004_1010;
pub const IOCTL_SCSI_GET_ADDRESS: u32 = 0x0004_1018;
const STORAGE_DEVICE_PROPERTY: u32 = 0;
const STORAGE_ADAPTER_PROPERTY: u32 = 1;
const PROPERTY_STANDARD_QUERY: u32 = 0;
const PROPERTY_EXISTS_QUERY: u32 = 1;
/// STORAGE_ADAPTER_DESCRIPTOR_WIN8, through AddressType.
const ADAPTER_DESCRIPTOR_SIZE: usize = 32;
/// STORAGE_DEVICE_DESCRIPTOR up to RawDeviceProperties, rounded; the strings follow.
const DEVICE_DESCRIPTOR_SIZE: usize = 40;
const SCSI_CAPABILITIES_SIZE: usize = 24;
const PAGE: u32 = 4096;

fn put32(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

fn put16(b: &mut [u8], at: usize, v: u16) {
    b[at..at + 2].copy_from_slice(&v.to_le_bytes());
}

fn le32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

/// Answer a storage IOCTL. `buf` is the IRP's system buffer, at least
/// max(in_len, out_len) bytes. Returns (status, bytes written).
pub fn device_control(disk: &Disk, code: u32, buf: &mut [u8], in_len: usize, out_len: usize) -> (i32, usize) {
    match code {
        IOCTL_STORAGE_QUERY_PROPERTY => {
            if in_len < 8 || buf.len() < 8 {
                return (STATUS_INVALID_PARAMETER, 0);
            }
            let (id, kind) = (le32(&buf[0..4]), le32(&buf[4..8]));
            if id != STORAGE_DEVICE_PROPERTY && id != STORAGE_ADAPTER_PROPERTY {
                return (STATUS_NOT_SUPPORTED, 0);
            }
            match kind {
                PROPERTY_EXISTS_QUERY => return (STATUS_SUCCESS, 0),
                PROPERTY_STANDARD_QUERY => {}
                _ => return (STATUS_NOT_SUPPORTED, 0),
            }
            let mut d = [0u8; 128];
            let total = if id == STORAGE_ADAPTER_PROPERTY {
                put32(&mut d, 0, ADAPTER_DESCRIPTOR_SIZE as u32);
                put32(&mut d, 4, ADAPTER_DESCRIPTOR_SIZE as u32);
                put32(&mut d, 8, disk.max_transfer);
                put32(&mut d, 12, disk.max_transfer / PAGE + 1); // MaximumPhysicalPages
                put32(&mut d, 16, 0); // AlignmentMask: the drivers copy any alignment
                d[20] = 1; // AdapterUsesPio: they read the buffer through a mapping
                d[24] = disk.bus_type;
                put16(&mut d, 26, disk.bus_version.0);
                put16(&mut d, 28, disk.bus_version.1);
                // SrbType 0: SCSI_REQUEST_BLOCK, not STORAGE_REQUEST_BLOCK.
                ADAPTER_DESCRIPTOR_SIZE
            } else {
                // Vendor, product, revision and serial strings after the header.
                let mut at = DEVICE_DESCRIPTOR_SIZE;
                let mut string = |d: &mut [u8; 128], s: &[u8], field: usize| {
                    put32(d, field, at as u32);
                    d[at..at + s.len()].copy_from_slice(s);
                    at += s.len() + 1;
                };
                string(&mut d, trimmed(&disk.vendor), 12);
                string(&mut d, trimmed(&disk.product), 16);
                string(&mut d, trimmed(&disk.revision), 20);
                string(&mut d, trimmed(&disk.serial), 24);
                put32(&mut d, 0, DEVICE_DESCRIPTOR_SIZE as u32); // Version
                put32(&mut d, 4, at as u32); // Size
                d[8] = 0x00; // direct access
                d[10] = disk.removable as u8;
                put32(&mut d, 28, disk.bus_type as u32);
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
                buf[..8].copy_from_slice(&d[..8]);
                return (STATUS_SUCCESS, 8);
            }
            buf[..total].copy_from_slice(&d[..total]);
            (STATUS_SUCCESS, total)
        }
        IOCTL_SCSI_GET_ADDRESS => {
            if out_len < 8 || buf.len() < 8 {
                return (STATUS_BUFFER_TOO_SMALL, 0);
            }
            put32(buf, 0, 8);
            buf[4..8].copy_from_slice(&disk.address);
            (STATUS_SUCCESS, 8)
        }
        IOCTL_SCSI_GET_CAPABILITIES => {
            if out_len < SCSI_CAPABILITIES_SIZE || buf.len() < SCSI_CAPABILITIES_SIZE {
                return (STATUS_BUFFER_TOO_SMALL, 0);
            }
            let c = &mut buf[..SCSI_CAPABILITIES_SIZE];
            c.fill(0);
            put32(c, 0, SCSI_CAPABILITIES_SIZE as u32);
            put32(c, 4, disk.max_transfer);
            put32(c, 8, disk.max_transfer / PAGE + 1);
            c[22] = 1; // AdapterUsesPio
            (STATUS_SUCCESS, SCSI_CAPABILITIES_SIZE)
        }
        _ => (STATUS_NOT_SUPPORTED, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disk() -> Disk {
        let mut d = Disk {
            vendor: [b' '; 8],
            product: [b' '; 16],
            revision: [b' '; 4],
            serial: [b' '; 20],
            block_size: 512,
            blocks: 0x1_0000_0000 + 7, // past READ CAPACITY(10)'s reach
            removable: false,
            writable: true,
            bus_type: BUS_TYPE_NVME,
            bus_version: (1, 4),
            max_transfer: 128 * 1024,
            address: [2, 0, 0, 1],
        };
        ascii_field(&mut d.vendor, b"NVMe");
        ascii_field(&mut d.product, b"QEMU NVMe Ctrl\x01 and more");
        ascii_field(&mut d.revision, b"8.2.2");
        ascii_field(&mut d.serial, b"deadbeef");
        d
    }

    fn cdb(bytes: &[u8]) -> [u8; 16] {
        let mut c = [0u8; 16];
        c[..bytes.len()].copy_from_slice(bytes);
        c
    }

    #[test]
    fn inquiry_carries_the_padded_strings() {
        let mut data = [0u8; 96];
        assert_eq!(decode(&disk(), &cdb(&[SCSIOP_INQUIRY]), &mut data), Request::Done(Ok(36)));
        assert_eq!(&data[8..16], b"NVMe    ");
        assert_eq!(&data[16..32], b"QEMU NVMe Ctrl_ ");
        assert_eq!(&data[32..36], b"8.2.");
        assert_eq!(data[1], 0, "an NVMe namespace is not removable");
    }

    #[test]
    fn capacity_saturates_at_ten_and_is_exact_at_sixteen() {
        let mut data = [0u8; 32];
        decode(&disk(), &cdb(&[SCSIOP_READ_CAPACITY]), &mut data);
        assert_eq!(&data[..8], &[0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 2, 0]);
        decode(&disk(), &cdb(&[SCSIOP_SERVICE_ACTION_IN16, SERVICE_ACTION_READ_CAPACITY16]), &mut data);
        assert_eq!(be(&data[..8]), 0x1_0000_0006);
        assert_eq!(be(&data[8..12]), 512);
    }

    #[test]
    fn every_read_and_write_cdb_size_addresses_the_same_extent() {
        let mut data = [0u8; 4 * 512];
        let read10 = cdb(&[SCSIOP_READ, 0, 0, 0, 0x12, 0x34, 0, 0, 4]);
        let read16 = cdb(&[SCSIOP_READ16, 0, 0, 0, 0, 0, 0, 0, 0x12, 0x34, 0, 0, 0, 4]);
        let read12 = cdb(&[SCSIOP_READ12, 0, 0, 0, 0x12, 0x34, 0, 0, 0, 4]);
        let read6 = cdb(&[SCSIOP_READ6, 0, 0x12, 0x34, 4]);
        for c in [read10, read16, read12, read6] {
            assert_eq!(decode(&disk(), &c, &mut data), Request::Read { lba: 0x1234, blocks: 4 });
        }
        let write16 = cdb(&[SCSIOP_WRITE16, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(decode(&disk(), &write16, &mut data), Request::Write { lba: 0x1_0000_0000, blocks: 1 });
    }

    #[test]
    fn transfers_past_the_end_or_the_buffer_are_refused() {
        let mut data = [0u8; 512];
        let end = disk().blocks;
        let past = cdb(&[SCSIOP_READ16, 0, 0, 0, 0, 1, 0, 0, 0, 7, 0, 0, 0, 1]);
        assert_eq!(end, 0x1_0000_0007);
        assert_eq!(
            decode(&disk(), &past, &mut data),
            Request::Done(Err(Sense(SCSI_SENSE_ILLEGAL_REQUEST, SCSI_ADSENSE_ILLEGAL_BLOCK)))
        );
        let two = cdb(&[SCSIOP_READ, 0, 0, 0, 0, 0, 0, 0, 2]);
        assert_eq!(
            decode(&disk(), &two, &mut data),
            Request::Done(Err(Sense(SCSI_SENSE_ILLEGAL_REQUEST, SCSI_ADSENSE_INVALID_CDB)))
        );
    }

    #[test]
    fn a_read_only_disk_refuses_writes_and_says_so_in_mode_sense() {
        let ro = Disk { writable: false, ..disk() };
        let mut data = [0u8; 512];
        let write = cdb(&[SCSIOP_WRITE, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(
            decode(&ro, &write, &mut data),
            Request::Done(Err(Sense(SCSI_SENSE_DATA_PROTECT, SCSI_ADSENSE_WRITE_PROTECT)))
        );
        decode(&ro, &cdb(&[SCSIOP_MODE_SENSE]), &mut data);
        assert_eq!(data[2], MODE_DSP_WRITE_PROTECT);
        decode(&disk(), &cdb(&[SCSIOP_MODE_SENSE]), &mut data);
        assert_eq!(data[2], 0);
    }

    #[test]
    fn synchronize_cache_is_a_flush() {
        assert_eq!(decode(&disk(), &cdb(&[SCSIOP_SYNCHRONIZE_CACHE]), &mut []), Request::Flush);
    }

    #[test]
    fn the_device_descriptor_carries_trimmed_strings() {
        let mut buf = [0u8; 256];
        buf[..8].copy_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);
        let (status, n) = device_control(&disk(), IOCTL_STORAGE_QUERY_PROPERTY, &mut buf, 12, 256);
        assert_eq!(status, STATUS_SUCCESS);
        assert_eq!(le32(&buf[4..8]) as usize, n);
        let string = |field: usize| {
            let at = le32(&buf[field..field + 4]) as usize;
            let end = at + buf[at..].iter().position(|&b| b == 0).unwrap();
            buf[at..end].to_vec()
        };
        assert_eq!(string(12), b"NVMe");
        assert_eq!(string(16), b"QEMU NVMe Ctrl_");
        assert_eq!(string(20), b"8.2.");
        assert_eq!(string(24), b"deadbeef");
        assert_eq!(le32(&buf[28..32]), BUS_TYPE_NVME as u32);
    }

    #[test]
    fn a_short_buffer_gets_the_descriptor_size() {
        let mut buf = [0u8; 8];
        buf[0] = 1; // adapter property
        let (status, n) = device_control(&disk(), IOCTL_STORAGE_QUERY_PROPERTY, &mut buf, 8, 8);
        assert_eq!((status, n), (STATUS_SUCCESS, 8));
        assert_eq!(le32(&buf[4..8]) as usize, ADAPTER_DESCRIPTOR_SIZE);
    }

    #[test]
    fn the_scsi_address_is_the_disks() {
        let mut buf = [0u8; 8];
        assert_eq!(device_control(&disk(), IOCTL_SCSI_GET_ADDRESS, &mut buf, 0, 8), (STATUS_SUCCESS, 8));
        assert_eq!(buf, [8, 0, 0, 0, 2, 0, 0, 1]);
    }

    #[test]
    fn finish_reports_underruns_and_autosense() {
        let mut sense = [0u8; 18];
        let mut srb: Srb = unsafe { core::mem::zeroed() };
        srb.DataTransferLength = 96;
        srb.SenseInfoBuffer = sense.as_mut_ptr().cast();
        srb.SenseInfoBufferLength = 18;
        assert_eq!(unsafe { finish(&mut srb, Ok(36)) }, STATUS_SUCCESS);
        assert_eq!((srb.SrbStatus, srb.DataTransferLength), (SRB_STATUS_DATA_OVERRUN, 36));
        let status = unsafe { finish(&mut srb, Err(media_error(true))) };
        assert_eq!(status, STATUS_IO_DEVICE_ERROR);
        assert_eq!(srb.SrbStatus, SRB_STATUS_ERROR | SRB_STATUS_AUTOSENSE_VALID);
        assert_eq!((sense[2], sense[12]), (SCSI_SENSE_MEDIUM_ERROR, SCSI_ADSENSE_WRITE_ERROR));
    }
}
