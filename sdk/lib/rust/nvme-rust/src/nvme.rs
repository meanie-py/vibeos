//! One NVMe controller, polled: the admin queue, one I/O queue pair, and a
//! DMA bounce buffer, all in one physically contiguous allocation.
//!
//! Every command is submitted and then polled to completion before the next
//! (the worker in pnp.rs serializes them), so a queue never holds more than
//! one entry and there is no command-ID bookkeeping beyond a check. Data moves
//! through the bounce buffer: its PRP list is built once, at start, so a
//! transfer never walks an MDL for physical pages and the device only ever
//! sees memory this driver owns. Interrupts are masked (INTMS, and the I/O
//! completion queue is created with interrupts off), so an unconnected INTx
//! line can never be left asserted on a pin another device shares.
//!
//! Register and command layouts are from the NVM Express Base Specification
//! (2.0); the section numbers below are that document's.

use core::ptr::{read_volatile, write_volatile};

use ntkrnl_sys::ddk::{
    KeDelayExecutionThread, KeQueryPerformanceCounter, KeStallExecutionProcessor, MmAllocateContiguousMemorySpecifyCache,
    MmGetPhysicalAddress, _MEMORY_CACHING_TYPE_MmCached, LARGE_INTEGER, PHYSICAL_ADDRESS,
};

use crate::print::nvprint;

pub const PAGE: usize = 4096;
/// The bounce buffer, and so the most one command moves (before MDTS).
const BOUNCE: usize = 128 * 1024;
const QUEUE_ENTRIES: u16 = 16;
const SQE: usize = 64;
const CQE: usize = 16;
/// A command that has not completed in this long has failed.
const COMMAND_TIMEOUT_US: u64 = 30_000_000;
/// Spin this long for a completion before sleeping between polls.
const SPIN_US: u64 = 2_000;

// Controller registers (3.1).
const REG_CAP: usize = 0x00;
const REG_VS: usize = 0x08;
const REG_INTMS: usize = 0x0C;
const REG_CC: usize = 0x14;
const REG_CSTS: usize = 0x1C;
const REG_AQA: usize = 0x24;
const REG_ASQ: usize = 0x28;
const REG_ACQ: usize = 0x30;
const DOORBELLS: usize = 0x1000;

const CC_EN: u32 = 1;
/// I/O submission and completion entry sizes, as powers of two (64 and 16 bytes).
const CC_IOSQES_IOCQES: u32 = 6 << 16 | 4 << 20;
const CSTS_RDY: u32 = 1;
const CSTS_CFS: u32 = 2;

// Admin opcodes (5).
const ADMIN_CREATE_SQ: u8 = 0x01;
const ADMIN_CREATE_CQ: u8 = 0x05;
const ADMIN_IDENTIFY: u8 = 0x06;
const ADMIN_SET_FEATURES: u8 = 0x09;
// NVM command set opcodes.
const NVM_FLUSH: u8 = 0x00;
const NVM_WRITE: u8 = 0x01;
const NVM_READ: u8 = 0x02;

const CNS_NAMESPACE: u32 = 0x00;
const CNS_CONTROLLER: u32 = 0x01;
const CNS_ACTIVE_NAMESPACES: u32 = 0x02;
const FEATURE_VOLATILE_WRITE_CACHE: u32 = 0x06;
const FEATURE_NUMBER_OF_QUEUES: u32 = 0x07;

// The fields are read by Debug, in the log lines; dead-code analysis does not see that.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Error {
    /// The register window does not hold an NVMe controller.
    NotNvme(u32),
    /// The controller needs pages larger than 4 KiB.
    PageSize(u32),
    NoMemory,
    /// The controller did not reach the state in time.
    Timeout(&'static str),
    /// CSTS.CFS: the controller hit a fatal error.
    Fatal,
    /// A command completed with an error: status code type and code.
    Status(&'static str, u8, u8),
}

/// Microseconds since boot, from the performance counter.
pub fn now_us() -> u64 {
    let mut freq: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    let count = unsafe { KeQueryPerformanceCounter(&mut freq).QuadPart } as u64;
    let f = (unsafe { freq.QuadPart } as u64).max(1);
    // Split so the multiply cannot overflow, and no 128-bit division on i386.
    count / f * 1_000_000 + count % f * 1_000_000 / f
}

fn sleep_ms(ms: u32) {
    let mut interval: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    interval.QuadPart = -10_000 * ms as i64;
    unsafe { KeDelayExecutionThread(0, 0, &mut interval) };
}

/// Make this CPU's stores to normal memory visible to the device before a
/// doorbell write tells it to look. x86 keeps stores in order on its own; arm64
/// needs the barrier to reach the outer-shareable domain the device is in.
#[inline(always)]
fn wmb() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("dsb st", options(nostack, preserves_flags))
    };
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

/// Order the read of a completion's phase bit before the reads of its fields.
#[inline(always)]
fn rmb() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("dsb ld", options(nostack, preserves_flags))
    };
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

struct Queue {
    id: u16,
    sq: *mut u8,
    sq_phys: u64,
    cq: *mut u8,
    cq_phys: u64,
    tail: u16,
    head: u16,
    phase: u32,
}

/// What Identify Controller says about the device.
#[derive(Clone, Copy)]
pub struct Identity {
    pub vendor: u16,
    pub serial: [u8; 20],
    pub model: [u8; 40],
    pub firmware: [u8; 8],
    pub namespaces: u32,
    pub volatile_cache: bool,
    pub version: u32,
}

#[derive(Clone, Copy)]
pub struct Namespace {
    pub id: u32,
    pub blocks: u64,
    pub block_shift: u32,
}

pub struct Controller {
    regs: *mut u8,
    doorbell_stride: usize,
    ready_timeout_ms: u64,
    admin: Queue,
    io: Queue,
    /// The PRP list page and the bounce buffer it describes.
    prp_list: *mut u64,
    prp_list_phys: u64,
    bounce: *mut u8,
    bounce_phys: u64,
    next_cid: u16,
    pub max_transfer: usize,
    pub identity: Identity,
}

// The controller lives in the FDO's extension and is used only by its worker
// thread (and by START before that thread exists).
unsafe impl Send for Controller {}

/// The command's 16 dwords (4.2).
#[derive(Clone, Copy, Default)]
struct Command([u32; 16]);

impl Command {
    fn new(opcode: u8, nsid: u32) -> Self {
        let mut c = Command::default();
        c.0[0] = opcode as u32;
        c.0[1] = nsid;
        c
    }
    fn prp(mut self, prp1: u64, prp2: u64) -> Self {
        self.0[6] = prp1 as u32;
        self.0[7] = (prp1 >> 32) as u32;
        self.0[8] = prp2 as u32;
        self.0[9] = (prp2 >> 32) as u32;
        self
    }
    fn dw(mut self, index: usize, value: u32) -> Self {
        self.0[index] = value;
        self
    }
}

impl Controller {
    fn rd(&self, off: usize) -> u32 {
        unsafe { read_volatile(self.regs.add(off).cast::<u32>()) }
    }
    fn wr(&self, off: usize, v: u32) {
        unsafe { write_volatile(self.regs.add(off).cast::<u32>(), v) }
    }
    /// A 64-bit register as two dword accesses, low first (3.1: allowed for
    /// every 64-bit register, and the only option on i386).
    fn wr64(&self, off: usize, v: u64) {
        self.wr(off, v as u32);
        self.wr(off + 4, (v >> 32) as u32);
    }

    /// Reset the controller behind `regs` (a mapping of BAR0, uncached, at
    /// least `len` bytes) and bring it up with one I/O queue pair.
    ///
    /// # Safety
    /// `regs` must map the controller's registers for `len` bytes.
    pub unsafe fn start(regs: *mut u8, len: usize) -> Result<Controller, Error> {
        let cap = read_volatile(regs.add(REG_CAP).cast::<u32>()) as u64 | (read_volatile(regs.add(REG_CAP + 4).cast::<u32>()) as u64) << 32;
        let version = read_volatile(regs.add(REG_VS).cast::<u32>());
        if cap == u64::MAX || version >> 16 == 0 || version == u32::MAX {
            return Err(Error::NotNvme(version));
        }
        let mqes = (cap & 0xFFFF) as u16 + 1;
        let doorbell_stride = 4usize << ((cap >> 32) & 0xF);
        let ready_timeout_ms = ((cap >> 24) & 0xFF).max(1) * 500;
        let mps_min = ((cap >> 48) & 0xF) as u32;
        if mps_min != 0 {
            return Err(Error::PageSize(4096 << mps_min));
        }
        if DOORBELLS + 4 * doorbell_stride > len {
            return Err(Error::NotNvme(version));
        }
        let entries = QUEUE_ENTRIES.min(mqes);
        nvprint!(
            "controller: NVMe {}.{}, {} queue entries max, doorbell stride {}, ready timeout {} ms\n",
            version >> 16,
            (version >> 8) & 0xFF,
            mqes,
            doorbell_stride,
            ready_timeout_ms
        );

        // Pages: admin SQ, admin CQ, I/O SQ, I/O CQ, PRP list, then the bounce buffer.
        let size = 5 * PAGE + BOUNCE;
        let mut low: PHYSICAL_ADDRESS = core::mem::zeroed();
        let mut high: PHYSICAL_ADDRESS = core::mem::zeroed();
        let boundary: PHYSICAL_ADDRESS = core::mem::zeroed();
        low.QuadPart = 0;
        // NVMe addresses all 64 bits; i386 kernels map no more than 4 GiB.
        high.QuadPart = if cfg!(target_pointer_width = "64") { 0x0000_FFFF_FFFF_FFFF } else { 0xFFFF_FFFF };
        // Cached: PCIe DMA snoops the CPU caches on every machine this targets
        // (x86, QEMU virt, Graviton). An uncached bounce buffer would make every
        // read of it a bus transaction.
        let base = MmAllocateContiguousMemorySpecifyCache(size as _, low, high, boundary, _MEMORY_CACHING_TYPE_MmCached)
            as *mut u8;
        if base.is_null() {
            return Err(Error::NoMemory);
        }
        core::ptr::write_bytes(base, 0, 5 * PAGE);
        let phys = |p: *mut u8| MmGetPhysicalAddress(p.cast()).QuadPart as u64;
        let queue = |id, sq: *mut u8, cq: *mut u8| Queue {
            id,
            sq,
            sq_phys: phys(sq),
            cq,
            cq_phys: phys(cq),
            tail: 0,
            head: 0,
            phase: 1,
        };
        let mut c = Controller {
            regs,
            doorbell_stride,
            ready_timeout_ms,
            admin: queue(0, base, base.add(PAGE)),
            io: queue(1, base.add(2 * PAGE), base.add(3 * PAGE)),
            prp_list: base.add(4 * PAGE).cast(),
            prp_list_phys: phys(base.add(4 * PAGE)),
            bounce: base.add(5 * PAGE),
            bounce_phys: phys(base.add(5 * PAGE)),
            next_cid: 0,
            max_transfer: BOUNCE,
            identity: core::mem::zeroed(),
        };
        // The PRP list (4.1.1): entry i is the bounce buffer's page i + 1, since
        // PRP1 always names page 0. The pages are physically contiguous.
        for i in 0..BOUNCE / PAGE - 1 {
            *c.prp_list.add(i) = c.bounce_phys + ((i + 1) * PAGE) as u64;
        }

        // Disable (firmware may have left it running), then program the admin
        // queue and enable (3.5.1).
        if c.rd(REG_CC) & CC_EN != 0 {
            c.wr(REG_CC, c.rd(REG_CC) & !CC_EN);
        }
        c.wait_ready(false)?;
        // Mask every pin-based/MSI vector: nothing here takes interrupts.
        c.wr(REG_INTMS, u32::MAX);
        c.wr(REG_AQA, ((entries as u32 - 1) << 16) | (entries as u32 - 1));
        c.wr64(REG_ASQ, c.admin.sq_phys);
        c.wr64(REG_ACQ, c.admin.cq_phys);
        c.wr(REG_CC, CC_IOSQES_IOCQES | CC_EN); // NVM command set, 4 KiB pages, round robin
        c.wait_ready(true)?;

        c.identity = c.identify_controller()?;
        // MDTS (byte 77): the largest transfer, in units of the minimum page size.
        let mdts = *c.bounce.add(77);
        if mdts != 0 && (PAGE << mdts) < c.max_transfer {
            c.max_transfer = PAGE << mdts;
        }

        // One I/O queue pair; the count is a request the controller may round.
        let _ = c.admin_command("set features (queues)", Command::new(ADMIN_SET_FEATURES, 0).dw(10, FEATURE_NUMBER_OF_QUEUES));
        // Completion queue first: physically contiguous, interrupts off.
        let cq = Command::new(ADMIN_CREATE_CQ, 0)
            .prp(c.io.cq_phys, 0)
            .dw(10, ((entries as u32 - 1) << 16) | 1)
            .dw(11, 1);
        c.admin_command("create I/O completion queue", cq)?;
        let sq = Command::new(ADMIN_CREATE_SQ, 0)
            .prp(c.io.sq_phys, 0)
            .dw(10, ((entries as u32 - 1) << 16) | 1)
            .dw(11, 1 << 16 | 1);
        c.admin_command("create I/O submission queue", sq)?;

        if c.identity.volatile_cache {
            // Every completed write is then durable, whatever disk.sys does
            // about flushing: a cache-flush policy is not this driver's to get
            // wrong yet. SYNCHRONIZE CACHE still flushes, should this fail.
            let off = Command::new(ADMIN_SET_FEATURES, 0).dw(10, FEATURE_VOLATILE_WRITE_CACHE);
            if let Err(e) = c.admin_command("disable the write cache", off) {
                nvprint!("the volatile write cache stays on: {:?}\n", e);
            }
        }
        Ok(c)
    }

    fn wait_ready(&self, ready: bool) -> Result<(), Error> {
        let t0 = now_us();
        loop {
            let csts = self.rd(REG_CSTS);
            if csts == u32::MAX {
                return Err(Error::NotNvme(csts));
            }
            if ready && csts & CSTS_CFS != 0 {
                return Err(Error::Fatal);
            }
            if (csts & CSTS_RDY != 0) == ready {
                return Ok(());
            }
            if now_us() - t0 > self.ready_timeout_ms * 1000 {
                return Err(Error::Timeout(if ready { "enable" } else { "disable" }));
            }
            sleep_ms(1);
        }
    }

    fn identify_controller(&mut self) -> Result<Identity, Error> {
        let cmd = Command::new(ADMIN_IDENTIFY, 0).prp(self.bounce_phys, 0).dw(10, CNS_CONTROLLER);
        self.admin_command("identify controller", cmd)?;
        let b = |i: usize| unsafe { read_volatile(self.bounce.add(i)) };
        let mut id = Identity {
            vendor: b(0) as u16 | (b(1) as u16) << 8,
            serial: [0; 20],
            model: [0; 40],
            firmware: [0; 8],
            namespaces: u32::from_le_bytes([b(516), b(517), b(518), b(519)]),
            volatile_cache: b(525) & 1 != 0,
            version: u32::from_le_bytes([b(80), b(81), b(82), b(83)]),
        };
        for (i, s) in id.serial.iter_mut().enumerate() {
            *s = b(4 + i);
        }
        for (i, m) in id.model.iter_mut().enumerate() {
            *m = b(24 + i);
        }
        for (i, f) in id.firmware.iter_mut().enumerate() {
            *f = b(64 + i);
        }
        Ok(id)
    }

    /// The active namespaces, at most `out.len()` of them. Returns the count.
    pub fn active_namespaces(&mut self, out: &mut [u32]) -> usize {
        let cmd = Command::new(ADMIN_IDENTIFY, 0).prp(self.bounce_phys, 0).dw(10, CNS_ACTIVE_NAMESPACES);
        let mut n = 0;
        if self.admin_command("list active namespaces", cmd).is_ok() {
            // A zero-terminated list of NSIDs, ascending.
            while n < out.len() && n < PAGE / 4 {
                let id = unsafe { read_volatile(self.bounce.cast::<u32>().add(n)) };
                if id == 0 {
                    break;
                }
                out[n] = id;
                n += 1;
            }
        } else {
            // Before NVMe 1.1 there is no list: every NSID up to NN is valid.
            while n < out.len() && (n as u32) < self.identity.namespaces {
                out[n] = n as u32 + 1;
                n += 1;
            }
        }
        n
    }

    /// Identify one namespace. None when it is inactive or this driver cannot
    /// address it (metadata interleaved with the data).
    pub fn namespace(&mut self, id: u32) -> Result<Option<Namespace>, Error> {
        let cmd = Command::new(ADMIN_IDENTIFY, id).prp(self.bounce_phys, 0).dw(10, CNS_NAMESPACE);
        self.admin_command("identify namespace", cmd)?;
        let b = |i: usize| unsafe { read_volatile(self.bounce.add(i)) };
        let blocks = u64::from_le_bytes(core::array::from_fn(|i| b(i)));
        if blocks == 0 {
            return Ok(None);
        }
        // FLBAS: the format in use, its upper bits in 6:5 since 2.0.
        let flbas = b(26);
        let format = ((flbas & 0xF) | (flbas >> 5 & 3) << 4) as usize;
        let lbaf = u32::from_le_bytes([b(128 + 4 * format), b(129 + 4 * format), b(130 + 4 * format), b(131 + 4 * format)]);
        let metadata = lbaf & 0xFFFF;
        let block_shift = (lbaf >> 16) & 0xFF;
        if metadata != 0 || !(9..=12).contains(&block_shift) {
            nvprint!("namespace {}: format {} ({} B blocks, {} B metadata) is not supported\n", id, format, 1u64 << block_shift.min(63), metadata);
            return Ok(None);
        }
        Ok(Some(Namespace { id, blocks, block_shift }))
    }

    /// The most blocks of `ns` one read or write may move.
    pub fn max_blocks(&self, ns: &Namespace) -> u64 {
        (self.max_transfer >> ns.block_shift) as u64
    }

    /// Read `blocks` blocks at `lba` into the bounce buffer; returns it.
    pub fn read(&mut self, ns: &Namespace, lba: u64, blocks: u64) -> Result<&[u8], Error> {
        let bytes = self.rw(NVM_READ, "read", ns, lba, blocks)?;
        Ok(unsafe { core::slice::from_raw_parts(self.bounce, bytes) })
    }

    /// Write `data` (whole blocks, at most max_blocks) to `lba`.
    pub fn write(&mut self, ns: &Namespace, lba: u64, data: &[u8]) -> Result<(), Error> {
        unsafe { core::ptr::copy_nonoverlapping(data.as_ptr(), self.bounce, data.len()) };
        self.rw(NVM_WRITE, "write", ns, lba, (data.len() >> ns.block_shift) as u64).map(|_| ())
    }

    pub fn flush(&mut self, ns: &Namespace) -> Result<(), Error> {
        self.io_command("flush", Command::new(NVM_FLUSH, ns.id)).map(|_| ())
    }

    fn rw(&mut self, opcode: u8, what: &'static str, ns: &Namespace, lba: u64, blocks: u64) -> Result<usize, Error> {
        let bytes = (blocks << ns.block_shift) as usize;
        assert!(blocks > 0 && bytes <= self.max_transfer, "{what} of {blocks} blocks");
        let prp2 = match bytes {
            0..=PAGE => 0,
            _ if bytes <= 2 * PAGE => self.bounce_phys + PAGE as u64,
            _ => self.prp_list_phys,
        };
        let cmd = Command::new(opcode, ns.id)
            .prp(self.bounce_phys, prp2)
            .dw(10, lba as u32)
            .dw(11, (lba >> 32) as u32)
            .dw(12, blocks as u32 - 1); // NLB is zero-based
        self.io_command(what, cmd)?;
        Ok(bytes)
    }

    fn admin_command(&mut self, what: &'static str, cmd: Command) -> Result<u32, Error> {
        self.submit(false, what, cmd)
    }

    fn io_command(&mut self, what: &'static str, cmd: Command) -> Result<u32, Error> {
        self.submit(true, what, cmd)
    }

    /// Submit `cmd` and poll its completion. Returns completion dword 0.
    fn submit(&mut self, io: bool, what: &'static str, mut cmd: Command) -> Result<u32, Error> {
        let cid = self.next_cid;
        self.next_cid = self.next_cid.wrapping_add(1);
        cmd.0[0] |= (cid as u32) << 16;
        let stride = self.doorbell_stride;
        let regs = self.regs;
        let q = if io { &mut self.io } else { &mut self.admin };
        unsafe {
            let slot = q.sq.add(q.tail as usize * SQE).cast::<u32>();
            for (i, &dw) in cmd.0.iter().enumerate() {
                write_volatile(slot.add(i), dw);
            }
        }
        q.tail = (q.tail + 1) % QUEUE_ENTRIES;
        wmb();
        // Submission queue y tail doorbell: 0x1000 + (2y) * stride (3.1.24).
        unsafe { write_volatile(regs.add(DOORBELLS + 2 * q.id as usize * stride).cast::<u32>(), q.tail as u32) };

        let t0 = now_us();
        loop {
            let entry = unsafe { q.cq.add(q.head as usize * CQE).cast::<u32>() };
            let dw3 = unsafe { read_volatile(entry.add(3)) };
            if dw3 >> 16 & 1 == q.phase {
                rmb();
                let dw0 = unsafe { read_volatile(entry) };
                q.head += 1;
                if q.head == QUEUE_ENTRIES {
                    q.head = 0;
                    q.phase ^= 1;
                }
                // Completion queue y head doorbell: 0x1000 + (2y + 1) * stride.
                unsafe { write_volatile(regs.add(DOORBELLS + (2 * q.id as usize + 1) * stride).cast::<u32>(), q.head as u32) };
                let returned = dw3 as u16;
                if returned != cid {
                    nvprint!("{}: completion for command {} while {} was outstanding\n", what, returned, cid);
                }
                // Status (4.2.3): code in 24:17, type in 27:25.
                let (sct, sc) = ((dw3 >> 25 & 7) as u8, (dw3 >> 17) as u8);
                if sct != 0 || sc != 0 {
                    return Err(Error::Status(what, sct, sc));
                }
                return Ok(dw0);
            }
            let elapsed = now_us() - t0;
            if elapsed > COMMAND_TIMEOUT_US {
                let csts = unsafe { read_volatile(regs.add(REG_CSTS).cast::<u32>()) };
                return Err(if csts & CSTS_CFS != 0 { Error::Fatal } else { Error::Timeout(what) });
            }
            if elapsed < SPIN_US {
                unsafe { KeStallExecutionProcessor(1) };
            } else {
                sleep_ms(1);
            }
        }
    }
}
