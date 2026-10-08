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

/// MCFG OEM IDs whose table must not be read as ECAM: "RPIFDN" is the Raspberry Pi
/// UEFI firmware, whose MCFG points inside the Broadcom PCIe controller's register
/// block (not ECAM-compliant; reads return 0xDEADDEAD and raise SError). The loader
/// keeps the same list (freeldr-rust `arm64_serial_acpi::MCFG_QUIRKS`) and reports
/// such a machine as having no PCI; keep the two in step.
const MCFG_QUIRKS: &[[u8; 6]] = &[*b"RPIFDN"];

/// The physical base of segment 0's ECAM window from the ACPI MCFG, i.e. the
/// address of bus 0, device 0, function 0. None when there is no MCFG, the MCFG
/// is quirked, or segment 0's allocation does not cover bus 0.
pub(crate) unsafe fn mcfg_bus0_base() -> Option<u64> {
    let xsdt = XSDT_PA.load(Ordering::Relaxed);
    if xsdt == 0 {
        return None;
    }
    let mut found = None;
    for_each_table(xsdt, |sig, t, len| {
        if found.is_some() || *sig != *b"MCFG" || len < 44 {
            return;
        }
        let oem = rd::<[u8; 6]>(t, 10);
        if MCFG_QUIRKS.contains(&oem) {
            return;
        }
        // 36-byte header, 8 reserved, then 16-byte allocations:
        // base u64, segment u16, start bus u8, end bus u8, reserved u32.
        let mut off = 44;
        while off + 16 <= len {
            let base = rd::<u64>(t, off);
            let segment = rd::<u16>(t, off + 8);
            let (start, end) = (rd::<u8>(t, off + 10), rd::<u8>(t, off + 11));
            if segment == 0 && base != 0 && start == 0 && start <= end {
                found = Some(base);
                return;
            }
            off += 16;
        }
    });
    found
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

// ETH0: the Raspberry Pi 4's GENET Ethernet MAC (BCM2711), also ACPI-only:
//
//   Device (ETH0) { _HID "BCM6E4E"  _CCA 0
//     Method (_CRS) { RBUF: Memory32Fixed(ReadWrite, 0xFD580000, 0x10000)
//                           Interrupt(Level, ActiveHigh) { 189, 190 }
//                     Return RBUF }
//     _DSD { "brcm,max-dma-burst-size" 8, "phy-mode" "rgmii-rxid" } }
//
// No MAC address in the table: the firmware's GENET driver leaves it in the
// controller's UMAC_MAC0/1 registers, where the OS driver reads it.
// ---------------------------------------------------------------------------

// SDC3: the Pi 4's EMMC2 SD host controller (the card slot), in SSDT RPI4EMMC
// under an ACPI0004 container whose _DMA describes the 3 GB DMA window:
//
//   Device (SDC3) { _HID "BRCME88C"  _CCA 0
//     Name (RBUF, ResourceTemplate () { Memory32Fixed(ReadWrite, 0, 0x100)
//                                       Interrupt(Level, ActiveHigh) { 158 } })
//     Method (_CRS) { CreateDWordField(^RBUF, 4, RBAS)
//                     Store(0xFE340000, RBAS) ; Return ^RBUF }
//     _DSD { "sdhci-caps-mask" ... } }
//
// The template's base is a placeholder that _CRS patches, so the base comes
// from the method (`patched_field`), not from the template.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
pub(crate) struct AcpiDevice {
    pub mmio: u64,
    pub mmio_len: u64,
    /// GSIV; on arm64 that is the GIC INTID, which is also our vector.
    pub gsiv: u32,
    pub edge: bool,
    /// Physical address of the device's PCI config space (from _INI's
    /// OperationRegion), or 0 when the table has none.
    pub cfg: u64,
    /// The node's _UID when it is a constant integer, else 0. Tells apart the
    /// instances of a _HID the table repeats (QEMU's 32 virtio-mmio slots).
    pub uid: u32,
}

const XHCI_HID_STRING: &[u8] = b"_HID\x0DPNP0D10\x00";
/// EISAID("PNP0D10") as the DWordConst form of _HID.
const XHCI_HID_EISAID: &[u8] = b"_HID\x0C\x41\xD0\x0D\x10";
const GENET_HID_STRING: &[u8] = b"_HID\x0DBCM6E4E\x00";
const SDHC_HID_STRING: &[u8] = b"_HID\x0DBRCME88C\x00";
const VIRTIO_MMIO_HID_STRING: &[u8] = b"_HID\x0DLNRO0005\x00";
/// The Pi 5's SD hosts (worproject): the card slot's _HID is a Method returning
/// this string, so match the string alone.
const SDHC5_HID_STRING: &[u8] = b"\x0DBRCM5D12\x00";
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

/// Where a template leaves values for its _CRS method to fill in: the byte
/// offsets (within the template) of the first memory range's base and of the
/// first interrupt's number, when the template holds 0 there.
#[derive(Clone, Copy, Default)]
struct Placeholders {
    base: Option<usize>,
    len: Option<usize>,
    gsiv: Option<usize>,
}

/// Walk a resource template (the bytes of a Buffer), taking the first memory
/// range and the first interrupt, and noting the fields left as 0 for the
/// method to patch.
fn parse_resources(t: &[u8], x: &mut AcpiDevice) -> Placeholders {
    let mut holes = Placeholders::default();
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
                if x.mmio == 0 {
                    holes.base = Some(p + 3 + 11);
                    holes.len = Some(p + 3 + 35);
                }
            }
            // DWord Address Space, memory.
            0x87 if body.first() == Some(&0) && x.mmio == 0 => {
                x.mmio = u32_at(body, 7).unwrap_or(0) as u64;
                x.mmio_len = u32_at(body, 19).unwrap_or(0) as u64;
                if x.mmio == 0 {
                    holes.base = Some(p + 3 + 7);
                }
            }
            // Memory32Fixed.
            0x86 if x.mmio == 0 => {
                x.mmio = u32_at(body, 1).unwrap_or(0) as u64;
                x.mmio_len = u32_at(body, 5).unwrap_or(0) as u64;
                if x.mmio == 0 {
                    holes.base = Some(p + 4);
                }
            }
            // Extended Interrupt: flags bit 1 = edge-triggered.
            0x89 if x.gsiv == 0 && holes.gsiv.is_none() && body.len() >= 6 && body[1] >= 1 => {
                x.edge = body[0] & 2 != 0;
                x.gsiv = u32_at(body, 2).unwrap_or(0);
                if x.gsiv == 0 {
                    holes.gsiv = Some(p + 3 + 2);
                }
            }
            _ => {}
        }
        p += 3 + len;
    }
    holes
}

/// The extent of the Method whose name is at `name`: (start, end) of
/// `MethodOp PkgLength NameString ...`, or None when `name` is not a Method's.
fn method_body(b: &[u8], name: usize) -> Option<(usize, usize)> {
    // MethodOp, then a 1-4 byte PkgLength, then the name.
    (2..=5).filter_map(|back| name.checked_sub(back)).find_map(|op| {
        if b[op] != 0x14 {
            return None;
        }
        let (len, n) = pkg_length(b, op + 1)?;
        (op + 1 + n == name).then_some((name, (op + 1 + len).min(b.len())))
    })
}

/// The 4-character name a method body returns: `Return (^NAME)` / `(NAME)`.
fn returned_name(m: &[u8]) -> Option<&[u8]> {
    let r = m.iter().rposition(|&c| c == 0xA4)?;
    let mut q = r + 1;
    while matches!(m.get(q), Some(0x5C | 0x5E)) {
        q += 1;
    }
    m.get(q..q + 4)
}

/// A NameString at `p`: its last 4-character segment and the bytes it took.
/// Handles the root/parent prefixes and Dual/MultiNamePrefix paths.
fn name_string(b: &[u8], p: usize) -> Option<(&[u8], usize)> {
    let mut q = p;
    while matches!(b.get(q), Some(0x5C | 0x5E)) {
        q += 1;
    }
    let segs = match *b.get(q)? {
        0x2E => {
            q += 1;
            2
        }
        0x2F => {
            q += 2;
            *b.get(q - 1)? as usize
        }
        0x00 => return None,
        _ => 1,
    };
    let end = q + 4 * segs;
    let last = b.get(end - 4..end)?;
    last.iter().all(|&c| c == b'_' || c.is_ascii_uppercase() || c.is_ascii_digit()).then_some((last, end - p))
}

/// The value of an operand: an integer, or a name declared `Name (X, Integer)`
/// somewhere in the definition block (RP1B's PBAR/PINT live on the parent).
fn operand(b: &[u8], aml: &[u8], p: usize) -> Option<(u64, usize)> {
    if let Some(v) = integer(b, p) {
        return Some(v);
    }
    let (name, n) = name_string(b, p)?;
    let mut decl = [0x08, 0, 0, 0, 0];
    decl[1..].copy_from_slice(name);
    let at = find(aml, &decl)?;
    Some((integer(aml, at + 5)?.0, n))
}

/// The value a _CRS method writes into the template field at byte `offset`.
/// The method names the field with `CreateDWordField`/`CreateQWordField (RBUF,
/// offset, NAME)` and then sets NAME with `Store (src, NAME)` or `Add (a, b,
/// NAME)`, where each operand is an integer or a `Name` declared in the block:
/// pftf's Pi 4 stores constants, worproject's Pi 5 adds an offset to its
/// parent's PBAR. Anything else is not evaluated.
fn patched_field(crs: &[u8], aml: &[u8], offset: usize) -> Option<u64> {
    // The field's name.
    let mut field: Option<&[u8]> = None;
    let mut p = 0;
    while field.is_none() && p < crs.len() {
        if matches!(crs[p], 0x8A | 0x8F) {
            if let Some((_, n)) = name_string(crs, p + 1) {
                if let Some((idx, m)) = integer(crs, p + 1 + n) {
                    if idx as usize == offset {
                        field = name_string(crs, p + 1 + n + m).map(|(f, _)| f);
                    }
                }
            }
        }
        p += 1;
    }
    let field = field?;
    // The first Store/Add whose target is that name.
    for r in 0..crs.len() {
        let (value, used) = match crs[r] {
            0x70 => match operand(crs, aml, r + 1) {
                Some((v, n)) => (v, 1 + n),
                None => continue,
            },
            0x72 => match operand(crs, aml, r + 1).and_then(|(a, n)| {
                operand(crs, aml, r + 1 + n).map(|(b, m)| (a.wrapping_add(b), 1 + n + m))
            }) {
                Some(v) => v,
                None => continue,
            },
            _ => continue,
        };
        if name_string(crs, r + used).map(|(t, _)| t) == Some(field) {
            return Some(value);
        }
    }
    None
}

/// Find the device whose _HID is one of `hids` in one definition block
/// (DSDT/SSDT body), taking the first whose body `accept` agrees to.
fn find_device_in(aml: &[u8], hids: &[&[u8]], accept: fn(&[u8]) -> bool) -> Option<AcpiDevice> {
    for needle in hids {
        let mut at = 0;
        while let Some(o) = find(&aml[at..], needle) {
            let hid = at + o;
            at = hid + 1;
            let Some(end) = device_end(aml, hid) else { continue };
            if accept(&aml[hid..end]) {
                if let Some(x) = device_resources(aml, &aml[hid..end]) {
                    return Some(x);
                }
            }
        }
    }
    None
}

/// The end of the Device() enclosing the _HID at `hid`: the nearest DeviceOp
/// (5B 82) before it whose PkgLength covers it. A Name _HID sits 12 bytes after
/// the DeviceOp; a Method _HID returning a string (the Pi 5's SDC0), further.
fn device_end(aml: &[u8], hid: usize) -> Option<usize> {
    (hid.saturating_sub(64)..hid).rev().find_map(|i| {
        if aml[i] != 0x5B || aml.get(i + 1) != Some(&0x82) {
            return None;
        }
        let (len, _) = pkg_length(aml, i + 2)?;
        (i + 2 + len > hid).then_some((i + 2 + len).min(aml.len()))
    })
}

/// The device whose _HID name sits at `hid` in a definition block.
fn device_at(aml: &[u8], hid: usize) -> Option<AcpiDevice> {
    device_resources(aml, &aml[hid..device_end(aml, hid)?])
}

/// The resources of the device whose body (from its _HID on) is `body`.
fn device_resources(aml: &[u8], body: &[u8]) -> Option<AcpiDevice> {
    let mut x = AcpiDevice::default();
    // _CRS: the resource template is the first Buffer after the name, whether
    // _CRS is a Name or a Method building one; or, for a Method that builds
    // nothing, the Buffer of the Name it returns (`Name (RBUF, ...)` declared
    // beside it, as SDC3 does).
    if let Some(crs) = find(body, b"_CRS") {
        let method = method_body(body, crs);
        let inner = method.and_then(|(s, e)| body[s..e].iter().position(|&c| c == 0x11).map(|o| s + o));
        let b = match (method, inner) {
            (_, Some(b)) => Some(b),
            (Some((s, e)), None) => returned_name(&body[s..e]).and_then(|n| {
                let mut decl = [0x08, 0, 0, 0, 0, 0x11];
                decl[1..5].copy_from_slice(n);
                find(body, &decl).map(|d| d + 5)
            }),
            (None, None) => body[crs..].iter().position(|&c| c == 0x11).map(|o| crs + o),
        };
        if let Some(b) = b {
            let (plen, pn) = pkg_length(body, b + 1)?;
            let (size, sn) = integer(body, b + 1 + pn)?;
            let start = b + 1 + pn + sn;
            let end = (b + 1 + plen).min(start + size as usize).min(body.len());
            let holes = parse_resources(&body[start..end], &mut x);
            let (s, e) = method.unwrap_or((crs, body.len()));
            if let Some(field) = holes.base {
                x.mmio = patched_field(&body[s..e], aml, field).unwrap_or(0);
            }
            if let Some(len) = holes.len.and_then(|f| patched_field(&body[s..e], aml, f)) {
                x.mmio_len = len;
            }
            if let Some(field) = holes.gsiv {
                x.gsiv = patched_field(&body[s..e], aml, field).unwrap_or(0) as u32;
            }
        }
    }
    if let Some(uid) = find(body, b"_UID") {
        x.uid = integer(body, uid + 4).map_or(0, |(v, _)| v as u32);
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
    Some(x)
}

/// Every ACPI-only xHCI controller the DSDT and SSDTs declare: the Pi 4's VL805,
/// or the Pi 5's two RP1 controllers (XHC0, XHC1: one interrupt, shared).
pub(crate) unsafe fn for_each_xhci(mut f: impl FnMut(AcpiDevice)) {
    for hid in [XHCI_HID_STRING, XHCI_HID_EISAID] {
        for_each_device(hid, |mut x| {
            if x.mmio_len < XHCI_MIN_WINDOW {
                x.mmio_len = XHCI_MIN_WINDOW;
            }
            f(x)
        });
    }
}

/// Search the DSDT and every SSDT for the Raspberry Pi 4's GENET Ethernet MAC.
pub(crate) unsafe fn find_genet() -> Option<AcpiDevice> {
    find_device(&[GENET_HID_STRING], any)
}

/// Search the DSDT and every SSDT for the Raspberry Pi 4's EMMC2 SD host.
pub(crate) unsafe fn find_sdhc() -> Option<AcpiDevice> {
    find_device(&[SDHC_HID_STRING], any)
}

/// Search for the Raspberry Pi 5's SD card host (SDC0). SDC1 answers the same
/// _HID but is the Wi-Fi's SDIO host: both declare a child Device (the card is
/// SDC0's `SDMM`), so tell them apart by SDC1's child, `WLAN`.
pub(crate) unsafe fn find_sdhc_pi5() -> Option<AcpiDevice> {
    find_device(&[SDHC5_HID_STRING], |body| find(body, b"WLAN").is_none())
}

/// RP1's Ethernet MAC (a Cadence GEM, rev 0x00070109) sits 0x100000 into RP1's
/// peripheral window, on the same shared interrupt as RP1's xHCIs.
const RP1_GEM_OFFSET: u64 = 0x10_0000;
const RP1_GEM_LEN: u64 = 0x4000;
/// What worproject's `RP1B._STA` treats as "PBAR never patched": RP1 absent.
const RP1_PBAR_UNSET: u64 = 0xABCD_EF01_2345_6789;

/// The value of `Name (<name>, Integer)` declared anywhere in `aml`.
fn named_integer(aml: &[u8], name: &[u8; 4]) -> Option<u64> {
    let mut decl = [0x08, 0, 0, 0, 0];
    decl[1..].copy_from_slice(name);
    integer(aml, find(aml, &decl)? + 5).map(|(v, _)| v)
}

/// The Raspberry Pi 5's Ethernet, which its firmware's tables leave out:
/// worproject's DSDT declares RP1 (`RP1B`, an ACPI0004 container with the
/// window base `PBAR` and interrupt `PINT`) but only its two xHCIs. RP1's layout
/// is fixed, so the GEM is published from the same two names. Level-triggered
/// and shared, as the xHCIs are: RP1 raises one line for all of its sources.
pub(crate) unsafe fn find_rp1_gem() -> Option<AcpiDevice> {
    let mut found = None;
    for_each_definition_block(|aml| {
        if find(aml, b"RP1B").is_none() {
            return false;
        }
        if let (Some(pbar), Some(pint)) = (named_integer(aml, b"PBAR"), named_integer(aml, b"PINT")) {
            if pbar != 0 && pbar != RP1_PBAR_UNSET && pint != 0 {
                found = Some(AcpiDevice {
                    mmio: pbar + RP1_GEM_OFFSET,
                    mmio_len: RP1_GEM_LEN,
                    gsiv: pint as u32,
                    edge: false,
                    cfg: 0,
                    uid: 0,
                });
            }
        }
        found.is_some()
    });
    found
}

fn any(_: &[u8]) -> bool {
    true
}

/// Every `_HID` one definition block declares, as (Device name, id). A string
/// _HID is `0D <ascii> 00`; an EISAID is `0C` + a 4-byte compressed id.
fn device_ids_in(aml: &[u8], mut f: impl FnMut(&[u8], &[u8])) {
    let mut at = 0;
    while let Some(o) = find(&aml[at..], b"_HID") {
        let hid = at + o;
        at = hid + 4;
        let name = (hid.saturating_sub(12)..hid)
            .rev()
            .find(|&i| aml[i] == 0x5B && aml.get(i + 1) == Some(&0x82))
            .and_then(|dev| pkg_length(aml, dev + 2).map(|(_, n)| dev + 2 + n))
            .and_then(|n| aml.get(n..n + 4))
            .unwrap_or(b"????");
        let mut eisa = [0u8; 7];
        // A Method _HID (SDC0's): report the first string it returns.
        if let Some((m, e)) = method_body(aml, hid) {
            if let Some(o) = aml[m..e].iter().position(|&c| c == 0x0D) {
                let s = &aml[m + o + 1..e];
                f(name, &s[..s.iter().position(|&c| c == 0).unwrap_or(0).min(16)]);
            }
            continue;
        }
        let id: &[u8] = match aml.get(hid + 4) {
            Some(0x0D) => {
                let s = &aml[hid + 5..];
                &s[..s.iter().position(|&c| c == 0).unwrap_or(0).min(16)]
            }
            Some(0x0C) => match aml.get(hid + 5..hid + 9) {
                Some(b) => {
                    let v = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
                    eisa[0] = b'@' + ((v >> 26) & 0x1F) as u8;
                    eisa[1] = b'@' + ((v >> 21) & 0x1F) as u8;
                    eisa[2] = b'@' + ((v >> 16) & 0x1F) as u8;
                    for (k, sh) in [12u32, 8, 4, 0].iter().enumerate() {
                        eisa[3 + k] = b"0123456789ABCDEF"[((v >> sh) & 0xF) as usize];
                    }
                    &eisa
                }
                None => continue,
            },
            _ => continue,
        };
        f(name, id);
    }
}

/// Print every device the firmware's DSDT and SSDTs declare, so a new board's
/// inventory is one boot log away: `[ACPI] DSDT XHC0 _HID PNP0D10`.
pub(crate) unsafe fn log_device_ids() {
    extern "C" {
        fn DbgPrint(Format: *const u8, ...) -> u32;
    }
    let xsdt = XSDT_PA.load(Ordering::Relaxed);
    if xsdt == 0 {
        return;
    }
    let print = |table: &[u8; 5], name: &[u8], id: &[u8]| {
        let mut n = [0u8; 5];
        n[..name.len().min(4)].copy_from_slice(&name[..name.len().min(4)]);
        let mut i = [0u8; 17];
        i[..id.len().min(16)].copy_from_slice(&id[..id.len().min(16)]);
        DbgPrint(b"[ACPI] %s %s _HID %s\n\0".as_ptr(), table.as_ptr(), n.as_ptr(), i.as_ptr());
    };
    let mut dsdt_pa = 0u64;
    for_each_table(xsdt, |sig, t, len| {
        if sig == b"FACP" && len >= 148 {
            dsdt_pa = rd::<u64>(t, 140);
            if dsdt_pa == 0 {
                dsdt_pa = rd::<u32>(t, 40) as u64;
            }
        } else if sig == b"SSDT" && len > 36 {
            device_ids_in(core::slice::from_raw_parts(t.add(36), len - 36), |n, i| print(b"SSDT\0", n, i));
        }
    });
    if dsdt_pa != 0 {
        let h = map_table(dsdt_pa, 36);
        if !h.is_null() {
            let len = rd::<u32>(h, 4) as usize;
            unmap_table(h, 36);
            if len > 36 {
                let t = map_table(dsdt_pa, len);
                if !t.is_null() {
                    device_ids_in(core::slice::from_raw_parts(t.add(36), len - 36), |n, i| print(b"DSDT\0", n, i));
                    unmap_table(t, len);
                }
            }
        }
    }
}

/// Search the SSDTs, then the DSDT, for the first device with one of `hids`.
unsafe fn find_device(hids: &[&[u8]], accept: fn(&[u8]) -> bool) -> Option<AcpiDevice> {
    let mut found = None;
    for_each_definition_block(|aml| {
        found = find_device_in(aml, hids, accept);
        found.is_some()
    });
    found
}

/// Call `f` for every device whose _HID is `hid`, in every definition block.
unsafe fn for_each_device(hid: &[u8], mut f: impl FnMut(AcpiDevice)) {
    for_each_definition_block(|aml| {
        let mut p = 0;
        while let Some(o) = find(&aml[p..], hid) {
            if let Some(x) = device_at(aml, p + o) {
                f(x);
            }
            p += o + hid.len();
        }
        false
    });
}

/// QEMU virt's virtio-mmio transports (_HID LNRO0005, the Linaro ID the
/// kernel's virtio-mmio driver binds), each with its 0x200-byte register window
/// and level-triggered SPI. The table lists every slot; most are empty.
pub(crate) unsafe fn for_each_virtio_mmio(f: impl FnMut(AcpiDevice)) {
    for_each_device(VIRTIO_MMIO_HID_STRING, f)
}

/// Call `f` with the body of each SSDT and then the DSDT, until it returns true.
unsafe fn for_each_definition_block(mut f: impl FnMut(&[u8]) -> bool) {
    let xsdt = XSDT_PA.load(Ordering::Relaxed);
    if xsdt == 0 {
        return;
    }
    let mut done = false;
    let mut dsdt_pa = 0u64;
    for_each_table(xsdt, |sig, t, len| {
        if sig == b"FACP" && len >= 148 {
            // X_DSDT (ACPI 2.0+) at 140, else the 32-bit DSDT at 40.
            dsdt_pa = rd::<u64>(t, 140);
            if dsdt_pa == 0 {
                dsdt_pa = rd::<u32>(t, 40) as u64;
            }
        } else if sig == b"SSDT" && !done {
            done = f(core::slice::from_raw_parts(t.add(36), len - 36));
        }
    });
    if done || dsdt_pa == 0 {
        return;
    }
    let h = map_table(dsdt_pa, 36);
    if h.is_null() {
        return;
    }
    let len = rd::<u32>(h, 4) as usize;
    unmap_table(h, 36);
    if len > 36 {
        let t = map_table(dsdt_pa, len);
        if !t.is_null() {
            f(core::slice::from_raw_parts(t.add(36), len - 36));
            unmap_table(t, len);
        }
    }
}
