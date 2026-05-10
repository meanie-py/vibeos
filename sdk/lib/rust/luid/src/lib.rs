//! RTL LUID (Locally Unique Identifier) Routines — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/luid.c` (64 lines).
//! Three functions: `RtlCopyLuid`, `RtlCopyLuidAndAttributesArray`, `RtlEqualLuid`.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

/// LUID — Locally Unique Identifier.
/// Matches Windows SDK `LUID` struct layout.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Luid {
    pub low_part: u32,
    pub high_part: i32,
}

/// LUID_AND_ATTRIBUTES — LUID with associated attributes.
/// Matches Windows SDK `LUID_AND_ATTRIBUTES` struct layout.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LuidAndAttributes {
    pub luid: Luid,
    pub attributes: u32,
}

// Compile-time size assertions
const _: () = assert!(core::mem::size_of::<Luid>() == 8);
const _: () = assert!(core::mem::size_of::<LuidAndAttributes>() == 12);

/// Copy a LUID from source to destination.
#[inline]
pub unsafe fn copy_luid(dest: *mut Luid, src: *const Luid) {
    unsafe {
        (*dest).low_part = (*src).low_part;
        (*dest).high_part = (*src).high_part;
    }
}

/// Copy an array of LUID_AND_ATTRIBUTES from source to destination.
#[inline]
pub unsafe fn copy_luid_and_attributes_array(
    count: u32,
    src: *const LuidAndAttributes,
    dest: *mut LuidAndAttributes,
) {
    for i in 0..count as usize {
        unsafe { *dest.add(i) = *src.add(i) };
    }
}

/// Compare two LUIDs for equality.
#[inline]
pub unsafe fn equal_luid(luid1: *const Luid, luid2: *const Luid) -> bool {
    unsafe {
        (*luid1).low_part == (*luid2).low_part && (*luid1).high_part == (*luid2).high_part
    }
}

// ============================================================================
// FFI entry points
// ============================================================================

#[cfg(not(test))]
mod ffi {
    use super::*;

    #[no_mangle]
    pub unsafe extern "system" fn RtlCopyLuid(dest: *mut Luid, src: *const Luid) {
        unsafe { copy_luid(dest, src) };
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlCopyLuidAndAttributesArray(
        count: u32,
        src: *const LuidAndAttributes,
        dest: *mut LuidAndAttributes,
    ) {
        unsafe { copy_luid_and_attributes_array(count, src, dest) };
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlEqualLuid(luid1: *const Luid, luid2: *const Luid) -> u8 {
        if unsafe { equal_luid(luid1, luid2) } { 1 } else { 0 }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_luid() {
        let src = Luid { low_part: 42, high_part: -1 };
        let mut dest = Luid { low_part: 0, high_part: 0 };
        unsafe { copy_luid(&mut dest, &src) };
        assert_eq!(dest, src);
    }

    #[test]
    fn test_copy_luid_zero() {
        let src = Luid { low_part: 0, high_part: 0 };
        let mut dest = Luid { low_part: 99, high_part: 99 };
        unsafe { copy_luid(&mut dest, &src) };
        assert_eq!(dest, src);
    }

    #[test]
    fn test_equal_luid_same() {
        let a = Luid { low_part: 100, high_part: 200 };
        let b = Luid { low_part: 100, high_part: 200 };
        assert!(unsafe { equal_luid(&a, &b) });
    }

    #[test]
    fn test_equal_luid_different_low() {
        let a = Luid { low_part: 100, high_part: 200 };
        let b = Luid { low_part: 101, high_part: 200 };
        assert!(!unsafe { equal_luid(&a, &b) });
    }

    #[test]
    fn test_equal_luid_different_high() {
        let a = Luid { low_part: 100, high_part: 200 };
        let b = Luid { low_part: 100, high_part: 201 };
        assert!(!unsafe { equal_luid(&a, &b) });
    }

    #[test]
    fn test_copy_luid_and_attributes_array() {
        let src = [
            LuidAndAttributes { luid: Luid { low_part: 1, high_part: 2 }, attributes: 0x10 },
            LuidAndAttributes { luid: Luid { low_part: 3, high_part: 4 }, attributes: 0x20 },
            LuidAndAttributes { luid: Luid { low_part: 5, high_part: 6 }, attributes: 0x30 },
        ];
        let mut dest = [LuidAndAttributes { luid: Luid { low_part: 0, high_part: 0 }, attributes: 0 }; 3];
        unsafe { copy_luid_and_attributes_array(3, src.as_ptr(), dest.as_mut_ptr()) };
        assert_eq!(dest, src);
    }

    #[test]
    fn test_copy_luid_and_attributes_empty() {
        let src: [LuidAndAttributes; 0] = [];
        let mut dest: [LuidAndAttributes; 0] = [];
        // count=0 should be a no-op
        unsafe { copy_luid_and_attributes_array(0, src.as_ptr(), dest.as_mut_ptr()) };
        assert_eq!(dest.len(), 0);
    }

    #[test]
    fn test_size_assertions() {
        assert_eq!(core::mem::size_of::<Luid>(), 8);
        assert_eq!(core::mem::size_of::<LuidAndAttributes>(), 12);
    }
}
