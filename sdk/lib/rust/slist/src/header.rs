/// Unpacked SLIST_HEADER fields.
///
/// On 64-bit (Header16 variant):
/// - `Alignment` (u64 @ offset 0): `depth[15:0] | sequence[63:16]`
/// - `Region`    (u64 @ offset 8): `HeaderType[0] | Init[1] | Reserved[3:2] | NextEntry[63:4]`
///
/// On 32-bit:
/// - `Alignment` (u64 @ offset 0): `Next[31:0] | Depth[47:32] | Sequence[63:48]`
/// - `Region` is unused (always 0).
///
/// All entries are 16-byte aligned on 64-bit, 8-byte aligned on 32-bit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SListFields {
    pub depth: u16,
    pub sequence: u64, // 48 bits on 64-bit, 16 bits on 32-bit
    pub next_entry: u64, // Full pointer on 32-bit; Region value on 64-bit (ptr in bits[63:4])
}

// ============================================================================
// 64-bit layout (Header16)
// ============================================================================

#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub fn unpack(alignment: u64, region: u64) -> SListFields {
    SListFields {
        depth: (alignment & 0xFFFF) as u16,
        sequence: alignment >> 16,
        next_entry: region,
    }
}

#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub fn pack(f: &SListFields) -> (u64, u64) {
    let alignment = (f.depth as u64) | (f.sequence << 16);
    // When the list is non-empty, set HeaderType=1 and Init=1 (bits 0-1).
    // RtlFirstEntrySList checks HeaderType to select the code path.
    let region = if f.next_entry != 0 {
        f.next_entry | 0x3
    } else {
        0
    };
    (alignment, region)
}

#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub fn region_to_ptr(region: u64) -> u64 {
    region & !0xF
}

// ============================================================================
// 32-bit layout
// ============================================================================

#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub fn unpack(alignment: u64, _region: u64) -> SListFields {
    // Alignment layout: Next[31:0] | Depth[47:32] | Sequence[63:48]
    SListFields {
        depth: ((alignment >> 32) & 0xFFFF) as u16,
        sequence: alignment >> 48,
        next_entry: alignment & 0xFFFF_FFFF,
    }
}

#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub fn pack(f: &SListFields) -> (u64, u64) {
    // Alignment layout: Next[31:0] | Depth[47:32] | Sequence[63:48]
    let alignment = (f.next_entry & 0xFFFF_FFFF)
        | ((f.depth as u64) << 32)
        | ((f.sequence & 0xFFFF) << 48);
    (alignment, 0)
}

#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub fn region_to_ptr(region: u64) -> u64 {
    // On 32-bit, next_entry is stored directly (no low-bit flags)
    region
}

// ============================================================================
// Host-only helpers for testing both layouts from 64-bit host
// ============================================================================

#[cfg(test)]
pub mod pack32 {
    use super::SListFields;

    pub fn unpack(alignment: u64) -> SListFields {
        SListFields {
            depth: ((alignment >> 32) & 0xFFFF) as u16,
            sequence: alignment >> 48,
            next_entry: alignment & 0xFFFF_FFFF,
        }
    }

    pub fn pack(f: &SListFields) -> u64 {
        (f.next_entry & 0xFFFF_FFFF)
            | ((f.depth as u64) << 32)
            | ((f.sequence & 0xFFFF) << 48)
    }

    pub fn region_to_ptr(next_entry: u64) -> u64 {
        next_entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- 64-bit layout tests (native on host) ---

    #[test]
    fn pack_unpack_roundtrip() {
        let f = SListFields {
            depth: 42,
            sequence: 0x1234_5678_ABCD,
            next_entry: 0xDEAD_BEEF_0000,
        };
        let (a, r) = pack(&f);
        let f2 = unpack(a, r);
        assert_eq!(f.depth, f2.depth);
        assert_eq!(f.sequence, f2.sequence);
        // next_entry will have HeaderType+Init bits set
        assert_eq!(region_to_ptr(f2.next_entry), region_to_ptr(f.next_entry));
    }

    #[test]
    fn depth_in_low_bits() {
        let f = SListFields { depth: 7, sequence: 0, next_entry: 0 };
        let (a, _) = pack(&f);
        assert_eq!(a & 0xFFFF, 7);
    }

    #[test]
    fn sequence_in_high_bits() {
        let f = SListFields { depth: 0, sequence: 3, next_entry: 0 };
        let (a, _) = pack(&f);
        assert_eq!(a, 3 << 16);
    }

    #[test]
    fn region_ptr_mask() {
        assert_eq!(region_to_ptr(0xDEAD_BEEF_0013), 0xDEAD_BEEF_0010);
        assert_eq!(region_to_ptr(0), 0);
    }

    #[test]
    fn header_type_init_set_when_nonempty() {
        let f = SListFields {
            depth: 1,
            sequence: 1,
            next_entry: 0x1000_0000_0010, // 16-byte aligned
        };
        let (_, r) = pack(&f);
        // Bits 0 and 1 should be set
        assert_eq!(r & 0x3, 0x3);
        // Pointer should be recoverable
        assert_eq!(region_to_ptr(r), 0x1000_0000_0010);
    }

    #[test]
    fn header_type_init_zero_when_empty() {
        let f = SListFields {
            depth: 0,
            sequence: 5,
            next_entry: 0,
        };
        let (_, r) = pack(&f);
        assert_eq!(r, 0);
    }

    // --- 32-bit layout tests (using pack32 helper module) ---

    #[test]
    fn pack32_unpack_roundtrip() {
        let f = SListFields {
            depth: 42,
            sequence: 0x1234, // 16-bit on 32-bit
            next_entry: 0xDEAD_BEE0, // 32-bit pointer, 8-byte aligned
        };
        let a = pack32::pack(&f);
        let f2 = pack32::unpack(a);
        assert_eq!(f.depth, f2.depth);
        assert_eq!(f.sequence, f2.sequence);
        assert_eq!(f.next_entry, f2.next_entry);
    }

    #[test]
    fn pack32_layout_bits() {
        let f = SListFields {
            depth: 3,
            sequence: 7,
            next_entry: 0x1000_0000,
        };
        let a = pack32::pack(&f);
        // Next in low 32 bits
        assert_eq!(a & 0xFFFF_FFFF, 0x1000_0000);
        // Depth in bits 32-47
        assert_eq!((a >> 32) & 0xFFFF, 3);
        // Sequence in bits 48-63
        assert_eq!(a >> 48, 7);
    }

    #[test]
    fn pack32_empty_list() {
        let f = SListFields {
            depth: 0,
            sequence: 0,
            next_entry: 0,
        };
        let a = pack32::pack(&f);
        assert_eq!(a, 0);
    }

    #[test]
    fn pack32_sequence_wraps_16bit() {
        let f = SListFields {
            depth: 1,
            sequence: 0xFFFF,
            next_entry: 0x1000,
        };
        let a = pack32::pack(&f);
        let f2 = pack32::unpack(a);
        assert_eq!(f2.sequence, 0xFFFF);
    }

    #[test]
    fn pack32_region_to_ptr() {
        assert_eq!(pack32::region_to_ptr(0x1234_5678), 0x1234_5678);
        assert_eq!(pack32::region_to_ptr(0), 0);
    }
}
