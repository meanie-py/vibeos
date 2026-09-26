//! RTL Compression Functions — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/compress.c` (439 lines). Platform-independent: builds
//! for i386, amd64, and arm64 identically.
//!
//! - `lznt1.rs`: Pure LZNT1 compression/decompression (testable, no unsafe)
//! - `lib.rs`: FFI entry points matching the C ABI

#![cfg_attr(not(any(test, kani)), no_std)]
#![allow(unexpected_cfgs)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod lznt1;

// ============================================================================
// NTSTATUS constants
// ============================================================================

const STATUS_SUCCESS: u32 = 0x0000_0000;
const STATUS_INVALID_PARAMETER: u32 = 0xC000_000D;
const STATUS_UNSUPPORTED_COMPRESSION: u32 = 0xC000_025F;
const STATUS_NOT_SUPPORTED: u32 = 0xC000_00BB;
const STATUS_NOT_IMPLEMENTED: u32 = 0xC000_0002;
const STATUS_BUFFER_TOO_SMALL: u32 = 0xC000_0023;

// Compression format/engine masks and values
const COMPRESSION_FORMAT_MASK: u16 = 0x00FF;
const COMPRESSION_ENGINE_MASK: u16 = 0xFF00;
const COMPRESSION_FORMAT_NONE: u16 = 0x0000;
const COMPRESSION_FORMAT_DEFAULT: u16 = 0x0001;
const COMPRESSION_FORMAT_LZNT1: u16 = 0x0002;
const COMPRESSION_ENGINE_MAXIMUM: u16 = 0x0100;

// ============================================================================
// FFI entry points
// ============================================================================

#[cfg(not(any(test, kani)))]
mod ffi {
    use super::*;

    #[no_mangle]
    pub unsafe extern "system" fn RtlCompressBuffer(
        format_and_engine: u16,
        uncompressed_buffer: *const u8,
        uncompressed_buffer_size: u32,
        compressed_buffer: *mut u8,
        compressed_buffer_size: u32,
        _uncompressed_chunk_size: u32,
        final_compressed_size: *mut u32,
        _workspace: *mut u8,
    ) -> u32 {
        let format = format_and_engine & COMPRESSION_FORMAT_MASK;

        if format == COMPRESSION_FORMAT_NONE || format == COMPRESSION_FORMAT_DEFAULT {
            return STATUS_INVALID_PARAMETER;
        }

        if format == COMPRESSION_FORMAT_LZNT1 {
            let src =
                unsafe { core::slice::from_raw_parts(uncompressed_buffer, uncompressed_buffer_size as usize) };
            let dst =
                unsafe { core::slice::from_raw_parts_mut(compressed_buffer, compressed_buffer_size as usize) };

            match lznt1::compress(dst, src) {
                Ok(written) => {
                    // compress.c: `if (final_size) *final_size = ...` — NULL is tolerated.
                    if !final_compressed_size.is_null() {
                        unsafe { *final_compressed_size = written as u32 };
                    }
                    STATUS_SUCCESS
                }
                Err(status) => status,
            }
        } else {
            STATUS_UNSUPPORTED_COMPRESSION
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDecompressBuffer(
        compression_format: u16,
        uncompressed_buffer: *mut u8,
        uncompressed_buffer_size: u32,
        compressed_buffer: *const u8,
        compressed_buffer_size: u32,
        final_uncompressed_size: *mut u32,
    ) -> u32 {
        unsafe {
            RtlDecompressFragment(
                compression_format,
                uncompressed_buffer,
                uncompressed_buffer_size,
                compressed_buffer,
                compressed_buffer_size,
                0,
                final_uncompressed_size,
                core::ptr::null_mut(),
            )
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDecompressFragment(
        format: u16,
        uncompressed: *mut u8,
        uncompressed_size: u32,
        compressed: *const u8,
        compressed_size: u32,
        offset: u32,
        final_size: *mut u32,
        workspace: *mut u8,
    ) -> u32 {
        let fmt = format & !COMPRESSION_ENGINE_MAXIMUM;

        match fmt {
            COMPRESSION_FORMAT_LZNT1 => {
                let src =
                    unsafe { core::slice::from_raw_parts(compressed, compressed_size as usize) };
                let dst =
                    unsafe { core::slice::from_raw_parts_mut(uncompressed, uncompressed_size as usize) };

                // Workspace: if provided and offset > 0, pass as Some
                let ws = if !workspace.is_null() {
                    Some(unsafe { core::slice::from_raw_parts_mut(workspace, 0x1000) })
                } else {
                    None
                };

                let result = lznt1::decompress(dst, src, offset as usize, ws);
                // compress.c writes *final_size only at its `out:` label — i.e. on success,
                // never on an early error return — and only when the pointer is non-NULL.
                // Writing unconditionally faulted on a NULL FinalUncompressedSize and
                // clobbered the caller's value on STATUS_BAD_COMPRESSION_BUFFER.
                if result.status == STATUS_SUCCESS && !final_size.is_null() {
                    unsafe { *final_size = result.bytes_written as u32 };
                }
                result.status
            }
            COMPRESSION_FORMAT_NONE | COMPRESSION_FORMAT_DEFAULT => STATUS_INVALID_PARAMETER,
            _ => STATUS_UNSUPPORTED_COMPRESSION,
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlGetCompressionWorkSpaceSize(
        format_and_engine: u16,
        compress_buffer_workspace_size: *mut u32,
        compress_fragment_workspace_size: *mut u32,
    ) -> u32 {
        let format = format_and_engine & COMPRESSION_FORMAT_MASK;
        let engine = format_and_engine & COMPRESSION_ENGINE_MASK;

        if format == COMPRESSION_FORMAT_NONE || format == COMPRESSION_FORMAT_DEFAULT {
            return STATUS_INVALID_PARAMETER;
        }

        if format == COMPRESSION_FORMAT_LZNT1 {
            let (buf_size, frag_size) = lznt1::workspace_size(engine);
            if buf_size == 0 {
                return STATUS_NOT_SUPPORTED;
            }
            unsafe {
                *compress_buffer_workspace_size = buf_size;
                *compress_fragment_workspace_size = frag_size;
            }
            STATUS_SUCCESS
        } else {
            STATUS_UNSUPPORTED_COMPRESSION
        }
    }

    // Unimplemented stubs matching the C code
    #[no_mangle]
    pub unsafe extern "system" fn RtlCompressChunks(
        _uncompressed_buffer: *const u8,
        _uncompressed_buffer_size: u32,
        _compressed_buffer: *mut u8,
        _compressed_buffer_size: u32,
        _compressed_data_info: *mut u8,
        _compressed_data_info_length: u32,
        _workspace: *mut u8,
    ) -> u32 {
        STATUS_NOT_IMPLEMENTED
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDecompressChunks(
        _uncompressed_buffer: *mut u8,
        _uncompressed_buffer_size: u32,
        _compressed_buffer: *const u8,
        _compressed_buffer_size: u32,
        _compressed_tail: *const u8,
        _compressed_tail_size: u32,
        _compressed_data_info: *const u8,
    ) -> u32 {
        STATUS_NOT_IMPLEMENTED
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDescribeChunk(
        _compression_format: u16,
        _compressed_buffer: *mut *mut u8,
        _end_of_compressed_buffer_plus1: *const u8,
        _chunk_buffer: *mut *mut u8,
        _chunk_size: *mut u32,
    ) -> u32 {
        STATUS_NOT_IMPLEMENTED
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlReserveChunk(
        _compression_format: u16,
        _compressed_buffer: *mut *mut u8,
        _end_of_compressed_buffer_plus1: *const u8,
        _chunk_buffer: *mut *mut u8,
        _chunk_size: u32,
    ) -> u32 {
        STATUS_NOT_IMPLEMENTED
    }
}
