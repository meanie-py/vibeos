//! GICv2m: message-signaled interrupts on a GICv2 (crate::message's arm64 backend).
//!
//! A GICv2 has no native MSI support. A v2m "MSI frame" bolts it on: a device that
//! writes an SPI's INTID to the frame's MSI_SETSPI_NS register raises that SPI, edge
//! triggered. Each frame owns a contiguous run of SPIs, advertised either in the
//! ACPI MADT's GIC MSI Frame entry or in the frame's own MSI_TYPER register. QEMU's
//! virt machine has one frame when it runs a GICv2 (ours does). The Pi 4's GIC-400
//! has none, and a GICv3 (Graviton's) delivers MSIs through an ITS instead (its.rs).
//!
//! Vectors are handed out first-come and never returned: the PnP manager does not
//! release a device's message resources yet, so neither does this.

#![cfg(target_arch = "aarch64")]

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use ntkrnl_sys::ddk;

use super::acpi::{for_each_table, rd, XSDT_PA};

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
static SPI_NEXT: AtomicU32 = AtomicU32::new(0);

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

/// Did this INTID come from the frame? Those are edge-triggered by construction.
pub(crate) fn owns(intid: u32) -> bool {
    FRAME_PA.load(Ordering::Acquire) != ABSENT
        && intid >= SPI_BASE.load(Ordering::Relaxed)
        && intid < SPI_BASE.load(Ordering::Relaxed) + SPI_NEXT.load(Ordering::Relaxed)
}

pub(crate) unsafe fn message_capacity() -> u32 {
    if probe().is_none() {
        return 0;
    }
    SPI_COUNT.load(Ordering::Relaxed).saturating_sub(SPI_NEXT.load(Ordering::Relaxed))
}

pub(crate) unsafe fn allocate_message_vector() -> Option<(u32, ddk::KIRQL, ddk::KAFFINITY)> {
    probe()?;
    let count = SPI_COUNT.load(Ordering::Relaxed);
    let n = SPI_NEXT
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| (n < count).then_some(n + 1))
        .ok()?;
    // Every device SPI runs at the one DIRQL, as HalGetInterruptVector reports.
    Some((SPI_BASE.load(Ordering::Relaxed) + n, super::clock::DEVICE_IRQL, 1))
}

/// MSI_SETSPI_NS takes the INTID itself as the data, whoever writes it.
pub(crate) unsafe fn message_target(intid: u32, _requester: u32) -> Option<(u64, u32)> {
    let pa = probe()?;
    owns(intid).then_some((pa + MSI_SETSPI_NS as u64, intid))
}
