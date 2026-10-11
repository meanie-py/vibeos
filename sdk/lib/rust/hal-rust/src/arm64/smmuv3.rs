//! An SMMUv3 driven for real: every PCI device's DMA translated, through one stage-1
//! identity map that holds the machine's RAM and its MSI doorbells and nothing else.
//!
//! Our drivers hand their devices physical addresses (MmGetPhysicalAddress, an MDL's
//! PFNs), so the only translation that keeps them working is the identity. What it
//! buys is the edge of that map: a device that DMAs to MMIO, to a hole, or to an
//! address it made up faults at the SMMU, and the fault is reported with the
//! device's StreamID and the address, instead of landing somewhere and corrupting
//! it. Each device gets its own stream table entry when pci.sys starts it
//! ([`attach`]), before its bus mastering is turned on; a StreamID nobody attached
//! is aborted.
//!
//! A device whose driver declares itself DMA-remapping compatible (its service's
//! Parameters\DmaRemappingCompatible = 1, which pci.sys reads, as Windows does) gets
//! a domain of its own instead: its own context descriptor, ASID and page tables,
//! holding its MSI doorbells and nothing else until its DMA adapter maps a buffer
//! (dma.rs: common buffers, MapTransfer, scatter/gather lists). Addresses stay the
//! identity, so the logical address a driver is handed is still the physical one;
//! what changes is that a device can reach only the pages it was given, while it
//! holds them. A page is mapped once however many buffers share it (a count in the
//! descriptor's software bits) and unmapped, with its TLB entry invalidated, when the
//! last of them is let go.
//!
//! Opt-in: the `SMMU=TRANSLATE` tripwire. Without it the HAL bypasses every SMMU
//! (smmu.rs), which is what Windows does when DMA remapping is not asked for, and
//! what any SMMU this driver cannot run falls back to.
//!
//! Layouts: Arm IHI 0070 (SMMUv3), sections 6.3 (registers), 5.2 (STE), 5.4 (CD),
//! 4 (commands), 7.3 (events); the page tables are the VMSAv8-64 stage-1 format.

#![cfg(target_arch = "aarch64")]

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use ntkrnl_sys::ddk;

unsafe extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

// Registers, page 0 unless noted.
const IDR0: usize = 0x00;
const IDR1: usize = 0x04;
const IDR5: usize = 0x14;
const CR0: usize = 0x20;
const CR0ACK: usize = 0x24;
const CR1: usize = 0x28;
const CR2: usize = 0x2C;
const GERROR: usize = 0x60;
const STRTAB_BASE: usize = 0x80;
const STRTAB_BASE_CFG: usize = 0x88;
const CMDQ_BASE: usize = 0x90;
const CMDQ_PROD: usize = 0x98;
const CMDQ_CONS: usize = 0x9C;
const EVENTQ_BASE: usize = 0xA0;
/// Page 1.
const EVENTQ_PROD: usize = 0x1_00A8;
const EVENTQ_CONS: usize = 0x1_00AC;
const REGISTER_SPAN: usize = 0x2_0000;

const CR0_SMMUEN: u32 = 1 << 0;
const CR0_EVENTQEN: u32 = 1 << 2;
const CR0_CMDQEN: u32 = 1 << 3;
const CR2_RECINVSID: u32 = 1 << 1;
const CR2_PTM: u32 = 1 << 2;
const IDR0_S1P: u32 = 1 << 1;
const IDR0_TTF_AARCH64: u32 = 1 << 3;
const IDR5_GRAN4K: u32 = 1 << 4;
const CONS_ERR_SHIFT: u32 = 24;

/// Two-level stream table: each level-1 descriptor covers 2^SPLIT StreamIDs, whose
/// 64-byte entries (16 KiB) are allocated when the first of them is attached.
const SPLIT: u32 = 8;
const STE_BYTES: usize = 64;
/// StreamID bits the table covers at most: 2^(20 - SPLIT) level-1 descriptors.
const MAX_SID_BITS: u32 = 20;
const CMDQ_LOG2: u32 = 8;
const EVENTQ_LOG2: u32 = 7;
const MAX_UNITS: usize = 8;
/// Level-2 stream table spans one unit can hold (each a PCI bus's worth of IDs).
const MAX_SPANS: usize = 64;

// Commands.
const CMD_CFGI_STE: u64 = 0x03;
const CMD_CFGI_ALL: u64 = 0x04;
const CMD_TLBI_NH_VA: u64 = 0x12;
const CMD_TLBI_NSNH_ALL: u64 = 0x30;
const CMD_SYNC: u64 = 0x46;

// Stage-1 descriptors: AttrIndx 0 is Normal write-back memory, 1 is Device-nGnRE
// (MAIR below); AP 01 lets unprivileged accesses, which is what a PCI device
// makes, read and write; AF set, so no access-flag faults.
const MAIR: u64 = 0x04_FF;
const DESC_NORMAL: u64 = (0 << 2) | (0b01 << 6) | (0b11 << 8) | (1 << 10);
const DESC_DEVICE: u64 = (1 << 2) | (0b01 << 6) | (1 << 10);
const DESC_TABLE: u64 = 0b11;
const DESC_BLOCK: u64 = 0b01;
const DESC_PAGE: u64 = 0b11;
const ADDRESS: u64 = ((1u64 << 48) - 1) & !0xFFF;
/// Page-table pages for the identity map, allocated once.
const POOL_PAGES: usize = 64;
/// A strict domain's page-table pages: the root and its walk down to 4 KiB pages,
/// one level-3 table per 2 MiB of memory its buffers touch. Never freed, so a
/// domain that wants more is told no and its mapping fails loudly.
const DOMAIN_POOL_PAGES: usize = 128;
const PAGE: usize = 0x1000;
/// Descriptor software bits 58:55: how many mappings hold a strict domain's page.
/// At the top it stays mapped for good, which is the safe way to be wrong.
const REFS_SHIFT: u32 = 55;
const REFS_MAX: u64 = 15;
const REFS: u64 = REFS_MAX << REFS_SHIFT;
const MAX_DOMAINS: usize = 32;
/// ASID 1 is the shared identity map; strict domains take 2 up.
const FIRST_ASID: u16 = 2;

struct Unit {
    base: u64,
    va: usize,
    cmdq: usize,
    cmdq_prod: u32,
    eventq: usize,
    eventq_cons: u32,
    l1: usize,
    sid_bits: u32,
    /// (level-1 index, VA of its 16 KiB of entries).
    spans: [(u32, usize); MAX_SPANS],
    span_count: usize,
}

/// Page-table pages, carved from one contiguous allocation.
#[derive(Clone, Copy)]
struct Pool {
    va: usize,
    pa: u64,
    used: usize,
    pages: usize,
}

/// A device of its own: the stream that is its, and the tables it can reach.
struct Domain {
    /// pci.sys's requester: the PCI segment in bits 31:16 over the requester ID.
    requester: u32,
    unit: u64,
    asid: u16,
    root: usize,
    pool: Pool,
    /// Pages mapped now.
    pages: u32,
    cd: u64,
    /// Attached by a driver that found the device itself (Attach::Claimed): kept
    /// when pci.sys starts the device's PDO with no driver of its own.
    claimed: bool,
}

/// How a stream is attached (HalVibeosDmaAttach).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Attach {
    /// The shared identity map of RAM.
    Shared,
    /// A domain of its own: its driver is DMA-remapping compatible.
    Strict,
    /// The same, claimed by a driver that found the device outside PnP.
    Claimed,
}

static mut UNITS: [Option<Unit>; MAX_UNITS] = [const { None }; MAX_UNITS];
static UNIT_COUNT: AtomicUsize = AtomicUsize::new(0);
/// The context descriptor every stream without a domain of its own uses, and its
/// page tables.
static CD_PA: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static mut SHARED: Pool = Pool { va: 0, pa: 0, used: 0, pages: POOL_PAGES };
/// The first SMMU's output address size (IDR5.OAS), for every CD.
static OAS: AtomicU32 = AtomicU32::new(0);
static mut DOMAINS: [Option<Domain>; MAX_DOMAINS] = [const { None }; MAX_DOMAINS];
static LOCK: AtomicU32 = AtomicU32::new(0);
static EVENTS_REPORTED: AtomicU32 = AtomicU32::new(0);
static TIMER_ARMED: AtomicBool = AtomicBool::new(false);

struct Locked(ddk::KIRQL);
impl Locked {
    unsafe fn take() -> Self {
        let irql = unsafe { super::halstubs::KfRaiseIrql(2) };
        while LOCK.compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
        Locked(irql)
    }
}
impl Drop for Locked {
    fn drop(&mut self) {
        LOCK.store(0, Ordering::Release);
        unsafe { super::halstubs::KfLowerIrql(self.0) };
    }
}

unsafe fn rd32(a: usize) -> u32 {
    core::ptr::read_volatile(a as *const u32)
}
unsafe fn wr32(a: usize, v: u32) {
    core::ptr::write_volatile(a as *mut u32, v)
}
unsafe fn wr64(a: usize, v: u64) {
    core::ptr::write_volatile(a as *mut u64, v)
}
unsafe fn dsb() {
    core::arch::asm!("dsb sy", options(nostack));
}

/// Zeroed, physically contiguous, non-cached memory aligned to its own size (at
/// least a page): every queue and table the SMMU is handed must be. Non-cached
/// because the SMMU reads them with non-cacheable attributes (CR1 says so).
unsafe fn alloc(bytes: usize) -> Option<(usize, u64)> {
    let bytes = bytes.max(PAGE).next_power_of_two();
    let mut highest: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    highest.QuadPart = (1i64 << 48) - 1;
    let zero: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut boundary: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    boundary.QuadPart = 0;
    // Ask for enough to align within: the allocator promises only a page.
    let span = bytes * 2 - PAGE;
    let va = ddk::MmAllocateContiguousMemorySpecifyCache(
        span as ddk::SIZE_T,
        zero,
        highest,
        boundary,
        ddk::_MEMORY_CACHING_TYPE_MmNonCached,
    ) as usize;
    if va == 0 {
        return None;
    }
    let pa = ddk::MmGetPhysicalAddress(va as ddk::PVOID).QuadPart as u64;
    let skip = ((pa + bytes as u64 - 1) & !(bytes as u64 - 1)) - pa;
    let va = va + skip as usize;
    core::ptr::write_bytes(va as *mut u8, 0, bytes);
    Some((va, pa + skip))
}

/// Spin until `(reg & mask) == want`; an SMMU that never answers is reported.
unsafe fn settle(reg: usize, mask: u32, want: u32) -> bool {
    (0..1_000_000).any(|_| rd32(reg) & mask == want)
}

// ---- the identity map ----

fn table_page(pool: &mut Pool) -> Option<(usize, u64)> {
    if pool.used >= pool.pages {
        return None;
    }
    let off = pool.used * PAGE;
    pool.used += 1;
    Some((pool.va + off, pool.pa + off as u64))
}

fn pool_va(pool: &Pool, pa: u64) -> usize {
    pool.va + (pa - pool.pa) as usize
}

/// Map `addr` (aligned to the level's size) at `level` (1: 1 GiB, 2: 2 MiB, 3:
/// 4 KiB). False when a finer table already occupies the slot, so the caller
/// tries a smaller size; true when it is mapped (or a coarser block already maps
/// it).
unsafe fn map_one(pool: &mut Pool, root: usize, addr: u64, level: u32, attrs: u64) -> Option<bool> {
    let mut table = root;
    for l in 0..=level {
        let index = ((addr >> (39 - 9 * l)) & 511) as usize;
        let slot = (table + index * 8) as *mut u64;
        let entry = core::ptr::read_volatile(slot);
        if l == level {
            if entry & DESC_TABLE == DESC_TABLE && l < 3 {
                return Some(false);
            }
            if entry == 0 {
                let kind = if level == 3 { DESC_PAGE } else { DESC_BLOCK };
                core::ptr::write_volatile(slot, addr | attrs | kind);
            }
            return Some(true);
        }
        match entry & 3 {
            0 => {
                let (va, pa) = table_page(pool)?;
                core::ptr::write_volatile(slot, pa | DESC_TABLE);
                table = va;
            }
            DESC_TABLE => table = pool_va(pool, entry & ADDRESS),
            _ => return Some(true), // a block above already maps it
        }
    }
    Some(true)
}

/// Identity-map [start, end), in the largest blocks that fit.
unsafe fn map_range(pool: &mut Pool, root: usize, start: u64, end: u64, attrs: u64) -> bool {
    let mut addr = start & !0xFFF;
    let end = (end + 0xFFF) & !0xFFF;
    while addr < end {
        let mut done = 0;
        for (level, size) in [(1u32, 1u64 << 30), (2, 1 << 21), (3, 1 << 12)] {
            if addr % size == 0 && addr + size <= end {
                match map_one(pool, root, addr, level, attrs) {
                    Some(true) => {
                        done = size;
                        break;
                    }
                    Some(false) => {}
                    None => return false, // pool exhausted
                }
            }
        }
        if done == 0 {
            return false;
        }
        addr += done;
    }
    true
}

/// The shared context descriptor and its page tables: RAM as Normal memory, the
/// MSI doorbells (where a device's interrupt writes go) as Device memory.
unsafe fn build_map(oas: u32) -> bool {
    let Some((pool_va, pool_pa)) = alloc(POOL_PAGES * PAGE) else {
        return false;
    };
    OAS.store(oas, Ordering::Relaxed);
    let pool = &mut *(&raw mut SHARED);
    *pool = Pool { va: pool_va, pa: pool_pa, used: 0, pages: POOL_PAGES };
    let Some((root, root_pa)) = table_page(pool) else {
        return false;
    };
    let ranges = ddk::MmGetPhysicalMemoryRanges();
    if ranges.is_null() {
        return false;
    }
    let mut ram = 0u64;
    let mut ok = true;
    let mut r = ranges;
    while (*r).BaseAddress.QuadPart != 0 || (*r).NumberOfBytes.QuadPart != 0 {
        let (start, bytes) = ((*r).BaseAddress.QuadPart as u64, (*r).NumberOfBytes.QuadPart as u64);
        ok &= map_range(pool, root, start, start + bytes, DESC_NORMAL);
        ram += bytes;
        r = r.add(1);
    }
    ddk::ExFreePool(ranges as ddk::PVOID);
    let doorbells = map_doorbells(pool, root);
    ok &= doorbells.is_some();
    if !ok {
        DbgPrint(b"[HAL] SMMUv3: the identity map did not fit its %u table pages\n\0".as_ptr(), POOL_PAGES as u32);
        return false;
    }
    let Some(cd_pa) = context_descriptor(root_pa, 1) else {
        return false;
    };
    CD_PA.store(cd_pa, Ordering::Release);
    DbgPrint(
        b"[HAL] SMMUv3: identity map of %llu MiB of RAM and %u MSI doorbell page(s), %u table pages\n\0".as_ptr(),
        ram >> 20,
        doorbells.unwrap_or(0),
        pool.used as u32,
    );
    true
}

/// Map every MSI doorbell page (where a device's interrupt writes go) as Device
/// memory; how many, or None when the pool ran out.
unsafe fn map_doorbells(pool: &mut Pool, root: usize) -> Option<u32> {
    let mut doorbells = 0u32;
    for pa in super::its::doorbell_pages().into_iter().flatten().chain(super::v2m::doorbell_page()) {
        if !map_range(pool, root, pa, pa + PAGE as u64, DESC_DEVICE) {
            return None;
        }
        doorbells += 1;
    }
    Some(doorbells)
}

/// A context descriptor for the tables at `root_pa`, tagged `asid`: T0SZ 16 (48-bit
/// input), 4 KiB granule, non-cacheable walks, TTB1 off (EPD1), valid, IPS from the
/// SMMU's output size, no access-flag faults, AArch64 tables, faults recorded and
/// aborted (R, A).
unsafe fn context_descriptor(root_pa: u64, asid: u16) -> Option<u64> {
    let (cd, cd_pa) = alloc(PAGE)?;
    let oas = OAS.load(Ordering::Relaxed) as u64;
    let words: [u64; 8] = [
        16 | (1 << 30) | (1 << 31) | (oas << 32) | (1 << 35) | (1 << 41) | (1 << 45) | (1 << 46) | ((asid as u64) << 48),
        root_pa & ADDRESS,
        0,
        MAIR,
        0,
        0,
        0,
        0,
    ];
    for (i, w) in words.iter().enumerate().rev() {
        core::ptr::write_volatile((cd + i * 8) as *mut u64, *w);
    }
    dsb();
    Some(cd_pa)
}

/// The level-3 descriptor slot for `addr` in a strict domain, its tables made on the
/// way when `make`. None: not there (or no pool page left to make it).
unsafe fn leaf(pool: &mut Pool, root: usize, addr: u64, make: bool) -> Option<*mut u64> {
    let mut table = root;
    for l in 0..3 {
        let slot = (table + ((addr >> (39 - 9 * l)) & 511) as usize * 8) as *mut u64;
        let entry = core::ptr::read_volatile(slot);
        table = match entry & 3 {
            DESC_TABLE => pool_va(pool, entry & ADDRESS),
            0 if make => {
                let (va, pa) = table_page(pool)?;
                core::ptr::write_volatile(slot, pa | DESC_TABLE);
                va
            }
            _ => return None,
        };
    }
    Some((table + ((addr >> 12) & 511) as usize * 8) as *mut u64)
}

// ---- one SMMU ----

/// Queue a command and wait until the SMMU has consumed it. False on a command
/// error (CMDQ_CONS.ERR) or no progress.
unsafe fn command(u: &mut Unit, words: [u64; 2]) -> bool {
    let mask = (1u32 << CMDQ_LOG2) - 1;
    let slot = u.cmdq + ((u.cmdq_prod & mask) as usize) * 16;
    wr64(slot, words[0]);
    wr64(slot + 8, words[1]);
    dsb();
    u.cmdq_prod = (u.cmdq_prod + 1) & ((mask << 1) | 1);
    wr32(u.va + CMDQ_PROD, u.cmdq_prod);
    for _ in 0..1_000_000 {
        let cons = rd32(u.va + CMDQ_CONS);
        if (cons >> CONS_ERR_SHIFT) & 0x7F != 0 {
            DbgPrint(
                b"[HAL] SMMUv3 at %p: command %llx refused (error %u)\n\0".as_ptr(),
                u.base as ddk::PVOID,
                words[0],
                (cons >> CONS_ERR_SHIFT) & 0x7F,
            );
            return false;
        }
        if cons & ((mask << 1) | 1) == u.cmdq_prod {
            return true;
        }
        core::hint::spin_loop();
    }
    DbgPrint(b"[HAL] SMMUv3 at %p: command %llx never consumed\n\0".as_ptr(), u.base as ddk::PVOID, words[0]);
    false
}

unsafe fn sync(u: &mut Unit) -> bool {
    command(u, [CMD_SYNC, 0])
}

/// Bring one SMMUv3 up translating, with no stream attached yet.
unsafe fn bring_up(base: u64) -> Result<Unit, &'static [u8]> {
    let mut phys: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    phys.QuadPart = base as i64;
    let va = ddk::MmMapIoSpace(phys, REGISTER_SPAN as ddk::SIZE_T, ddk::_MEMORY_CACHING_TYPE_MmNonCached) as usize;
    if va == 0 {
        return Err(b"its registers could not be mapped\0");
    }
    let (idr0, idr1, idr5) = (rd32(va + IDR0), rd32(va + IDR1), rd32(va + IDR5));
    if idr0 & IDR0_S1P == 0 {
        return Err(b"it has no stage 1\0");
    }
    if (idr0 >> 27) & 3 == 0 {
        return Err(b"it has no two-level stream table\0");
    }
    if idr0 & IDR0_TTF_AARCH64 == 0 || idr5 & IDR5_GRAN4K == 0 {
        return Err(b"it cannot walk AArch64 4 KiB tables\0");
    }
    let sid_bits = (idr1 & 0x3F).clamp(SPLIT, MAX_SID_BITS);
    let cmdq_log2 = CMDQ_LOG2.min((idr1 >> 21) & 0x1F);
    let eventq_log2 = EVENTQ_LOG2.min((idr1 >> 16) & 0x1F);
    if cmdq_log2 != CMDQ_LOG2 || eventq_log2 != EVENTQ_LOG2 {
        return Err(b"its queues are smaller than this driver's\0");
    }
    if CD_PA.load(Ordering::Acquire) == 0 && !build_map(idr5 & 7) {
        return Err(b"the identity map could not be built\0");
    }

    wr32(va + CR0, 0);
    if !settle(va + CR0ACK, CR0_SMMUEN | CR0_CMDQEN | CR0_EVENTQEN, 0) {
        return Err(b"it would not disable\0");
    }
    let l1_entries = 1usize << (sid_bits - SPLIT);
    let (Some((cmdq, cmdq_pa)), Some((eventq, eventq_pa)), Some((l1, l1_pa))) =
        (alloc(16 << CMDQ_LOG2), alloc(32 << EVENTQ_LOG2), alloc((l1_entries * 8).max(PAGE)))
    else {
        return Err(b"its queues and stream table could not be allocated\0");
    };
    // Queues and tables: non-cacheable, non-shareable, as allocated.
    wr32(va + CR1, 0);
    wr32(va + CR2, CR2_RECINVSID | CR2_PTM);
    wr64(va + STRTAB_BASE, l1_pa & ((1u64 << 52) - 1) & !0x3F);
    wr32(va + STRTAB_BASE_CFG, (1 << 16) | (SPLIT << 6) | sid_bits);
    wr64(va + CMDQ_BASE, (cmdq_pa & ((1u64 << 52) - 1) & !0x1F) | CMDQ_LOG2 as u64);
    wr32(va + CMDQ_PROD, 0);
    wr32(va + CMDQ_CONS, 0);
    wr64(va + EVENTQ_BASE, (eventq_pa & ((1u64 << 52) - 1) & !0x1F) | EVENTQ_LOG2 as u64);
    wr32(va + EVENTQ_PROD, 0);
    wr32(va + EVENTQ_CONS, 0);
    dsb();
    wr32(va + CR0, CR0_CMDQEN);
    if !settle(va + CR0ACK, CR0_CMDQEN, CR0_CMDQEN) {
        return Err(b"its command queue would not start\0");
    }
    let mut u = Unit {
        base,
        va,
        cmdq,
        cmdq_prod: 0,
        eventq,
        eventq_cons: 0,
        l1,
        sid_bits,
        spans: [(0, 0); MAX_SPANS],
        span_count: 0,
    };
    if !command(&mut u, [CMD_CFGI_ALL, 31]) || !command(&mut u, [CMD_TLBI_NSNH_ALL, 0]) || !sync(&mut u) {
        return Err(b"it refused the invalidations\0");
    }
    wr32(va + CR0, CR0_CMDQEN | CR0_EVENTQEN | CR0_SMMUEN);
    if !settle(va + CR0ACK, CR0_SMMUEN, CR0_SMMUEN) {
        return Err(b"it would not enable\0");
    }
    Ok(u)
}

/// Turn translation on for one SMMUv3 the IORT lists. False to fall back to bypass,
/// with the reason said.
pub(crate) unsafe fn enable(base: u64) -> bool {
    if UNIT_COUNT.load(Ordering::Relaxed) >= MAX_UNITS {
        return false;
    }
    match bring_up(base) {
        Ok(u) => {
            let units = &raw mut UNITS;
            let n = UNIT_COUNT.fetch_add(1, Ordering::AcqRel);
            DbgPrint(
                b"[HAL] SMMUv3 at %p: TRANSLATING - %u StreamID bits, identity map, unattached streams aborted\n\0".as_ptr(),
                base as ddk::PVOID,
                u.sid_bits,
            );
            (*units)[n] = Some(u);
            arm_event_timer();
            true
        }
        Err(why) => {
            DbgPrint(b"[HAL] SMMUv3 at %p: not translating: %s\n\0".as_ptr(), base as ddk::PVOID, why.as_ptr());
            false
        }
    }
}

/// Whether `sid`'s level-2 span on the SMMU at `base` still has to be allocated.
/// Asked before LOCK is taken: the contiguous allocator cannot run at DISPATCH.
unsafe fn needs_span(base: u64, sid: u32) -> bool {
    let units = &raw const UNITS;
    (*units).iter().flatten().find(|u| u.base == base).is_some_and(|u| {
        !u.spans[..u.span_count].iter().any(|s| s.0 == sid >> SPLIT)
    })
}

/// Point StreamID `sid` on the SMMU at `base` at the context descriptor `cd`.
/// Idempotent. `fresh` is a span allocated beforehand for when the one holding
/// `sid` is missing.
unsafe fn attach_stream(base: u64, sid: u32, cd: u64, fresh: Option<(usize, u64)>) -> bool {
    let units = &raw mut UNITS;
    let Some(u) = (*units).iter_mut().flatten().find(|u| u.base == base) else {
        return true; // that SMMU is bypassing: nothing to attach
    };
    if sid >> u.sid_bits != 0 {
        DbgPrint(b"[HAL] SMMUv3 at %p: StreamID %x is past its stream table\n\0".as_ptr(), base as ddk::PVOID, sid);
        return false;
    }
    let index = sid >> SPLIT;
    let span = match u.spans[..u.span_count].iter().find(|s| s.0 == index) {
        Some(s) => s.1,
        None => {
            if u.span_count == MAX_SPANS {
                DbgPrint(b"[HAL] SMMUv3 at %p: no room for another stream table span\n\0".as_ptr(), base as ddk::PVOID);
                return false;
            }
            let Some((va, pa)) = fresh else {
                DbgPrint(b"[HAL] SMMUv3 at %p: no memory for a stream table span\n\0".as_ptr(), base as ddk::PVOID);
                return false;
            };
            // Level-1 descriptor: the span's address and Span = SPLIT + 1.
            wr64(u.l1 + index as usize * 8, (pa & ((1u64 << 52) - 1) & !0x3F) | (SPLIT as u64 + 1));
            u.spans[u.span_count] = (index, va);
            u.span_count += 1;
            va
        }
    };
    let ste = span + (sid as usize & ((1 << SPLIT) - 1)) * STE_BYTES;
    let word0 = 1 | (0b101 << 1) | (cd & ((1u64 << 52) - 1) & !0x3F);
    let now = core::ptr::read_volatile(ste as *const u64);
    if now == word0 {
        return true;
    }
    if now & 1 != 0 {
        // Another CD (the device now has a domain of its own, or no longer):
        // invalid first, so the SMMU never sees half an entry.
        wr64(ste, 0);
        dsb();
        if !command(u, [CMD_CFGI_STE | (sid as u64) << 32, 0]) || !sync(u) {
            return false;
        }
    }
    // STE: stage 1 translates (Config 0b101) through `cd`, stage 2 bypassed;
    // shareability from the incoming transaction. Valid last.
    for i in (1..8).rev() {
        wr64(ste + i * 8, if i == 1 { 1 << 44 } else { 0 });
    }
    dsb();
    wr64(ste, word0);
    dsb();
    // Leaf 0: the level-1 descriptor may be new as well.
    command(u, [CMD_CFGI_STE | (sid as u64) << 32, 0]) && sync(u)
}

/// A strict domain for `requester` behind the SMMU at `unit`: the one it already has
/// (a restarted device keeps its tables), else a new one with only its doorbells
/// mapped. Its context descriptor's address, or None.
unsafe fn domain_for(requester: u32, unit: u64) -> Option<u64> {
    let domains = &raw mut DOMAINS;
    if let Some(d) = (*domains).iter().flatten().find(|d| d.requester == requester) {
        return Some(d.cd_pa());
    }
    let slot = (*domains).iter().position(Option::is_none)?;
    let (va, pa) = alloc(DOMAIN_POOL_PAGES * PAGE)?;
    let mut pool = Pool { va, pa, used: 0, pages: DOMAIN_POOL_PAGES };
    let (root, root_pa) = table_page(&mut pool)?;
    map_doorbells(&mut pool, root)?;
    let asid = FIRST_ASID + slot as u16;
    let cd_pa = context_descriptor(root_pa, asid)?;
    let _guard = Locked::take();
    (*domains)[slot] = Some(Domain { requester, unit, asid, root, pool, pages: 0, cd: cd_pa, claimed: false });
    Some(cd_pa)
}

impl Domain {
    fn cd_pa(&self) -> u64 {
        self.cd
    }
}

/// pci.sys, starting a device, before it may master the bus: give its StreamID (on
/// whichever SMMU its PCI segment's root complex sends it to) the identity map, or
/// when `strict` (its driver is DMA-remapping compatible) a domain of its own that
/// maps nothing until its DMA adapter does. `requester` is the PCI segment in bits
/// 31:16 over the requester ID.
pub(crate) unsafe fn attach(requester: u32, mode: Attach) -> bool {
    if UNIT_COUNT.load(Ordering::Acquire) == 0 {
        return true;
    }
    let (segment, rid) = (requester >> 16, requester & 0xFFFF);
    let mut found = None;
    let xsdt = super::acpi::XSDT_PA.load(Ordering::Relaxed);
    super::acpi::for_each_table(xsdt, |sig, t, len| {
        if sig == b"IORT" && found.is_none() {
            let bytes = core::slice::from_raw_parts(t, len);
            found = super::iort::table(bytes).and_then(|t| super::iort::stream(t, super::ecam::root_complex_segment(segment), rid));
        }
    });
    let Some((smmu, sid)) = found else {
        return true; // no SMMU on its path
    };
    if (*(&raw const UNITS)).iter().flatten().all(|u| u.base != smmu.base) {
        return true; // that SMMU is bypassing
    }
    let fresh = if needs_span(smmu.base, sid) { alloc(STE_BYTES << SPLIT) } else { None };
    // A device its driver claimed keeps that domain when pci.sys starts its PDO.
    let kept = mode == Attach::Shared && (*(&raw const DOMAINS)).iter().flatten().any(|d| d.requester == requester && d.claimed);
    let strict = mode != Attach::Shared || kept;
    // Allocated here, at PASSIVE: the contiguous allocator cannot run under LOCK.
    let cd = if strict { domain_for(requester, smmu.base) } else { Some(CD_PA.load(Ordering::Acquire)) };
    if mode == Attach::Claimed && cd.is_some() {
        let _guard = Locked::take();
        if let Some(d) = (*(&raw mut DOMAINS)).iter_mut().flatten().find(|d| d.requester == requester) {
            d.claimed = true;
        }
    }
    let ok = cd.is_some_and(|cd| {
        let _guard = Locked::take();
        let ok = attach_stream(smmu.base, sid, cd, fresh);
        poll_events();
        ok
    });
    DbgPrint(
        b"[HAL] SMMUv3 at %p: requester %04x is StreamID %x - %s\n\0".as_ptr(),
        smmu.base as ddk::PVOID,
        requester,
        sid,
        match (ok, mode, kept) {
            (true, Attach::Shared, false) => b"translating\0".as_ptr(),
            (true, Attach::Shared, true) => b"translating in the domain its driver claimed\0".as_ptr(),
            (true, Attach::Strict, _) => b"translating in a domain of its own (DMA-remapping compatible)\0".as_ptr(),
            (true, Attach::Claimed, _) => {
                b"translating in a domain of its own (DMA-remapping compatible, claimed by its driver)\0".as_ptr()
            }
            (false, _, _) => b"NOT ATTACHED, its DMA will abort\0".as_ptr(),
        },
    );
    ok
}

/// Whether `requester` has a domain of its own: its DMA adapter then has to map
/// what it hands the device.
pub(crate) unsafe fn is_strict(requester: u32) -> bool {
    (*(&raw const DOMAINS)).iter().flatten().any(|d| d.requester == requester)
}

/// Let `requester`'s device reach [pa, pa + bytes) (map), or stop holding it
/// (unmap). Nothing to do for a device without a domain of its own. False when
/// the domain's tables are out of pages: the device would fault on that buffer.
/// At or below DISPATCH_LEVEL.
pub(crate) unsafe fn map(requester: u32, pa: u64, bytes: u64) -> bool {
    update(requester, pa, bytes, true)
}

pub(crate) unsafe fn unmap(requester: u32, pa: u64, bytes: u64) {
    update(requester, pa, bytes, false);
}

unsafe fn update(requester: u32, pa: u64, bytes: u64, mapping: bool) -> bool {
    if bytes == 0 {
        return true;
    }
    let _guard = Locked::take();
    let domains = &raw mut DOMAINS;
    let Some(d) = (*domains).iter_mut().flatten().find(|d| d.requester == requester) else {
        return true;
    };
    let units = &raw mut UNITS;
    let Some(u) = (*units).iter_mut().flatten().find(|u| u.base == d.unit) else {
        return true;
    };
    let (first, last) = (pa & !0xFFF, (pa + bytes - 1) & !0xFFF);
    let mut ok = true;
    let mut invalidated = false;
    let mut page = first;
    while page <= last {
        let Some(slot) = leaf(&mut d.pool, d.root, page, mapping) else {
            if mapping {
                ok = false;
                DbgPrint(
                    b"[HAL] SMMUv3: requester %04x's domain has no table page left to map %p\n\0".as_ptr(),
                    requester,
                    page as ddk::PVOID,
                );
                break;
            }
            page += PAGE as u64;
            continue;
        };
        let entry = core::ptr::read_volatile(slot);
        let refs = (entry & REFS) >> REFS_SHIFT;
        if mapping {
            let next = if entry & 3 == DESC_PAGE {
                (entry & !REFS) | ((refs + 1).min(REFS_MAX) << REFS_SHIFT)
            } else {
                d.pages += 1;
                page | DESC_NORMAL | DESC_PAGE | (1 << REFS_SHIFT)
            };
            core::ptr::write_volatile(slot, next);
        } else if entry & 3 == DESC_PAGE && refs != 0 && refs != REFS_MAX {
            if refs == 1 {
                core::ptr::write_volatile(slot, 0);
                d.pages -= 1;
                dsb();
                // TLBI_NH_VA: ASID, then the address, Leaf.
                ok &= command(u, [CMD_TLBI_NH_VA | (d.asid as u64) << 48, page | 1]);
                invalidated = true;
            } else {
                core::ptr::write_volatile(slot, (entry & !REFS) | ((refs - 1) << REFS_SHIFT));
            }
        }
        page += PAGE as u64;
    }
    dsb();
    if invalidated {
        ok &= sync(u);
    }
    ok
}

// ---- faults ----

/// Report what the SMMUs have recorded since the last look. Under LOCK.
unsafe fn poll_events() {
    let units = &raw mut UNITS;
    let mask = (1u32 << EVENTQ_LOG2) - 1;
    for u in (*units).iter_mut().flatten() {
        let gerror = rd32(u.va + GERROR);
        loop {
            let prod = rd32(u.va + EVENTQ_PROD);
            if (prod & ((mask << 1) | 1)) == u.eventq_cons {
                break;
            }
            let e = u.eventq + ((u.eventq_cons & mask) as usize) * 32;
            let (w0, w2) = (core::ptr::read_volatile(e as *const u64), core::ptr::read_volatile((e + 16) as *const u64));
            if EVENTS_REPORTED.fetch_add(1, Ordering::Relaxed) < 32 {
                DbgPrint(
                    b"[HAL] SMMUv3 at %p: FAULT type %x StreamID %x address %p\n\0".as_ptr(),
                    u.base as ddk::PVOID,
                    (w0 & 0xFF) as u32,
                    (w0 >> 32) as u32,
                    w2 as ddk::PVOID,
                );
            }
            u.eventq_cons = (u.eventq_cons + 1) & ((mask << 1) | 1);
            wr32(u.va + EVENTQ_CONS, u.eventq_cons);
        }
        if gerror != 0 && EVENTS_REPORTED.fetch_add(1, Ordering::Relaxed) < 32 {
            DbgPrint(b"[HAL] SMMUv3 at %p: global error %x\n\0".as_ptr(), u.base as ddk::PVOID, gerror);
        }
    }
}

static mut TIMER: core::mem::MaybeUninit<ddk::KTIMER> = core::mem::MaybeUninit::uninit();
static mut DPC: core::mem::MaybeUninit<ddk::KDPC> = core::mem::MaybeUninit::uninit();

unsafe extern "system" fn event_dpc(_dpc: *mut ddk::KDPC, _context: ddk::PVOID, _a: ddk::PVOID, _b: ddk::PVOID) {
    let _guard = Locked::take();
    poll_events();
}

/// Look at the event queues once a second: faults are rare and loud, and polling
/// needs no interrupt wiring (the IORT names the SMMU's event SPI if that is ever
/// wanted).
unsafe fn arm_event_timer() {
    if TIMER_ARMED.swap(true, Ordering::AcqRel) {
        return;
    }
    let timer = (&raw mut TIMER).cast::<ddk::KTIMER>();
    let dpc = (&raw mut DPC).cast::<ddk::KDPC>();
    ddk::KeInitializeDpc(dpc, Some(event_dpc), core::ptr::null_mut());
    ddk::KeInitializeTimerEx(timer, ddk::_TIMER_TYPE_NotificationTimer);
    let mut due: ddk::LARGE_INTEGER = core::mem::zeroed();
    due.QuadPart = -10_000_000;
    ddk::KeSetTimerEx(timer, due, 1000, dpc);
}
