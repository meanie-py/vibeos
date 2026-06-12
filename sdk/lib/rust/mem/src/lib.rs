//! RTL Memory Routines — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/mem.c` (162 lines) for amd64 and arm64.
//! Seven functions: RtlCompareMemory, RtlCompareMemoryUlong,
//! RtlFillMemory, RtlFillMemoryUlong, RtlFillMemoryUlonglong,
//! RtlMoveMemory, RtlZeroMemory.
//!
//! On i386, all memory functions are provided by `i386/rtlmem.s` (assembly).
//! The FFI exports here are guarded with `cfg(not(target_arch = "x86"))`.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

// ============================================================================
// Algorithm
// ============================================================================

/// Compare two memory blocks byte-by-byte.
/// Returns the number of bytes that match before the first difference.
#[inline]
pub fn compare_memory(source1: *const u8, source2: *const u8, length: usize) -> usize {
    let mut i = 0;
    while i < length {
        if unsafe { *source1.add(i) != *source2.add(i) } {
            break;
        }
        i += 1;
    }
    i
}

/// Compare a block of memory against a ULONG pattern.
/// Returns the number of bytes that match (always a multiple of 4).
#[inline]
pub fn compare_memory_ulong(source: *const u32, length: usize, value: u32) -> usize {
    let count = length / 4;
    let mut i = 0;
    while i < count {
        if unsafe { *source.add(i) } != value {
            break;
        }
        i += 1;
    }
    i * 4
}

/// Fill memory with a ULONG pattern.
#[inline]
pub unsafe fn fill_memory_ulong(dest: *mut u32, length: usize, fill: u32) {
    let count = length / 4;
    let mut i = 0;
    while i < count {
        *dest.add(i) = fill;
        i += 1;
    }
}

/// Fill memory with a ULONGLONG pattern.
#[inline]
pub unsafe fn fill_memory_ulonglong(dest: *mut u64, length: usize, fill: u64) {
    let count = length / 8;
    let mut i = 0;
    while i < count {
        *dest.add(i) = fill;
        i += 1;
    }
}

// ============================================================================
// FFI entry points (amd64 + arm64 only)
//
// i386 uses rtlmem.s (assembly) for all memory functions.
// ============================================================================

#[cfg(all(not(test), not(target_arch = "x86")))]
mod ffi {
    use super::*;

    #[no_mangle]
    pub unsafe extern "system" fn RtlCompareMemory(
        source1: *const u8,
        source2: *const u8,
        length: usize,
    ) -> usize {
        compare_memory(source1, source2, length)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlCompareMemoryUlong(
        source: *const u32,
        length: usize,
        value: u32,
    ) -> usize {
        compare_memory_ulong(source, length, value)
    }

    /// RtlFillMemory(Destination, Length, Fill) — note parameter order differs
    /// from memset(dest, val, len). Calls write_bytes which lowers to memset.
    #[no_mangle]
    pub unsafe extern "system" fn RtlFillMemory(
        destination: *mut u8,
        length: usize,
        fill: u8,
    ) {
        core::ptr::write_bytes(destination, fill, length);
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlMoveMemory(
        destination: *mut u8,
        source: *const u8,
        length: usize,
    ) {
        core::ptr::copy(source, destination, length);
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlZeroMemory(
        destination: *mut u8,
        length: usize,
    ) {
        core::ptr::write_bytes(destination, 0, length);
    }

    // RtlFillMemoryUlong — arm64 + amd64 (rtl/mem.c was its real home on amd64;
    // the old "CRT provides it" state was a stale-fat-librtl.a zombie object).
    // i386 stays asm (rtlmem.s: _RtlFillMemoryUlong@12).
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    #[no_mangle]
    pub unsafe extern "system" fn RtlFillMemoryUlong(
        destination: *mut u32,
        length: usize,
        fill: u32,
    ) {
        fill_memory_ulong(destination, length, fill);
    }

    // RtlFillMemoryUlonglong — arm64 only (_WIN64 && !_M_AMD64)
    #[cfg(target_arch = "aarch64")]
    #[no_mangle]
    pub unsafe extern "system" fn RtlFillMemoryUlonglong(
        destination: *mut u64,
        length: usize,
        fill: u64,
    ) {
        fill_memory_ulonglong(destination, length, fill);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_memory_equal() {
        let a = [1u8, 2, 3, 4, 5];
        let b = [1u8, 2, 3, 4, 5];
        assert_eq!(compare_memory(a.as_ptr(), b.as_ptr(), 5), 5);
    }

    #[test]
    fn test_compare_memory_diff() {
        let a = [1u8, 2, 3, 4, 5];
        let b = [1u8, 2, 99, 4, 5];
        assert_eq!(compare_memory(a.as_ptr(), b.as_ptr(), 5), 2);
    }

    #[test]
    fn test_compare_memory_empty() {
        let a = [1u8];
        let b = [2u8];
        assert_eq!(compare_memory(a.as_ptr(), b.as_ptr(), 0), 0);
    }

    #[test]
    fn test_compare_memory_first_byte_diff() {
        let a = [0u8, 1, 2];
        let b = [1u8, 1, 2];
        assert_eq!(compare_memory(a.as_ptr(), b.as_ptr(), 3), 0);
    }

    #[test]
    fn test_compare_memory_ulong_all_match() {
        let data = [0xDEADBEEFu32, 0xDEADBEEF, 0xDEADBEEF];
        assert_eq!(compare_memory_ulong(data.as_ptr(), 12, 0xDEADBEEF), 12);
    }

    #[test]
    fn test_compare_memory_ulong_partial() {
        let data = [42u32, 42, 99, 42];
        assert_eq!(compare_memory_ulong(data.as_ptr(), 16, 42), 8);
    }

    #[test]
    fn test_compare_memory_ulong_none() {
        let data = [1u32, 2, 3];
        assert_eq!(compare_memory_ulong(data.as_ptr(), 12, 99), 0);
    }

    #[test]
    fn test_compare_memory_ulong_truncated_length() {
        // Length not multiple of 4 — remainder is ignored
        let data = [42u32, 42];
        assert_eq!(compare_memory_ulong(data.as_ptr(), 7, 42), 4);
    }

    #[test]
    fn test_fill_memory_ulong() {
        let mut data = [0u32; 4];
        unsafe { fill_memory_ulong(data.as_mut_ptr(), 16, 0xCAFEBABE) };
        assert_eq!(data, [0xCAFEBABE; 4]);
    }

    #[test]
    fn test_fill_memory_ulonglong() {
        let mut data = [0u64; 3];
        unsafe { fill_memory_ulonglong(data.as_mut_ptr(), 24, 0x0102030405060708) };
        assert_eq!(data, [0x0102030405060708; 3]);
    }
}
