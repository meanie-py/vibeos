//! LZNT1 compression/decompression — pure algorithm, no unsafe.
//!
//! LZNT1 is a chunk-based LZ77 variant used by NTFS and the Windows NT kernel.
//! Each chunk is up to 4096 bytes uncompressed. Compressed chunks use variable-width
//! displacement/length encoding where the displacement bits shrink as more data
//! is output (giving longer match lengths for nearby references).

/// Decompress a single LZNT1 chunk.
///
/// Returns the number of bytes written to `dst`, or `None` on corrupt input.
pub fn decompress_chunk(dst: &mut [u8], src: &[u8]) -> Option<usize> {
    let mut si = 0usize;
    let mut di = 0usize;

    while si < src.len() && di < dst.len() {
        // Read flags header: 8 entities follow
        let flags_byte = src[si];
        si += 1;
        let mut flags: u16 = 0x8000 | flags_byte as u16;

        // Parse following 8 entities
        while (flags & 0xFF00) != 0 && si < src.len() {
            if (flags & 1) != 0 {
                // Backwards reference
                if si + 2 > src.len() {
                    return None;
                }
                let code = (src[si] as u16) | ((src[si + 1] as u16) << 8);
                si += 2;

                // Find displacement/length bits based on output position
                let mut displacement_bits: u32 = 12;
                while displacement_bits > 4 {
                    if (1u32 << (displacement_bits - 1)) < di as u32 {
                        break;
                    }
                    displacement_bits -= 1;
                }
                let length_bits = 16 - displacement_bits;
                let code_length = (code & ((1u16 << length_bits) - 1)) as usize + 3;
                let code_displacement = (code >> length_bits) as usize + 1;

                // Ensure reference is valid
                if di < code_displacement {
                    return None;
                }

                // Copy bytes — source and dest can overlap
                for _ in 0..code_length {
                    if di >= dst.len() {
                        return Some(di);
                    }
                    dst[di] = dst[di - code_displacement];
                    di += 1;
                }
            } else {
                // Uncompressed literal byte
                if di >= dst.len() {
                    return Some(di);
                }
                dst[di] = src[si];
                di += 1;
                si += 1;
            }
            flags >>= 1;
        }
    }

    Some(di)
}

/// Result from decompressing an LZNT1 stream.
pub struct DecompressResult {
    pub status: u32,
    pub bytes_written: usize,
}

// NTSTATUS constants (as u32 for no_std compatibility)
pub const STATUS_SUCCESS: u32 = 0x0000_0000;
pub const STATUS_BAD_COMPRESSION_BUFFER: u32 = 0xC000_0242;
pub const STATUS_ACCESS_VIOLATION: u32 = 0xC000_0005;

/// Decompress an LZNT1 stream with offset support (for fragment decompression).
///
/// - `dst`: output buffer
/// - `src`: compressed input
/// - `offset`: byte offset into the uncompressed data to start writing from
/// - `workspace`: optional 4096-byte workspace for partial chunk decompression
pub fn decompress(
    dst: &mut [u8],
    src: &[u8],
    offset: usize,
    workspace: Option<&mut [u8]>,
) -> DecompressResult {
    let mut si = 0usize;
    let mut di = 0usize;
    let mut remaining_offset = offset;

    if si + 2 > src.len() {
        return DecompressResult {
            status: STATUS_BAD_COMPRESSION_BUFFER,
            bytes_written: 0,
        };
    }

    // Skip over chunks with big distance (>= 0x1000) to the destination offset
    while remaining_offset >= 0x1000 && si + 2 <= src.len() {
        let chunk_header = read_u16(src, si);
        si += 2;
        if chunk_header == 0 {
            return DecompressResult {
                status: STATUS_SUCCESS,
                bytes_written: di,
            };
        }
        let chunk_size = (chunk_header & 0xFFF) as usize + 1;

        if si + chunk_size > src.len() {
            return DecompressResult {
                status: STATUS_BAD_COMPRESSION_BUFFER,
                bytes_written: 0,
            };
        }

        si += chunk_size;
        remaining_offset -= 0x1000;
    }

    // This chunk can be included partially
    if remaining_offset > 0 && si + 2 <= src.len() {
        let chunk_header = read_u16(src, si);
        si += 2;
        if chunk_header == 0 {
            return DecompressResult {
                status: STATUS_SUCCESS,
                bytes_written: di,
            };
        }
        let chunk_size = (chunk_header & 0xFFF) as usize + 1;

        if si + chunk_size > src.len() {
            return DecompressResult {
                status: STATUS_BAD_COMPRESSION_BUFFER,
                bytes_written: 0,
            };
        }

        if di < dst.len() {
            if (chunk_header & 0x8000) != 0 {
                // Compressed chunk — need workspace
                match workspace {
                    Some(ws_buf) if ws_buf.len() >= 0x1000 => {
                        let decompressed_len = match decompress_chunk(
                            &mut ws_buf[..0x1000],
                            &src[si..si + chunk_size],
                        ) {
                            Some(n) => n,
                            None => {
                                return DecompressResult {
                                    status: STATUS_BAD_COMPRESSION_BUFFER,
                                    bytes_written: 0,
                                };
                            }
                        };
                        if decompressed_len > remaining_offset {
                            let block_size =
                                (decompressed_len - remaining_offset).min(dst.len() - di);
                            dst[di..di + block_size].copy_from_slice(
                                &ws_buf[remaining_offset..remaining_offset + block_size],
                            );
                            di += block_size;
                        }
                    }
                    _ => {
                        return DecompressResult {
                            status: STATUS_ACCESS_VIOLATION,
                            bytes_written: 0,
                        };
                    }
                }
            } else {
                // Uncompressed chunk
                if chunk_size > remaining_offset {
                    let block_size = (chunk_size - remaining_offset).min(dst.len() - di);
                    dst[di..di + block_size].copy_from_slice(
                        &src[si + remaining_offset..si + remaining_offset + block_size],
                    );
                    di += block_size;
                }
            }
        }

        si += chunk_size;
    }

    // Handle remaining chunks
    while si + 2 <= src.len() {
        let chunk_header = read_u16(src, si);
        si += 2;
        if chunk_header == 0 {
            break;
        }
        let chunk_size = (chunk_header & 0xFFF) as usize + 1;

        if si + chunk_size > src.len() {
            return DecompressResult {
                status: STATUS_BAD_COMPRESSION_BUFFER,
                bytes_written: 0,
            };
        }

        // Add padding if required
        let block_offset = (di + offset) & 0xFFF;
        if block_offset != 0 {
            let pad = 0x1000 - block_offset;
            if di + pad >= dst.len() {
                break;
            }
            for b in &mut dst[di..di + pad] {
                *b = 0;
            }
            di += pad;
        }

        if di >= dst.len() {
            break;
        }

        if (chunk_header & 0x8000) != 0 {
            // Compressed chunk
            match decompress_chunk(&mut dst[di..], &src[si..si + chunk_size]) {
                Some(n) => di += n,
                None => {
                    return DecompressResult {
                        status: STATUS_BAD_COMPRESSION_BUFFER,
                        bytes_written: 0,
                    };
                }
            }
        } else {
            // Uncompressed chunk
            let block_size = chunk_size.min(dst.len() - di);
            dst[di..di + block_size].copy_from_slice(&src[si..si + block_size]);
            di += block_size;
        }

        si += chunk_size;
    }

    DecompressResult {
        status: STATUS_SUCCESS,
        bytes_written: di,
    }
}

/// Compress data as LZNT1.
///
/// Current implementation writes uncompressed chunks only (matches the C code).
/// Each chunk has a 2-byte header (0x3000 | (size-1)) followed by raw data.
///
/// Returns `Ok(bytes_written)` or `Err(STATUS_BUFFER_TOO_SMALL)`.
pub fn compress(dst: &mut [u8], src: &[u8]) -> Result<usize, u32> {
    let mut si = 0usize;
    let mut di = 0usize;

    while si < src.len() {
        let block_size = (src.len() - si).min(0x1000);
        if di + 2 + block_size > dst.len() {
            return Err(0xC000_0023); // STATUS_BUFFER_TOO_SMALL
        }

        // Write uncompressed chunk header: 0x3000 | (block_size - 1)
        let header = 0x3000u16 | (block_size as u16 - 1);
        dst[di] = header as u8;
        dst[di + 1] = (header >> 8) as u8;
        di += 2;

        // Write chunk content
        dst[di..di + block_size].copy_from_slice(&src[si..si + block_size]);
        di += block_size;
        si += block_size;
    }

    Ok(di)
}

/// Read a little-endian u16 from a byte slice at the given offset.
#[inline(always)]
fn read_u16(buf: &[u8], offset: usize) -> u16 {
    (buf[offset] as u16) | ((buf[offset + 1] as u16) << 8)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_roundtrip() {
        let data = b"Hello, World! Hello, World! Hello, World!";
        let mut compressed = [0u8; 256];
        let clen = compress(&mut compressed, data).unwrap();

        let mut decompressed = [0u8; 256];
        let result = decompress(&mut decompressed, &compressed[..clen], 0, None);
        assert_eq!(result.status, STATUS_SUCCESS);
        assert_eq!(&decompressed[..result.bytes_written], &data[..]);
    }

    #[test]
    fn test_compress_empty() {
        let mut compressed = [0u8; 256];
        let clen = compress(&mut compressed, &[]).unwrap();
        assert_eq!(clen, 0);
    }

    #[test]
    fn test_compress_exact_chunk() {
        // Exactly 4096 bytes
        let data = [0x42u8; 4096];
        let mut compressed = [0u8; 8192];
        let clen = compress(&mut compressed, &data).unwrap();

        // Should be header(2) + data(4096) = 4098
        assert_eq!(clen, 4098);

        let mut decompressed = [0u8; 4096];
        let result = decompress(&mut decompressed, &compressed[..clen], 0, None);
        assert_eq!(result.status, STATUS_SUCCESS);
        assert_eq!(result.bytes_written, 4096);
        assert_eq!(&decompressed[..], &data[..]);
    }

    #[test]
    fn test_compress_multi_chunk() {
        // 5000 bytes = two chunks (4096 + 904)
        let data = [0xABu8; 5000];
        let mut compressed = [0u8; 8192];
        let clen = compress(&mut compressed, &data).unwrap();

        let mut decompressed = [0u8; 5000];
        let result = decompress(&mut decompressed, &compressed[..clen], 0, None);
        assert_eq!(result.status, STATUS_SUCCESS);
        assert_eq!(result.bytes_written, 5000);
        assert_eq!(&decompressed[..], &data[..]);
    }

    #[test]
    fn test_decompress_chunk_literal() {
        // Flags byte: 0x00 means all 8 entities are literals
        let src = [0x00u8, b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H'];
        let mut dst = [0u8; 16];
        let n = decompress_chunk(&mut dst, &src).unwrap();
        assert_eq!(n, 8);
        assert_eq!(&dst[..8], b"ABCDEFGH");
    }

    #[test]
    fn test_decompress_chunk_backref() {
        // Build a compressed chunk: 4 literals "ABCD", then a back-reference
        // copying from displacement=4, length=3 (copies "ABC")
        //
        // Flags byte: bit 0-3 = 0 (4 literals), bit 4 = 1 (backref)
        // After 4 bytes output, displacement_bits = 4, length_bits = 12
        // (because (1 << 3) = 8 > 4 = dst_cur - dst, so loop goes to 4)
        // code = ((displacement-1) << length_bits) | (length-3)
        //      = (3 << 12) | 0 = 0x3000
        let src = [
            0x10, // flags: bit4=1 (backref at position 4), bits 0-3=0 (literals)
            b'A', b'B', b'C', b'D', // 4 literal bytes
            0x00, 0x30, // backref: displacement=4, length=3 (LE: 0x3000)
        ];
        let mut dst = [0u8; 16];
        let n = decompress_chunk(&mut dst, &src).unwrap();
        assert_eq!(n, 7);
        assert_eq!(&dst[..7], b"ABCDABC");
    }

    #[test]
    fn test_decompress_bad_input() {
        // Too short for chunk header
        let src = [0x01u8];
        let mut dst = [0u8; 256];
        let result = decompress(&mut dst, &src, 0, None);
        assert_eq!(result.status, STATUS_BAD_COMPRESSION_BUFFER);
    }

    #[test]
    fn test_decompress_empty_stream() {
        // Chunk header = 0 means end of stream
        let src = [0x00u8, 0x00];
        let mut dst = [0u8; 256];
        let result = decompress(&mut dst, &src, 0, None);
        assert_eq!(result.status, STATUS_SUCCESS);
        assert_eq!(result.bytes_written, 0);
    }

    #[test]
    fn test_decompress_with_offset() {
        // Create an uncompressed chunk of "ABCDEFGH" (8 bytes)
        let src = [
            0x07, 0x30, // header: uncompressed, size=8
            b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H',
        ];
        let mut dst = [0u8; 256];
        // Read starting from offset 3
        let result = decompress(&mut dst, &src, 3, None);
        assert_eq!(result.status, STATUS_SUCCESS);
        assert_eq!(result.bytes_written, 5);
        assert_eq!(&dst[..5], b"DEFGH");
    }

    #[test]
    fn test_workspace_size() {
        // STANDARD engine
        let (buf_size, frag_size) = workspace_size(0x0000); // STANDARD
        assert_eq!(buf_size, 0x8010);
        assert_eq!(frag_size, 0x1000);

        // MAXIMUM engine
        let (buf_size, frag_size) = workspace_size(0x0100); // MAXIMUM
        assert_eq!(buf_size, 0x10);
        assert_eq!(frag_size, 0x1000);
    }

    #[test]
    fn test_compress_buffer_too_small() {
        let data = [0u8; 100];
        let mut small_buf = [0u8; 10];
        assert!(compress(&mut small_buf, &data).is_err());
    }

    #[test]
    fn test_decompress_overlapping_backref() {
        // Create a pattern that uses overlapping back-references:
        // Output "A" then backref displacement=1, length=7 → "AAAAAAAA"
        // flags: bit0=0 (literal), bit1=1 (backref)
        // After 1 byte, displacement_bits = 12, length_bits = 4
        // code = ((1-1) << 4) | (7-3) = 0x0004
        let src = [
            0x02, // flags: bit0=0(literal), bit1=1(backref)
            b'A', // literal
            0x04, 0x00, // backref: displacement=1, length=7
        ];
        let mut dst = [0u8; 16];
        let n = decompress_chunk(&mut dst, &src).unwrap();
        assert_eq!(n, 8);
        assert_eq!(&dst[..8], b"AAAAAAAA");
    }
}

/// Get workspace sizes for LZNT1.
///
/// Returns `(buffer_and_workspace_size, fragment_workspace_size)` for the given engine,
/// or `(0, 0)` if the engine is not supported.
pub fn workspace_size(engine: u16) -> (u32, u32) {
    match engine {
        0x0000 => (0x8010, 0x1000), // STANDARD
        0x0100 => (0x0010, 0x1000), // MAXIMUM
        _ => (0, 0),
    }
}
