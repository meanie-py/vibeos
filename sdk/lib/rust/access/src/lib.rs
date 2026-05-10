//! RTL Access Rights Routines — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/access.c` (65 lines).
//! Three functions: `RtlAreAllAccessesGranted`, `RtlAreAnyAccessesGranted`,
//! `RtlMapGenericMask`.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

// Windows generic access rights
const GENERIC_READ: u32 = 0x80000000;
const GENERIC_WRITE: u32 = 0x40000000;
const GENERIC_EXECUTE: u32 = 0x20000000;
const GENERIC_ALL: u32 = 0x10000000;

/// GENERIC_MAPPING — maps generic access rights to specific rights.
/// Matches Windows SDK `GENERIC_MAPPING` struct layout.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GenericMapping {
    pub generic_read: u32,
    pub generic_write: u32,
    pub generic_execute: u32,
    pub generic_all: u32,
}

// Compile-time size assertion
const _: () = assert!(core::mem::size_of::<GenericMapping>() == 16);

/// Check if all desired accesses are granted.
/// Returns true if every bit in `desired` is also set in `granted`.
#[inline]
pub fn are_all_accesses_granted(granted: u32, desired: u32) -> bool {
    (!granted & desired) == 0
}

/// Check if any desired access is granted.
/// Returns true if at least one bit in `desired` is also set in `granted`.
#[inline]
pub fn are_any_accesses_granted(granted: u32, desired: u32) -> bool {
    (granted & desired) != 0
}

/// Map generic access rights to specific rights using the provided mapping,
/// then clear the generic bits.
#[inline]
pub unsafe fn map_generic_mask(access_mask: *mut u32, mapping: *const GenericMapping) {
    let mask = unsafe { *access_mask };
    let map = unsafe { &*mapping };
    let mut result = mask;

    if mask & GENERIC_READ != 0 {
        result |= map.generic_read;
    }
    if mask & GENERIC_WRITE != 0 {
        result |= map.generic_write;
    }
    if mask & GENERIC_EXECUTE != 0 {
        result |= map.generic_execute;
    }
    if mask & GENERIC_ALL != 0 {
        result |= map.generic_all;
    }

    // Clear generic flags
    result &= !(GENERIC_READ | GENERIC_WRITE | GENERIC_EXECUTE | GENERIC_ALL);
    unsafe { *access_mask = result };
}

// ============================================================================
// FFI entry points
// ============================================================================

#[cfg(not(test))]
mod ffi {
    use super::*;

    #[no_mangle]
    pub unsafe extern "system" fn RtlAreAllAccessesGranted(
        granted_access: u32,
        desired_access: u32,
    ) -> u8 {
        if are_all_accesses_granted(granted_access, desired_access) { 1 } else { 0 }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlAreAnyAccessesGranted(
        granted_access: u32,
        desired_access: u32,
    ) -> u8 {
        if are_any_accesses_granted(granted_access, desired_access) { 1 } else { 0 }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlMapGenericMask(
        access_mask: *mut u32,
        generic_mapping: *const GenericMapping,
    ) {
        unsafe { map_generic_mask(access_mask, generic_mapping) };
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_accesses_granted() {
        assert!(are_all_accesses_granted(0xFF, 0x0F));
        assert!(are_all_accesses_granted(0xFF, 0xFF));
        assert!(are_all_accesses_granted(0xFF, 0x00));
        assert!(!are_all_accesses_granted(0x0F, 0xF0));
        assert!(!are_all_accesses_granted(0x00, 0x01));
    }

    #[test]
    fn test_any_accesses_granted() {
        assert!(are_any_accesses_granted(0xFF, 0x01));
        assert!(are_any_accesses_granted(0x01, 0x01));
        assert!(!are_any_accesses_granted(0xF0, 0x0F));
        assert!(!are_any_accesses_granted(0x00, 0xFF));
    }

    #[test]
    fn test_map_generic_mask_read() {
        let mapping = GenericMapping {
            generic_read: 0x01,
            generic_write: 0x02,
            generic_execute: 0x04,
            generic_all: 0x07,
        };
        let mut mask: u32 = GENERIC_READ;
        unsafe { map_generic_mask(&mut mask, &mapping) };
        assert_eq!(mask, 0x01); // GENERIC_READ mapped + generic bits cleared
    }

    #[test]
    fn test_map_generic_mask_all() {
        let mapping = GenericMapping {
            generic_read: 0x01,
            generic_write: 0x02,
            generic_execute: 0x04,
            generic_all: 0xFF,
        };
        let mut mask: u32 = GENERIC_ALL;
        unsafe { map_generic_mask(&mut mask, &mapping) };
        assert_eq!(mask, 0xFF);
    }

    #[test]
    fn test_map_generic_mask_combined() {
        let mapping = GenericMapping {
            generic_read: 0x01,
            generic_write: 0x02,
            generic_execute: 0x04,
            generic_all: 0x07,
        };
        let mut mask: u32 = GENERIC_READ | GENERIC_WRITE;
        unsafe { map_generic_mask(&mut mask, &mapping) };
        assert_eq!(mask, 0x03); // read | write mapped, generic bits cleared
    }

    #[test]
    fn test_map_generic_mask_preserves_specific() {
        let mapping = GenericMapping {
            generic_read: 0x01,
            generic_write: 0x02,
            generic_execute: 0x04,
            generic_all: 0x07,
        };
        let mut mask: u32 = GENERIC_READ | 0x100; // specific bit 0x100 already set
        unsafe { map_generic_mask(&mut mask, &mapping) };
        assert_eq!(mask, 0x101); // 0x01 from mapping + 0x100 preserved
    }

    #[test]
    fn test_map_generic_mask_no_generics() {
        let mapping = GenericMapping {
            generic_read: 0x01,
            generic_write: 0x02,
            generic_execute: 0x04,
            generic_all: 0x07,
        };
        let mut mask: u32 = 0x123; // no generic bits
        unsafe { map_generic_mask(&mut mask, &mapping) };
        assert_eq!(mask, 0x123); // unchanged
    }

    #[test]
    fn test_size_assertion() {
        assert_eq!(core::mem::size_of::<GenericMapping>(), 16);
    }
}
