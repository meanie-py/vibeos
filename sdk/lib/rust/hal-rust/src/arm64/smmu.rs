//! What the IORT says about this machine's PCI, reported once at boot, and the SMMUs on
//! the way made to pass DMA through untranslated.
//!
//! No vibeos driver knows about an IOMMU: each hands its device physical addresses
//! (MmGetPhysicalAddress, the MDL's PFNs). Behind an SMMU that firmware left enabled,
//! or disabled but set to abort, every one of those DMAs faults, and the device looks
//! dead with nothing pointing at why. So each SMMU the IORT describes is checked
//! before the first driver starts, and put in bypass if it is not already: global
//! bypass (SMMUv3 GBPA, SMMUv1/v2 sCR0.CLIENTPD) is the "no IOMMU" machine our
//! drivers assume. Driving an SMMU for real (isolation, DMA remapping) is a separate
//! piece of work this does not start.
//!
//! The report also makes a new board's IORT legible from one serial log: what it
//! holds, and where each PCI segment's requester ID 0 lands.

#![cfg(target_arch = "aarch64")]

use core::sync::atomic::Ordering;
use ntkrnl_sys::ddk;

use super::acpi::{for_each_table, XSDT_PA};
use super::iort;

unsafe extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

// SMMUv3, page 0.
const SMMU_CR0: usize = 0x20;
const SMMU_CR0ACK: usize = 0x24;
const SMMU_GBPA: usize = 0x44;
const CR0_SMMUEN: u32 = 1 << 0;
const GBPA_UPDATE: u32 = 1 << 31;
const GBPA_ABORT: u32 = 1 << 20;

// SMMUv1/v2, global register space 0.
const SMMU_SCR0: usize = 0x0;
const SCR0_CLIENTPD: u32 = 1 << 0;

const MAX_SMMUS: usize = 8;

#[inline(always)]
unsafe fn rd32(a: usize) -> u32 {
    core::ptr::read_volatile(a as *const u32)
}
#[inline(always)]
unsafe fn wr32(a: usize, v: u32) {
    core::ptr::write_volatile(a as *mut u32, v)
}

/// Spin until `(reg & mask) == want`. Bounded: an SMMU that never acknowledges has
/// to be reported, not waited on forever.
unsafe fn settle(reg: usize, mask: u32, want: u32) -> bool {
    (0..1_000_000).any(|_| rd32(reg) & mask == want)
}

/// Put one SMMU in global bypass, saying what it was and what it is now.
unsafe fn bypass(smmu: iort::Smmu) {
    let mut phys: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    phys.QuadPart = smmu.base as i64;
    let va = ddk::MmMapIoSpace(phys, 0x1000, ddk::_MEMORY_CACHING_TYPE_MmNonCached) as usize;
    if va == 0 {
        DbgPrint(b"[HAL] IORT: could not map the SMMU at %p\n\0".as_ptr(), smmu.base as ddk::PVOID);
        return;
    }
    if smmu.version == 3 {
        let cr0 = rd32(va + SMMU_CR0);
        let gbpa = rd32(va + SMMU_GBPA);
        if cr0 & CR0_SMMUEN != 0 {
            // Firmware's stream tables would translate every DMA; we own none of them.
            wr32(va + SMMU_CR0, 0);
            if !settle(va + SMMU_CR0ACK, CR0_SMMUEN, 0) {
                DbgPrint(b"[HAL] IORT: SMMUv3 at %p would not disable - DMA behind it will fault\n\0".as_ptr(), smmu.base as ddk::PVOID);
                ddk::MmUnmapIoSpace(va as ddk::PVOID, 0x1000);
                return;
            }
        }
        if gbpa & GBPA_ABORT != 0 {
            wr32(va + SMMU_GBPA, GBPA_UPDATE | (gbpa & !GBPA_ABORT));
            settle(va + SMMU_GBPA, GBPA_UPDATE, 0);
        }
        let now = rd32(va + SMMU_GBPA);
        DbgPrint(
            b"[HAL] IORT: SMMUv3 at %p was %s%s; now %s\n\0".as_ptr(),
            smmu.base as ddk::PVOID,
            if cr0 & CR0_SMMUEN != 0 { b"ENABLED by firmware\0".as_ptr() } else { b"disabled\0".as_ptr() },
            if gbpa & GBPA_ABORT != 0 { b", aborting unmatched DMA\0".as_ptr() } else { b"\0".as_ptr() },
            if rd32(va + SMMU_CR0) & CR0_SMMUEN == 0 && now & GBPA_ABORT == 0 {
                b"bypassing: device DMA reaches physical memory untranslated\0".as_ptr()
            } else {
                b"STILL BLOCKING - DMA behind it will fault\0".as_ptr()
            },
        );
    } else {
        let scr0 = rd32(va + SMMU_SCR0);
        if scr0 & SCR0_CLIENTPD == 0 {
            wr32(va + SMMU_SCR0, scr0 | SCR0_CLIENTPD);
        }
        DbgPrint(
            b"[HAL] IORT: SMMUv1/v2 at %p was %s; now %s\n\0".as_ptr(),
            smmu.base as ddk::PVOID,
            if scr0 & SCR0_CLIENTPD == 0 { b"translating client DMA\0".as_ptr() } else { b"bypassing\0".as_ptr() },
            if rd32(va + SMMU_SCR0) & SCR0_CLIENTPD != 0 {
                b"bypassing\0".as_ptr()
            } else {
                b"STILL TRANSLATING - DMA behind it will fault\0".as_ptr()
            },
        );
    }
    ddk::MmUnmapIoSpace(va as ddk::PVOID, 0x1000);
}

/// Report the IORT and bypass its SMMUs. Once, from clock init: after MmMapIoSpace
/// works and before IoInitSystem starts the first driver.
pub(crate) unsafe fn platform_init() {
    let xsdt = XSDT_PA.load(Ordering::Relaxed);
    if xsdt == 0 {
        return;
    }
    let mut smmus = [None::<iort::Smmu>; MAX_SMMUS];
    let mut seen = false;
    for_each_table(xsdt, |sig, t, len| {
        if sig != b"IORT" || seen {
            return;
        }
        seen = true;
        let Some(t) = iort::table(core::slice::from_raw_parts(t, len)) else {
            DbgPrint(b"[HAL] IORT: present but malformed (bad length or checksum) - ignored\n\0".as_ptr());
            return;
        };
        let mut counts = [0u32; 6];
        for node in iort::nodes(t) {
            if let Some(c) = counts.get_mut(node.kind as usize) {
                *c += 1;
            }
            if let (Some(s), Some(free)) = (node.smmu(), smmus.iter_mut().find(|s| s.is_none())) {
                *free = Some(s);
            }
        }
        DbgPrint(
            b"[HAL] IORT rev %u: %u ITS group(s), %u root complex(es), %u named component(s), %u SMMUv3, %u SMMUv1/v2\n\0".as_ptr(),
            t[8] as u32,
            counts[iort::ITS_GROUP as usize],
            counts[iort::ROOT_COMPLEX as usize],
            counts[iort::NAMED_COMPONENT as usize],
            counts[iort::SMMU_V3 as usize],
            counts[iort::SMMU_V1_V2 as usize],
        );
        if counts[iort::ITS_GROUP as usize] == 0 {
            // A GICv2 machine: its MSIs, if any, go to a v2m frame, which the IORT
            // does not describe.
            return;
        }
        for node in iort::nodes(t) {
            let Some(segment) = node.segment() else {
                continue;
            };
            match iort::translate(t, segment, 0) {
                Ok(iort::Route { its, device_id, smmu: Some(s) }) => DbgPrint(
                    b"[HAL] IORT: segment %u requester 0000 -> ITS %u DeviceID %x through SMMUv%u at %p\n\0".as_ptr(),
                    segment,
                    its,
                    device_id,
                    s.version as u32,
                    s.base as ddk::PVOID,
                ),
                Ok(r) => DbgPrint(
                    b"[HAL] IORT: segment %u requester 0000 -> ITS %u DeviceID %x\n\0".as_ptr(),
                    segment,
                    r.its,
                    r.device_id,
                ),
                Err(_) => DbgPrint(
                    b"[HAL] IORT: segment %u requester 0000 has no MSI route\n\0".as_ptr(),
                    segment,
                ),
            };
        }
    });
    for smmu in smmus.iter().flatten() {
        bypass(*smmu);
    }
}
