//! One NVMe controller: the admin queue, and an I/O queue pair per CPU (one per
//! MSI-X message the PnP manager assigned past the admin's, up to
//! [`MAX_IO_QUEUES`]; one on the pin), each many commands deep with a DMA bounce
//! buffer of its own.
//!
//! Admin commands run one at a time and are polled. They are issued at START,
//! before the interrupt is armed, and again by a reset, while the I/O path is
//! held; so they never meet it.
//!
//! I/O is asynchronous. A caller hands [`Controller::submit`] a [`Request`] (a
//! read or write of any length, or a flush) and gets its `done` callback, at or
//! below DISPATCH_LEVEL, once all of it has completed. It goes on the queue of
//! the CPU it was submitted on, whose completion message the HAL delivers to that
//! CPU, so a CPU submits, takes the interrupt and completes without touching
//! another CPU's queue or lock. Each queue cuts a request into commands of at most
//! `max_transfer` bytes and keeps up to one per [`Slot`] in flight, the slot index
//! serving as the command ID. A request that cannot start yet waits in its
//! queue's FIFO, so a queue starts requests in the order they were submitted, and
//! only the one at its head is ever part-way issued. A flush starts only once
//! everything ahead of it on its queue has completed. Writes still in flight on
//! another CPU's queue are not waited for: NVMe's Flush covers the writes that
//! completed before it was submitted, which is also all Windows promises for a
//! flush (FlushFileBuffers flushes what its writes completed).
//!
//! Data moves by DMA straight to and from the requester's pages, through PRP
//! entries (4.1.1) built from its MDL, whenever the MDL is one whose page list
//! can be trusted (locked, nonpaged or partial) and the data starts on a dword,
//! as PRP1's offset must. Anything else goes through the bounce buffer, one
//! command at a time; its PRP list is built once, at start.
//!
//! Completions are reaped in the queue's DPC, queued by its ISR, and by a 10 ms
//! timer that runs while anything is in flight on any queue. With an interrupt connected the timer
//! reaps only a completion it already saw at its previous tick, so it cannot
//! steal one from a working interrupt; when it does reap, the interrupt went
//! missing, and it says so on the debug port the first time. With none
//! connected the timer is all there is.
//!
//! Two kinds of interrupt. MSI-X, when the PnP manager assigned messages: the
//! admin queue raises message 0 and I/O queue q message q + 1 (all of them 0 when
//! there is only one message, and then there is one I/O queue), each an
//! exclusive, edge-triggered vector, so the ISR has nothing to check or mask.
//! Otherwise the pin (INTx) and one I/O queue: both queues raise vector 0, which
//! stays masked (INTMS) whenever nothing is connected to it, so the line can never
//! be left asserted on a pin another device shares; the ISR masks it and the DPC
//! unmasks it once the queue is drained. The spec forbids touching INTMS/INTMC
//! under MSI-X, so the controller is told which it has before bring-up.
//!
//! A command that has not completed in 30 s resets the controller, from a work
//! item at PASSIVE_LEVEL. Clearing CC.EN stops every command and every DMA, so
//! only then can the requests in flight be handed back: each fails with
//! [`Error::Reset`], which the disk class driver retries. The queues are then
//! rebuilt and the requests that waited start. A controller that will not come
//! back fails everything from then on.
//!
//! Register and command layouts are from the NVM Express Base Specification
//! (2.0); the section numbers below are that document's.

use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{null, null_mut, read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use ntkrnl_sys::ddk::{
    ExQueueWorkItem, KeDelayExecutionThread, KeInitializeDpc, KeInitializeTimer, KeInsertQueueDpc, KeQueryPerformanceCounter,
    KeQueryActiveProcessors, KeSetTimer, KeStallExecutionProcessor, ExAllocatePoolWithTag, MmAllocateContiguousMemorySpecifyCache, MmFreeContiguousMemory,
    MmGetPhysicalAddress, _MEMORY_CACHING_TYPE_MmCached, _WORK_QUEUE_TYPE_DelayedWorkQueue, BOOLEAN, KDPC, KTIMER, LARGE_INTEGER,
    MDL, PHYSICAL_ADDRESS, PKINTERRUPT, PMDL, PROCESSOR_NUMBER, PVOID, ULONG, WORK_QUEUE_ITEM,
};
use ntkrnl_sys::guards::SpinLockGuard;
use ntkrnl_sys::ke::KSPIN_LOCK;

use crate::print::nvprint;

unsafe extern "system" {
    fn KeGetCurrentProcessorNumberEx(ProcNumber: *mut PROCESSOR_NUMBER) -> ULONG;
}

pub const PAGE: usize = 4096;
/// Each queue's bounce buffer, and so the most one command moves (before MDTS).
const BOUNCE: usize = 128 * 1024;
const ADMIN_ENTRIES: u16 = 16;
/// I/O queue entries. A queue of N entries holds N - 1 commands, so this many
/// less one is the deepest an I/O queue gets: one command per slot.
const IO_ENTRIES: u16 = 64;
pub const SLOTS: usize = IO_ENTRIES as usize - 1;
/// I/O queue pairs at most: one per CPU, up to this many.
pub const MAX_IO_QUEUES: usize = 8;
/// CPUs a queue can be chosen for; any past this share the queues by number.
const MAX_CPUS: usize = 64;
const NONPAGED_POOL: i32 = 0;
const IO_POOL_TAG: u32 = u32::from_le_bytes(*b"NVMq");
/// Each slot's PRP list: room for a whole command starting anywhere in a page
/// (BOUNCE / PAGE + 1 pages, less the one PRP1 names), and a power of two, so
/// that no list crosses a page boundary (4.1.1 forbids it without chaining).
const PRP_LIST: usize = 512;
const _: () = assert!((BOUNCE / PAGE) * 8 <= PRP_LIST && PAGE % PRP_LIST == 0);
const SLOT_PRP_PAGES: usize = (SLOTS * PRP_LIST).div_ceil(PAGE);
/// One queue's memory: SQ, CQ, the bounce buffer's PRP list, the slots' PRP lists,
/// then the bounce buffer.
const QUEUE_TABLES: usize = 3 + SLOT_PRP_PAGES;
const SQE: usize = 64;
const CQE: usize = 16;
/// A command that has not completed in this long has failed: an admin command's
/// caller is told, and an I/O command resets the controller.
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

/// The test rig's `NVME_LOSE_COMPLETION=<n>` (pnp.rs reads it): the n-th I/O
/// completion is consumed and thrown away, as if the controller had never posted
/// it, so its command hangs and the reset path runs. 0: off.
pub static LOSE_COMPLETION: AtomicU32 = AtomicU32::new(0);
/// The test rig's `NVME_BAD_DMA=<hex>` (pnp.rs reads it): bring-up sends one more
/// Identify whose data goes to this address, which is not memory, so a translating
/// SMMU can be seen to fault it. 0: off.
pub static BAD_DMA: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static REAPED: AtomicU32 = AtomicU32::new(0);

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
    /// The controller was reset with the command in flight (or did not come back
    /// from a reset); a retry may succeed.
    Reset,
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
    fn new(id: u16, entries: u16, sq: *mut u8, cq: *mut u8) -> Self {
        let phys = |p: *mut u8| unsafe { MmGetPhysicalAddress(p.cast()).QuadPart } as u64;
        Queue { id, entries, sq, sq_phys: phys(sq), cq, cq_phys: phys(cq), tail: 0, head: 0, phase: 1 }
    }

    /// Back to empty, as a controller that has just been enabled sees it: both
    /// rings at 0, and no completion with the phase the next one will carry.
    fn clear(&mut self) {
        unsafe { core::ptr::write_bytes(self.cq, 0, self.entries as usize * CQE) };
        self.tail = 0;
        self.head = 0;
        self.phase = 1;
    }

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
    /// Runs at or below DISPATCH_LEVEL once every command of the request has
    /// completed.
    pub done: unsafe fn(*mut Request),
    pub context: *mut c_void,
    /// The first command to fail, if one did; the rest were not issued.
    pub error: Option<Error>,
    // The engine's, under its queue's lock.
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

/// One I/O queue pair and everything its submitters, its DPC and the timer
/// share, under its own lock. In nonpaged pool for the life of the system.
struct Io {
    lock: UnsafeCell<KSPIN_LOCK>,
    controller: *const Controller,
    index: usize,
    queue: Queue,
    /// The completion message this queue raises (MSI-X), or vector 0.
    vector: u16,
    slots: [Slot; SLOTS],
    /// Slots usable: fewer than SLOTS when the controller's queues are shallower.
    depth: usize,
    in_flight: usize,
    /// This queue's bounce buffer, and the PRP list page describing it.
    bounce: *mut u8,
    bounce_phys: u64,
    bounce_prp_phys: u64,
    bounce_busy: bool,
    /// The FIFO of requests not yet fully issued.
    head: *mut Request,
    tail: *mut Request,
    /// With an interrupt connected: where the timer saw a completion waiting
    /// at its last tick (`Queue::position`), or u32::MAX.
    poll_mark: u32,
    stats: Stats,
    /// Queued by this queue's message.
    dpc: KDPC,
}

#[derive(Clone, Copy, Default)]
pub struct Stats {
    /// Completions the ISR's DPC reaped, and those the timer had to.
    pub by_interrupt: u32,
    pub by_poll: u32,
    /// The most commands in flight at once on one queue.
    pub peak: u32,
    /// Commands whose data moved by DMA to the requester's pages, and through the bounce buffer.
    pub direct: u32,
    pub bounced: u32,
    /// Controller resets, and the commands they failed back.
    pub resets: u32,
    pub reset_failed: u32,
}

pub struct Controller {
    regs: *mut u8,
    doorbell_stride: usize,
    ready_timeout_ms: u64,
    admin: Queue,
    /// A page for identify data.
    identify: *mut u8,
    identify_phys: u64,
    next_cid: u16,
    pub max_transfer: usize,
    pub identity: Identity,
    /// Interrupts are MSI-X messages, not the pin: INTMS/INTMC are off limits.
    msix: bool,
    /// The I/O queues, and which one each CPU submits to.
    ios: [*mut Io; MAX_IO_QUEUES],
    io_count: usize,
    cpu_queue: [u8; MAX_CPUS],
    /// Which queue each MSI-X message completes (u8::MAX: none, the admin's alone).
    message_queue: [u8; MAX_IO_QUEUES + 1],
    /// Null until [`Controller::arm`].
    irq: *mut Irq,
    /// An interrupt is connected (else the timer alone reaps).
    interrupts: bool,
    /// The controller's number, for the debug port.
    number: u32,
    timer_set: AtomicBool,
    /// A reset is queued or running: nothing is issued until it is done.
    resetting: AtomicBool,
    /// A reset failed: every request fails.
    dead: AtomicBool,
    resets: AtomicU32,
    /// The first completion by interrupt has been reported.
    announced: AtomicBool,
    reset_item: UnsafeCell<WORK_QUEUE_ITEM>,
}

// Started on one thread, then shared by the submitters, the DPCs, the timer and
// the reset worker, which touch a queue's state only under its lock.
unsafe impl Send for Controller {}
unsafe impl Sync for Controller {}

/// Pin-based interrupt vector 0, the one both completion queues raise.
const VECTOR_0: u32 = 1;

/// What the interrupt path shares with the I/O engine. Lives in the FDO's
/// extension, so its address is stable for the ISRs, the DPCs and the timer.
pub struct Irq {
    regs: *mut u8,
    /// Admin and (first) I/O completion queues, and for each the slot the next
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
    /// Under MSI-X: interrupts per message (admin, then each I/O queue's).
    pub messages: [AtomicU32; MAX_IO_QUEUES + 1],
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
            messages: [const { AtomicU32::new(0) }; MAX_IO_QUEUES + 1],
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
/// to mask. It queues its I/O queue's DPC, on this CPU, the one the queue's
/// submitters run on. One that lands before the controller is armed (the admin
/// commands at START, which poll), or the admin queue's own, is just counted.
pub unsafe extern "system" fn message_isr(_interrupt: PKINTERRUPT, context: PVOID, message: ULONG) -> BOOLEAN {
    let irq = &*(context as *const Irq);
    let m = (message as usize).min(MAX_IO_QUEUES);
    irq.messages[m].fetch_add(1, Ordering::Relaxed);
    if irq.armed.load(Ordering::Acquire) {
        let c = &*irq.controller;
        if let Some(&io) = c.ios.get(c.message_queue[m] as usize) {
            KeInsertQueueDpc(&raw mut (*io).dpc, null_mut(), null_mut());
        }
    }
    1
}

/// The pin interrupt's DPC (`DeferredContext` is the [`Irq`]): the one I/O queue.
pub unsafe extern "system" fn dpc(_dpc: *mut KDPC, context: PVOID, _a: PVOID, _b: PVOID) {
    let irq = &*(context as *const Irq);
    if irq.armed.load(Ordering::Acquire) {
        let c = &*irq.controller;
        c.service(&mut *c.ios[0], true);
    }
}

/// A queue's message DPC (`DeferredContext` is its [`Io`]).
unsafe extern "system" fn queue_dpc(_dpc: *mut KDPC, context: PVOID, _a: PVOID, _b: PVOID) {
    let io = &mut *(context as *mut Io);
    (*io.controller).service(io, true);
}

/// The completion timer's DPC (`DeferredContext` is the [`Irq`]): every queue.
unsafe extern "system" fn timer_dpc(_dpc: *mut KDPC, context: PVOID, _a: PVOID, _b: PVOID) {
    let irq = &*(context as *const Irq);
    let c = &*irq.controller;
    c.timer_set.store(false, Ordering::Release);
    for &io in &c.ios[..c.io_count] {
        c.service(&mut *io, false);
    }
}

/// The reset work item (`Parameter` is the [`Controller`]), at PASSIVE_LEVEL.
unsafe extern "system" fn reset_worker(parameter: PVOID) {
    Controller::reset(parameter as *mut Controller);
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

/// Physically contiguous, cached memory the controller can address. Cached:
/// PCIe DMA snoops the CPU caches on every machine this targets (x86, QEMU virt,
/// Graviton), as it must for the requesters' own pages.
unsafe fn contiguous(size: usize) -> *mut u8 {
    let low: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut high: PHYSICAL_ADDRESS = core::mem::zeroed();
    let boundary: PHYSICAL_ADDRESS = core::mem::zeroed();
    // NVMe addresses all 64 bits; i386 kernels map no more than 4 GiB.
    high.QuadPart = if cfg!(target_pointer_width = "64") { 0x0000_FFFF_FFFF_FFFF } else { 0xFFFF_FFFF };
    MmAllocateContiguousMemorySpecifyCache(size as _, low, high, boundary, _MEMORY_CACHING_TYPE_MmCached) as *mut u8
}

/// One I/O queue pair's memory and state, built in place in pool: with its
/// slots it is kilobytes, and i386's kernel stack is 12 KiB all told.
unsafe fn new_io(index: usize, entries: u16, vector: u16) -> Result<*mut Io, Error> {
    let base = contiguous(QUEUE_TABLES * PAGE + BOUNCE);
    if base.is_null() {
        return Err(Error::NoMemory);
    }
    let io = ExAllocatePoolWithTag(NONPAGED_POOL as _, size_of::<Io>() as _, IO_POOL_TAG) as *mut Io;
    if io.is_null() {
        MmFreeContiguousMemory(base.cast());
        return Err(Error::NoMemory);
    }
    core::ptr::write_bytes(base, 0, QUEUE_TABLES * PAGE);
    core::ptr::write_bytes(io.cast::<u8>(), 0, size_of::<Io>());
    let phys = |p: *mut u8| MmGetPhysicalAddress(p.cast()).QuadPart as u64;
    let bounce_prp = base.add(2 * PAGE).cast::<u64>();
    let slot_prps = base.add(3 * PAGE);
    let bounce = base.add(QUEUE_TABLES * PAGE);
    (&raw mut (*io).index).write(index);
    (&raw mut (*io).queue).write(Queue::new(index as u16 + 1, entries, base, base.add(PAGE)));
    (&raw mut (*io).vector).write(vector);
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
    (&raw mut (*io).depth).write(entries as usize - 1);
    (&raw mut (*io).bounce).write(bounce);
    (&raw mut (*io).bounce_phys).write(phys(bounce));
    (&raw mut (*io).bounce_prp_phys).write(phys(bounce_prp.cast()));
    (&raw mut (*io).poll_mark).write(u32::MAX);
    // The bounce buffer's PRP list (4.1.1): entry i is its page i + 1, since
    // PRP1 always names page 0. The pages are physically contiguous.
    for i in 0..BOUNCE / PAGE - 1 {
        *bounce_prp.add(i) = (*io).bounce_phys + ((i + 1) * PAGE) as u64;
    }
    Ok(io)
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
    /// least `len` bytes) and bring it up with its I/O queues. `targets` holds,
    /// per MSI-X message connected, the processors it is delivered to (empty: the
    /// pin): message 0 is the admin queue's, and each further one gets an I/O
    /// queue, for the CPUs it targets.
    ///
    /// # Safety
    /// `regs` must map the controller's registers for `len` bytes.
    pub unsafe fn start(regs: *mut u8, len: usize, targets: &[u64]) -> Result<Controller, Error> {
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
        let wanted = if targets.len() >= 2 { (targets.len() - 1).min(MAX_IO_QUEUES) } else { 1 };
        if DOORBELLS + 2 * (wanted + 1) * doorbell_stride > len {
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

        // Admin SQ, admin CQ, the identify page.
        let base = contiguous(3 * PAGE);
        if base.is_null() {
            return Err(Error::NoMemory);
        }
        core::ptr::write_bytes(base, 0, 3 * PAGE);
        let mut c = Controller {
            regs,
            doorbell_stride,
            ready_timeout_ms,
            admin: Queue::new(0, admin_entries, base, base.add(PAGE)),
            identify: base.add(2 * PAGE),
            identify_phys: MmGetPhysicalAddress(base.add(2 * PAGE).cast()).QuadPart as u64,
            next_cid: 0,
            max_transfer: BOUNCE,
            identity: core::mem::zeroed(),
            msix: !targets.is_empty(),
            ios: [null_mut(); MAX_IO_QUEUES],
            io_count: 0,
            cpu_queue: [0; MAX_CPUS],
            message_queue: [u8::MAX; MAX_IO_QUEUES + 1],
            irq: null_mut(),
            interrupts: false,
            number: 0,
            timer_set: AtomicBool::new(false),
            resetting: AtomicBool::new(false),
            dead: AtomicBool::new(false),
            resets: AtomicU32::new(0),
            announced: AtomicBool::new(false),
            reset_item: UnsafeCell::new(core::mem::zeroed()),
        };
        c.enable()?;
        c.identity = c.identify_controller()?;
        let bad = BAD_DMA.load(Ordering::Relaxed);
        if bad != 0 {
            let cmd = Command::new(ADMIN_IDENTIFY, 0).prp(bad, 0).dw(10, CNS_CONTROLLER);
            match c.admin_command("tripwire identify", cmd) {
                Ok(_) => nvprint!("tripwire: an Identify to {:#x} completed\n", bad),
                Err(e) => nvprint!("tripwire: an Identify to {:#x} failed: {:?}\n", bad, e),
            }
        }
        // MDTS (byte 77): the largest transfer, in units of the minimum page size.
        let mdts = *c.identify.add(77);
        if mdts != 0 && (PAGE << mdts) < c.max_transfer {
            c.max_transfer = PAGE << mdts;
        }

        // The queue count is a request the controller may round down (5.27.1.7:
        // both counts zero-based, submission in 15:0, completion in 31:16).
        let ask = (wanted as u32 - 1) | (wanted as u32 - 1) << 16;
        let granted = match c.admin_command("set features (queues)", Command::new(ADMIN_SET_FEATURES, 0).dw(10, FEATURE_NUMBER_OF_QUEUES).dw(11, ask)) {
            Ok(dw0) => ((dw0 & 0xFFFF).min(dw0 >> 16) + 1) as usize,
            Err(_) => 1,
        };
        let count = wanted.min(granted).max(1);
        for q in 0..count {
            // Under MSI-X queue q raises message q + 1 when there is one, so its
            // interrupts are its own; on the pin, vector 0.
            let vector = if targets.len() >= 2 { q as u16 + 1 } else { 0 };
            c.ios[q] = new_io(q, io_entries, vector)?;
            c.io_count = q + 1;
            if c.msix {
                c.message_queue[vector as usize] = q as u8;
            }
        }
        // Each CPU submits to the queue whose message is delivered to it; one no
        // message targets shares a queue by number.
        for cpu in 0..MAX_CPUS {
            let mine = (0..count).find(|&q| targets.get((*c.ios[q]).vector as usize).is_some_and(|&t| t & (1 << cpu) != 0));
            c.cpu_queue[cpu] = mine.unwrap_or(cpu % count) as u8;
        }
        c.create_io_queues()?;
        c.cache_off();
        Ok(c)
    }

    /// Program the admin queue and enable the controller (3.5.1). It must be
    /// disabled, and the admin queue empty.
    fn enable(&mut self) -> Result<(), Error> {
        // Disable (firmware may have left it running).
        if self.rd(REG_CC) & CC_EN != 0 {
            self.wr(REG_CC, self.rd(REG_CC) & !CC_EN);
        }
        self.wait_ready(false)?;
        // Mask every pin-based vector until an interrupt is connected to it.
        if !self.msix {
            self.wr(REG_INTMS, u32::MAX);
        }
        let entries = self.admin.entries as u32 - 1;
        self.wr(REG_AQA, (entries << 16) | entries);
        self.wr64(REG_ASQ, self.admin.sq_phys);
        self.wr64(REG_ACQ, self.admin.cq_phys);
        self.wr(REG_CC, CC_IOSQES_IOCQES | CC_EN); // NVM command set, 4 KiB pages, round robin
        self.wait_ready(true)
    }

    /// Create each I/O queue pair: completion queue first, physically
    /// contiguous, interrupts enabled on its vector.
    fn create_io_queues(&mut self) -> Result<(), Error> {
        for q in 0..self.io_count {
            let io = unsafe { &*self.ios[q] };
            let (id, size) = (io.queue.id as u32, io.queue.entries as u32 - 1);
            let cq = Command::new(ADMIN_CREATE_CQ, 0)
                .prp(io.queue.cq_phys, 0)
                .dw(10, (size << 16) | id)
                .dw(11, (io.vector as u32) << 16 | 1 << 1 | 1);
            self.admin_command("create I/O completion queue", cq)?;
            let sq = Command::new(ADMIN_CREATE_SQ, 0)
                .prp(io.queue.sq_phys, 0)
                .dw(10, (size << 16) | id)
                .dw(11, id << 16 | 1);
            self.admin_command("create I/O submission queue", sq)?;
        }
        Ok(())
    }

    fn cache_off(&mut self) {
        if self.identity.volatile_cache {
            // Every completed write is then durable, whatever disk.sys does
            // about flushing: a cache-flush policy is not this driver's to get
            // wrong yet. SYNCHRONIZE CACHE still flushes, should this fail.
            let off = Command::new(ADMIN_SET_FEATURES, 0).dw(10, FEATURE_VOLATILE_WRITE_CACHE);
            if let Err(e) = self.admin_command("disable the write cache", off) {
                nvprint!("the volatile write cache stays on: {:?}\n", e);
            }
        }
    }

    /// The controller's number, for the debug port.
    pub fn number(&self) -> u32 {
        self.number
    }

    /// How many commands one I/O queue holds at once.
    pub fn depth(&self) -> usize {
        unsafe { (*self.ios[0]).depth }
    }

    /// The I/O queues, and for each its message (0 on the pin) and the CPUs that
    /// submit to it, for the log.
    pub fn queues(&self) -> impl Iterator<Item = (usize, u16, u64)> + '_ {
        (0..self.io_count).map(|q| {
            let active = unsafe { KeQueryActiveProcessors() } as u64;
            let cpus = (0..MAX_CPUS).filter(|&c| self.cpu_queue[c] as usize == q).fold(0u64, |m, c| m | 1 << c) & active;
            (q, unsafe { (*self.ios[q]).vector }, cpus)
        })
    }

    /// Open the I/O path. The controller must be where it will stay (its
    /// address goes to the DPCs and the reset work item), every admin command
    /// must be done (they poll, and would race the DPC for the pin), and when
    /// `interrupts` is set, `irq`'s interrupt must be connected with `irq` as its
    /// context.
    ///
    /// # Safety
    /// `irq` must point to a live [`Irq`] that outlives the controller, and
    /// neither may move from here on.
    pub unsafe fn arm(&mut self, irq: *mut Irq, number: u32, interrupts: bool) {
        self.number = number;
        self.irq = irq;
        self.interrupts = interrupts;
        for &io in &self.ios[..self.io_count] {
            (*io).controller = self;
            KeInitializeDpc(&raw mut (*io).dpc, Some(queue_dpc), io.cast());
        }
        let this: *mut Controller = self;
        let item = self.reset_item.get();
        (*item).WorkerRoutine = Some(reset_worker);
        (*item).Parameter = this.cast();
        (*irq).regs = self.regs;
        (*irq).cqs = [self.admin.cq, (*self.ios[0]).queue.cq];
        self.publish_positions();
        (*irq).controller = self;
        KeInitializeTimer(&raw mut (*irq).timer);
        KeInitializeDpc(&raw mut (*irq).timer_dpc, Some(timer_dpc), irq.cast());
        (*irq).armed.store(true, Ordering::Release);
        if interrupts && !self.msix {
            self.wr(REG_INTMC, VECTOR_0);
        }
    }

    /// Where the pin ISR looks for news: the admin queue and the one I/O queue.
    unsafe fn publish_positions(&self) {
        (*self.irq).next[0].store(self.admin.position(), Ordering::Release);
        (*self.irq).next[1].store((*self.ios[0]).queue.position(), Ordering::Release);
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
        let cmd = Command::new(ADMIN_IDENTIFY, 0).prp(self.identify_phys, 0).dw(10, CNS_CONTROLLER);
        self.admin_command("identify controller", cmd)?;
        let b = |i: usize| unsafe { read_volatile(self.identify.add(i)) };
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
        let cmd = Command::new(ADMIN_IDENTIFY, 0).prp(self.identify_phys, 0).dw(10, CNS_ACTIVE_NAMESPACES);
        let mut n = 0;
        if self.admin_command("list active namespaces", cmd).is_ok() {
            // A zero-terminated list of NSIDs, ascending.
            while n < out.len() && n < PAGE / 4 {
                let id = unsafe { read_volatile(self.identify.cast::<u32>().add(n)) };
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
        let cmd = Command::new(ADMIN_IDENTIFY, id).prp(self.identify_phys, 0).dw(10, CNS_NAMESPACE);
        self.admin_command("identify namespace", cmd)?;
        let b = |i: usize| unsafe { read_volatile(self.identify.add(i)) };
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
    /// dword 0. Only before [`Controller::arm`], or from a reset: see the module
    /// comment.
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

    /// Start `r` (at <= DISPATCH_LEVEL, after [`Controller::arm`]), on the queue
    /// of the CPU this runs on. Its `done` runs once it has completed, never from
    /// inside this call, unless a failed reset has left the controller dead.
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
        if self.dead.load(Ordering::Acquire) {
            req.error = Some(Error::Reset);
            (req.done)(r);
            return;
        }
        let cpu = KeGetCurrentProcessorNumberEx(null_mut()) as usize;
        let io = &mut *self.ios[self.cpu_queue[cpu % MAX_CPUS] as usize];
        let _g = SpinLockGuard::acquire(io.lock.get());
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
    /// them, then ring the doorbell once for the lot. Under the queue's lock.
    /// Nothing is issued while a reset is pending: the queue is about to go.
    unsafe fn pump(&self, io: &mut Io) {
        let mut issued = false;
        while !io.head.is_null() && !self.resetting.load(Ordering::Acquire) {
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
                    Self::direct_prps(r, slot, offset, bytes)
                } else {
                    io.stats.bounced += 1;
                    io.bounce_busy = true;
                    if r.op == Op::Write {
                        core::ptr::copy_nonoverlapping(r.system.add(offset), io.bounce, bytes);
                    }
                    let prp2 = match bytes {
                        0..=PAGE => 0,
                        _ if bytes <= 2 * PAGE => io.bounce_phys + PAGE as u64,
                        _ => io.bounce_prp_phys,
                    };
                    (io.bounce_phys, prp2)
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
        if io.in_flight > 0 && !self.timer_set.swap(true, Ordering::AcqRel) {
            let mut due: LARGE_INTEGER = core::mem::zeroed();
            due.QuadPart = -10_000 * POLL_MS;
            KeSetTimer(&raw mut (*self.irq).timer, due, &raw mut (*self.irq).timer_dpc);
        }
    }

    /// PRP1 and PRP2 for `bytes` bytes at `offset` into `r`'s data, straight
    /// from its MDL's page list (4.1.1): PRP1 the first byte, PRP2 the second
    /// page or, past two, the slot's list of the rest.
    unsafe fn direct_prps(r: &Request, slot: &mut Slot, offset: usize, bytes: usize) -> (u64, u64) {
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

    /// Reap what has completed on `io`, start what now can, and call the
    /// finished requests back. From the queue's interrupt DPC (`interrupt`) or the
    /// timer's.
    unsafe fn service(&self, io: &mut Io, interrupt: bool) {
        let mut finished: *mut Request = null_mut();
        let mut overdue = false;
        {
            let _g = SpinLockGuard::acquire(io.lock.get());
            // With an interrupt connected the timer reaps only what it saw
            // waiting last tick too: anything newer is the interrupt's.
            let reap = interrupt || !self.interrupts || {
                let waiting = io.queue.peek().is_some();
                let seen = waiting && io.poll_mark == io.queue.position();
                io.poll_mark = if waiting { io.queue.position() } else { u32::MAX };
                seen
            };
            if reap && !self.resetting.load(Ordering::Acquire) {
                let reaped = self.reap(io, &mut finished);
                io.poll_mark = u32::MAX;
                if reaped > 0 {
                    self.tally(io, interrupt, reaped);
                }
            }
            if !interrupt {
                overdue = self.check_overdue(io);
            }
            self.pump(io);
        }
        if overdue {
            self.request_reset();
        }
        // The queue is drained: the pin may raise the next one.
        if interrupt && !self.msix {
            self.wr(REG_INTMC, VECTOR_0);
        }
        Self::call_back(finished);
    }

    unsafe fn call_back(mut finished: *mut Request) {
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
            let lose = LOSE_COMPLETION.load(Ordering::Relaxed);
            if lose != 0 && REAPED.fetch_add(1, Ordering::Relaxed) + 1 == lose {
                nvprint!("controller {}: tripwire: the completion of command {} on queue {} is thrown away\n", self.number, cid, io.index);
                continue;
            }
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
                    core::ptr::copy_nonoverlapping(io.bounce, r.system.add(slot.offset), slot.bytes);
                }
                io.bounce_busy = false;
            }
            if let Some((sct, sc)) = status(dw3) {
                r.error.get_or_insert(Error::Status(r.op.name(), sct, sc));
            }
            Self::finish_if_done(io, r, finished);
        }
        if n > 0 {
            self.ring(io.queue.id, true, io.queue.head);
            if io.index == 0 {
                (*self.irq).next[1].store(io.queue.position(), Ordering::Release);
            }
        }
        n
    }

    /// Link `r` onto `finished` when nothing of it is left to do.
    unsafe fn finish_if_done(io: &mut Io, r: &mut Request, finished: &mut *mut Request) {
        if r.finished() {
            if r.queued {
                // Only the head is ever part-way issued.
                Self::pop(io);
            }
            r.next = *finished;
            *finished = r;
        }
    }

    /// Keep score of who found the completions, and say on the debug port the
    /// first time each way happens: the first by interrupt proves the line
    /// works; the first by the timer means one went missing.
    fn tally(&self, io: &mut Io, interrupt: bool, reaped: u32) {
        let s = &mut io.stats;
        if interrupt {
            s.by_interrupt += reaped;
            if self.interrupts && !self.announced.swap(true, Ordering::Relaxed) {
                nvprint!("controller {}: interrupts working ({} completion(s) on the first)\n", self.number, reaped);
            }
        } else {
            s.by_poll += reaped;
            if s.by_poll == reaped && self.interrupts {
                nvprint!(
                    "controller {}: {} completion(s) on queue {} came with no interrupt; the {} ms poll found them ({} by interrupt so far)\n",
                    self.number,
                    reaped,
                    io.index,
                    POLL_MS,
                    s.by_interrupt
                );
            }
        }
    }

    /// Report each command in flight past the timeout, once. True when one is:
    /// the controller then needs a reset.
    fn check_overdue(&self, io: &mut Io) -> bool {
        let now = now_us();
        let mut overdue = false;
        for (cid, s) in io.slots[..io.depth].iter_mut().enumerate() {
            if !s.request.is_null() && !s.warned && now.saturating_sub(s.started) > COMMAND_TIMEOUT_US {
                s.warned = true;
                overdue = true;
                let r = unsafe { &*s.request };
                nvprint!(
                    "controller {}: command {} on queue {} ({} of {} bytes at block {}) has not completed in {} s; CSTS {:#x}\n",
                    self.number,
                    cid,
                    io.index,
                    r.op.name(),
                    s.bytes,
                    r.lba + (s.offset >> r.block_shift) as u64,
                    COMMAND_TIMEOUT_US / 1_000_000,
                    self.rd(REG_CSTS)
                );
            }
        }
        overdue
    }

    /// Queue the reset, once.
    unsafe fn request_reset(&self) {
        if !self.resetting.swap(true, Ordering::AcqRel) {
            ExQueueWorkItem(self.reset_item.get(), _WORK_QUEUE_TYPE_DelayedWorkQueue);
        }
    }

    /// Reset the controller (at PASSIVE_LEVEL, from the work item, with
    /// `resetting` set so nothing new is issued): disable it, which ends every
    /// command and DMA; fail what was in flight; bring it back up with the same
    /// queues; then start what waited.
    unsafe fn reset(this: *mut Controller) {
        let c = &mut *this;
        let t0 = now_us();
        nvprint!("controller {}: resetting it\n", c.number);
        if !c.msix {
            c.wr(REG_INTMS, VECTOR_0);
        }
        c.resets.fetch_add(1, Ordering::Relaxed);
        c.wr(REG_CC, c.rd(REG_CC) & !CC_EN);
        if let Err(e) = c.wait_ready(false) {
            // Still running, so still free to DMA into the commands' pages: they
            // stay in flight, never handed back, and the controller is given up.
            nvprint!("controller {}: it would not stop ({:?}); its commands stay in flight and every new request fails
", c.number, e);
            c.dead.store(true, Ordering::Release);
            c.resetting.store(false, Ordering::Release);
            Self::fail_waiting(c);
            return;
        }

        // Nothing is in flight any more: every command fails back for a retry.
        let mut failed = 0;
        let mut finished: *mut Request = null_mut();
        for &io in &c.ios[..c.io_count] {
            let io = &mut *io;
            let _g = SpinLockGuard::acquire(io.lock.get());
            let before = failed;
            for i in 0..io.depth {
                let slot = &mut io.slots[i];
                if slot.request.is_null() {
                    continue;
                }
                let r = &mut *slot.request;
                slot.request = null_mut();
                r.outstanding -= 1;
                r.error.get_or_insert(Error::Reset);
                io.in_flight -= 1;
                failed += 1;
                Self::finish_if_done(io, r, &mut finished);
            }
            io.bounce_busy = false;
            io.poll_mark = u32::MAX;
            io.queue.clear();
            io.stats.reset_failed += failed - before;
        }
        Self::call_back(finished);

        c.admin.clear();
        let back = c.enable().and_then(|()| c.create_io_queues());
        match back {
            Ok(()) => {
                c.cache_off();
                c.publish_positions();
                if c.interrupts && !c.msix {
                    c.wr(REG_INTMC, VECTOR_0);
                }
                nvprint!(
                    "controller {}: reset in {} ms; {} command(s) failed back for a retry\n",
                    c.number,
                    (now_us() - t0) / 1000,
                    failed
                );
            }
            Err(e) => {
                nvprint!("controller {}: it did not come back from the reset ({:?}); every request fails from now on\n", c.number, e);
                c.dead.store(true, Ordering::Release);
            }
        }
        c.resetting.store(false, Ordering::Release);
        if c.dead.load(Ordering::Acquire) {
            Self::fail_waiting(c);
            return;
        }
        // What waited starts now.
        for &io in &c.ios[..c.io_count] {
            let io = &mut *io;
            let _g = SpinLockGuard::acquire(io.lock.get());
            c.pump(io);
        }
    }

    /// A dead controller: fail every request still waiting to be issued. One with
    /// commands still in flight stays with them.
    unsafe fn fail_waiting(c: &Controller) {
        let mut finished: *mut Request = null_mut();
        for &io in &c.ios[..c.io_count] {
            let io = &mut *io;
            let _g = SpinLockGuard::acquire(io.lock.get());
            while !io.head.is_null() {
                let r = &mut *io.head;
                Self::pop(io);
                r.error.get_or_insert(Error::Reset);
                if r.outstanding == 0 {
                    r.next = finished;
                    finished = r;
                }
            }
        }
        Self::call_back(finished);
    }

    pub fn stats(&self) -> Stats {
        let mut total = Stats::default();
        for &io in &self.ios[..self.io_count] {
            let _g = unsafe { SpinLockGuard::acquire((*io).lock.get()) };
            let s = unsafe { (*io).stats };
            total.by_interrupt += s.by_interrupt;
            total.by_poll += s.by_poll;
            total.peak = total.peak.max(s.peak);
            total.direct += s.direct;
            total.bounced += s.bounced;
            total.reset_failed += s.reset_failed;
        }
        total.resets = self.resets.load(Ordering::Relaxed);
        total
    }

    /// Under MSI-X: the interrupts taken on the admin message and on the I/O
    /// queues' messages together.
    pub fn message_counts(&self) -> Option<(u32, u32)> {
        if self.irq.is_null() || !self.msix {
            return None;
        }
        let m = unsafe { &(*self.irq).messages };
        let admin = m[0].load(Ordering::Relaxed);
        // One message: the admin's is the I/O queue's too.
        let io = if self.message_queue[0] != u8::MAX { admin } else { m[1..].iter().map(|c| c.load(Ordering::Relaxed)).sum() };
        Some((admin, io))
    }
}
