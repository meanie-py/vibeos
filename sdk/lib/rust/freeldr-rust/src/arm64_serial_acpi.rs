//! Pre-allocator console-UART discovery from ACPI. Layouts follow ACPICA
//! actbl.h, actbl1.h (DBG2), and actbl3.h (SPCR). Keep parsing independent of
//! firmware pointers so captured hardware tables can be tested on the host.
//!
//! arm64 reads a PL011 MMIO base (DBG2/SPCR, plus the GIC from the MADT).
//! x86 reads a 16550-family I/O-port base from SPCR: real PCs without a legacy
//! COM1 describe their console UART there (Intel AMT serial-over-LAN, a BMC, a
//! PCIe serial card), at an address nothing else would guess.
#[cfg(test)]
use core::mem::offset_of;
use core::mem::size_of;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Header {
    pub signature: [u8; 4],
    pub length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: [u8; 4],
    creator_revision: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Address {
    space: u8,
    width: u8,
    bit_offset: u8,
    access: u8,
    address: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct SpcrPrefix {
    header: Header,
    interface: u8,
    reserved: [u8; 3],
    address: Address,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Dbg2 {
    header: Header,
    offset: u32,
    count: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Device {
    revision: u8,
    length: u16,
    register_count: u8,
    name_length: u16,
    name_offset: u16,
    oem_length: u16,
    oem_offset: u16,
    port_type: u16,
    subtype: u16,
    reserved: u16,
    address_offset: u16,
    size_offset: u16,
}

// MADT ("APIC") GIC records — ACPICA actbl2.h: ACPI_MADT_GENERIC_INTERRUPT
// (type 11) carries each core's CPU-interface base, ACPI_MADT_GENERIC_DISTRIBUTOR
// (type 12) the distributor base. GICv3+ reports 0 for the CPU interface (the
// redistributor replaces it), so a zero there is "absent", not an error.
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct SubHeader {
    kind: u8,
    length: u8,
}

const MADT_GICC: u8 = 11;
const MADT_GICD: u8 = 12;
/// ACPI_MADT_GENERIC_INTERRUPT.BaseAddress — the GICC MMIO base.
const GICC_BASE_OFFSET: usize = 32;
/// Through BaseAddress inclusive; shorter records cannot carry it.
const GICC_MIN_LENGTH: usize = GICC_BASE_OFFSET + 8;
/// ACPI_MADT_GENERIC_DISTRIBUTOR.BaseAddress.
const GICD_BASE_OFFSET: usize = 8;
const GICD_MIN_LENGTH: usize = GICD_BASE_OFFSET + 8;
/// MADT header: ACPI_TABLE_MADT is the common header + Address + Flags.
const MADT_RECORDS_OFFSET: usize = size_of::<Header>() + 8;

// ACPICA actbl1.h: ACPI_DBG2_SERIAL_PORT, ACPI_DBG2_ARM_PL011,
// ACPI_DBG2_ARM_SBSA_32BIT, ACPI_DBG2_ARM_SBSA_GENERIC.
const SERIAL_PORT: u16 = 0x8000;
const PL011: u16 = 3;
const SBSA_32: u16 = 0x0d;
const SBSA_GENERIC: u16 = 0x0e;

fn read<T: Copy>(data: &[u8], offset: usize) -> Option<T> {
    let bytes = data.get(offset..offset.checked_add(size_of::<T>())?)?;
    // All call sites use packed integer-only ACPI records, valid for any bits.
    Some(unsafe { core::ptr::read_unaligned(bytes.as_ptr().cast::<T>()) })
}

pub fn valid_table(data: &[u8]) -> bool {
    let Some(header) = read::<Header>(data, 0) else {
        return false;
    };
    header.length as usize == data.len()
        && data.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)) == 0
}

fn pl011(subtype: u16) -> bool {
    matches!(subtype, PL011 | SBSA_32 | SBSA_GENERIC)
}

fn mmio(address: Address) -> Option<u64> {
    // System memory, no bit shift, a 32-bit-capable register interface.
    if address.space != 0
        || address.bit_offset != 0
        || !matches!(address.access, 0 | 3)
        || address.address == 0
        || address.address & 3 != 0
    {
        return None;
    }
    Some(address.address)
}

pub fn serial_base(data: &[u8]) -> Option<u64> {
    if !valid_table(data) {
        return None;
    }
    let header = read::<Header>(data, 0)?;
    match &header.signature {
        b"SPCR" => {
            let spcr = read::<SpcrPrefix>(data, 0)?;
            if !pl011(spcr.interface as u16) {
                return None;
            }
            mmio(spcr.address)
        }
        b"DBG2" => {
            let dbg = read::<Dbg2>(data, 0)?;
            let mut offset = dbg.offset as usize;
            if offset < size_of::<Dbg2>() {
                return None;
            }
            for _ in 0..dbg.count {
                let dev = read::<Device>(data, offset)?;
                let length = dev.length as usize;
                if length < size_of::<Device>() {
                    return None;
                }
                let record = data.get(offset..offset.checked_add(length)?)?;
                if dev.port_type == SERIAL_PORT && pl011(dev.subtype) && dev.register_count != 0 {
                    if (dev.address_offset as usize) < size_of::<Device>()
                        || (dev.size_offset as usize) < size_of::<Device>()
                    {
                        return None;
                    }
                    let address = read::<Address>(record, dev.address_offset as usize)?;
                    let size = read::<u32>(record, dev.size_offset as usize)?;
                    // PL011 FR at 0x18 must fit as well as DR at zero.
                    if size >= 0x1c {
                        if let Some(base) = mmio(address) {
                            return Some(base);
                        }
                    }
                }
                offset += length;
            }
            None
        }
        _ => None,
    }
}

/// Firmware owns these identity-accessible allocations while the loader runs.
/// Reject implausible lengths before forming a slice; no allocator required.
pub unsafe fn table_at<'a>(address: u64) -> Option<&'a [u8]> {
    if address == 0 || address > usize::MAX as u64 {
        return None;
    }
    let ptr = address as *const Header;
    let length = core::ptr::read_unaligned(core::ptr::addr_of!((*ptr).length)) as usize;
    if !(size_of::<Header>()..=1024 * 1024).contains(&length) {
        return None;
    }
    let data = core::slice::from_raw_parts(ptr.cast::<u8>(), length);
    valid_table(data).then_some(data)
}

/// The first answer `probe` gives for a table the XSDT at `address` lists.
unsafe fn walk_xsdt<T>(address: u64, probe: impl Fn(&[u8]) -> Option<T>) -> Option<T> {
    let table = table_at(address)?;
    if table.get(..4)? != b"XSDT" {
        return None;
    }
    let entries = table.get(size_of::<Header>()..)?;
    if entries.len() % size_of::<u64>() != 0 {
        return None;
    }
    for entry in entries.chunks_exact(size_of::<u64>()) {
        let address = read::<u64>(entry, 0)?;
        if let Some(child) = table_at(address) {
            if let Some(found) = probe(child) {
                return Some(found);
            }
        }
    }
    None
}

pub unsafe fn from_xsdt(address: u64) -> Option<u64> {
    walk_xsdt(address, serial_base)
}

// SPCR interface types for the 16550 family (ACPI SPCR rev 2 / DBG2 serial
// subtypes): full 16550, 16450 (a 16550 subset), and 16550-compatible with the
// register layout given by the GAS.
const NS16550: u8 = 0x00;
const NS16450: u8 = 0x01;
const NS16550_GAS: u8 = 0x12;
const SYSTEM_IO: u8 = 1;

/// The I/O-port base of a 16550-family console UART that an SPCR describes.
/// None for anything else (MMIO UARTs, PL011, a malformed table): the x86
/// loader drives its UART with port I/O.
pub fn spcr_io_port(data: &[u8]) -> Option<u16> {
    if !valid_table(data) {
        return None;
    }
    let spcr = read::<SpcrPrefix>(data, 0)?;
    if &spcr.header.signature != b"SPCR"
        || !matches!(spcr.interface, NS16550 | NS16450 | NS16550_GAS)
    {
        return None;
    }
    let a = spcr.address;
    // A 16550 occupies eight consecutive ports, all of which must fit.
    if a.space != SYSTEM_IO || a.bit_offset != 0 || a.address == 0 || a.address > 0xfff8 {
        return None;
    }
    Some(a.address as u16)
}

pub unsafe fn spcr_io_from_xsdt(address: u64) -> Option<u16> {
    walk_xsdt(address, spcr_io_port)
}

/// GICv2 distributor and CPU-interface bases from a MADT. Returns None unless
/// BOTH are present: the early-SMP path needs the pair, and half a GIC is worse
/// than none (it would program a distributor nothing can acknowledge).
pub fn gic_bases(data: &[u8]) -> Option<(u64, u64)> {
    if !valid_table(data) {
        return None;
    }
    let header = read::<Header>(data, 0)?;
    if &header.signature != b"APIC" {
        return None;
    }
    let mut offset = MADT_RECORDS_OFFSET;
    let mut gicd = 0u64;
    let mut gicc = 0u64;
    while offset < data.len() {
        let sub = read::<SubHeader>(data, offset)?;
        let length = sub.length as usize;
        // A zero-length record would spin forever; a short one cannot be valid.
        if length < size_of::<SubHeader>() {
            return None;
        }
        let record = data.get(offset..offset.checked_add(length)?)?;
        match sub.kind {
            // Every enabled core repeats the same CPU-interface base; first wins.
            MADT_GICC if gicc == 0 && length >= GICC_MIN_LENGTH => {
                gicc = read::<u64>(record, GICC_BASE_OFFSET)?;
            }
            MADT_GICD if gicd == 0 && length >= GICD_MIN_LENGTH => {
                gicd = read::<u64>(record, GICD_BASE_OFFSET)?;
            }
            _ => {}
        }
        offset += length;
    }
    // Page-aligned MMIO is what the loader can identity-map a page of.
    if gicd == 0 || gicc == 0 || gicd & 0xFFF != 0 || gicc & 0xFFF != 0 {
        return None;
    }
    Some((gicd, gicc))
}

/// Walk the XSDT for the MADT and return its GICv2 bases.
pub unsafe fn gic_from_xsdt(address: u64) -> Option<(u64, u64)> {
    let table = table_at(address)?;
    if table.get(..4)? != b"XSDT" {
        return None;
    }
    let entries = table.get(size_of::<Header>()..)?;
    if entries.len() % size_of::<u64>() != 0 {
        return None;
    }
    for entry in entries.chunks_exact(size_of::<u64>()) {
        let address = read::<u64>(entry, 0)?;
        if let Some(child) = table_at(address) {
            if let Some(bases) = gic_bases(child) {
                return Some(bases);
            }
        }
    }
    None
}

/// The ECAM window of PCI segment 0 an MCFG describes: (base, first bus, last bus).
/// The base addresses bus 0 even when the range starts later (PCI Firmware spec 4.1.2).
/// ACPICA actbl1.h: ACPI_TABLE_MCFG is the header plus 8 reserved bytes, then 16-byte
/// ACPI_MCFG_ALLOCATION records {u64 Address, u16 PciSegment, u8 StartBusNumber,
/// u8 EndBusNumber, u32 Reserved}.
pub fn mcfg_segment0(data: &[u8]) -> Option<(u64, u8, u8)> {
    if !valid_table(data) || data.get(..4)? != b"MCFG" {
        return None;
    }
    let records = data.get(size_of::<Header>() + 8..)?;
    records.chunks_exact(16).find_map(|r| {
        let base = read::<u64>(r, 0)?;
        let segment = read::<u16>(r, 8)?;
        let (start, end) = (read::<u8>(r, 10)?, read::<u8>(r, 11)?);
        (segment == 0 && base != 0 && start <= end).then_some((base, start, end))
    })
}

pub unsafe fn mcfg_from_xsdt(address: u64) -> Option<(u64, u8, u8)> {
    walk_xsdt(address, mcfg_segment0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcfg_segment0_reads_the_allocation() {
        // QEMU virt with highmem: one allocation, segment 0, buses 0..=255.
        let mut data = vec![0u8; size_of::<Header>() + 8 + 16];
        data[..4].copy_from_slice(b"MCFG");
        let len = data.len() as u32;
        data[4..8].copy_from_slice(&len.to_le_bytes());
        let r = size_of::<Header>() + 8;
        data[r..r + 8].copy_from_slice(&0x40_1000_0000u64.to_le_bytes());
        data[r + 11] = 0xFF;
        checksum(&mut data);
        assert_eq!(mcfg_segment0(&data), Some((0x40_1000_0000, 0, 0xFF)));
        // Another segment only: nothing for segment 0.
        data[r + 8] = 1;
        checksum(&mut data);
        assert_eq!(mcfg_segment0(&data), None);
    }

    // Exact DBG2 bytes from Pi 4B UEFI v1.53, acpiview over COM5 on
    // 2026-09-20; unlike QEMU's SPCR-only path this catches real layout gaps.
    fn pi_dbg2() -> Vec<u8> {
        let hex = "4442473261000000009752504946444e52504934202020200002000045444b32000300002c00000001000000003500010f002600000000000080030000001600220000200003001020fe00000000001000005c5f53422e474456302e5552543000";
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    fn checksum(data: &mut [u8]) {
        let index = offset_of!(Header, checksum);
        data[index] = 0;
        data[index] = 0u8.wrapping_sub(data.iter().fold(0u8, |s, b| s.wrapping_add(*b)));
    }

    #[test]
    fn captured_pi_dbg2() {
        assert_eq!(serial_base(&pi_dbg2()), Some(0xfe201000));
    }

    #[test]
    fn rejects_truncated_corrupt_and_wrong_interface() {
        let original = pi_dbg2();
        for len in 0..original.len() {
            assert_eq!(serial_base(&original[..len]), None);
        }
        let mut data = original.clone();
        data[0] ^= 1;
        assert_eq!(serial_base(&data), None);
        data = original;
        data[size_of::<Dbg2>() + offset_of!(Device, subtype)] = 0;
        checksum(&mut data);
        assert_eq!(serial_base(&data), None);
    }

    #[test]
    fn rejects_out_of_record_address_and_zero_length() {
        for field in [
            offset_of!(Device, address_offset),
            offset_of!(Device, length),
        ] {
            let mut data = pi_dbg2();
            let at = size_of::<Dbg2>() + field;
            data[at..at + 2].copy_from_slice(&0u16.to_le_bytes());
            checksum(&mut data);
            assert_eq!(serial_base(&data), None);
        }
    }

    /// Synthetic, unlike `pi_dbg2` above: we have acpiview's decoded APIC
    /// summary for this board (GICv2, distributor 0xFF841000, CPU interface
    /// 0xFF842000) but not its raw bytes, so this pins the record LAYOUT and
    /// the board's values are confirmed against the loader's own printout.
    fn madt(gicd: u64, gicc: u64, gicc_len: u8) -> Vec<u8> {
        let mut data = vec![0u8; MADT_RECORDS_OFFSET];
        data[..4].copy_from_slice(b"APIC");
        // Four enabled cores, as this board reports, then the distributor —
        // so the walk must skip records it does not care about to reach it.
        for _ in 0..4 {
            let mut rec = vec![0u8; gicc_len as usize];
            rec[0] = MADT_GICC;
            rec[1] = gicc_len;
            if (gicc_len as usize) >= GICC_MIN_LENGTH {
                rec[GICC_BASE_OFFSET..GICC_BASE_OFFSET + 8].copy_from_slice(&gicc.to_le_bytes());
            }
            data.extend_from_slice(&rec);
        }
        let mut rec = vec![0u8; 24];
        rec[0] = MADT_GICD;
        rec[1] = 24;
        rec[GICD_BASE_OFFSET..GICD_BASE_OFFSET + 8].copy_from_slice(&gicd.to_le_bytes());
        data.extend_from_slice(&rec);
        let length = data.len() as u32;
        data[offset_of!(Header, length)..offset_of!(Header, length) + 4]
            .copy_from_slice(&length.to_le_bytes());
        checksum(&mut data);
        data
    }

    #[test]
    fn pi_style_madt() {
        assert_eq!(gic_bases(&madt(0xFF84_1000, 0xFF84_2000, 80)), Some((0xFF84_1000, 0xFF84_2000)));
    }

    #[test]
    fn madt_rejects_partial_unaligned_and_truncated() {
        // GICv3 reports no CPU interface: absent, not half a GIC.
        assert_eq!(gic_bases(&madt(0xFF84_1000, 0, 80)), None);
        // A GICC too short to carry BaseAddress leaves the pair incomplete.
        assert_eq!(gic_bases(&madt(0xFF84_1000, 0xFF84_2000, 32)), None);
        // Not page-aligned: the loader could not identity-map a page of it.
        assert_eq!(gic_bases(&madt(0xFF84_1000, 0xFF84_2004, 80)), None);
        let full = madt(0xFF84_1000, 0xFF84_2000, 80);
        for len in 0..full.len() {
            assert_eq!(gic_bases(&full[..len]), None);
        }
        // A zero-length subrecord must terminate the walk, not spin on it.
        let mut zero = full.clone();
        zero[MADT_RECORDS_OFFSET + 1] = 0;
        checksum(&mut zero);
        assert_eq!(gic_bases(&zero), None);
        // Right layout, wrong table: only a MADT may answer.
        let mut wrong = full;
        wrong[..4].copy_from_slice(b"SRAT");
        checksum(&mut wrong);
        assert_eq!(gic_bases(&wrong), None);
    }

    #[test]
    fn qemu_style_spcr() {
        let mut data = vec![0u8; 80];
        data[..4].copy_from_slice(b"SPCR");
        data[offset_of!(Header, length)..offset_of!(Header, length) + 4]
            .copy_from_slice(&80u32.to_le_bytes());
        data[offset_of!(SpcrPrefix, interface)] = PL011 as u8;
        let gas = offset_of!(SpcrPrefix, address);
        data[gas + offset_of!(Address, access)] = 3;
        let at = gas + offset_of!(Address, address);
        data[at..at + 8].copy_from_slice(&0x09000000u64.to_le_bytes());
        checksum(&mut data);
        assert_eq!(serial_base(&data), Some(0x09000000));
        data[gas + offset_of!(Address, space)] = 1;
        checksum(&mut data);
        assert_eq!(serial_base(&data), None);
    }

    fn spcr(interface: u8, space: u8, address: u64) -> Vec<u8> {
        let mut data = vec![0u8; 80];
        data[..4].copy_from_slice(b"SPCR");
        data[4..8].copy_from_slice(&80u32.to_le_bytes());
        data[8] = 2;
        data[36] = interface;
        data[40] = space;
        data[41] = 8;
        data[44..52].copy_from_slice(&address.to_le_bytes());
        checksum(&mut data);
        data
    }

    #[test]
    fn spcr_16550_in_io_space() {
        assert_eq!(spcr_io_port(&spcr(NS16550, SYSTEM_IO, 0x5000)), Some(0x5000));
        assert_eq!(spcr_io_port(&spcr(NS16550_GAS, SYSTEM_IO, 0x3f8)), Some(0x3f8));
        assert_eq!(spcr_io_port(&spcr(NS16450, SYSTEM_IO, 0x2f8)), Some(0x2f8));
        // MMIO 16550, PL011, port space overflow, zero, corrupt: not an x86 port UART.
        assert_eq!(spcr_io_port(&spcr(NS16550, 0, 0xfe00_0000)), None);
        assert_eq!(spcr_io_port(&spcr(PL011 as u8, SYSTEM_IO, 0x5000)), None);
        assert_eq!(spcr_io_port(&spcr(NS16550, SYSTEM_IO, 0xfffc)), None);
        assert_eq!(spcr_io_port(&spcr(NS16550, SYSTEM_IO, 0)), None);
        let mut bad = spcr(NS16550, SYSTEM_IO, 0x5000);
        bad[50] ^= 1;
        assert_eq!(spcr_io_port(&bad), None);
    }

}
