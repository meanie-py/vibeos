//! GICv2m: message-signaled interrupts on a GICv2 (crate::message's arm64 backend).
//!
//! A GICv2 has no native MSI support. A v2m "MSI frame" bolts it on: a device that
//! writes an SPI's INTID to the frame's MSI_SETSPI_NS register raises that SPI, edge
//! triggered. Each frame owns a contiguous run of SPIs, advertised either in the
//! ACPI MADT's GIC MSI Frame entry or in the frame's own MSI_TYPER register. QEMU's
//! virt machine has one frame when it runs a GICv2 (ours does). The Pi 4's GIC-400
//! has none, and a GICv3 (Graviton's) delivers MSIs through an ITS instead (its.rs).
//!
//! SPIs are handed out lowest-free and come back when the PnP manager releases the
//! device's resources (HalVibeosMessageRelease).

#![cfg(target_arch = "aarch64")]

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};
use ntkrnl_sys::ddk;

use super::acpi::{for_each_table, rd, XSDT_PA};
use super::bits::Bits;

unsafe extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

const MADT_RECORDS: usize = 44;
const MADT_TYPE_GIC_MSI_FRAME: u8 = 0x0D;
const MSI_FRAME_LENGTH: usize = 24;
// GIC MSI Frame entry fields (ACPI 6.x, MADT type 0x0D).
const MSI_FRAME_BASE: usize = 8;
const MSI_FRAME_FLAGS: usize = 16;
const MSI_FRAME_SPI_COUNT: usize = 20;
const MSI_FRAME_SPI_BASE: usize = 22;
/// Flags bit 0: SPI Count/Base in the entry override MSI_TYPER.
const MSI_FRAME_SPI_SELECT: u32 = 1;

// Frame registers.
const MSI_TYPER: usize = 0x008;
const MSI_SETSPI_NS: usize = 0x040;

const UNPROBED: u64 = 0;
const ABSENT: u64 = u64::MAX;
/// Physical base of the frame, or UNPROBED / ABSENT.
static FRAME_PA: AtomicU64 = AtomicU64::new(UNPROBED);
static SPI_BASE: AtomicU32 = AtomicU32::new(0);
static SPI_COUNT: AtomicU32 = AtomicU32::new(0);
/// Which of the frame's SPIs are handed out (at most 988: INTIDs 32-1019).
static SPIS: Bits<16> = Bits::new();
/// At an MSI block's first SPI, how many it holds (0 for a lone SPI, BLOCK_MEMBER at
/// the rest), indexed like SPIS.
static BLOCKS: [AtomicU8; 1024] = [const { AtomicU8::new(0) }; 1024];
const BLOCK_MEMBER: u8 = 0xFF;

/// Find the frame once. Runs at PnP time, so MmMapIoSpace is up.
unsafe fn probe() -> Option<u64> {
    match FRAME_PA.load(Ordering::Acquire) {
        ABSENT => return None,
        UNPROBED => {}
        pa => return Some(pa),
    }
    let mut frame: Option<(u64, u32, u16, u16)> = None;
    let xsdt = XSDT_PA.load(Ordering::Relaxed);
    if xsdt != 0 {
        for_each_table(xsdt, |sig, t, len| {
            if sig != b"APIC" || frame.is_some() {
                return;
            }
            let mut off = MADT_RECORDS;
            while off + 2 <= len {
                let ty = rd::<u8>(t, off);
                let rl = rd::<u8>(t, off + 1) as usize;
                if rl < 2 || off + rl > len {
                    break;
                }
                if ty == MADT_TYPE_GIC_MSI_FRAME && rl >= MSI_FRAME_LENGTH {
                    frame = Some((
                        rd::<u64>(t, off + MSI_FRAME_BASE),
                        rd::<u32>(t, off + MSI_FRAME_FLAGS),
                        rd::<u16>(t, off + MSI_FRAME_SPI_COUNT),
                        rd::<u16>(t, off + MSI_FRAME_SPI_BASE),
                    ));
                    break;
                }
                off += rl;
            }
        });
    }
    let Some((pa, flags, count, base)) = frame else {
        FRAME_PA.store(ABSENT, Ordering::Release);
        return None;
    };
    let (base, count) = if flags & MSI_FRAME_SPI_SELECT != 0 {
        (base as u32, count as u32)
    } else {
        let mut phys: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
        phys.QuadPart = pa as i64;
        let va = ddk::MmMapIoSpace(phys, 0x1000, ddk::_MEMORY_CACHING_TYPE_MmNonCached) as *const u8;
        if va.is_null() {
            FRAME_PA.store(ABSENT, Ordering::Release);
            return None;
        }
        let typer = core::ptr::read_volatile(va.add(MSI_TYPER) as *const u32);
        ddk::MmUnmapIoSpace(va as ddk::PVOID, 0x1000);
        ((typer >> 16) & 0x3FF, typer & 0x3FF)
    };
    if count == 0 || base < 32 || base + count > 1020 {
        FRAME_PA.store(ABSENT, Ordering::Release);
        return None;
    }
    SPI_BASE.store(base, Ordering::Relaxed);
    SPI_COUNT.store(count, Ordering::Relaxed);
    FRAME_PA.store(pa, Ordering::Release);
    DbgPrint(
        b"[HAL] GICv2m MSI frame at %p: INTIDs %u-%u\n\0".as_ptr(),
        pa as ddk::PVOID,
        base,
        base + count - 1,
    );
    Some(pa)
}

/// Is this INTID one of the frame's? Those are edge-triggered by construction.
pub(crate) fn owns(intid: u32) -> bool {
    let base = SPI_BASE.load(Ordering::Relaxed);
    FRAME_PA.load(Ordering::Acquire) != ABSENT && intid >= base && intid < base + SPI_COUNT.load(Ordering::Relaxed)
}

pub(crate) unsafe fn message_capacity() -> u32 {
    if probe().is_none() {
        return 0;
    }
    let count = SPI_COUNT.load(Ordering::Relaxed);
    count - SPIS.count(count)
}

/// `count` free SPIs (a power of two; 1 for a lone message), the first INTID a
/// multiple of `count`: the device ORs the message number into the data, which is the
/// INTID. Every SPI is routed to CPU0 on a GICv2 (ITARGETSR = 1), so a request that
/// leaves CPU0 out cannot be honoured; it is said so, not silently moved.
pub(crate) unsafe fn allocate_message_block(count: u32, requested: ddk::KAFFINITY) -> Option<(u32, ddk::KIRQL, ddk::KAFFINITY)> {
    probe()?;
    let base = SPI_BASE.load(Ordering::Relaxed);
    let n = SPIS.claim_block(count, SPI_COUNT.load(Ordering::Relaxed), base)?;
    for k in 0..count {
        BLOCKS[(n + k) as usize].store(if k > 0 { BLOCK_MEMBER } else if count > 1 { count as u8 } else { 0 }, Ordering::Relaxed);
    }
    let intid = base + n;
    if count > 1 {
        DbgPrint(b"[HAL] GICv2m: INTIDs %u-%u, an MSI block of %u\n\0".as_ptr(), intid, intid + count - 1, count);
    }
    if requested & 1 == 0 {
        DbgPrint(b"[HAL] GICv2m: INTID %u asked for CPUs %llx, but every SPI goes to CPU0\n\0".as_ptr(), intid, requested as u64);
    }
    // Every device SPI runs at the one DIRQL, as HalGetInterruptVector reports.
    Some((intid, super::clock::DEVICE_IRQL, 1))
}

/// Return an SPI to the frame, its line off; an MSI block's first SPI returns the
/// whole block. An SPI inside a block is not released alone.
pub(crate) unsafe fn release_message_vector(intid: u32) -> bool {
    if !owns(intid) {
        return false;
    }
    let n = intid - SPI_BASE.load(Ordering::Relaxed);
    let count = match BLOCKS[n as usize].load(Ordering::Relaxed) {
        BLOCK_MEMBER => return false,
        0 => 1,
        c => c as u32,
    };
    if !SPIS.is_set(n) {
        return false;
    }
    for k in 0..count {
        super::clock::disable_gic_interrupt(intid + k);
        BLOCKS[(n + k) as usize].store(0, Ordering::Relaxed);
        SPIS.release(n + k);
    }
    DbgPrint(b"[HAL] GICv2m: INTIDs %u-%u released\n\0".as_ptr(), intid, intid + count - 1);
    true
}

/// MSI_SETSPI_NS takes the INTID itself as the data, whoever writes it.
pub(crate) unsafe fn message_target(intid: u32, _requester: u32) -> Option<(u64, u32)> {
    let pa = probe()?;
    (owns(intid) && SPIS.is_set(intid - SPI_BASE.load(Ordering::Relaxed))).then_some((pa + MSI_SETSPI_NS as u64, intid))
}
