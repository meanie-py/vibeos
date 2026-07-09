//! RTL UTF-8 Conversion Functions — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/utf8.c` (264 lines). Platform-independent: builds
//! for i386, amd64, and arm64 identically.
//!
//! - `conv.rs`: Pure UTF-16↔UTF-8 conversion (testable, no unsafe)
//! - `lib.rs`: FFI entry points matching the C ABI

#![cfg_attr(not(any(test, kani)), no_std)]
#![allow(unexpected_cfgs)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod conv;

// ============================================================================
// NTSTATUS constants
// ============================================================================

const STATUS_INVALID_PARAMETER: u32 = 0xC000_000D;
const STATUS_INVALID_PARAMETER_4: u32 = 0xC000_00F2;
const STATUS_INVALID_PARAMETER_5: u32 = 0xC000_00F3;

// ============================================================================
// FFI entry points
// ============================================================================

#[cfg(not(any(test, kani)))]
mod ffi {
    use super::*;

    /// RtlUnicodeToUTF8N — Convert UTF-16 to UTF-8.
    ///
    /// Parameters match the C prototype exactly:
    /// - `utf8_dest`: output buffer (NULL = count only)
    /// - `utf8_bytes_max`: size of output buffer
    /// - `utf8_bytes_written`: receives number of bytes written
    /// - `uni_src`: input UTF-16 buffer
    /// - `uni_bytes`: size of input in bytes (not characters)
    #[no_mangle]
    pub unsafe extern "system" fn RtlUnicodeToUTF8N(
        utf8_dest: *mut u8,
        utf8_bytes_max: u32,
        utf8_bytes_written: *mut u32,
        uni_src: *const u16,
        uni_bytes: u32,
    ) -> u32 {
        if uni_src.is_null() {
            return STATUS_INVALID_PARAMETER_4;
        }
        if utf8_bytes_written.is_null() {
            return STATUS_INVALID_PARAMETER;
        }
        if !utf8_dest.is_null() && (uni_bytes % 2) != 0 {
            return STATUS_INVALID_PARAMETER_5;
        }

        let uni_count = (uni_bytes / 2) as usize;
        let src = unsafe { core::slice::from_raw_parts(uni_src, uni_count) };

        let result = if utf8_dest.is_null() {
            conv::utf16_to_utf8_count(src)
        } else {
            let dst = unsafe {
                core::slice::from_raw_parts_mut(utf8_dest, utf8_bytes_max as usize)
            };
            conv::utf16_to_utf8(dst, src)
        };

        unsafe { *utf8_bytes_written = result.bytes_written as u32 };
        result.status
    }

    /// RtlUTF8ToUnicodeN — Convert UTF-8 to UTF-16.
    ///
    /// Parameters match the C prototype exactly:
    /// - `uni_dest`: output buffer (NULL = count only)
    /// - `uni_bytes_max`: size of output buffer in bytes
    /// - `uni_bytes_written`: receives number of bytes written
    /// - `utf8_src`: input UTF-8 buffer
    /// - `utf8_bytes`: size of input in bytes
    #[no_mangle]
    pub unsafe extern "system" fn RtlUTF8ToUnicodeN(
        uni_dest: *mut u16,
        uni_bytes_max: u32,
        uni_bytes_written: *mut u32,
        utf8_src: *const u8,
        utf8_bytes: u32,
    ) -> u32 {
        if utf8_src.is_null() {
            return STATUS_INVALID_PARAMETER_4;
        }
        if uni_bytes_written.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

        let src = unsafe { core::slice::from_raw_parts(utf8_src, utf8_bytes as usize) };

        let result = if uni_dest.is_null() {
            conv::utf8_to_utf16_count(src)
        } else {
            let uni_units_max = (uni_bytes_max / 2) as usize;
            let dst = unsafe {
                core::slice::from_raw_parts_mut(uni_dest, uni_units_max)
            };
            conv::utf8_to_utf16(dst, src)
        };

        unsafe { *uni_bytes_written = result.bytes_written as u32 };
        result.status
    }
}

#[cfg(not(any(test, kani)))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
