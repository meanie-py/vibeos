//! des.rs — DES/3DES cipher (LibTomCrypt, sdk/lib/cryptlib/des.c).
//! Cohabitation module. ABI: cdecl (extern "C") for all public fns.
//! All tables sourced from c2rust transpilation (bit-exact vs C source).
//!
//! FIXES applied:
//!   - cookey: copy_nonoverlapping src/dst was swapped (CRITICAL bug fix).
//!   - ROR / byte: were extern "C" decls; replaced with local inline fns
//!     matching the tomcrypt.h macro expansions exactly.
//!   - Removed duplicate EN0/DE1/CRYPT_* consts (now in preamble).
//!   - des_ecb_encrypt/decrypt: kept extern "C" (cdecl, not NTAPI).

use crate::types::*;

/// ROR macro expansion from tomcrypt.h:
/// #define ROR(x, y) (((x>>y) | (x<<(32-(y)))) & 0xFFFFFFFF)
/// des.c uses GCC inline asm on x86 — we use the portable form.
#[inline(always)]
unsafe fn ROR(x: u32, y: i32) -> u32 {
    let y = (y & 31) as u32;
    if y == 0 { return x; }
    (x >> y) | (x << (32 - y))
}

/// byte macro from tomcrypt.h: #define byte(x, n) (((x) >> (8 * (n))) & 255)
#[inline(always)]
unsafe fn byte(x: u32, n: i32) -> u8 {
    ((x >> (8 * n as u32)) & 255) as u8
}

const CRYPT_OK: i32 = 0;
const CRYPT_INVALID_KEYSIZE: i32 = 3;
const CRYPT_INVALID_ROUNDS: i32 = 4;

pub const EN0: i32 = 0;
pub const DE1: i32 = 1;
static bytebit: [ulong32; 8] = [
    0o200 as u32,
    0o100 as u32,
    0o40 as u32,
    0o20 as u32,
    0o10 as u32,
    0o4 as u32,
    0o2 as u32,
    0o1 as u32,
];
static bigbyte: [ulong32; 24] = [
    0x800000 as u32,
    0x400000 as u32,
    0x200000 as u32,
    0x100000 as u32,
    0x80000 as u32,
    0x40000 as u32,
    0x20000 as u32,
    0x10000 as u32,
    0x8000 as u32,
    0x4000 as u32,
    0x2000 as u32,
    0x1000 as u32,
    0x800 as u32,
    0x400 as u32,
    0x200 as u32,
    0x100 as u32,
    0x80 as u32,
    0x40 as u32,
    0x20 as u32,
    0x10 as u32,
    0x8 as u32,
    0x4 as u32,
    0x2 as u32,
    0x1 as u32,
];
static pc1: [u8; 56] = [
    56 as u8,
    48 as u8,
    40 as u8,
    32 as u8,
    24 as u8,
    16 as u8,
    8 as u8,
    0 as u8,
    57 as u8,
    49 as u8,
    41 as u8,
    33 as u8,
    25 as u8,
    17 as u8,
    9 as u8,
    1 as u8,
    58 as u8,
    50 as u8,
    42 as u8,
    34 as u8,
    26 as u8,
    18 as u8,
    10 as u8,
    2 as u8,
    59 as u8,
    51 as u8,
    43 as u8,
    35 as u8,
    62 as u8,
    54 as u8,
    46 as u8,
    38 as u8,
    30 as u8,
    22 as u8,
    14 as u8,
    6 as u8,
    61 as u8,
    53 as u8,
    45 as u8,
    37 as u8,
    29 as u8,
    21 as u8,
    13 as u8,
    5 as u8,
    60 as u8,
    52 as u8,
    44 as u8,
    36 as u8,
    28 as u8,
    20 as u8,
    12 as u8,
    4 as u8,
    27 as u8,
    19 as u8,
    11 as u8,
    3 as u8,
];
static totrot: [u8; 16] = [
    1 as u8,
    2 as u8,
    4 as u8,
    6 as u8,
    8 as u8,
    10 as u8,
    12 as u8,
    14 as u8,
    15 as u8,
    17 as u8,
    19 as u8,
    21 as u8,
    23 as u8,
    25 as u8,
    27 as u8,
    28 as u8,
];
static pc2: [u8; 48] = [
    13 as u8,
    16 as u8,
    10 as u8,
    23 as u8,
    0 as u8,
    4 as u8,
    2 as u8,
    27 as u8,
    14 as u8,
    5 as u8,
    20 as u8,
    9 as u8,
    22 as u8,
    18 as u8,
    11 as u8,
    3 as u8,
    25 as u8,
    7 as u8,
    15 as u8,
    6 as u8,
    26 as u8,
    19 as u8,
    12 as u8,
    1 as u8,
    40 as u8,
    51 as u8,
    30 as u8,
    36 as u8,
    46 as u8,
    54 as u8,
    29 as u8,
    39 as u8,
    50 as u8,
    44 as u8,
    32 as u8,
    47 as u8,
    43 as u8,
    48 as u8,
    38 as u8,
    55 as u8,
    33 as u8,
    52 as u8,
    45 as u8,
    41 as u8,
    49 as u8,
    35 as u8,
    28 as u8,
    31 as u8,
];
static SP1: [ulong32; 64] = [
    0x1010400 as u32,
    0 as u32,
    0x10000 as u32,
    0x1010404 as u32,
    0x1010004 as u32,
    0x10404 as u32,
    0x4 as u32,
    0x10000 as u32,
    0x400 as u32,
    0x1010400 as u32,
    0x1010404 as u32,
    0x400 as u32,
    0x1000404 as u32,
    0x1010004 as u32,
    0x1000000 as u32,
    0x4 as u32,
    0x404 as u32,
    0x1000400 as u32,
    0x1000400 as u32,
    0x10400 as u32,
    0x10400 as u32,
    0x1010000 as u32,
    0x1010000 as u32,
    0x1000404 as u32,
    0x10004 as u32,
    0x1000004 as u32,
    0x1000004 as u32,
    0x10004 as u32,
    0 as u32,
    0x404 as u32,
    0x10404 as u32,
    0x1000000 as u32,
    0x10000 as u32,
    0x1010404 as u32,
    0x4 as u32,
    0x1010000 as u32,
    0x1010400 as u32,
    0x1000000 as u32,
    0x1000000 as u32,
    0x400 as u32,
    0x1010004 as u32,
    0x10000 as u32,
    0x10400 as u32,
    0x1000004 as u32,
    0x400 as u32,
    0x4 as u32,
    0x1000404 as u32,
    0x10404 as u32,
    0x1010404 as u32,
    0x10004 as u32,
    0x1010000 as u32,
    0x1000404 as u32,
    0x1000004 as u32,
    0x404 as u32,
    0x10404 as u32,
    0x1010400 as u32,
    0x404 as u32,
    0x1000400 as u32,
    0x1000400 as u32,
    0 as u32,
    0x10004 as u32,
    0x10400 as u32,
    0 as u32,
    0x1010004 as u32,
];
static SP2: [ulong32; 64] = [
    0x80108020 as u32,
    0x80008000 as u32,
    0x8000 as u32,
    0x108020 as u32,
    0x100000 as u32,
    0x20 as u32,
    0x80100020 as u32,
    0x80008020 as u32,
    0x80000020 as u32,
    0x80108020 as u32,
    0x80108000 as u32,
    0x80000000 as u32,
    0x80008000 as u32,
    0x100000 as u32,
    0x20 as u32,
    0x80100020 as u32,
    0x108000 as u32,
    0x100020 as u32,
    0x80008020 as u32,
    0 as u32,
    0x80000000 as u32,
    0x8000 as u32,
    0x108020 as u32,
    0x80100000 as u32,
    0x100020 as u32,
    0x80000020 as u32,
    0 as u32,
    0x108000 as u32,
    0x8020 as u32,
    0x80108000 as u32,
    0x80100000 as u32,
    0x8020 as u32,
    0 as u32,
    0x108020 as u32,
    0x80100020 as u32,
    0x100000 as u32,
    0x80008020 as u32,
    0x80100000 as u32,
    0x80108000 as u32,
    0x8000 as u32,
    0x80100000 as u32,
    0x80008000 as u32,
    0x20 as u32,
    0x80108020 as u32,
    0x108020 as u32,
    0x20 as u32,
    0x8000 as u32,
    0x80000000 as u32,
    0x8020 as u32,
    0x80108000 as u32,
    0x100000 as u32,
    0x80000020 as u32,
    0x100020 as u32,
    0x80008020 as u32,
    0x80000020 as u32,
    0x100020 as u32,
    0x108000 as u32,
    0 as u32,
    0x80008000 as u32,
    0x8020 as u32,
    0x80000000 as u32,
    0x80100020 as u32,
    0x80108020 as u32,
    0x108000 as u32,
];
static SP3: [ulong32; 64] = [
    0x208 as u32,
    0x8020200 as u32,
    0 as u32,
    0x8020008 as u32,
    0x8000200 as u32,
    0 as u32,
    0x20208 as u32,
    0x8000200 as u32,
    0x20008 as u32,
    0x8000008 as u32,
    0x8000008 as u32,
    0x20000 as u32,
    0x8020208 as u32,
    0x20008 as u32,
    0x8020000 as u32,
    0x208 as u32,
    0x8000000 as u32,
    0x8 as u32,
    0x8020200 as u32,
    0x200 as u32,
    0x20200 as u32,
    0x8020000 as u32,
    0x8020008 as u32,
    0x20208 as u32,
    0x8000208 as u32,
    0x20200 as u32,
    0x20000 as u32,
    0x8000208 as u32,
    0x8 as u32,
    0x8020208 as u32,
    0x200 as u32,
    0x8000000 as u32,
    0x8020200 as u32,
    0x8000000 as u32,
    0x20008 as u32,
    0x208 as u32,
    0x20000 as u32,
    0x8020200 as u32,
    0x8000200 as u32,
    0 as u32,
    0x200 as u32,
    0x20008 as u32,
    0x8020208 as u32,
    0x8000200 as u32,
    0x8000008 as u32,
    0x200 as u32,
    0 as u32,
    0x8020008 as u32,
    0x8000208 as u32,
    0x20000 as u32,
    0x8000000 as u32,
    0x8020208 as u32,
    0x8 as u32,
    0x20208 as u32,
    0x20200 as u32,
    0x8000008 as u32,
    0x8020000 as u32,
    0x8000208 as u32,
    0x208 as u32,
    0x8020000 as u32,
    0x20208 as u32,
    0x8 as u32,
    0x8020008 as u32,
    0x20200 as u32,
];
static SP4: [ulong32; 64] = [
    0x802001 as u32,
    0x2081 as u32,
    0x2081 as u32,
    0x80 as u32,
    0x802080 as u32,
    0x800081 as u32,
    0x800001 as u32,
    0x2001 as u32,
    0 as u32,
    0x802000 as u32,
    0x802000 as u32,
    0x802081 as u32,
    0x81 as u32,
    0 as u32,
    0x800080 as u32,
    0x800001 as u32,
    0x1 as u32,
    0x2000 as u32,
    0x800000 as u32,
    0x802001 as u32,
    0x80 as u32,
    0x800000 as u32,
    0x2001 as u32,
    0x2080 as u32,
    0x800081 as u32,
    0x1 as u32,
    0x2080 as u32,
    0x800080 as u32,
    0x2000 as u32,
    0x802080 as u32,
    0x802081 as u32,
    0x81 as u32,
    0x800080 as u32,
    0x800001 as u32,
    0x802000 as u32,
    0x802081 as u32,
    0x81 as u32,
    0 as u32,
    0 as u32,
    0x802000 as u32,
    0x2080 as u32,
    0x800080 as u32,
    0x800081 as u32,
    0x1 as u32,
    0x802001 as u32,
    0x2081 as u32,
    0x2081 as u32,
    0x80 as u32,
    0x802081 as u32,
    0x81 as u32,
    0x1 as u32,
    0x2000 as u32,
    0x800001 as u32,
    0x2001 as u32,
    0x802080 as u32,
    0x800081 as u32,
    0x2001 as u32,
    0x2080 as u32,
    0x800000 as u32,
    0x802001 as u32,
    0x80 as u32,
    0x800000 as u32,
    0x2000 as u32,
    0x802080 as u32,
];
static SP5: [ulong32; 64] = [
    0x100 as u32,
    0x2080100 as u32,
    0x2080000 as u32,
    0x42000100 as u32,
    0x80000 as u32,
    0x100 as u32,
    0x40000000 as u32,
    0x2080000 as u32,
    0x40080100 as u32,
    0x80000 as u32,
    0x2000100 as u32,
    0x40080100 as u32,
    0x42000100 as u32,
    0x42080000 as u32,
    0x80100 as u32,
    0x40000000 as u32,
    0x2000000 as u32,
    0x40080000 as u32,
    0x40080000 as u32,
    0 as u32,
    0x40000100 as u32,
    0x42080100 as u32,
    0x42080100 as u32,
    0x2000100 as u32,
    0x42080000 as u32,
    0x40000100 as u32,
    0 as u32,
    0x42000000 as u32,
    0x2080100 as u32,
    0x2000000 as u32,
    0x42000000 as u32,
    0x80100 as u32,
    0x80000 as u32,
    0x42000100 as u32,
    0x100 as u32,
    0x2000000 as u32,
    0x40000000 as u32,
    0x2080000 as u32,
    0x42000100 as u32,
    0x40080100 as u32,
    0x2000100 as u32,
    0x40000000 as u32,
    0x42080000 as u32,
    0x2080100 as u32,
    0x40080100 as u32,
    0x100 as u32,
    0x2000000 as u32,
    0x42080000 as u32,
    0x42080100 as u32,
    0x80100 as u32,
    0x42000000 as u32,
    0x42080100 as u32,
    0x2080000 as u32,
    0 as u32,
    0x40080000 as u32,
    0x42000000 as u32,
    0x80100 as u32,
    0x2000100 as u32,
    0x40000100 as u32,
    0x80000 as u32,
    0 as u32,
    0x40080000 as u32,
    0x2080100 as u32,
    0x40000100 as u32,
];
static SP6: [ulong32; 64] = [
    0x20000010 as u32,
    0x20400000 as u32,
    0x4000 as u32,
    0x20404010 as u32,
    0x20400000 as u32,
    0x10 as u32,
    0x20404010 as u32,
    0x400000 as u32,
    0x20004000 as u32,
    0x404010 as u32,
    0x400000 as u32,
    0x20000010 as u32,
    0x400010 as u32,
    0x20004000 as u32,
    0x20000000 as u32,
    0x4010 as u32,
    0 as u32,
    0x400010 as u32,
    0x20004010 as u32,
    0x4000 as u32,
    0x404000 as u32,
    0x20004010 as u32,
    0x10 as u32,
    0x20400010 as u32,
    0x20400010 as u32,
    0 as u32,
    0x404010 as u32,
    0x20404000 as u32,
    0x4010 as u32,
    0x404000 as u32,
    0x20404000 as u32,
    0x20000000 as u32,
    0x20004000 as u32,
    0x10 as u32,
    0x20400010 as u32,
    0x404000 as u32,
    0x20404010 as u32,
    0x400000 as u32,
    0x4010 as u32,
    0x20000010 as u32,
    0x400000 as u32,
    0x20004000 as u32,
    0x20000000 as u32,
    0x4010 as u32,
    0x20000010 as u32,
    0x20404010 as u32,
    0x404000 as u32,
    0x20400000 as u32,
    0x404010 as u32,
    0x20404000 as u32,
    0 as u32,
    0x20400010 as u32,
    0x10 as u32,
    0x4000 as u32,
    0x20400000 as u32,
    0x404010 as u32,
    0x4000 as u32,
    0x400010 as u32,
    0x20004010 as u32,
    0 as u32,
    0x20404000 as u32,
    0x20000000 as u32,
    0x400010 as u32,
    0x20004010 as u32,
];
static SP7: [ulong32; 64] = [
    0x200000 as u32,
    0x4200002 as u32,
    0x4000802 as u32,
    0 as u32,
    0x800 as u32,
    0x4000802 as u32,
    0x200802 as u32,
    0x4200800 as u32,
    0x4200802 as u32,
    0x200000 as u32,
    0 as u32,
    0x4000002 as u32,
    0x2 as u32,
    0x4000000 as u32,
    0x4200002 as u32,
    0x802 as u32,
    0x4000800 as u32,
    0x200802 as u32,
    0x200002 as u32,
    0x4000800 as u32,
    0x4000002 as u32,
    0x4200000 as u32,
    0x4200800 as u32,
    0x200002 as u32,
    0x4200000 as u32,
    0x800 as u32,
    0x802 as u32,
    0x4200802 as u32,
    0x200800 as u32,
    0x2 as u32,
    0x4000000 as u32,
    0x200800 as u32,
    0x4000000 as u32,
    0x200800 as u32,
    0x200000 as u32,
    0x4000802 as u32,
    0x4000802 as u32,
    0x4200002 as u32,
    0x4200002 as u32,
    0x2 as u32,
    0x200002 as u32,
    0x4000000 as u32,
    0x4000800 as u32,
    0x200000 as u32,
    0x4200800 as u32,
    0x802 as u32,
    0x200802 as u32,
    0x4200800 as u32,
    0x802 as u32,
    0x4000002 as u32,
    0x4200802 as u32,
    0x4200000 as u32,
    0x200800 as u32,
    0 as u32,
    0x2 as u32,
    0x4200802 as u32,
    0 as u32,
    0x200802 as u32,
    0x4200000 as u32,
    0x800 as u32,
    0x4000002 as u32,
    0x4000800 as u32,
    0x800 as u32,
    0x200002 as u32,
];
static SP8: [ulong32; 64] = [
    0x10001040 as u32,
    0x1000 as u32,
    0x40000 as u32,
    0x10041040 as u32,
    0x10000000 as u32,
    0x10001040 as u32,
    0x40 as u32,
    0x10000000 as u32,
    0x40040 as u32,
    0x10040000 as u32,
    0x10041040 as u32,
    0x41000 as u32,
    0x10041000 as u32,
    0x41040 as u32,
    0x1000 as u32,
    0x40 as u32,
    0x10040000 as u32,
    0x10000040 as u32,
    0x10001000 as u32,
    0x1040 as u32,
    0x41000 as u32,
    0x40040 as u32,
    0x10040040 as u32,
    0x10041000 as u32,
    0x1040 as u32,
    0 as u32,
    0 as u32,
    0x10040040 as u32,
    0x10000040 as u32,
    0x10001000 as u32,
    0x41040 as u32,
    0x40000 as u32,
    0x41040 as u32,
    0x40000 as u32,
    0x10041000 as u32,
    0x1000 as u32,
    0x40 as u32,
    0x10040040 as u32,
    0x1000 as u32,
    0x41040 as u32,
    0x10001000 as u32,
    0x40 as u32,
    0x10000040 as u32,
    0x10040000 as u32,
    0x10040040 as u32,
    0x10000000 as u32,
    0x40000 as u32,
    0x10001040 as u32,
    0 as u32,
    0x10041040 as u32,
    0x40040 as u32,
    0x10000040 as u32,
    0x10040000 as u32,
    0x10001000 as u32,
    0x10001040 as u32,
    0 as u32,
    0x10041040 as u32,
    0x41000 as u32,
    0x41000 as u32,
    0x1040 as u32,
    0x1040 as u32,
    0x40040 as u32,
    0x10000000 as u32,
    0x10041000 as u32,
];
static des_ip: [[ulong64; 256]; 8] = [
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x1010 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x100010 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x101010 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10000010 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10001010 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10100010 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10101010 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x808 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x80008 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x80808 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8000008 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8000808 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8080008 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8080808 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x404 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x40004 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x40404 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4000004 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4000404 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4040004 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4040404 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x202 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x20002 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x20202 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2000002 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2000202 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2020002 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2020202 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x101 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x10001 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x10101 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1000001 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1000101 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1010001 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1010101 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x8080 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x800080 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x808080 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80000080 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80008080 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80800080 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80808080 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x4040 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x400040 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x404040 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40000040 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40004040 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40400040 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40404040 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x2020 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x200020 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x202020 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20000020 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20002020 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20200020 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20202020 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
    ],
];
static des_fp: [[ulong64; 256]; 8] = [
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x80 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2000000 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2000080 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x20000 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x20080 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2020000 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2020080 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x200 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x280 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2000200 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2000280 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x20200 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x20280 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x2000000 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x20000 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x2020000 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x200 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x2000200 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x20200 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x2020200 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x2 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x2000002 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x20002 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x2020002 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x202 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x2000202 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x20202 as u64,
        (0x2020200 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
        (0x2020280 as i32 as u64) << 32 as i32
            | 0x2020202 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x2 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8000000 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8000002 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x80000 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x80002 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8080000 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8080002 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x800 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x802 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8000800 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8000802 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x80800 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x80802 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x8000000 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x80000 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x8080000 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x800 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x8000800 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x80800 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x8080800 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x8 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x8000008 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x80008 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x8080008 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x808 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x8000808 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x80808 as u64,
        (0x8080800 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
        (0x8080802 as i32 as u64) << 32 as i32
            | 0x8080808 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x8 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20000000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20000008 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x200000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x200008 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20200000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20200008 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x2000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x2008 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20002000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20002008 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x202000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x202008 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x20000000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x200000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x20200000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x2000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x20002000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x202000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x20202000 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x20 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x20000020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x200020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x20200020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x2020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x20002020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x202020 as u64,
        (0x20202000 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
        (0x20202008 as i32 as u64) << 32 as i32
            | 0x20202020 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x20 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80000000 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80000020 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x800000 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x800020 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80800000 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80800020 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x8000 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x8020 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80008000 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80008020 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x808000 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x808020 as i32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x80000000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x800000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x80800000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x8000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x80008000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x808000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x80808000 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x80 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x80000080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x800080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x80800080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x8080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x80008080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x808080 as u64,
        (0x80808000 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
        (0x80808020 as u32 as u64) << 32 as i32
            | 0x80808080 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x40 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1000000 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1000040 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x10000 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x10040 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1010000 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1010040 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x100 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x140 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1000100 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1000140 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x10100 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x10140 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x1000000 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x10000 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x1010000 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x100 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x1000100 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x10100 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x1010100 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x1 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x1000001 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x10001 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x1010001 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x101 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x1000101 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x10101 as u64,
        (0x1010100 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
        (0x1010140 as i32 as u64) << 32 as i32
            | 0x1010101 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x1 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4000000 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4000001 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x40000 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x40001 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4040000 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4040001 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x400 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x401 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4000400 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4000401 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x40400 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x40401 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x4000000 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x40000 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x4040000 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x400 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x4000400 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x40400 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x4040400 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x4 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x4000004 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x40004 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x4040004 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x404 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x4000404 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x40404 as u64,
        (0x4040400 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
        (0x4040401 as i32 as u64) << 32 as i32
            | 0x4040404 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x4 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10000000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10000004 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x100000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x100004 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10100000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10100004 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x1000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x1004 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10001000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10001004 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x101000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x101004 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x10000000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x100000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x10100000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x1000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x10001000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x101000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x10101000 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x10 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x10000010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x100010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x10100010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x1010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x10001010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x101010 as u64,
        (0x10101000 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
        (0x10101004 as i32 as u64) << 32 as i32
            | 0x10101010 as u64,
    ],
    [
        (0 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x10 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40000000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40000010 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x400000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x400010 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40400000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40400010 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x4000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x4010 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40004000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40004010 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x404000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x404010 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x40000000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x400000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x40400000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x4000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x40004000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x404000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x40404000 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x40 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x40000040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x400040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x40400040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x4040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x40004040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x404040 as u64,
        (0x40404000 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
        (0x40404010 as i32 as u64) << 32 as i32
            | 0x40404040 as u64,
    ],
];


unsafe fn deskey(
    key: *const u8,
    edf: i16,
    keyout: *mut ulong32,
) {
    let mut i: ulong32;
    let mut j: ulong32;
    let mut l: ulong32;
    let mut m: ulong32;
    let mut n: ulong32;
    let mut kn: [ulong32; 32] = [0; 32];
    let mut pc1m: [u8; 56] = [0; 56];
    let mut pcr: [u8; 56] = [0; 56];

    j = 0;
    while j < 56 {
        l = pc1[j as usize] as ulong32;
        m = l & 7;
        pc1m[j as usize] = if *key.offset((l >> 3) as isize) as ulong32 & bytebit[m as usize]
            == bytebit[m as usize]
        {
            1
        } else {
            0
        } as u8;
        j = j.wrapping_add(1);
    }

    i = 0;
    while i < 16 {
        if edf as i32 == DE1 {
            m = (15u32).wrapping_sub(i) << 1;
        } else {
            m = i << 1;
        }
        n = m.wrapping_add(1);
        kn[n as usize] = 0;
        kn[m as usize] = kn[n as usize];

        j = 0;
        while j < 28 {
            l = j.wrapping_add(totrot[i as usize] as ulong32);
            if l < 28 {
                pcr[j as usize] = pc1m[l as usize];
            } else {
                pcr[j as usize] = pc1m[l.wrapping_sub(28) as usize];
            }
            j = j.wrapping_add(1);
        }
        while j < 56 {
            l = j.wrapping_add(totrot[i as usize] as ulong32);
            if l < 56 {
                pcr[j as usize] = pc1m[l as usize];
            } else {
                pcr[j as usize] = pc1m[l.wrapping_sub(28) as usize];
            }
            j = j.wrapping_add(1);
        }

        j = 0;
        while j < 24 {
            if pcr[pc2[j as usize] as i32 as usize] as i32 != 0 {
                kn[m as usize] |= bigbyte[j as usize];
            }
            if pcr[pc2[(j.wrapping_add(24)) as usize] as i32 as usize] as i32 != 0 {
                kn[n as usize] |= bigbyte[j as usize];
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    cookey(kn.as_mut_ptr(), keyout);
}

unsafe fn cookey(mut raw1: *const u32, keyout: *mut u32) {
    let mut dough: [u32; 32] = [0; 32];
    let mut cook = core::ptr::addr_of_mut!(dough) as *mut u32;
    let mut raw0: *const u32;
    let mut i: i32 = 0;

    while i < 16 {
        raw0 = raw1;
        raw1 = raw1.offset(1);

        *cook = (*raw0 & 0x00FC_0000) << 6;
        *cook |= (*raw0 & 0x0000_0FC0) << 10;
        *cook |= (*raw1 & 0x00FC_0000) >> 10;
        {
            let tmp = cook;
            cook = cook.offset(1);
            *tmp |= (*raw1 & 0x0000_0FC0) >> 6;
        }

        *cook = (*raw0 & 0x0003_F000) << 12;
        *cook |= (*raw0 & 0x0000_003F) << 16;
        *cook |= (*raw1 & 0x0003_F000) >> 4;
        {
            let tmp = cook;
            cook = cook.offset(1);
            *tmp |= *raw1 & 0x0000_003F;
        }

        i += 1;
        raw1 = raw1.offset(1);
    }

    core::ptr::copy_nonoverlapping(dough.as_ptr(), keyout, 32);
}

unsafe fn desfunc(block: *mut u32, keys: *const u32) {
    let mut tmp: u64;
    let mut work: u32;
    let mut right: u32 = unsafe { *block.offset(1) };
    let mut leftt: u32 = unsafe { *block.offset(0) };
    let mut keys = keys;

    tmp = des_ip[0][byte(leftt, 0) as usize] ^
          des_ip[1][byte(leftt, 1) as usize] ^
          des_ip[2][byte(leftt, 2) as usize] ^
          des_ip[3][byte(leftt, 3) as usize] ^
          des_ip[4][byte(right, 0) as usize] ^
          des_ip[5][byte(right, 1) as usize] ^
          des_ip[6][byte(right, 2) as usize] ^
          des_ip[7][byte(right, 3) as usize];
    leftt = (tmp >> 32) as u32;
    right = (tmp & 0xFFFFFFFFu64) as u32;

    for cur_round in 0i32..8 {
        work = ROR(right, 4) ^ unsafe { *keys };
        keys = keys.add(1);
        leftt ^= SP7[(work & 0x3f) as usize]
               ^ SP5[((work >> 8) & 0x3f) as usize]
               ^ SP3[((work >> 16) & 0x3f) as usize]
               ^ SP1[((work >> 24) & 0x3f) as usize];

        work = right ^ unsafe { *keys };
        keys = keys.add(1);
        leftt ^= SP8[(work & 0x3f) as usize]
               ^ SP6[((work >> 8) & 0x3f) as usize]
               ^ SP4[((work >> 16) & 0x3f) as usize]
               ^ SP2[((work >> 24) & 0x3f) as usize];

        work = ROR(leftt, 4) ^ unsafe { *keys };
        keys = keys.add(1);
        right ^= SP7[(work & 0x3f) as usize]
               ^ SP5[((work >> 8) & 0x3f) as usize]
               ^ SP3[((work >> 16) & 0x3f) as usize]
               ^ SP1[((work >> 24) & 0x3f) as usize];

        work = leftt ^ unsafe { *keys };
        keys = keys.add(1);
        right ^= SP8[(work & 0x3f) as usize]
               ^ SP6[((work >> 8) & 0x3f) as usize]
               ^ SP4[((work >> 16) & 0x3f) as usize]
               ^ SP2[((work >> 24) & 0x3f) as usize];
    }

    tmp = des_fp[0][byte(leftt, 0) as usize] ^
          des_fp[1][byte(leftt, 1) as usize] ^
          des_fp[2][byte(leftt, 2) as usize] ^
          des_fp[3][byte(leftt, 3) as usize] ^
          des_fp[4][byte(right, 0) as usize] ^
          des_fp[5][byte(right, 1) as usize] ^
          des_fp[6][byte(right, 2) as usize] ^
          des_fp[7][byte(right, 3) as usize];
    leftt = (tmp >> 32) as u32;
    right = (tmp & 0xFFFFFFFFu64) as u32;

    unsafe {
        *block.offset(0) = right;
        *block.offset(1) = leftt;
    }
}
#[no_mangle]
pub unsafe extern "C" fn des_setup(
    key: *const ::core::ffi::c_uchar,
    keylen: ::core::ffi::c_int,
    num_rounds: ::core::ffi::c_int,
    des: *mut des_key,
) -> ::core::ffi::c_int {
    if num_rounds != 0 && num_rounds != 16 {
        return CRYPT_INVALID_ROUNDS;
    }
    if keylen != 8 {
        return CRYPT_INVALID_KEYSIZE;
    }
    deskey(key, EN0 as i16, &raw mut (*des).ek as *mut ulong32);
    deskey(key, DE1 as i16, &raw mut (*des).dk as *mut ulong32);
    return CRYPT_OK;
}

#[no_mangle]
pub unsafe extern "C" fn des3_setup(
    key: *const u8,
    keylen: i32,
    num_rounds: i32,
    des3: *mut des3_key,
) -> i32 {
    if num_rounds != 0 && num_rounds != 16 {
        return 9; /* CRYPT_INVALID_ROUNDS */
    }
    if keylen != 24 {
        return 4; /* CRYPT_INVALID_KEYSIZE */
    }
    deskey(key, 0i16, core::ptr::addr_of_mut!((*des3).ek[0]) as *mut u32); /* EN0 */
    deskey(key.add(8), 1i16, core::ptr::addr_of_mut!((*des3).ek[1]) as *mut u32); /* DE1 */
    deskey(key.add(16), 0i16, core::ptr::addr_of_mut!((*des3).ek[2]) as *mut u32); /* EN0 */
    deskey(key, 1i16, core::ptr::addr_of_mut!((*des3).dk[2]) as *mut u32); /* DE1 */
    deskey(key.add(8), 0i16, core::ptr::addr_of_mut!((*des3).dk[1]) as *mut u32); /* EN0 */
    deskey(key.add(16), 1i16, core::ptr::addr_of_mut!((*des3).dk[0]) as *mut u32); /* DE1 */
    return 0; /* CRYPT_OK */
}
#[no_mangle]
pub unsafe extern "C" fn des_ecb_encrypt(
    pt: *const u8,
    ct: *mut u8,
    des: *const des_key,
) {
    let mut work: [u32; 2] = [0; 2];

    work[0] = ((*pt.add(0) as u32) << 24)
        | ((*pt.add(1) as u32) << 16)
        | ((*pt.add(2) as u32) << 8)
        | (*pt.add(3) as u32);

    work[1] = ((*pt.add(4) as u32) << 24)
        | ((*pt.add(5) as u32) << 16)
        | ((*pt.add(6) as u32) << 8)
        | (*pt.add(7) as u32);

    desfunc(
        core::ptr::addr_of_mut!(work) as *mut u32,
        core::ptr::addr_of!((*des).ek) as *const u32,
    );

    *ct.add(0) = (work[0] >> 24) as u8;
    *ct.add(1) = (work[0] >> 16) as u8;
    *ct.add(2) = (work[0] >> 8) as u8;
    *ct.add(3) = work[0] as u8;

    *ct.add(4) = (work[1] >> 24) as u8;
    *ct.add(5) = (work[1] >> 16) as u8;
    *ct.add(6) = (work[1] >> 8) as u8;
    *ct.add(7) = work[1] as u8;
}
#[no_mangle]
pub unsafe extern "C" fn des_ecb_decrypt(ct: *const u8, pt: *mut u8, des: *const des_key) {
    let mut work: [u32; 2] = [0u32; 2];
    work[0] = ((*ct as u32 & 0xFF) << 24)
        | ((*ct.add(1) as u32 & 0xFF) << 16)
        | ((*ct.add(2) as u32 & 0xFF) << 8)
        | (*ct.add(3) as u32 & 0xFF);
    work[1] = ((*ct.add(4) as u32 & 0xFF) << 24)
        | ((*ct.add(5) as u32 & 0xFF) << 16)
        | ((*ct.add(6) as u32 & 0xFF) << 8)
        | (*ct.add(7) as u32 & 0xFF);
    desfunc(
        core::ptr::addr_of_mut!(work) as *mut u32,
        core::ptr::addr_of!((*des).dk) as *const u32,
    );
    *pt.add(0) = ((work[0] >> 24) & 0xFF) as u8;
    *pt.add(1) = ((work[0] >> 16) & 0xFF) as u8;
    *pt.add(2) = ((work[0] >> 8) & 0xFF) as u8;
    *pt.add(3) = (work[0] & 0xFF) as u8;
    *pt.add(4) = ((work[1] >> 24) & 0xFF) as u8;
    *pt.add(5) = ((work[1] >> 16) & 0xFF) as u8;
    *pt.add(6) = ((work[1] >> 8) & 0xFF) as u8;
    *pt.add(7) = (work[1] & 0xFF) as u8;
}

#[no_mangle]
pub unsafe extern "C" fn des3_ecb_encrypt(
    pt: *const u8,
    ct: *mut u8,
    des3: *const des3_key,
) {
    let mut work: [u32; 2] = [0; 2];
    work[0] = (((*pt.add(0) as i32 & 255) as u64) << 24
        | ((*pt.add(1) as i32 & 255) as u64) << 16
        | ((*pt.add(2) as i32 & 255) as u64) << 8
        | (*pt.add(3) as i32 & 255) as u64) as u32;
    work[1] = (((*pt.add(4) as i32 & 255) as u64) << 24
        | ((*pt.add(5) as i32 & 255) as u64) << 16
        | ((*pt.add(6) as i32 & 255) as u64) << 8
        | (*pt.add(7) as i32 & 255) as u64) as u32;
    desfunc(
        &raw mut work as *mut u32,
        &raw const (*des3).ek[0] as *const u32,
    );
    desfunc(
        &raw mut work as *mut u32,
        &raw const (*des3).ek[1] as *const u32,
    );
    desfunc(
        &raw mut work as *mut u32,
        &raw const (*des3).ek[2] as *const u32,
    );
    *ct.add(0) = (work[0] >> 24 & 255) as u8;
    *ct.add(1) = (work[0] >> 16 & 255) as u8;
    *ct.add(2) = (work[0] >> 8 & 255) as u8;
    *ct.add(3) = (work[0] & 255) as u8;
    *ct.add(4) = (work[1] >> 24 & 255) as u8;
    *ct.add(5) = (work[1] >> 16 & 255) as u8;
    *ct.add(6) = (work[1] >> 8 & 255) as u8;
    *ct.add(7) = (work[1] & 255) as u8;
}

#[no_mangle]
pub unsafe extern "C" fn des3_ecb_decrypt(
    ct: *const u8,
    pt: *mut u8,
    des3: *const des3_key,
) {
    let mut work: [u32; 2] = [0u32; 2];
    work[0] = ((*ct.add(0) as u32) << 24)
        | ((*ct.add(1) as u32) << 16)
        | ((*ct.add(2) as u32) << 8)
        | (*ct.add(3) as u32);
    work[1] = ((*ct.add(4) as u32) << 24)
        | ((*ct.add(5) as u32) << 16)
        | ((*ct.add(6) as u32) << 8)
        | (*ct.add(7) as u32);
    let dk_base = core::ptr::addr_of!((*des3).dk) as *const [u32; 32];
    desfunc(work.as_mut_ptr(), dk_base.add(0) as *const u32);
    desfunc(work.as_mut_ptr(), dk_base.add(1) as *const u32);
    desfunc(work.as_mut_ptr(), dk_base.add(2) as *const u32);
    *pt.add(0) = (work[0] >> 24) as u8;
    *pt.add(1) = (work[0] >> 16) as u8;
    *pt.add(2) = (work[0] >> 8) as u8;
    *pt.add(3) = work[0] as u8;
    *pt.add(4) = (work[1] >> 24) as u8;
    *pt.add(5) = (work[1] >> 16) as u8;
    *pt.add(6) = (work[1] >> 8) as u8;
    *pt.add(7) = work[1] as u8;
}