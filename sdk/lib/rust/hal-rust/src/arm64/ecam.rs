//! PCI configuration space on arm64: ECAM, as the ACPI MCFG describes it.
//!
//! Each MCFG allocation is one PCI segment's window over a range of buses; bus b,
//! device d, function f is the 4 KiB at base + (b << 20 | d << 15 | f << 12), and
//! the base addresses bus 0 even when the range starts later (PCI Firmware spec
//! 4.1.2). The bus number the HAL's bus-data calls take is Windows'
//! PCI_SEGMENT_BUS_NUMBER: bus in bits 7:0, segment in bits 23:8. So pci.sys reaches
//! a device behind a root port (bus 1 and up) or on a second root complex (segment
//! 1 and up, which Graviton-class machines have) the way it reaches bus 0.
//!
//! A bus is mapped the first time it is touched, 1 MiB at a time, and stays mapped.
//!
//! An MCFG on the quirk list describes a host bridge that is not ECAM and is read as
//! no ECAM at all (the loader keeps the same list: freeldr's arm64_serial_acpi.rs).
//! A machine with no MCFG keeps the probe this HAL always did: QEMU virt's high, then
//! low, ECAM base, taken when its host bridge answers at 00:00.0.
//!
//! The parser is pure, so tools/test_arm64_firmware_tables.py runs its tests on the
//! host.

#[cfg(target_arch = "aarch64")]
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
#[cfg(target_arch = "aarch64")]
use ntkrnl_sys::ddk;

/// One MCFG allocation: a segment's ECAM window over `start..=end`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub base: u64,
    pub segment: u16,
    pub start: u8,
    pub end: u8,
}

pub const MAX_WINDOWS: usize = 8;

/// OEM IDs whose MCFG must not be read as ECAM. "RPIFDN": the Raspberry Pi UEFI
/// firmware, whose MCFG names the Broadcom PCIe controller's own registers, which
/// SError on an ECAM read.
const MCFG_QUIRKS: &[[u8; 6]] = &[*b"RPIFDN"];

/// The MCFG's allocations, or None when the table is not an MCFG, is damaged, or is
/// quirked. ACPICA actbl1.h: the 36-byte header, 8 reserved bytes, then 16-byte
/// records {u64 Address, u16 PciSegment, u8 StartBusNumber, u8 EndBusNumber, u32}.
pub fn parse(t: &[u8]) -> Option<([Option<Window>; MAX_WINDOWS], usize)> {
    if t.len() < 44 || &t[..4] != b"MCFG" {
        return None;
    }
    let len = u32::from_le_bytes(t[4..8].try_into().ok()?) as usize;
    if len > t.len() || len < 44 || t[..len].iter().fold(0u8, |a, &b| a.wrapping_add(b)) != 0 {
        return None;
    }
    if MCFG_QUIRKS.iter().any(|q| t[10..16] == q[..]) {
        return None;
    }
    let mut out = [None; MAX_WINDOWS];
    let mut n = 0;
    for r in t[44..len].chunks_exact(16) {
        let w = Window {
            base: u64::from_le_bytes(r[0..8].try_into().ok()?),
            segment: u16::from_le_bytes(r[8..10].try_into().ok()?),
            start: r[10],
            end: r[11],
        };
        if w.base == 0 || w.start > w.end || n == MAX_WINDOWS {
            continue;
        }
        out[n] = Some(w);
        n += 1;
    }
    Some((out, n))
}

/// Which window holds (segment, bus), by index.
pub fn find(windows: &[Option<Window>], segment: u16, bus: u8) -> Option<usize> {
    windows.iter().position(|w| w.is_some_and(|w| w.segment == segment && (w.start..=w.end).contains(&bus)))
}

/// The physical address of (bus, device, function)'s config space in `w`.
pub fn function_pa(w: &Window, bus: u8, slot: u32) -> u64 {
    let device = (slot & 0x1F) as u64;
    let function = ((slot >> 5) & 7) as u64;
    w.base + ((bus as u64) << 20 | device << 15 | function << 12)
}

#[cfg(target_arch = "aarch64")]
mod live {
    use super::*;
    use crate::arm64::acpi::{for_each_table, XSDT_PA};

    unsafe extern "C" {
        fn DbgPrint(Format: *const u8, ...) -> u32;
    }

    const QEMU_HIGH: Window = Window { base: 0x40_1000_0000, segment: 0, start: 0, end: 255 };
    const QEMU_LOW: Window = Window { base: 0x3F00_0000, segment: 0, start: 0, end: 15 };
    const BUS_BYTES: usize = 1 << 20;

    const UNPROBED: u8 = 0;
    const PROBING: u8 = 1;
    const READY: u8 = 2;
    static STATE: AtomicU8 = AtomicU8::new(UNPROBED);
    static mut WINDOWS: [Option<Window>; MAX_WINDOWS] = [None; MAX_WINDOWS];
    /// Each window's buses, mapped on first touch: 0 = not yet.
    static BUSES: [[AtomicUsize; 256]; MAX_WINDOWS] = [const { [const { AtomicUsize::new(0) }; 256] }; MAX_WINDOWS];

    unsafe fn map(pa: u64, bytes: usize) -> usize {
        let mut phys: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
        phys.QuadPart = pa as i64;
        ddk::MmMapIoSpace(phys, bytes as ddk::SIZE_T, ddk::_MEMORY_CACHING_TYPE_MmNonCached) as usize
    }

    /// The windows: the MCFG's, else QEMU's probed one.
    unsafe fn discover() {
        let windows = &raw mut WINDOWS;
        let mut mcfg: Option<Option<([Option<Window>; MAX_WINDOWS], usize)>> = None;
        let xsdt = XSDT_PA.load(Ordering::Relaxed);
        if xsdt != 0 {
            for_each_table(xsdt, |sig, t, len| {
                if sig == b"MCFG" && mcfg.is_none() {
                    mcfg = Some(parse(core::slice::from_raw_parts(t, len)));
                }
            });
        }
        match mcfg {
            Some(Some((found, n))) => {
                *windows = found;
                for w in found.iter().flatten() {
                    DbgPrint(
                        b"[HAL] PCI: segment %u buses %u-%u, ECAM at %p (MCFG)\n\0".as_ptr(),
                        w.segment as u32,
                        w.start as u32,
                        w.end as u32,
                        w.base as ddk::PVOID,
                    );
                }
                if n == 0 {
                    DbgPrint(b"[HAL] PCI: the MCFG lists no usable ECAM window\n\0".as_ptr());
                }
            }
            Some(None) => {
                DbgPrint(b"[HAL] PCI: the MCFG is quirked or damaged - no ECAM config space\n\0".as_ptr());
            }
            None => {
                for w in [QEMU_HIGH, QEMU_LOW] {
                    let va = map(w.base, BUS_BYTES);
                    if va == 0 {
                        continue;
                    }
                    if core::ptr::read_volatile(va as *const u32) & 0xFFFF != 0xFFFF {
                        (*windows)[0] = Some(w);
                        BUSES[0][0].store(va, Ordering::Release);
                        DbgPrint(b"[HAL] PCI: no MCFG; QEMU's ECAM at %p answered\n\0".as_ptr(), w.base as ddk::PVOID);
                        return;
                    }
                    ddk::MmUnmapIoSpace(va as ddk::PVOID, BUS_BYTES as ddk::SIZE_T);
                }
            }
        }
    }

    unsafe fn windows() -> &'static [Option<Window>; MAX_WINDOWS] {
        if STATE.compare_exchange(UNPROBED, PROBING, Ordering::AcqRel, Ordering::Acquire).is_ok() {
            discover();
            STATE.store(READY, Ordering::Release);
        }
        while STATE.load(Ordering::Acquire) != READY {
            core::hint::spin_loop();
        }
        &*(&raw const WINDOWS)
    }

    /// The mapped config space of `slot` on PCI_SEGMENT_BUS_NUMBER `bus_number`, or
    /// None where no window covers it.
    pub(crate) unsafe fn function(bus_number: u32, slot: u32) -> Option<*mut u8> {
        let (bus, segment) = (bus_number as u8, (bus_number >> 8) as u16);
        let windows = windows();
        let index = find(windows, segment, bus)?;
        let w = windows[index]?;
        let cell = &BUSES[index][bus as usize];
        let mut va = cell.load(Ordering::Acquire);
        if va == 0 {
            let fresh = map(function_pa(&w, bus, 0), BUS_BYTES);
            if fresh == 0 {
                return None;
            }
            va = match cell.compare_exchange(0, fresh, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => fresh,
                Err(won) => {
                    ddk::MmUnmapIoSpace(fresh as ddk::PVOID, BUS_BYTES as ddk::SIZE_T);
                    won
                }
            };
        }
        Some((va + (function_pa(&w, bus, slot) - function_pa(&w, bus, 0)) as usize) as *mut u8)
    }

    /// Every window, for the PnP driver's root buses.
    pub(crate) unsafe fn all() -> impl Iterator<Item = Window> {
        windows().iter().flatten().copied()
    }
}
#[cfg(target_arch = "aarch64")]
pub(crate) use live::{all, function};

#[cfg(test)]
mod tests {
    use super::*;

    fn mcfg(oem: &[u8; 6], records: &[(u64, u16, u8, u8)]) -> Vec<u8> {
        let mut t = vec![0u8; 44 + 16 * records.len()];
        t[..4].copy_from_slice(b"MCFG");
        let len = t.len() as u32;
        t[4..8].copy_from_slice(&len.to_le_bytes());
        t[10..16].copy_from_slice(oem);
        for (i, &(base, seg, start, end)) in records.iter().enumerate() {
            let r = 44 + 16 * i;
            t[r..r + 8].copy_from_slice(&base.to_le_bytes());
            t[r + 8..r + 10].copy_from_slice(&seg.to_le_bytes());
            t[r + 10] = start;
            t[r + 11] = end;
        }
        let sum = t.iter().fold(0u8, |a, &b| a.wrapping_add(b));
        t[9] = 0u8.wrapping_sub(sum);
        t
    }

    #[test]
    fn reads_every_segment() {
        // A Graviton-shaped table: several root complexes, each its own segment.
        let t = mcfg(b"AMAZON", &[(0x10_0000_0000, 0, 0, 0xFF), (0x20_0000_0000, 1, 0x10, 0x1F), (0x30_0000_0000, 2, 0, 0)]);
        let (w, n) = parse(&t).unwrap();
        assert_eq!(n, 3);
        assert_eq!(find(&w, 1, 0x12), Some(1));
        assert_eq!(find(&w, 1, 0x0F), None, "bus below the segment's range");
        assert_eq!(find(&w, 2, 0), Some(2));
        assert_eq!(find(&w, 3, 0), None);
        // The base addresses bus 0, even for a range that starts at 0x10.
        assert_eq!(function_pa(&w[1].unwrap(), 0x12, 3 | 2 << 5), 0x20_0000_0000 + (0x12 << 20 | 3 << 15 | 2 << 12));
    }

    #[test]
    fn refuses_quirked_damaged_and_foreign_tables() {
        assert_eq!(parse(&mcfg(b"RPIFDN", &[(0x10_0010_8000, 0, 0, 0)])), None);
        let mut t = mcfg(b"QEMU  ", &[(0x40_1000_0000, 0, 0, 0xFF)]);
        t[20] ^= 1;
        assert_eq!(parse(&t), None, "bad checksum");
        let mut t = mcfg(b"QEMU  ", &[]);
        t[..4].copy_from_slice(b"APIC");
        assert_eq!(parse(&t), None);
    }

    #[test]
    fn skips_unusable_records() {
        let (w, n) = parse(&mcfg(b"QEMU  ", &[(0, 0, 0, 0xFF), (0x40_1000_0000, 0, 5, 4), (0x40_1000_0000, 0, 0, 0xFF)])).unwrap();
        assert_eq!(n, 1);
        assert_eq!(w[0].unwrap().end, 0xFF);
    }
}
