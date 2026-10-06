//! Pure-decision leaf kernels of `sdk/lib/rtl/atom.c`.
//!
//! The pure pieces that fall out of the atom-table translation: the
//! integer-atom predicate, the hash→bucket index, and the atom-name
//! length validation. No `unsafe`, no FFI — `cargo test`-able directly.
//! The cabi shims (FAST_MUTEX/handle-table FFI + UNICODE_STRING
//! reconstruction) wrap these.
//!
//! Faithful to ReactOS `sdk/lib/rtl/atom.c` (Thomas Weidenmueller).

#![allow(dead_code)]

/// `RTL_MAXIMUM_ATOM_LENGTH` (rtltypes.h) — max chars in a string atom.
pub const RTL_MAXIMUM_ATOM_LENGTH: u32 = 255;

/// `RTL_ATOM_IS_PINNED` (rtltypes.h) — Entry.Flags bit.
pub const RTL_ATOM_IS_PINNED: u8 = 0x1;

/// Atoms below this are integer atoms; at/above are string-atom handles.
pub const RTL_ATOM_MAXIMUM_INTEGER_ATOM: u16 = 0xC000;

/// Result of `RtlpCheckIntegerAtom`: is `AtomName` an integer atom, and
/// if so what 16-bit value? Mirrors the C predicate's two paths:
///   1. pointer with zero high half  → `LOWORD(ptr)` (0 ⇒ 0xC000)
///   2. real pointer to `L"#<digits>"` → parsed decimal (low 16 bits)
/// The `#`-string parse is left to the cabi shim (needs
/// RtlUnicodeStringToInteger); this models only the pointer-as-integer
/// path, which is the pure part.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IntegerAtom {
    /// `AtomName` was a packed integer-atom pointer; this is its value.
    Packed(u16),
    /// `AtomName` is a real string pointer — caller must inspect the
    /// string (`#`-prefix decimal, else genuine string atom).
    NotPacked,
}

/// The high-half test from `RtlpCheckIntegerAtom`: a pointer whose bits
/// above the low 16 are all clear is an integer atom packed into them.
/// `ptr_bits` is `(ULONG_PTR)AtomName`.
///
/// Exceeds the C, which tests `AtomName & 0xFFFF0000` and so looks only at
/// bits 16-31. On a 64-bit kernel every real pointer whose bits 16-31 happen
/// to be zero passed as an integer atom: NtAddAtom/NtFindAtom capture the
/// caller's name into a pool block, and when that block sat at
/// 0xFFFF....0000xxxx, EVERY name captured there "was" atom 0xxxxx and the
/// table was never consulted. Unrelated names then shared one atom, so
/// uxtheme's ux_WndContext answered to shell32's CPLFlags property (data 1)
/// and control-panel applets faulted in OnPostWinPosChanged with no window.
/// Windows' own test is MAKEINTATOM's: the whole pointer is at most 0xFFFF.
#[inline]
pub fn check_packed_integer_atom(ptr_bits: usize) -> IntegerAtom {
    if ptr_bits & !0xFFFF == 0 {
        let lo = (ptr_bits & 0xFFFF) as u16;
        // C: "if (LoValue == 0) LoValue = 0xC000;"
        IntegerAtom::Packed(if lo == 0 { 0xC000 } else { lo })
    } else {
        IntegerAtom::NotPacked
    }
}

/// Hash bucket index: `Hash % NumberOfBuckets`. Split out so the modulo
/// (and its zero guard) is tested independently of the FFI hash call.
#[inline]
pub fn bucket_index(hash: u32, number_of_buckets: u32) -> u32 {
    // C never guards NumberOfBuckets==0 because RtlCreateAtomTable forces
    // a minimum of 37; mirror that (caller guarantees non-zero).
    hash % number_of_buckets
}

/// `RtlCreateAtomTable`'s size clamp: "if (TableSize <= 1) TableSize = 37".
#[inline]
pub fn clamp_table_size(table_size: u32) -> u32 {
    if table_size <= 1 {
        37
    } else {
        table_size
    }
}

/// Whether a freshly-resolved string-atom name length is acceptable
/// (`AtomNameLen > RTL_MAXIMUM_ATOM_LENGTH` ⇒ invalid).
#[inline]
pub fn atom_name_len_ok(atom_name_len: u32) -> bool {
    atom_name_len <= RTL_MAXIMUM_ATOM_LENGTH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_integer_zero_becomes_c000() {
        assert_eq!(check_packed_integer_atom(0), IntegerAtom::Packed(0xC000));
    }

    #[test]
    fn packed_integer_low_half() {
        assert_eq!(check_packed_integer_atom(0x1234), IntegerAtom::Packed(0x1234));
        assert_eq!(check_packed_integer_atom(0xFFFF), IntegerAtom::Packed(0xFFFF));
    }

    #[test]
    fn real_pointer_not_packed() {
        assert_eq!(check_packed_integer_atom(0x1_0000), IntegerAtom::NotPacked);
        assert_eq!(check_packed_integer_atom(0xDEAD_BEEF), IntegerAtom::NotPacked);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn pointer_with_zero_bits_16_to_31_is_not_packed() {
        // A paged-pool name buffer at ...0000_7480 is a string, not atom 0x7480.
        assert_eq!(check_packed_integer_atom(0xFFFF_FA00_0000_7480), IntegerAtom::NotPacked);
        assert_eq!(check_packed_integer_atom(0x0000_0001_0000_0000), IntegerAtom::NotPacked);
    }

    #[test]
    fn bucket_index_basic() {
        assert_eq!(bucket_index(100, 37), 100 % 37);
        assert_eq!(bucket_index(0, 37), 0);
        assert_eq!(bucket_index(37, 37), 0);
    }

    #[test]
    fn clamp_table_size_uses_default_for_small() {
        assert_eq!(clamp_table_size(0), 37);
        assert_eq!(clamp_table_size(1), 37);
        assert_eq!(clamp_table_size(2), 2);
        assert_eq!(clamp_table_size(37), 37);
        assert_eq!(clamp_table_size(1000), 1000);
    }

    #[test]
    fn atom_name_len_boundary() {
        assert!(atom_name_len_ok(0));
        assert!(atom_name_len_ok(255));
        assert!(!atom_name_len_ok(256));
    }
}
