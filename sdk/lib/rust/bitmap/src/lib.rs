//! RTL Bitmap Functions — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/bitmap.c` (957 lines). Platform-independent: builds
//! for i386, amd64, and arm64 identically.
//!
//! - `ops.rs`: Core bitmap operations on `&[u32]` slices (testable, Kani-verifiable)
//! - `lib.rs`: FFI entry points matching the C ABI

#![cfg_attr(not(any(test, kani)), no_std)]
#![allow(unexpected_cfgs)]

pub mod ops;

/// RTL_BITMAP layout (matches `sdk/include/xdk/rtltypes.h`).
/// On i386: 8 bytes (4 + 4). On amd64/arm64: 16 bytes (4 + padding + 8).
#[repr(C)]
struct RtlBitmap {
    size_of_bitmap: u32,
    buffer: *mut u32,
}

/// RTL_BITMAP_RUN layout.
#[repr(C)]
struct RtlBitmapRun {
    starting_index: u32,
    number_of_bits: u32,
}

/// Convert raw RTL_BITMAP pointer to a Rust slice.
/// The caller guarantees the bitmap is valid for the duration of the call.
#[cfg(not(any(test, kani)))]
unsafe fn bitmap_slice(header: *const RtlBitmap) -> (&'static [u32], u32) {
    let h = unsafe { &*header };
    let words = ((h.size_of_bitmap + 31) / 32) as usize;
    let buf = unsafe { core::slice::from_raw_parts(h.buffer, words) };
    (buf, h.size_of_bitmap)
}

#[cfg(not(any(test, kani)))]
unsafe fn bitmap_slice_mut(header: *mut RtlBitmap) -> (&'static mut [u32], u32) {
    let h = unsafe { &*header };
    let words = ((h.size_of_bitmap + 31) / 32) as usize;
    let buf = unsafe { core::slice::from_raw_parts_mut(h.buffer, words) };
    (buf, h.size_of_bitmap)
}

// ============================================================================
// FFI entry points — #[no_mangle] extern "C" matching bitmap.c signatures
// ============================================================================

#[cfg(not(any(test, kani)))]
mod ffi {
    use super::*;

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindMostSignificantBit(value: u64) -> i8 {
        ops::find_most_significant_bit(value)
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindLeastSignificantBit(value: u64) -> i8 {
        ops::find_least_significant_bit(value)
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlInitializeBitMap(
        header: *mut RtlBitmap,
        buffer: *mut u32,
        size: u32,
    ) {
        let h = unsafe { &mut *header };
        h.size_of_bitmap = size;
        h.buffer = buffer;
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlClearAllBits(header: *mut RtlBitmap) {
        let (buf, size) = unsafe { bitmap_slice_mut(header) };
        ops::clear_all(buf, size);
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlSetAllBits(header: *mut RtlBitmap) {
        let (buf, size) = unsafe { bitmap_slice_mut(header) };
        ops::set_all(buf, size);
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlClearBit(header: *mut RtlBitmap, bit: u32) {
        let (buf, _) = unsafe { bitmap_slice_mut(header) };
        ops::clear_bit(buf, bit);
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlSetBit(header: *mut RtlBitmap, bit: u32) {
        let (buf, _) = unsafe { bitmap_slice_mut(header) };
        ops::set_bit(buf, bit);
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlTestBit(header: *const RtlBitmap, bit: u32) -> u8 {
        let (buf, _) = unsafe { bitmap_slice(header) };
        ops::test_bit(buf, bit) as u8
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlClearBits(header: *mut RtlBitmap, start: u32, count: u32) {
        let (buf, _) = unsafe { bitmap_slice_mut(header) };
        ops::clear_bits(buf, start, count);
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlSetBits(header: *mut RtlBitmap, start: u32, count: u32) {
        let (buf, _) = unsafe { bitmap_slice_mut(header) };
        ops::set_bits(buf, start, count);
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlAreBitsClear(
        header: *const RtlBitmap,
        start: u32,
        length: u32,
    ) -> u8 {
        let (buf, size) = unsafe { bitmap_slice(header) };
        ops::are_bits_clear(buf, size, start, length) as u8
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlAreBitsSet(
        header: *const RtlBitmap,
        start: u32,
        length: u32,
    ) -> u8 {
        let (buf, size) = unsafe { bitmap_slice(header) };
        ops::are_bits_set(buf, size, start, length) as u8
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlNumberOfSetBits(header: *const RtlBitmap) -> u32 {
        let (buf, size) = unsafe { bitmap_slice(header) };
        ops::number_of_set_bits(buf, size)
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlNumberOfClearBits(header: *const RtlBitmap) -> u32 {
        let (buf, size) = unsafe { bitmap_slice(header) };
        ops::number_of_clear_bits(buf, size)
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindClearBits(
        header: *const RtlBitmap,
        number_to_find: u32,
        hint: u32,
    ) -> u32 {
        if header.is_null() {
            return 0xFFFF_FFFF;
        }
        let (buf, size) = unsafe { bitmap_slice(header) };
        ops::find_clear_bits(buf, size, number_to_find, hint)
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindSetBits(
        header: *const RtlBitmap,
        number_to_find: u32,
        hint: u32,
    ) -> u32 {
        if header.is_null() {
            return 0xFFFF_FFFF;
        }
        let (buf, size) = unsafe { bitmap_slice(header) };
        ops::find_set_bits(buf, size, number_to_find, hint)
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindClearBitsAndSet(
        header: *mut RtlBitmap,
        number_to_find: u32,
        hint: u32,
    ) -> u32 {
        let (buf, size) = unsafe { bitmap_slice_mut(header) };
        ops::find_clear_bits_and_set(buf, size, number_to_find, hint)
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindSetBitsAndClear(
        header: *mut RtlBitmap,
        number_to_find: u32,
        hint: u32,
    ) -> u32 {
        let (buf, size) = unsafe { bitmap_slice_mut(header) };
        ops::find_set_bits_and_clear(buf, size, number_to_find, hint)
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindNextForwardRunClear(
        header: *const RtlBitmap,
        from: u32,
        starting_index: *mut u32,
    ) -> u32 {
        let (buf, size) = unsafe { bitmap_slice(header) };
        let (start, len) = ops::find_next_forward_run_clear(buf, size, from);
        unsafe { *starting_index = start };
        len
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindNextForwardRunSet(
        header: *const RtlBitmap,
        from: u32,
        starting_index: *mut u32,
    ) -> u32 {
        let (buf, size) = unsafe { bitmap_slice(header) };
        let (start, len) = ops::find_next_forward_run_set(buf, size, from);
        unsafe { *starting_index = start };
        len
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindFirstRunClear(
        header: *const RtlBitmap,
        starting_index: *mut u32,
    ) -> u32 {
        let (buf, size) = unsafe { bitmap_slice(header) };
        let (start, len) = ops::find_first_run_clear(buf, size);
        unsafe { *starting_index = start };
        len
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindLastBackwardRunClear(
        header: *const RtlBitmap,
        from: u32,
        starting_index: *mut u32,
    ) -> u32 {
        let (buf, size) = unsafe { bitmap_slice(header) };
        let (start, len) = ops::find_last_backward_run_clear(buf, size, from);
        unsafe { *starting_index = start };
        len
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindClearRuns(
        header: *const RtlBitmap,
        run_array: *mut RtlBitmapRun,
        array_size: u32,
        locate_longest: u8,
    ) -> u32 {
        let (buf, size) = unsafe { bitmap_slice(header) };
        // Create a temporary runs buffer (on stack, max reasonable size)
        let array_size = array_size as usize;
        // Use the raw pointer to write results directly
        let mut runs_buf: [_; 64] = [(0u32, 0u32); 64];
        let use_size = array_size.min(64);
        let count = ops::find_clear_runs(buf, size, &mut runs_buf[..use_size], locate_longest != 0);
        // Copy results to the C array
        for i in 0..count as usize {
            let entry = unsafe { &mut *run_array.add(i) };
            entry.starting_index = runs_buf[i].0;
            entry.number_of_bits = runs_buf[i].1;
        }
        count
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindLongestRunClear(
        header: *const RtlBitmap,
        starting_index: *mut u32,
    ) -> u32 {
        let (buf, size) = unsafe { bitmap_slice(header) };
        let (start, len) = ops::find_longest_run_clear(buf, size);
        unsafe { *starting_index = start };
        len
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFindLongestRunSet(
        header: *const RtlBitmap,
        starting_index: *mut u32,
    ) -> u32 {
        let (buf, size) = unsafe { bitmap_slice(header) };
        let (start, len) = ops::find_longest_run_set(buf, size);
        unsafe { *starting_index = start };
        len
    }
}

#[cfg(not(any(test, kani)))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
