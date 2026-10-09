//! GICv3 ITS: message-signaled interrupts on a GICv3 (crate::message's arm64 backend
//! when the GIC is a v3; v2m.rs is the GICv2 one).
//!
//! A device raises an MSI by writing an EventID to an ITS's GITS_TRANSLATER. The bus
//! tags the write with the device's DeviceID, the ITS looks the pair up (DeviceID ->
//! that device's Interrupt Translation Table -> EventID -> an LPI and a collection),
//! and the collection names the redistributor the LPI is delivered to. An LPI is an
//! INTID from 8192 up, with no distributor state at all: its priority and enable bit
//! are one byte each in a table in memory (GICR_PROPBASER), and its pending bit is in
//! another per redistributor (GICR_PENDBASER), all owned by the OS.
//!
//! So unlike a v2m SPI, a message here is only meaningful for one device: the HAL has
//! to know who will write it. pci.sys passes the requester ID (bus/device/function)
//! when it asks for the address/data pair, and the IORT (iort.rs) says which ITS that
//! device's writes reach and which DeviceID it sees there.
//!
//! Every ITS the MADT lists is brought up, each with its own command queue and tables.
//! Every CPU whose redistributor takes LPIs gets a collection (ICID = its processor
//! number) on every ITS, and messages are spread across those CPUs as they are
//! handed out, the way Windows spreads a device's MSI-X messages: a driver whose
//! queues each have a message then completes on more than one CPU. A device whose
//! affinity policy names processors is spread over those instead.
//!
//! An LPI follows its interrupt object: KeConnectInterrupt enables it on the processor
//! the object was initialized for, and an LPI enabled on a CPU it does not target is
//! moved there (MOVI). When the PnP manager releases the device's resources each LPI
//! comes back: its event is discarded, and the device unmapped once it has none left.

#![cfg(target_arch = "aarch64")]

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use ntkrnl_sys::ddk;

use super::acpi::{for_each_table, rd, XSDT_PA};
use super::bits::{nth_cpu, Bits};
use super::gic3;
use super::iort;

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
const BASER_INDIRECT: u64 = 1 << 62;
const BASER_TYPE_DEVICES: u64 = 1;
const BASER_TYPE_COLLECTIONS: u64 = 4;
/// Every table and the command queue is ours, in non-cached memory; tell the ITS so
/// (InnerCache = Normal Non-cacheable, Shareability = Non-shareable, OuterCache = same
/// as inner), and nothing needs cache maintenance on either side.
const CACHE_NONCACHEABLE: u64 = 1 << 59;
/// A level-1 device table entry: the level-2 page it points at is valid.
const L1_VALID: u64 = 1 << 63;
/// The same attribute where GICR_PROPBASER and GICR_PENDBASER keep it.
const GICR_CACHE_NONCACHEABLE: u64 = 1 << 7;
const GICR_PENDBASER_PTZ: u64 = 1 << 62;
const GICR_CTLR_ENABLE_LPIS: u32 = 1 << 0;
const GICR_TYPER_PLPIS: u64 = 1 << 0;

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
const EVENT_WORDS: usize = EVENTS_PER_DEVICE as usize / 64;
const MAX_DEVICES: usize = 64;
const MAX_ITS: usize = 4;
const MAX_CPUS: u32 = 64;
const NO_REQUESTER: u32 = u32::MAX;

// ACPI.
const MADT_RECORDS: usize = 44;
const MADT_TYPE_GIC_ITS: u8 = 0x0F;
const MADT_ITS_ID: usize = 4;
const MADT_ITS_BASE: usize = 8;
const MADT_ITS_LENGTH: usize = 20;

const UNPROBED: u32 = 0;
const PROBING: u32 = 1;
const READY: u32 = 2;
const ABSENT: u32 = 3;
static STATE: AtomicU32 = AtomicU32::new(UNPROBED);

/// One ITS, as brought up.
#[derive(Clone, Copy)]
struct Its {
    /// The MADT's GIC ITS ID, which the IORT names it by.
    id: u32,
    pa: u64,
    va: usize,
    queue: usize,
    queue_next: usize,
    itt_entry: u32,
    event_bits: u32,
    /// GITS_TYPER.PTA: collections target a redistributor by address, not number.
    pta: bool,
    /// The device table: flat, or level 1 of two with `table_page`-sized level 2 pages.
    indirect: bool,
    table: usize,
    table_page: usize,
    table_entry: usize,
    /// DeviceIDs the device table covers.
    device_limit: u32,
}
const NO_ITS: Its = Its {
    id: 0,
    pa: 0,
    va: 0,
    queue: 0,
    queue_next: 0,
    itt_entry: 0,
    event_bits: 0,
    pta: false,
    indirect: false,
    table: 0,
    table_page: 0,
    table_entry: 0,
    device_limit: 0,
};
static mut ITSES: [Its; MAX_ITS] = [NO_ITS; MAX_ITS];
static ITS_COUNT: AtomicUsize = AtomicUsize::new(0);

static LPI_COUNT: AtomicU32 = AtomicU32::new(0);
/// Which LPIs are handed out.
static LPIS: Bits<{ LPI_SLOTS as usize / 64 }> = Bits::new();
/// Counts messages handed out; each takes the next CPU of its allowed set in turn.
static ROTOR: AtomicU32 = AtomicU32::new(0);
static PROPERTIES: AtomicUsize = AtomicUsize::new(0);
/// The CPUs whose redistributors take LPIs, each with a collection on every ITS.
static CPUS: AtomicU64 = AtomicU64::new(0);
/// Which CPU each LPI was handed out for, chosen at allocation.
static LPI_CPU: [AtomicU8; LPI_SLOTS as usize] = [const { AtomicU8::new(0) }; LPI_SLOTS as usize];
/// log2 of the MSI block each LPI belongs to (0: a lone message). A block's LPIs are
/// consecutive and aligned to its size, so its first is `n & !(size - 1)`.
static LPI_ORDER: [AtomicU8; LPI_SLOTS as usize] = [const { AtomicU8::new(0) }; LPI_SLOTS as usize];
/// One ITT page per device slot, allocated at bring-up: nothing under LOCK allocates.
static ITTS: AtomicU64 = AtomicU64::new(0);
static ITTS_VA: AtomicUsize = AtomicUsize::new(0);

/// The command queues, the device list and the LPI bindings, under one lock taken at
/// DISPATCH_LEVEL: every caller is PnP or KeConnectInterrupt, never an ISR.
static LOCK: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Copy)]
struct Device {
    its: u8,
    id: u32,
    /// Which EventIDs are bound.
    events: [u64; EVENT_WORDS],
}
const NO_DEVICE: Device = Device { its: 0, id: NO_REQUESTER, events: [0; EVENT_WORDS] };
static mut DEVICES: [Device; MAX_DEVICES] = [NO_DEVICE; MAX_DEVICES];

/// What an LPI is bound to: (ITS index, DeviceID, EventID); DeviceID NO_REQUESTER while
/// unbound.
#[derive(Clone, Copy)]
struct Binding {
    its: u8,
    device: u32,
    event: u32,
}
static mut BINDINGS: [Binding; LPI_SLOTS as usize] = [Binding { its: 0, device: NO_REQUESTER, event: 0 }; LPI_SLOTS as usize];

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

unsafe fn its(index: usize) -> &'static mut Its {
    &mut (*(&raw mut ITSES))[index]
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

/// Queue one command on an ITS and wait for it to be consumed.
unsafe fn command(i: &mut Its, c: [u64; 4]) -> bool {
    let at = i.queue_next;
    for (n, word) in c.iter().enumerate() {
        wr64(i.queue + at + n * 8, *word);
    }
    let next = (at + COMMAND_SIZE) % COMMAND_QUEUE;
    i.queue_next = next;
    dsb();
    wr64(i.va + GITS_CWRITER, next as u64);
    for _ in 0..1_000_000 {
        let creadr = rd64(i.va + GITS_CREADR);
        if creadr & GITS_CREADR_STALLED != 0 {
            DbgPrint(b"[HAL] GICv3 ITS %u: command %llx STALLED - the ITS rejected it\n\0".as_ptr(), i.id, c[0]);
            return false;
        }
        if creadr as usize & !(COMMAND_SIZE - 1) == next {
            return true;
        }
        core::hint::spin_loop();
    }
    DbgPrint(b"[HAL] GICv3 ITS %u: command %llx never consumed\n\0".as_ptr(), i.id, c[0]);
    false
}

/// How an ITS names a CPU's redistributor in MAPC and SYNC: its address >> 16 or its
/// processor number, per GITS_TYPER.PTA.
fn target(i: &Its, cpu: u32) -> u64 {
    match gic3::frame_target(cpu) {
        Some((pa, number)) => if i.pta { pa >> 16 } else { number as u64 },
        None => 0,
    }
}

unsafe fn sync(i: &mut Its, cpu: u32) -> bool {
    let t = target(i, cpu);
    command(i, [0x05, 0, t << 16, 0])
}

/// Point a CPU's redistributor at the LPI property table and a pending table of its
/// own, and turn LPIs on. Once on they cannot be turned off again, and while on the
/// tables cannot move, which is why a redistributor firmware left on is passed over
/// rather than reused.
unsafe fn enable_lpis(cpu: u32, properties_pa: u64) -> bool {
    let (Some(rd), Some(typer)) = (gic3::frame(cpu), gic3::redistributor_typer(cpu)) else {
        return false;
    };
    if typer & GICR_TYPER_PLPIS == 0 {
        return false;
    }
    if rd32(rd + gic3::GICR_CTLR) & GICR_CTLR_ENABLE_LPIS != 0 {
        DbgPrint(b"[HAL] GICv3 ITS: CPU%u's redistributor has LPIs on with firmware's tables - no MSIs there\n\0".as_ptr(), cpu);
        return false;
    }
    let Some((_, pending_pa)) = table((1usize << LPI_ID_BITS) / 8, 0x1_0000) else {
        return false;
    };
    wr64(rd + gic3::GICR_PROPBASER, properties_pa | GICR_CACHE_NONCACHEABLE | (LPI_ID_BITS as u64 - 1));
    wr64(rd + gic3::GICR_PENDBASER, pending_pa | GICR_CACHE_NONCACHEABLE | GICR_PENDBASER_PTZ);
    dsb();
    wr32(rd + gic3::GICR_CTLR, rd32(rd + gic3::GICR_CTLR) | GICR_CTLR_ENABLE_LPIS);
    dsb();
    true
}

/// Give the ITS memory for each table it asks for: devices and collections. vPE tables
/// are left unset, since we do not virtualize.
///
/// The device table is two-level when the ITS can do that and one page would not hold
/// every DeviceID: level 1 is an array of pointers to level-2 pages, filled in as
/// devices appear (prepare_device). Flat, all of a large DeviceID space costs
/// megabytes of contiguous memory up front; QEMU's 16 bits alone are 512 KiB.
unsafe fn provision_tables(i: &mut Its, device_bits: u32) -> bool {
    for n in 0..8 {
        let reg = i.va + GITS_BASER + n * 8;
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
        let size_code = match page {
            0x1000 => 0u64,
            0x4000 => 1,
            _ => 2,
        };
        let wanted = if kind == BASER_TYPE_DEVICES { 1usize << device_bits } else { MAX_CPUS as usize };
        let per_page = page / entry;
        let mut indirect = false;
        if kind == BASER_TYPE_DEVICES && wanted > per_page {
            wr64(reg, (baser & !(3 << 8)) | (size_code << 8) | BASER_INDIRECT);
            indirect = rd64(reg) & BASER_INDIRECT != 0;
        }
        let bytes = if indirect { wanted.div_ceil(per_page) * 8 } else { wanted * entry };
        let pages = bytes.div_ceil(page).clamp(1, 256);
        let Some((va, pa)) = table(pages * page, page) else {
            return false;
        };
        wr64(
            reg,
            BASER_VALID
                | if indirect { BASER_INDIRECT } else { 0 }
                | CACHE_NONCACHEABLE
                | (baser & (7 << 56))
                | (baser & (0x1F << 48))
                | pa
                | (size_code << 8)
                | (pages as u64 - 1),
        );
        if rd64(reg) & BASER_VALID == 0 {
            DbgPrint(b"[HAL] GICv3 ITS %u: GITS_BASER%u refused its table\n\0".as_ptr(), i.id, n as u32);
            return false;
        }
        if kind == BASER_TYPE_DEVICES {
            let covered = if indirect { pages * page / 8 * per_page } else { pages * page / entry };
            i.indirect = indirect;
            i.table = va;
            i.table_page = page;
            i.table_entry = entry;
            i.device_limit = covered.min(wanted).min(u32::MAX as usize) as u32;
        }
    }
    i.device_limit != 0
}

/// Every ITS the MADT lists: (GIC ITS ID, physical base).
unsafe fn madt_itses() -> ([(u32, u64); MAX_ITS], usize) {
    let mut found = [(0u32, 0u64); MAX_ITS];
    let mut count = 0;
    let xsdt = XSDT_PA.load(Ordering::Relaxed);
    if xsdt == 0 {
        return (found, 0);
    }
    for_each_table(xsdt, |sig, t, len| {
        if sig != b"APIC" || count != 0 {
            return;
        }
        let mut off = MADT_RECORDS;
        while off + 2 <= len {
            let ty = rd::<u8>(t, off);
            let rl = rd::<u8>(t, off + 1) as usize;
            if rl < 2 || off + rl > len {
                break;
            }
            if ty == MADT_TYPE_GIC_ITS && rl >= MADT_ITS_LENGTH && count < MAX_ITS {
                found[count] = (rd::<u32>(t, off + MADT_ITS_ID), rd::<u64>(t, off + MADT_ITS_BASE));
                count += 1;
            }
            off += rl;
        }
    });
    (found, count)
}

/// Bring one ITS up: quiesce it, give it a command queue and its tables, enable it,
/// and map a collection for every CPU that takes LPIs.
unsafe fn bring_up_its(id: u32, pa: u64, cpus: u64) -> Option<Its> {
    let mut phys: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    phys.QuadPart = pa as i64;
    let va = ddk::MmMapIoSpace(phys, 0x1_0000, ddk::_MEMORY_CACHING_TYPE_MmNonCached) as usize;
    if va == 0 {
        return None;
    }
    // GITS_TYPER: Physical (bit 0), ITT_entry_size (7:4), ID_bits (12:8), Devbits
    // (17:13), PTA (19).
    let typer = rd64(va + GITS_TYPER);
    if typer & 1 == 0 {
        DbgPrint(b"[HAL] GICv3 ITS %u at %p does not do physical LPIs\n\0".as_ptr(), id, pa as ddk::PVOID);
        return None;
    }
    let mut i = Its {
        id,
        pa,
        va,
        itt_entry: (((typer >> 4) & 0xF) + 1) as u32,
        event_bits: (((typer >> 8) & 0x1F) + 1) as u32,
        pta: typer & (1 << 19) != 0,
        ..NO_ITS
    };
    let device_bits = ((((typer >> 13) & 0x1F) + 1) as u32).min(24);

    // Firmware may have left it on; it has to be off (and quiet) to be reprogrammed.
    if rd32(va + GITS_CTLR) & GITS_CTLR_ENABLED != 0 {
        wr32(va + GITS_CTLR, rd32(va + GITS_CTLR) & !GITS_CTLR_ENABLED);
    }
    if !(0..1_000_000).any(|_| rd32(va + GITS_CTLR) & GITS_CTLR_QUIESCENT != 0) {
        DbgPrint(b"[HAL] GICv3 ITS %u at %p never went quiescent\n\0".as_ptr(), id, pa as ddk::PVOID);
        return None;
    }
    let (queue, queue_pa) = table(COMMAND_QUEUE, PAGE)?;
    i.queue = queue;
    wr64(va + GITS_CBASER, BASER_VALID | CACHE_NONCACHEABLE | queue_pa | (COMMAND_QUEUE / PAGE - 1) as u64);
    wr64(va + GITS_CWRITER, 0);
    if !provision_tables(&mut i, device_bits) {
        return None;
    }
    dsb();
    wr32(va + GITS_CTLR, rd32(va + GITS_CTLR) | GITS_CTLR_ENABLED);
    // MAPC: ICID = processor number -> that CPU's redistributor, Valid.
    let mut rest = cpus;
    while rest != 0 {
        let cpu = rest.trailing_zeros();
        rest &= rest - 1;
        let t = target(&i, cpu);
        if !command(&mut i, [0x09, 0, (1 << 63) | (t << 16) | cpu as u64, 0]) || !sync(&mut i, cpu) {
            return None;
        }
    }
    DbgPrint(
        b"[HAL] GICv3 ITS %u at %p: DeviceIDs below %u (%s table, %u-byte entries), %u-byte ITT entries\n\0".as_ptr(),
        id,
        pa as ddk::PVOID,
        i.device_limit,
        if i.indirect { b"two-level\0".as_ptr() } else { b"flat\0".as_ptr() },
        i.table_entry as u32,
        i.itt_entry,
    );
    Some(i)
}

unsafe fn bring_up() -> bool {
    if !gic3::present() || !gic3::live() {
        return false;
    }
    let (found, count) = madt_itses();
    if count == 0 {
        return false;
    }
    // LPI tables: the GIC must implement LPIs and enough INTID bits for ours.
    let id_bits = gic3::id_bits();
    if id_bits < LPI_ID_BITS {
        DbgPrint(b"[HAL] GICv3 implements %u INTID bits, too few for LPIs\n\0".as_ptr(), id_bits);
        return false;
    }
    let total = 1usize << LPI_ID_BITS;
    let Some((properties, properties_pa)) = table(total - LPI_BASE as usize, PAGE) else {
        return false;
    };
    let disabled = (super::clock::gic_priority(super::clock::DEVICE_IRQL) & 0xFC) | LPI_PROP_RES1;
    core::ptr::write_bytes(properties as *mut u8, disabled, total - LPI_BASE as usize);
    PROPERTIES.store(properties, Ordering::Relaxed);

    // Every CPU that is up and whose redistributor will take them. The boot CPU must
    // be one: an LPI routed nowhere is a lost interrupt.
    let mut cpus = 0u64;
    let mut rest = gic3::cpus();
    while rest != 0 {
        let cpu = rest.trailing_zeros();
        rest &= rest - 1;
        if enable_lpis(cpu, properties_pa) {
            cpus |= 1 << cpu;
        }
    }
    if cpus & 1 == 0 {
        DbgPrint(b"[HAL] GICv3 ITS: the boot CPU takes no LPIs - no MSIs\n\0".as_ptr());
        return false;
    }
    CPUS.store(cpus, Ordering::Relaxed);
    let Some((itts_va, itts)) = table(MAX_DEVICES * PAGE, PAGE) else {
        return false;
    };
    ITTS.store(itts, Ordering::Relaxed);
    ITTS_VA.store(itts_va, Ordering::Relaxed);

    let mut up = 0;
    for &(id, pa) in &found[..count] {
        if let Some(i) = bring_up_its(id, pa, cpus) {
            *its(up) = i;
            up += 1;
        }
    }
    if up == 0 {
        return false;
    }
    ITS_COUNT.store(up, Ordering::Release);
    LPI_COUNT.store(LPI_SLOTS.min((total - LPI_BASE as usize) as u32), Ordering::Relaxed);
    DbgPrint(
        b"[HAL] GICv3 ITS: %u of %u up, LPIs %u-%u spread over CPUs %llx\n\0".as_ptr(),
        up as u32,
        count as u32,
        LPI_BASE,
        LPI_BASE + LPI_COUNT.load(Ordering::Relaxed) - 1,
        cpus,
    );
    true
}

/// Bring the ITSes up once. Runs at PnP time, so MmMapIoSpace is up and every CPU's
/// redistributor has been found. Bring-up allocates and maps, so it cannot run under
/// LOCK; the first caller claims it and anyone else waits for the answer.
unsafe fn probe() -> bool {
    if STATE.compare_exchange(UNPROBED, PROBING, Ordering::AcqRel, Ordering::Acquire).is_ok() {
        let ok = bring_up();
        STATE.store(if ok { READY } else { ABSENT }, Ordering::Release);
        return ok;
    }
    loop {
        match STATE.load(Ordering::Acquire) {
            PROBING => core::hint::spin_loop(),
            s => return s == READY,
        }
    }
}

/// Where a requester's MSIs go: (ITS index, DeviceID). `requester` is pci.sys's: the
/// PCI segment in bits 31:16 above the requester ID. The IORT says, from the root
/// complex of that segment; with no IORT at all the requester ID is the DeviceID at
/// the first ITS, which is what QEMU does.
unsafe fn route(requester: u32) -> Option<(usize, u32)> {
    let (segment, rid) = (requester >> 16, requester & 0xFFFF);
    let xsdt = XSDT_PA.load(Ordering::Relaxed);
    let mut answer: Option<Result<iort::Route, iort::Error>> = None;
    if xsdt != 0 {
        for_each_table(xsdt, |sig, t, len| {
            if sig != b"IORT" || answer.is_some() {
                return;
            }
            let bytes = core::slice::from_raw_parts(t, len);
            answer = Some(iort::table(bytes).ok_or(iort::Error::Malformed).and_then(|t| iort::translate(t, segment, rid)));
        });
    }
    let (its_id, device) = match answer {
        None => (its(0).id, rid),
        Some(Ok(r)) => (r.its, r.device_id),
        Some(Err(e)) => {
            report_route_error(requester, e);
            return None;
        }
    };
    let index = (0..ITS_COUNT.load(Ordering::Acquire)).find(|&n| its(n).id == its_id);
    if index.is_none() {
        DbgPrint(b"[HAL] GICv3 ITS: requester %04x routes to ITS %u, which is not up\n\0".as_ptr(), requester, its_id);
    }
    Some((index?, device))
}

unsafe fn report_route_error(requester: u32, e: iort::Error) {
    match e {
        iort::Error::Malformed => DbgPrint(b"[HAL] IORT: malformed - no MSI route for requester %04x\n\0".as_ptr(), requester),
        iort::Error::NoRootComplex(s) => DbgPrint(b"[HAL] IORT: no root complex for segment %u\n\0".as_ptr(), s),
        iort::Error::NoMapping { node, id } => {
            DbgPrint(b"[HAL] IORT: node at %x maps no ID %x (requester %04x)\n\0".as_ptr(), node, id, requester)
        }
        iort::Error::Unsupported(kind) => DbgPrint(b"[HAL] IORT: requester %04x's path reaches a type %u node\n\0".as_ptr(), requester, kind as u32),
        iort::Error::TooDeep => DbgPrint(b"[HAL] IORT: requester %04x's path never reaches an ITS\n\0".as_ptr(), requester),
    };
}

/// Make sure a two-level device table has the level-2 page that holds `device`. Runs
/// before LOCK is taken, because it may allocate; a page two callers both allocate is
/// installed once and the other is simply never used.
unsafe fn prepare_device(index: usize, device: u32) -> bool {
    let i = its(index);
    if !i.indirect {
        return true;
    }
    let per_page = i.table_page / i.table_entry;
    let slot = (i.table + (device as usize / per_page) * 8) as *mut u64;
    if core::ptr::read_volatile(slot) & L1_VALID != 0 {
        return true;
    }
    let Some((_, pa)) = table(i.table_page, i.table_page) else {
        return false;
    };
    let _guard = Locked::take();
    if core::ptr::read_volatile(slot) & L1_VALID == 0 {
        core::ptr::write_volatile(slot, L1_VALID | pa);
        dsb();
    }
    true
}

/// MAPD a device with an ITT for events_per_device() events, or find it mapped.
unsafe fn map_device(index: usize, id: u32) -> Option<usize> {
    let devices = &raw mut DEVICES;
    if let Some(slot) = (0..MAX_DEVICES).find(|&n| (*devices)[n].its as usize == index && (*devices)[n].id == id) {
        return Some(slot);
    }
    // A released device leaves a hole before devices still mapped: look for the
    // device first, then for a hole.
    let slot = (0..MAX_DEVICES).find(|&n| (*devices)[n].id == NO_REQUESTER)?;
    let i = its(index);
    let events = events_per_device(i);
    let itt = ITTS.load(Ordering::Relaxed) + (slot * PAGE) as u64;
    // A slot a released device used: the ITS must not see its old translations.
    core::ptr::write_bytes((ITTS_VA.load(Ordering::Relaxed) + slot * PAGE) as *mut u8, 0, PAGE);
    dsb();
    // MAPD: DeviceID, Size (event ID bits - 1), ITT address, Valid.
    let size = events.trailing_zeros().max(1) - 1;
    if !command(i, [0x08 | ((id as u64) << 32), size as u64, (1 << 63) | itt, 0]) {
        return None;
    }
    (*devices)[slot] = Device { its: index as u8, id, ..NO_DEVICE };
    Some(slot)
}

/// Events one device gets: what its ITT page holds, within what the ITS can name.
fn events_per_device(i: &Its) -> u32 {
    let fit = PAGE as u32 / i.itt_entry.max(1);
    EVENTS_PER_DEVICE.min(1 << i.event_bits.min(31)).min(1 << (31 - fit.leading_zeros()))
}

/// Is this INTID one of the LPIs the ITS hands out?
pub(crate) fn owns(intid: u32) -> bool {
    intid >= LPI_BASE && intid < LPI_BASE + LPI_COUNT.load(Ordering::Relaxed)
}

pub(crate) unsafe fn message_capacity() -> u32 {
    if !probe() {
        return 0;
    }
    let count = LPI_COUNT.load(Ordering::Relaxed);
    count - LPIS.count(count)
}

/// `count` free LPIs (a power of two; 1 for a lone message), consecutive so a block's
/// message k is its first LPI + k, on the next CPU in turn of those `requested` that
/// take LPIs. A request naming none of them is said so and spread over all of them:
/// an LPI has to go somewhere that can take it. A block's LPIs share one CPU, as its
/// messages share one address everywhere else.
pub(crate) unsafe fn allocate_message_block(count: u32, requested: ddk::KAFFINITY) -> Option<(u32, ddk::KIRQL, ddk::KAFFINITY)> {
    if !probe() || count > EVENTS_PER_DEVICE {
        return None;
    }
    let n = LPIS.claim_block(count, LPI_COUNT.load(Ordering::Relaxed), LPI_BASE)?;
    let cpus = CPUS.load(Ordering::Relaxed);
    let mut allowed = cpus & requested as u64;
    if allowed == 0 {
        DbgPrint(
            b"[HAL] GICv3 ITS: LPI %u asked for CPUs %llx, none of which take LPIs (%llx) - spreading it\n\0".as_ptr(),
            LPI_BASE + n,
            requested as u64,
            cpus,
        );
        allowed = cpus;
    }
    let Some(cpu) = nth_cpu(allowed, ROTOR.fetch_add(1, Ordering::Relaxed)) else {
        (n..n + count).for_each(|m| _ = LPIS.release(m));
        return None;
    };
    for m in n..n + count {
        LPI_CPU[m as usize].store(cpu as u8, Ordering::Relaxed);
        LPI_ORDER[m as usize].store(count.trailing_zeros() as u8, Ordering::Relaxed);
    }
    if count > 1 {
        DbgPrint(b"[HAL] GICv3 ITS: LPIs %u-%u, an MSI block of %u, on CPU%u\n\0".as_ptr(), LPI_BASE + n, LPI_BASE + n + count - 1, count, cpu);
    }
    Some((LPI_BASE + n, super::clock::DEVICE_IRQL, 1 << cpu))
}

/// The block slot `n` belongs to: its first slot and its size.
fn block_of(n: u32) -> (u32, u32) {
    let size = 1u32 << LPI_ORDER[n as usize].load(Ordering::Relaxed);
    (n & !(size - 1), size)
}

/// Give an LPI back: off, its event discarded (which also drops anything pending),
/// and its device unmapped once that was the device's last event. An MSI block's
/// first LPI gives back the whole block; an LPI inside a block is not released alone.
pub(crate) unsafe fn release_message_vector(lpi: u32) -> bool {
    if !owns(lpi) || PROPERTIES.load(Ordering::Relaxed) == 0 {
        return false;
    }
    let n = lpi - LPI_BASE;
    if !LPIS.is_set(n) {
        DbgPrint(b"[HAL] GICv3 ITS: LPI %u released, but it was not handed out\n\0".as_ptr(), lpi);
        return false;
    }
    let (first, size) = block_of(n);
    if first != n {
        DbgPrint(b"[HAL] GICv3 ITS: LPI %u released alone, but it is inside an MSI block\n\0".as_ptr(), lpi);
        return false;
    }
    (n..n + size).fold(true, |ok, m| release_one(m) && ok)
}

unsafe fn release_one(n: u32) -> bool {
    let n = n as usize;
    let lpi = LPI_BASE + n as u32;
    let properties = PROPERTIES.load(Ordering::Relaxed);
    let disabled = (super::clock::gic_priority(super::clock::DEVICE_IRQL) & 0xFC) | LPI_PROP_RES1;
    {
        let _guard = Locked::take();
        core::ptr::write_volatile((properties + n) as *mut u8, disabled);
        dsb();
        let bindings = &raw mut BINDINGS;
        let b = (*bindings)[n];
        if b.device != NO_REQUESTER {
            let i = its(b.its as usize);
            // DISCARD: DeviceID, EventID.
            let ok = command(i, [0x0F | ((b.device as u64) << 32), b.event as u64, 0, 0])
                && sync(i, LPI_CPU[n].load(Ordering::Relaxed) as u32);
            let devices = &raw mut DEVICES;
            if let Some(slot) = (0..MAX_DEVICES).find(|&s| (*devices)[s].its == b.its && (*devices)[s].id == b.device) {
                let d = &mut (*devices)[slot];
                d.events[b.event as usize / 64] &= !(1u64 << (b.event % 64));
                // MAPD with Valid clear: the device has no events left.
                if d.events.iter().all(|&w| w == 0) && command(i, [0x08 | ((b.device as u64) << 32), 0, 0, 0]) {
                    *d = NO_DEVICE;
                    DbgPrint(b"[HAL] GICv3 ITS %u: DeviceID %x unmapped, its last event gone\n\0".as_ptr(), i.id, b.device);
                }
            }
            (*bindings)[n] = Binding { its: 0, device: NO_REQUESTER, event: 0 };
            DbgPrint(
                b"[HAL] GICv3 ITS %u: LPI %u released (DeviceID %x event %u)%s\n\0".as_ptr(),
                i.id,
                lpi,
                b.device,
                b.event,
                if ok { b"\0".as_ptr() } else { b" - DISCARD failed, the event may still fire\0".as_ptr() },
            );
        }
    }
    LPI_ORDER[n].store(0, Ordering::Relaxed);
    LPIS.release(n as u32)
}

/// Point a bound LPI at `cpu`'s collection (MOVI), when it targets another CPU and
/// `cpu` takes LPIs.
unsafe fn steer(lpi: u32, cpu: u32) {
    let n = (lpi - LPI_BASE) as usize;
    let from = LPI_CPU[n].load(Ordering::Relaxed) as u32;
    if from == cpu || cpu >= MAX_CPUS || CPUS.load(Ordering::Relaxed) & (1 << cpu) == 0 {
        return;
    }
    let _guard = Locked::take();
    let b = (*(&raw const BINDINGS))[n];
    if b.device != NO_REQUESTER {
        let i = its(b.its as usize);
        // MOVI: DeviceID, EventID -> ICID. The SYNC on the old target waits out
        // anything already on its way there.
        if !command(i, [0x01 | ((b.device as u64) << 32), b.event as u64, cpu as u64, 0]) || !sync(i, from) {
            return;
        }
        DbgPrint(b"[HAL] GICv3 ITS %u: LPI %u moved from CPU%u to CPU%u\n\0".as_ptr(), i.id, lpi, from, cpu);
    }
    LPI_CPU[n].store(cpu as u8, Ordering::Relaxed);
}

/// The address/data pair that raises `lpi`. The first ask, from pci.sys, names the
/// requester and binds the LPI to one of that device's events on the ITS its writes
/// reach; later asks (the I/O manager filling in a driver's message table) get the
/// same pair back.
pub(crate) unsafe fn message_target(lpi: u32, requester: u32) -> Option<(u64, u32)> {
    if !probe() || !owns(lpi) || !LPIS.is_set(lpi - LPI_BASE) {
        return None;
    }
    let n = (lpi - LPI_BASE) as usize;
    {
        let _guard = Locked::take();
        let b = (*(&raw const BINDINGS))[n];
        if b.device != NO_REQUESTER {
            return Some((its(b.its as usize).pa + GITS_TRANSLATER, b.event));
        }
    }
    if requester == NO_REQUESTER {
        return None;
    }
    let (index, device) = route(requester)?;
    if device >= its(index).device_limit {
        DbgPrint(b"[HAL] GICv3 ITS %u: DeviceID %x is past its device table\n\0".as_ptr(), its(index).id, device);
        return None;
    }
    if !prepare_device(index, device) {
        return None;
    }
    let _guard = Locked::take();
    let bindings = &raw mut BINDINGS;
    if (*bindings)[n].device != NO_REQUESTER {
        let b = (*bindings)[n];
        return Some((its(b.its as usize).pa + GITS_TRANSLATER, b.event));
    }
    let slot = map_device(index, device)?;
    let devices = &raw mut DEVICES;
    let i = its(index);
    let limit = events_per_device(i);
    // A block takes an aligned run of events, one per LPI in order: the device sends
    // the first event's ID with the message number ORed into its low bits.
    let (first, size) = block_of(n as u32);
    let words = (*devices)[slot].events;
    let free = |e: u32| words[e as usize / 64] & (1 << (e % 64)) == 0;
    let Some(base_event) = (0..limit).step_by(size as usize).find(|&e| e + size <= limit && (e..e + size).all(free)) else {
        DbgPrint(b"[HAL] GICv3 ITS %u: DeviceID %x has no %u free events left of %u\n\0".as_ptr(), i.id, device, size, limit);
        return None;
    };
    // MAPTI: DeviceID, EventID -> pINTID, ICID (the CPU it was handed out for).
    let cpu = LPI_CPU[n].load(Ordering::Relaxed) as u32;
    for k in 0..size {
        let (event, m) = (base_event + k, (first + k) as usize);
        let lpi = LPI_BASE + m as u32;
        if !command(i, [0x0A | ((device as u64) << 32), event as u64 | ((lpi as u64) << 32), cpu as u64, 0]) {
            return None;
        }
        (*devices)[slot].events[event as usize / 64] |= 1 << (event % 64);
        (*bindings)[m] = Binding { its: index as u8, device, event };
        DbgPrint(
            b"[HAL] GICv3 ITS %u: requester %04x (DeviceID %x) event %u -> LPI %u on CPU%u\n\0".as_ptr(),
            i.id,
            requester,
            device,
            event,
            lpi,
            cpu,
        );
    }
    if !sync(i, cpu) {
        return None;
    }
    let b = (*bindings)[n];
    Some((i.pa + GITS_TRANSLATER, b.event))
}

/// Write an LPI's property byte and make the redistributor drop its cached copy.
unsafe fn set_property(lpi: u32, value: u8) -> bool {
    let properties = PROPERTIES.load(Ordering::Relaxed);
    if properties == 0 || !owns(lpi) {
        return false;
    }
    let n = (lpi - LPI_BASE) as usize;
    let _guard = Locked::take();
    core::ptr::write_volatile((properties + n) as *mut u8, value);
    dsb();
    let b = (*(&raw const BINDINGS))[n];
    if b.device == NO_REQUESTER {
        // Never bound to a device, so nothing can raise it; INV needs a device.
        return true;
    }
    let i = its(b.its as usize);
    command(i, [0x0C | ((b.device as u64) << 32), b.event as u64, 0, 0]) && sync(i, LPI_CPU[n].load(Ordering::Relaxed) as u32)
}

/// HalEnableSystemInterrupt for an LPI, on the processor its interrupt object is
/// for: delivered there, at the priority of its IRQL.
pub(crate) unsafe fn enable(lpi: u32, irql: ddk::KIRQL) -> bool {
    if !LPIS.is_set(lpi - LPI_BASE) {
        return false;
    }
    steer(lpi, super::halstubs::halp_cpu_number());
    set_property(lpi, (super::clock::gic_priority(irql) & 0xFC) | LPI_PROP_RES1 | LPI_PROP_ENABLE)
}

pub(crate) unsafe fn disable(lpi: u32) {
    let properties = PROPERTIES.load(Ordering::Relaxed);
    if properties != 0 && owns(lpi) {
        let current = core::ptr::read_volatile((properties + (lpi - LPI_BASE) as usize) as *const u8);
        set_property(lpi, current & !LPI_PROP_ENABLE);
    }
}
