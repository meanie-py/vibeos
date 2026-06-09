//! Shared type definitions for cryptlib-rust.
//! Sourced from c2rust transpilation of each .c file plus the C headers
//! (tomcrypt.h, md4.h, md5.h, sha1.h, rc4.h, mvAesAlg.h).
//! All structs verified #[repr(C)] to match the C layout.

// ---- LibTomCrypt types (tomcrypt.h) ----
pub type ulong32 = u32;
pub type ulong64 = u64;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tag_des_key {
    pub ek: [ulong32; 32],
    pub dk: [ulong32; 32],
}
pub type des_key = tag_des_key;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tag_des3_key {
    pub ek: [[ulong32; 32]; 3],
    pub dk: [[ulong32; 32]; 3],
}
pub type des3_key = tag_des3_key;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tag_aes_key {
    pub eK: [ulong32; 64],
    pub dK: [ulong32; 64],
    pub Nr: i32,
}
pub type aes_key = tag_aes_key;

// ---- MD4 CTX (md4.h) ----
// NOTE: field order must match C header: buf[4], i[2], in[64], digest[16]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MD4_CTX {
    pub buf: [u32; 4],
    pub i: [u32; 2],
    pub in_0: [u8; 64],
    pub digest: [u8; 16],
}

// ---- MD5 CTX (md5.h) ----
// NOTE: C header field order: i[2], buf[4], in[64], digest[16]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MD5_CTX {
    pub i: [u32; 2],
    pub buf: [u32; 4],
    pub in_0: [u8; 64],
    pub digest: [u8; 16],
}

// ---- SHA_CTX (sha1.h) ----
// NOTE: C header field order: Buffer[64], State[5], Count[2]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SHA_CTX {
    pub Buffer: [u8; 64],
    pub State: [u32; 5],
    pub Count: [u32; 2],
}
pub type PSHA_CTX = *mut SHA_CTX;

// ---- RC4 context (rc4.h) ----
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _RC4_CONTEXT {
    pub state: [u8; 256],
    pub x: u8,
    pub y: u8,
}
pub type RC4_CONTEXT = _RC4_CONTEXT;

// ---- mvAesAlg types (mvOs.h, mvAesAlg.h) ----
pub type MV_U8 = u8;
pub type MV_U32 = u32;

pub const MAXBC: usize = 8;
pub const MAXKC: usize = 8;
pub const MAXROUNDS: usize = 14;
