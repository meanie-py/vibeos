//! One NVMe controller: the admin queue, one I/O queue pair many commands deep,
//! and a DMA bounce buffer, all in one physically contiguous allocation.
//!
//! Admin commands run one at a time and are polled. All of them are issued at
//! START, before the interrupt is armed, so they never meet the I/O path.
//!
//! I/O is asynchronous. A caller hands [`Controller::submit`] a [`Request`] (a
//! read or write of any length, or a flush) and gets its `done` callback, at
//! DISPATCH_LEVEL, once all of it has completed. The engine cuts a request into
//! commands of at most `max_transfer` bytes and keeps up to one per [`Slot`] in
//! flight, the slot index serving as the command ID. A request that cannot start
//! yet waits in a FIFO, so requests start in the order they were submitted, and
//! only the one at its head is ever part-way issued. A flush starts only once
//! everything ahead of it has completed: NVMe's Flush covers the writes that
//! completed before it was submitted, not the ones still in flight.
//!
//! Data moves by DMA straight to and from the requester's pages, through PRP
//! entries (4.1.1) built from its MDL, whenever the MDL is one whose page list
//! can be trusted (locked, nonpaged or partial) and the data starts on a dword,
//! as PRP1's offset must. Anything else goes through the bounce buffer, one
//! command at a time; its PRP list is built once, at start.
//!
//! Completions are reaped in the DPC, queued by the ISR and by a 10 ms timer
//! that runs while anything is in flight. With an interrupt connected the timer
//! reaps only a completion it already saw at its previous tick, so it cannot
//! steal one from a working interrupt; when it does reap, the interrupt went
//! missing, and it says so on the debug port the first time. With none
//! connected the timer is all there is.
//!
//! Two kinds of interrupt. MSI-X, when the PnP manager assigned messages: the
//! admin queue raises message 0 and the I/O queue message 1 (both 0 when there
//! is only one), each an exclusive, edge-triggered vector, so the ISR has
//! nothing to check or mask. Otherwise the pin (INTx): both queues raise vector
//! 0, which stays masked (INTMS) whenever nothing is connected to it, so the
//! line can never be left asserted on a pin another device shares; the ISR
//! masks it and the DPC unmasks it once the queue is drained. The spec forbids
//! touching INTMS/INTMC under MSI-X, so the controller is told which it has
//! before bring-up.
//!
//! A command that never completes is reported after 30 s and left in flight:
//! its pages may still be the target of a DMA, so the request cannot be handed
//! back. Recovering from that needs a controller reset, which this driver does
//! not do yet.
//!
//! Register and command layouts are from the NVM Express Base Specification
//! (2.0); the section numbers below are that document's.

use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{null, null_mut, read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use ntkrnl_sys::ddk::{
    KeDelayExecutionThread, KeInitializeDpc, KeInitializeTimer, KeInsertQueueDpc, KeQueryPerformanceCounter, KeSetTimer,
    KeStallExecutionProcessor, ExAllocatePoolWithTag, MmAllocateContiguousMemorySpecifyCache, MmFreeContiguousMemory, MmGetPhysicalAddress, _MEMORY_CACHING_TYPE_MmCached,
    BOOLEAN, KDPC, KTIMER, LARGE_INTEGER, MDL, PHYSICAL_ADDRESS, PKINTERRUPT, PMDL, PVOID, ULONG,
};
use ntkrnl_sys::guards::SpinLockGuard;
use ntkrnl_sys::ke::KSPIN_LOCK;

use crate::print::nvprint;

pub const PAGE: usize = 4096;
/// The bounce buffer, and so the most one command moves (before MDTS).
const BOUNCE: usize = 128 * 1024;
const ADMIN_ENTRIES: u16 = 16;
/// I/O queue entries. A queue of N entries holds N - 1 commands, so this many
/// less one is the deepest the I/O queue gets: one command per slot.
const IO_ENTRIES: u16 = 64;
pub const SLOTS: usize = IO_ENTRIES as usize - 1;
const NONPAGED_POOL: i32 = 0;
const IO_POOL_TAG: u32 = u32::from_le_bytes(*b"NVMq");
/// Each slot's PRP list: room for a whole command starting anywhere in a page
/// (BOUNCE / PAGE + 1 pages, less the one PRP1 names), and a power of two, so
/// that no list crosses a page boundary (4.1.1 forbids it without chaining).
const PRP_LIST: usize = 512;
const _: () = assert!((BOUNCE / PAGE) * 8 <= PRP_LIST && PAGE % PRP_LIST == 0);
const SLOT_PRP_PAGES: usize = (SLOTS * PRP_LIST).div_ceil(PAGE);
const SQE: usize = 64;
const CQE: usize = 16;
/// An admin command that has not completed in this long has failed.
const COMMAND_TIMEOUT_US: u64 = 30_000_000;
/// Spin this long for an admin completion before sleeping between polls.
const SPIN_US: u64 = 2_000;
/// The completion timer's period, while anything is in flight.
pub const POLL_MS: i64 = 10;

// Controller registers (3.1).
const REG_CAP: usize = 0x00;
const REG_VS: usize = 0x08;
const REG_INTMS: usize = 0x0C;
const REG_INTMC: usize = 0x10;
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

// mmtypes.h: MDLs whose page list describes locked, present pages.
const MDL_PAGES_LOCKED: i16 = 0x0002;
const MDL_SOURCE_IS_NONPAGED_POOL: i16 = 0x0004;
const MDL_PARTIAL: i16 = 0x0010;
const MDL_IO_SPACE: i16 = 0x0800;

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
    entries: u16,
    sq: *mut u8,
    sq_phys: u64,
    cq: *mut u8,
    cq_phys: u64,
    tail: u16,
    head: u16,
    phase: u32,
}

impl Queue {
    /// Write `cmd` at the tail; the caller rings the doorbell.
    fn push(&mut self, cmd: &Command) {
        unsafe {
            let slot = self.sq.add(self.tail as usize * SQE).cast::<u32>();
            for (i, &dw) in cmd.0.iter().enumerate() {
                write_volatile(slot.add(i), dw);
            }
        }
        self.tail = (self.tail + 1) % self.entries;
    }

    /// The completion at the head, if it has arrived: dwords 0 and 3.
    fn peek(&self) -> Option<(u32, u32)> {
        let entry = unsafe { self.cq.add(self.head as usize * CQE).cast::<u32>() };
        let dw3 = unsafe { read_volatile(entry.add(3)) };
        if dw3 >> 16 & 1 != self.phase {
            return None;
        }
        rmb();
        Some((unsafe { read_volatile(entry) }, dw3))
    }

    fn advance(&mut self) {
        self.head += 1;
        if self.head == self.entries {
            self.head = 0;
            self.phase ^= 1;
        }
    }

    /// Where the next completion lands and the phase it will carry, as one word.
    fn position(&self) -> u32 {
        self.head as u32 | self.phase << 16
    }
}

/// Status (4.2.3): code type in 27:25, code in 24:17. None when it succeeded.
fn status(dw3: u32) -> Option<(u8, u8)> {
    let (sct, sc) = ((dw3 >> 25 & 7) as u8, (dw3 >> 17) as u8);
    (sct != 0 || sc != 0).then_some((sct, sc))
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    Read,
    Write,
    Flush,
}

impl Op {
    pub fn name(self) -> &'static str {
        match self {
            Op::Read => "read",
            Op::Write => "write",
            Op::Flush => "flush",
        }
    }
}

/// One read, write or flush, from [`Controller::submit`] to `done`. The caller
/// owns the memory, which must stay put and untouched in between.
pub struct Request {
    pub op: Op,
    pub nsid: u32,
    pub block_shift: u32,
    pub lba: u64,
    pub blocks: u64,
    /// The data's pages, and how far into the MDL's bytes the data starts.
    /// Null: there is no MDL, and the data goes through the bounce buffer.
    pub mdl: PMDL,
    pub mdl_offset: usize,
    /// The data at a system address, which the bounce buffer copies to or
    /// from. Needed only when [`dma_ready`] says the pages cannot be used.
    pub system: *mut u8,
    /// Runs at DISPATCH_LEVEL once every command of the request has completed.
    pub done: unsafe fn(*mut Request),
    pub context: *mut c_void,
    /// The first command to fail, if one did; the rest were not issued.
    pub error: Option<Error>,
    // The engine's, under its lock.
    direct: bool,
    issued: u64,
    outstanding: u32,
    queued: bool,
    next: *mut Request,
}

impl Request {
    pub fn new(op: Op, ns: &Namespace, lba: u64, blocks: u64, done: unsafe fn(*mut Request), context: *mut c_void) -> Self {
        Request {
            op,
            nsid: ns.id,
            block_shift: ns.block_shift,
            lba,
            blocks: if op == Op::Flush { 0 } else { blocks },
            mdl: null_mut(),
            mdl_offset: 0,
            system: null_mut(),
            done,
            context,
            error: None,
            direct: false,
            issued: 0,
            outstanding: 0,
            queued: false,
            next: null_mut(),
        }
    }

    pub fn bytes(&self) -> usize {
        (self.blocks << self.block_shift) as usize
    }

    fn finished(&self) -> bool {
        self.outstanding == 0 && (self.issued == self.blocks || self.error.is_some())
    }
}

/// Can the device DMA straight to `bytes` bytes at `offset` into `mdl`'s
/// data? Its page list must describe pages that stay put, which a locked,
/// nonpaged-pool or partial MDL's does (and nothing else's need), and PRP1's
/// offset must be dword aligned.
///
/// # Safety
/// `mdl` must be null or a valid MDL.
pub unsafe fn dma_ready(mdl: PMDL, offset: usize, bytes: usize) -> bool {
    if mdl.is_null() {
        return false;
    }
    let m = &*mdl;
    m.MdlFlags & (MDL_PAGES_LOCKED | MDL_SOURCE_IS_NONPAGED_POOL | MDL_PARTIAL) != 0
        && m.MdlFlags & MDL_IO_SPACE == 0
        && (m.ByteOffset as usize + offset) & 3 == 0
        && offset.checked_add(bytes).is_some_and(|end| end <= m.ByteCount as usize)
}

/// The physical address `offset` bytes into `mdl`'s data, from its page list
/// (the PFN array that follows the MDL header).
unsafe fn mdl_phys(mdl: PMDL, offset: usize) -> u64 {
    let at = (*mdl).ByteOffset as usize + offset;
    let pfns = mdl.cast::<MDL>().add(1).cast::<usize>();
    ((*pfns.add(at / PAGE) as u64) << 12) | (at % PAGE) as u64
}

/// One command's worth of a request, in flight. The index is its command ID.
struct Slot {
    request: *mut Request,
    /// Where in the request this command's data starts, and how much it moves.
    offset: usize,
    bytes: usize,
    bounce: bool,
    started: u64,
    warned: bool,
    prp: *mut u64,
    prp_phys: u64,
}

/// The I/O path's state: everything the submitters, the DPC and the timer
/// share, under [`Controller`]'s lock.
struct Io {
    queue: Queue,
    slots: [Slot; SLOTS],
    /// Slots usable: fewer than SLOTS when the controller's queues are shallower.
    depth: usize,
    in_flight: usize,
    bounce_busy: bool,
    /// The FIFO of requests not yet fully issued.
    head: *mut Request,
    tail: *mut Request,
    timer_set: bool,
    /// With an interrupt connected: where the timer saw a completion waiting
    /// at its last tick (`Queue::position`), or u32::MAX.
    poll_mark: u32,
    stats: Stats,
}

#[derive(Clone, Copy, Default)]
pub struct Stats {
    /// Completions the ISR's DPC reaped, and those the timer had to.
    pub by_interrupt: u32,
    pub by_poll: u32,
    /// The most commands in flight at once.
    pub peak: u32,
    /// Commands whose data moved by DMA to the requester's pages, and through the bounce buffer.
    pub direct: u32,
    pub bounced: u32,
}

pub struct Controller {
    regs: *mut u8,
    doorbell_stride: usize,
    ready_timeout_ms: u64,
    admin: Queue,
    /// The bounce buffer, and the PRP list page describing it.
    bounce: *mut u8,
    bounce_phys: u64,
    bounce_prp_phys: u64,
    next_cid: u16,
    pub max_transfer: usize,
    pub identity: Identity,
    /// Interrupts are MSI-X messages, not the pin: INTMS/INTMC are off limits.
    msix: bool,
    /// Null until [`Controller::arm`].
    irq: *mut Irq,
    /// An interrupt is connected (else the timer alone reaps).
    interrupts: bool,
    /// The controller's number, for the debug port.
    number: u32,
    lock: UnsafeCell<KSPIN_LOCK>,
    /// In nonpaged pool, for the life of the system (see `start`).
    io: *mut Io,
}

// Started on one thread, then shared by the submitters, the DPC and the timer,
// which touch the I/O state only under the lock.
unsafe impl Send for Controller {}
unsafe impl Sync for Controller {}

/// Pin-based interrupt vector 0, the one both completion queues raise.
const VECTOR_0: u32 = 1;

/// What the interrupt path shares with the I/O engine. Lives in the FDO's
/// extension, so its address is stable for the ISR, the DPC and the timer.
pub struct Irq {
    regs: *mut u8,
    /// Admin and I/O completion queues, and for each the slot the next
    /// completion will land in and the phase it will carry (`Queue::position`),
    /// published every time one is consumed. The pin ISR reads them.
    cqs: [*const u8; 2],
    next: [AtomicU32; 2],
    /// Set once the controller is armed and the state above is valid.
    armed: AtomicBool,
    controller: *const Controller,
    dpc: KDPC,
    timer: KTIMER,
    timer_dpc: KDPC,
    pub interrupt: PKINTERRUPT,
    /// Under MSI-X: interrupts per message (admin, I/O).
    pub messages: [AtomicU32; 2],
}

impl Irq {
    pub const fn new() -> Self {
        Irq {
            regs: null_mut(),
            cqs: [null(); 2],
            next: [AtomicU32::new(0), AtomicU32::new(0)],
            armed: AtomicBool::new(false),
            controller: null(),
            dpc: unsafe { core::mem::zeroed() },
            timer: unsafe { core::mem::zeroed() },
            timer_dpc: unsafe { core::mem::zeroed() },
            interrupt: null_mut(),
            messages: [AtomicU32::new(0), AtomicU32::new(0)],
        }
    }

    pub fn dpc(&mut self) -> *mut KDPC {
        &raw mut self.dpc
    }

    /// Does either completion queue hold an entry not yet consumed?
    fn pending(&self) -> bool {
        (0..2).any(|i| {
            let next = self.next[i].load(Ordering::Acquire);
            let (head, phase) = ((next & 0xFFFF) as usize, next >> 16);
            let dw3 = unsafe { read_volatile(self.cqs[i].add(head * CQE + 12).cast::<u32>()) };
            dw3 >> 16 & 1 == phase
        })
    }
}

/// The interrupt service routine (`ServiceContext` is the [`Irq`]). The line may
/// be shared and is level-triggered, so it claims the interrupt only when a
/// completion queue has news, and then masks vector 0 so the line drops until
/// the DPC has drained the queue and unmasked it.
pub unsafe extern "system" fn isr(_interrupt: PKINTERRUPT, context: PVOID) -> BOOLEAN {
    let irq = &*(context as *const Irq);
    if !irq.armed.load(Ordering::Acquire) || !irq.pending() {
        return 0;
    }
    write_volatile(irq.regs.add(REG_INTMS).cast::<u32>(), VECTOR_0);
    KeInsertQueueDpc(&irq.dpc as *const KDPC as *mut KDPC, null_mut(), null_mut());
    1
}

/// The MSI-X message service routine (`ServiceContext` is the [`Irq`]). A
/// message is this device's alone and edge-triggered: nothing to check, nothing
/// to mask. One that lands before the controller is armed (the admin commands
/// at START, which poll) is just counted.
pub unsafe extern "system" fn message_isr(_interrupt: PKINTERRUPT, context: PVOID, message: ULONG) -> BOOLEAN {
    let irq = &*(context as *const Irq);
    irq.messages[(message as usize).min(1)].fetch_add(1, Ordering::Relaxed);
    if irq.armed.load(Ordering::Acquire) {
        KeInsertQueueDpc(&irq.dpc as *const KDPC as *mut KDPC, null_mut(), null_mut());
    }
    1
}

/// The interrupt's DPC (`DeferredContext` is the [`Irq`]).
pub unsafe extern "system" fn dpc(_dpc: *mut KDPC, context: PVOID, _a: PVOID, _b: PVOID) {
    let irq = &*(context as *const Irq);
    if irq.armed.load(Ordering::Acquire) {
        (*irq.controller).service(true);
    }
}

/// The completion timer's DPC (`DeferredContext` is the [`Irq`]).
unsafe extern "system" fn timer_dpc(_dpc: *mut KDPC, context: PVOID, _a: PVOID, _b: PVOID) {
    let irq = &*(context as *const Irq);
    (*irq.controller).service(false);
}

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
    fn cid(mut self, cid: u16) -> Self {
        self.0[0] |= (cid as u32) << 16;
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
    /// Submission queue y tail doorbell: 0x1000 + (2y) * stride (3.1.24);
    /// completion queue y head doorbell: 0x1000 + (2y + 1) * stride.
    fn ring(&self, queue: u16, completion: bool, value: u16) {
        self.wr(DOORBELLS + (2 * queue as usize + completion as usize) * self.doorbell_stride, value as u32);
    }

    /// Reset the controller behind `regs` (a mapping of BAR0, uncached, at
    /// least `len` bytes) and bring it up with one I/O queue pair. `messages`
    /// is how many MSI-X messages are connected (0: the pin).
    ///
    /// # Safety
    /// `regs` must map the controller's registers for `len` bytes.
    pub unsafe fn start(regs: *mut u8, len: usize, messages: u32) -> Result<Controller, Error> {
        let cap = read_volatile(regs.add(REG_CAP).cast::<u32>()) as u64 | (read_volatile(regs.add(REG_CAP + 4).cast::<u32>()) as u64) << 32;
        let version = read_volatile(regs.add(REG_VS).cast::<u32>());
        if cap == u64::MAX || version >> 16 == 0 || version == u32::MAX {
            return Err(Error::NotNvme(version));
        }
        let mqes = (cap & 0xFFFF) as u32 + 1;
        let doorbell_stride = 4usize << ((cap >> 32) & 0xF);
        let ready_timeout_ms = ((cap >> 24) & 0xFF).max(1) * 500;
        let mps_min = ((cap >> 48) & 0xF) as u32;
        if mps_min != 0 {
            return Err(Error::PageSize(4096 << mps_min));
        }
        if DOORBELLS + 4 * doorbell_stride > len {
            return Err(Error::NotNvme(version));
        }
        let admin_entries = (ADMIN_ENTRIES as u32).min(mqes) as u16;
        let io_entries = (IO_ENTRIES as u32).min(mqes) as u16;
        nvprint!(
            "controller: NVMe {}.{}, {} queue entries max, doorbell stride {}, ready timeout {} ms\n",
            version >> 16,
            (version >> 8) & 0xFF,
            mqes,
            doorbell_stride,
            ready_timeout_ms
        );

        // Pages: admin SQ, admin CQ, I/O SQ, I/O CQ, the bounce buffer's PRP
        // list, the slots' PRP lists, then the bounce buffer.
        let tables = 5 + SLOT_PRP_PAGES;
        let size = tables * PAGE + BOUNCE;
        let mut low: PHYSICAL_ADDRESS = core::mem::zeroed();
        let mut high: PHYSICAL_ADDRESS = core::mem::zeroed();
        let boundary: PHYSICAL_ADDRESS = core::mem::zeroed();
        low.QuadPart = 0;
        // NVMe addresses all 64 bits; i386 kernels map no more than 4 GiB.
        high.QuadPart = if cfg!(target_pointer_width = "64") { 0x0000_FFFF_FFFF_FFFF } else { 0xFFFF_FFFF };
        // Cached: PCIe DMA snoops the CPU caches on every machine this targets
        // (x86, QEMU virt, Graviton), as it must for the requesters' own pages.
        let base = MmAllocateContiguousMemorySpecifyCache(size as _, low, high, boundary, _MEMORY_CACHING_TYPE_MmCached)
            as *mut u8;
        if base.is_null() {
            return Err(Error::NoMemory);
        }
        core::ptr::write_bytes(base, 0, tables * PAGE);
        let phys = |p: *mut u8| MmGetPhysicalAddress(p.cast()).QuadPart as u64;
        let queue = |id, entries, sq: *mut u8, cq: *mut u8| Queue {
            id,
            entries,
            sq,
            sq_phys: phys(sq),
            cq,
            cq_phys: phys(cq),
            tail: 0,
            head: 0,
            phase: 1,
        };
        let bounce_prp = base.add(4 * PAGE).cast::<u64>();
        let bounce = base.add(tables * PAGE);
        let slot_prps = base.add(5 * PAGE);
        // The I/O state is built in place, in pool: with its slots it is
        // kilobytes, and i386's kernel stack is 12 KiB all told.
        let io = ExAllocatePoolWithTag(NONPAGED_POOL as _, size_of::<Io>() as _, IO_POOL_TAG) as *mut Io;
        if io.is_null() {
            MmFreeContiguousMemory(base.cast());
            return Err(Error::NoMemory);
        }
        (&raw mut (*io).queue).write(queue(1, io_entries, base.add(2 * PAGE), base.add(3 * PAGE)));
        let slots = (&raw mut (*io).slots).cast::<Slot>();
        for i in 0..SLOTS {
            slots.add(i).write(Slot {
                request: null_mut(),
                offset: 0,
                bytes: 0,
                bounce: false,
                started: 0,
                warned: false,
                prp: slot_prps.add(i * PRP_LIST).cast(),
                prp_phys: phys(slot_prps.add(i * PRP_LIST)),
            });
        }
        (&raw mut (*io).depth).write(io_entries as usize - 1);
        (&raw mut (*io).in_flight).write(0);
        (&raw mut (*io).bounce_busy).write(false);
        (&raw mut (*io).head).write(null_mut());
        (&raw mut (*io).tail).write(null_mut());
        (&raw mut (*io).timer_set).write(false);
        (&raw mut (*io).poll_mark).write(u32::MAX);
        (&raw mut (*io).stats).write(Stats::default());
        let mut c = Controller {
            regs,
            doorbell_stride,
            ready_timeout_ms,
            admin: queue(0, admin_entries, base, base.add(PAGE)),
            bounce,
            bounce_phys: phys(bounce),
            bounce_prp_phys: phys(bounce_prp.cast()),
            next_cid: 0,
            max_transfer: BOUNCE,
            identity: core::mem::zeroed(),
            msix: messages > 0,
            irq: null_mut(),
            interrupts: false,
            number: 0,
            lock: UnsafeCell::new(0),
            io,
        };
        // The bounce buffer's PRP list (4.1.1): entry i is its page i + 1, since
        // PRP1 always names page 0. The pages are physically contiguous.
        for i in 0..BOUNCE / PAGE - 1 {
            *bounce_prp.add(i) = c.bounce_phys + ((i + 1) * PAGE) as u64;
        }

        // Disable (firmware may have left it running), then program the admin
        // queue and enable (3.5.1).
        if c.rd(REG_CC) & CC_EN != 0 {
            c.wr(REG_CC, c.rd(REG_CC) & !CC_EN);
        }
        c.wait_ready(false)?;
        // Mask every pin-based vector until an interrupt is connected to it.
        if !c.msix {
            c.wr(REG_INTMS, u32::MAX);
        }
        c.wr(REG_AQA, ((admin_entries as u32 - 1) << 16) | (admin_entries as u32 - 1));
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
        // Completion queue first: physically contiguous, interrupts enabled. On
        // the pin, vector 0, which INTMS masks until one is connected; under
        // MSI-X, message 1 when there is one, so its interrupts are its own.
        let vector = (messages >= 2) as u32;
        let io = &mut *c.io;
        let (cq_phys, sq_phys, size) = (io.queue.cq_phys, io.queue.sq_phys, io_entries as u32 - 1);
        let cq = Command::new(ADMIN_CREATE_CQ, 0)
            .prp(cq_phys, 0)
            .dw(10, (size << 16) | 1)
            .dw(11, vector << 16 | 1 << 1 | 1);
        c.admin_command("create I/O completion queue", cq)?;
        let sq = Command::new(ADMIN_CREATE_SQ, 0)
            .prp(sq_phys, 0)
            .dw(10, (size << 16) | 1)
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

    /// How many commands the I/O queue holds at once.
    pub fn depth(&mut self) -> usize {
        unsafe { (*self.io).depth }
    }

    /// Open the I/O path. The controller must be where it will stay (its
    /// address goes to the DPCs), every admin command must be done (they poll,
    /// and would race the DPC for the pin), and when `interrupts` is set,
    /// `irq`'s interrupt must be connected with `irq` as its context.
    ///
    /// # Safety
    /// `irq` must point to a live [`Irq`] that outlives the controller, and
    /// neither may move from here on.
    pub unsafe fn arm(&mut self, irq: *mut Irq, number: u32, interrupts: bool) {
        self.number = number;
        self.irq = irq;
        self.interrupts = interrupts;
        let io = &mut *self.io;
        (*irq).regs = self.regs;
        (*irq).cqs = [self.admin.cq, io.queue.cq];
        (*irq).next[0].store(self.admin.position(), Ordering::Release);
        (*irq).next[1].store(io.queue.position(), Ordering::Release);
        (*irq).controller = self;
        KeInitializeTimer(&raw mut (*irq).timer);
        KeInitializeDpc(&raw mut (*irq).timer_dpc, Some(timer_dpc), irq.cast());
        (*irq).armed.store(true, Ordering::Release);
        if interrupts && !self.msix {
            self.wr(REG_INTMC, VECTOR_0);
        }
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

    /// Submit an admin command and poll for its completion. Returns completion
    /// dword 0. Only before [`Controller::arm`]: see the module comment.
    fn admin_command(&mut self, what: &'static str, cmd: Command) -> Result<u32, Error> {
        let cid = self.next_cid;
        self.next_cid = self.next_cid.wrapping_add(1);
        self.admin.push(&cmd.cid(cid));
        wmb();
        self.ring(0, false, self.admin.tail);

        let t0 = now_us();
        loop {
            if let Some((dw0, dw3)) = self.admin.peek() {
                self.admin.advance();
                self.ring(0, true, self.admin.head);
                if dw3 as u16 != cid {
                    nvprint!("{}: completion for command {} while {} was outstanding\n", what, dw3 as u16, cid);
                }
                return match status(dw3) {
                    Some((sct, sc)) => Err(Error::Status(what, sct, sc)),
                    None => Ok(dw0),
                };
            }
            let elapsed = now_us() - t0;
            if elapsed > COMMAND_TIMEOUT_US {
                return Err(if self.rd(REG_CSTS) & CSTS_CFS != 0 { Error::Fatal } else { Error::Timeout(what) });
            }
            if elapsed < SPIN_US {
                unsafe { KeStallExecutionProcessor(1) };
            } else {
                sleep_ms(1);
            }
        }
    }

    // -----------------------------------------------------------------------
    // The I/O path
    // -----------------------------------------------------------------------

    /// The most blocks of `ns` one command may move.
    pub fn max_blocks(&self, ns: &Namespace) -> u64 {
        (self.max_transfer >> ns.block_shift) as u64
    }

    /// Start `r` (at <= DISPATCH_LEVEL, after [`Controller::arm`]). Its `done`
    /// runs once it has completed, never from inside this call.
    ///
    /// # Safety
    /// `r` must stay valid and untouched until its `done` runs. Its data must
    /// be reachable as the [`Request`] fields promise.
    pub unsafe fn submit(&self, r: *mut Request) {
        let req = &mut *r;
        assert!(req.op == Op::Flush || req.blocks > 0, "an empty {}", req.op.name());
        assert!(
            req.op == Op::Flush || dma_ready(req.mdl, req.mdl_offset, req.bytes()) || !req.system.is_null(),
            "a {} with neither DMA-ready pages nor a system address",
            req.op.name()
        );
        req.direct = req.op != Op::Flush && dma_ready(req.mdl, req.mdl_offset, req.bytes());
        req.issued = 0;
        req.outstanding = 0;
        req.error = None;
        req.next = null_mut();
        let _g = SpinLockGuard::acquire(self.lock.get());
        let io = &mut *self.io;
        req.queued = true;
        if io.tail.is_null() {
            io.head = r;
        } else {
            (*io.tail).next = r;
        }
        io.tail = r;
        self.pump(io);
    }

    fn pop(io: &mut Io) {
        let r = io.head;
        unsafe {
            io.head = (*r).next;
            (*r).next = null_mut();
            (*r).queued = false;
        }
        if io.head.is_null() {
            io.tail = null_mut();
        }
    }

    /// Issue commands from the head of the FIFO while there are slots for
    /// them, then ring the doorbell once for the lot. Under the lock.
    unsafe fn pump(&self, io: &mut Io) {
        let mut issued = false;
        while !io.head.is_null() {
            let r = &mut *io.head;
            if r.error.is_some() {
                // A command of it failed (and others are still in flight, or
                // the reap would have finished it): issue no more of it.
                Self::pop(io);
                continue;
            }
            if r.op == Op::Flush && io.in_flight > 0 {
                break;
            }
            if !r.direct && r.op != Op::Flush && io.bounce_busy {
                break;
            }
            let Some(index) = io.slots[..io.depth].iter().position(|s| s.request.is_null()) else {
                break;
            };
            // Every field set afresh: the slot's last command may have been anything.
            let slot = &mut io.slots[index];
            slot.request = r;
            slot.started = now_us();
            slot.warned = false;
            slot.offset = 0;
            slot.bytes = 0;
            slot.bounce = false;
            let cmd = if r.op == Op::Flush {
                Command::new(NVM_FLUSH, r.nsid)
            } else {
                let blocks = (self.max_transfer as u64 >> r.block_shift).min(r.blocks - r.issued);
                let offset = (r.issued << r.block_shift) as usize;
                let bytes = (blocks << r.block_shift) as usize;
                let lba = r.lba + r.issued;
                slot.offset = offset;
                slot.bytes = bytes;
                slot.bounce = !r.direct;
                let (prp1, prp2) = if r.direct {
                    io.stats.direct += 1;
                    self.direct_prps(r, slot, offset, bytes)
                } else {
                    io.stats.bounced += 1;
                    io.bounce_busy = true;
                    if r.op == Op::Write {
                        core::ptr::copy_nonoverlapping(r.system.add(offset), self.bounce, bytes);
                    }
                    let prp2 = match bytes {
                        0..=PAGE => 0,
                        _ if bytes <= 2 * PAGE => self.bounce_phys + PAGE as u64,
                        _ => self.bounce_prp_phys,
                    };
                    (self.bounce_phys, prp2)
                };
                r.issued += blocks;
                let opcode = if r.op == Op::Write { NVM_WRITE } else { NVM_READ };
                Command::new(opcode, r.nsid)
                    .prp(prp1, prp2)
                    .dw(10, lba as u32)
                    .dw(11, (lba >> 32) as u32)
                    .dw(12, blocks as u32 - 1) // NLB is zero-based
            };
            io.queue.push(&cmd.cid(index as u16));
            r.outstanding += 1;
            io.in_flight += 1;
            io.stats.peak = io.stats.peak.max(io.in_flight as u32);
            issued = true;
            if r.op == Op::Flush || r.issued == r.blocks {
                Self::pop(io);
            }
        }
        if issued {
            wmb();
            self.ring(io.queue.id, false, io.queue.tail);
        }
        if io.in_flight > 0 && !io.timer_set {
            let mut due: LARGE_INTEGER = core::mem::zeroed();
            due.QuadPart = -10_000 * POLL_MS;
            KeSetTimer(&raw mut (*self.irq).timer, due, &raw mut (*self.irq).timer_dpc);
            io.timer_set = true;
        }
    }

    /// PRP1 and PRP2 for `bytes` bytes at `offset` into `r`'s data, straight
    /// from its MDL's page list (4.1.1): PRP1 the first byte, PRP2 the second
    /// page or, past two, the slot's list of the rest.
    unsafe fn direct_prps(&self, r: &Request, slot: &mut Slot, offset: usize, bytes: usize) -> (u64, u64) {
        let at = r.mdl_offset + offset;
        let prp1 = mdl_phys(r.mdl, at);
        let first = PAGE - (prp1 as usize % PAGE);
        if bytes <= first {
            return (prp1, 0);
        }
        let rest = (bytes - first).div_ceil(PAGE);
        if rest == 1 {
            return (prp1, mdl_phys(r.mdl, at + first));
        }
        for i in 0..rest {
            *slot.prp.add(i) = mdl_phys(r.mdl, at + first + i * PAGE);
        }
        (prp1, slot.prp_phys)
    }

    /// Reap what has completed, start what now can, and call the finished
    /// requests back. From the interrupt's DPC (`interrupt`) or the timer's.
    unsafe fn service(&self, interrupt: bool) {
        let mut finished: *mut Request = null_mut();
        {
            let _g = SpinLockGuard::acquire(self.lock.get());
            let io = &mut *self.io;
            if !interrupt {
                io.timer_set = false;
            }
            // With an interrupt connected the timer reaps only what it saw
            // waiting last tick too: anything newer is the interrupt's.
            let reap = interrupt || !self.interrupts || {
                let waiting = io.queue.peek().is_some();
                let seen = waiting && io.poll_mark == io.queue.position();
                io.poll_mark = if waiting { io.queue.position() } else { u32::MAX };
                seen
            };
            if reap {
                let reaped = self.reap(io, &mut finished);
                io.poll_mark = u32::MAX;
                if reaped > 0 {
                    self.tally(io, interrupt, reaped);
                }
            }
            if !interrupt {
                self.check_overdue(io);
            }
            self.pump(io);
        }
        // The queue is drained: the pin may raise the next one.
        if interrupt && !self.msix {
            self.wr(REG_INTMC, VECTOR_0);
        }
        while !finished.is_null() {
            let r = finished;
            finished = (*r).next;
            (*r).next = null_mut();
            ((*r).done)(r);
        }
    }

    /// Consume every completion that has arrived; requests now finished are
    /// linked onto `finished`. Under the lock. Returns how many were reaped.
    unsafe fn reap(&self, io: &mut Io, finished: &mut *mut Request) -> u32 {
        let mut n = 0;
        while let Some((_, dw3)) = io.queue.peek() {
            io.queue.advance();
            n += 1;
            let cid = (dw3 & 0xFFFF) as usize;
            let Some(slot) = io.slots[..io.depth].get_mut(cid).filter(|s| !s.request.is_null()) else {
                nvprint!("controller {}: a completion for command {}, which is not in flight\n", self.number, cid);
                continue;
            };
            let r = &mut *slot.request;
            slot.request = null_mut();
            io.in_flight -= 1;
            r.outstanding -= 1;
            if slot.bounce {
                if r.op == Op::Read && status(dw3).is_none() {
                    core::ptr::copy_nonoverlapping(self.bounce, r.system.add(slot.offset), slot.bytes);
                }
                io.bounce_busy = false;
            }
            if let Some((sct, sc)) = status(dw3) {
                r.error.get_or_insert(Error::Status(r.op.name(), sct, sc));
            }
            if r.finished() {
                if r.queued {
                    // Only the head is ever part-way issued.
                    Self::pop(io);
                }
                r.next = *finished;
                *finished = r;
            }
        }
        if n > 0 {
            self.ring(io.queue.id, true, io.queue.head);
            (*self.irq).next[1].store(io.queue.position(), Ordering::Release);
        }
        n
    }

    /// Keep score of who found the completions, and say on the debug port the
    /// first time each way happens: the first by interrupt proves the line
    /// works; the first by the timer means one went missing.
    fn tally(&self, io: &mut Io, interrupt: bool, reaped: u32) {
        let s = &mut io.stats;
        if interrupt {
            s.by_interrupt += reaped;
            if s.by_interrupt == reaped && self.interrupts {
                nvprint!("controller {}: interrupts working ({} completion(s) on the first)\n", self.number, reaped);
            }
        } else {
            s.by_poll += reaped;
            if s.by_poll == reaped && self.interrupts {
                nvprint!(
                    "controller {}: {} completion(s) came with no interrupt; the {} ms poll found them ({} by interrupt so far)\n",
                    self.number,
                    reaped,
                    POLL_MS,
                    s.by_interrupt
                );
            }
        }
    }

    /// Report each command in flight past the timeout, once (see the module
    /// comment for why it stays in flight).
    fn check_overdue(&self, io: &mut Io) {
        let now = now_us();
        for (cid, s) in io.slots[..io.depth].iter_mut().enumerate() {
            if !s.request.is_null() && !s.warned && now.saturating_sub(s.started) > COMMAND_TIMEOUT_US {
                s.warned = true;
                let r = unsafe { &*s.request };
                nvprint!(
                    "controller {}: command {} ({} of {} bytes at block {}) has not completed in {} s; CSTS {:#x}\n",
                    self.number,
                    cid,
                    r.op.name(),
                    s.bytes,
                    r.lba + (s.offset >> r.block_shift) as u64,
                    COMMAND_TIMEOUT_US / 1_000_000,
                    self.rd(REG_CSTS)
                );
            }
        }
    }

    pub fn stats(&self) -> Stats {
        let _g = unsafe { SpinLockGuard::acquire(self.lock.get()) };
        unsafe { (*self.io).stats }
    }

    /// Under MSI-X: the interrupts taken on the admin and I/O messages.
    pub fn message_counts(&self) -> Option<(u32, u32)> {
        if self.irq.is_null() || !self.msix {
            return None;
        }
        let m = unsafe { &(*self.irq).messages };
        Some((m[0].load(Ordering::Relaxed), m[1].load(Ordering::Relaxed)))
    }
}
