//! The ACPI IORT (IO Remapping Table): where a PCI device's MSI writes end up.
//!
//! A root complex node maps each requester ID (bus/device/function) on its segment to
//! an output ID and names the node that receives it: an ITS group, whose ITS sees that
//! output as the DeviceID, or an SMMU, which maps it again on its own way to an ITS.
//! The same walk says whether the device's DMA crosses an SMMU, which matters to every
//! driver that hands a device a physical address.
//!
//! Pure over the table's bytes and nothing else, so it runs as a host test
//! (tools/test_arm64_firmware_tables.py); the HAL's ITS backend is the only caller.
//! Layouts: ACPICA actbl2.h (ACPI_IORT_*), which every revision since E.a keeps for
//! the fields read here.

/// Node types.
pub const ITS_GROUP: u8 = 0;
pub const NAMED_COMPONENT: u8 = 1;
pub const ROOT_COMPLEX: u8 = 2;
pub const SMMU_V1_V2: u8 = 3;
pub const SMMU_V3: u8 = 4;

const HEADER: usize = 36;
const NODE_COUNT: usize = 36;
const NODE_OFFSET: usize = 40;
/// Every node: type, length, revision, identifier, mapping count, mapping offset.
const NODE_HEADER: usize = 16;
const NODE_LENGTH: usize = 1;
const NODE_MAPPINGS: usize = 8;
const NODE_MAPPING_OFFSET: usize = 12;
/// ACPI_IORT_ITS_GROUP: ITS count, then that many GIC ITS IDs (the MADT's).
const ITS_GROUP_COUNT: usize = 16;
const ITS_GROUP_IDS: usize = 20;
/// ACPI_IORT_ROOT_COMPLEX.PciSegmentNumber.
const ROOT_COMPLEX_SEGMENT: usize = 28;
/// ACPI_IORT_SMMU.BaseAddress and ACPI_IORT_SMMU_V3.BaseAddress.
const SMMU_BASE: usize = 16;
/// ACPI_IORT_ID_MAPPING: input base, ID count minus one, output base, output node, flags.
const MAPPING: usize = 20;
/// The mapping is the node's own single ID (an SMMU's MSIs), not a range translation.
const MAPPING_SINGLE: u32 = 1;
/// Root complex -> SMMU -> ITS is two hops; anything past this is a loop.
const MAX_HOPS: usize = 8;

/// An SMMU on a device's path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Smmu {
    /// 2 for SMMUv1/v2 (one IORT node type), 3 for SMMUv3.
    pub version: u8,
    pub base: u64,
}

/// Where a requester ID's MSI writes land.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Route {
    /// The GIC ITS ID (the MADT's GIC ITS entry `GicItsId`) of the group's first ITS.
    pub its: u32,
    /// The DeviceID that ITS sees.
    pub device_id: u32,
    /// The first SMMU the path crosses, if any.
    pub smmu: Option<Smmu>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Not an IORT, a bad checksum, or a node or reference outside the table.
    Malformed,
    /// No root complex node describes this PCI segment.
    NoRootComplex(u32),
    /// The node at this table offset has no mapping covering the ID.
    NoMapping { node: u32, id: u32 },
    /// The path reaches a node type that is not an SMMU or an ITS group.
    Unsupported(u8),
    /// The path never reaches an ITS group.
    TooDeep,
}

fn read<T: Copy>(data: &[u8], offset: usize) -> Option<T> {
    let end = offset.checked_add(core::mem::size_of::<T>())?;
    let bytes = data.get(offset..end)?;
    // SAFETY: `bytes` is exactly size_of::<T>() long and T is plain data.
    Some(unsafe { core::ptr::read_unaligned(bytes.as_ptr() as *const T) })
}

/// The table, cut to its own length, if it is an IORT whose bytes sum to zero.
pub fn table(data: &[u8]) -> Option<&[u8]> {
    if data.get(..4)? != b"IORT" {
        return None;
    }
    let length = read::<u32>(data, 4)? as usize;
    let data = data.get(..length)?;
    if length < HEADER + 8 || data.iter().fold(0u8, |sum, b| sum.wrapping_add(*b)) != 0 {
        return None;
    }
    Some(data)
}

/// One node: its offset in the table, type, and whole bytes.
#[derive(Clone, Copy, Debug)]
pub struct Node<'a> {
    pub offset: usize,
    pub kind: u8,
    pub bytes: &'a [u8],
}

fn node_at(t: &[u8], offset: usize) -> Option<Node<'_>> {
    let kind = read::<u8>(t, offset)?;
    let length = read::<u16>(t, offset + NODE_LENGTH)? as usize;
    if length < NODE_HEADER {
        return None;
    }
    Some(Node { offset, kind, bytes: t.get(offset..offset.checked_add(length)?)? })
}

/// Every node, in table order. Stops at the first one that does not fit.
pub fn nodes(t: &[u8]) -> impl Iterator<Item = Node<'_>> {
    let count = read::<u32>(t, NODE_COUNT).unwrap_or(0) as usize;
    let mut offset = read::<u32>(t, NODE_OFFSET).unwrap_or(0) as usize;
    let mut left = count;
    core::iter::from_fn(move || {
        if left == 0 {
            return None;
        }
        left -= 1;
        let node = node_at(t, offset)?;
        offset += node.bytes.len();
        Some(node)
    })
}

impl Node<'_> {
    /// A root complex's PCI segment.
    pub fn segment(&self) -> Option<u32> {
        (self.kind == ROOT_COMPLEX).then(|| read::<u32>(self.bytes, ROOT_COMPLEX_SEGMENT)).flatten()
    }

    /// An SMMU node's register base.
    pub fn smmu(&self) -> Option<Smmu> {
        let version = match self.kind {
            SMMU_V1_V2 => 2,
            SMMU_V3 => 3,
            _ => return None,
        };
        Some(Smmu { version, base: read::<u64>(self.bytes, SMMU_BASE)? })
    }

    /// An ITS group's first GIC ITS ID.
    pub fn first_its(&self) -> Option<u32> {
        if self.kind != ITS_GROUP || read::<u32>(self.bytes, ITS_GROUP_COUNT)? == 0 {
            return None;
        }
        read::<u32>(self.bytes, ITS_GROUP_IDS)
    }

    /// Map `id` through this node's ID mappings: (output node offset, output ID).
    fn map(&self, id: u32) -> Option<(usize, u32)> {
        let count = read::<u32>(self.bytes, NODE_MAPPINGS)? as usize;
        let first = read::<u32>(self.bytes, NODE_MAPPING_OFFSET)? as usize;
        (0..count).find_map(|i| {
            let m = first.checked_add(i.checked_mul(MAPPING)?)?;
            let (input, ids, output, reference, flags) = (
                read::<u32>(self.bytes, m)?,
                read::<u32>(self.bytes, m + 4)?,
                read::<u32>(self.bytes, m + 8)?,
                read::<u32>(self.bytes, m + 12)?,
                read::<u32>(self.bytes, m + 16)?,
            );
            (flags & MAPPING_SINGLE == 0 && id >= input && id - input <= ids)
                .then(|| Some((reference as usize, output.checked_add(id - input)?)))
                .flatten()
        })
    }
}

/// Where requester ID `requester` on PCI segment `segment` sends its MSIs.
pub fn translate(t: &[u8], segment: u32, requester: u32) -> Result<Route, Error> {
    let rc = nodes(t)
        .find(|n| n.segment() == Some(segment))
        .ok_or(Error::NoRootComplex(segment))?;
    let (mut node, mut id, mut smmu) = (rc, requester, None);
    for _ in 0..MAX_HOPS {
        let (next, out) = node.map(id).ok_or(Error::NoMapping { node: node.offset as u32, id })?;
        node = node_at(t, next).ok_or(Error::Malformed)?;
        id = out;
        match node.kind {
            ITS_GROUP => {
                let its = node.first_its().ok_or(Error::Malformed)?;
                return Ok(Route { its, device_id: id, smmu });
            }
            SMMU_V1_V2 | SMMU_V3 => smmu = smmu.or(node.smmu()),
            kind => return Err(Error::Unsupported(kind)),
        }
    }
    Err(Error::TooDeep)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A node: header, `body` (the type-specific fields after it), then `mappings`
    /// (input, ids-1, output, output node offset, flags).
    fn node(kind: u8, body: &[u8], mappings: &[[u32; 5]]) -> Vec<u8> {
        let mapping_offset = NODE_HEADER + body.len();
        let length = mapping_offset + mappings.len() * MAPPING;
        let mut n = vec![kind];
        n.extend_from_slice(&(length as u16).to_le_bytes());
        n.push(0); // revision
        n.extend_from_slice(&0u32.to_le_bytes()); // identifier
        n.extend_from_slice(&(mappings.len() as u32).to_le_bytes());
        n.extend_from_slice(&(if mappings.is_empty() { 0 } else { mapping_offset as u32 }).to_le_bytes());
        n.extend_from_slice(body);
        for m in mappings {
            for v in m {
                n.extend_from_slice(&v.to_le_bytes());
            }
        }
        n
    }

    fn its_group(ids: &[u32]) -> Vec<u8> {
        let mut body = (ids.len() as u32).to_le_bytes().to_vec();
        for id in ids {
            body.extend_from_slice(&id.to_le_bytes());
        }
        node(ITS_GROUP, &body, &[])
    }

    fn root_complex(segment: u32, mappings: &[[u32; 5]]) -> Vec<u8> {
        // Memory access properties (8), ATS attribute (4), then the segment.
        let mut body = vec![0u8; ROOT_COMPLEX_SEGMENT - NODE_HEADER];
        body.extend_from_slice(&segment.to_le_bytes());
        body.extend_from_slice(&[0u8; 4]); // address size limit + reserved
        node(ROOT_COMPLEX, &body, mappings)
    }

    fn smmu_v3(base: u64, mappings: &[[u32; 5]]) -> Vec<u8> {
        let mut body = base.to_le_bytes().to_vec();
        body.extend_from_slice(&[0u8; 52]); // flags .. device ID mapping index
        node(SMMU_V3, &body, mappings)
    }

    /// An IORT holding `nodes` back to back; references are offsets into the result,
    /// which [`offsets`] computes before the nodes are built.
    fn iort(nodes: &[Vec<u8>]) -> Vec<u8> {
        let mut t = b"IORT".to_vec();
        t.extend_from_slice(&[0u8; HEADER - 4]);
        t.extend_from_slice(&(nodes.len() as u32).to_le_bytes());
        t.extend_from_slice(&((HEADER + 12) as u32).to_le_bytes());
        t.extend_from_slice(&[0u8; 4]); // reserved
        for n in nodes {
            t.extend_from_slice(n);
        }
        let length = t.len() as u32;
        t[4..8].copy_from_slice(&length.to_le_bytes());
        let sum = t.iter().fold(0u8, |s, b| s.wrapping_add(*b));
        t[9] = t[9].wrapping_sub(sum);
        t
    }

    /// The table offset of each node, given their lengths.
    fn offsets(lengths: &[usize]) -> Vec<u32> {
        let mut at = HEADER + 12;
        lengths
            .iter()
            .map(|l| {
                let o = at;
                at += l;
                o as u32
            })
            .collect()
    }

    /// QEMU virt at gic-version=3: one ITS group, one root complex mapping every
    /// requester ID straight through.
    fn qemu() -> Vec<u8> {
        let its = its_group(&[0]);
        let at = offsets(&[its.len()]);
        let rc = root_complex(0, &[[0, 0xFFFF, 0, at[0], 0]]);
        iort(&[its, rc])
    }

    #[test]
    fn qemu_maps_every_requester_straight_to_the_its() {
        let raw = qemu();
        let t = table(&raw).unwrap();
        assert_eq!(translate(t, 0, 0x48), Ok(Route { its: 0, device_id: 0x48, smmu: None }));
        assert_eq!(translate(t, 0, 0xFFFF), Ok(Route { its: 0, device_id: 0xFFFF, smmu: None }));
        assert_eq!(translate(t, 1, 0x48), Err(Error::NoRootComplex(1)));
    }

    /// QEMU virt with iommu=smmuv3: the root complex goes to the SMMU, whose first
    /// mapping is its own single MSI ID and whose second translates the range.
    #[test]
    fn a_path_through_an_smmu_is_followed_and_reported() {
        let its = its_group(&[7]);
        let probe = smmu_v3(0x0905_0000, &[[0, 0, 0, 0, 0], [0, 0, 0, 0, 0]]);
        let at = offsets(&[its.len(), probe.len()]);
        let smmu = smmu_v3(0x0905_0000, &[[0, 0, 0x7FFF, at[0], MAPPING_SINGLE], [0, 0xFFFF, 0x1_0000, at[0], 0]]);
        let rc = root_complex(0, &[[0, 0xFFFF, 0, at[1], 0]]);
        let raw = iort(&[its, smmu, rc]);
        let t = table(&raw).unwrap();
        assert_eq!(
            translate(t, 0, 0x48),
            Ok(Route { its: 7, device_id: 0x1_0048, smmu: Some(Smmu { version: 3, base: 0x0905_0000 }) })
        );
    }

    /// Two segments, each behind its own ITS, the way a server lays out its PCIe.
    #[test]
    fn each_segment_reaches_its_own_its() {
        let a = its_group(&[0]);
        let b = its_group(&[1, 2]);
        let at = offsets(&[a.len(), b.len()]);
        let rc0 = root_complex(0, &[[0, 0xFF, 0, at[0], 0]]);
        let rc1 = root_complex(1, &[[0x100, 0xFF, 0x2_0000, at[1], 0]]);
        let raw = iort(&[a, b, rc0, rc1]);
        let t = table(&raw).unwrap();
        assert_eq!(translate(t, 0, 0x10).map(|r| (r.its, r.device_id)), Ok((0, 0x10)));
        assert_eq!(translate(t, 1, 0x105).map(|r| (r.its, r.device_id)), Ok((1, 0x2_0005)));
        // Segment 0 maps only bus 0; segment 1 only bus 1.
        assert!(matches!(translate(t, 0, 0x100), Err(Error::NoMapping { id: 0x100, .. })));
        assert!(matches!(translate(t, 1, 0x5), Err(Error::NoMapping { .. })));
    }

    #[test]
    fn broken_paths_are_errors_not_guesses() {
        let its = its_group(&[0]);
        let at = offsets(&[its.len()]);
        // A reference past the end of the table.
        let raw = iort(&[its.clone(), root_complex(0, &[[0, 0xFFFF, 0, 0xFFFF, 0]])]);
        assert_eq!(translate(table(&raw).unwrap(), 0, 1), Err(Error::Malformed));
        // A reference to the root complex itself: a loop.
        let rc_at = at[0] + its.len() as u32;
        let raw = iort(&[its.clone(), root_complex(0, &[[0, 0xFFFF, 0, rc_at, 0]])]);
        assert_eq!(translate(table(&raw).unwrap(), 0, 1), Err(Error::Unsupported(ROOT_COMPLEX)));
        // An SMMU that maps to itself never reaches an ITS.
        let looped = smmu_v3(0x1000, &[[0, 0xFFFF, 0, at[0], 0]]);
        let raw = iort(&[looped, root_complex(0, &[[0, 0xFFFF, 0, at[0], 0]])]);
        assert_eq!(translate(table(&raw).unwrap(), 0, 1), Err(Error::TooDeep));
        // A single mapping is never a translation, even when its input matches.
        let raw = iort(&[its, root_complex(0, &[[0, 0xFFFF, 0, at[0], MAPPING_SINGLE]])]);
        assert!(matches!(translate(table(&raw).unwrap(), 0, 0), Err(Error::NoMapping { .. })));
    }

    #[test]
    fn a_damaged_table_is_refused_and_never_panics() {
        let raw = qemu();
        let mut bad = raw.clone();
        bad[20] ^= 1;
        assert!(table(&bad).is_none(), "checksum");
        let mut wrong = raw.clone();
        wrong[..4].copy_from_slice(b"APIC");
        assert!(table(&wrong).is_none());
        for len in 0..raw.len() {
            assert!(table(&raw[..len]).is_none());
            // Below the table check: the walk itself must survive any truncation.
            let _ = translate(&raw[..len], 0, 0x48);
            let _ = nodes(&raw[..len]).count();
        }
    }
}
