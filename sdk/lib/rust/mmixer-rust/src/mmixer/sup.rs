// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_00A6C7A05271: [u16; 18] = [
    0x0044u16, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0050, 0x0061, 0x0072,
    0x0061, 0x006D, 0x0065, 0x0074, 0x0065, 0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_76E480513E2C: [u16; 5] = [0x004Eu16, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_8EEFDCC15199: [u16; 67] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0059,
    0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
    0x005C, 0x004D, 0x0065, 0x0064, 0x0069, 0x0061, 0x0043, 0x0061, 0x0074, 0x0065,
    0x0067, 0x006F, 0x0072, 0x0069, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_B5B0A80B585D: [u16; 13] = [
    0x0046u16, 0x0072, 0x0069, 0x0065, 0x006E, 0x0064, 0x006C, 0x0079, 0x004E, 0x0061,
    0x006D, 0x0065, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{GUID, INT64, LPGUID, LPWSTR, RtlFreeUnicodeString, RtlStringFromGUID};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut KSNODETYPE_SUM: GUID = _GUID {
    Data1: 0xda441a60 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xc556 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_DAC: GUID = _GUID {
    Data1: 0x507ae360 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xc554 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_ADC: GUID = _GUID {
    Data1: 0x4d837fe0 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xc555 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_AGC: GUID = _GUID {
    Data1: 0xe88c9ba0 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xc557 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_LOUDNESS: GUID = _GUID {
    Data1: 0x41887440 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xc558 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_MUTE: GUID = _GUID {
    Data1: 0x2b223c0 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xc557 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_TONE: GUID = _GUID {
    Data1: 0x7607e580 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xc557 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_VOLUME: GUID = _GUID {
    Data1: 0x3a5acc00 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xc557 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_PEAKMETER: GUID = _GUID {
    Data1: 0xa085651e as ::core::ffi::c_uint,
    Data2: 0x5f0d as ::core::ffi::c_ushort,
    Data3: 0x4b36 as ::core::ffi::c_ushort,
    Data4: [
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x69 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x95 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_MUX: GUID = _GUID {
    Data1: 0x2ceaf780 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xc556 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_STEREO_WIDE: GUID = _GUID {
    Data1: 0xa9e69800 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xc558 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_CHORUS: GUID = _GUID {
    Data1: 0x20173f20 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xc559 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_REVERB: GUID = _GUID {
    Data1: 0xef0328e0 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xc558 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_SUPERMIX: GUID = _GUID {
    Data1: 0xe573adc0 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xc555 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSPROPSETID_Audio: GUID = _GUID {
    Data1: 0x45ffaaa0 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x6e1b as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x54 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSPROPSETID_Pin: GUID = _GUID {
    Data1: 0x8c134960 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x51ad as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSPROPSETID_General: GUID = _GUID {
    Data1: 0x1464eda5 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x6a8f as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSPROPSETID_Topology: GUID = _GUID {
    Data1: 0x720d4ac0 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x7533 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSEVENTSETID_AudioControlChange: GUID = _GUID {
    Data1: 0xe85e9698 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xfa2f as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x95 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSDATAFORMAT_TYPE_MUSIC: GUID = _GUID {
    Data1: 0xe725d360 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x62cc as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSDATAFORMAT_SUBTYPE_MIDI: GUID = _GUID {
    Data1: 0x1d262760 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xe957 as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSDATAFORMAT_SPECIFIER_NONE: GUID = _GUID {
    Data1: 0xf6417d6 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xc318 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};
#[no_mangle]
pub unsafe extern "C" fn MMixerVerifyContext(mut MixerContext: PMIXER_CONTEXT) -> MIXER_STATUS {
    if (*MixerContext).SizeOfStruct as usize != ::core::mem::size_of::<MIXER_CONTEXT>() as usize {
        return MM_STATUS_INVALID_PARAMETER;
    }
    if (*MixerContext).Alloc.is_none()
        || (*MixerContext).Control.is_none()
        || (*MixerContext).Free.is_none()
        || (*MixerContext).Open.is_none()
        || (*MixerContext).AllocEventData.is_none()
        || (*MixerContext).FreeEventData.is_none()
        || (*MixerContext).Close.is_none()
        || (*MixerContext).OpenKey.is_none()
        || (*MixerContext).QueryKeyValue.is_none()
        || (*MixerContext).CloseKey.is_none()
    {
        return MM_STATUS_INVALID_PARAMETER;
    }
    if (*MixerContext).MixerContext.is_null() {
        return MM_STATUS_INVALID_PARAMETER;
    }
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetMixerLineContainingNodeId(
    mut MixerInfo: LPMIXER_INFO,
    mut NodeID: ULONG,
) -> LPMIXERLINE_EXT {
    let mut Entry: PLIST_ENTRY = ::core::ptr::null_mut::<_LIST_ENTRY>();
    let mut ControlEntry: PLIST_ENTRY = ::core::ptr::null_mut::<_LIST_ENTRY>();
    let mut MixerLineSrc: LPMIXERLINE_EXT = ::core::ptr::null_mut::<MIXERLINE_EXT>();
    let mut MixerControl: LPMIXERCONTROL_EXT = ::core::ptr::null_mut::<MIXERCONTROL_EXT>();
    Entry = (*MixerInfo).LineList.Flink as PLIST_ENTRY;
    while Entry != &raw mut (*MixerInfo).LineList {
        MixerLineSrc = (Entry as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(MIXERLINE_EXT, Entry) as ULONG_PTR)
            as *mut MIXERLINE_EXT as LPMIXERLINE_EXT;
        ControlEntry = (*MixerLineSrc).ControlsList.Flink as PLIST_ENTRY;
        while ControlEntry != &raw mut (*MixerLineSrc).ControlsList {
            MixerControl = (ControlEntry as ULONG_PTR).wrapping_sub(
                core::mem::offset_of!(MIXERCONTROL_EXT, Entry) as ULONG_PTR,
            ) as *mut MIXERCONTROL_EXT as LPMIXERCONTROL_EXT;
            if (*MixerControl).NodeID == NodeID {
                return MixerLineSrc;
            }
            ControlEntry = (*ControlEntry).Flink as PLIST_ENTRY;
        }
        Entry = (*Entry).Flink as PLIST_ENTRY;
    }
    return ::core::ptr::null_mut::<MIXERLINE_EXT>();
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetLowestLogicalTopologyPinOffsetFromArray(
    mut LogicalPinArrayCount: ULONG,
    mut LogicalPinArray: PULONG,
    mut PinOffset: PULONG,
) {
    let mut Index: ULONG = 0;
    let mut LowestId: ULONG = 0;
    Index = 1;
    while Index < LogicalPinArrayCount {
        if *LogicalPinArray.offset(Index as isize) != MAXULONG {
            ASSERT!(
                *LogicalPinArray.offset(Index as isize)
                    != *LogicalPinArray.offset(LowestId as isize)
            );
        }
        if *LogicalPinArray.offset(Index as isize) < *LogicalPinArray.offset(LowestId as isize) {
            LowestId = Index;
        }
        Index = Index.wrapping_add(1);
    }
    *PinOffset = LowestId;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerFreeMixerInfo(MixerContext: PMIXER_CONTEXT, MixerInfo: LPMIXER_INFO) {
    ((*MixerContext).Free.unwrap_unchecked())(MixerInfo as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetMixerDataByDeviceHandle(
    MixerContext: PMIXER_CONTEXT,
    hDevice: HANDLE,
) -> LPMIXER_DATA {
    let mut MixerData: LPMIXER_DATA;
    let mut Entry: PLIST_ENTRY;
    let mut MixerList: PMIXER_LIST;

    /* get mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    if (*MixerList).MixerDataCount == 0 {
        return core::ptr::null_mut();
    }

    Entry = (*MixerList).MixerData.Flink;

    while Entry != core::ptr::addr_of_mut!((*MixerList).MixerData) {
        MixerData = (Entry as usize)
            .wrapping_sub(core::mem::offset_of!(MIXER_DATA, Entry))
            as LPMIXER_DATA;

        if (*MixerData).hDevice == hDevice {
            return MixerData;
        }

        /* move to next mixer entry */
        Entry = (*Entry).Flink;
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetMixerInfoByIndex(
    MixerContext: PMIXER_CONTEXT,
    MixerIndex: ULONG,
) -> LPMIXER_INFO {
    let mut MixerInfo: LPMIXER_INFO = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut Index: ULONG = 0;

    /* get mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    if (*MixerList).MixerListCount == 0 {
        return core::ptr::null_mut();
    }

    Entry = (*MixerList).MixerList.Flink;

    while Entry != core::ptr::addr_of_mut!((*MixerList).MixerList) {
        MixerInfo = (Entry as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(MIXER_INFO, Entry) as ULONG_PTR)
            as LPMIXER_INFO;

        if Index == MixerIndex {
            return MixerInfo;
        }

        /* move to next mixer entry */
        Index = Index.wrapping_add(1);
        Entry = (*Entry).Flink;
    }

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetMixerByName(
    mut MixerList: PMIXER_LIST,
    mut MixerName: LPWSTR,
    mut OutMixerInfo: *mut LPMIXER_INFO,
) -> MIXER_STATUS {
    let mut MixerInfo: LPMIXER_INFO = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();

    Entry = (*MixerList).MixerList.Flink;
    while Entry != &raw mut (*MixerList).MixerList {
        MixerInfo = CONTAINING_RECORD!(Entry, MIXER_INFO, Entry) as LPMIXER_INFO;

        DPRINT1!(
            "MixerName %S MixerName %S\n",
            &raw mut (*MixerInfo).MixCaps.szPname as *mut WCHAR,
            MixerName
        );
        if _wcsicmp(
            &raw mut (*MixerInfo).MixCaps.szPname as *mut WCHAR,
            MixerName,
        ) == 0
        {
            *OutMixerInfo = MixerInfo;
            return MM_STATUS_SUCCESS;
        }
        /* move to next mixer entry */
        Entry = (*Entry).Flink;
    }

    return MM_STATUS_UNSUCCESSFUL;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetSourceMixerLineByLineId(
    mut MixerInfo: LPMIXER_INFO,
    mut dwLineID: DWORD,
) -> LPMIXERLINE_EXT {
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut MixerLineSrc: LPMIXERLINE_EXT = core::ptr::null_mut();

    /* get first entry */
    Entry = (*MixerInfo).LineList.Flink;

    while Entry != core::ptr::addr_of_mut!((*MixerInfo).LineList) {
        MixerLineSrc = ((Entry as usize)
            .wrapping_sub(core::mem::offset_of!(MIXERLINE_EXT, Entry)))
            as LPMIXERLINE_EXT;
        DPRINT!(
            "dwLineID %x dwLineID %x MixerLineSrc %p\n",
            (*MixerLineSrc).Line.dwLineID,
            dwLineID,
            MixerLineSrc
        );
        if (*MixerLineSrc).Line.dwLineID == dwLineID {
            return MixerLineSrc;
        }

        Entry = (*Entry).Flink;
    }

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetNodeType(
       MultipleItem: PKSMULTIPLE_ITEM,
       Index: ULONG) -> LPGUID {
    let mut NodeType: LPGUID;

    ASSERT!(Index < (*MultipleItem).Count);

    NodeType = (MultipleItem.offset(1 as isize)) as LPGUID;
    return NodeType.offset(Index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetSourceMixerLineByComponentType(
    mut MixerInfo: LPMIXER_INFO,
    mut dwComponentType: DWORD,
) -> LPMIXERLINE_EXT {
    let mut Entry: PLIST_ENTRY = ::core::ptr::null_mut::<_LIST_ENTRY>();
    let mut MixerLineSrc: LPMIXERLINE_EXT = ::core::ptr::null_mut::<MIXERLINE_EXT>();

    /* get first entry */
    Entry = (*MixerInfo).LineList.Flink as PLIST_ENTRY;

    while Entry != &raw mut (*MixerInfo).LineList {
        MixerLineSrc = (Entry as usize)
            .wrapping_sub(core::mem::offset_of!(MIXERLINE_EXT, Entry) as usize)
            as *mut MIXERLINE_EXT as LPMIXERLINE_EXT;
        if (*MixerLineSrc).Line.dwComponentType == dwComponentType {
            return MixerLineSrc;
        }

        Entry = (*Entry).Flink as PLIST_ENTRY;
    }

    return ::core::ptr::null_mut::<MIXERLINE_EXT>();
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetMixerControlById(
    mut MixerInfo: LPMIXER_INFO,
    mut dwControlID: DWORD,
    mut OutMixerLine: *mut LPMIXERLINE_EXT,
    mut OutMixerControl: *mut LPMIXERCONTROL_EXT,
    mut NodeId: PULONG,
) -> MIXER_STATUS {
    let mut Entry: PLIST_ENTRY = ::core::ptr::null_mut::<_LIST_ENTRY>();
    let mut ControlEntry: PLIST_ENTRY = ::core::ptr::null_mut::<_LIST_ENTRY>();
    let mut MixerLineSrc: LPMIXERLINE_EXT = ::core::ptr::null_mut::<MIXERLINE_EXT>();
    let mut MixerControl: LPMIXERCONTROL_EXT = ::core::ptr::null_mut::<MIXERCONTROL_EXT>();
    /* get first entry */
    Entry = (*MixerInfo).LineList.Flink as PLIST_ENTRY;
    while Entry != &raw mut (*MixerInfo).LineList {
        MixerLineSrc = (Entry as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(MIXERLINE_EXT, Entry) as ULONG_PTR)
            as *mut MIXERLINE_EXT as LPMIXERLINE_EXT;
        ControlEntry = (*MixerLineSrc).ControlsList.Flink as PLIST_ENTRY;
        while ControlEntry != &raw mut (*MixerLineSrc).ControlsList {
            MixerControl = (ControlEntry as ULONG_PTR).wrapping_sub(
                core::mem::offset_of!(MIXERCONTROL_EXT, Entry) as ULONG_PTR,
            ) as *mut MIXERCONTROL_EXT as LPMIXERCONTROL_EXT;
            if (*MixerControl).Control.dwControlID == dwControlID {
                if !OutMixerLine.is_null() {
                    *OutMixerLine = MixerLineSrc;
                }
                if !OutMixerControl.is_null() {
                    *OutMixerControl = MixerControl;
                }
                if !NodeId.is_null() {
                    *NodeId = (*MixerControl).NodeID as ::core::ffi::c_ulong;
                }
                return MM_STATUS_SUCCESS;
            }
            ControlEntry = (*ControlEntry).Flink as PLIST_ENTRY;
        }
        Entry = (*Entry).Flink as PLIST_ENTRY;
    }

    return MM_STATUS_UNSUCCESSFUL;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerNotifyControlChange(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerInfo: LPMIXER_INFO,
    mut NotificationType: ULONG,
    mut Value: ULONG,
) {
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut NotificationEntry: PEVENT_NOTIFICATION_ENTRY = core::ptr::null_mut();

    /* enumerate list and perform notification */
    Entry = (*MixerInfo).EventList.Flink as PLIST_ENTRY;
    while Entry != core::ptr::addr_of_mut!((*MixerInfo).EventList) as PLIST_ENTRY {
        /* get notification entry offset */
        NotificationEntry = CONTAINING_RECORD!(Entry, EVENT_NOTIFICATION_ENTRY, Entry) as PEVENT_NOTIFICATION_ENTRY;

        if let Some(routine) = (*NotificationEntry).MixerEventRoutine {
            /* now perform the callback */
            routine(
                (*NotificationEntry).MixerEventContext,
                MixerInfo as HANDLE,
                NotificationType,
                Value,
            );
        }

        /* move to next notification entry */
        Entry = (*Entry).Flink as PLIST_ENTRY;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerSetGetMuteControlDetails(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerInfo: LPMIXER_INFO,
    mut MixerControl: LPMIXERCONTROL_EXT,
    mut dwLineID: ULONG,
    mut MixerControlDetails: LPMIXERCONTROLDETAILS,
    mut bSet: ULONG,
) -> MIXER_STATUS {
    let mut Input: LPMIXERCONTROLDETAILS_BOOLEAN = core::ptr::null_mut();
    let mut Value: LONG = 0;
    let mut Status: MIXER_STATUS;

    if (*MixerControlDetails).cbDetails as usize != core::mem::size_of::<MIXERCONTROLDETAILS_BOOLEAN>() as usize {
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* get input */
    Input = (*MixerControlDetails).paDetails as LPMIXERCONTROLDETAILS_BOOLEAN;

    /* FIXME SEH */
    if bSet != 0 {
        Value = (*Input).fValue;
    }

    /* set control details */
    Status = MMixerSetGetControlDetails(
        MixerContext,
        (*MixerControl).hDevice,
        (*MixerControl).NodeID,
        bSet,
        KSPROPERTY_AUDIO_MUTE as ULONG,
        0 as ULONG,
        core::ptr::addr_of_mut!(Value),
    );

    if Status != MM_STATUS_SUCCESS as MIXER_STATUS {
        return Status;
    }

    /* FIXME SEH */
    if bSet == 0 {
        (*Input).fValue = Value;
        return Status;
    } else {
        /* notify wdmaud clients MM_MIXM_LINE_CHANGE dwLineID */
        MMixerNotifyControlChange(
            MixerContext,
            MixerInfo,
            MM_MIXM_LINE_CHANGE as ULONG,
            dwLineID,
        );
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn MMixerSetGetMuxControlDetails(
    MixerContext: PMIXER_CONTEXT,
    MixerInfo: LPMIXER_INFO,
    NodeId: ULONG,
    bSet: ULONG,
    Flags: ULONG,
    MixerControl: LPMIXERCONTROL_EXT,
    MixerControlDetails: LPMIXERCONTROLDETAILS,
    MixerLine: LPMIXERLINE_EXT,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS = core::mem::zeroed();
    let mut LogicalNodes: PULONG = core::ptr::null_mut();
    let mut ConnectedNodes: PULONG = core::ptr::null_mut();
    let mut LogicalNodesCount: ULONG = 0;
    let mut ConnectedNodesCount: ULONG = 0;
    let mut Index: ULONG = 0;
    let mut CurLogicalPinOffset: ULONG = 0;
    let mut BytesReturned: ULONG = 0;
    let mut OldLogicalPinOffset: ULONG = 0;
    let mut MixerData: LPMIXER_DATA = core::ptr::null_mut();
    let mut ListText: LPMIXERCONTROLDETAILS_LISTTEXTW = core::ptr::null_mut();
    let mut Values: LPMIXERCONTROLDETAILS_BOOLEAN = core::ptr::null_mut();
    let mut SourceLine: LPMIXERLINE_EXT = core::ptr::null_mut();
    let mut Request: KSNODEPROPERTY = core::mem::zeroed();

    DPRINT!("MixerControlDetails %p\n", MixerControlDetails);
    DPRINT!("bSet %lx\n", bSet);
    DPRINT!("Flags %lx\n", Flags);
    DPRINT!("NodeId %lu\n", (*MixerControl).NodeID);
    DPRINT!("MixerControlDetails dwControlID %lu\n", (*MixerControlDetails).dwControlID);
    DPRINT!("MixerControlDetails cChannels %lu\n", (*MixerControlDetails).cChannels);
    DPRINT!("MixerControlDetails cMultipleItems %lu\n", (*MixerControlDetails).cMultipleItems);
    DPRINT!("MixerControlDetails cbDetails %lu\n", (*MixerControlDetails).cbDetails);
    DPRINT!("MixerControlDetails paDetails %p\n", (*MixerControlDetails).paDetails);

    if (*MixerControl).Control.fdwControl & MIXERCONTROL_CONTROLF_UNIFORM != 0 {
        /* control acts uniform */
        if (*MixerControlDetails).cChannels != 1 {
            /* expected 1 channel */
            DPRINT1!("Expected 1 channel but got %lu\n", (*MixerControlDetails).cChannels);
            return MM_STATUS_UNSUCCESSFUL;
        }
    }

    /* check if multiple items match */
    if (*MixerControlDetails).__bindgen_anon_1.cMultipleItems != (*MixerControl).Control.cMultipleItems {
        DPRINT1!(
            "MultipleItems mismatch %lu expected %lu\n",
            (*MixerControlDetails).__bindgen_anon_1.cMultipleItems,
            (*MixerControl).Control.cMultipleItems
        );
        return MM_STATUS_UNSUCCESSFUL;
    }

    if bSet != 0 {
        if (Flags & MIXER_SETCONTROLDETAILSF_QUERYMASK) == MIXER_SETCONTROLDETAILSF_CUSTOM {
            /* tell me when this is hit */
            ASSERT!(FALSE);
        } else if (Flags & (MIXER_SETCONTROLDETAILSF_VALUE | MIXER_SETCONTROLDETAILSF_CUSTOM)) == MIXER_SETCONTROLDETAILSF_VALUE {
            /* sanity check */
            ASSERT!(bSet == (TRUE as u32));
            ASSERT!(
                (*MixerControlDetails).cbDetails as usize
                    == core::mem::size_of::<MIXERCONTROLDETAILS_BOOLEAN>()
            );

            Values = (*MixerControlDetails).paDetails as LPMIXERCONTROLDETAILS_BOOLEAN;
            CurLogicalPinOffset = MAXULONG;
            Index = 0;
            while Index < (*MixerControlDetails).__bindgen_anon_1.cMultipleItems {
                if (*Values.add(Index as usize)).fValue != 0 {
                    /* mux can only activate one line at a time */
                    ASSERT!(CurLogicalPinOffset == MAXULONG);
                    CurLogicalPinOffset = Index;
                }
                Index = Index.wrapping_add(1);
            }

            /* setup request */
            Request.NodeId = NodeId;
            Request.Reserved = 0;
            Request.Property.Flags = KSPROPERTY_TYPE_TOPOLOGY | KSPROPERTY_TYPE_GET;
            Request.Property.Id = KSPROPERTY_AUDIO_MUX_SOURCE;
            Request.Property.Set = KSPROPSETID_Audio;

            /* perform getting source */
            Status = (*MixerContext).Control.unwrap_unchecked()(
                (*MixerControl).hDevice,
                IOCTL_KS_PROPERTY,
                core::ptr::addr_of_mut!(Request) as PVOID,
                core::mem::size_of::<KSNODEPROPERTY>() as ULONG,
                core::ptr::addr_of_mut!(OldLogicalPinOffset) as PVOID,
                core::mem::size_of::<ULONG>() as ULONG,
                core::ptr::addr_of_mut!(BytesReturned),
            );
            if Status != MM_STATUS_SUCCESS {
                /* failed to get source */
                return Status;
            }

            DPRINT!(
                "OldLogicalPinOffset %lu CurLogicalPinOffset %lu\n",
                OldLogicalPinOffset,
                CurLogicalPinOffset
            );

            if OldLogicalPinOffset == CurLogicalPinOffset {
                /* cannot be unselected */
                return MM_STATUS_UNSUCCESSFUL;
            }

            /* perform setting source */
            Request.Property.Flags = KSPROPERTY_TYPE_TOPOLOGY | KSPROPERTY_TYPE_SET;
            Status = (*MixerContext).Control.unwrap_unchecked()(
                (*MixerControl).hDevice,
                IOCTL_KS_PROPERTY,
                core::ptr::addr_of_mut!(Request) as PVOID,
                core::mem::size_of::<KSNODEPROPERTY>() as ULONG,
                core::ptr::addr_of_mut!(CurLogicalPinOffset) as PVOID,
                core::mem::size_of::<ULONG>() as ULONG,
                core::ptr::addr_of_mut!(BytesReturned),
            );
            if Status != MM_STATUS_SUCCESS {
                /* failed to set source */
                return Status;
            }

            /* notify control change */
            MMixerNotifyControlChange(
                MixerContext,
                MixerInfo,
                MM_MIXM_CONTROL_CHANGE,
                (*MixerControl).Control.dwControlID,
            );

            return Status;
        }
    } else {
        if (Flags & MIXER_GETCONTROLDETAILSF_QUERYMASK) == MIXER_GETCONTROLDETAILSF_VALUE {
            /* setup request */
            Request.NodeId = NodeId;
            Request.Reserved = 0;
            Request.Property.Flags = KSPROPERTY_TYPE_TOPOLOGY | KSPROPERTY_TYPE_GET;
            Request.Property.Id = KSPROPERTY_AUDIO_MUX_SOURCE;
            Request.Property.Set = KSPROPSETID_Audio;

            /* perform getting source */
            Status = (*MixerContext).Control.unwrap_unchecked()(
                (*MixerControl).hDevice,
                IOCTL_KS_PROPERTY,
                core::ptr::addr_of_mut!(Request) as PVOID,
                core::mem::size_of::<KSNODEPROPERTY>() as ULONG,
                core::ptr::addr_of_mut!(OldLogicalPinOffset) as PVOID,
                core::mem::size_of::<ULONG>() as ULONG,
                core::ptr::addr_of_mut!(BytesReturned),
            );
            if Status != MM_STATUS_SUCCESS {
                /* failed to get source */
                return Status;
            }

            /* gets the corresponding mixer data */
            MixerData = MMixerGetMixerDataByDeviceHandle(MixerContext, (*MixerControl).hDevice);

            /* sanity check */
            ASSERT!(!MixerData.is_null());
            ASSERT!(!(*MixerData).Topology.is_null());
            ASSERT!((*MixerData).MixerInfo == MixerInfo);

            /* now allocate logical pin array */
            Status = MMixerAllocateTopologyNodeArray(
                MixerContext,
                (*MixerData).Topology,
                core::ptr::addr_of_mut!(LogicalNodes),
            );
            if Status != MM_STATUS_SUCCESS {
                /* no memory */
                return MM_STATUS_NO_MEMORY;
            }

            /* get logical pin nodes */
            MMixerGetConnectedFromLogicalTopologyPins(
                (*MixerData).Topology,
                (*MixerControl).NodeID,
                core::ptr::addr_of_mut!(LogicalNodesCount),
                LogicalNodes,
            );

            /* sanity check */
            ASSERT!(LogicalNodesCount == (*MixerControlDetails).__bindgen_anon_1.cMultipleItems);
            ASSERT!(
                LogicalNodesCount
                    == (*MixerControl).Control.Metrics.dwReserved[0 as usize]
            );

            Values = (*MixerControlDetails).paDetails as LPMIXERCONTROLDETAILS_BOOLEAN;
            Index = 0;
            while Index < LogicalNodesCount {
                /* getting logical pin offset */
                MMixerGetLowestLogicalTopologyPinOffsetFromArray(
                    LogicalNodesCount,
                    LogicalNodes,
                    core::ptr::addr_of_mut!(CurLogicalPinOffset),
                );

                if CurLogicalPinOffset == OldLogicalPinOffset {
                    /* mark index as active */
                    (*Values.add(Index as usize)).fValue = (TRUE as i32);
                } else {
                    /* index not active */
                    (*Values.add(Index as usize)).fValue = (FALSE as i32);
                }

                /* mark offset as consumed */
                *LogicalNodes.add(CurLogicalPinOffset as usize) = MAXULONG;
                Index = Index.wrapping_add(1);
            }

            /* cleanup */
            (*MixerContext).Free.unwrap_unchecked()(LogicalNodes as PVOID);

            /* done */
            return MM_STATUS_SUCCESS;
        } else if (Flags & MIXER_GETCONTROLDETAILSF_QUERYMASK) == MIXER_GETCONTROLDETAILSF_LISTTEXT {
            /* sanity check */
            ASSERT!(bSet == (FALSE as u32));

            /* gets the corresponding mixer data */
            MixerData = MMixerGetMixerDataByDeviceHandle(MixerContext, (*MixerControl).hDevice);

            /* sanity check */
            ASSERT!(!MixerData.is_null());
            ASSERT!(!(*MixerData).Topology.is_null());
            ASSERT!((*MixerData).MixerInfo == MixerInfo);

            /* now allocate logical pin array */
            Status = MMixerAllocateTopologyNodeArray(
                MixerContext,
                (*MixerData).Topology,
                core::ptr::addr_of_mut!(LogicalNodes),
            );
            if Status != MM_STATUS_SUCCESS {
                /* no memory */
                return MM_STATUS_NO_MEMORY;
            }

            /* allocate connected node array */
            Status = MMixerAllocateTopologyNodeArray(
                MixerContext,
                (*MixerData).Topology,
                core::ptr::addr_of_mut!(ConnectedNodes),
            );
            if Status != MM_STATUS_SUCCESS {
                /* no memory */
                (*MixerContext).Free.unwrap_unchecked()(LogicalNodes as PVOID);
                return MM_STATUS_NO_MEMORY;
            }

            /* get logical pin nodes */
            MMixerGetConnectedFromLogicalTopologyPins(
                (*MixerData).Topology,
                (*MixerControl).NodeID,
                core::ptr::addr_of_mut!(LogicalNodesCount),
                LogicalNodes,
            );

            /* get connected nodes */
            MMixerGetNextNodesFromNodeIndex(
                MixerContext,
                (*MixerData).Topology,
                (*MixerControl).NodeID,
                (TRUE) as ULONG,
                core::ptr::addr_of_mut!(ConnectedNodesCount),
                ConnectedNodes,
            );

            /* sanity check */
            ASSERT!(ConnectedNodesCount == LogicalNodesCount);
            ASSERT!(ConnectedNodesCount == (*MixerControlDetails).__bindgen_anon_1.cMultipleItems);
            ASSERT!(
                ConnectedNodesCount
                    == (*MixerControl).Control.Metrics.dwReserved[0 as usize]
            );

            ListText = (*MixerControlDetails).paDetails as LPMIXERCONTROLDETAILS_LISTTEXTW;

            Index = 0;
            while Index < ConnectedNodesCount {
                /* getting logical pin offset */
                MMixerGetLowestLogicalTopologyPinOffsetFromArray(
                    LogicalNodesCount,
                    LogicalNodes,
                    core::ptr::addr_of_mut!(CurLogicalPinOffset),
                );

                /* get mixer line with that node */
                SourceLine = MMixerGetMixerLineContainingNodeId(
                    MixerInfo,
                    *ConnectedNodes.add(CurLogicalPinOffset as usize),
                );

                /* sanity check */
                ASSERT!(!SourceLine.is_null());

                DPRINT1!(
                    "PinOffset %lu LogicalPin %lu NodeId %lu LineName %S\n",
                    CurLogicalPinOffset,
                    *LogicalNodes.add(CurLogicalPinOffset as usize),
                    *ConnectedNodes.add(CurLogicalPinOffset as usize),
                    core::ptr::addr_of_mut!((*SourceLine).Line.szName) as *mut WCHAR
                );

                /* copy details */
                (*ListText.add(Index as usize)).dwParam1 = (*SourceLine).Line.dwLineID;
                (*ListText.add(Index as usize)).dwParam2 = (*SourceLine).Line.dwComponentType;
                (*MixerContext).Copy.unwrap_unchecked()(
                    core::ptr::addr_of_mut!((*ListText.add(Index as usize)).szName) as *mut WCHAR
                        as PVOID,
                    core::ptr::addr_of_mut!((*SourceLine).Line.szName) as *mut WCHAR as PVOID,
                    ((wcslen(
                        core::ptr::addr_of_mut!((*SourceLine).Line.szName) as *mut WCHAR
                    ) + 1 as usize)
                        * core::mem::size_of::<WCHAR>()) as ULONG,
                );

                /* mark offset as consumed */
                *LogicalNodes.add(CurLogicalPinOffset as usize) = MAXULONG;
                Index = Index.wrapping_add(1);
            }

            /* cleanup */
            (*MixerContext).Free.unwrap_unchecked()(LogicalNodes as PVOID);
            (*MixerContext).Free.unwrap_unchecked()(ConnectedNodes as PVOID);

            /* done */
            return MM_STATUS_SUCCESS;
        }
    }

    return MM_STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerSetGetVolumeControlDetails(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerInfo: LPMIXER_INFO,
    mut NodeId: ULONG,
    mut bSet: ULONG,
    mut MixerControl: LPMIXERCONTROL_EXT,
    mut MixerControlDetails: LPMIXERCONTROLDETAILS,
    mut MixerLine: LPMIXERLINE_EXT,
) -> MIXER_STATUS {
    let mut Input: LPMIXERCONTROLDETAILS_UNSIGNED = core::ptr::null_mut();
    let mut MaxRange: LONG = 0;
    let mut Value: LONG = 0;
    let mut Channel: ULONG = 0;
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;
    let mut VolumeData: LPMIXERVOLUME_DATA = core::ptr::null_mut();

    if (*MixerControlDetails).cbDetails as usize != core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() {
        return MM_STATUS_INVALID_PARAMETER;
    }

    VolumeData = (*MixerControl).ExtraData as LPMIXERVOLUME_DATA;
    if VolumeData.is_null() {
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* Get input */
    Input = (*MixerControlDetails).paDetails as LPMIXERCONTROLDETAILS_UNSIGNED;
    if Input.is_null() {
        return MM_STATUS_UNSUCCESSFUL; /* To prevent dereferencing NULL */
    }

    /* Get maximum available range */
    MaxRange = (*VolumeData).SignedMaximum - (*VolumeData).SignedMinimum;

    /* Loop for each channel */
    Channel = 0 as ULONG;
    while Channel < (*MixerControlDetails).cChannels {
        if bSet != 0 {
            /* FIXME SEH */
            /* Convert from logical units to hardware range (DB) */
            Value = (((*Input.offset(Channel as isize)).dwValue as INT64 * MaxRange as INT64)
                / 0x10000 as INT64
                + (*VolumeData).SignedMinimum as INT64) as LONG;
        }

        /* Get/set control details */
        Status = MMixerSetGetControlDetails(
            MixerContext,
            (*MixerControl).hDevice,
            NodeId,
            bSet,
            KSPROPERTY_AUDIO_VOLUMELEVEL as ULONG,
            Channel,
            core::ptr::addr_of_mut!(Value),
        );

        if bSet == 0 {
            /* FIXME SEH */
            /* Convert from hardware range (DB) to logical units */
            (*Input.offset(Channel as isize)).dwValue = ((((Value as INT64
                - (*VolumeData).SignedMinimum as INT64
                + 1 as INT64)
                * 0x10000 as INT64)
                / MaxRange as INT64) as ULONG) as DWORD;
        }

        Channel = Channel.wrapping_add(1);
    }

    if bSet != 0 {
        /* Notify clients of a line change */
        MMixerNotifyControlChange(
            MixerContext,
            MixerInfo,
            MM_MIXM_CONTROL_CHANGE as ULONG,
            (*MixerControl).Control.dwControlID as ULONG,
        );
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetDataByDeviceId(
    mut MixerList: PMIXER_LIST,
    mut DeviceId: ULONG,
) -> LPMIXER_DATA {
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut MixerData: LPMIXER_DATA = core::ptr::null_mut();
    Entry = (*MixerList).MixerData.Flink as PLIST_ENTRY;
    while Entry != core::ptr::addr_of_mut!((*MixerList).MixerData) {
        MixerData = (Entry as usize)
            .wrapping_sub(core::mem::offset_of!(MIXER_DATA, Entry))
            as *mut MIXER_DATA as LPMIXER_DATA;
        if (*MixerData).DeviceId == DeviceId {
            return MixerData;
        }
        Entry = (*Entry).Flink as PLIST_ENTRY;
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetDataByDeviceName(
    MixerList: PMIXER_LIST,
    DeviceName: LPWSTR,
) -> LPMIXER_DATA {
    let mut Entry: PLIST_ENTRY;
    let mut MixerData: LPMIXER_DATA;

    Entry = (*MixerList).MixerData.Flink;
    while Entry != core::ptr::addr_of_mut!((*MixerList).MixerData) {
        MixerData = CONTAINING_RECORD!(Entry, MIXER_DATA, Entry) as LPMIXER_DATA;
        if _wcsicmp(
            DeviceName.offset(2 as isize) as *mut WCHAR,
            (*MixerData).DeviceName.offset(2 as isize) as *mut WCHAR,
        ) == 0
        {
            /* found entry */
            return MixerData;
        }
        Entry = (*Entry).Flink;
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn MMixerCreateMixerData(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerList: PMIXER_LIST,
    mut DeviceId: ULONG,
    mut DeviceName: LPWSTR,
    mut hDevice: HANDLE,
    mut hKey: HANDLE,
) -> MIXER_STATUS {
    let mut MixerData: LPMIXER_DATA = core::ptr::null_mut();
    MixerData = (*MixerContext).Alloc.unwrap_unchecked()(
        core::mem::size_of::<MIXER_DATA>() as ULONG,
    ) as LPMIXER_DATA;
    if MixerData.is_null() {
        return MM_STATUS_NO_MEMORY;
    }
    (*MixerData).DeviceId = DeviceId;
    (*MixerData).DeviceName = DeviceName;
    (*MixerData).hDevice = hDevice;
    (*MixerData).hDeviceInterfaceKey = hKey;
    (*MixerData).Topology = core::ptr::null_mut();
    InsertTailList(
        core::ptr::addr_of_mut!((*MixerList).MixerData),
        core::ptr::addr_of_mut!((*MixerData).Entry),
    );
    (*MixerList).MixerDataCount = (*MixerList).MixerDataCount.wrapping_add(1);
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetDeviceNameWithComponentId(
    mut MixerContext: PMIXER_CONTEXT,
    mut hMixer: HANDLE,
    mut OutDeviceName: LPWSTR,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;
    let mut Property: KSPROPERTY = core::mem::zeroed();
    let mut ComponentId: KSCOMPONENTID = core::mem::zeroed();
    let mut Length: ULONG = 0;
    let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
    let mut ResultLength: ULONG = 0;
    let mut KeyType: ULONG = 0;
    let mut hMediaKey: HANDLE = core::ptr::null_mut();
    let mut hGuidKey: HANDLE = core::ptr::null_mut();
    let mut DeviceName: LPWSTR = core::ptr::null_mut();

    /* prepare property */
    Property.Flags = KSPROPERTY_TYPE_GET as ULONG;
    Property.Set = KSPROPSETID_General;
    Property.Id = KSPROPERTY_GENERAL_COMPONENTID as ULONG;

    /* try get component id */
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hMixer,
        IOCTL_KS_PROPERTY as ULONG,
        &raw mut Property as PVOID,
        core::mem::size_of::<KSPROPERTY>() as ULONG,
        &raw mut ComponentId as PVOID,
        core::mem::size_of::<KSCOMPONENTID>() as ULONG,
        &raw mut Length,
    );

    if Status == MM_STATUS_SUCCESS as MIXER_STATUS
    {
        Status = (*MixerContext).OpenKey.unwrap_unchecked()(
            NULL,
            RUSTCONV_WSTR_8EEFDCC15199.as_ptr() as LPWSTR,
            KEY_READ as ULONG,
            &raw mut hMediaKey,
        );
        if Status == MM_STATUS_SUCCESS as MIXER_STATUS
        {
            RtlStringFromGUID(&raw mut ComponentId.Name, &raw mut GuidString);
            Status = (*MixerContext).OpenKey.unwrap_unchecked()(
                hMediaKey,
                GuidString.Buffer as LPWSTR,
                KEY_READ as ULONG,
                &raw mut hGuidKey,
            );
            RtlFreeUnicodeString(&raw mut GuidString);
            if Status == MM_STATUS_SUCCESS as MIXER_STATUS
            {
                Status = (*MixerContext).QueryKeyValue.unwrap_unchecked()(
                    hGuidKey,
                    RUSTCONV_WSTR_76E480513E2C.as_ptr() as LPWSTR,
                    &raw mut DeviceName as *mut PVOID,
                    &raw mut ResultLength,
                    &raw mut KeyType,
                );
                if Status == MM_STATUS_SUCCESS as MIXER_STATUS
                {
                    (*MixerContext).Copy.unwrap_unchecked()(
                        OutDeviceName as PVOID,
                        DeviceName as PVOID,
                        min(
                            ResultLength,
                            ((MAXPNAMELEN - 1) * 2) as ULONG,
                        ),
                    );
                    (*MixerContext).Free.unwrap_unchecked()(DeviceName as PVOID);
                }

                (*MixerContext).CloseKey.unwrap_unchecked()(hGuidKey);
            }
            (*MixerContext).CloseKey.unwrap_unchecked()(hMediaKey);
        }
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetDeviceName(
    mut MixerContext: PMIXER_CONTEXT,
    mut DeviceName: LPWSTR,
    mut hKey: HANDLE,
) -> MIXER_STATUS {
    let mut Name: LPWSTR = core::ptr::null_mut();
    let mut hTemp: HANDLE = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut Type: ULONG = 0;
    let mut Status: MIXER_STATUS;

    Status = (*MixerContext).QueryKeyValue.unwrap_unchecked()(
        hKey,
        RUSTCONV_WSTR_B5B0A80B585D.as_ptr() as LPWSTR,
        &raw mut Name as *mut PVOID,
        &raw mut Length,
        &raw mut Type,
    );
    if Status == MM_STATUS_SUCCESS {
        /* copy device name */
        (*MixerContext).Copy.unwrap_unchecked()(
            DeviceName as PVOID,
            Name as PVOID,
            (min(
                wcslen(Name as *const WCHAR) as usize,
                (MAXPNAMELEN - 1) as usize,
            ))
            .wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
        );

        /* make sure its null terminated */
        *DeviceName.offset((MAXPNAMELEN - 1) as isize) =
            0x0000u16;

        /* free device name */
        (*MixerContext).Free.unwrap_unchecked()(Name as PVOID);

        /* done */
        return Status;
    }

    Status = (*MixerContext).OpenKey.unwrap_unchecked()(
        hKey,
        RUSTCONV_WSTR_00A6C7A05271.as_ptr() as LPWSTR,
        KEY_READ as ULONG,
        &raw mut hTemp,
    );
    if Status != MM_STATUS_SUCCESS {
        return Status;
    }

    Status = (*MixerContext).QueryKeyValue.unwrap_unchecked()(
        hTemp,
        RUSTCONV_WSTR_B5B0A80B585D.as_ptr() as LPWSTR,
        &raw mut Name as *mut PVOID,
        &raw mut Length,
        &raw mut Type,
    );
    if Status == MM_STATUS_SUCCESS {
        /* copy device name */
        (*MixerContext).Copy.unwrap_unchecked()(
            DeviceName as PVOID,
            Name as PVOID,
            (min(
                wcslen(Name as *const WCHAR) as usize,
                (MAXPNAMELEN - 1) as usize,
            ))
            .wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
        );

        /* make sure its null terminated */
        *DeviceName.offset((MAXPNAMELEN - 1) as isize) =
            0x0000u16;

        /* free device name */
        (*MixerContext).Free.unwrap_unchecked()(Name as PVOID);
    }

    (*MixerContext).CloseKey.unwrap_unchecked()(hTemp);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerInitializePinConnect(
    PinConnect: PKSPIN_CONNECT,
    PinId: ULONG,
) {
    (*PinConnect).Interface.Set = KSINTERFACESETID_Standard;
    (*PinConnect).Interface.Id = KSINTERFACE_STANDARD_STREAMING as ULONG;
    (*PinConnect).Interface.Flags = 0 as ULONG;
    (*PinConnect).Medium.Set = KSMEDIUMSETID_Standard;
    (*PinConnect).Medium.Id = KSMEDIUM_TYPE_ANYINSTANCE as ULONG;
    (*PinConnect).Medium.Flags = 0 as ULONG;
    (*PinConnect).PinToHandle = core::ptr::null_mut();
    (*PinConnect).PinId = PinId;
    (*PinConnect).Priority.PriorityClass = KSPRIORITY_NORMAL as ULONG;
    (*PinConnect).Priority.PrioritySubClass = 1 as ULONG;
}
