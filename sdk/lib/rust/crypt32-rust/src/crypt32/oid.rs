use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::main::CryptMemRealloc;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_4928BE03934B: [u16; 6] = [0x0046u16, 0x006C, 0x0061, 0x0067, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_52A123613C9E: [u16; 10] = [0x0045u16, 0x0078, 0x0074, 0x0072, 0x0061, 0x0049, 0x006E, 0x0066, 0x006F, 0x0000];
pub static RUSTCONV_WSTR_541A926BC431: [u16; 14] = [
    0x0043u16, 0x004E, 0x0047, 0x0045, 0x0078, 0x0074, 0x0072, 0x0061, 0x0041, 0x006C,
    0x0067, 0x0069, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_6184A7B89A8A: [u16; 4] = [0x0044u16, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_6200FA5BD108: [u16; 71] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0043,
    0x0072, 0x0079, 0x0070, 0x0074, 0x006F, 0x0067, 0x0072, 0x0061, 0x0070, 0x0068,
    0x0079, 0x005C, 0x004F, 0x0049, 0x0044, 0x005C, 0x0045, 0x006E, 0x0063, 0x006F,
    0x0064, 0x0069, 0x006E, 0x0067, 0x0054, 0x0079, 0x0070, 0x0065, 0x0020, 0x0030,
    0x005C, 0x0043, 0x0072, 0x0079, 0x0070, 0x0074, 0x0044, 0x006C, 0x006C, 0x0046,
    0x0069, 0x006E, 0x0064, 0x004F, 0x0049, 0x0044, 0x0049, 0x006E, 0x0066, 0x006F,
    0x0000,
];
pub static RUSTCONV_WSTR_6F50C773FDBD: [u16; 6] = [0x0041u16, 0x006C, 0x0067, 0x0069, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_76E480513E2C: [u16; 5] = [0x004Eu16, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_8AFF6EE2D972: [u16; 9] = [0x0043u16, 0x004E, 0x0047, 0x0041, 0x006C, 0x0067, 0x0069, 0x0064, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_01FE3A650989: [u16; 5] = [0x0045u16, 0x0053, 0x0044, 0x0048, 0x0000];
pub static RUSTCONV_C2R_WSTR_0579D3D5469E: [u16; 3] = [0x0047u16, 0x004E, 0x0000];
pub static RUSTCONV_C2R_WSTR_0AB677189BBD: [u16; 2] = [0x0047u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_0B4A96579C77: [u16; 7] = [0x006Du16, 0x0064, 0x0034, 0x0052, 0x0053, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_0C38FC8A9E0F: [u16; 4] = [0x0052u16, 0x0053, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_177EF5904691: [u16; 17] = [
    0x006Du16, 0x006F, 0x0073, 0x0061, 0x0069, 0x0063, 0x0055, 0x0070, 0x0064, 0x0061,
    0x0074, 0x0065, 0x0064, 0x0053, 0x0069, 0x0067, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_17B90C614D61: [u16; 7] = [0x0073u16, 0x0068, 0x0061, 0x0032, 0x0035, 0x0036, 0x0000];
pub static RUSTCONV_C2R_WSTR_1FFF2DC848DB: [u16; 7] = [0x0073u16, 0x0068, 0x0061, 0x0052, 0x0053, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_2129F45562B7: [u16; 18] = [
    0x006Du16, 0x006F, 0x0073, 0x0061, 0x0069, 0x0063, 0x004B, 0x004D, 0x0061, 0x006E,
    0x0064, 0x0055, 0x0070, 0x0064, 0x0053, 0x0069, 0x0067, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_219032C80178: [u16; 17] = [
    0x0054u16, 0x0072, 0x0075, 0x0073, 0x0074, 0x0065, 0x0064, 0x0050, 0x0075, 0x0062,
    0x006C, 0x0069, 0x0073, 0x0068, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_25546986ABCE: [u16; 3] = [0x004Fu16, 0x0055, 0x0000];
pub static RUSTCONV_C2R_WSTR_28F6ADD86F3C: [u16; 5] = [0x0033u16, 0x0064, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_2D3BB1F8582B: [u16; 16] = [
    0x0053u16, 0x0070, 0x0063, 0x0053, 0x0070, 0x0041, 0x0067, 0x0065, 0x006E, 0x0063,
    0x0079, 0x0049, 0x006E, 0x0066, 0x006F, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_2FD0C55DD168: [u16; 9] = [0x0052u16, 0x0053, 0x0041, 0x005F, 0x0053, 0x0049, 0x0047, 0x004E, 0x0000];
pub static RUSTCONV_C2R_WSTR_30CE221CF490: [u16; 8] = [0x0073u16, 0x0068, 0x0061, 0x0031, 0x0052, 0x0053, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_33149FF2DC42: [u16; 2] = [0x004Cu16, 0x0000];
pub static RUSTCONV_C2R_WSTR_3422FF7B7FF7: [u16; 19] = [
    0x0053u16, 0x0070, 0x0063, 0x004D, 0x0069, 0x006E, 0x0069, 0x006D, 0x0061, 0x006C,
    0x0043, 0x0072, 0x0069, 0x0074, 0x0065, 0x0072, 0x0069, 0x0061, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_35B543933849: [u16; 11] = [
    0x0050u16, 0x006F, 0x0073, 0x0074, 0x0061, 0x006C, 0x0043, 0x006F, 0x0064, 0x0065,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_3A4C0616016C: [u16; 9] = [0x0052u16, 0x0053, 0x0041, 0x005F, 0x004B, 0x0045, 0x0059, 0x0058, 0x0000];
pub static RUSTCONV_C2R_WSTR_3BAC45FDAD1C: [u16; 6] = [0x0054u16, 0x0069, 0x0074, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_3F9B2FCDB174: [u16; 12] = [
    0x0041u16, 0x0044, 0x0044, 0x0052, 0x0045, 0x0053, 0x0053, 0x0042, 0x004F, 0x004F,
    0x004B, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_3FBF037CB6BC: [u16; 2] = [0x0054u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_43FF56D4E41E: [u16; 11] = [
    0x0043u16, 0x004D, 0x0053, 0x0052, 0x0043, 0x0032, 0x0077, 0x0072, 0x0061, 0x0070,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_4664D9C9B4B4: [u16; 3] = [0x0053u16, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_4979BF393372: [u16; 4] = [0x006Du16, 0x0064, 0x0032, 0x0000];
pub static RUSTCONV_C2R_WSTR_4A7FEDDCAC3E: [u16; 3] = [0x0043u16, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_4FD33DCDE470: [u16; 2] = [0x0045u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_53A00CD4BB7F: [u16; 9] = [0x0044u16, 0x0053, 0x0041, 0x005F, 0x0053, 0x0049, 0x0047, 0x004E, 0x0000];
pub static RUSTCONV_C2R_WSTR_5BB10A0EA4AA: [u16; 2] = [0x0053u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_657E199B2FCE: [u16; 10] = [0x0073u16, 0x0068, 0x0061, 0x0035, 0x0031, 0x0032, 0x0052, 0x0053, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_699F058FE8D8: [u16; 8] = [0x0064u16, 0x0073, 0x0061, 0x0053, 0x0048, 0x0041, 0x0031, 0x0000];
pub static RUSTCONV_C2R_WSTR_6AA387BF4CB9: [u16; 3] = [0x0044u16, 0x0048, 0x0000];
pub static RUSTCONV_C2R_WSTR_6AC123285056: [u16; 12] = [
    0x0044u16, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_6CB5A8EC7215: [u16; 2] = [0x0049u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_6E9668E556EE: [u16; 7] = [0x0073u16, 0x0068, 0x0061, 0x0044, 0x0053, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_700AFFCD0C9F: [u16; 4] = [0x006Du16, 0x0064, 0x0035, 0x0000];
pub static RUSTCONV_C2R_WSTR_70A60BDF9EBF: [u16; 4] = [0x0072u16, 0x0063, 0x0034, 0x0000];
pub static RUSTCONV_C2R_WSTR_7B8E1FF9CAD5: [u16; 8] = [0x0073u16, 0x0068, 0x0061, 0x0031, 0x0044, 0x0053, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_7CE7C36007E8: [u16; 10] = [0x0073u16, 0x0068, 0x0061, 0x0033, 0x0038, 0x0034, 0x0052, 0x0053, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_832ABBDE2DC9: [u16; 7] = [0x004Eu16, 0x004F, 0x0053, 0x0049, 0x0047, 0x004E, 0x0000];
pub static RUSTCONV_C2R_WSTR_8693927E0562: [u16; 9] = [0x0049u16, 0x006E, 0x0069, 0x0074, 0x0069, 0x0061, 0x006C, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_86E423968AAB: [u16; 21] = [
    0x0053u16, 0x0070, 0x0063, 0x0046, 0x0069, 0x006E, 0x0061, 0x006E, 0x0063, 0x0069,
    0x0061, 0x006C, 0x0043, 0x0072, 0x0069, 0x0074, 0x0065, 0x0072, 0x0069, 0x0061,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_897FF00D4F10: [u16; 4] = [0x0044u16, 0x0053, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_90953F2876F4: [u16; 5] = [0x0052u16, 0x004F, 0x004F, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_937515BA47B8: [u16; 7] = [0x0053u16, 0x0054, 0x0052, 0x0045, 0x0045, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_C2R_WSTR_986E97A25E53: [u16; 7] = [0x0073u16, 0x0068, 0x0061, 0x0033, 0x0038, 0x0034, 0x0000];
pub static RUSTCONV_C2R_WSTR_9AAA20F77D73: [u16; 10] = [0x0073u16, 0x0068, 0x0061, 0x0032, 0x0035, 0x0036, 0x0052, 0x0053, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_9ECA39E104E6: [u16; 11] = [
    0x0058u16, 0x0032, 0x0031, 0x0041, 0x0064, 0x0064, 0x0072, 0x0065, 0x0073, 0x0073,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_A875FC2B07CD: [u16; 5] = [0x0073u16, 0x0068, 0x0061, 0x0031, 0x0000];
pub static RUSTCONV_C2R_WSTR_A97874299B48: [u16; 7] = [0x006Du16, 0x0064, 0x0032, 0x0052, 0x0053, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_B1864A7F14E8: [u16; 12] = [
    0x0073u16, 0x0068, 0x0061, 0x0032, 0x0035, 0x0036, 0x0045, 0x0043, 0x0044, 0x0053,
    0x0041, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_B89AF30006C8: [u16; 11] = [
    0x0044u16, 0x0069, 0x0073, 0x0061, 0x006C, 0x006C, 0x006F, 0x0077, 0x0065, 0x0064,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_BBD9F07EA39C: [u16; 3] = [0x0053u16, 0x004E, 0x0000];
pub static RUSTCONV_C2R_WSTR_BCE20B4ED641: [u16; 12] = [
    0x0073u16, 0x0068, 0x0061, 0x0033, 0x0038, 0x0034, 0x0045, 0x0043, 0x0044, 0x0053,
    0x0041, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_C0390936EE95: [u16; 7] = [0x006Du16, 0x0064, 0x0035, 0x0052, 0x0053, 0x0041, 0x0000];
pub static RUSTCONV_C2R_WSTR_C13B14C662E4: [u16; 4] = [0x0064u16, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_C1612A44FB42: [u16; 6] = [0x0050u16, 0x0068, 0x006F, 0x006E, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_C6488821BD31: [u16; 4] = [0x0044u16, 0x0053, 0x0053, 0x0000];
pub static RUSTCONV_C2R_WSTR_CE12E1294E6F: [u16; 6] = [0x0045u16, 0x006D, 0x0061, 0x0069, 0x006C, 0x0000];
pub static RUSTCONV_C2R_WSTR_CF4658672D80: [u16; 3] = [0x0044u16, 0x0043, 0x0000];
pub static RUSTCONV_C2R_WSTR_CFF611E6FECB: [u16; 2] = [0x004Fu16, 0x0000];
pub static RUSTCONV_C2R_WSTR_D23F96D836EB: [u16; 2] = [0x0043u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_D26390A5FDEE: [u16; 6] = [0x0050u16, 0x004F, 0x0042, 0x006F, 0x0078, 0x0000];
pub static RUSTCONV_C2R_WSTR_D2D100D7E39F: [u16; 4] = [0x0045u16, 0x0043, 0x0043, 0x0000];
pub static RUSTCONV_C2R_WSTR_D38D07CD8B99: [u16; 12] = [
    0x0043u16, 0x004D, 0x0053, 0x0033, 0x0044, 0x0045, 0x0053, 0x0077, 0x0072, 0x0061,
    0x0070, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_D90081DF0539: [u16; 3] = [0x004Du16, 0x0059, 0x0000];
pub static RUSTCONV_C2R_WSTR_E465C4D6880D: [u16; 4] = [0x0073u16, 0x0068, 0x0061, 0x0000];
pub static RUSTCONV_C2R_WSTR_E759B459F85B: [u16; 7] = [0x0073u16, 0x0068, 0x0061, 0x0035, 0x0031, 0x0032, 0x0000];
pub static RUSTCONV_C2R_WSTR_F03765AE14C6: [u16; 4] = [0x0072u16, 0x0063, 0x0032, 0x0000];
pub static RUSTCONV_C2R_WSTR_F097069263D2: [u16; 12] = [
    0x0064u16, 0x006E, 0x0051, 0x0075, 0x0061, 0x006C, 0x0069, 0x0066, 0x0069, 0x0065,
    0x0072, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_F5D02491CB00: [u16; 10] = [0x0047u16, 0x0069, 0x0076, 0x0065, 0x006E, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_F93A8BF12ADB: [u16; 4] = [0x006Du16, 0x0064, 0x0034, 0x0000];
pub static RUSTCONV_C2R_WSTR_FB67DA7C36CC: [u16; 13] = [
    0x0053u16, 0x0045, 0x0052, 0x0049, 0x0041, 0x004C, 0x004E, 0x0055, 0x004D, 0x0042,
    0x0045, 0x0052, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_FBCEAF5CF84D: [u16; 3] = [0x0043u16, 0x004E, 0x0000];

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
    pub Flags: DWORD,
    pub CreatorBackTraceIndexHigh: WORD,
    pub SpareWORD: WORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_10 {
    pub WineDebugString: DWORD_PTR,
    pub Spare: [DWORD_PTR; 1],
    pub c2rust_unnamed: C2RustUnnamed_11,
}

pub static BCRYPT_SHA256_ALGORITHM: [::core::ffi::c_ushort; 7] = unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_ushort; 7]>(*b"S\0H\0A\x002\x005\x006\0\0\0")
};

pub static BCRYPT_SHA384_ALGORITHM: [::core::ffi::c_ushort; 7] = unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_ushort; 7]>(*b"S\0H\0A\x003\08\x004\0\0\0")
};

pub static CRYPT_OID_INFO_ECC_PARAMETERS_ALGORITHM: [::core::ffi::c_ushort; 26] = unsafe {
    ::core::mem::transmute::<[u8; 52], [::core::ffi::c_ushort; 26]>(
        *b"C\0r\0y\0p\0t\0O\0I\0D\0I\0n\0f\0o\0E\0C\0C\0P\0a\0r\0a\0m\0e\0t\0e\0r\0s\0\0\0",
    )
};

pub static szOID_RSA_RSA: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.1.1\0")
};

pub static szOID_RSA_MD2RSA: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.1.2\0")
};

pub static szOID_RSA_MD4RSA: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.1.3\0")
};

pub static szOID_RSA_MD5RSA: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.1.4\0")
};

pub static szOID_RSA_SHA1RSA: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.1.5\0")
};

pub static szOID_RSA_SHA256RSA: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.2.840.113549.1.1.11\0")
};

pub static szOID_RSA_SHA384RSA: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.2.840.113549.1.1.12\0")
};

pub static szOID_RSA_SHA512RSA: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.2.840.113549.1.1.13\0")
};

pub static szOID_RSA_DH: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.3.1\0")
};

pub static szOID_RSA_data: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.7.1\0")
};

pub static szOID_RSA_signedData: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.7.2\0")
};

pub static szOID_RSA_envelopedData: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.7.3\0")
};

pub static szOID_RSA_signEnvData: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.7.4\0")
};

pub static szOID_RSA_digestedData: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.7.5\0")
};

pub static szOID_RSA_encryptedData: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.7.6\0")
};

pub static szOID_RSA_emailAddr: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.1\0")
};

pub static szOID_RSA_unstructName: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.2\0")
};

pub static szOID_RSA_contentType: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.3\0")
};

pub static szOID_RSA_messageDigest: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.4\0")
};

pub static szOID_RSA_signingTime: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.5\0")
};

pub static szOID_RSA_counterSign: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.6\0")
};

pub static szOID_RSA_challengePwd: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.7\0")
};

pub static szOID_RSA_unstructAddr: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.9\0")
};

pub static szOID_RSA_certExtensions: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.2.840.113549.1.9.14\0")
};

pub static szOID_RSA_SMIMECapabilities: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.2.840.113549.1.9.15\0")
};

pub static szOID_RSA_preferSignedData: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"1.2.840.113549.1.9.15.1\0")
};

pub static szOID_RSA_SMIMEalgESDH: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"1.2.840.113549.1.9.16.3.5\0")
};

pub static szOID_RSA_SMIMEalgCMS3DESwrap: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"1.2.840.113549.1.9.16.3.6\0")
};

pub static szOID_RSA_SMIMEalgCMSRC2wrap: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"1.2.840.113549.1.9.16.3.7\0")
};

pub static szOID_RSA_MD2: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.2.840.113549.2.2\0")
};

pub static szOID_RSA_MD4: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.2.840.113549.2.4\0")
};

pub static szOID_RSA_MD5: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.2.840.113549.2.5\0")
};

pub static szOID_RSA_RC2CBC: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.2.840.113549.3.2\0")
};

pub static szOID_RSA_RC4: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.2.840.113549.3.4\0")
};

pub static szOID_RSA_DES_EDE3_CBC: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.2.840.113549.3.7\0")
};

pub static szOID_ANSI_X942_DH: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.2.840.10046.2.1\0")
};

pub static szOID_X957_DSA: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.2.840.10040.4.1\0")
};

pub static szOID_X957_SHA1DSA: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.2.840.10040.4.3\0")
};

pub static szOID_ECC_PUBLIC_KEY: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.2.840.10045.2.1\0")
};

pub static szOID_ECDSA_SHA256: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"1.2.840.10045.4.3.2\0")
};

pub static szOID_ECDSA_SHA384: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"1.2.840.10045.4.3.3\0")
};

pub static szOID_OIWSEC_md4RSA: [::core::ffi::c_char; 13] =
    unsafe { ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"1.3.14.3.2.2\0") };

pub static szOID_OIWSEC_md5RSA: [::core::ffi::c_char; 13] =
    unsafe { ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"1.3.14.3.2.3\0") };

pub static szOID_OIWSEC_md4RSA2: [::core::ffi::c_char; 13] =
    unsafe { ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"1.3.14.3.2.4\0") };

pub static szOID_OIWSEC_desCBC: [::core::ffi::c_char; 13] =
    unsafe { ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"1.3.14.3.2.7\0") };

pub static szOID_OIWSEC_dsa: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"1.3.14.3.2.12\0") };

pub static szOID_OIWSEC_shaDSA: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"1.3.14.3.2.13\0") };

pub static szOID_OIWSEC_shaRSA: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"1.3.14.3.2.15\0") };

pub static szOID_OIWSEC_sha: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"1.3.14.3.2.18\0") };

pub static szOID_OIWSEC_rsaXchg: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"1.3.14.3.2.22\0") };

pub static szOID_OIWSEC_sha1: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"1.3.14.3.2.26\0") };

pub static szOID_OIWSEC_dsaSHA1: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"1.3.14.3.2.27\0") };

pub static szOID_OIWSEC_sha1RSASign: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"1.3.14.3.2.29\0") };

pub static szOID_OIWDIR_md2RSA: [::core::ffi::c_char; 15] =
    unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"1.3.14.7.2.3.1\0") };

pub static szOID_INFOSEC_mosaicUpdatedSig: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"2.16.840.1.101.2.1.1.19\0")
};

pub static szOID_INFOSEC_mosaicKMandUpdSig: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"2.16.840.1.101.2.1.1.20\0")
};

pub static szOID_NIST_sha256: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"2.16.840.1.101.3.4.2.1\0")
};

pub static szOID_NIST_sha384: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"2.16.840.1.101.3.4.2.2\0")
};

pub static szOID_NIST_sha512: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"2.16.840.1.101.3.4.2.3\0")
};

pub static szOID_COMMON_NAME: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"2.5.4.3\0") };

pub static szOID_SUR_NAME: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"2.5.4.4\0") };

pub static szOID_DEVICE_SERIAL_NUMBER: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"2.5.4.5\0") };

pub static szOID_COUNTRY_NAME: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"2.5.4.6\0") };

pub static szOID_LOCALITY_NAME: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"2.5.4.7\0") };

pub static szOID_STATE_OR_PROVINCE_NAME: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"2.5.4.8\0") };

pub static szOID_STREET_ADDRESS: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"2.5.4.9\0") };

pub static szOID_ORGANIZATION_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.10\0") };

pub static szOID_ORGANIZATIONAL_UNIT_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.11\0") };

pub static szOID_TITLE: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.12\0") };

pub static szOID_DESCRIPTION: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.13\0") };

pub static szOID_POSTAL_CODE: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.17\0") };

pub static szOID_POST_OFFICE_BOX: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.18\0") };

pub static szOID_TELEPHONE_NUMBER: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.20\0") };

pub static szOID_X21_ADDRESS: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.24\0") };

pub static szOID_GIVEN_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.42\0") };

pub static szOID_INITIALS: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.43\0") };

pub static szOID_DN_QUALIFIER: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.46\0") };

pub static szOID_AUTHORITY_KEY_IDENTIFIER: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.1\0") };

pub static szOID_KEY_ATTRIBUTES: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.2\0") };

pub static szOID_KEY_USAGE_RESTRICTION: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.4\0") };

pub static szOID_LEGACY_POLICY_MAPPINGS: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.5\0") };

pub static szOID_SUBJECT_ALT_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.7\0") };

pub static szOID_BASIC_CONSTRAINTS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.10\0") };

pub static szOID_SUBJECT_KEY_IDENTIFIER: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.14\0") };

pub static szOID_KEY_USAGE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.15\0") };

pub static szOID_PRIVATEKEY_USAGE_PERIOD: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.16\0") };

pub static szOID_SUBJECT_ALT_NAME2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.17\0") };

pub static szOID_ISSUER_ALT_NAME2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.18\0") };

pub static szOID_BASIC_CONSTRAINTS2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.19\0") };

pub static szOID_CRL_NUMBER: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.20\0") };

pub static szOID_CRL_REASON_CODE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.21\0") };

pub static szOID_DELTA_CRL_INDICATOR: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.27\0") };

pub static szOID_ISSUING_DIST_POINT: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.28\0") };

pub static szOID_NAME_CONSTRAINTS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.30\0") };

pub static szOID_CRL_DIST_POINTS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.31\0") };

pub static szOID_CERT_POLICIES: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.32\0") };

pub static szOID_ANY_CERT_POLICY: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"2.5.29.32.0\0") };

pub static szOID_POLICY_MAPPINGS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.33\0") };

pub static szOID_AUTHORITY_KEY_IDENTIFIER2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.35\0") };

pub static szOID_POLICY_CONSTRAINTS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.36\0") };

pub static szOID_ENHANCED_KEY_USAGE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.37\0") };

pub static szOID_FRESHEST_CRL: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.46\0") };

pub static szOID_DOMAIN_COMPONENT: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"0.9.2342.19200300.100.1.25\0")
};

pub static szOID_CERT_EXTENSIONS: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.2.1.14\0")
};

pub static szOID_NEXT_UPDATE_LOCATION: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.10.2\0")
};

pub static szOID_KP_CTL_USAGE_SIGNING: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.3.1\0")
};

pub static szOID_KP_TIME_STAMP_SIGNING: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.3.2\0")
};

pub static szOID_KP_EFS: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.3.4\0")
};

pub static szOID_EFS_RECOVERY: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"1.3.6.1.4.1.311.10.3.4.1\0")
};

pub static szOID_WHQL_CRYPTO: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.3.5\0")
};

pub static szOID_NT5_CRYPTO: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.3.6\0")
};

pub static szOID_OEM_WHQL_CRYPTO: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.3.7\0")
};

pub static szOID_EMBEDDED_NT_CRYPTO: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.3.8\0")
};

pub static szOID_ROOT_LIST_SIGNER: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.3.9\0")
};

pub static szOID_KP_QUALIFIED_SUBORDINATION: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"1.3.6.1.4.1.311.10.3.10\0")
};

pub static szOID_KP_KEY_RECOVERY: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"1.3.6.1.4.1.311.10.3.11\0")
};

pub static szOID_KP_DOCUMENT_SIGNING: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"1.3.6.1.4.1.311.10.3.12\0")
};

pub static szOID_KP_LIFETIME_SIGNING: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"1.3.6.1.4.1.311.10.3.13\0")
};

pub static szOID_YESNO_TRUST_ATTR: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.4.1\0")
};

pub static szOID_DRM: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.5.1\0")
};

pub static szOID_LICENSES: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.6.1\0")
};

pub static szOID_LICENSE_SERVER: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.6.2\0")
};

pub static szOID_CROSS_CERT_DIST_POINTS: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.9.1\0")
};

pub static szOID_CTL: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.10.1\0")
};

pub static szOID_SORTED_CTL: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.1.1\0")
};

pub static szOID_ANY_APPLICATION_POLICY: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"1.3.6.1.4.1.311.10.12.1\0")
};

pub static szOID_ENROLLMENT_NAME_VALUE_PAIR: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.13.2.1\0")
};

pub static szOID_ENROLLMENT_CSP_PROVIDER: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.13.2.2\0")
};

pub static szOID_OS_VERSION: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.13.2.3\0")
};

pub static szOID_ENROLL_CERTTYPE_EXTENSION: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.20.2\0")
};

pub static szOID_ENROLLMENT_AGENT: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.20.2.1\0")
};

pub static szOID_KP_SMARTCARD_LOGON: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.20.2.2\0")
};

pub static szOID_NT_PRINCIPAL_NAME: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.20.2.3\0")
};

pub static szOID_CERT_MANIFOLD: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.20.3\0")
};

pub static szOID_CERTSRV_CA_VERSION: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.21.1\0")
};

pub static szOID_CERTSRV_PREVIOUS_CERT_HASH: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.21.2\0")
};

pub static szOID_CRL_VIRTUAL_BASE: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.21.3\0")
};

pub static szOID_CRL_NEXT_PUBLISH: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.21.4\0")
};

pub static szOID_KP_CA_EXCHANGE: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.21.5\0")
};

pub static szOID_KP_KEY_RECOVERY_AGENT: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.21.6\0")
};

pub static szOID_CERTIFICATE_TEMPLATE: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.21.7\0")
};

pub static szOID_ENTERPRISE_OID_ROOT: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.21.8\0")
};

pub static szOID_RDN_DUMMY_SIGNER: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.21.9\0")
};

pub static szOID_APPLICATION_CERT_POLICIES: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.3.6.1.4.1.311.21.10\0")
};

pub static szOID_APPLICATION_POLICY_MAPPINGS: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.3.6.1.4.1.311.21.11\0")
};

pub static szOID_APPLICATION_POLICY_CONSTRAINTS: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.3.6.1.4.1.311.21.12\0")
};

pub static szOID_ARCHIVED_KEY_ATTR: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.3.6.1.4.1.311.21.13\0")
};

pub static szOID_CRL_SELF_CDP: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.3.6.1.4.1.311.21.14\0")
};

pub static szOID_REQUIRE_CERT_CHAIN_POLICY: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.3.6.1.4.1.311.21.15\0")
};

pub static szOID_ARCHIVED_KEY_CERT_HASH: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.3.6.1.4.1.311.21.16\0")
};

pub static szOID_DS_EMAIL_REPLICATION: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.3.6.1.4.1.311.21.19\0")
};

pub static szOID_REQUEST_CLIENT_INFO: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.3.6.1.4.1.311.21.20\0")
};

pub static szOID_CERTSRV_CROSSCA_VERSION: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.3.6.1.4.1.311.21.22\0")
};

pub static szOID_AUTHORITY_INFO_ACCESS: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.1.1\0")
};

pub static szOID_PKIX_POLICY_QUALIFIER_CPS: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.2.1\0")
};

pub static szOID_PKIX_POLICY_QUALIFIER_USERNOTICE: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.2.2\0")
};

pub static szOID_PKIX_KP_SERVER_AUTH: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.3.1\0")
};

pub static szOID_PKIX_KP_CLIENT_AUTH: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.3.2\0")
};

pub static szOID_PKIX_KP_CODE_SIGNING: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.3.3\0")
};

pub static szOID_PKIX_KP_EMAIL_PROTECTION: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.3.4\0")
};

pub static szOID_PKIX_KP_IPSEC_END_SYSTEM: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.3.5\0")
};

pub static szOID_PKIX_KP_IPSEC_TUNNEL: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.3.6\0")
};

pub static szOID_PKIX_KP_IPSEC_USER: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.3.7\0")
};

pub static szOID_PKIX_KP_TIMESTAMP_SIGNING: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.3.8\0")
};

pub static szOID_PKIX_NO_SIGNATURE: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.6.2\0")
};

pub static szOID_CMC: [::core::ffi::c_char; 16] =
    unsafe { ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(*b"1.3.6.1.5.5.7.7\0") };

pub static szOID_CMC_STATUS_INFO: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.7.1\0")
};

pub static szOID_CMC_TRANSACTION_ID: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.7.5\0")
};

pub static szOID_CMC_SENDER_NONCE: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.7.6\0")
};

pub static szOID_CMC_RECIPIENT_NONCE: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.7.7\0")
};

pub static szOID_CMC_ADD_EXTENSIONS: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.7.8\0")
};

pub static szOID_CMC_GET_CERT: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.3.6.1.5.5.7.7.15\0")
};

pub static szOID_CMC_GET_CRL: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.3.6.1.5.5.7.7.16\0")
};

pub static szOID_CMC_REVOKE_REQUEST: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.3.6.1.5.5.7.7.17\0")
};

pub static szOID_CMC_REG_INFO: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.3.6.1.5.5.7.7.18\0")
};

pub static szOID_CMC_QUERY_PENDING: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.3.6.1.5.5.7.7.21\0")
};

pub static szOID_CT_PKI_DATA: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.3.6.1.5.5.7.12.2\0")
};

pub static szOID_CT_PKI_RESPONSE: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.3.6.1.5.5.7.12.3\0")
};

pub static szOID_PKIX_OCSP: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.3.6.1.5.5.7.48.1\0")
};

pub static szOID_PKIX_CA_ISSUERS: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.3.6.1.5.5.7.48.2\0")
};

pub static szOID_IPSEC_KP_IKE_INTERMEDIATE: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.8.2.2\0")
};

pub static szOID_SERIALIZED: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"1.3.6.1.4.1.311.10.3.3.1\0")
};

pub static szOID_PRODUCT_UPDATE: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.31.1\0")
};

pub static szOID_NETSCAPE_CERT_TYPE: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.1\0")
};

pub static szOID_NETSCAPE_BASE_URL: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.2\0")
};

pub static szOID_NETSCAPE_REVOCATION_URL: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.3\0")
};

pub static szOID_NETSCAPE_CA_REVOCATION_URL: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.4\0")
};

pub static szOID_NETSCAPE_CERT_RENEWAL_URL: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.7\0")
};

pub static szOID_NETSCAPE_CA_POLICY_URL: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.8\0")
};

pub static szOID_NETSCAPE_SSL_SERVER_NAME: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"2.16.840.1.113730.1.12\0")
};

pub static szOID_NETSCAPE_COMMENT: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"2.16.840.1.113730.1.13\0")
};

static mut funcSetCSDebug: CRITICAL_SECTION_DEBUG = unsafe { _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut funcSetCS,
        ProcessLocksList: _LIST_ENTRY {
            Flink: &raw mut funcSetCSDebug.ProcessLocksList,
            Blink: &raw mut funcSetCSDebug.ProcessLocksList,
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    }};

static mut funcSetCS: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const funcSetCSDebug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

static mut funcSets: list = unsafe {
    list {
        next: &raw const funcSets as *mut list,
        prev: &raw const funcSets as *mut list,
    }
};

static mut LocalizedKeys: [LPCWSTR; 6] = unsafe {
    [
        RUSTCONV_C2R_WSTR_90953F2876F4.as_ptr(),
        RUSTCONV_C2R_WSTR_D90081DF0539.as_ptr(),
        RUSTCONV_C2R_WSTR_4A7FEDDCAC3E.as_ptr(),
        RUSTCONV_C2R_WSTR_3F9B2FCDB174.as_ptr(),
        RUSTCONV_C2R_WSTR_219032C80178.as_ptr(),
        RUSTCONV_C2R_WSTR_B89AF30006C8.as_ptr(),
    ]
};

static mut LocalizedNames: [[WCHAR; 256]; 6] = [[0; 256]; 6];

static mut oidInfoCSDebug: CRITICAL_SECTION_DEBUG = unsafe { _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut oidInfoCS,
        ProcessLocksList: _LIST_ENTRY {
            Flink: &raw mut oidInfoCSDebug.ProcessLocksList,
            Blink: &raw mut oidInfoCSDebug.ProcessLocksList,
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    }};

static mut oidInfoCS: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const oidInfoCSDebug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

static mut oidInfo: list = unsafe {
    list {
        next: &raw const oidInfo as *mut list,
        prev: &raw const oidInfo as *mut list,
    }
};

static mut noNullFlag: DWORD = CRYPT_OID_NO_NULL_ALGORITHM_PARA_FLAG as DWORD;

static mut mosaicFlags: DWORD =
    (CRYPT_OID_INHIBIT_SIGNATURE_FORMAT_FLAG | CRYPT_OID_NO_NULL_ALGORITHM_PARA_FLAG) as DWORD;

static mut noNullBlob: CRYPT_DATA_BLOB = unsafe {
    _CRYPTOAPI_BLOB {
        cbData: ::core::mem::size_of::<DWORD>() as DWORD,
        pbData: &raw const noNullFlag as *mut BYTE,
    }
};

static mut mosaicFlagsBlob: CRYPT_DATA_BLOB = unsafe {
    _CRYPTOAPI_BLOB {
        cbData: ::core::mem::size_of::<DWORD>() as DWORD,
        pbData: &raw const mosaicFlags as *mut BYTE,
    }
};

static mut rsaSign: DWORD = CALG_RSA_SIGN as DWORD;

static mut dssSign: [DWORD; 2] = [
    CALG_DSS_SIGN as DWORD,
    CRYPT_OID_NO_NULL_ALGORITHM_PARA_FLAG as DWORD,
];

static mut mosaicSign: [DWORD; 2] = [
    CALG_DSS_SIGN as DWORD,
    (CRYPT_OID_INHIBIT_SIGNATURE_FORMAT_FLAG | CRYPT_OID_NO_NULL_ALGORITHM_PARA_FLAG) as DWORD,
];

static mut ecdsaSign: [DWORD; 2] = [
    CALG_OID_INFO_PARAMETERS as DWORD,
    CRYPT_OID_NO_NULL_ALGORITHM_PARA_FLAG as DWORD,
];

static mut rsaSignBlob: CRYPT_DATA_BLOB = unsafe {
    _CRYPTOAPI_BLOB {
        cbData: ::core::mem::size_of::<DWORD>() as DWORD,
        pbData: &raw const rsaSign as *mut BYTE,
    }
};

static mut dssSignBlob: CRYPT_DATA_BLOB = unsafe {
    _CRYPTOAPI_BLOB {
        cbData: ::core::mem::size_of::<[DWORD; 2]>() as DWORD,
        pbData: &raw const dssSign as *const DWORD as *mut BYTE,
    }
};

static mut mosaicSignBlob: CRYPT_DATA_BLOB = unsafe {
    _CRYPTOAPI_BLOB {
        cbData: ::core::mem::size_of::<[DWORD; 2]>() as DWORD,
        pbData: &raw const mosaicSign as *const DWORD as *mut BYTE,
    }
};

static mut ecdsaSignBlob: CRYPT_DATA_BLOB = unsafe {
    _CRYPTOAPI_BLOB {
        cbData: ::core::mem::size_of::<[DWORD; 2]>() as DWORD,
        pbData: &raw const ecdsaSign as *const DWORD as *mut BYTE,
    }
};

static mut ia5String: [DWORD; 2] = [
    CERT_RDN_IA5_STRING as DWORD,
    0 as ::core::ffi::c_int as DWORD,
];

static mut numericString: [DWORD; 2] = [
    CERT_RDN_NUMERIC_STRING as DWORD,
    0 as ::core::ffi::c_int as DWORD,
];

static mut printableString: [DWORD; 2] = [
    CERT_RDN_PRINTABLE_STRING as DWORD,
    0 as ::core::ffi::c_int as DWORD,
];

static mut domainCompTypes: [DWORD; 3] = [
    CERT_RDN_IA5_STRING as DWORD,
    CERT_RDN_UTF8_STRING as DWORD,
    0 as ::core::ffi::c_int as DWORD,
];

static mut ia5StringBlob: CRYPT_DATA_BLOB = unsafe {
    _CRYPTOAPI_BLOB {
        cbData: ::core::mem::size_of::<[DWORD; 2]>() as DWORD,
        pbData: &raw const ia5String as *const DWORD as *mut BYTE,
    }
};

static mut numericStringBlob: CRYPT_DATA_BLOB = unsafe {
    _CRYPTOAPI_BLOB {
        cbData: ::core::mem::size_of::<[DWORD; 2]>() as DWORD,
        pbData: &raw const numericString as *const DWORD as *mut BYTE,
    }
};

static mut printableStringBlob: CRYPT_DATA_BLOB = unsafe {
    _CRYPTOAPI_BLOB {
        cbData: ::core::mem::size_of::<[DWORD; 2]>() as DWORD,
        pbData: &raw const printableString as *const DWORD as *mut BYTE,
    }
};

static mut domainCompTypesBlob: CRYPT_DATA_BLOB = unsafe {
    _CRYPTOAPI_BLOB {
        cbData: ::core::mem::size_of::<[DWORD; 3]>() as DWORD,
        pbData: &raw const domainCompTypes as *const DWORD as *mut BYTE,
    }
};

static mut oidInfoConstructors: [OIDInfoConstructor; 226] = unsafe {
    [
        OIDInfoConstructor {
            dwGroupId: 1 as DWORD,
            pszOID: szOID_OIWSEC_sha1.as_ptr() as LPCSTR,
            Algid: CALG_SHA1 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_A875FC2B07CD.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 1 as DWORD,
            pszOID: szOID_OIWSEC_sha1.as_ptr() as LPCSTR,
            Algid: CALG_SHA1 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_E465C4D6880D.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 1 as DWORD,
            pszOID: szOID_OIWSEC_sha.as_ptr() as LPCSTR,
            Algid: CALG_SHA as UINT,
            pwszName: RUSTCONV_C2R_WSTR_E465C4D6880D.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 1 as DWORD,
            pszOID: szOID_RSA_MD5.as_ptr() as LPCSTR,
            Algid: CALG_MD5 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_700AFFCD0C9F.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 1 as DWORD,
            pszOID: szOID_RSA_MD4.as_ptr() as LPCSTR,
            Algid: CALG_MD4 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_F93A8BF12ADB.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 1 as DWORD,
            pszOID: szOID_RSA_MD2.as_ptr() as LPCSTR,
            Algid: CALG_MD2 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_4979BF393372.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 1 as DWORD,
            pszOID: szOID_NIST_sha256.as_ptr() as LPCSTR,
            Algid: CALG_SHA_256 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_17B90C614D61.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 1 as DWORD,
            pszOID: szOID_NIST_sha384.as_ptr() as LPCSTR,
            Algid: CALG_SHA_384 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_986E97A25E53.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 1 as DWORD,
            pszOID: szOID_NIST_sha512.as_ptr() as LPCSTR,
            Algid: CALG_SHA_512 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_E759B459F85B.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 2 as DWORD,
            pszOID: szOID_OIWSEC_desCBC.as_ptr() as LPCSTR,
            Algid: CALG_DES as UINT,
            pwszName: RUSTCONV_C2R_WSTR_C13B14C662E4.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 2 as DWORD,
            pszOID: szOID_RSA_DES_EDE3_CBC.as_ptr() as LPCSTR,
            Algid: CALG_3DES as UINT,
            pwszName: RUSTCONV_C2R_WSTR_28F6ADD86F3C.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 2 as DWORD,
            pszOID: szOID_RSA_RC2CBC.as_ptr() as LPCSTR,
            Algid: CALG_RC2 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_F03765AE14C6.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 2 as DWORD,
            pszOID: szOID_RSA_RC4.as_ptr() as LPCSTR,
            Algid: CALG_RC4 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_70A60BDF9EBF.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 2 as DWORD,
            pszOID: szOID_RSA_SMIMEalgCMS3DESwrap.as_ptr() as LPCSTR,
            Algid: CALG_3DES as UINT,
            pwszName: RUSTCONV_C2R_WSTR_D38D07CD8B99.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 2 as DWORD,
            pszOID: szOID_RSA_SMIMEalgCMSRC2wrap.as_ptr() as LPCSTR,
            Algid: CALG_RC2 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_43FF56D4E41E.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_RSA_RSA.as_ptr() as LPCSTR,
            Algid: CALG_RSA_KEYX as UINT,
            pwszName: RUSTCONV_C2R_WSTR_0C38FC8A9E0F.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_X957_DSA.as_ptr() as LPCSTR,
            Algid: CALG_DSS_SIGN as UINT,
            pwszName: RUSTCONV_C2R_WSTR_897FF00D4F10.as_ptr() as LPCWSTR,
            blob: &raw const noNullBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_ANSI_X942_DH.as_ptr() as LPCSTR,
            Algid: CALG_DH_SF as UINT,
            pwszName: RUSTCONV_C2R_WSTR_6AA387BF4CB9.as_ptr() as LPCWSTR,
            blob: &raw const noNullBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_RSA_RSA.as_ptr() as LPCSTR,
            Algid: CALG_RSA_KEYX as UINT,
            pwszName: RUSTCONV_C2R_WSTR_3A4C0616016C.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_RSA_RSA.as_ptr() as LPCSTR,
            Algid: CALG_RSA_SIGN as UINT,
            pwszName: RUSTCONV_C2R_WSTR_0C38FC8A9E0F.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_RSA_RSA.as_ptr() as LPCSTR,
            Algid: CALG_RSA_SIGN as UINT,
            pwszName: RUSTCONV_C2R_WSTR_2FD0C55DD168.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_OIWSEC_dsa.as_ptr() as LPCSTR,
            Algid: CALG_DSS_SIGN as UINT,
            pwszName: RUSTCONV_C2R_WSTR_897FF00D4F10.as_ptr() as LPCWSTR,
            blob: &raw const noNullBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_OIWSEC_dsa.as_ptr() as LPCSTR,
            Algid: CALG_DSS_SIGN as UINT,
            pwszName: RUSTCONV_C2R_WSTR_C6488821BD31.as_ptr() as LPCWSTR,
            blob: &raw const noNullBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_OIWSEC_dsa.as_ptr() as LPCSTR,
            Algid: CALG_DSS_SIGN as UINT,
            pwszName: RUSTCONV_C2R_WSTR_53A00CD4BB7F.as_ptr() as LPCWSTR,
            blob: &raw const noNullBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_RSA_DH.as_ptr() as LPCSTR,
            Algid: CALG_DH_SF as UINT,
            pwszName: RUSTCONV_C2R_WSTR_6AA387BF4CB9.as_ptr() as LPCWSTR,
            blob: &raw const noNullBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_OIWSEC_rsaXchg.as_ptr() as LPCSTR,
            Algid: CALG_RSA_KEYX as UINT,
            pwszName: RUSTCONV_C2R_WSTR_3A4C0616016C.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_INFOSEC_mosaicKMandUpdSig.as_ptr() as LPCSTR,
            Algid: CALG_DSS_SIGN as UINT,
            pwszName: RUSTCONV_C2R_WSTR_2129F45562B7.as_ptr() as LPCWSTR,
            blob: &raw const mosaicFlagsBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_RSA_SMIMEalgESDH.as_ptr() as LPCSTR,
            Algid: CALG_DH_EPHEM as UINT,
            pwszName: RUSTCONV_C2R_WSTR_01FE3A650989.as_ptr() as LPCWSTR,
            blob: &raw const noNullBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_PKIX_NO_SIGNATURE.as_ptr() as LPCSTR,
            Algid: CALG_NO_SIGN as UINT,
            pwszName: RUSTCONV_C2R_WSTR_832ABBDE2DC9.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 3 as DWORD,
            pszOID: szOID_ECC_PUBLIC_KEY.as_ptr() as LPCSTR,
            Algid: CALG_OID_INFO_PARAMETERS,
            pwszName: RUSTCONV_C2R_WSTR_D2D100D7E39F.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: CRYPT_OID_INFO_ECC_PARAMETERS_ALGORITHM.as_ptr() as *const WCHAR,
            pwszCNGExtraAlgid: RUSTCONV_C2R_WSTR_96A296D224F2.as_ptr() as *const WCHAR,
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_RSA_SHA1RSA.as_ptr() as LPCSTR,
            Algid: CALG_SHA1 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_30CE221CF490.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_RSA_SHA256RSA.as_ptr() as LPCSTR,
            Algid: CALG_SHA_256 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_9AAA20F77D73.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_RSA_SHA384RSA.as_ptr() as LPCSTR,
            Algid: CALG_SHA_384 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_7CE7C36007E8.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_RSA_SHA512RSA.as_ptr() as LPCSTR,
            Algid: CALG_SHA_512 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_657E199B2FCE.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_RSA_MD5RSA.as_ptr() as LPCSTR,
            Algid: CALG_MD5 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_C0390936EE95.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_X957_SHA1DSA.as_ptr() as LPCSTR,
            Algid: CALG_SHA1 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_7B8E1FF9CAD5.as_ptr() as LPCWSTR,
            blob: &raw const dssSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_OIWSEC_sha1RSASign.as_ptr() as LPCSTR,
            Algid: CALG_SHA1 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_30CE221CF490.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_OIWSEC_sha1RSASign.as_ptr() as LPCSTR,
            Algid: CALG_SHA1 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_1FFF2DC848DB.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_OIWSEC_shaRSA.as_ptr() as LPCSTR,
            Algid: CALG_SHA1 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_1FFF2DC848DB.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_OIWSEC_md5RSA.as_ptr() as LPCSTR,
            Algid: CALG_MD5 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_C0390936EE95.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_RSA_MD2RSA.as_ptr() as LPCSTR,
            Algid: CALG_MD2 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_A97874299B48.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_RSA_MD4RSA.as_ptr() as LPCSTR,
            Algid: CALG_MD4 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_0B4A96579C77.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_OIWSEC_md4RSA.as_ptr() as LPCSTR,
            Algid: CALG_MD4 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_0B4A96579C77.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_OIWSEC_md4RSA2.as_ptr() as LPCSTR,
            Algid: CALG_MD4 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_0B4A96579C77.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_OIWDIR_md2RSA.as_ptr() as LPCSTR,
            Algid: CALG_MD2 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_A97874299B48.as_ptr() as LPCWSTR,
            blob: &raw const rsaSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_OIWSEC_shaDSA.as_ptr() as LPCSTR,
            Algid: CALG_SHA1 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_7B8E1FF9CAD5.as_ptr() as LPCWSTR,
            blob: &raw const dssSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_OIWSEC_shaDSA.as_ptr() as LPCSTR,
            Algid: CALG_SHA1 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_6E9668E556EE.as_ptr() as LPCWSTR,
            blob: &raw const dssSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_OIWSEC_dsaSHA1.as_ptr() as LPCSTR,
            Algid: CALG_SHA1 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_699F058FE8D8.as_ptr() as LPCWSTR,
            blob: &raw const dssSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_INFOSEC_mosaicUpdatedSig.as_ptr() as LPCSTR,
            Algid: CALG_SHA1 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_177EF5904691.as_ptr() as LPCWSTR,
            blob: &raw const mosaicSignBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_ECDSA_SHA256.as_ptr() as LPCSTR,
            Algid: CALG_OID_INFO_CNG_ONLY,
            pwszName: RUSTCONV_C2R_WSTR_B1864A7F14E8.as_ptr() as LPCWSTR,
            blob: &raw const ecdsaSignBlob,
            pwszCNGAlgid: BCRYPT_SHA256_ALGORITHM.as_ptr() as *const WCHAR,
            pwszCNGExtraAlgid: CRYPT_OID_INFO_ECC_PARAMETERS_ALGORITHM.as_ptr() as *const WCHAR,
        },
        OIDInfoConstructor {
            dwGroupId: 4 as DWORD,
            pszOID: szOID_ECDSA_SHA384.as_ptr() as LPCSTR,
            Algid: CALG_OID_INFO_CNG_ONLY,
            pwszName: RUSTCONV_C2R_WSTR_BCE20B4ED641.as_ptr() as LPCWSTR,
            blob: &raw const ecdsaSignBlob,
            pwszCNGAlgid: BCRYPT_SHA384_ALGORITHM.as_ptr() as *const WCHAR,
            pwszCNGExtraAlgid: CRYPT_OID_INFO_ECC_PARAMETERS_ALGORITHM.as_ptr() as *const WCHAR,
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_COMMON_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_FBCEAF5CF84D.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_LOCALITY_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_33149FF2DC42.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_ORGANIZATION_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_CFF611E6FECB.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_ORGANIZATIONAL_UNIT_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_25546986ABCE.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_RSA_emailAddr.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_4FD33DCDE470.as_ptr() as LPCWSTR,
            blob: &raw const ia5StringBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_RSA_emailAddr.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_CE12E1294E6F.as_ptr() as LPCWSTR,
            blob: &raw const ia5StringBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_COUNTRY_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_D23F96D836EB.as_ptr() as LPCWSTR,
            blob: &raw const printableStringBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_STATE_OR_PROVINCE_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_5BB10A0EA4AA.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_STATE_OR_PROVINCE_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_4664D9C9B4B4.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_STREET_ADDRESS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_937515BA47B8.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_TITLE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_3FBF037CB6BC.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_TITLE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_3BAC45FDAD1C.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_GIVEN_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_0AB677189BBD.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_GIVEN_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_0579D3D5469E.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_GIVEN_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_F5D02491CB00.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_INITIALS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_6CB5A8EC7215.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_INITIALS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_8693927E0562.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_SUR_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_BBD9F07EA39C.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_DOMAIN_COMPONENT.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_CF4658672D80.as_ptr() as LPCWSTR,
            blob: &raw const domainCompTypesBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_DESCRIPTION.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_6AC123285056.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_POSTAL_CODE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_35B543933849.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_POST_OFFICE_BOX.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_D26390A5FDEE.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_TELEPHONE_NUMBER.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_C1612A44FB42.as_ptr() as LPCWSTR,
            blob: &raw const printableStringBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_X21_ADDRESS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_9ECA39E104E6.as_ptr() as LPCWSTR,
            blob: &raw const numericStringBlob,
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_DN_QUALIFIER.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_F097069263D2.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 5 as DWORD,
            pszOID: szOID_DEVICE_SERIAL_NUMBER.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_FB67DA7C36CC.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_AUTHORITY_KEY_IDENTIFIER2.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_AUTHORITY_KEY_ID as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_AUTHORITY_KEY_IDENTIFIER.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_AUTHORITY_KEY_ID as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_KEY_ATTRIBUTES.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_KEY_ATTRIBUTES as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_KEY_USAGE_RESTRICTION.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_KEY_USAGE_RESTRICTION as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_SUBJECT_ALT_NAME2.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SUBJECT_ALT_NAME as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_SUBJECT_ALT_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SUBJECT_ALT_NAME as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ISSUER_ALT_NAME2.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ISSUER_ALT_NAME as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ISSUER_ALT_NAME2.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ISSUER_ALT_NAME as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_BASIC_CONSTRAINTS2.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_BASIC_CONSTRAINTS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_BASIC_CONSTRAINTS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_BASIC_CONSTRAINTS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_KEY_USAGE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_KEY_USAGE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CERT_POLICIES.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CERT_POLICIES as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_SUBJECT_KEY_IDENTIFIER.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SUBJECT_KEY_IDENTIFIER as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CRL_REASON_CODE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CRL_REASON_CODE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CRL_DIST_POINTS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CRL_DIST_POINTS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ENHANCED_KEY_USAGE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ENHANCED_KEY_USAGE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_AUTHORITY_INFO_ACCESS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_AUTHORITY_INFO_ACCESS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CERT_EXTENSIONS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CERT_EXTENSIONS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_certExtensions.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CERT_EXTENSIONS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_NEXT_UPDATE_LOCATION.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_NEXT_UPDATE_LOCATION as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_YESNO_TRUST_ATTR.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_YES_OR_NO_TRUST as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_emailAddr.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_EMAIL_ADDRESS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_unstructName.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_UNSTRUCTURED_NAME as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_contentType.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CONTENT_TYPE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_messageDigest.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_MESSAGE_DIGEST as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_signingTime.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SIGNING_TIME as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_counterSign.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_COUNTER_SIGN as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_challengePwd.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CHALLENGE_PASSWORD as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_unstructAddr.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_UNSTRUCTURED_ADDRESS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_SMIMECapabilities.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SMIME_CAPABILITIES as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_preferSignedData.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_PREFER_SIGNED_DATA as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_PKIX_POLICY_QUALIFIER_CPS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CPS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_PKIX_POLICY_QUALIFIER_USERNOTICE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_USER_NOTICE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_PKIX_OCSP.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_OCSP as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_PKIX_CA_ISSUERS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CA_ISSUER as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ENROLL_CERTTYPE_EXTENSION.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CERT_TEMPLATE_NAME as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ENROLL_CERTTYPE_EXTENSION.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CERT_TYPE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CERT_MANIFOLD.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CERT_MANIFOLD as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_NETSCAPE_CERT_TYPE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_NETSCAPE_CERT_TYPE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_NETSCAPE_BASE_URL.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_NETSCAPE_BASE_URL as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_NETSCAPE_REVOCATION_URL.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_NETSCAPE_REVOCATION_URL as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_NETSCAPE_CA_REVOCATION_URL.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_NETSCAPE_CA_REVOCATION_URL as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_NETSCAPE_CERT_RENEWAL_URL.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_NETSCAPE_CERT_RENEWAL_URL as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_NETSCAPE_CA_POLICY_URL.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_NETSCAPE_CA_POLICY_URL as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_NETSCAPE_SSL_SERVER_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_NETSCAPE_SSL_SERVER_NAME as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_NETSCAPE_COMMENT.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_NETSCAPE_COMMENT as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: b"1.3.6.1.4.1.311.2.1.10\0" as *const u8 as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_2D3BB1F8582B.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: b"1.3.6.1.4.1.311.2.1.27\0" as *const u8 as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_86E423968AAB.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: b"1.3.6.1.4.1.311.2.1.26\0" as *const u8 as LPCSTR,
            Algid: 0 as UINT,
            pwszName: RUSTCONV_C2R_WSTR_3422FF7B7FF7.as_ptr() as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_COUNTRY_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_COUNTRY as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ORGANIZATION_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ORGANIZATION as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ORGANIZATIONAL_UNIT_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ORGANIZATIONAL_UNIT as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_COMMON_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_COMMON_NAME as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_LOCALITY_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_LOCALITY as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_STATE_OR_PROVINCE_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_STATE_OR_PROVINCE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_TITLE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_TITLE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_GIVEN_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_GIVEN_NAME as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_INITIALS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_INITIALS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_SUR_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SUR_NAME as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_DOMAIN_COMPONENT.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_DOMAIN_COMPONENT as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_STREET_ADDRESS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_STREET_ADDRESS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_DEVICE_SERIAL_NUMBER.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SERIAL_NUMBER as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CERTSRV_CA_VERSION.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CA_VERSION as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CERTSRV_CROSSCA_VERSION.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CROSS_CA_VERSION as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_SERIALIZED.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SERIALIZED_SIG_SERIAL_NUMBER as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_NT_PRINCIPAL_NAME.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_PRINCIPAL_NAME as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_PRODUCT_UPDATE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_WINDOWS_PRODUCT_UPDATE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ENROLLMENT_NAME_VALUE_PAIR.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ENROLLMENT_NAME_VALUE_PAIR as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_OS_VERSION.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_OS_VERSION as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ENROLLMENT_CSP_PROVIDER.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ENROLLMENT_CSP as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CRL_NUMBER.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CRL_NUMBER as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_DELTA_CRL_INDICATOR.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_DELTA_CRL_INDICATOR as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ISSUING_DIST_POINT.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ISSUING_DIST_POINT as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_FRESHEST_CRL.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_FRESHEST_CRL as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_NAME_CONSTRAINTS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_NAME_CONSTRAINTS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_POLICY_MAPPINGS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_POLICY_MAPPINGS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_LEGACY_POLICY_MAPPINGS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_POLICY_MAPPINGS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_POLICY_CONSTRAINTS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_POLICY_CONSTRAINTS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CROSS_CERT_DIST_POINTS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CROSS_CERT_DIST_POINTS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_APPLICATION_CERT_POLICIES.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_APPLICATION_POLICIES as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_APPLICATION_POLICY_MAPPINGS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_APPLICATION_POLICY_MAPPINGS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_APPLICATION_POLICY_CONSTRAINTS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_APPLICATION_POLICY_CONSTRAINTS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CT_PKI_DATA.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CMC_DATA as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CT_PKI_RESPONSE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CMC_RESPONSE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CMC.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_UNSIGNED_CMC_REQUEST as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CMC_STATUS_INFO.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CMC_STATUS_INFO as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CMC_ADD_EXTENSIONS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CMC_EXTENSIONS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CTL.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CMC_ATTRIBUTES as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_data.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_PKCS_7_DATA as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_signedData.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_PKCS_7_SIGNED as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_envelopedData.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_PKCS_7_ENVELOPED as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_signEnvData.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_PKCS_7_SIGNED_ENVELOPED as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_digestedData.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_PKCS_7_DIGESTED as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RSA_encryptedData.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_PKCS_7_ENCRYPTED as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CERTSRV_PREVIOUS_CERT_HASH.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_PREVIOUS_CA_CERT_HASH as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CRL_VIRTUAL_BASE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CRL_VIRTUAL_BASE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CRL_NEXT_PUBLISH.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CRL_NEXT_PUBLISH as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_KP_CA_EXCHANGE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CA_EXCHANGE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_KP_KEY_RECOVERY_AGENT.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_KEY_RECOVERY_AGENT as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CERTIFICATE_TEMPLATE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CERTIFICATE_TEMPLATE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ENTERPRISE_OID_ROOT.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ENTERPRISE_ROOT_OID as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_RDN_DUMMY_SIGNER.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_RDN_DUMMY_SIGNER as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ARCHIVED_KEY_ATTR.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ARCHIVED_KEY_ATTR as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CRL_SELF_CDP.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CRL_SELF_CDP as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_REQUIRE_CERT_CHAIN_POLICY.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_REQUIRE_CERT_CHAIN_POLICY as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CMC_TRANSACTION_ID.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_TRANSACTION_ID as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CMC_SENDER_NONCE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SENDER_NONCE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CMC_RECIPIENT_NONCE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_RECIPIENT_NONCE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CMC_REG_INFO.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_REG_INFO as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CMC_GET_CERT.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_GET_CERTIFICATE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CMC_GET_CRL.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_GET_CRL as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CMC_REVOKE_REQUEST.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_REVOKE_REQUEST as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_CMC_QUERY_PENDING.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_QUERY_PENDING as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_SORTED_CTL.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SORTED_CTL as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_ARCHIVED_KEY_CERT_HASH.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ARCHIVED_KEY_CERT_HASH as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_PRIVATEKEY_USAGE_PERIOD.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_PRIVATE_KEY_USAGE_PERIOD as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 6 as DWORD,
            pszOID: szOID_REQUEST_CLIENT_INFO.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CLIENT_INFORMATION as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_PKIX_KP_SERVER_AUTH.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SERVER_AUTHENTICATION as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_PKIX_KP_CLIENT_AUTH.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CLIENT_AUTHENTICATION as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_PKIX_KP_CODE_SIGNING.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CODE_SIGNING as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_PKIX_KP_EMAIL_PROTECTION.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SECURE_EMAIL as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_PKIX_KP_TIMESTAMP_SIGNING.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_TIME_STAMPING as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_KP_CTL_USAGE_SIGNING.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_MICROSOFT_TRUST_LIST_SIGNING as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_KP_TIME_STAMP_SIGNING.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_MICROSOFT_TIME_STAMPING as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_PKIX_KP_IPSEC_END_SYSTEM.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_IPSEC_END_SYSTEM as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_PKIX_KP_IPSEC_TUNNEL.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_IPSEC_TUNNEL as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_PKIX_KP_IPSEC_USER.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_IPSEC_USER as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_KP_EFS.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_EFS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_WHQL_CRYPTO.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_WHQL_CRYPTO as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_NT5_CRYPTO.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_NT5_CRYPTO as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_OEM_WHQL_CRYPTO.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_OEM_WHQL_CRYPTO as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_EMBEDDED_NT_CRYPTO.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_EMBEDDED_NT_CRYPTO as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_LICENSES.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_KEY_PACK_LICENSES as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_LICENSE_SERVER.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_LICENSE_SERVER as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_KP_SMARTCARD_LOGON.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_SMART_CARD_LOGON as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_DRM.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_DIGITAL_RIGHTS as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_KP_QUALIFIED_SUBORDINATION.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_QUALIFIED_SUBORDINATION as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_KP_KEY_RECOVERY.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_KEY_RECOVERY as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_KP_DOCUMENT_SIGNING.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_DOCUMENT_SIGNING as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_IPSEC_KP_IKE_INTERMEDIATE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_IPSEC_IKE_INTERMEDIATE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_EFS_RECOVERY.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_FILE_RECOVERY as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_ROOT_LIST_SIGNER.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ROOT_LIST_SIGNER as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_ANY_APPLICATION_POLICY.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ANY_APPLICATION_POLICIES as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_DS_EMAIL_REPLICATION.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_DS_EMAIL_REPLICATION as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_ENROLLMENT_AGENT.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ENROLLMENT_AGENT as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_KP_KEY_RECOVERY_AGENT.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_KEY_RECOVERY_AGENT as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_KP_CA_EXCHANGE.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_CA_EXCHANGE as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 7 as DWORD,
            pszOID: szOID_KP_LIFETIME_SIGNING.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_LIFETIME_SIGNING as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
        OIDInfoConstructor {
            dwGroupId: 8 as DWORD,
            pszOID: szOID_ANY_CERT_POLICY.as_ptr() as LPCSTR,
            Algid: 0 as UINT,
            pwszName: IDS_ANY_CERT_POLICY as LPCWSTR,
            blob: ::core::ptr::null::<CRYPT_DATA_BLOB>(),
            pwszCNGAlgid: ::core::ptr::null::<WCHAR>(),
            pwszCNGExtraAlgid: ::core::ptr::null::<WCHAR>(),
        },
    ]
};

// CRYPT_OID_INFO_HAS_EXTRA_FIELDS (flag-only macro, no value emitted)

// WINE_DEFAULT_DEBUG_CHANNEL(crypt)




#[repr(C)]
pub struct OIDFunctionSet {
    pub name: LPSTR,
    pub cs: CRITICAL_SECTION, /* protects functions */
    pub functions: list,
    pub next: list,
}

#[repr(C)]
pub struct OIDFunction {
    pub hModule: HMODULE,
    pub encoding: DWORD,
    pub entry: CRYPT_OID_FUNC_ENTRY,
    pub next: list,
}

#[repr(C)]
pub struct FuncAddr {
    pub lib: HMODULE,
    pub dllList: LPWSTR,
    pub currentDll: LPWSTR,
}

#[repr(C)]
pub struct OIDInfoConstructor {
    pub dwGroupId: DWORD,
    pub pszOID: LPCSTR,
    pub Algid: UINT,
    pub pwszName: LPCWSTR,
    pub blob: *const CRYPT_DATA_BLOB,
    pub pwszCNGAlgid: *const WCHAR,
    pub pwszCNGExtraAlgid: *const WCHAR,
}
unsafe impl Sync for OIDInfoConstructor {}

#[repr(C)]
pub struct OIDInfo {
    pub info: CRYPT_OID_INFO,
    pub entry: list,
}

unsafe extern "C" fn free_function_sets() {
    let mut setCursor: *mut OIDFunctionSet = core::ptr::null_mut::<OIDFunctionSet>();
    let mut setNext: *mut OIDFunctionSet = core::ptr::null_mut::<OIDFunctionSet>();
    setCursor = (funcSets.next as *mut u8).offset(
        -(core::mem::offset_of!(OIDFunctionSet, next) as isize),
    ) as *mut OIDFunctionSet;
    setNext = ((*setCursor).next.next as *mut u8).offset(
        -(core::mem::offset_of!(OIDFunctionSet, next) as isize),
    ) as *mut OIDFunctionSet;
    while core::ptr::addr_of_mut!((*setCursor).next) != core::ptr::addr_of_mut!(funcSets) {
        let mut functionCursor: *mut OIDFunction = core::ptr::null_mut::<OIDFunction>();
        let mut funcNext: *mut OIDFunction = core::ptr::null_mut::<OIDFunction>();
        list_remove(core::ptr::addr_of_mut!((*setCursor).next));
        CryptMemFree((*setCursor).name as LPVOID);
        functionCursor = ((*setCursor).functions.next as *mut u8).offset(
            -(core::mem::offset_of!(OIDFunction, next) as isize),
        ) as *mut OIDFunction;
        funcNext = ((*functionCursor).next.next as *mut u8).offset(
            -(core::mem::offset_of!(OIDFunction, next) as isize),
        ) as *mut OIDFunction;
        while core::ptr::addr_of_mut!((*functionCursor).next)
            != core::ptr::addr_of_mut!((*setCursor).functions)
        {
            list_remove(core::ptr::addr_of_mut!((*functionCursor).next));
            CryptMemFree(functionCursor as LPVOID);
            functionCursor = funcNext;
            funcNext = ((*functionCursor).next.next as *mut u8).offset(
                -(core::mem::offset_of!(OIDFunction, next) as isize),
            ) as *mut OIDFunction;
        }
        (*(*setCursor).cs.DebugInfo).__bindgen_anon_1.Spare[0] = 0;
        DeleteCriticalSection(core::ptr::addr_of_mut!((*setCursor).cs));
        CryptMemFree(setCursor as LPVOID);
        setCursor = setNext;
        setNext = ((*setCursor).next.next as *mut u8).offset(
            -(core::mem::offset_of!(OIDFunctionSet, next) as isize),
        ) as *mut OIDFunctionSet;
    }
    DeleteCriticalSection(core::ptr::addr_of_mut!(funcSetCS));
}

#[no_mangle]
pub unsafe extern "system" fn CryptInitOIDFunctionSet(
    mut pszFuncName: LPCSTR,
    mut dwFlags: DWORD,
) -> HCRYPTOIDFUNCSET {
    let mut cursor: *mut OIDFunctionSet = ::core::ptr::null_mut::<OIDFunctionSet>();
    let mut ret: *mut OIDFunctionSet = ::core::ptr::null_mut::<OIDFunctionSet>();
    WINE_TRACE!("(%s, %lx)\n", debugstr_a(pszFuncName as *const ::core::ffi::c_char), dwFlags);
    EnterCriticalSection(&raw mut funcSetCS);
    cursor = (funcSets.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(OIDFunctionSet, next) as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut OIDFunctionSet;
    while &raw mut (*cursor).next != &raw mut funcSets {
        if stricmp(
            pszFuncName as *const ::core::ffi::c_char,
            (*cursor).name as *const ::core::ffi::c_char,
        ) == 0
        {
            ret = cursor;
            break;
        } else {
            cursor = ((*cursor).next.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(OIDFunctionSet, next)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut OIDFunctionSet;
        }
    }
    if ret.is_null() {
        ret = CryptMemAlloc(::core::mem::size_of::<OIDFunctionSet>() as ULONG)
            as *mut OIDFunctionSet;
        if !ret.is_null() {
            memset(
                ret as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<OIDFunctionSet>() as size_t,
            );
            (*ret).name = CryptMemAlloc(
                strlen(pszFuncName as *const ::core::ffi::c_char)
                    .wrapping_add(1 as size_t) as ULONG,
            ) as LPSTR;
            if !(*ret).name.is_null() {
                InitializeCriticalSectionEx(
                    &raw mut (*ret).cs,
                    0 as DWORD,
                    RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO as DWORD,
                );
                (*(*ret).cs.DebugInfo).__bindgen_anon_1.Spare[0 as usize] =
                    (concat!(file!(), ": OIDFunctionSet.cs\0").as_ptr()) as DWORD_PTR;
                list_init(&raw mut (*ret).functions);
                strcpy(
                    (*ret).name as *mut ::core::ffi::c_char,
                    pszFuncName as *const ::core::ffi::c_char,
                );
                list_add_tail(&raw mut funcSets, &raw mut (*ret).next);
            } else {
                CryptMemFree(ret as LPVOID);
                ret = ::core::ptr::null_mut::<OIDFunctionSet>();
            }
        }
    }
    LeaveCriticalSection(&raw mut funcSetCS);
    return ret as HCRYPTOIDFUNCSET;
}

unsafe extern "C" fn CRYPT_GetKeyName(
    dwEncodingType: DWORD,
    pszFuncName: LPCSTR,
    pszOID: LPCSTR,
) -> LPSTR {
    static szEncodingTypeFmt: [u8; 59] =
        *b"Software\\Microsoft\\Cryptography\\OID\\EncodingType %ld\\%s\\%s\0";
    let mut len: UINT = 0;
    let mut numericOID: [::core::ffi::c_char; 7] = [0; 7];
    let mut oid: LPCSTR = core::ptr::null();
    let mut szKey: LPSTR = core::ptr::null_mut();

    /* MSDN says the encoding type is a mask, but it isn't treated that way.
     * (E.g., if dwEncodingType were 3, the key names "EncodingType 1" and
     * "EncodingType 2" would be expected if it were a mask.  Instead native
     * stores values in "EncodingType 3".
     */
    if IS_INTOID!(pszOID) != false {
        snprintf(
            core::ptr::addr_of_mut!(numericOID) as *mut ::core::ffi::c_char,
            core::mem::size_of::<[::core::ffi::c_char; 7]>(),
            b"#%d\0".as_ptr() as *const ::core::ffi::c_char,
            LOWORD!(pszOID) as ::core::ffi::c_int,
        );
        oid = core::ptr::addr_of_mut!(numericOID) as *mut ::core::ffi::c_char as LPCSTR;
    } else {
        oid = pszOID;
    }

    /* This is enough: the lengths of the two string parameters are explicitly
     * counted, and we need up to five additional characters for the encoding
     * type.  These are covered by the "%d", "%s", and "%s" characters in the
     * format specifier that are removed by sprintf.
     */
    len = (core::mem::size_of::<[::core::ffi::c_char; 59]>() as UINT)
        .wrapping_add(lstrlenA(pszFuncName) as UINT)
        .wrapping_add(lstrlenA(oid) as UINT);
    szKey = CryptMemAlloc(len as ULONG) as LPSTR;
    if !szKey.is_null() {
        sprintf(
            szKey as *mut ::core::ffi::c_char,
            core::ptr::addr_of!(szEncodingTypeFmt) as *const ::core::ffi::c_char,
            GET_CERT_ENCODING_TYPE!(dwEncodingType) as ::core::ffi::c_long,
            pszFuncName,
            oid,
        );
    }
    return szKey;
}

#[no_mangle]
pub unsafe extern "system" fn CryptGetDefaultOIDDllList(
    hFuncSet: HCRYPTOIDFUNCSET,
    dwEncodingType: DWORD,
    pwszDllList: LPWSTR,
    pcchDllList: *mut DWORD,
) -> BOOL {
    let set: *mut OIDFunctionSet = hFuncSet as *mut OIDFunctionSet;
    let mut keyName: *mut ::core::ffi::c_char = core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut rc: LSTATUS = 0;
    WINE_TRACE!("(%p, %ld, %p, %p)\n", hFuncSet, dwEncodingType, pwszDllList, pcchDllList);
    keyName = CRYPT_GetKeyName(
        dwEncodingType,
        (*set).name as LPCSTR,
        b"DEFAULT\0" as *const u8 as LPCSTR,
    );
    rc = RegCreateKeyExA(
        HKEY_LOCAL_MACHINE,
        keyName as LPCSTR,
        0,
        core::ptr::null_mut::<CHAR>(),
        0,
        KEY_READ,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        core::ptr::addr_of_mut!(key),
        core::ptr::null_mut::<DWORD>(),
    );
    if rc == 0 {
        let mut size: DWORD =
            (*pcchDllList as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD;
        rc = RegQueryValueExW(
            key,
            RUSTCONV_WSTR_6184A7B89A8A.as_ptr(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            pwszDllList as LPBYTE,
            core::ptr::addr_of_mut!(size),
        );
        if rc == 0 {
            *pcchDllList =
                (size as usize).wrapping_div(core::mem::size_of::<WCHAR>() as usize) as DWORD;
        } else {
            /* No value, return an empty list */
            if !pwszDllList.is_null() && *pcchDllList != 0 {
                *pwszDllList = 0 as WCHAR;
            }
            *pcchDllList = 1;
        }
        RegCloseKey(key);
    } else {
        /* No value, return an empty list */
        if !pwszDllList.is_null() && *pcchDllList != 0 {
            *pwszDllList = 0 as WCHAR;
        }
        *pcchDllList = 1;
    }
    CryptMemFree(keyName as LPVOID);

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptInstallOIDFunctionAddress(
    hModule: HMODULE,
    dwEncodingType: DWORD,
    pszFuncName: LPCSTR,
    cFuncEntry: DWORD,
    rgFuncEntry: *const CRYPT_OID_FUNC_ENTRY,
    dwFlags: DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut set: *mut OIDFunctionSet = core::ptr::null_mut::<OIDFunctionSet>();
    WINE_TRACE!("(%p, %ld, %s, %ld, %p, %08lx)\n", hModule, dwEncodingType, debugstr_a(pszFuncName as *const core::ffi::c_char), cFuncEntry, rgFuncEntry, dwFlags);
    set = CryptInitOIDFunctionSet(pszFuncName, 0) as *mut OIDFunctionSet;
    if !set.is_null() {
        let mut i: DWORD = 0;
        EnterCriticalSection(&raw mut (*set).cs);
        i = 0;
        while ret != 0 && i < cFuncEntry {
            let mut func: *mut OIDFunction = core::ptr::null_mut::<OIDFunction>();
            WINE_TRACE!("OID %s, func %p\n", debugstr_a( (*rgFuncEntry.offset(i as isize)).pszOID as *const core::ffi::c_char, ), (*rgFuncEntry.offset(i as isize)).pvFuncAddr);
            if !IS_INTOID!((*rgFuncEntry.offset(i as isize)).pszOID) {
                func = CryptMemAlloc(
                    (core::mem::size_of::<OIDFunction>() as usize)
                        .wrapping_add(strlen(
                            (*rgFuncEntry.offset(i as isize)).pszOID as *const core::ffi::c_char,
                        ) as usize)
                        .wrapping_add(1 as usize) as ULONG,
                ) as *mut OIDFunction;
            } else {
                func = CryptMemAlloc(core::mem::size_of::<OIDFunction>() as ULONG)
                    as *mut OIDFunction;
            }
            if !func.is_null() {
                (*func).encoding = GET_CERT_ENCODING_TYPE!(dwEncodingType);
                if !IS_INTOID!((*rgFuncEntry.offset(i as isize)).pszOID) {
                    let mut oid: LPSTR = core::ptr::null_mut::<CHAR>();
                    oid = (func as LPBYTE)
                        .offset(core::mem::size_of::<OIDFunction>() as usize as isize)
                        as LPSTR;
                    strcpy(
                        oid as *mut core::ffi::c_char,
                        (*rgFuncEntry.offset(i as isize)).pszOID as *const core::ffi::c_char,
                    );
                    (*func).entry.pszOID = oid as LPCSTR;
                } else {
                    (*func).entry.pszOID = (*rgFuncEntry.offset(i as isize)).pszOID;
                }
                (*func).entry.pvFuncAddr = (*rgFuncEntry.offset(i as isize)).pvFuncAddr;
                (*func).hModule = hModule;
                list_add_tail(&raw mut (*set).functions, &raw mut (*func).next);
            } else {
                ret = (FALSE as i32);
            }
            i = i.wrapping_add(1);
        }
        LeaveCriticalSection(&raw mut (*set).cs);
    } else {
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_GetFuncFromReg(
    dwEncodingType: DWORD,
    pszOID: LPCSTR,
    szFuncName: LPCSTR,
    ppvFuncAddr: *mut LPVOID,
    phFuncAddr: *mut HCRYPTOIDFUNCADDR,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut keyName: *mut c_char;
    let mut funcName: *const c_char;
    let mut key: HKEY = core::ptr::null_mut();
    let mut rc: LSTATUS;

    keyName = CRYPT_GetKeyName(dwEncodingType, szFuncName, pszOID);
    rc = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        keyName as LPCSTR,
        0 as DWORD,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(key),
    );
    if rc == 0 {
        let mut type_0: DWORD = 0;
        let mut size: DWORD = 0 as DWORD;

        rc = RegQueryValueExA(
            key,
            b"FuncName\0".as_ptr() as LPCSTR,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(type_0),
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(size),
        );
        if (rc == 0 || rc == ERROR_MORE_DATA as LSTATUS) && type_0 == REG_SZ as DWORD {
            funcName = CryptMemAlloc(size as ULONG) as *const c_char;
            rc = RegQueryValueExA(
                key,
                b"FuncName\0".as_ptr() as LPCSTR,
                core::ptr::null_mut::<DWORD>(),
                core::ptr::addr_of_mut!(type_0),
                funcName as LPBYTE,
                core::ptr::addr_of_mut!(size),
            );
        } else {
            funcName = szFuncName as *const c_char;
        }
        rc = RegQueryValueExW(
            key,
            RUSTCONV_WSTR_6184A7B89A8A.as_ptr(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(type_0),
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(size),
        );
        if (rc == 0 || rc == ERROR_MORE_DATA as LSTATUS) && type_0 == REG_SZ as DWORD {
            let dllName: LPWSTR = CryptMemAlloc(size as ULONG) as LPWSTR;

            if !dllName.is_null() {
                rc = RegQueryValueExW(
                    key,
                    RUSTCONV_WSTR_6184A7B89A8A.as_ptr(),
                    core::ptr::null_mut::<DWORD>(),
                    core::ptr::null_mut::<DWORD>(),
                    dllName as LPBYTE,
                    core::ptr::addr_of_mut!(size),
                );
                if rc == 0 {
                    let mut lib: HMODULE = core::ptr::null_mut();

                    /* This is a bit of a hack; MSDN describes a more
                     * complicated unload routine than this will allow.
                     * Still, this seems to suffice for now.
                     */
                    lib = LoadLibraryW(dllName as LPCWSTR) as HMODULE;
                    if !lib.is_null() {
                        *ppvFuncAddr = GetProcAddress(lib as HINSTANCE, funcName as LPCSTR) as LPVOID;
                        if !(*ppvFuncAddr).is_null() {
                            let addr: *mut FuncAddr =
                                CryptMemAlloc(core::mem::size_of::<FuncAddr>() as ULONG) as *mut FuncAddr;

                            if !addr.is_null() {
                                (*addr).lib = lib;
                                (*addr).currentDll = core::ptr::null_mut::<WCHAR>();
                                (*addr).dllList = (*addr).currentDll;
                                *phFuncAddr = addr as HCRYPTOIDFUNCADDR;
                                ret = (TRUE as i32);
                            } else {
                                *phFuncAddr = core::ptr::null_mut::<core::ffi::c_void>() as HCRYPTOIDFUNCADDR;
                                FreeLibrary(lib);
                            }
                        } else {
                            /* Unload the library, the caller doesn't want
                             * to unload it when the return value is NULL.
                             */
                            FreeLibrary(lib);
                        }
                    }
                } else {
                    SetLastError(rc as DWORD);
                }
                CryptMemFree(dllName as LPVOID);
            }
        } else {
            SetLastError(rc as DWORD);
        }
        if funcName != szFuncName as *const c_char {
            CryptMemFree(funcName as *mut c_char as LPVOID);
        }
        RegCloseKey(key);
    } else {
        SetLastError(rc as DWORD);
    }
    CryptMemFree(keyName as LPVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptGetOIDFunctionAddress(
    hFuncSet: HCRYPTOIDFUNCSET,
    dwEncodingType: DWORD,
    pszOID: LPCSTR,
    dwFlags: DWORD,
    ppvFuncAddr: *mut *mut core::ffi::c_void,
    phFuncAddr: *mut HCRYPTOIDFUNCADDR,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut set: *mut OIDFunctionSet = hFuncSet as *mut OIDFunctionSet;

    TRACE!(
        "(%p, %ld, %s, %08lx, %p, %p)\n",
        hFuncSet,
        dwEncodingType,
        debugstr_a(pszOID as *const core::ffi::c_char),
        dwFlags,
        ppvFuncAddr,
        phFuncAddr
    );

    *ppvFuncAddr = core::ptr::null_mut();
    if (dwFlags & CRYPT_GET_INSTALLED_OID_FUNC_FLAG) == 0 {
        let mut function: *mut OIDFunction = core::ptr::null_mut();

        EnterCriticalSection(core::ptr::addr_of_mut!((*set).cs));
        function = ((*set).functions.next as *mut u8)
            .sub(core::mem::offset_of!(OIDFunction, next)) as *mut OIDFunction;
        while core::ptr::addr_of_mut!((*function).next) != core::ptr::addr_of_mut!((*set).functions) {
            // LIST_FOR_EACH_ENTRY body
            if (*function).encoding == (dwEncodingType & 0xFFFF) { // CERT_ENCODING_TYPE_MASK
                if !IS_INTOID!(pszOID) {
                    if !IS_INTOID!((*function).entry.pszOID)
                        && stricmp(
                            (*function).entry.pszOID as *const core::ffi::c_char,
                            pszOID as *const core::ffi::c_char,
                        ) == 0
                    {
                        *ppvFuncAddr = (*function).entry.pvFuncAddr;
                        *phFuncAddr = core::ptr::null_mut() as HCRYPTOIDFUNCADDR; /* FIXME: what should it be? */
                        ret = (TRUE as i32);
                        break;
                    }
                } else if (*function).entry.pszOID == pszOID {
                    *ppvFuncAddr = (*function).entry.pvFuncAddr;
                    *phFuncAddr = core::ptr::null_mut() as HCRYPTOIDFUNCADDR; /* FIXME: what should it be? */
                    ret = (TRUE as i32);
                    break;
                }
            }
            function = ((*function).next.next as *mut u8)
                .sub(core::mem::offset_of!(OIDFunction, next)) as *mut OIDFunction;
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!((*set).cs));
    }
    if (*ppvFuncAddr).is_null() {
        ret = CRYPT_GetFuncFromReg(
            dwEncodingType,
            pszOID,
            (*set).name as LPCSTR,
            ppvFuncAddr as *mut LPVOID,
            phFuncAddr,
        );
    }

    TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn is_module_registered(hModule: HMODULE) -> BOOL {
    let mut set: *mut OIDFunctionSet = core::ptr::null_mut::<OIDFunctionSet>();
    let mut ret: BOOL = (FALSE as i32);
    EnterCriticalSection(core::ptr::addr_of_mut!(funcSetCS));
    set = ((funcSets.next as *mut u8).sub(core::mem::offset_of!(OIDFunctionSet, next))) as *mut OIDFunctionSet;
    while core::ptr::addr_of_mut!((*set).next) != core::ptr::addr_of_mut!(funcSets) {
        let mut function: *mut OIDFunction = core::ptr::null_mut::<OIDFunction>();
        EnterCriticalSection(core::ptr::addr_of_mut!((*set).cs));
        function = (((*set).functions.next as *mut u8).sub(core::mem::offset_of!(OIDFunction, next))) as *mut OIDFunction;
        while core::ptr::addr_of_mut!((*function).next) != core::ptr::addr_of_mut!((*set).functions) {
            if (*function).hModule == hModule {
                ret = (TRUE as i32);
                break;
            } else {
                function = (((*function).next.next as *mut u8).sub(core::mem::offset_of!(OIDFunction, next))) as *mut OIDFunction;
            }
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!((*set).cs));
        if ret != 0 {
            break;
        }
        set = (((*set).next.next as *mut u8).sub(core::mem::offset_of!(OIDFunctionSet, next))) as *mut OIDFunctionSet;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(funcSetCS));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptFreeOIDFunctionAddress(hFuncAddr: HCRYPTOIDFUNCADDR, dwFlags: DWORD) -> BOOL {
    if !hFuncAddr.is_null() {
        let addr: *mut FuncAddr = hFuncAddr as *mut FuncAddr;

        if is_module_registered((*addr).lib) == 0 {
            CryptMemFree((*addr).dllList as LPVOID);
            FreeLibrary((*addr).lib);
            CryptMemFree(addr as LPVOID);
        }
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn CRYPT_GetFuncFromDll(dll: LPCWSTR, func: LPCSTR, lib: *mut HMODULE, ppvFuncAddr: *mut *mut core::ffi::c_void) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    *lib = LoadLibraryW(dll) as HMODULE;
    if !(*lib).is_null() {
        *ppvFuncAddr = core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(GetProcAddress(*lib, func));
        if !(*ppvFuncAddr).is_null() {
            ret = (TRUE as i32);
        } else {
            FreeLibrary(*lib);
            *lib = core::ptr::null_mut();
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptGetDefaultOIDFunctionAddress(
    hFuncSet: HCRYPTOIDFUNCSET,
    dwEncodingType: DWORD,
    pwszDll: LPCWSTR,
    dwFlags: DWORD,
    ppvFuncAddr: *mut *mut core::ffi::c_void,
    phFuncAddr: *mut HCRYPTOIDFUNCADDR,
) -> BOOL {
    let set: *mut OIDFunctionSet = hFuncSet as *mut OIDFunctionSet;
    let mut ret: BOOL = (FALSE as i32);

    TRACE!(
        "(%p, %ld, %s, %08lx, %p, %p)\n",
        hFuncSet,
        dwEncodingType,
        debugstr_w(pwszDll),
        dwFlags,
        ppvFuncAddr,
        phFuncAddr
    );

    if !pwszDll.is_null() {
        let mut lib: HMODULE = core::ptr::null_mut();

        *phFuncAddr = core::ptr::null_mut();
        ret = CRYPT_GetFuncFromDll(
            pwszDll,
            (*set).name as LPCSTR,
            core::ptr::addr_of_mut!(lib),
            ppvFuncAddr as *mut _,
        );
        if ret != 0 {
            let addr: *mut FuncAddr =
                CryptMemAlloc(core::mem::size_of::<FuncAddr>() as ULONG) as *mut FuncAddr;

            if !addr.is_null() {
                (*addr).lib = lib;
                (*addr).currentDll = core::ptr::null_mut();
                (*addr).dllList = (*addr).currentDll;
                *phFuncAddr = addr as HCRYPTOIDFUNCADDR;
            } else {
                FreeLibrary(lib);
                *ppvFuncAddr = core::ptr::null_mut();
                SetLastError(ERROR_OUTOFMEMORY as DWORD);
                ret = (FALSE as i32);
            }
        } else {
            SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
        }
    } else {
        let mut addr: *mut FuncAddr = *phFuncAddr as *mut FuncAddr;

        if addr.is_null() {
            let mut size: DWORD = 0;

            ret = CryptGetDefaultOIDDllList(
                hFuncSet,
                dwEncodingType,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                let dllList: LPWSTR = CryptMemAlloc(
                    (size as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
                ) as LPWSTR;

                if !dllList.is_null() {
                    ret = CryptGetDefaultOIDDllList(
                        hFuncSet,
                        dwEncodingType,
                        dllList,
                        core::ptr::addr_of_mut!(size),
                    );
                    if ret != 0 {
                        addr = CryptMemAlloc(core::mem::size_of::<FuncAddr>() as ULONG)
                            as *mut FuncAddr;
                        if !addr.is_null() {
                            (*addr).dllList = dllList;
                            (*addr).currentDll = dllList;
                            (*addr).lib = core::ptr::null_mut();
                            *phFuncAddr = addr as HCRYPTOIDFUNCADDR;
                        } else {
                            CryptMemFree(dllList as LPVOID);
                            SetLastError(ERROR_OUTOFMEMORY as DWORD);
                            ret = (FALSE as i32);
                        }
                    }
                } else {
                    SetLastError(ERROR_OUTOFMEMORY as DWORD);
                    ret = (FALSE as i32);
                }
            }
        }
        if !addr.is_null() {
            if *(*addr).currentDll == 0 {
                CryptFreeOIDFunctionAddress(*phFuncAddr, 0 as DWORD);
                SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
                *phFuncAddr = core::ptr::null_mut();
                ret = (FALSE as i32);
            } else {
                /* FIXME: as elsewhere, can't free until DllCanUnloadNow says
                 * it's possible, and should defer unloading for some time to
                 * avoid repeated LoadLibrary/FreeLibrary on the same dll.
                 */
                FreeLibrary((*addr).lib);
                ret = CRYPT_GetFuncFromDll(
                    (*addr).currentDll as LPCWSTR,
                    (*set).name as LPCSTR,
                    core::ptr::addr_of_mut!((*addr).lib),
                    ppvFuncAddr as *mut _,
                );
                if ret != 0 {
                    /* Move past the current DLL */
                    (*addr).currentDll = (*addr).currentDll.offset(
                        (lstrlenW((*addr).currentDll as LPCWSTR)
                            + 1 as i32) as isize,
                    );
                    *phFuncAddr = addr as HCRYPTOIDFUNCADDR;
                } else {
                    CryptFreeOIDFunctionAddress(*phFuncAddr, 0 as DWORD);
                    SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
                    *phFuncAddr = core::ptr::null_mut();
                }
            }
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptRegisterOIDFunction(
    dwEncodingType: DWORD,
    pszFuncName: LPCSTR,
    pszOID: LPCSTR,
    pwszDll: LPCWSTR,
    pszOverrideFuncName: LPCSTR,
) -> BOOL {
    let mut r: LONG = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szKey: LPSTR = core::ptr::null_mut();

    TRACE!(
        "(%lx, %s, %s, %s, %s)\n",
        dwEncodingType,
        debugstr_a(pszFuncName as *const i8),
        debugstr_a(pszOID as *const i8),
        debugstr_w(pwszDll as *const u16),
        debugstr_a(pszOverrideFuncName as *const i8)
    );

    /* Native does nothing pwszDll is NULL */
    if pwszDll.is_null() {
        return (TRUE) as BOOL;
    }

    /* I'm not matching MS bug for bug here, because I doubt any app depends on
     * it:  native "succeeds" if pszFuncName is NULL, but the nonsensical entry
     * it creates would never be used.
     */
    if pszFuncName.is_null() || pszOID.is_null() {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }

    szKey = CRYPT_GetKeyName(dwEncodingType, pszFuncName, pszOID) as LPSTR;

    TRACE!("Key name is %s\n", debugstr_a(szKey as *const i8));

    if szKey.is_null() {
        return (FALSE) as BOOL;
    }

    r = RegCreateKeyA(
        HKEY_LOCAL_MACHINE,
        szKey as LPCSTR,
        core::ptr::addr_of_mut!(hKey),
    ) as LONG;
    CryptMemFree(szKey as LPVOID);

    'error_close_key: {
        if r != ERROR_SUCCESS as LONG {
            break 'error_close_key;
        }

        /* write the values */
        if !pszOverrideFuncName.is_null() {
            r = RegSetValueExA(
                hKey,
                b"FuncName\0" as *const u8 as LPCSTR,
                0 as DWORD,
                REG_SZ as DWORD,
                pszOverrideFuncName as *const BYTE,
                (lstrlenA(pszOverrideFuncName) + 1 as core::ffi::c_int) as DWORD,
            ) as LONG;
            if r != ERROR_SUCCESS as LONG {
                break 'error_close_key;
            }
        }
        r = RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_6184A7B89A8A.as_ptr(),
            0 as DWORD,
            REG_SZ as DWORD,
            pwszDll as *const BYTE,
            ((lstrlenW(pwszDll) + 1 as core::ffi::c_int) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
        ) as LONG;
    }

    RegCloseKey(hKey);

    if r != ERROR_SUCCESS as LONG {
        SetLastError(r as DWORD);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptUnregisterOIDInfo(info: PCCRYPT_OID_INFO) -> BOOL {
    let mut key_name: *mut ::core::ffi::c_char = core::ptr::null_mut();
    let mut root: HKEY = core::ptr::null_mut();
    let mut err: DWORD = 0;

    if info.is_null()
        || (*info).cbSize as usize != core::mem::size_of::<CRYPT_OID_INFO>()
        || (*info).pszOID.is_null()
    {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }

    err = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_6200FA5BD108.as_ptr(),
        0,
        KEY_ALL_ACCESS,
        core::ptr::addr_of_mut!(root),
    ) as DWORD;
    if err != ERROR_SUCCESS {
        SetLastError(err);
        return (FALSE) as BOOL;
    }

    key_name = CryptMemAlloc(
        (strlen((*info).pszOID as *const ::core::ffi::c_char)
            .wrapping_add(16 as usize)) as ULONG,
    ) as *mut ::core::ffi::c_char;
    if !key_name.is_null() {
        sprintf(
            key_name,
            b"%s!%lu\0" as *const u8 as *const ::core::ffi::c_char,
            (*info).pszOID,
            (*info).dwGroupId,
        );
        err = RegDeleteKeyA(root, key_name as LPCSTR) as DWORD;
    } else {
        err = ERROR_OUTOFMEMORY;
    }

    CryptMemFree(key_name as LPVOID);
    RegCloseKey(root);

    if err != 0 {
        SetLastError(err);
    }

    (err == 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn CryptRegisterOIDInfo(info: PCCRYPT_OID_INFO, flags: DWORD) -> BOOL {
    let mut key_name: *mut c_char = core::ptr::null_mut();
    let mut root: HKEY = core::ptr::null_mut();
    let mut key: HKEY = core::ptr::null_mut();
    let mut err: DWORD = 0;

    if info.is_null()
        || (*info).cbSize as usize != core::mem::size_of::<CRYPT_OID_INFO>()
        || (*info).pszOID.is_null()
    {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }

    if (*info).dwGroupId == 0 {
        return (TRUE) as BOOL;
    }

    'done: {
        key_name = CryptMemAlloc(
            (strlen((*info).pszOID as *const c_char) + 16) as ULONG,
        ) as *mut c_char;
        if key_name.is_null() {
            err = ERROR_OUTOFMEMORY;
            break 'done;
        }

        err = (RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_6200FA5BD108.as_ptr(),
            0,
            core::ptr::null_mut(),
            0,
            KEY_ALL_ACCESS,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(root),
            core::ptr::null_mut(),
        )) as u32;
        if err != ERROR_SUCCESS {
            break 'done;
        }

        sprintf(
            key_name,
            b"%s!%lu\0".as_ptr() as *const c_char,
            (*info).pszOID,
            (*info).dwGroupId,
        );
        err = RegCreateKeyA(root, key_name as LPCSTR, core::ptr::addr_of_mut!(key)) as DWORD;
        if err != ERROR_SUCCESS {
            break 'done;
        }

        if flags != 0 {
            err = (RegSetValueExW(
                key,
                RUSTCONV_WSTR_4928BE03934B.as_ptr(),
                0,
                REG_DWORD,
                core::ptr::addr_of!(flags) as *const BYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            )) as u32;
            if err != ERROR_SUCCESS {
                break 'done;
            }
        }

        if !(*info).pwszName.is_null() {
            err = (RegSetValueExW(
                key,
                RUSTCONV_WSTR_76E480513E2C.as_ptr(),
                0,
                REG_SZ,
                (*info).pwszName as *const BYTE,
                ((lstrlenW((*info).pwszName) + 1) as usize
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            )) as u32;
            if err != ERROR_SUCCESS {
                break 'done;
            }
        }

        if (*info).__bindgen_anon_1.Algid != 0 {
            err = (RegSetValueExW(
                key,
                RUSTCONV_WSTR_6F50C773FDBD.as_ptr(),
                0,
                REG_DWORD,
                core::ptr::addr_of!((*info).__bindgen_anon_1.Algid) as *const BYTE,
                core::mem::size_of::<ALG_ID>() as DWORD,
            )) as u32;
            if err != ERROR_SUCCESS {
                break 'done;
            }
        }

        if (*info).ExtraInfo.cbData != 0 && !(*info).ExtraInfo.pbData.is_null() {
            err = (RegSetValueExW(
                key,
                RUSTCONV_WSTR_52A123613C9E.as_ptr(),
                0,
                REG_BINARY,
                (*info).ExtraInfo.pbData,
                (*info).ExtraInfo.cbData,
            )) as u32;
            if err != ERROR_SUCCESS {
                break 'done;
            }
        }

        if !(*info).pwszCNGAlgid.is_null() {
            err = (RegSetValueExW(
                key,
                RUSTCONV_WSTR_8AFF6EE2D972.as_ptr(),
                0,
                REG_SZ,
                (*info).pwszCNGAlgid as *const BYTE,
                ((lstrlenW((*info).pwszCNGAlgid) + 1) as usize
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            )) as u32;
            if err != ERROR_SUCCESS {
                break 'done;
            }
        }

        if !(*info).pwszCNGExtraAlgid.is_null() {
            err = (RegSetValueExW(
                key,
                RUSTCONV_WSTR_541A926BC431.as_ptr(),
                0,
                REG_SZ,
                (*info).pwszCNGExtraAlgid as *const BYTE,
                ((lstrlenW((*info).pwszCNGExtraAlgid) + 1) as usize
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            )) as u32;
            if err != ERROR_SUCCESS {
                break 'done;
            }
        }
    }

    CryptMemFree(key_name as LPVOID);
    if !key.is_null() {
        RegCloseKey(key);
    }
    if !root.is_null() {
        RegCloseKey(root);
    }

    if err != 0 {
        SetLastError(err);
    }

    return (err == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptUnregisterOIDFunction(
    dwEncodingType: DWORD,
    pszFuncName: LPCSTR,
    pszOID: LPCSTR,
) -> BOOL {
    let mut szKey: LPSTR = core::ptr::null_mut();
    let mut rc: LONG = 0;
    WINE_TRACE!("%lx %s %s\n", dwEncodingType, debugstr_a(pszFuncName as *const core::ffi::c_char), debugstr_a(pszOID as *const core::ffi::c_char));
    if pszFuncName.is_null() || pszOID.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    szKey = CRYPT_GetKeyName(dwEncodingType, pszFuncName, pszOID) as LPSTR;
    rc = RegDeleteKeyA(HKEY_LOCAL_MACHINE, szKey as LPCSTR) as LONG;
    CryptMemFree(szKey as LPVOID);
    if rc != 0 {
        SetLastError(rc as DWORD);
    }
    return (rc == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptGetOIDFunctionValue(
    mut dwEncodingType: DWORD,
    mut pszFuncName: LPCSTR,
    mut pszOID: LPCSTR,
    mut pwszValueName: LPCWSTR,
    mut pdwValueType: *mut DWORD,
    mut pbValueData: *mut BYTE,
    mut pcbValueData: *mut DWORD,
) -> BOOL {
    let mut szKey: LPSTR = core::ptr::null_mut::<CHAR>();
    let mut rc: LONG = 0;
    let mut hKey: HKEY = core::ptr::null_mut::<HKEY__>();
    WINE_TRACE!("%lx %s %s %s %p %p %p\n", dwEncodingType, debugstr_a(pszFuncName as *const ::core::ffi::c_char), debugstr_a(pszOID as *const ::core::ffi::c_char), debugstr_w(pwszValueName as *const WCHAR), pdwValueType, pbValueData, pcbValueData);
    if GET_CERT_ENCODING_TYPE!(dwEncodingType) == 0 {
        return (TRUE) as BOOL;
    }
    if pszFuncName.is_null() || pszOID.is_null() || pwszValueName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    szKey = CRYPT_GetKeyName(dwEncodingType, pszFuncName, pszOID) as LPSTR;
    rc = RegOpenKeyA(
        HKEY_LOCAL_MACHINE,
        szKey as LPCSTR,
        core::ptr::addr_of_mut!(hKey),
    ) as LONG;
    CryptMemFree(szKey as LPVOID);
    if rc != 0 {
        SetLastError(rc as DWORD);
    } else {
        rc = RegQueryValueExW(
            hKey,
            pwszValueName,
            core::ptr::null_mut::<DWORD>(),
            pdwValueType as LPDWORD,
            pbValueData as LPBYTE,
            pcbValueData as LPDWORD,
        ) as LONG;
        if rc != 0 {
            SetLastError(rc as DWORD);
        }
        RegCloseKey(hKey);
    }
    return (rc == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSetOIDFunctionValue(
    dwEncodingType: DWORD,
    pszFuncName: LPCSTR,
    pszOID: LPCSTR,
    pwszValueName: LPCWSTR,
    dwValueType: DWORD,
    pbValueData: *const BYTE,
    cbValueData: DWORD,
) -> BOOL {
    let mut szKey: LPSTR = core::ptr::null_mut();
    let mut rc: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();

    TRACE!(
        "{:x} {} {} {} {} {:p} {}\n",
        dwEncodingType,
        debugstr_a(pszFuncName as *const i8),
        debugstr_a(pszOID as *const i8),
        debugstr_w(pwszValueName as *const u16),
        dwValueType,
        pbValueData,
        cbValueData
    );

    if GET_CERT_ENCODING_TYPE!(dwEncodingType) == 0 {
        return (TRUE) as BOOL;
    }

    if pszFuncName.is_null() || pszOID.is_null() || pwszValueName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    szKey = CRYPT_GetKeyName(dwEncodingType, pszFuncName, pszOID) as LPSTR;
    rc = RegOpenKeyA(
        HKEY_LOCAL_MACHINE,
        szKey as LPCSTR,
        core::ptr::addr_of_mut!(hKey),
    ) as LONG;
    CryptMemFree(szKey as LPVOID);
    if rc != 0 {
        SetLastError(rc as DWORD);
    } else {
        rc = RegSetValueExW(
            hKey,
            pwszValueName,
            0,
            dwValueType,
            pbValueData,
            cbValueData,
        ) as LONG;
        if rc != 0 {
            SetLastError(rc as DWORD);
        }
        RegCloseKey(hKey);
    }
    return (rc == 0) as BOOL;
}

unsafe extern "C" fn CRYPT_FindStringInMultiString(multi: LPCWSTR, toFind: LPCWSTR) -> LPCWSTR {
    let mut ret: LPCWSTR = core::ptr::null();
    let mut ptr: LPCWSTR = multi;
    while !ptr.is_null() && *ptr != 0 && ret.is_null() {
        if lstrcmpiW(ptr, toFind) == 0 {
            ret = ptr;
        }
        ptr = ptr.offset((lstrlenW(ptr) + 1) as isize);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_GetMultiStringCharacterLen(multi: LPCWSTR) -> DWORD {
    let mut ret: DWORD;
    if !multi.is_null() {
        let mut ptr: LPCWSTR;
        /* Count terminating empty string */
        ret = 1 as DWORD;
        ptr = multi;
        while *ptr != 0 {
            ret = ret.wrapping_add((lstrlenW(ptr) + 1) as DWORD);
            ptr = ptr.offset((lstrlenW(ptr) + 1) as isize);
        }
    } else {
        ret = 0 as DWORD;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AddStringToMultiString(mut multi: LPWSTR, mut toAdd: LPCWSTR, mut index: DWORD) -> LPWSTR {
    let mut ret: LPWSTR = core::ptr::null_mut();
    if multi.is_null() {
        /* FIXME: ignoring index, is that okay? */
        ret = CryptMemAlloc(
            ((lstrlenW(toAdd) + 2) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
        ) as LPWSTR;
        if !ret.is_null() {
            /* copy string, including NULL terminator */
            memcpy(
                ret as *mut core::ffi::c_void,
                toAdd as *const core::ffi::c_void,
                ((lstrlenW(toAdd) + 1) as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>()),
            );
            /* add terminating empty string */
            *ret.offset((lstrlenW(toAdd) + 1) as isize) =
                0 as WCHAR;
        }
    } else {
        let mut len: DWORD = CRYPT_GetMultiStringCharacterLen(multi as LPCWSTR);

        ret = CryptMemRealloc(
            multi as LPVOID,
            ((len.wrapping_add(lstrlenW(toAdd) as DWORD))
                .wrapping_add(1 as DWORD) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
        ) as LPWSTR;
        if !ret.is_null() {
            let mut spotToAdd: LPWSTR = core::ptr::null_mut();

            if index == CRYPT_REGISTER_LAST_INDEX as DWORD {
                spotToAdd = ret.offset(len as isize).offset(-(1 as isize));
            } else {
                let mut i: DWORD = 0;

                /* FIXME: if index is too large for the string, toAdd is
                 * added to the end.  Is that okay?
                 */
                i = 0 as DWORD;
                spotToAdd = ret;
                while i < index && *spotToAdd != 0 {
                    spotToAdd = spotToAdd.offset(
                        (lstrlenW(spotToAdd as LPCWSTR) + 1 as core::ffi::c_int)
                            as isize,
                    );
                }
            }
            if !spotToAdd.is_null() {
                /* Copy existing string "right" */
                memmove(
                    spotToAdd.offset(
                        (lstrlenW(toAdd) + 1 as core::ffi::c_int)
                            as isize,
                    ) as *mut core::ffi::c_void,
                    spotToAdd as *const core::ffi::c_void,
                    ((len as core::ffi::c_longlong
                        - spotToAdd.offset_from(ret) as core::ffi::c_longlong)
                        as usize)
                        .wrapping_mul(core::mem::size_of::<WCHAR>()),
                );
                /* Copy new string */
                memcpy(
                    spotToAdd as *mut core::ffi::c_void,
                    toAdd as *const core::ffi::c_void,
                    ((lstrlenW(toAdd) + 1 as core::ffi::c_int)
                        as usize)
                        .wrapping_mul(core::mem::size_of::<WCHAR>()),
                );
            } else {
                CryptMemFree(ret as LPVOID);
                ret = core::ptr::null_mut();
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_RemoveStringFromMultiString(multi: LPWSTR, toRemove: LPCWSTR) -> BOOL {
    let mut spotToRemove: LPWSTR =
        CRYPT_FindStringInMultiString(multi as LPCWSTR, toRemove) as LPWSTR;
    let mut ret: BOOL;

    if !spotToRemove.is_null() {
        let mut len: DWORD = CRYPT_GetMultiStringCharacterLen(multi as LPCWSTR);

        if spotToRemove
            .offset(lstrlenW(toRemove) as isize)
            .offset(2 as isize)
            >= multi.offset(len as isize)
        {
            /* Removing last string in list, terminate multi string directly */
            *spotToRemove = 0 as WCHAR;
            *spotToRemove.offset(1 as isize) = 0 as WCHAR;
        } else {
            let mut nextStr: LPCWSTR = spotToRemove
                .offset(lstrlenW(toRemove) as isize)
                .offset(1 as isize) as LPCWSTR;

            /* Copy remainder of string "left" */
            memmove(
                spotToRemove as *mut core::ffi::c_void,
                nextStr as *const core::ffi::c_void,
                ((len as isize - nextStr.offset_from(multi)) as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>()),
            );
        }
        ret = TRUE as BOOL;
    } else {
        SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_GetDefaultOIDKey(dwEncodingType: DWORD, pszFuncName: LPCSTR, key: PHKEY) -> BOOL {
    let mut keyName: LPSTR;
    let mut r: LONG;

    keyName = CRYPT_GetKeyName(dwEncodingType, pszFuncName, b"DEFAULT\0".as_ptr() as LPCSTR) as LPSTR;

    if keyName.is_null() {
        return (FALSE) as BOOL;
    }

    r = RegCreateKeyExA(
        HKEY_LOCAL_MACHINE,
        keyName as LPCSTR,
        0 as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        KEY_ALL_ACCESS as REGSAM,
        core::ptr::null_mut(),
        key,
        core::ptr::null_mut(),
    ) as LONG;
    CryptMemFree(keyName as LPVOID);
    if r != ERROR_SUCCESS as LONG {
        SetLastError(r as DWORD);
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn CRYPT_GetDefaultOIDDlls(key: HKEY) -> LPWSTR {
    let mut r: LONG;
    let mut type_: DWORD = core::mem::zeroed();
    let mut size: DWORD = core::mem::zeroed();
    let mut dlls: LPWSTR;

    r = RegQueryValueExW(key, RUSTCONV_WSTR_6184A7B89A8A.as_ptr(), core::ptr::null_mut(), &mut type_ as *mut DWORD, core::ptr::null_mut(), &mut size as *mut DWORD) as LONG;
    if r == ERROR_SUCCESS as LONG && type_ == REG_MULTI_SZ as DWORD {
        dlls = CryptMemAlloc(size as _) as LPWSTR;
        r = RegQueryValueExW(key, RUSTCONV_WSTR_6184A7B89A8A.as_ptr(), core::ptr::null_mut(), &mut type_ as *mut DWORD, dlls as LPBYTE, &mut size as *mut DWORD) as LONG;
        if r != ERROR_SUCCESS as LONG {
            CryptMemFree(dlls as LPVOID);
            dlls = core::ptr::null_mut();
        }
    } else {
        dlls = core::ptr::null_mut();
    }
    return dlls;
}

unsafe extern "C" fn CRYPT_SetDefaultOIDDlls(key: HKEY, dlls: LPCWSTR) -> BOOL {
    let mut len: DWORD = CRYPT_GetMultiStringCharacterLen(dlls);
    let mut r: LONG = 0;

    r = RegSetValueExW(key, RUSTCONV_WSTR_6184A7B89A8A.as_ptr(), 0, REG_MULTI_SZ, dlls as *const BYTE, (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD) as LONG;
    if r != 0 {
        SetLastError(r as DWORD);
    }
    return (r == ERROR_SUCCESS as LONG) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptRegisterDefaultOIDFunction(
    dwEncodingType: DWORD,
    pszFuncName: LPCSTR,
    dwIndex: DWORD,
    pwszDll: LPCWSTR,
) -> BOOL {
    let mut key: HKEY = core::ptr::null_mut();
    let mut dlls: LPWSTR = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    if pwszDll.is_null() {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }

    if CRYPT_GetDefaultOIDKey(dwEncodingType, pszFuncName, core::ptr::addr_of_mut!(key)) == 0 {
        return (FALSE) as BOOL;
    }

    dlls = CRYPT_GetDefaultOIDDlls(key);
    if !CRYPT_FindStringInMultiString(dlls as LPCWSTR, pwszDll).is_null() {
        SetLastError(ERROR_FILE_EXISTS as DWORD);
    } else {
        dlls = CRYPT_AddStringToMultiString(dlls, pwszDll, dwIndex);
        if !dlls.is_null() {
            ret = CRYPT_SetDefaultOIDDlls(key, dlls as LPCWSTR);
        }
    }
    CryptMemFree(dlls as LPVOID);
    RegCloseKey(key);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptUnregisterDefaultOIDFunction(dwEncodingType: DWORD, pszFuncName: LPCSTR, pwszDll: LPCWSTR) -> BOOL {
    let mut key: HKEY = core::ptr::null_mut();
    let mut dlls: LPWSTR = core::ptr::null_mut();
    let mut ret: BOOL = 0;

    if pwszDll.is_null() {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    if CRYPT_GetDefaultOIDKey(dwEncodingType, pszFuncName, core::ptr::addr_of_mut!(key)) == 0 {
        return (FALSE) as BOOL;
    }
    dlls = CRYPT_GetDefaultOIDDlls(key);
    ret = CRYPT_RemoveStringFromMultiString(dlls, pwszDll);
    if ret != 0 {
        ret = CRYPT_SetDefaultOIDDlls(key, dlls as LPCWSTR);
    }
    CryptMemFree(dlls as LPVOID);
    RegCloseKey(key);
    return ret;
}

unsafe extern "C" fn oid_init_localizednames() {
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(LocalizedKeys) as usize {
        LoadStringW(
            hInstance,
            (IDS_LOCALIZEDNAME_ROOT as UINT).wrapping_add(i as UINT),
            (core::ptr::addr_of_mut!(LocalizedNames) as LPWSTR).add(i as usize * 256),
            256 as ::core::ffi::c_int,
        );
        i = i.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CryptFindLocalizedName(pwszCryptName: LPCWSTR) -> LPCWSTR {
    let mut i: ::core::ffi::c_uint;

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(LocalizedKeys) as usize {
        if lstrcmpiW(LocalizedKeys[i as usize], pwszCryptName) == 0 {
            return (core::ptr::addr_of_mut!(LocalizedNames) as *mut [WCHAR; 256])
                .offset(i as isize) as *mut WCHAR as LPCWSTR;
        }
        i = i.wrapping_add(1);
    }

    return core::ptr::null::<WCHAR>();
}

unsafe extern "C" fn read_oid_info(
    mut root: HKEY,
    mut key_name: *mut ::core::ffi::c_char,
    mut flags: *mut DWORD,
) -> *mut OIDInfo {
    let mut key: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut len: DWORD = 0;
    let mut oid_len: DWORD = 0;
    let mut name_len: DWORD = 0;
    let mut extra_len: DWORD = 0;
    let mut cngalgid_len: DWORD = 0;
    let mut cngextra_len: DWORD = 0;
    let mut group_id: DWORD = 0;
    let mut info: *mut OIDInfo = ::core::ptr::null_mut::<OIDInfo>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();

    if RegOpenKeyExA(
        root,
        key_name as LPCSTR,
        0,
        KEY_READ,
        &raw mut key,
    ) != 0
    {
        return ::core::ptr::null_mut::<OIDInfo>();
    }

    p = strchr(key_name, '!' as i32);
    if !p.is_null() {
        group_id = strtol(
            p.offset(1 as isize),
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            10,
        ) as DWORD;
        *p = 0;
    }

    oid_len = strlen(key_name).wrapping_add(1 as size_t) as DWORD;

    RegQueryValueExW(
        key,
        RUSTCONV_WSTR_76E480513E2C.as_ptr(),
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<BYTE>(),
        &raw mut name_len,
    );
    RegQueryValueExW(
        key,
        RUSTCONV_WSTR_52A123613C9E.as_ptr(),
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<BYTE>(),
        &raw mut extra_len,
    );
    RegQueryValueExW(
        key,
        RUSTCONV_WSTR_8AFF6EE2D972.as_ptr(),
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<BYTE>(),
        &raw mut cngalgid_len,
    );
    RegQueryValueExW(
        key,
        RUSTCONV_WSTR_541A926BC431.as_ptr(),
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<BYTE>(),
        &raw mut cngextra_len,
    );

    info = CryptMemAlloc(
        (::core::mem::size_of::<OIDInfo>() as usize)
            .wrapping_add(oid_len as usize)
            .wrapping_add(name_len as usize)
            .wrapping_add(extra_len as usize)
            .wrapping_add(cngalgid_len as usize)
            .wrapping_add(cngextra_len as usize) as ULONG,
    ) as *mut OIDInfo;
    if !info.is_null() {
        *flags = 0;
        len = ::core::mem::size_of::<DWORD>() as DWORD;
        RegQueryValueExW(
            key,
            RUSTCONV_WSTR_4928BE03934B.as_ptr(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            flags as LPBYTE,
            &raw mut len,
        );

        memset(
            info as *mut ::core::ffi::c_void,
            0,
            ::core::mem::size_of::<OIDInfo>() as size_t,
        );
        (*info).info.cbSize = ::core::mem::size_of::<CRYPT_OID_INFO>() as DWORD;

        p = info.offset(1 as isize) as *mut ::core::ffi::c_char;

        (*info).info.pszOID = p as LPCSTR;
        strcpy((*info).info.pszOID as *mut ::core::ffi::c_char, key_name);
        p = p.offset(oid_len as isize);

        if name_len != 0 {
            (*info).info.pwszName = p as *mut WCHAR as LPCWSTR;
            RegQueryValueExW(
                key,
                RUSTCONV_WSTR_76E480513E2C.as_ptr(),
                ::core::ptr::null_mut::<DWORD>(),
                ::core::ptr::null_mut::<DWORD>(),
                (*info).info.pwszName as LPBYTE,
                &raw mut name_len,
            );
            p = p.offset(name_len as isize);
        }

        (*info).info.dwGroupId = group_id;

        len = ::core::mem::size_of::<ALG_ID>() as DWORD;
        RegQueryValueExW(
            key,
            RUSTCONV_WSTR_6F50C773FDBD.as_ptr(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            &raw mut (*info).info.__bindgen_anon_1.Algid as LPBYTE,
            &raw mut len,
        );

        if extra_len != 0 {
            (*info).info.ExtraInfo.cbData = extra_len;
            (*info).info.ExtraInfo.pbData = p as *mut BYTE;
            RegQueryValueExW(
                key,
                RUSTCONV_WSTR_52A123613C9E.as_ptr(),
                ::core::ptr::null_mut::<DWORD>(),
                ::core::ptr::null_mut::<DWORD>(),
                (*info).info.ExtraInfo.pbData as LPBYTE,
                &raw mut extra_len,
            );
            p = p.offset(extra_len as isize);
        }

        if cngalgid_len != 0 {
            (*info).info.pwszCNGAlgid = p as *mut WCHAR as LPCWSTR;
            RegQueryValueExW(
                key,
                RUSTCONV_WSTR_8AFF6EE2D972.as_ptr(),
                ::core::ptr::null_mut::<DWORD>(),
                ::core::ptr::null_mut::<DWORD>(),
                (*info).info.pwszCNGAlgid as LPBYTE,
                &raw mut cngalgid_len,
            );
            p = p.offset(cngalgid_len as isize);
        }

        if cngextra_len != 0 {
            (*info).info.pwszCNGExtraAlgid = p as *mut WCHAR as LPCWSTR;
            RegQueryValueExW(
                key,
                RUSTCONV_WSTR_541A926BC431.as_ptr(),
                ::core::ptr::null_mut::<DWORD>(),
                ::core::ptr::null_mut::<DWORD>(),
                (*info).info.pwszCNGExtraAlgid as LPBYTE,
                &raw mut cngalgid_len,
            );
        }
    }

    RegCloseKey(key);

    return info;
}

unsafe extern "C" fn init_registered_oid_info() {
    let mut err: DWORD = 0;
    let mut idx: DWORD = 0;
    let mut root: HKEY = core::ptr::null_mut();

    err = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_6200FA5BD108.as_ptr(),
        0,
        KEY_ALL_ACCESS,
        core::ptr::addr_of_mut!(root),
    ) as DWORD;
    if err != ERROR_SUCCESS {
        return;
    }

    idx = 0;
    loop {
        let mut key_name: [core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut info: *mut OIDInfo = core::ptr::null_mut();
        let mut flags: DWORD = 0;

        let fresh_idx = idx;
        idx = idx.wrapping_add(1);
        err = RegEnumKeyA(
            root,
            fresh_idx,
            core::ptr::addr_of_mut!(key_name) as LPSTR,
            MAX_PATH,
        ) as DWORD;
        if err == ERROR_NO_MORE_ITEMS {
            break;
        }

        if err == ERROR_SUCCESS {
            info = read_oid_info(
                root,
                core::ptr::addr_of_mut!(key_name) as *mut core::ffi::c_char,
                core::ptr::addr_of_mut!(flags),
            );
            if !info.is_null() {
                WINE_TRACE!("adding oid %s, name %s, groupid %lu, algid %u, extra %lu, CNG algid %s, CNG extra %s\n", debugstr_a((*info).info.pszOID as *const core::ffi::c_char), debugstr_w((*info).info.pwszName as *const WCHAR), (*info).info.dwGroupId, (*info).info.__bindgen_anon_1.Algid, (*info).info.ExtraInfo.cbData, debugstr_w((*info).info.pwszCNGAlgid as *const WCHAR), debugstr_w((*info).info.pwszCNGExtraAlgid as *const WCHAR));
                if (flags & CRYPT_INSTALL_OID_INFO_BEFORE_FLAG) != 0 {
                    list_add_head(
                        core::ptr::addr_of_mut!(oidInfo),
                        core::ptr::addr_of_mut!((*info).entry),
                    );
                } else {
                    list_add_tail(
                        core::ptr::addr_of_mut!(oidInfo),
                        core::ptr::addr_of_mut!((*info).entry),
                    );
                }
            }
        }
    }

    RegCloseKey(root);
}

unsafe extern "C" fn init_oid_info() {
    let mut i: DWORD = 0;

    oid_init_localizednames();
    i = 0;
    while (i as usize) < ARRAY_SIZE!(oidInfoConstructors) as usize {
        if IS_INTRESOURCE!(oidInfoConstructors[i as usize].pwszName) == false {
            let mut info: *mut OIDInfo = core::ptr::null_mut::<OIDInfo>();

            /* The name is a static string, so just use the same pointer */
            info = CryptMemAlloc(core::mem::size_of::<OIDInfo>() as ULONG) as *mut OIDInfo;
            if !info.is_null() {
                memset(
                    info as *mut core::ffi::c_void,
                    0,
                    core::mem::size_of::<OIDInfo>() as size_t,
                );
                (*info).info.cbSize = core::mem::size_of::<CRYPT_OID_INFO>() as DWORD;
                (*info).info.pszOID = oidInfoConstructors[i as usize].pszOID;
                (*info).info.pwszName = oidInfoConstructors[i as usize].pwszName;
                (*info).info.dwGroupId = oidInfoConstructors[i as usize].dwGroupId;
                (*info).info.__bindgen_anon_1.Algid = oidInfoConstructors[i as usize].Algid as ALG_ID;
                if !oidInfoConstructors[i as usize].blob.is_null() {
                    (*info).info.ExtraInfo.cbData = (*oidInfoConstructors[i as usize].blob).cbData;
                    (*info).info.ExtraInfo.pbData = (*oidInfoConstructors[i as usize].blob).pbData;
                }
                (*info).info.pwszCNGAlgid = oidInfoConstructors[i as usize].pwszCNGAlgid as LPCWSTR;
                (*info).info.pwszCNGExtraAlgid =
                    oidInfoConstructors[i as usize].pwszCNGExtraAlgid as LPCWSTR;
                list_add_tail(
                    core::ptr::addr_of_mut!(oidInfo),
                    core::ptr::addr_of_mut!((*info).entry),
                );
            }
        } else {
            let mut stringresource: LPCWSTR = core::ptr::null::<WCHAR>();
            let mut len: core::ffi::c_int = LoadStringW(
                hInstance,
                oidInfoConstructors[i as usize].pwszName as UINT_PTR as UINT,
                core::ptr::addr_of_mut!(stringresource) as LPWSTR,
                0,
            );

            if len != 0 {
                let info: *mut OIDInfo = CryptMemAlloc(
                    (core::mem::size_of::<OIDInfo>() as usize).wrapping_add(
                        ((len + 1) as usize)
                            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
                    ) as ULONG,
                ) as *mut OIDInfo;

                if !info.is_null() {
                    memset(
                        info as *mut core::ffi::c_void,
                        0,
                        core::mem::size_of::<OIDInfo>() as size_t,
                    );
                    (*info).info.cbSize = core::mem::size_of::<CRYPT_OID_INFO>() as DWORD;
                    (*info).info.pszOID = oidInfoConstructors[i as usize].pszOID;
                    (*info).info.pwszName =
                        info.offset(1 as isize) as LPWSTR as LPCWSTR;
                    (*info).info.dwGroupId = oidInfoConstructors[i as usize].dwGroupId;
                    (*info).info.__bindgen_anon_1.Algid = oidInfoConstructors[i as usize].Algid as ALG_ID;
                    memcpy(
                        info.offset(1 as isize) as *mut core::ffi::c_void,
                        stringresource as *const core::ffi::c_void,
                        (len as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
                    );
                    *(info.offset(1 as isize) as LPWSTR).offset(len as isize) =
                        0 as WCHAR;
                    if !oidInfoConstructors[i as usize].blob.is_null() {
                        (*info).info.ExtraInfo.cbData = (*oidInfoConstructors[i as usize].blob).cbData;
                        (*info).info.ExtraInfo.pbData = (*oidInfoConstructors[i as usize].blob).pbData;
                    }
                    (*info).info.pwszCNGAlgid = oidInfoConstructors[i as usize].pwszCNGAlgid as LPCWSTR;
                    (*info).info.pwszCNGExtraAlgid =
                        oidInfoConstructors[i as usize].pwszCNGExtraAlgid as LPCWSTR;
                    list_add_tail(
                        core::ptr::addr_of_mut!(oidInfo),
                        core::ptr::addr_of_mut!((*info).entry),
                    );
                }
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn free_oid_info() {
    let mut info: *mut OIDInfo = core::ptr::null_mut();
    let mut next: *mut OIDInfo = core::ptr::null_mut();
    info = ((*core::ptr::addr_of_mut!(oidInfo)).next as *mut u8).sub(core::mem::offset_of!(OIDInfo, entry)) as *mut OIDInfo;
    next = (((*info).entry.next) as *mut u8).sub(core::mem::offset_of!(OIDInfo, entry)) as *mut OIDInfo;
    while core::ptr::addr_of_mut!((*info).entry) != core::ptr::addr_of_mut!(oidInfo) {
        list_remove(core::ptr::addr_of_mut!((*info).entry));
        CryptMemFree((info).cast());
        info = next;
        next = (((*info).entry.next) as *mut u8).sub(core::mem::offset_of!(OIDInfo, entry)) as *mut OIDInfo;
    }
    DeleteCriticalSection(core::ptr::addr_of_mut!(oidInfoCS));
}

#[no_mangle]
pub unsafe extern "system" fn CryptEnumOIDInfo(
    dwGroupId: DWORD,
    dwFlags: DWORD,
    pvArg: LPVOID,
    pfnEnumOIDInfo: PFN_CRYPT_ENUM_OID_INFO,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut info: *mut OIDInfo = core::ptr::null_mut::<OIDInfo>();

    EnterCriticalSection(core::ptr::addr_of_mut!(oidInfoCS));
    info = ((*core::ptr::addr_of_mut!(oidInfo)).next as *mut u8)
        .offset(-(core::mem::offset_of!(OIDInfo, entry) as isize)) as *mut OIDInfo;
    while core::ptr::addr_of_mut!((*info).entry) != core::ptr::addr_of_mut!(oidInfo) {
        if dwGroupId == 0 || dwGroupId == (*info).info.dwGroupId {
            ret = (pfnEnumOIDInfo.unwrap_unchecked())(
                core::ptr::addr_of_mut!((*info).info) as PCCRYPT_OID_INFO,
                pvArg,
            );
            if ret == 0 {
                break;
            }
        }
        info = ((*info).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(OIDInfo, entry) as isize)) as *mut OIDInfo;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(oidInfoCS));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptFindOIDInfo(
    mut dwKeyType: DWORD,
    mut pvKey: *mut ::core::ffi::c_void,
    mut dwGroupId: DWORD,
) -> PCCRYPT_OID_INFO {
    let mut ret: PCCRYPT_OID_INFO = ::core::ptr::null::<CRYPT_OID_INFO>();
    WINE_TRACE!("(%ld, %p, %ld)\n", dwKeyType, pvKey, dwGroupId);
    match dwKeyType {
        CRYPT_OID_INFO_ALGID_KEY => {
            let mut info: *mut OIDInfo = ::core::ptr::null_mut::<OIDInfo>();
            WINE_TRACE!("CRYPT_OID_INFO_ALGID_KEY: %ld\n", *(pvKey as *mut DWORD));
            EnterCriticalSection(&raw mut oidInfoCS);
            info = (oidInfo.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(OIDInfo, entry) as ::core::ffi::c_ulonglong
                    as isize),
            ) as *mut OIDInfo;
            while &raw mut (*info).entry != &raw mut oidInfo {
                if (*info).info.__bindgen_anon_1.Algid as DWORD == *(pvKey as *mut DWORD)
                    && (dwGroupId == 0 || (*info).info.dwGroupId == dwGroupId)
                {
                    ret = &raw mut (*info).info as PCCRYPT_OID_INFO;
                    break;
                } else {
                    info = ((*info).entry.next as *mut ::core::ffi::c_char).offset(
                        -(core::mem::offset_of!(OIDInfo, entry)
                            as ::core::ffi::c_ulonglong as isize),
                    ) as *mut OIDInfo;
                }
            }
            LeaveCriticalSection(&raw mut oidInfoCS);
        }
        CRYPT_OID_INFO_NAME_KEY => {
            let mut info_0: *mut OIDInfo = ::core::ptr::null_mut::<OIDInfo>();
            WINE_TRACE!("CRYPT_OID_INFO_NAME_KEY: %s\n", debugstr_w(pvKey as *const WCHAR));
            EnterCriticalSection(&raw mut oidInfoCS);
            info_0 = (oidInfo.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(OIDInfo, entry) as ::core::ffi::c_ulonglong
                    as isize),
            ) as *mut OIDInfo;
            while &raw mut (*info_0).entry != &raw mut oidInfo {
                if wcscmp(
                    (*info_0).info.pwszName as *const wchar_t,
                    pvKey as *const wchar_t,
                ) == 0
                    && (dwGroupId == 0 || (*info_0).info.dwGroupId == dwGroupId)
                {
                    ret = &raw mut (*info_0).info as PCCRYPT_OID_INFO;
                    break;
                } else {
                    info_0 = ((*info_0).entry.next as *mut ::core::ffi::c_char).offset(
                        -(core::mem::offset_of!(OIDInfo, entry)
                            as ::core::ffi::c_ulonglong as isize),
                    ) as *mut OIDInfo;
                }
            }
            LeaveCriticalSection(&raw mut oidInfoCS);
        }
        CRYPT_OID_INFO_OID_KEY => {
            let mut info_1: *mut OIDInfo = ::core::ptr::null_mut::<OIDInfo>();
            let mut oid: LPSTR = pvKey as LPSTR;
            WINE_TRACE!("CRYPT_OID_INFO_OID_KEY: %s\n", debugstr_a(oid as *const ::core::ffi::c_char));
            EnterCriticalSection(&raw mut oidInfoCS);
            info_1 = (oidInfo.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(OIDInfo, entry) as ::core::ffi::c_ulonglong
                    as isize),
            ) as *mut OIDInfo;
            while &raw mut (*info_1).entry != &raw mut oidInfo {
                if lstrcmpA((*info_1).info.pszOID, oid as LPCSTR) == 0
                    && (dwGroupId == 0 || (*info_1).info.dwGroupId == dwGroupId)
                {
                    ret = &raw mut (*info_1).info as PCCRYPT_OID_INFO;
                    break;
                } else {
                    info_1 = ((*info_1).entry.next as *mut ::core::ffi::c_char).offset(
                        -(core::mem::offset_of!(OIDInfo, entry)
                            as ::core::ffi::c_ulonglong as isize),
                    ) as *mut OIDInfo;
                }
            }
            LeaveCriticalSection(&raw mut oidInfoCS);
        }
        CRYPT_OID_INFO_SIGN_KEY => {
            let mut info_2: *mut OIDInfo = ::core::ptr::null_mut::<OIDInfo>();
            WINE_TRACE!("CRYPT_OID_INFO_SIGN_KEY: %ld\n", *(pvKey as *mut DWORD));
            EnterCriticalSection(&raw mut oidInfoCS);
            info_2 = (oidInfo.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(OIDInfo, entry) as ::core::ffi::c_ulonglong
                    as isize),
            ) as *mut OIDInfo;
            while &raw mut (*info_2).entry != &raw mut oidInfo {
                if (*info_2).info.__bindgen_anon_1.Algid as DWORD == *(pvKey as *mut DWORD)
                    && (*info_2).info.ExtraInfo.cbData as usize
                        >= ::core::mem::size_of::<DWORD>() as usize
                    && *((*info_2).info.ExtraInfo.pbData as *mut DWORD)
                        == *((pvKey as LPBYTE)
                            .offset(::core::mem::size_of::<DWORD>() as usize as isize)
                            as *mut DWORD)
                    && (dwGroupId == 0 || (*info_2).info.dwGroupId == dwGroupId)
                {
                    ret = &raw mut (*info_2).info as PCCRYPT_OID_INFO;
                    break;
                } else {
                    info_2 = ((*info_2).entry.next as *mut ::core::ffi::c_char).offset(
                        -(core::mem::offset_of!(OIDInfo, entry)
                            as ::core::ffi::c_ulonglong as isize),
                    ) as *mut OIDInfo;
                }
            }
            LeaveCriticalSection(&raw mut oidInfoCS);
        }
        _ => {}
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertAlgIdToOID(mut dwAlgId: DWORD) -> LPCSTR {
    let mut ret: LPCSTR = core::ptr::null();
    let info: PCCRYPT_OID_INFO = CryptFindOIDInfo(
        CRYPT_OID_INFO_ALGID_KEY as DWORD,
        core::ptr::addr_of_mut!(dwAlgId) as *mut core::ffi::c_void,
        0 as DWORD,
    );

    if !info.is_null() {
        ret = (*info).pszOID;
    } else {
        ret = core::ptr::null();
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertOIDToAlgId(pszObjId: LPCSTR) -> DWORD {
    let mut ret: DWORD;
    let info: PCCRYPT_OID_INFO = CryptFindOIDInfo(
        CRYPT_OID_INFO_OID_KEY as DWORD,
        pszObjId as *mut core::ffi::c_void,
        0 as DWORD,
    );

    if !info.is_null() {
        ret = (*info).__bindgen_anon_1.Algid;
    } else {
        ret = 0 as DWORD;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn crypt_oid_init() {
    init_oid_info();
    init_registered_oid_info();
}
#[no_mangle]
pub unsafe extern "C" fn crypt_oid_free() {
    free_function_sets();
    free_oid_info();
}
