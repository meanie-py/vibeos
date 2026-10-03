//! ARM64 HAL: reading the firmware's ACPI tables without an AML interpreter.
//!
//! arm64 VibeOS has no ACPI bus driver (`acpi.sys` is not staged), so the HAL
//! reads what it needs straight from the tables the loader hands over
//! (`LoaderBlock->u.Arm.AcpiTablePhysicalAddress`, the XSDT): the FADT/MADT for
//! SMP (mpboot.rs) and, here, the handful of devices that only ACPI describes
//! and that pnpdd.rs publishes as PnP children.
//!
//! The device finder is deliberately narrow: it recognizes a node by its _HID
//! and reads the constant parts of its AML (the _CRS resource template, an
//! OperationRegion address). It never evaluates a method, so anything a method
//! computes is applied as a documented fixup for the one table we know.

#![cfg(target_arch = "aarch64")]

use core::sync::atomic::{AtomicU64, Ordering};
use ntkrnl_sys::ddk;

/// Physical address of the XSDT, recorded by HalInitSystem phase 0 while the
/// loader block is at hand; 0 = the loader found no ACPI.
pub(crate) static XSDT_PA: AtomicU64 = AtomicU64::new(0);

pub(crate) unsafe fn record_xsdt(loader_block: ddk::PLOADER_PARAMETER_BLOCK) {
    if !loader_block.is_null() {
        XSDT_PA.store((*loader_block).u.Arm.AcpiTablePhysicalAddress, Ordering::Relaxed);
    }
}

/// Map `len` bytes of firmware table memory (ordinary RAM: cached is correct,
/// and ACPI fields are unaligned, which Device memory would fault on).
pub(crate) unsafe fn map_table(pa: u64, len: usize) -> *const u8 {
    let mut phys: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    phys.QuadPart = pa as i64;
    ddk::MmMapIoSpace(phys, len as ddk::SIZE_T, ddk::_MEMORY_CACHING_TYPE_MmCached) as *const u8
}
pub(crate) unsafe fn unmap_table(va: *const u8, len: usize) {
    ddk::MmUnmapIoSpace(va as ddk::PVOID, len as ddk::SIZE_T);
}
#[inline]
pub(crate) unsafe fn rd<T: Copy>(base: *const u8, off: usize) -> T {
    core::ptr::read_unaligned(base.add(off) as *const T)
}

/// Call `f(signature, table)` for each table the XSDT lists.
pub(crate) unsafe fn for_each_table(xsdt_pa: u64, mut f: impl FnMut(&[u8; 4], *const u8, usize)) -> bool {
    let hdr = map_table(xsdt_pa, 36);
    if hdr.is_null() {
        return false;
    }
    let ok = rd::<[u8; 4]>(hdr, 0) == *b"XSDT";
    let xlen = rd::<u32>(hdr, 4) as usize;
    unmap_table(hdr, 36);
    if !ok || xlen < 36 {
        return false;
    }
    let xsdt = map_table(xsdt_pa, xlen);
    if xsdt.is_null() {
        return false;
    }
    for i in 0..(xlen - 36) / 8 {
        let pa = rd::<u64>(xsdt, 36 + i * 8);
        let th = map_table(pa, 36);
        if th.is_null() {
            continue;
        }
        let sig = rd::<[u8; 4]>(th, 0);
        let len = rd::<u32>(th, 4) as usize;
        unmap_table(th, 36);
        if len < 36 {
            continue;
        }
        let t = map_table(pa, len);
        if t.is_null() {
            continue;
        }
        f(&sig, t, len);
        unmap_table(t, len);
    }
    unmap_table(xsdt, xlen);
    true
}

// ---------------------------------------------------------------------------
// XHC0: a generic xHCI controller (_HID PNP0D10) described only by ACPI.
//
// On the Raspberry Pi 4 (pftf UEFI) the VL805 sits behind a PCIe root complex
// that is not ECAM-compliant and has no MCFG, so no PCI driver can find it.
// SSDT RPI4XHCI describes it instead:
//
//   Device (XHC0) { _HID "PNP0D10"  _CCA 0
//     Method (_CRS) { RBUF: QWordMemory(Min=Max=0x6_0000_0000, Len=1)
//                           Interrupt(Level, ActiveHigh) { 175 }
//                     Add(Max, 0xFFF) ; Add(Len, 0xFFF) ; Return RBUF }
//     Method (_INI) { OperationRegion(PCFG, SystemMemory, 0xFD508000, 0x1000)
//                     Field { VNID,16  DVID,16  CMND,16  STAT,16 }
//                     CMND = 6 } }
//
// PCFG is the root complex's config-space window onto the VL805 (bus 1,
// device 0): a standard PCI header, which pnpdd.rs exposes to usbport as the
// device's config space.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
pub(crate) struct AcpiXhci {
    pub mmio: u64,
    pub mmio_len: u64,
    /// GSIV; on arm64 that is the GIC INTID, which is also our vector.
    pub gsiv: u32,
    pub edge: bool,
    /// Physical address of the device's PCI config space (from _INI's
    /// OperationRegion), or 0 when the table has none.
    pub cfg: u64,
}

const HID_STRING: &[u8] = b"_HID\x0DPNP0D10\x00";
/// EISAID("PNP0D10") as the DWordConst form of _HID.
const HID_EISAID: &[u8] = b"_HID\x0C\x41\xD0\x0D\x10";
/// An xHCI register file is never smaller than this, and it is what the Pi's
/// _CRS method computes (Len 1 + 0xFFF) from its placeholder template.
const XHCI_MIN_WINDOW: u64 = 0x1000;

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.len() > hay.len() {
        return None;
    }
    (0..=hay.len() - needle.len()).find(|&i| &hay[i..i + needle.len()] == needle)
}

/// Decode an AML PkgLength at `p`: (value, bytes it occupied).
fn pkg_length(b: &[u8], p: usize) -> Option<(usize, usize)> {
    let lead = *b.get(p)? as usize;
    let extra = lead >> 6;
    if extra == 0 {
        return Some((lead & 0x3F, 1));
    }
    let mut v = lead & 0x0F;
    for i in 0..extra {
        v |= (*b.get(p + 1 + i)? as usize) << (4 + 8 * i);
    }
    Some((v, 1 + extra))
}

/// Decode an AML integer term at `p`: (value, bytes it occupied).
fn integer(b: &[u8], p: usize) -> Option<(u64, usize)> {
    let le = |n: usize| -> Option<u64> {
        let s = b.get(p + 1..p + 1 + n)?;
        Some(s.iter().rev().fold(0u64, |a, &x| (a << 8) | x as u64))
    };
    match *b.get(p)? {
        0x00 => Some((0, 1)),
        0x01 => Some((1, 1)),
        0xFF => Some((u64::MAX, 1)),
        0x0A => Some((le(1)?, 2)),
        0x0B => Some((le(2)?, 3)),
        0x0C => Some((le(4)?, 5)),
        0x0E => Some((le(8)?, 9)),
        _ => None,
    }
}

fn u64_at(b: &[u8], p: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(p..p + 8)?.try_into().ok()?))
}
fn u32_at(b: &[u8], p: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(p..p + 4)?.try_into().ok()?))
}

/// Walk a resource template (the bytes of a Buffer), taking the first memory
/// range and the first interrupt.
fn parse_resources(t: &[u8], x: &mut AcpiXhci) {
    let mut p = 0;
    while p < t.len() {
        let tag = t[p];
        if tag & 0x80 == 0 {
            if tag == 0x79 {
                break; // EndTag
            }
            p += 1 + (tag & 7) as usize;
            continue;
        }
        let Some(len) = t.get(p + 1..p + 3).map(|s| u16::from_le_bytes([s[0], s[1]]) as usize) else { break };
        let body = &t[p + 3..(p + 3 + len).min(t.len())];
        match tag {
            // QWord Address Space; ResourceType 0 = memory.
            0x8A if body.first() == Some(&0) && x.mmio == 0 => {
                x.mmio = u64_at(body, 11).unwrap_or(0);
                x.mmio_len = u64_at(body, 35).unwrap_or(0);
            }
            // DWord Address Space, memory.
            0x87 if body.first() == Some(&0) && x.mmio == 0 => {
                x.mmio = u32_at(body, 7).unwrap_or(0) as u64;
                x.mmio_len = u32_at(body, 19).unwrap_or(0) as u64;
            }
            // Memory32Fixed.
            0x86 if x.mmio == 0 => {
                x.mmio = u32_at(body, 1).unwrap_or(0) as u64;
                x.mmio_len = u32_at(body, 5).unwrap_or(0) as u64;
            }
            // Extended Interrupt: flags bit 1 = edge-triggered.
            0x89 if x.gsiv == 0 && body.len() >= 6 && body[1] >= 1 => {
                x.edge = body[0] & 2 != 0;
                x.gsiv = u32_at(body, 2).unwrap_or(0);
            }
            _ => {}
        }
        p += 3 + len;
    }
}

/// Find PNP0D10 in one definition block (DSDT/SSDT body).
fn find_xhci_in(aml: &[u8]) -> Option<AcpiXhci> {
    let hid = find(aml, HID_STRING).or_else(|| find(aml, HID_EISAID))?;
    // Bound the search to the enclosing Device(): its DeviceOp (5B 82) is
    // PkgLength + 4-byte name before the _HID's NameOp.
    let dev = (hid.saturating_sub(12)..hid).rev().find(|&i| aml[i] == 0x5B && aml.get(i + 1) == Some(&0x82))?;
    let (len, _) = pkg_length(aml, dev + 2)?;
    let body = &aml[hid..(dev + 2 + len).min(aml.len())];

    let mut x = AcpiXhci::default();
    // _CRS: the first Buffer after the name is the resource template, whether
    // _CRS is a Name or a Method building one.
    if let Some(crs) = find(body, b"_CRS") {
        if let Some(b) = body[crs..].iter().position(|&c| c == 0x11).map(|o| crs + o) {
            let (plen, pn) = pkg_length(body, b + 1)?;
            let (size, sn) = integer(body, b + 1 + pn)?;
            let start = b + 1 + pn + sn;
            let end = (b + 1 + plen).min(start + size as usize).min(body.len());
            parse_resources(&body[start..end], &mut x);
        }
    }
    // _INI: OperationRegion(Name, SystemMemory, Offset, Length).
    if let Some(ini) = find(body, b"_INI") {
        if let Some(op) = find(&body[ini..], b"\x5B\x80").map(|o| ini + o) {
            if body.get(op + 6) == Some(&0) {
                if let Some((base, _)) = integer(body, op + 7) {
                    x.cfg = base;
                }
            }
        }
    }
    if x.mmio == 0 || x.gsiv == 0 {
        return None;
    }
    if x.mmio_len < XHCI_MIN_WINDOW {
        x.mmio_len = XHCI_MIN_WINDOW;
    }
    Some(x)
}

/// Search the DSDT and every SSDT for an ACPI-only xHCI controller.
pub(crate) unsafe fn find_xhci() -> Option<AcpiXhci> {
    let xsdt = XSDT_PA.load(Ordering::Relaxed);
    if xsdt == 0 {
        return None;
    }
    let mut found: Option<AcpiXhci> = None;
    let mut dsdt_pa = 0u64;
    for_each_table(xsdt, |sig, t, len| {
        if sig == b"FACP" && len >= 148 {
            // X_DSDT (ACPI 2.0+) at 140, else the 32-bit DSDT at 40.
            dsdt_pa = rd::<u64>(t, 140);
            if dsdt_pa == 0 {
                dsdt_pa = rd::<u32>(t, 40) as u64;
            }
        } else if sig == b"SSDT" && found.is_none() {
            found = find_xhci_in(core::slice::from_raw_parts(t.add(36), len - 36));
        }
    });
    if found.is_none() && dsdt_pa != 0 {
        let h = map_table(dsdt_pa, 36);
        if !h.is_null() {
            let len = rd::<u32>(h, 4) as usize;
            unmap_table(h, 36);
            if len > 36 {
                let t = map_table(dsdt_pa, len);
                if !t.is_null() {
                    found = find_xhci_in(core::slice::from_raw_parts(t.add(36), len - 36));
                    unmap_table(t, len);
                }
            }
        }
    }
    found
}
