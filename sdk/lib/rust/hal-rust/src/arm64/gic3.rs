//! GICv3: the arm64 HAL's interrupt controller when the MADT describes redistributors.
//!
//! What changes from the GICv2 in clock.rs:
//!   * The CPU interface is system registers (ICC_*_EL1), not a GICC page: acknowledge
//!     is ICC_IAR1_EL1, the IRQL mask is ICC_PMR_EL1, an IPI is one ICC_SGI1R_EL1
//!     write naming its targets by MPIDR affinity.
//!   * SGIs and PPIs live in each CPU's own redistributor, not in banked distributor
//!     registers. The redistributors are ordinary MMIO, so any CPU can reach any of
//!     them; each frame names its CPU in GICR_TYPER, and a frame starts asleep until
//!     GICR_WAKER says otherwise.
//!   * SPIs are routed by affinity (GICD_IROUTER) instead of the 8-bit target mask.
//!   * Every interrupt we use is Group 1, which is what an OS running Non-secure gets,
//!     and is acknowledged through the Group 1 registers.
//!
//! The IRQL <-> priority map is the one clock.rs documents: priorities are a byte, lower
//! is more urgent, the PMR passes what is strictly below it. A Non-secure view of a GIC
//! with two security states shifts every priority it stores, but it shifts the PMR the
//! same way, so the ordering we rely on survives.
//!
//! Addresses: the loader identity-maps the distributor page and every redistributor
//! frame (winldr_arm64.rs), because an AP wakes its own redistributor in
//! HalInitializeProcessor, before MmMapIoSpace exists. HalInitializeClockArm64 maps
//! both for real and swaps the bases, which is why a CPU's frame is kept as an offset.

#![cfg(target_arch = "aarch64")]

use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use ntkrnl_sys::ddk;

unsafe extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

// Distributor.
const GICD_CTLR: usize = 0x0000;
const GICD_TYPER: usize = 0x0004;
const GICD_IGROUPR: usize = 0x0080;
const GICD_ISENABLER: usize = 0x0100;
const GICD_ICENABLER: usize = 0x0180;
const GICD_ISPENDR: usize = 0x0200;
const GICD_ISACTIVER: usize = 0x0300;
const GICD_IPRIORITYR: usize = 0x0400;
const GICD_ICFGR: usize = 0x0C00;
const GICD_IROUTER: usize = 0x6000;
/// The Non-secure view names these EnableGrp1, EnableGrp1A and ARE_NS; with a single
/// security state they are EnableGrp0, EnableGrp1 and ARE. Setting all three is right
/// in both views, which is what Linux does too.
const GICD_CTLR_ENABLE: u32 = (1 << 0) | (1 << 1);
const GICD_CTLR_ARE: u32 = 1 << 4;
const GICD_CTLR_RWP: u32 = 1 << 31;

// Redistributor, RD_base frame.
pub(crate) const GICR_CTLR: usize = 0x0000;
const GICR_TYPER: usize = 0x0008;
const GICR_WAKER: usize = 0x0014;
pub(crate) const GICR_PROPBASER: usize = 0x0070;
pub(crate) const GICR_PENDBASER: usize = 0x0078;
const GICR_CTLR_RWP: u32 = 1 << 3;
const GICR_TYPER_VLPIS: u64 = 1 << 1;
const GICR_TYPER_LAST: u64 = 1 << 4;
const GICR_WAKER_PROCESSOR_SLEEP: u32 = 1 << 1;
const GICR_WAKER_CHILDREN_ASLEEP: u32 = 1 << 2;
/// RD_base + SGI_base; a GICv4 with virtual LPIs adds two more 64 KiB pages.
const GICR_FRAME: usize = 0x2_0000;

// Redistributor, SGI_base frame: the SGI/PPI copies of the distributor registers.
const SGI_BASE: usize = 0x1_0000;
const GICR_IGROUPR0: usize = SGI_BASE + 0x0080;
const GICR_ISENABLER0: usize = SGI_BASE + 0x0100;
const GICR_ICENABLER0: usize = SGI_BASE + 0x0180;
const GICR_ISPENDR0: usize = SGI_BASE + 0x0200;
const GICR_ISACTIVER0: usize = SGI_BASE + 0x0300;
const GICR_IPRIORITYR: usize = SGI_BASE + 0x0400;

/// MPIDR_EL1's affinity fields (Aff3 at 39:32, Aff2..Aff0 at 23:0), which is also how
/// GICD_IROUTER and the top half of GICR_TYPER lay them out.
const MPIDR_AFFINITY: u64 = 0xFF_00FF_FFFF;

const MAX_CPUS: usize = 64;

/// Physical bases from the loader block, recorded at HalInitSystem. Present means the
/// firmware described a GICv3; everything in clock.rs asks [`present`].
static PRESENT: AtomicBool = AtomicBool::new(false);
static GICD_PA: AtomicU64 = AtomicU64::new(0);
static GICR_PA: AtomicU64 = AtomicU64::new(0);
static GICR_LEN: AtomicUsize = AtomicUsize::new(0);
/// Current addresses: the identity-mapped physical ones until clock init, then kernel
/// VAs. Zero until this CPU's path through the early bases or clock init ran.
static GICD: AtomicUsize = AtomicUsize::new(0);
static GICR: AtomicUsize = AtomicUsize::new(0);
/// Each CPU's redistributor as an offset into the region, plus one (zero: not found).
static FRAME: [AtomicUsize; MAX_CPUS] = [const { AtomicUsize::new(0) }; MAX_CPUS];
/// Each CPU's MPIDR affinity, which is how an SGI and an SPI route name it.
static AFFINITY: [AtomicU64; MAX_CPUS] = [const { AtomicU64::new(u64::MAX) }; MAX_CPUS];

/// Did the firmware describe a GICv3? Fixed from HalInitSystem on.
#[inline(always)]
pub(crate) fn present() -> bool {
    PRESENT.load(Ordering::Relaxed)
}

/// Has this CPU interface's owner published the bases yet (the GICv2 path's
/// "GICC_BASE != 0")? Before that the PMR is never touched.
#[inline(always)]
pub(crate) fn live() -> bool {
    GICR.load(Ordering::Relaxed) != 0
}

pub(crate) fn distributor() -> usize {
    GICD.load(Ordering::SeqCst)
}

/// Record the loader's GICv3 description. Runs on the BSP in HalInitSystem Phase 0.
pub(crate) unsafe fn record_platform(loader_block: ddk::PLOADER_PARAMETER_BLOCK) {
    if loader_block.is_null() {
        return;
    }
    let arm = &(*loader_block).u.Arm;
    if arm.Arm64GicDistributorBase == 0 || arm.Arm64GicRedistributorBase == 0 || arm.Arm64GicRedistributorLength == 0 {
        return;
    }
    GICD_PA.store(arm.Arm64GicDistributorBase, Ordering::SeqCst);
    GICR_PA.store(arm.Arm64GicRedistributorBase, Ordering::SeqCst);
    GICR_LEN.store(arm.Arm64GicRedistributorLength as usize, Ordering::SeqCst);
    // The boot CPU's affinity now, not at clock init: an AP's IPI to it in the early
    // SMP window then pends in its redistributor until it takes interrupts, as it
    // would pend in a GICv2's distributor, instead of being dropped for want of a
    // route.
    AFFINITY[0].store(own_affinity(), Ordering::SeqCst);
    PRESENT.store(true, Ordering::SeqCst);
}

/// Seed the bases with the identity-mapped physical addresses if clock init has not
/// published kernel VAs yet: an AP in the early-SMP window.
pub(crate) fn ensure_early_bases() {
    let _ = GICD.compare_exchange(0, GICD_PA.load(Ordering::SeqCst) as usize, Ordering::SeqCst, Ordering::SeqCst);
    let _ = GICR.compare_exchange(0, GICR_PA.load(Ordering::SeqCst) as usize, Ordering::SeqCst, Ordering::SeqCst);
}

// ---------------------------------------------------------------------------
// System registers. Named by encoding: the assembler's GIC register names are not
// something every LLVM we build with is guaranteed to accept.
// ---------------------------------------------------------------------------

macro_rules! icc_write {
    ($reg:literal, $v:expr) => {
        core::arch::asm!(concat!("msr ", $reg, ", {0}"), in(reg) $v as u64, options(nomem, nostack, preserves_flags))
    };
}
macro_rules! icc_read {
    ($reg:literal) => {{
        let v: u64;
        core::arch::asm!(concat!("mrs {0}, ", $reg), out(reg) v, options(nomem, nostack, preserves_flags));
        v
    }};
}

#[inline(always)]
unsafe fn isb() {
    core::arch::asm!("isb", options(nostack, preserves_flags));
}

/// ICC_PMR_EL1: the IRQL mask. Writes to it are self-synchronising for masking.
#[inline(always)]
pub(crate) unsafe fn set_pmr(priority: u8) {
    icc_write!("S3_0_C4_C6_0", priority);
}

/// ICC_IAR1_EL1: acknowledge the highest-priority pending Group 1 interrupt.
#[inline(always)]
pub(crate) unsafe fn acknowledge() -> u32 {
    (icc_read!("S3_0_C12_C12_0") & 0xFF_FFFF) as u32
}

/// ICC_EOIR1_EL1: priority drop and deactivate (EOImode 0).
#[inline(always)]
pub(crate) unsafe fn end_of_interrupt(intid: u32) {
    icc_write!("S3_0_C12_C12_1", intid);
    isb();
}

// ---------------------------------------------------------------------------
// MMIO.
// ---------------------------------------------------------------------------

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

/// Spin until a register's write-pending bit clears. Bounded: a GIC that never
/// finishes a write is a broken GIC, and hanging here would hide that.
unsafe fn wait_rwp(reg: usize, bit: u32) -> bool {
    for _ in 0..1_000_000 {
        if rd32(reg) & bit == 0 {
            return true;
        }
        core::hint::spin_loop();
    }
    false
}

/// This CPU's MPIDR affinity.
unsafe fn own_affinity() -> u64 {
    let mpidr: u64;
    core::arch::asm!("mrs {}, mpidr_el1", out(reg) mpidr, options(nomem, nostack, preserves_flags));
    mpidr & MPIDR_AFFINITY
}

/// The region offset of the redistributor whose GICR_TYPER names `affinity`.
unsafe fn find_frame(gicr: usize, affinity: u64) -> Option<usize> {
    let len = GICR_LEN.load(Ordering::SeqCst);
    let mut offset = 0;
    while offset + GICR_FRAME <= len {
        let typer = rd64(gicr + offset + GICR_TYPER);
        if typer >> 32 == ((affinity >> 8) & 0xFF00_0000) | (affinity & 0x00FF_FFFF) {
            return Some(offset);
        }
        if typer & GICR_TYPER_LAST != 0 {
            break;
        }
        offset += if typer & GICR_TYPER_VLPIS != 0 { 2 * GICR_FRAME } else { GICR_FRAME };
    }
    None
}

/// A CPU's redistributor (RD_base) at the current base, if it has been found.
pub(crate) fn frame(cpu: u32) -> Option<usize> {
    let gicr = GICR.load(Ordering::SeqCst);
    match FRAME.get(cpu as usize)?.load(Ordering::SeqCst) {
        0 => None,
        f if gicr != 0 => Some(gicr + f - 1),
        _ => None,
    }
}

/// The physical address of a CPU's redistributor, and its GICR_TYPER.Processor_Number:
/// an ITS names a collection's target by one or the other (GITS_TYPER.PTA).
pub(crate) fn frame_target(cpu: u32) -> Option<(u64, u32)> {
    let f = FRAME.get(cpu as usize)?.load(Ordering::SeqCst);
    let rd = frame(cpu)?;
    let number = unsafe { ((rd64(rd + GICR_TYPER) >> 8) & 0xFFFF) as u32 };
    (f != 0).then(|| (GICR_PA.load(Ordering::SeqCst) + (f - 1) as u64, number))
}

/// GICD_TYPER.IDbits: how many INTID bits this GIC implements (16 or more with LPIs).
pub(crate) unsafe fn id_bits() -> u32 {
    ((rd32(distributor() + GICD_TYPER) >> 19) & 0x1F) + 1
}

/// Turn on affinity routing and both groups' forwarding. Idempotent; the BSP does it
/// in clock init. ARE may only change while the groups are disabled.
unsafe fn init_distributor(gicd: usize) {
    let ctlr = rd32(gicd + GICD_CTLR);
    if ctlr & GICD_CTLR_ARE == 0 {
        wr32(gicd + GICD_CTLR, 0);
        wait_rwp(gicd + GICD_CTLR, GICD_CTLR_RWP);
        wr32(gicd + GICD_CTLR, GICD_CTLR_ARE);
        wait_rwp(gicd + GICD_CTLR, GICD_CTLR_RWP);
    }
    if ctlr & (GICD_CTLR_ARE | GICD_CTLR_ENABLE) != GICD_CTLR_ARE | GICD_CTLR_ENABLE {
        wr32(gicd + GICD_CTLR, GICD_CTLR_ARE | GICD_CTLR_ENABLE);
        wait_rwp(gicd + GICD_CTLR, GICD_CTLR_RWP);
    }
}

/// Swap to the kernel mappings and bring up the distributor. The BSP, clock init.
pub(crate) unsafe fn init(gicd_va: usize) -> bool {
    let mut pa: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    pa.QuadPart = GICR_PA.load(Ordering::SeqCst) as i64;
    let len = GICR_LEN.load(Ordering::SeqCst);
    let gicr = ddk::MmMapIoSpace(pa, len as ddk::SIZE_T, ddk::_MEMORY_CACHING_TYPE_MmNonCached) as usize;
    if gicr == 0 {
        DbgPrint(b"[GIC] GICv3: could not map %u bytes of redistributors - no interrupts\n\0".as_ptr(), len as u32);
        return false;
    }
    GICD.store(gicd_va, Ordering::SeqCst);
    GICR.store(gicr, Ordering::SeqCst);
    init_distributor(gicd_va);
    DbgPrint(
        b"[GIC] GICv3: distributor %p, %u bytes of redistributors at %p, %u INTID bits\n\0".as_ptr(),
        GICD_PA.load(Ordering::SeqCst) as ddk::PVOID,
        len as u32,
        GICR_PA.load(Ordering::SeqCst) as ddk::PVOID,
        id_bits(),
    );
    true
}

/// Make one SGI or PPI of THIS CPU's redistributor a Group 1 interrupt at `priority`,
/// and enable it.
pub(crate) unsafe fn enable_private(intid: u32, priority: u8) -> bool {
    let Some(rd) = frame(super::halstubs::halp_cpu_number()) else {
        return false;
    };
    let bit = 1u32 << (intid % 32);
    wr32(rd + GICR_IGROUPR0, rd32(rd + GICR_IGROUPR0) | bit);
    core::ptr::write_volatile((rd + GICR_IPRIORITYR + intid as usize) as *mut u8, priority);
    wr32(rd + GICR_ISENABLER0, bit);
    true
}

/// Program THIS CPU's half of the GIC: find and wake its redistributor, enable the NT
/// IPI there, and turn on the system-register CPU interface with the PMR at the
/// current IRQL. The BSP from clock init, each AP from HalInitializeProcessor.
pub(crate) unsafe fn percpu_init(ipi_intid: u32, ipi_priority: u8, pmr: u8) -> bool {
    let cpu = super::halstubs::halp_cpu_number();
    let gicr = GICR.load(Ordering::SeqCst);
    if gicr == 0 || cpu as usize >= MAX_CPUS {
        return false;
    }
    let affinity = own_affinity();
    AFFINITY[cpu as usize].store(affinity, Ordering::SeqCst);
    let Some(offset) = find_frame(gicr, affinity) else {
        DbgPrint(
            b"[GIC] CPU%u: no redistributor names affinity %llx - this CPU takes no interrupts\n\0".as_ptr(),
            cpu,
            affinity,
        );
        return false;
    };
    FRAME[cpu as usize].store(offset + 1, Ordering::SeqCst);
    let rd = gicr + offset;
    // The distributor's group enables gate SGIs and PPIs too, not just SPIs, and an AP
    // starts long before clock init: the first IPI of the early-SMP window needs them.
    let gicd = GICD.load(Ordering::SeqCst);
    if gicd != 0 {
        init_distributor(gicd);
    }
    // Wake it: a sleeping redistributor forwards nothing to its CPU.
    wr32(rd + GICR_WAKER, rd32(rd + GICR_WAKER) & !GICR_WAKER_PROCESSOR_SLEEP);
    let awake = wait_rwp(rd + GICR_WAKER, GICR_WAKER_CHILDREN_ASLEEP);

    // The system-register interface: SRE (with FIQ/IRQ bypass disabled), then the
    // mask, binary point 0 (every priority bit is group priority), EOImode 0 (EOIR
    // also deactivates), and Group 1 on.
    icc_write!("S3_0_C12_C12_5", icc_read!("S3_0_C12_C12_5") | 0b111);
    isb();
    let sre = icc_read!("S3_0_C12_C12_5") & 1 != 0;
    set_pmr(pmr);
    icc_write!("S3_0_C12_C12_3", 0u64);
    icc_write!("S3_0_C12_C12_4", icc_read!("S3_0_C12_C12_4") & !(1u64 << 1));
    icc_write!("S3_0_C12_C12_7", 1u64);
    isb();

    let enabled = enable_private(ipi_intid, ipi_priority)
        && rd32(rd + GICR_ISENABLER0) & (1 << ipi_intid) != 0
        && rd32(rd + GICR_IGROUPR0) & (1 << ipi_intid) != 0;
    DbgPrint(
        b"[GIC] CPU%u percpu (GICv3): affinity %llx, redistributor %u, awake=%u SRE=%u SGI%u enabled=%u%s\n\0".as_ptr(),
        cpu,
        affinity,
        (offset / GICR_FRAME) as u32,
        awake as u32,
        sre as u32,
        ipi_intid,
        enabled as u32,
        if enabled && awake && sre {
            b"\0".as_ptr()
        } else {
            b" - IPIs to this CPU are dead\0".as_ptr()
        },
    );
    enabled && awake && sre
}

/// Configure an SPI edge-triggered. Call with it disabled.
pub(crate) unsafe fn set_edge(intid: u32) {
    let gicd = distributor();
    let id = intid as usize;
    if gicd == 0 || !(32..1020).contains(&id) {
        return;
    }
    let reg = gicd + GICD_ICFGR + (id / 16) * 4;
    let bit = 1u32 << ((id % 16) * 2 + 1);
    if rd32(reg) & bit == 0 {
        wr32(reg, rd32(reg) | bit);
    }
}

/// Enable an SPI: Group 1, priority, trigger, routed to the boot CPU. A PPI goes to
/// this CPU's redistributor instead.
pub(crate) unsafe fn enable(intid: u32, priority: u8, level: bool) {
    let gicd = distributor();
    let id = intid as usize;
    if gicd == 0 || id >= 1020 {
        return;
    }
    if id < 32 {
        enable_private(intid, priority);
        return;
    }
    let bit = 1u32 << (id % 32);
    let group = gicd + GICD_IGROUPR + (id / 32) * 4;
    wr32(group, rd32(group) | bit);
    core::ptr::write_volatile((gicd + GICD_IPRIORITYR + id) as *mut u8, priority);
    if level {
        let reg = gicd + GICD_ICFGR + (id / 16) * 4;
        let edge = 1u32 << ((id % 16) * 2 + 1);
        if rd32(reg) & edge != 0 {
            wr32(reg, rd32(reg) & !edge);
        }
    }
    // Device interrupts all go to CPU0, as on the GICv2 (ITARGETSR = 1).
    let target = AFFINITY[0].load(Ordering::SeqCst);
    wr64(gicd + GICD_IROUTER + id * 8, if target == u64::MAX { 0 } else { target });
    wr32(gicd + GICD_ISENABLER + (id / 32) * 4, bit);
}

pub(crate) unsafe fn disable(intid: u32) {
    let gicd = distributor();
    let id = intid as usize;
    if gicd == 0 || id >= 1020 {
        return;
    }
    if id < 32 {
        if let Some(rd) = frame(super::halstubs::halp_cpu_number()) {
            wr32(rd + GICR_ICENABLER0, 1 << (id % 32));
            wait_rwp(rd + GICR_CTLR, GICR_CTLR_RWP);
        }
        return;
    }
    wr32(gicd + GICD_ICENABLER + (id / 32) * 4, 1 << (id % 32));
    wait_rwp(gicd + GICD_CTLR, GICD_CTLR_RWP);
}

/// Software-pend an SPI (the delivery self-test).
pub(crate) unsafe fn pend(intid: u32) {
    let gicd = distributor();
    let id = intid as usize;
    if gicd != 0 && (32..1020).contains(&id) {
        wr32(gicd + GICD_ISPENDR + (id / 32) * 4, 1 << (id % 32));
    }
}

/// ICC_SGI1R_EL1 for one target: its Aff3.Aff2.Aff1 cluster, and the bit for its Aff0
/// within the 16-wide range the RS field selects.
fn sgi1r(intid: u32, affinity: u64) -> u64 {
    let aff0 = affinity & 0xFF;
    let aff1 = (affinity >> 8) & 0xFF;
    let aff2 = (affinity >> 16) & 0xFF;
    let aff3 = (affinity >> 32) & 0xFF;
    (aff3 << 48) | (aff2 << 32) | ((aff0 >> 4) << 44) | (((intid & 0xF) as u64) << 24) | (aff1 << 16) | (1 << (aff0 & 0xF))
}

/// Send SGI `intid` to every CPU in `targets` (bit N == NT processor N). A CPU whose
/// affinity is not known yet has not run percpu_init, so it could not take it anyway.
pub(crate) unsafe fn send_sgi(intid: u32, targets: u64) {
    core::arch::asm!("dsb ish", options(nostack, preserves_flags));
    let mut rest = targets;
    while rest != 0 {
        let cpu = rest.trailing_zeros() as usize;
        rest &= rest - 1;
        let affinity = match AFFINITY.get(cpu) {
            Some(a) => a.load(Ordering::SeqCst),
            None => break,
        };
        if affinity == u64::MAX {
            DbgPrint(b"[GIC] IPI to CPU%u, which has no redistributor yet - DROPPED\n\0".as_ptr(), cpu as u32);
            continue;
        }
        icc_write!("S3_0_C12_C11_5", sgi1r(intid, affinity));
    }
    isb();
}

/// SGI `intid` to every CPU but this one (IRM = 1).
pub(crate) unsafe fn send_sgi_to_others(intid: u32) {
    core::arch::asm!("dsb ish", options(nostack, preserves_flags));
    icc_write!("S3_0_C12_C11_5", (1u64 << 40) | (((intid & 0xF) as u64) << 24));
    isb();
}

/// The GICv3 half of HalDumpGicArm64: mask registers and this CPU's redistributor.
pub(crate) unsafe fn dump() {
    let gicd = distributor();
    let cpu = super::halstubs::halp_cpu_number();
    DbgPrint(
        b"[GIC] GICv3 GICD_CTLR=%x PMR=%02x RPR=%02x IGRPEN1=%x soft-IRQL=%u cpu=%u affinity=%llx\n\0".as_ptr(),
        rd32(gicd + GICD_CTLR),
        icc_read!("S3_0_C4_C6_0") as u32,
        icc_read!("S3_0_C12_C11_3") as u32,
        icc_read!("S3_0_C12_C12_7") as u32,
        super::halstubs::KeGetCurrentIrql() as u32,
        cpu,
        own_affinity(),
    );
    if let Some(rd) = frame(cpu) {
        DbgPrint(
            b"[GIC] redistributor (INTID 0..31): group1=%08x enabled=%08x pending=%08x active=%08x WAKER=%x CTLR=%x\n\0".as_ptr(),
            rd32(rd + GICR_IGROUPR0),
            rd32(rd + GICR_ISENABLER0),
            rd32(rd + GICR_ISPENDR0),
            rd32(rd + GICR_ISACTIVER0),
            rd32(rd + GICR_WAKER),
            rd32(rd + GICR_CTLR),
        );
    }
    for bank in 1..3usize {
        DbgPrint(
            b"[GIC] bank%u (INTID %u..%u): group1=%08x enabled=%08x pending=%08x active=%08x\n\0".as_ptr(),
            bank as u32,
            (bank * 32) as u32,
            (bank * 32 + 31) as u32,
            rd32(gicd + GICD_IGROUPR + bank * 4),
            rd32(gicd + GICD_ISENABLER + bank * 4),
            rd32(gicd + GICD_ISPENDR + bank * 4),
            rd32(gicd + GICD_ISACTIVER + bank * 4),
        );
    }
}

/// The priority and route of an SPI, for the self-test line.
pub(crate) unsafe fn describe_spi(intid: u32) -> (u32, u32, u64) {
    let gicd = distributor();
    let id = intid as usize;
    (
        (rd32(gicd + GICD_ISENABLER + (id / 32) * 4) >> (id % 32)) & 1,
        core::ptr::read_volatile((gicd + GICD_IPRIORITYR + id) as *const u8) as u32,
        rd64(gicd + GICD_IROUTER + id * 8),
    )
}

/// Priority of one of this CPU's SGIs/PPIs, for the dump.
pub(crate) unsafe fn private_priority(intid: u32) -> u32 {
    match frame(super::halstubs::halp_cpu_number()) {
        Some(rd) => core::ptr::read_volatile((rd + GICR_IPRIORITYR + intid as usize) as *const u8) as u32,
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::sgi1r;

    #[test]
    fn sgi1r_names_the_target_by_affinity() {
        // CPU 0.0.0.1 on a flat QEMU virt: TargetList bit 1, everything else zero.
        assert_eq!(sgi1r(0, 0x1), 0b10);
        // Aff0 = 0x13 is bit 3 of range 1; Aff1/Aff2/Aff3 land in their fields.
        assert_eq!(sgi1r(2, 0x04_0005_0613), (4 << 48) | (5 << 32) | (1 << 44) | (2 << 24) | (6 << 16) | (1 << 3));
    }
}
