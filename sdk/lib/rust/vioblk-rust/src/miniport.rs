//! The SCSIPORT miniport: one SCSI disk (bus 0, target 0, LUN 0) per virtio-blk
//! PCI function.
//!
//! Reads and writes are interrupt-driven: HwStartIo puts the request on the virtqueue
//! and returns, and HwInterrupt completes it when the device reports the used buffer.
//! One transfer is in flight per adapter — the arena it is staged through is per
//! adapter — so the NextRequest notification goes out only at completion, and
//! SCSIPORT holds every other request until then. Everything else (INQUIRY, the
//! capacity and mode pages, flush) completes inside HwStartIo.

use core::fmt::Write as _;
use core::mem::size_of;
use core::ptr;

use virtio_drivers::device::blk::{BlkReq, BlkResp, VirtIOBlk, SECTOR_SIZE};
use virtio_drivers::transport::pci::{bus::PciRoot, PciTransport};
use virtio_drivers::transport::pci::bus::ConfigurationAccess;

use crate::hal::{alloc_contiguous, init_bounce_pool, register_arena, VioHal};
use crate::pci::{device_function, enable_device, HalPciAccess};
use crate::types::*;
use ntkrnl_sys::base::{BOOLEAN, PCHAR, PVOID, UCHAR, ULONG};

const VIRTIO_VENDOR: u32 = 0x1AF4;
/// Transitional virtio-blk (QEMU's default on `-M pc`), and the modern-only id.
const VIRTIO_BLK_TRANSITIONAL: u32 = 0x1001;
const VIRTIO_BLK_MODERN: u32 = 0x1042;

/// Every transfer is staged through a per-adapter contiguous arena of this size, which
/// is therefore also the largest transfer the port driver may send us.
const ARENA_LEN: usize = 64 * 1024;
/// The arena's tail past the data: the in-flight request's header (16 bytes) and status
/// byte live there, so `share` resolves them as arena slices instead of bouncing them.
const ARENA_TAIL: usize = 64;

/// HwStartIo's verdict for a request now on the virtqueue: HwInterrupt completes it.
const PENDING: u32 = u32::MAX;

/// `_INTERFACE_TYPE_PCIBus`.
const PCI_BUS: i32 = 5;
const TRUE: BOOLEAN = 1;
const FALSE: BOOLEAN = 0;

mod ffi {
    use super::*;
    // cdecl varargs (srb.h: `SCSIPORT_API VOID __cdecl ScsiPortNotification(...)`),
    // declared here because a variadic in the substrate's `extern "system"` block is
    // not the ABI scsiport exports on i386.
    extern "C" {
        pub fn ScsiPortNotification(NotificationType: i32, HwDeviceExtension: PVOID, ...);
    }
    extern "system" {
        pub fn ScsiPortInitialize(
            DriverObject: PVOID,
            Argument2: PVOID,
            HwInitializationData: *mut HW_INITIALIZATION_DATA,
            HwContext: PVOID,
        ) -> ULONG;
        pub fn ScsiPortStallExecution(Delay: ULONG);
        pub fn ScsiPortCompleteRequest(
            HwDeviceExtension: PVOID,
            PathId: UCHAR,
            TargetId: UCHAR,
            Lun: UCHAR,
            SrbStatus: UCHAR,
        );
    }
}

/// Lives in the scsiport device extension (non-paged, one per adapter).
pub struct Adapter {
    blk: Option<VirtIOBlk<VioHal, PciTransport>>,
    arena: *mut u8,
    req: *mut BlkReq,
    resp: *mut BlkResp,
    sectors: u64,
    readonly: bool,
    io: Option<Io>,
}

/// The transfer in flight. A request larger than the arena goes as several chunks,
/// each submitted when the previous one completes.
struct Io {
    srb: PSCSI_REQUEST_BLOCK,
    write: bool,
    /// First sector and length of the whole request.
    lba: u64,
    bytes: usize,
    /// Bytes finished before the chunk in flight, and that chunk's length and token.
    done: usize,
    chunk: usize,
    token: u16,
}

// ---- logging: one DbgPrint line, formatted into a stack buffer ----------------------

struct Line {
    buf: [u8; 192],
    len: usize,
}

impl core::fmt::Write for Line {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let room = self.buf.len() - 1 - self.len;
        let n = s.len().min(room);
        self.buf[self.len..self.len + n].copy_from_slice(&s.as_bytes()[..n]);
        self.len += n;
        Ok(())
    }
}

extern "C" {
    fn DbgPrint(format: *const u8, ...) -> u32;
}

pub(crate) fn log(args: core::fmt::Arguments) {
    let mut line = Line { buf: [0; 192], len: 0 };
    let _ = line.write_fmt(args);
    line.buf[line.len] = 0;
    unsafe {
        DbgPrint(b"[VIOBLK] %s\n\0".as_ptr(), line.buf.as_ptr());
    }
}

macro_rules! vlog {
    ($($t:tt)*) => { $crate::miniport::log(format_args!($($t)*)) };
}
pub(crate) use vlog;

// ---- DriverEntry ----------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(DriverObject: PVOID, RegistryPath: PVOID) -> ULONG {
    let mut status = u32::MAX;
    for device_id in [b"1001", b"1042"] {
        let mut init: HW_INITIALIZATION_DATA = core::mem::zeroed();
        init.HwInitializationDataSize = size_of::<HW_INITIALIZATION_DATA>() as ULONG;
        init.AdapterInterfaceType = PCI_BUS as _;
        init.HwInitialize = Some(hw_initialize);
        init.HwStartIo = Some(hw_start_io);
        init.HwInterrupt = Some(hw_interrupt);
        init.HwFindAdapter = Some(hw_find_adapter);
        init.HwResetBus = Some(hw_reset_bus);
        init.DeviceExtensionSize = size_of::<Adapter>() as ULONG;
        init.MapBuffers = TRUE;
        init.VendorId = b"1af4".as_ptr() as PVOID;
        init.VendorIdLength = 4;
        init.DeviceId = device_id.as_ptr() as PVOID;
        init.DeviceIdLength = 4;
        let s = ffi::ScsiPortInitialize(DriverObject, RegistryPath, &mut init, ptr::null_mut());
        // STATUS_NO_SUCH_DEVICE just means no function with this id is present.
        if (s as i32) < 0 && s != 0xC000_00C0 {
            vlog!("ScsiPortInitialize(1af4:{}) = {:#x}", core::str::from_utf8(device_id).unwrap_or("?"), s);
        }
        // SCSIPORT convention: a driver that registers for several hardware IDs
        // returns the LOWEST status, i.e. success if any registration succeeded.
        status = status.min(s);
    }
    // virtio-mmio disks are PnP children of the arm64 HAL's ACPI root, which
    // only that HAL publishes; the driver must then stay loaded even when no
    // PCI function matched, to take them when the PnP manager offers them.
    #[cfg(target_arch = "aarch64")]
    {
        crate::pnp::install(DriverObject.cast());
        status = 0;
    }
    status
}

// ---- HwFindAdapter --------------------------------------------------------------------

unsafe extern "system" fn hw_find_adapter(
    DeviceExtension: PVOID,
    _HwContext: PVOID,
    _BusInformation: PVOID,
    _ArgumentString: PCHAR,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    Again: *mut BOOLEAN,
) -> ULONG {
    // TRUE: keep scanning. Each virtio-blk PCI function is its own adapter, and with
    // FALSE scsiport stopped after the first — the boot ISO, the second function, was
    // never offered.
    *Again = TRUE;
    let ext = DeviceExtension as *mut Adapter;
    ptr::write(
        ext,
        Adapter {
            blk: None,
            arena: ptr::null_mut(),
            req: ptr::null_mut(),
            resp: ptr::null_mut(),
            sectors: 0,
            readonly: false,
            io: None,
        },
    );
    let cfg = &mut *ConfigInfo;
    let df = device_function(cfg.SystemIoBusNumber, cfg.SlotNumber);

    let id = HalPciAccess.read_word(df, 0x00);
    let (vendor, device) = (id & 0xFFFF, id >> 16);
    if vendor != VIRTIO_VENDOR || (device != VIRTIO_BLK_TRANSITIONAL && device != VIRTIO_BLK_MODERN) {
        return SP_RETURN_NOT_FOUND;
    }
    if !init_bounce_pool() {
        vlog!("{:02x}:{:02x}.{}: no memory for the bounce pool", df.bus, df.device, df.function);
        return SP_RETURN_ERROR;
    }
    enable_device(df);
    // Adapters are found one at a time at init, so a per-adapter reset is race-free.
    crate::hal::MAP_FAILED.store(false, core::sync::atomic::Ordering::Release);
    let mut root = PciRoot::new(HalPciAccess);
    let transport = match PciTransport::new::<VioHal, _>(&mut root, df) {
        Ok(t) => t,
        Err(e) => {
            vlog!("{:02x}:{:02x}.{}: virtio PCI transport: {:?}", df.bus, df.device, df.function, e);
            return SP_RETURN_ERROR;
        }
    };
    if crate::hal::MAP_FAILED.load(core::sync::atomic::Ordering::Acquire) {
        vlog!("{:02x}:{:02x}.{}: a BAR could not be mapped (MmMapIoSpace)", df.bus, df.device, df.function);
        return SP_RETURN_ERROR;
    }
    let mut blk = match VirtIOBlk::<VioHal, _>::new(transport) {
        Ok(b) => b,
        Err(e) => {
            vlog!("{:02x}:{:02x}.{}: virtio-blk init: {:?}", df.bus, df.device, df.function, e);
            return SP_RETURN_ERROR;
        }
    };
    // Quiet until HwInitialize: SCSIPORT connects the interrupt after this returns.
    blk.disable_interrupts();
    let (arena, arena_pa) = alloc_contiguous(ARENA_LEN + ARENA_TAIL);
    if arena.is_null() || !register_arena(arena, arena_pa, ARENA_LEN + ARENA_TAIL) {
        vlog!("{:02x}:{:02x}.{}: no transfer arena", df.bus, df.device, df.function);
        return SP_RETURN_ERROR;
    }
    let a = &mut *ext;
    a.sectors = blk.capacity();
    a.readonly = blk.readonly();
    a.arena = arena;
    a.req = arena.add(ARENA_LEN) as *mut BlkReq;
    a.resp = arena.add(ARENA_LEN + size_of::<BlkReq>()) as *mut BlkResp;
    ptr::write(a.req, BlkReq::default());
    ptr::write(a.resp, BlkResp::default());
    vlog!(
        "{:02x}:{:02x}.{} dev {:04x}: {} sectors ({} MiB){}",
        df.bus,
        df.device,
        df.function,
        device,
        a.sectors,
        a.sectors * SECTOR_SIZE as u64 >> 20,
        if a.readonly { ", read-only" } else { "" }
    );
    a.blk = Some(blk);

    cfg.MaximumTransferLength = ARENA_LEN as ULONG;
    cfg.NumberOfPhysicalBreaks = (ARENA_LEN / crate::hal::PAGE_SIZE) as ULONG;
    cfg.AlignmentMask = 0;
    cfg.NumberOfBuses = 1;
    cfg.MaximumNumberOfTargets = 1;
    // SCSIPORT scans LUNs below this and does not default it (C or Rust); left at 0
    // the bus scan skipped every LUN, never sent INQUIRY, and found no disk.
    cfg.MaximumNumberOfLogicalUnits = 1;
    cfg.InitiatorBusId[0] = 7;
    cfg.ScatterGather = FALSE;
    cfg.Master = FALSE;
    cfg.CachesData = FALSE;
    cfg.MapBuffers = TRUE;
    cfg.NeedPhysicalAddresses = FALSE;
    SP_RETURN_FOUND
}

/// Runs once SCSIPORT has connected the interrupt, so the device may now raise it.
unsafe extern "system" fn hw_initialize(DeviceExtension: PVOID) -> BOOLEAN {
    let a = &mut *(DeviceExtension as *mut Adapter);
    if let Some(blk) = a.blk.as_mut() {
        blk.enable_interrupts();
    }
    TRUE
}

/// Acknowledge, claim the interrupt if it was ours, and complete the transfer it
/// reports.
///
/// The ISR status read is the acknowledgement: a virtio INTx stays asserted until it
/// is read, and an HwInterrupt that only answered "not mine" left the level-triggered
/// line asserted and hung the boot in an interrupt storm (`Isr entry count=100000`).
/// An empty status means another device on a shared line. SCSIPORT runs this and
/// HwStartIo under the same interrupt lock.
unsafe extern "system" fn hw_interrupt(DeviceExtension: PVOID) -> BOOLEAN {
    let a = &mut *(DeviceExtension as *mut Adapter);
    let Some(blk) = a.blk.as_mut() else {
        return FALSE;
    };
    if blk.ack_interrupt().is_empty() {
        return FALSE;
    }
    service(a, DeviceExtension);
    TRUE
}

/// Advance the transfer in flight if the device has finished its chunk.
unsafe fn service(a: &mut Adapter, ext: PVOID) {
    let status = {
        let (Some(io), Some(blk)) = (a.io.as_mut(), a.blk.as_mut()) else {
            return;
        };
        if blk.peek_used() != Some(io.token) {
            return;
        }
        let stage = core::slice::from_raw_parts_mut(a.arena, io.chunk);
        let r = if io.write {
            blk.complete_write_blocks(io.token, &*a.req, stage, &mut *a.resp)
        } else {
            blk.complete_read_blocks(io.token, &*a.req, stage, &mut *a.resp)
        };
        match r {
            Err(e) => {
                vlog!(
                    "{} of {} sectors at lba {} failed: {:?}",
                    if io.write { "write" } else { "read" },
                    io.chunk / SECTOR_SIZE,
                    io.lba as usize + io.done / SECTOR_SIZE,
                    e
                );
                Some(SRB_STATUS_ERROR)
            }
            Ok(()) => {
                if !io.write {
                    let user = (*io.srb).DataBuffer as *mut u8;
                    ptr::copy_nonoverlapping(a.arena, user.add(io.done), io.chunk);
                }
                io.done += io.chunk;
                if io.done == io.bytes {
                    (*io.srb).DataTransferLength = io.bytes as ULONG;
                    Some(SRB_STATUS_SUCCESS)
                } else {
                    None
                }
            }
        }
    };
    match status {
        Some(st) => finish(a, ext, st),
        None => {
            if !submit(a) {
                finish(a, ext, SRB_STATUS_ERROR);
            }
        }
    }
}

/// Put the next chunk of the transfer in flight on the virtqueue.
unsafe fn submit(a: &mut Adapter) -> bool {
    let (Some(io), Some(blk)) = (a.io.as_mut(), a.blk.as_mut()) else {
        return false;
    };
    let n = (io.bytes - io.done).min(ARENA_LEN);
    let stage = core::slice::from_raw_parts_mut(a.arena, n);
    let block = io.lba as usize + io.done / SECTOR_SIZE;
    let r = if io.write {
        let user = (*io.srb).DataBuffer as *const u8;
        ptr::copy_nonoverlapping(user.add(io.done), a.arena, n);
        blk.write_blocks_nb(block, &mut *a.req, stage, &mut *a.resp)
    } else {
        blk.read_blocks_nb(block, &mut *a.req, stage, &mut *a.resp)
    };
    match r {
        Ok(token) => {
            io.token = token;
            io.chunk = n;
            true
        }
        Err(e) => {
            vlog!("submit {} sectors at lba {}: {:?}", n / SECTOR_SIZE, block, e);
            false
        }
    }
}

/// Complete the transfer in flight and let SCSIPORT send the next request.
unsafe fn finish(a: &mut Adapter, ext: PVOID, status: u32) {
    let Some(io) = a.io.take() else {
        return;
    };
    (*io.srb).SrbStatus = status as UCHAR;
    trace_failure(&*io.srb);
    ffi::ScsiPortNotification(_SCSI_NOTIFICATION_TYPE_RequestComplete as i32, ext, io.srb);
    ffi::ScsiPortNotification(_SCSI_NOTIFICATION_TYPE_NextRequest as i32, ext, ptr::null_mut::<u8>());
}

/// How long a bus reset waits for a transfer the device still owns: 1 s in 10 us
/// steps, well past any virtio-blk request and well inside the SRB timeout that led here.
const RESET_POLLS: u32 = 100_000;

unsafe extern "system" fn hw_reset_bus(DeviceExtension: PVOID, PathId: ULONG) -> BOOLEAN {
    let a = &mut *(DeviceExtension as *mut Adapter);
    // The SRB timed out, so its interrupt never came. The device still owns the arena
    // and would DMA into the next request's data if the SRB were simply completed, so
    // first poll for the completion the interrupt should have delivered.
    if a.io.is_some() {
        let mut polls = 0;
        while a.io.is_some() && polls < RESET_POLLS {
            service(a, DeviceExtension);
            ffi::ScsiPortStallExecution(10);
            polls += 1;
        }
        if a.io.is_none() {
            vlog!("bus reset: the transfer had completed but its interrupt never arrived");
        } else {
            // Still owned by the device after a second: retire the adapter rather than
            // reuse an arena it may yet write. Forgotten, not dropped: dropping frees
            // the queue, and contiguous memory cannot be freed at this IRQL.
            vlog!("bus reset: the device never completed the transfer; adapter disabled");
            a.io = None;
            core::mem::forget(a.blk.take());
        }
    }
    ffi::ScsiPortCompleteRequest(DeviceExtension, PathId as UCHAR, 0xFF, 0xFF, SRB_STATUS_BUS_RESET as UCHAR);
    TRUE
}

/// Requests that do not succeed are logged, up to a budget per boot. classpnp probes
/// for optional pages and descriptors, so a handful of rejections is normal.
static TRACE_LEFT: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(16);

// ---- HwStartIo ------------------------------------------------------------------------

unsafe extern "system" fn hw_start_io(DeviceExtension: PVOID, Srb: PSCSI_REQUEST_BLOCK) -> BOOLEAN {
    let a = &mut *(DeviceExtension as *mut Adapter);
    let srb = &mut *Srb;
    let status = match srb.Function as u32 {
        // SCSIPORT sends nothing else before NextRequest; only a reset or an abort for
        // the request in flight can still arrive, and those are answered without
        // touching the device.
        _ if a.io.is_some() => match srb.Function as u32 {
            SRB_FUNCTION_RESET_BUS | SRB_FUNCTION_RESET_DEVICE => SRB_STATUS_SUCCESS,
            SRB_FUNCTION_ABORT_COMMAND => SRB_STATUS_ABORT_FAILED,
            _ => SRB_STATUS_BUSY,
        },
        SRB_FUNCTION_EXECUTE_SCSI => {
            if srb.PathId != 0 || srb.TargetId != 0 {
                SRB_STATUS_SELECTION_TIMEOUT
            } else if srb.Lun != 0 {
                SRB_STATUS_INVALID_LUN
            } else {
                execute(a, srb)
            }
        }
        SRB_FUNCTION_FLUSH | SRB_FUNCTION_SHUTDOWN => flush(a),
        SRB_FUNCTION_RESET_BUS | SRB_FUNCTION_RESET_DEVICE => SRB_STATUS_SUCCESS,
        _ => SRB_STATUS_INVALID_REQUEST,
    };
    if status == PENDING {
        return TRUE;
    }
    srb.SrbStatus = status as UCHAR;
    trace_failure(srb);
    ffi::ScsiPortNotification(_SCSI_NOTIFICATION_TYPE_RequestComplete as i32, DeviceExtension, Srb);
    ffi::ScsiPortNotification(_SCSI_NOTIFICATION_TYPE_NextRequest as i32, DeviceExtension, ptr::null_mut::<u8>());
    TRUE
}

fn trace_failure(srb: &SCSI_REQUEST_BLOCK) {
    if srb.SrbStatus as u32 != SRB_STATUS_SUCCESS
        && TRACE_LEFT.fetch_sub(1, core::sync::atomic::Ordering::Relaxed) > 0
    {
        vlog!(
            "srb fn={} path={} tid={} lun={} cdb0={:#04x} len={} -> status {:#04x}",
            srb.Function, srb.PathId, srb.TargetId, srb.Lun, srb.Cdb[0], srb.DataTransferLength, srb.SrbStatus
        );
    }
}

fn be16(b: &[u8]) -> u64 {
    u16::from_be_bytes([b[0], b[1]]) as u64
}
fn be32(b: &[u8]) -> u64 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as u64
}
fn be64(b: &[u8]) -> u64 {
    u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
}

/// Copy `reply` into the SRB's buffer, truncated to what the caller allotted, and record
/// how much was returned.
unsafe fn reply(srb: &mut SCSI_REQUEST_BLOCK, reply: &[u8]) -> u32 {
    let n = reply.len().min(srb.DataTransferLength as usize);
    if n > 0 && srb.DataBuffer.is_null() {
        return SRB_STATUS_ERROR;
    }
    ptr::copy_nonoverlapping(reply.as_ptr(), srb.DataBuffer as *mut u8, n);
    srb.DataTransferLength = n as ULONG;
    SRB_STATUS_SUCCESS
}

unsafe fn execute(a: &mut Adapter, srb: &mut SCSI_REQUEST_BLOCK) -> u32 {
    let cdb = srb.Cdb;
    match cdb[0] as u32 {
        SCSIOP_TEST_UNIT_READY | SCSIOP_VERIFY | SCSIOP_START_STOP_UNIT | SCSIOP_MEDIUM_REMOVAL => {
            srb.DataTransferLength = 0;
            SRB_STATUS_SUCCESS
        }
        SCSIOP_SYNCHRONIZE_CACHE => flush(a),
        SCSIOP_INQUIRY => {
            if cdb[1] & 1 != 0 {
                // No vital product data pages; classpnp copes without them.
                return SRB_STATUS_INVALID_REQUEST;
            }
            // Standard INQUIRY: direct-access device, SPC-3, 36 bytes.
            let mut d = [0u8; 36];
            d[0] = DIRECT_ACCESS_DEVICE as u8;
            d[2] = 5;
            d[3] = 2;
            d[4] = 31;
            d[8..16].copy_from_slice(b"VibeOS  ");
            d[16..32].copy_from_slice(b"VirtIO Block    ");
            d[32..36].copy_from_slice(b"1.0 ");
            reply(srb, &d)
        }
        SCSIOP_READ_CAPACITY => {
            let last = a.sectors.saturating_sub(1).min(0xFFFF_FFFF) as u32;
            let mut d = [0u8; 8];
            d[0..4].copy_from_slice(&last.to_be_bytes());
            d[4..8].copy_from_slice(&(SECTOR_SIZE as u32).to_be_bytes());
            reply(srb, &d)
        }
        SCSIOP_READ_CAPACITY16 if cdb[1] & 0x1F == 0x10 => {
            let mut d = [0u8; 32];
            d[0..8].copy_from_slice(&a.sectors.saturating_sub(1).to_be_bytes());
            d[8..12].copy_from_slice(&(SECTOR_SIZE as u32).to_be_bytes());
            reply(srb, &d)
        }
        // Headers only, no pages: disk.sys reads the write-protect bit from here.
        SCSIOP_MODE_SENSE => {
            let wp = if a.readonly { 0x80 } else { 0 };
            reply(srb, &[3, 0, wp, 0])
        }
        SCSIOP_MODE_SENSE10 => {
            let wp = if a.readonly { 0x80 } else { 0 };
            reply(srb, &[0, 6, 0, wp, 0, 0, 0, 0])
        }
        SCSIOP_READ => transfer(a, srb, be32(&cdb[2..6]), be16(&cdb[7..9]), false),
        SCSIOP_WRITE => transfer(a, srb, be32(&cdb[2..6]), be16(&cdb[7..9]), true),
        SCSIOP_READ16 => transfer(a, srb, be64(&cdb[2..10]), be32(&cdb[10..14]), false),
        SCSIOP_WRITE16 => transfer(a, srb, be64(&cdb[2..10]), be32(&cdb[10..14]), true),
        _ => SRB_STATUS_INVALID_REQUEST,
    }
}

/// Validate a READ/WRITE and put its first chunk on the virtqueue; HwInterrupt does
/// the rest.
unsafe fn transfer(a: &mut Adapter, srb: &mut SCSI_REQUEST_BLOCK, lba: u64, count: u64, write: bool) -> u32 {
    let bytes = count as usize * SECTOR_SIZE;
    if count == 0 {
        srb.DataTransferLength = 0;
        return SRB_STATUS_SUCCESS;
    }
    if lba.saturating_add(count) > a.sectors || bytes > srb.DataTransferLength as usize || srb.DataBuffer.is_null() {
        return SRB_STATUS_ERROR;
    }
    if (write && a.readonly) || a.blk.is_none() {
        return SRB_STATUS_ERROR;
    }
    a.io = Some(Io { srb: ptr::from_mut(srb), write, lba, bytes, done: 0, chunk: 0, token: 0 });
    if submit(a) {
        PENDING
    } else {
        a.io = None;
        SRB_STATUS_ERROR
    }
}

fn flush(a: &mut Adapter) -> u32 {
    match a.blk.as_mut().map(|b| b.flush()) {
        Some(Ok(())) | None => SRB_STATUS_SUCCESS,
        Some(Err(_)) => SRB_STATUS_ERROR,
    }
}
