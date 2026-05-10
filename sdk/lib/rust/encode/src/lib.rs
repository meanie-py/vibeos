//! RTL Unicode String Encoding — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/encode.c` (78 lines).
//! Two functions: `RtlRunDecodeUnicodeString`, `RtlRunEncodeUnicodeString`.
//! XOR-based encoding/decoding of UNICODE_STRING buffers.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

/// UNICODE_STRING layout (matches `sdk/include/ndk/umtypes.h`).
/// On i386: 8 bytes (2+2+4). On amd64/arm64: 16 bytes (2+2+4pad+8).
#[repr(C)]
pub struct UnicodeString {
    pub length: u16,
    pub maximum_length: u16,
    pub buffer: *mut u16,
}

/// Decode a previously encoded UNICODE_STRING.
/// Operates on raw bytes of the buffer, XOR-ing from back to front.
pub unsafe fn run_decode(hash: u8, string: *mut UnicodeString) {
    let len = unsafe { (*string).length } as usize;
    let ptr = unsafe { (*string).buffer } as *mut u8;

    if len > 1 {
        for i in (1..len).rev() {
            let prev = unsafe { *ptr.add(i - 1) };
            unsafe { *ptr.add(i) ^= prev ^ hash };
        }
    }

    if len >= 1 {
        unsafe { *ptr ^= hash | 0x43 };
    }
}

/// Encode a UNICODE_STRING with XOR cipher.
/// If `*hash == 0`, generates a hash from the current system time.
pub unsafe fn run_encode(hash: *mut u8, string: *mut UnicodeString) {
    let len = unsafe { (*string).length } as usize;
    let ptr = unsafe { (*string).buffer } as *mut u8;

    if unsafe { *hash } == 0 {
        // Generate hash from system time
        generate_hash(hash);
    }

    let h = unsafe { *hash };

    if len >= 1 {
        unsafe { *ptr ^= h | 0x43 };
        for i in 1..len {
            let prev = unsafe { *ptr.add(i - 1) };
            unsafe { *ptr.add(i) ^= prev ^ h };
        }
    }
}

/// Generate a non-zero hash byte from system time.
#[cfg(not(test))]
unsafe fn generate_hash(hash: *mut u8) {
    extern "system" {
        fn NtQuerySystemTime(system_time: *mut i64) -> i32;
    }

    let mut time: i64 = 0;
    let status = unsafe { NtQuerySystemTime(&mut time) };

    if status >= 0 {
        // NT_SUCCESS
        let time_bytes = time.to_ne_bytes();
        for i in 1..8 {
            unsafe { *hash |= time_bytes[i] };
            if unsafe { *hash } != 0 {
                break;
            }
        }
    }

    if unsafe { *hash } == 0 {
        unsafe { *hash = 1 };
    }
}

/// Test stub: use a fixed hash instead of system time.
#[cfg(test)]
unsafe fn generate_hash(hash: *mut u8) {
    unsafe { *hash = 0x42 };
}

// ============================================================================
// FFI entry points
// ============================================================================

#[cfg(not(test))]
mod ffi {
    use super::*;

    #[no_mangle]
    pub unsafe extern "system" fn RtlRunDecodeUnicodeString(
        hash: u8,
        string: *mut UnicodeString,
    ) {
        unsafe { run_decode(hash, string) };
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlRunEncodeUnicodeString(
        hash: *mut u8,
        string: *mut UnicodeString,
    ) {
        unsafe { run_encode(hash, string) };
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_string(data: &mut [u8]) -> UnicodeString {
        UnicodeString {
            length: data.len() as u16,
            maximum_length: data.len() as u16,
            buffer: data.as_mut_ptr() as *mut u16,
        }
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = b"Hello World!".to_vec();
        let mut data = original.clone();
        let mut string = make_string(&mut data);

        let mut hash: u8 = 0x55;
        unsafe { run_encode(&mut hash, &mut string) };
        assert_eq!(hash, 0x55); // hash was non-zero, shouldn't change
        assert_ne!(data, original); // data should be modified

        unsafe { run_decode(hash, &mut string) };
        assert_eq!(data, original); // should be restored
    }

    #[test]
    fn test_encode_decode_empty() {
        let mut data: [u8; 0] = [];
        let mut string = make_string(&mut data);
        let mut hash: u8 = 0x55;
        unsafe { run_encode(&mut hash, &mut string) };
        unsafe { run_decode(hash, &mut string) };
        // No crash, no-op
    }

    #[test]
    fn test_encode_decode_single_byte() {
        let original = [0xAA];
        let mut data = original;
        let mut string = make_string(&mut data);

        let mut hash: u8 = 0x33;
        unsafe { run_encode(&mut hash, &mut string) };
        assert_ne!(data, original);

        unsafe { run_decode(hash, &mut string) };
        assert_eq!(data, original);
    }

    #[test]
    fn test_hash_generation() {
        let mut hash: u8 = 0;
        unsafe { generate_hash(&mut hash) };
        assert_ne!(hash, 0); // should never be zero after generate
    }

    #[test]
    fn test_encode_with_zero_hash() {
        let original = b"Test".to_vec();
        let mut data = original.clone();
        let mut string = make_string(&mut data);

        let mut hash: u8 = 0;
        unsafe { run_encode(&mut hash, &mut string) };
        assert_ne!(hash, 0); // hash should have been generated
        assert_ne!(data, original);

        unsafe { run_decode(hash, &mut string) };
        assert_eq!(data, original);
    }

    #[test]
    fn test_different_hashes_different_output() {
        let mut data1 = b"Same".to_vec();
        let mut data2 = b"Same".to_vec();
        let mut s1 = make_string(&mut data1);
        let mut s2 = make_string(&mut data2);

        let mut h1: u8 = 0x11;
        let mut h2: u8 = 0x22;
        unsafe { run_encode(&mut h1, &mut s1) };
        unsafe { run_encode(&mut h2, &mut s2) };
        assert_ne!(data1, data2);
    }

    #[test]
    fn test_size_assertions() {
        #[cfg(target_pointer_width = "64")]
        assert_eq!(core::mem::size_of::<UnicodeString>(), 16);
        #[cfg(target_pointer_width = "32")]
        assert_eq!(core::mem::size_of::<UnicodeString>(), 8);
    }
}
