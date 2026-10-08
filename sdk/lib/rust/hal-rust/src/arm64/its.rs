//! GICv3 ITS: message-signaled interrupts on a GICv3 (crate::message's arm64 backend
//! when the GIC is a v3; v2m.rs is the GICv2 one).
//!
//! A device raises an MSI by writing an EventID to the ITS's GITS_TRANSLATER. The bus
//! tags the write with the device's DeviceID, the ITS looks the pair up (DeviceID ->
//! that device's Interrupt Translation Table -> EventID -> an LPI and a collection),
//! and the collection names the redistributor the LPI is delivered to. An LPI is an
//! INTID from 8192 up, with no distributor state at all: its priority and enable bit
//! are one byte each in a table in memory (GICR_PROPBASER), and its pending bit is in
//! another (GICR_PENDBASER), both owned by the OS.
//!
//! So unlike a v2m SPI, a message here is only meaningful for one device: the HAL has
//! to know who will write it. pci.sys passes the requester ID (bus/device/function)
//! when it asks for the address/data pair, and the IORT says which DeviceID the ITS
//! sees for it. Every LPI targets one collection, mapped to the boot CPU, where every
//! other device interrupt goes too.
//!
//! What is owed: a two-level device table (the flat one covers DeviceIDs up to 256
//! pages' worth, which is all of QEMU's 16 bits), more than one ITS, routing anywhere
//! but CPU0, and returning LPIs and events when a device goes away.

#![cfg(target_arch = "aarch64")]

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use ntkrnl_sys::ddk;

use super::acpi::{for_each_table, rd, XSDT_PA};
use super::gic3;

unsafe extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

// ITS control frame.
const GITS_CTLR: usize = 0x0000;
const GITS_TYPER: usize = 0x0008;
const GITS_CBASER: usize = 0x0080;
const GITS_CWRITER: usize = 0x0088;
const GITS_CREADR: usize = 0x0090;
const GITS_BASER: usize = 0x0100;
const GITS_CTLR_ENABLED: u32 = 1 << 0;
const GITS_CTLR_QUIESCENT: u32 = 1 << 31;
const GITS_CREADR_STALLED: u64 = 1 << 0;
/// The translation frame follows the control frame; devices write here.
const GITS_TRANSLATER: u64 = 0x1_0040;

// GITS_BASER<n>.
const BASER_VALID: u64 = 1 << 63;
const BASER_TYPE_DEVICES: u64 = 1;
const BASER_TYPE_COLLECTIONS: u64 = 4;
/// Every table and the command queue is ours, in non-cached memory; tell the ITS so
/// (InnerCache = Normal Non-cacheable, Shareability = Non-shareable, OuterCache = same
/// as inner), and nothing needs cache maintenance on either side.
const CACHE_NONCACHEABLE: u64 = 1 << 59;
/// The same attribute where GICR_PROPBASER and GICR_PENDBASER keep it.
const GICR_CACHE_NONCACHEABLE: u64 = 1 << 7;
const GICR_PENDBASER_PTZ: u64 = 1 << 62;
const GICR_CTLR_ENABLE_LPIS: u32 = 1 << 0;

/// One LPI property byte: priority in bits 7:2, bit 1 RES1, bit 0 enable.
const LPI_PROP_RES1: u8 = 1 << 1;
const LPI_PROP_ENABLE: u8 = 1 << 0;

const LPI_BASE: u32 = 8192;
/// The kernel's interrupt table has this many LPI slots (ntoskrnl ke/arm64/stubs.rs).
const LPI_SLOTS: u32 = 1024;
/// 2^14 INTIDs: 8192 LPIs, of which we use the first LPI_SLOTS. The property table is
/// 8 KiB and each pending table 2 KiB at this size.
const LPI_ID_BITS: u32 = 14;

const PAGE: usize = 0x1000;
const COMMAND_QUEUE: usize = PAGE;
const COMMAND_SIZE: usize = 32;
/// Events (messages) one device can have. MSI-X allows 2048; nothing we drive asks for
/// more than a handful.
const EVENTS_PER_DEVICE: u32 = 256;
const MAX_DEVICES: usize = 32;
const NO_REQUESTER: u32 = u32::MAX;

// ACPI.
const MADT_RECORDS: usize = 44;
const MADT_TYPE_GIC_ITS: u8 = 0x0F;
const MADT_ITS_BASE: usize = 8;
const MADT_ITS_LENGTH: usize = 20;
const IORT_NODE_COUNT: usize = 36;
const IORT_NODE_OFFSET: usize = 40;
const IORT_ITS_GROUP: u8 = 0;
const IORT_SMMU: u8 = 3;
const IORT_ROOT_COMPLEX: u8 = 2;
const IORT_SMMU_V3: u8 = 4;
const IORT_RC_SEGMENT: usize = 28;
const IORT_ID_MAPPING_SINGLE: u32 = 1;

const UNPROBED: u64 = 0;
const PROBING: u64 = 1;
const ABSENT: u64 = u64::MAX;
/// Physical base of the ITS, or UNPROBED / PROBING / ABSENT.
static ITS_PA: AtomicU64 = AtomicU64::new(UNPROBED);
static ITS: AtomicUsize = AtomicUsize::new(0);
static ITT_ENTRY_SIZE: AtomicU32 = AtomicU32::new(0);
static DEVICE_BITS: AtomicU32 = AtomicU32::new(0);
/// DeviceIDs the flat device table covers.
static DEVICE_LIMIT: AtomicU32 = AtomicU32::new(0);
static EVENT_BITS: AtomicU32 = AtomicU32::new(0);
static LPI_COUNT: AtomicU32 = AtomicU32::new(0);
static LPI_NEXT: AtomicU32 = AtomicU32::new(0);
static PROPERTIES: AtomicUsize = AtomicUsize::new(0);
static QUEUE: AtomicUsize = AtomicUsize::new(0);
static QUEUE_NEXT: AtomicUsize = AtomicUsize::new(0);
/// GITS_TYPER.PTA ? the boot CPU's redistributor address >> 16 : its processor number.
static TARGET: AtomicU64 = AtomicU64::new(0);
/// One ITT page per device slot, allocated at bring-up: nothing under LOCK allocates.
static ITTS: AtomicU64 = AtomicU64::new(0);

/// The command queue, the device list and the LPI bindings, under one lock taken at
/// DISPATCH_LEVEL: every caller is PnP or KeConnectInterrupt, never an ISR.
static LOCK: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Copy)]
struct Device {
    id: u32,
    events: u32,
}
static mut DEVICES: [Device; MAX_DEVICES] = [Device { id: NO_REQUESTER, events: 0 }; MAX_DEVICES];
/// For each LPI: (DeviceID, EventID), or DeviceID NO_REQUESTER while unbound.
static mut BINDINGS: [(u32, u32); LPI_SLOTS as usize] = [(NO_REQUESTER, 0); LPI_SLOTS as usize];

struct Locked(ddk::KIRQL);
impl Locked {
    unsafe fn take() -> Self {
        let irql = super::halstubs::KfRaiseIrql(2);
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

#[inline(always)]
unsafe fn dsb() {
    core::arch::asm!("dsb sy", options(nostack, preserves_flags));
}
#[inline(always)]
unsafe fn rd32(a: usize) -> u32 {
    core::ptr::read_volatile(a as *const u32)
}
#[inline(always)]
unsafe fn wr32(a: usize, v: u32) {
    core::ptr::write_volatile(a as *mut u32, v)
}
#[inline(always)]
unsafe fn rd64(a: usize) -> u64 {
    core::ptr::read_volatile(a as *const u64)
}
#[inline(always)]
unsafe fn wr64(a: usize, v: u64) {
    core::ptr::write_volatile(a as *mut u64, v)
}

/// Zeroed, physically contiguous, non-cached memory aligned to `align`, which the
/// contiguous allocator does not promise beyond a page: ask for enough to align
/// within. Never freed, like everything the ITS is handed.
unsafe fn table(bytes: usize, align: usize) -> Option<(usize, u64)> {
    let mut highest: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    highest.QuadPart = (1i64 << 48) - 1;
    let low: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    let none: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    let span = bytes + align.saturating_sub(PAGE);
    let va = ddk::MmAllocateContiguousMemorySpecifyCache(
        span as ddk::SIZE_T,
        low,
        highest,
        none,
        ddk::_MEMORY_CACHING_TYPE_MmNonCached,
    ) as usize;
    if va == 0 {
        return None;
    }
    let pa = ddk::MmGetPhysicalAddress(va as ddk::PVOID).QuadPart as u64;
    let skip = ((pa + align as u64 - 1) & !(align as u64 - 1)) - pa;
    let va = va + skip as usize;
    core::ptr::write_bytes(va as *mut u8, 0, bytes);
    Some((va, pa + skip))
}

/// Queue one command and wait for the ITS to consume it.
unsafe fn command(c: [u64; 4]) -> bool {
    let its = ITS.load(Ordering::Relaxed);
    let queue = QUEUE.load(Ordering::Relaxed);
    let at = QUEUE_NEXT.load(Ordering::Relaxed);
    for (i, word) in c.iter().enumerate() {
        wr64(queue + at + i * 8, *word);
    }
    let next = (at + COMMAND_SIZE) % COMMAND_QUEUE;
    QUEUE_NEXT.store(next, Ordering::Relaxed);
    dsb();
    wr64(its + GITS_CWRITER, next as u64);
    for _ in 0..1_000_000 {
        let creadr = rd64(its + GITS_CREADR);
        if creadr & GITS_CREADR_STALLED != 0 {
            DbgPrint(b"[HAL] GICv3 ITS: command %llx STALLED - the ITS rejected it\n\0".as_ptr(), c[0]);
            return false;
        }
        if creadr as usize & !(COMMAND_SIZE - 1) == next {
            return true;
        }
        core::hint::spin_loop();
    }
    DbgPrint(b"[HAL] GICv3 ITS: command %llx never consumed\n\0".as_ptr(), c[0]);
    false
}

fn sync_command() -> [u64; 4] {
    [0x05, 0, TARGET.load(Ordering::Relaxed) << 16, 0]
}

/// Map ICID 0 to the boot CPU's redistributor.
unsafe fn map_collection() -> bool {
    let target = TARGET.load(Ordering::Relaxed);
    command([0x09, 0, (1 << 63) | (target << 16), 0]) && command(sync_command())
}

/// Point the boot CPU's redistributor at the LPI property and pending tables and turn
/// LPIs on. Once on they cannot be turned off again, and while on the tables cannot
/// move, which is why firmware that left them on is refused rather than reused.
unsafe fn enable_lpis(properties_pa: u64, pending_pa: u64) -> bool {
    let Some(rd) = gic3::frame(0) else {
        return false;
    };
    if rd32(rd + gic3::GICR_CTLR) & GICR_CTLR_ENABLE_LPIS != 0 {
        DbgPrint(b"[HAL] GICv3 ITS: firmware left LPIs on with its own tables - no MSIs\n\0".as_ptr());
        return false;
    }
    wr64(rd + gic3::GICR_PROPBASER, properties_pa | GICR_CACHE_NONCACHEABLE | (LPI_ID_BITS as u64 - 1));
    wr64(rd + gic3::GICR_PENDBASER, pending_pa | GICR_CACHE_NONCACHEABLE | GICR_PENDBASER_PTZ);
    dsb();
    wr32(rd + gic3::GICR_CTLR, rd32(rd + gic3::GICR_CTLR) | GICR_CTLR_ENABLE_LPIS);
    dsb();
    true
}

/// Give the ITS memory for each table it asks for: devices (one entry per DeviceID,
/// flat) and collections. vPE tables are left unset, since we do not virtualize.
unsafe fn provision_tables(its: usize) -> bool {
    for n in 0..8 {
        let reg = its + GITS_BASER + n * 8;
        let baser = rd64(reg);
        let kind = (baser >> 56) & 7;
        if kind != BASER_TYPE_DEVICES && kind != BASER_TYPE_COLLECTIONS {
            continue;
        }
        let entry = (((baser >> 48) & 0x1F) + 1) as usize;
        // Try 4 KiB pages; an ITS that only does larger ones reads back what it does.
        wr64(reg, baser & !(3 << 8));
        let page = match (rd64(reg) >> 8) & 3 {
            0 => 0x1000usize,
            1 => 0x4000,
            _ => 0x1_0000,
        };
        let entries = if kind == BASER_TYPE_DEVICES { 1usize << DEVICE_BITS.load(Ordering::Relaxed) } else { 1 };
        let pages = (entries * entry).div_ceil(page).clamp(1, 256);
        let Some((_, pa)) = table(pages * page, page) else {
            return false;
        };
        let size_code = match page {
            0x1000 => 0u64,
            0x4000 => 1,
            _ => 2,
        };
        wr64(reg, BASER_VALID | CACHE_NONCACHEABLE | (baser & (7 << 56)) | (baser & (0x1F << 48)) | pa | (size_code << 8) | (pages as u64 - 1));
        if rd64(reg) & BASER_VALID == 0 {
            DbgPrint(b"[HAL] GICv3 ITS: GITS_BASER%u refused its table\n\0".as_ptr(), n as u32);
            return false;
        }
        if kind == BASER_TYPE_DEVICES {
            DEVICE_LIMIT.store(((pages * page) / entry).min(u32::MAX as usize) as u32, Ordering::Relaxed);
        }
    }
    DEVICE_LIMIT.load(Ordering::Relaxed) != 0
}

/// Find the ITS once and bring it up. Runs at PnP time, so MmMapIoSpace is up and the
/// boot CPU's redistributor has been found.
unsafe fn probe() -> Option<u64> {
    match ITS_PA.load(Ordering::Acquire) {
        ABSENT => return None,
        UNPROBED => {}
        pa => return Some(pa),
    }
    // Bring-up allocates and maps, so it cannot run under LOCK; the first caller claims
    // it and anyone else waits for the answer.
    if ITS_PA.compare_exchange(UNPROBED, PROBING, Ordering::AcqRel, Ordering::Acquire).is_err() {
        loop {
            match ITS_PA.load(Ordering::Acquire) {
                PROBING => core::hint::spin_loop(),
                ABSENT => return None,
                pa => return Some(pa),
            }
        }
    }
    let found = bring_up();
    ITS_PA.store(found.unwrap_or(ABSENT), Ordering::Release);
    found
}

unsafe fn madt_its() -> Option<u64> {
    let mut base = None;
    let xsdt = XSDT_PA.load(Ordering::Relaxed);
    if xsdt == 0 {
        return None;
    }
    for_each_table(xsdt, |sig, t, len| {
        if sig != b"APIC" || base.is_some() {
            return;
        }
        let mut off = MADT_RECORDS;
        while off + 2 <= len {
            let ty = rd::<u8>(t, off);
            let rl = rd::<u8>(t, off + 1) as usize;
            if rl < 2 || off + rl > len {
                break;
            }
            if ty == MADT_TYPE_GIC_ITS && rl >= MADT_ITS_LENGTH {
                base = Some(rd::<u64>(t, off + MADT_ITS_BASE));
                break;
            }
            off += rl;
        }
    });
    base
}

unsafe fn bring_up() -> Option<u64> {
    if !gic3::present() || !gic3::live() {
        return None;
    }
    let pa = madt_its()?;
    let mut phys: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    phys.QuadPart = pa as i64;
    let its = ddk::MmMapIoSpace(phys, 0x1_0000, ddk::_MEMORY_CACHING_TYPE_MmNonCached) as usize;
    if its == 0 {
        return None;
    }
    let typer = rd64(its + GITS_TYPER);
    // GITS_TYPER: Physical (bit 0), ITT_entry_size (7:4), ID_bits (12:8), Devbits
    // (17:13), PTA (19).
    if typer & 1 == 0 {
        DbgPrint(b"[HAL] GICv3 ITS at %p does not do physical LPIs\n\0".as_ptr(), pa as ddk::PVOID);
        return None;
    }
    ITT_ENTRY_SIZE.store((((typer >> 4) & 0xF) + 1) as u32, Ordering::Relaxed);
    EVENT_BITS.store((((typer >> 8) & 0x1F) + 1) as u32, Ordering::Relaxed);
    DEVICE_BITS.store(((((typer >> 13) & 0x1F) + 1) as u32).min(20), Ordering::Relaxed);
    let Some((rd_pa, processor)) = gic3::frame_target(0) else {
        return None;
    };
    TARGET.store(if typer & (1 << 19) != 0 { rd_pa >> 16 } else { processor as u64 }, Ordering::Relaxed);

    // Firmware may have left it on; it has to be off (and quiet) to be reprogrammed.
    if rd32(its + GITS_CTLR) & GITS_CTLR_ENABLED != 0 {
        wr32(its + GITS_CTLR, rd32(its + GITS_CTLR) & !GITS_CTLR_ENABLED);
    }
    let mut quiet = false;
    for _ in 0..1_000_000 {
        if rd32(its + GITS_CTLR) & GITS_CTLR_QUIESCENT != 0 {
            quiet = true;
            break;
        }
        core::hint::spin_loop();
    }
    if !quiet {
        DbgPrint(b"[HAL] GICv3 ITS at %p never went quiescent\n\0".as_ptr(), pa as ddk::PVOID);
        return None;
    }

    // LPI tables: the GIC must implement LPIs and enough INTID bits for ours.
    let id_bits = gic3::id_bits();
    if id_bits < LPI_ID_BITS {
        DbgPrint(b"[HAL] GICv3 implements %u INTID bits, too few for LPIs\n\0".as_ptr(), id_bits);
        return None;
    }
    let total = 1usize << LPI_ID_BITS;
    let (properties, properties_pa) = table(total - LPI_BASE as usize, PAGE)?;
    let (_, pending_pa) = table(total / 8, 0x1_0000)?;
    let disabled = (super::clock::gic_priority(super::clock::DEVICE_IRQL) & 0xFC) | LPI_PROP_RES1;
    core::ptr::write_bytes(properties as *mut u8, disabled, total - LPI_BASE as usize);
    PROPERTIES.store(properties, Ordering::Relaxed);

    let (_, itts) = table(MAX_DEVICES * PAGE, PAGE)?;
    ITTS.store(itts, Ordering::Relaxed);
    let (queue, queue_pa) = table(COMMAND_QUEUE, PAGE)?;
    QUEUE.store(queue, Ordering::Relaxed);
    QUEUE_NEXT.store(0, Ordering::Relaxed);
    wr64(its + GITS_CBASER, BASER_VALID | CACHE_NONCACHEABLE | queue_pa | (COMMAND_QUEUE / PAGE - 1) as u64);
    wr64(its + GITS_CWRITER, 0);
    if !provision_tables(its) {
        return None;
    }
    ITS.store(its, Ordering::Relaxed);
    dsb();
    wr32(its + GITS_CTLR, rd32(its + GITS_CTLR) | GITS_CTLR_ENABLED);
    if !enable_lpis(properties_pa, pending_pa) || !map_collection() {
        return None;
    }
    LPI_COUNT.store(LPI_SLOTS.min((total - LPI_BASE as usize) as u32), Ordering::Relaxed);
    DbgPrint(
        b"[HAL] GICv3 ITS at %p: LPIs %u-%u, DeviceIDs below %u, %u-byte ITT entries\n\0".as_ptr(),
        pa as ddk::PVOID,
        LPI_BASE,
        LPI_BASE + LPI_COUNT.load(Ordering::Relaxed) - 1,
        DEVICE_LIMIT.load(Ordering::Relaxed),
        ITT_ENTRY_SIZE.load(Ordering::Relaxed),
    );
    Some(pa)
}

/// The DeviceID the ITS sees for a PCI requester ID on segment 0, from the IORT: the
/// root complex's ID mapping, followed through an SMMU if it sits in between. With no
/// IORT at all the requester ID is the DeviceID, which is what QEMU does anyway.
unsafe fn device_id(requester: u32) -> Option<u32> {
    let xsdt = XSDT_PA.load(Ordering::Relaxed);
    let mut result: Option<Option<u32>> = None;
    if xsdt != 0 {
        for_each_table(xsdt, |sig, t, len| {
            if sig != b"IORT" || result.is_some() {
                return;
            }
            result = Some(iort_translate(t, len, requester));
        });
    }
    result.unwrap_or(Some(requester))
}

/// Follow `id` from the root complex of segment 0 to an ITS group.
unsafe fn iort_translate(t: *const u8, len: usize, requester: u32) -> Option<u32> {
    let count = rd::<u32>(t, IORT_NODE_COUNT) as usize;
    let mut node = rd::<u32>(t, IORT_NODE_OFFSET) as usize;
    let mut rc = None;
    for _ in 0..count {
        if node + 16 > len {
            return None;
        }
        if rd::<u8>(t, node) == IORT_ROOT_COMPLEX && node + IORT_RC_SEGMENT + 4 <= len && rd::<u32>(t, node + IORT_RC_SEGMENT) == 0 {
            rc = Some(node);
            break;
        }
        node += rd::<u16>(t, node + 1) as usize;
    }
    let (mut node, mut id) = (rc?, requester);
    for _ in 0..4 {
        let mappings = rd::<u32>(t, node + 8) as usize;
        let first = node + rd::<u32>(t, node + 12) as usize;
        let mut next = None;
        for m in 0..mappings {
            let e = first + m * 20;
            if e + 20 > len {
                return None;
            }
            let (input, ids, output, reference, flags) =
                (rd::<u32>(t, e), rd::<u32>(t, e + 4), rd::<u32>(t, e + 8), rd::<u32>(t, e + 12), rd::<u32>(t, e + 16));
            // A single mapping is the node's own MSI, not a translation of its input.
            if flags & IORT_ID_MAPPING_SINGLE == 0 && id >= input && id - input <= ids {
                next = Some((reference as usize, output + (id - input)));
                break;
            }
        }
        let (reference, out) = next?;
        if reference + 16 > len {
            return None;
        }
        match rd::<u8>(t, reference) {
            IORT_ITS_GROUP => return Some(out),
            IORT_SMMU | IORT_SMMU_V3 => (node, id) = (reference, out),
            _ => return None,
        }
    }
    None
}

/// MAPD a new device with an ITT for EVENTS_PER_DEVICE events.
unsafe fn map_device(id: u32) -> Option<usize> {
    let devices = &raw mut DEVICES;
    let slot = (0..MAX_DEVICES).find(|&i| (*devices)[i].id == id || (*devices)[i].id == NO_REQUESTER)?;
    if (*devices)[slot].id == id {
        return Some(slot);
    }
    let events = events_per_device();
    let itt = ITTS.load(Ordering::Relaxed) + (slot * PAGE) as u64;
    // MAPD: DeviceID, Size (event ID bits - 1), ITT address, Valid.
    let size = events.trailing_zeros().max(1) - 1;
    if !command([0x08 | ((id as u64) << 32), size as u64, (1 << 63) | itt, 0]) {
        return None;
    }
    (*devices)[slot] = Device { id, events: 0 };
    Some(slot)
}

/// Events one device gets: what its ITT page holds, within what the ITS can name.
fn events_per_device() -> u32 {
    let fit = PAGE as u32 / ITT_ENTRY_SIZE.load(Ordering::Relaxed).max(1);
    EVENTS_PER_DEVICE.min(1 << EVENT_BITS.load(Ordering::Relaxed)).min(1 << (31 - fit.leading_zeros()))
}

/// Did this INTID come from the ITS? Those are LPIs we handed out.
pub(crate) fn owns(intid: u32) -> bool {
    intid >= LPI_BASE && intid < LPI_BASE + LPI_NEXT.load(Ordering::Relaxed)
}

pub(crate) unsafe fn message_capacity() -> u32 {
    if probe().is_none() {
        return 0;
    }
    LPI_COUNT.load(Ordering::Relaxed).saturating_sub(LPI_NEXT.load(Ordering::Relaxed))
}

pub(crate) unsafe fn allocate_message_vector() -> Option<(u32, ddk::KIRQL, ddk::KAFFINITY)> {
    probe()?;
    let count = LPI_COUNT.load(Ordering::Relaxed);
    let n = LPI_NEXT
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| (n < count).then_some(n + 1))
        .ok()?;
    Some((LPI_BASE + n, super::clock::DEVICE_IRQL, 1))
}

/// The address/data pair that raises `lpi`. The first ask, from pci.sys, names the
/// requester and binds the LPI to one of that device's events; later asks (the I/O
/// manager filling in a driver's message table) get the same pair back.
pub(crate) unsafe fn message_target(lpi: u32, requester: u32) -> Option<(u64, u32)> {
    let pa = probe()?;
    if !owns(lpi) {
        return None;
    }
    let translater = pa + GITS_TRANSLATER;
    let _guard = Locked::take();
    let bindings = &raw mut BINDINGS;
    let binding = &mut (*bindings)[(lpi - LPI_BASE) as usize];
    if binding.0 != NO_REQUESTER {
        return Some((translater, binding.1));
    }
    if requester == NO_REQUESTER {
        return None;
    }
    let Some(id) = device_id(requester) else {
        DbgPrint(b"[HAL] GICv3 ITS: the IORT maps no DeviceID for requester %04x\n\0".as_ptr(), requester);
        return None;
    };
    if id >= DEVICE_LIMIT.load(Ordering::Relaxed) {
        DbgPrint(b"[HAL] GICv3 ITS: DeviceID %x is past the device table\n\0".as_ptr(), id);
        return None;
    }
    let slot = map_device(id)?;
    let devices = &raw mut DEVICES;
    let event = (*devices)[slot].events;
    if event >= events_per_device() {
        return None;
    }
    // MAPTI: DeviceID, EventID -> pINTID, ICID 0.
    if !command([0x0A | ((id as u64) << 32), event as u64 | ((lpi as u64) << 32), 0, 0]) || !command(sync_command()) {
        return None;
    }
    (*devices)[slot].events = event + 1;
    *binding = (id, event);
    Some((translater, event))
}

/// Write an LPI's property byte and make the redistributor drop its cached copy.
unsafe fn set_property(lpi: u32, value: u8) -> bool {
    let properties = PROPERTIES.load(Ordering::Relaxed);
    if properties == 0 || !owns(lpi) {
        return false;
    }
    let _guard = Locked::take();
    core::ptr::write_volatile((properties + (lpi - LPI_BASE) as usize) as *mut u8, value);
    dsb();
    let (id, event) = (*(&raw const BINDINGS))[(lpi - LPI_BASE) as usize];
    if id == NO_REQUESTER {
        // Never bound to a device, so nothing can raise it; INV needs a device.
        return true;
    }
    command([0x0C | ((id as u64) << 32), event as u64, 0, 0]) && command(sync_command())
}

/// HalEnableSystemInterrupt for an LPI: its priority from the IRQL, and enable.
pub(crate) unsafe fn enable(lpi: u32, irql: ddk::KIRQL) -> bool {
    set_property(lpi, (super::clock::gic_priority(irql) & 0xFC) | LPI_PROP_RES1 | LPI_PROP_ENABLE)
}

pub(crate) unsafe fn disable(lpi: u32) {
    let properties = PROPERTIES.load(Ordering::Relaxed);
    if properties != 0 && owns(lpi) {
        let current = core::ptr::read_volatile((properties + (lpi - LPI_BASE) as usize) as *const u8);
        set_property(lpi, current & !LPI_PROP_ENABLE);
    }
}
