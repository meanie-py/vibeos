//! RTL Byte Swap Routines — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/byteswap.c` (97 lines) for amd64 and arm64.
//! Three functions: `RtlUshortByteSwap`, `RtlUlongByteSwap`, `RtlUlonglongByteSwap`.
//!
//! On i386, these are provided by `rtlswap.S` (assembly) because the calling
//! convention is FASTCALL with `@Name@N` decoration, which requires unstable
//! Rust features. The FFI exports here are guarded with `cfg(not(target_arch = "x86"))`.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

/// Swap bytes of a 16-bit unsigned value.
#[inline]
pub fn ushort_byte_swap(source: u16) -> u16 {
    source.swap_bytes()
}

/// Swap bytes of a 32-bit unsigned value.
#[inline]
pub fn ulong_byte_swap(source: u32) -> u32 {
    source.swap_bytes()
}

/// Swap bytes of a 64-bit unsigned value.
#[inline]
pub fn ulonglong_byte_swap(source: u64) -> u64 {
    source.swap_bytes()
}

// ============================================================================
// FFI entry points (amd64 + arm64 only)
//
// On these architectures, FASTCALL is identical to the default calling
// convention, so extern "C" produces correct code.
// On i386, FASTCALL uses ECX/EDX — handled by rtlswap.S instead.
// ============================================================================

#[cfg(all(not(test), not(target_arch = "x86")))]
mod ffi {
    use super::*;

    #[no_mangle]
    pub unsafe extern "C" fn RtlUshortByteSwap(source: u16) -> u16 {
        ushort_byte_swap(source)
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlUlongByteSwap(source: u32) -> u32 {
        ulong_byte_swap(source)
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlUlonglongByteSwap(source: u64) -> u64 {
        ulonglong_byte_swap(source)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ushort_swap() {
        assert_eq!(ushort_byte_swap(0x1234), 0x3412);
        assert_eq!(ushort_byte_swap(0x0000), 0x0000);
        assert_eq!(ushort_byte_swap(0xFFFF), 0xFFFF);
        assert_eq!(ushort_byte_swap(0xFF00), 0x00FF);
        assert_eq!(ushort_byte_swap(0x00FF), 0xFF00);
    }

    #[test]
    fn test_ulong_swap() {
        assert_eq!(ulong_byte_swap(0x12345678), 0x78563412);
        assert_eq!(ulong_byte_swap(0x00000000), 0x00000000);
        assert_eq!(ulong_byte_swap(0xFFFFFFFF), 0xFFFFFFFF);
        assert_eq!(ulong_byte_swap(0xFF000000), 0x000000FF);
        assert_eq!(ulong_byte_swap(0x000000FF), 0xFF000000);
    }

    #[test]
    fn test_ulonglong_swap() {
        assert_eq!(ulonglong_byte_swap(0x0102030405060708), 0x0807060504030201);
        assert_eq!(ulonglong_byte_swap(0x0000000000000000), 0x0000000000000000);
        assert_eq!(ulonglong_byte_swap(0xFFFFFFFFFFFFFFFF), 0xFFFFFFFFFFFFFFFF);
        assert_eq!(ulonglong_byte_swap(0xFF00000000000000), 0x00000000000000FF);
    }

    #[test]
    fn test_ushort_roundtrip() {
        for val in [0u16, 1, 0x1234, 0xABCD, u16::MAX] {
            assert_eq!(ushort_byte_swap(ushort_byte_swap(val)), val);
        }
    }

    #[test]
    fn test_ulong_roundtrip() {
        for val in [0u32, 1, 0x12345678, 0xDEADBEEF, u32::MAX] {
            assert_eq!(ulong_byte_swap(ulong_byte_swap(val)), val);
        }
    }

    #[test]
    fn test_ulonglong_roundtrip() {
        for val in [0u64, 1, 0x0102030405060708, 0xDEADBEEFCAFEBABE, u64::MAX] {
            assert_eq!(ulonglong_byte_swap(ulonglong_byte_swap(val)), val);
        }
    }
}
