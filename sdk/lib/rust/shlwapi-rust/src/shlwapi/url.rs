use crate::shlwapi::string::ChrCmpIW;
use crate::shlwapi::ordinal::MLFreeLibrary;
use crate::shlwapi::ordinal::MLLoadLibraryW;
use crate::shlwapi::string::StrToIntExW;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_26 {
    pub scheme_number: URL_SCHEME,
    pub scheme_name: [WCHAR; 12],
}

pub const INTERNET_MAX_URL_LENGTH: usize = (INTERNET_MAX_SCHEME_LENGTH as usize)
    .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
    .wrapping_add(INTERNET_MAX_PATH_LENGTH as usize);

static mut shlwapi_schemes: [C2RustUnnamed_26; 18] = [
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_FTP,
        scheme_name: [
            'f' as i32 as WCHAR,
            't' as i32 as WCHAR,
            'p' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: (URL_SCHEME_HTTP as i32),
        scheme_name: [
            'h' as i32 as WCHAR,
            't' as i32 as WCHAR,
            't' as i32 as WCHAR,
            'p' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_GOPHER,
        scheme_name: [
            'g' as i32 as WCHAR,
            'o' as i32 as WCHAR,
            'p' as i32 as WCHAR,
            'h' as i32 as WCHAR,
            'e' as i32 as WCHAR,
            'r' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_MAILTO,
        scheme_name: [
            'm' as i32 as WCHAR,
            'a' as i32 as WCHAR,
            'i' as i32 as WCHAR,
            'l' as i32 as WCHAR,
            't' as i32 as WCHAR,
            'o' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_NEWS,
        scheme_name: [
            'n' as i32 as WCHAR,
            'e' as i32 as WCHAR,
            'w' as i32 as WCHAR,
            's' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_NNTP,
        scheme_name: [
            'n' as i32 as WCHAR,
            'n' as i32 as WCHAR,
            't' as i32 as WCHAR,
            'p' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_TELNET,
        scheme_name: [
            't' as i32 as WCHAR,
            'e' as i32 as WCHAR,
            'l' as i32 as WCHAR,
            'n' as i32 as WCHAR,
            'e' as i32 as WCHAR,
            't' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_WAIS,
        scheme_name: [
            'w' as i32 as WCHAR,
            'a' as i32 as WCHAR,
            'i' as i32 as WCHAR,
            's' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: (URL_SCHEME_FILE as i32),
        scheme_name: [
            'f' as i32 as WCHAR,
            'i' as i32 as WCHAR,
            'l' as i32 as WCHAR,
            'e' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_MK,
        scheme_name: [
            'm' as i32 as WCHAR,
            'k' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: (URL_SCHEME_HTTPS as i32),
        scheme_name: [
            'h' as i32 as WCHAR,
            't' as i32 as WCHAR,
            't' as i32 as WCHAR,
            'p' as i32 as WCHAR,
            's' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: (URL_SCHEME_SHELL as i32),
        scheme_name: [
            's' as i32 as WCHAR,
            'h' as i32 as WCHAR,
            'e' as i32 as WCHAR,
            'l' as i32 as WCHAR,
            'l' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_SNEWS,
        scheme_name: [
            's' as i32 as WCHAR,
            'n' as i32 as WCHAR,
            'e' as i32 as WCHAR,
            'w' as i32 as WCHAR,
            's' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_LOCAL,
        scheme_name: [
            'l' as i32 as WCHAR,
            'o' as i32 as WCHAR,
            'c' as i32 as WCHAR,
            'a' as i32 as WCHAR,
            'l' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_JAVASCRIPT,
        scheme_name: [
            'j' as i32 as WCHAR,
            'a' as i32 as WCHAR,
            'v' as i32 as WCHAR,
            'a' as i32 as WCHAR,
            's' as i32 as WCHAR,
            'c' as i32 as WCHAR,
            'r' as i32 as WCHAR,
            'i' as i32 as WCHAR,
            'p' as i32 as WCHAR,
            't' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_VBSCRIPT,
        scheme_name: [
            'v' as i32 as WCHAR,
            'b' as i32 as WCHAR,
            's' as i32 as WCHAR,
            'c' as i32 as WCHAR,
            'r' as i32 as WCHAR,
            'i' as i32 as WCHAR,
            'p' as i32 as WCHAR,
            't' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_ABOUT,
        scheme_name: [
            'a' as i32 as WCHAR,
            'b' as i32 as WCHAR,
            'o' as i32 as WCHAR,
            'u' as i32 as WCHAR,
            't' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_26 {
        scheme_number: URL_SCHEME_RES,
        scheme_name: [
            'r' as i32 as WCHAR,
            'e' as i32 as WCHAR,
            's' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    },
];

static mut hexDigits: [CHAR; 17] =
    unsafe { ::core::mem::transmute::<[u8; 17], [CHAR; 17]>(*b"0123456789ABCDEF\0") };

static mut fileW: [WCHAR; 5] = [
    'f' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

static mut HashDataLookup: [::core::ffi::c_uchar; 256] = [
    0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x6e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x61 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x7d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x1b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xe9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xc5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xea as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x1e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x41 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xd5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x68 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x39 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xd3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xdf as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x30 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x73 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xf as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x43 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xd2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x1c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xc as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x67 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x3a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x4e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xee as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x90 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x56 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x3c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xc6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xfe as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x58 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x23 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xce as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xfc as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x66 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xf0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x6a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x5d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xa6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xc3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x63 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x10 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x48 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xf5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x69 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x1d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xc2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x65 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x3b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x6c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xe8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x7f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x5b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xcf as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x62 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x54 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x12 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xc7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xd as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x5e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xdc as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xe0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xd4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xc4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xf9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xec as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x2d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xf4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x6f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xd9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xca as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xe7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xe6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xe3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x3e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x72 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xcd as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x2f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x2c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x26 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x95 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xd8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x27 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x92 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xb8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xc8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xde as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xeb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x98 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x7b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xcb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x78 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
];

pub const szResLen: usize = (::core::mem::size_of::<[WCHAR; 7]>() as usize)
    .wrapping_sub(::core::mem::size_of::<WCHAR>() as usize)
    .wrapping_div(::core::mem::size_of::<WCHAR>() as usize);

// WINE_DEFAULT_DEBUG_CHANNEL(shell)

#[repr(C)]
pub struct ShlwapiSchemeEntry {
    pub scheme_number: URL_SCHEME,
    pub scheme_name: [WCHAR; 12],
}

#[repr(C)]
pub struct WINE_PARSE_URL {
    pub pScheme: LPCWSTR,      /* [out] start of scheme */
    pub szScheme: DWORD,       /* [out] size of scheme (until colon) */
    pub pUserName: LPCWSTR,    /* [out] start of Username */
    pub szUserName: DWORD,     /* [out] size of Username (until ":" or "@") */
    pub pPassword: LPCWSTR,    /* [out] start of Password */
    pub szPassword: DWORD,     /* [out] size of Password (until "@") */
    pub pHostName: LPCWSTR,    /* [out] start of Hostname */
    pub szHostName: DWORD,     /* [out] size of Hostname (until ":" or "/") */
    pub pPort: LPCWSTR,        /* [out] start of Port */
    pub szPort: DWORD,         /* [out] size of Port (until "/" or eos) */
    pub pQuery: LPCWSTR,       /* [out] start of Query */
    pub szQuery: DWORD,        /* [out] size of Query (until eos) */
}
pub type PWINE_PARSE_URL = *mut WINE_PARSE_URL;

pub const SCHEME: i32 = 0;
pub const HOST: i32 = 1;
pub const PORT: i32 = 2;
pub const USERPASS: i32 = 3;
pub type WINE_URL_SCAN_TYPE = i32;

pub const WINE_URL_BASH_AS_SLASH: i32 = 0x01;
pub const WINE_URL_COLLAPSE_SLASHES: i32 = 0x02;
pub const WINE_URL_ESCAPE_SLASH: i32 = 0x04;
pub const WINE_URL_ESCAPE_HASH: i32 = 0x08;
pub const WINE_URL_ESCAPE_QUESTION: i32 = 0x10;
pub const WINE_URL_STOP_ON_HASH: i32 = 0x20;
pub const WINE_URL_STOP_ON_QUESTION: i32 = 0x40;
unsafe extern "C" fn heap_strdupAtoW(str: *const ::core::ffi::c_char) -> *mut WCHAR {
    let mut ret: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    if !str.is_null() {
        let mut len: DWORD = 0;
        len = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            str as LPCSTR,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        ) as DWORD;
        ret = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (len as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            str as LPCSTR,
            -(1 as ::core::ffi::c_int),
            ret,
            len as ::core::ffi::c_int,
        );
    }
    return ret as *mut WCHAR;
}

unsafe extern "C" fn get_scheme_code(scheme: LPCWSTR, scheme_len: DWORD) -> DWORD {
    let mut i: ::core::ffi::c_uint = 0;
    while (i as usize)
        < (core::mem::size_of_val(&shlwapi_schemes)
            / core::mem::size_of_val(&shlwapi_schemes[0]))
    {
        if scheme_len
            == strlenW(core::ptr::addr_of!(shlwapi_schemes[i as usize].scheme_name) as LPCWSTR)
                as DWORD
            && memicmpW!(
                scheme,
                core::ptr::addr_of!(shlwapi_schemes[i as usize].scheme_name) as LPCWSTR,
                scheme_len,
            ) == 0
        {
            return shlwapi_schemes[i as usize].scheme_number as DWORD;
        }
        i = i.wrapping_add(1);
    }
    return (URL_SCHEME_UNKNOWN) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn ParseURLA(x: LPCSTR, y: *mut PARSEDURLA) -> HRESULT {
    let mut scheme: [WCHAR; (INTERNET_MAX_SCHEME_LENGTH as _)] = [0; (INTERNET_MAX_SCHEME_LENGTH as _)];
    let mut ptr: *const ::core::ffi::c_char = x as *const ::core::ffi::c_char;
    let mut len: ::core::ffi::c_int = 0;
    WINE_TRACE!("%s %p\n", debugstr_a(x as *const ::core::ffi::c_char), y);
    if (*y).cbSize as usize != ::core::mem::size_of::<PARSEDURLA>() as usize {
        return E_INVALIDARG;
    }
    while *ptr as ::core::ffi::c_int != 0
        && (isalnum(*ptr as ::core::ffi::c_int) != 0
            || *ptr as ::core::ffi::c_int == '-' as i32
            || *ptr as ::core::ffi::c_int == '+' as i32
            || *ptr as ::core::ffi::c_int == '.' as i32)
    {
        ptr = ptr.offset(1);
    }
    if *ptr as ::core::ffi::c_int != ':' as i32
        || ptr <= x.offset(1 as isize)
    {
        (*y).pszProtocol = ::core::ptr::null::<CHAR>();
        return URL_E_INVALID_SYNTAX;
    }
    (*y).pszProtocol = x;
    (*y).cchProtocol = ptr.offset_from(x) as ::core::ffi::c_longlong as UINT;
    (*y).pszSuffix = ptr.offset(1 as isize) as LPCSTR;
    (*y).cchSuffix = strlen((*y).pszSuffix as *const ::core::ffi::c_char) as UINT;
    len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        x,
        ptr.offset_from(x) as ::core::ffi::c_longlong as ::core::ffi::c_int,
        &raw mut scheme as LPWSTR,
        (::core::mem::size_of::<[WCHAR; (INTERNET_MAX_SCHEME_LENGTH as _)]>() as usize)
            .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as ::core::ffi::c_int,
    );
    (*y).nScheme = get_scheme_code(&raw mut scheme as *mut WCHAR as LPCWSTR, len as DWORD) as UINT;
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn ParseURLW(x: LPCWSTR, y: *mut PARSEDURLW) -> HRESULT {
    let mut ptr: LPCWSTR = x;

    if (*y).cbSize != core::mem::size_of::<PARSEDURLW>() as DWORD {
        return E_INVALIDARG;
    }

    while *ptr != 0 && (isalnumW!(*ptr) != 0 || *ptr == '-' as WCHAR || *ptr == '+' as WCHAR || *ptr == '.' as WCHAR) {
        ptr = ptr.add(1);
    }

    if *ptr != ':' as WCHAR || ptr <= x.add(1 as usize) {
        (*y).pszProtocol = core::ptr::null_mut();
        return URL_E_INVALID_SYNTAX;
    }

    (*y).pszProtocol = x;
    (*y).cchProtocol = ptr.offset_from(x) as UINT;
    (*y).pszSuffix = ptr.add(1 as usize) as LPCWSTR;
    (*y).cchSuffix = strlenW((*y).pszSuffix) as UINT;
    (*y).nScheme = get_scheme_code(x, ptr.offset_from(x) as DWORD) as DWORD;

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn UrlCanonicalizeA(
    pszUrl: LPCSTR,
    pszCanonicalized: LPSTR,
    pcchCanonicalized: LPDWORD,
    dwFlags: DWORD,
) -> HRESULT {
    let mut url: LPWSTR;
    let mut canonical: LPWSTR;
    let mut ret: HRESULT;

    if pszUrl.is_null() || pszCanonicalized.is_null() || pcchCanonicalized.is_null() || *pcchCanonicalized == 0 {
        return E_INVALIDARG;
    }

    url = heap_strdupAtoW(pszUrl as *const ::core::ffi::c_char) as LPWSTR;
    canonical = HeapAlloc(
        GetProcessHeap(),
        0,
        (*pcchCanonicalized as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if url.is_null() || canonical.is_null() {
        HeapFree(GetProcessHeap(), 0, url as PVOID);
        HeapFree(GetProcessHeap(), 0, canonical as PVOID);
        return E_OUTOFMEMORY;
    }

    ret = UrlCanonicalizeW(url as LPCWSTR, canonical, pcchCanonicalized, dwFlags);
    if ret == S_OK {
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            canonical as LPCWSTR,
            -1 as ::core::ffi::c_int,
            pszCanonicalized,
            (*pcchCanonicalized).wrapping_add(1 as DWORD) as ::core::ffi::c_int,
            ::core::ptr::null::<CHAR>(),
            ::core::ptr::null_mut::<BOOL>(),
        );
    }

    HeapFree(GetProcessHeap(), 0, url as PVOID);
    HeapFree(GetProcessHeap(), 0, canonical as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn UrlCanonicalizeW(
    pszUrl: LPCWSTR,
    pszCanonicalized: LPWSTR,
    pcchCanonicalized: LPDWORD,
    mut dwFlags: DWORD,
) -> HRESULT {
    let mut hr: HRESULT = S_OK as HRESULT;
    let mut EscapeFlags: DWORD = 0;
    let mut wk1: LPCWSTR = core::ptr::null();
    let mut root: LPCWSTR = core::ptr::null();
    let mut lpszUrlCpy: LPWSTR = core::ptr::null_mut();
    let mut url: LPWSTR = core::ptr::null_mut();
    let mut wk2: LPWSTR = core::ptr::null_mut();
    let mut mp: LPWSTR = core::ptr::null_mut();
    let mut mp2: LPWSTR = core::ptr::null_mut();
    let mut state: INT = 0;
    let mut nByteLen: DWORD = 0;
    let mut nLen: DWORD = 0;
    let mut nWkLen: DWORD = 0;
    let mut is_file_url: BOOL = 0;
    let mut slash: WCHAR = '\0' as WCHAR;

    static wszFile: [u16; 5] = [0x0066u16, 0x0069, 0x006C, 0x0065, 0x003A];
    static wszRes: [u16; 4] = [0x0072u16, 0x0065, 0x0073, 0x003A];
    static wszHttp: [u16; 5] = [0x0068u16, 0x0074, 0x0074, 0x0070, 0x003A];
    static wszLocalhost: [u16; 9] = [0x006Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0068, 0x006F, 0x0073, 0x0074];
    static wszFilePrefix: [u16; 8] = [0x0066u16, 0x0069, 0x006C, 0x0065, 0x003A, 0x002F, 0x002F, 0x002F];

    if pszUrl.is_null() || pszCanonicalized.is_null() || pcchCanonicalized.is_null() || *pcchCanonicalized == 0 {
        return E_INVALIDARG as HRESULT;
    }

    if *pszUrl == 0 {
        *pszCanonicalized = 0 as WCHAR;
        return S_OK as HRESULT;
    }

    /* Remove '\t' characters from URL */
    nByteLen = (strlenW(pszUrl) as DWORD + 1 as DWORD) * core::mem::size_of::<WCHAR>() as DWORD; /* length in bytes */
    url = HeapAlloc(GetProcessHeap(), 0 as DWORD, nByteLen as SIZE_T) as LPWSTR;
    if url.is_null() {
        return E_OUTOFMEMORY as HRESULT;
    }

    wk1 = pszUrl;
    wk2 = url;
    loop {
        while *wk1 == '\t' as WCHAR {
            wk1 = wk1.offset(1);
        }
        *wk2 = *wk1;
        wk2 = wk2.offset(1);
        let fresh = *wk1;
        wk1 = wk1.offset(1);
        if !(fresh != 0) {
            break;
        }
    }

    /* Allocate memory for simplified URL (before escaping) */
    nByteLen = (wk2.offset_from(url) as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
    lpszUrlCpy = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        nByteLen as SIZE_T + core::mem::size_of::<[u16; 8]>() as SIZE_T + core::mem::size_of::<WCHAR>() as SIZE_T,
    ) as LPWSTR;
    if lpszUrlCpy.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, url as LPVOID);
        return E_OUTOFMEMORY as HRESULT;
    }

    is_file_url = (strncmpW!(
        core::ptr::addr_of!(wszFile) as LPCWSTR,
        url,
        core::mem::size_of::<[u16; 5]>() / core::mem::size_of::<WCHAR>(),
    ) == 0) as BOOL;

    if (nByteLen >= core::mem::size_of::<[u16; 5]>() as DWORD
        && memcmp(
            core::ptr::addr_of!(wszHttp) as *const core::ffi::c_void,
            url as *const core::ffi::c_void,
            core::mem::size_of::<[u16; 5]>() as usize,
        ) == 0)
        || is_file_url != 0
    {
        slash = '/' as WCHAR;
    }

    if (dwFlags & ((URL_FILE_USE_PATHURL as DWORD) | (URL_WININET_COMPATIBILITY as DWORD))) != 0 && is_file_url != 0 {
        slash = '\\' as WCHAR;
    }

    if nByteLen >= core::mem::size_of::<[u16; 4]>() as DWORD
        && memcmp(
            core::ptr::addr_of!(wszRes) as *const core::ffi::c_void,
            url as *const core::ffi::c_void,
            core::mem::size_of::<[u16; 4]>() as usize,
        ) == 0
    {
        dwFlags &= !(URL_FILE_USE_PATHURL as DWORD);
        slash = '\0' as WCHAR;
    }

    /*
     * state =
     *         0   initial  1,3
     *         1   have 2[+] alnum  2,3
     *         2   have scheme (found :)  4,6,3
     *         3   failed (no location)
     *         4   have //  5,3
     *         5   have 1[+] alnum  6,3
     *         6   have location (found /) save root location
     */

    wk1 = url;
    wk2 = lpszUrlCpy;
    state = 0 as INT;

    if *url.offset(1 as isize) == ':' as WCHAR { /* Assume path */
        memcpy(
            wk2 as *mut core::ffi::c_void,
            core::ptr::addr_of!(wszFilePrefix) as *const core::ffi::c_void,
            core::mem::size_of::<[u16; 8]>() as usize,
        );
        wk2 = wk2.offset((core::mem::size_of::<[u16; 8]>() / core::mem::size_of::<WCHAR>()) as isize);
        if (dwFlags & ((URL_FILE_USE_PATHURL as DWORD) | (URL_WININET_COMPATIBILITY as DWORD))) != 0 {
            slash = '\\' as WCHAR;
            wk2 = wk2.offset(-1);
        } else {
            dwFlags |= URL_ESCAPE_UNSAFE as DWORD;
        }
        state = 5 as INT;
        is_file_url = TRUE as BOOL;
    } else if *url.offset(0 as isize) == '/' as WCHAR {
        state = 5 as INT;
        is_file_url = TRUE as BOOL;
    }

    while *wk1 != 0 {
        'sw: {
            match state {
                0 => {
                    if isalnumW!(*wk1) == 0 {
                        state = 3 as INT;
                        break 'sw;
                    }
                    *wk2 = *wk1;
                    wk2 = wk2.offset(1);
                    wk1 = wk1.offset(1);
                    if isalnumW!(*wk1) == 0 {
                        state = 3 as INT;
                        break 'sw;
                    }
                    *wk2 = *wk1;
                    wk2 = wk2.offset(1);
                    wk1 = wk1.offset(1);
                    state = 1 as INT;
                    break 'sw;
                }
                1 => {
                    *wk2 = *wk1;
                    wk2 = wk2.offset(1);
                    let c = *wk1;
                    wk1 = wk1.offset(1);
                    if c == ':' as WCHAR {
                        state = 2 as INT;
                    }
                    break 'sw;
                }
                2 => {
                    *wk2 = *wk1;
                    wk2 = wk2.offset(1);
                    wk1 = wk1.offset(1);
                    if *wk1 != '/' as WCHAR {
                        state = 6 as INT;
                        break 'sw;
                    }
                    *wk2 = *wk1;
                    wk2 = wk2.offset(1);
                    wk1 = wk1.offset(1);
                    if (dwFlags & (URL_FILE_USE_PATHURL as DWORD)) != 0
                        && nByteLen >= core::mem::size_of::<[u16; 9]>() as DWORD
                        && is_file_url != 0
                        && memcmp(
                            core::ptr::addr_of!(wszLocalhost) as *const core::ffi::c_void,
                            wk1 as *const core::ffi::c_void,
                            core::mem::size_of::<[u16; 9]>() as usize,
                        ) == 0
                    {
                        wk1 = wk1.offset((core::mem::size_of::<[u16; 9]>() / core::mem::size_of::<WCHAR>()) as isize);
                        while *wk1 == '\\' as WCHAR && (dwFlags & (URL_FILE_USE_PATHURL as DWORD)) != 0 {
                            wk1 = wk1.offset(1);
                        }
                    }

                    if *wk1 == '/' as WCHAR && (dwFlags & (URL_FILE_USE_PATHURL as DWORD)) != 0 {
                        wk1 = wk1.offset(1);
                    } else if is_file_url != 0 {
                        let mut body: LPCWSTR = wk1;

                        while *body == '/' as WCHAR {
                            body = body.offset(1);
                        }

                        if isalnumW!(*body) != 0 && *body.offset(1 as isize) == ':' as WCHAR {
                            if (dwFlags & ((URL_WININET_COMPATIBILITY as DWORD) | (URL_FILE_USE_PATHURL as DWORD))) == 0 {
                                if slash != 0 {
                                    *wk2 = slash;
                                    wk2 = wk2.offset(1);
                                } else {
                                    *wk2 = '/' as WCHAR;
                                    wk2 = wk2.offset(1);
                                }
                            }
                        } else {
                            if (dwFlags & (URL_WININET_COMPATIBILITY as DWORD)) != 0 {
                                if *wk1 == '/' as WCHAR && *wk1.offset(1 as isize) != '/' as WCHAR {
                                    *wk2 = '\\' as WCHAR;
                                    wk2 = wk2.offset(1);
                                } else {
                                    *wk2 = '\\' as WCHAR;
                                    wk2 = wk2.offset(1);
                                    *wk2 = '\\' as WCHAR;
                                    wk2 = wk2.offset(1);
                                }
                            } else {
                                if *wk1 == '/' as WCHAR && *wk1.offset(1 as isize) != '/' as WCHAR {
                                    if slash != 0 {
                                        *wk2 = slash;
                                        wk2 = wk2.offset(1);
                                    } else {
                                        *wk2 = '/' as WCHAR;
                                        wk2 = wk2.offset(1);
                                    }
                                }
                            }
                        }
                        wk1 = body;
                    }
                    state = 4 as INT;
                    break 'sw;
                }
                3 => {
                    nWkLen = strlenW(wk1) as DWORD;
                    memcpy(
                        wk2 as *mut core::ffi::c_void,
                        wk1 as *const core::ffi::c_void,
                        ((nWkLen + 1 as DWORD) as usize) * core::mem::size_of::<WCHAR>(),
                    );
                    mp = wk2;
                    wk1 = wk1.offset(nWkLen as isize);
                    wk2 = wk2.offset(nWkLen as isize);

                    if slash != 0 {
                        while mp < wk2 {
                            if *mp == '/' as WCHAR || *mp == '\\' as WCHAR {
                                *mp = slash;
                            }
                            mp = mp.offset(1);
                        }
                    }
                    break 'sw;
                }
                4 => {
                    if isalnumW!(*wk1) == 0 && *wk1 != '-' as WCHAR && *wk1 != '.' as WCHAR && *wk1 != ':' as WCHAR {
                        state = 3 as INT;
                        break 'sw;
                    }
                    while isalnumW!(*wk1) != 0 || *wk1 == '-' as WCHAR || *wk1 == '.' as WCHAR || *wk1 == ':' as WCHAR {
                        *wk2 = *wk1;
                        wk2 = wk2.offset(1);
                        wk1 = wk1.offset(1);
                    }
                    state = 5 as INT;
                    if *wk1 == 0 {
                        if slash != 0 {
                            *wk2 = slash;
                            wk2 = wk2.offset(1);
                        } else {
                            *wk2 = '/' as WCHAR;
                            wk2 = wk2.offset(1);
                        }
                    }
                    break 'sw;
                }
                5 => {
                    if *wk1 != '/' as WCHAR && *wk1 != '\\' as WCHAR {
                        state = 3 as INT;
                        break 'sw;
                    }
                    while *wk1 == '/' as WCHAR || *wk1 == '\\' as WCHAR {
                        if slash != 0 {
                            *wk2 = slash;
                            wk2 = wk2.offset(1);
                        } else {
                            *wk2 = *wk1;
                            wk2 = wk2.offset(1);
                        }
                        wk1 = wk1.offset(1);
                    }
                    state = 6 as INT;
                    break 'sw;
                }
                6 => {
                    if (dwFlags & (URL_DONT_SIMPLIFY as DWORD)) != 0 {
                        state = 3 as INT;
                        break 'sw;
                    }

                    /* Now at root location, cannot back up any more. */
                    /* "root" will point at the '/' */

                    root = wk2.offset(-(1 as isize)) as LPCWSTR;
                    while *wk1 != 0 {
                        mp = strchrW!(wk1, '/' as WCHAR) as LPWSTR;
                        mp2 = strchrW!(wk1, '\\' as WCHAR) as LPWSTR;
                        if !mp2.is_null() && (mp.is_null() || mp2 < mp) {
                            mp = mp2;
                        }
                        if mp.is_null() {
                            nWkLen = strlenW(wk1) as DWORD;
                            memcpy(
                                wk2 as *mut core::ffi::c_void,
                                wk1 as *const core::ffi::c_void,
                                ((nWkLen + 1 as DWORD) as usize) * core::mem::size_of::<WCHAR>(),
                            );
                            wk1 = wk1.offset(nWkLen as isize);
                            wk2 = wk2.offset(nWkLen as isize);
                            continue;
                        }
                        nLen = mp.offset_from(wk1) as DWORD;
                        if nLen != 0 {
                            memcpy(
                                wk2 as *mut core::ffi::c_void,
                                wk1 as *const core::ffi::c_void,
                                (nLen as usize) * core::mem::size_of::<WCHAR>(),
                            );
                            wk2 = wk2.offset(nLen as isize);
                            wk1 = wk1.offset(nLen as isize);
                        }
                        if slash != 0 {
                            *wk2 = slash;
                            wk2 = wk2.offset(1);
                        } else {
                            *wk2 = *wk1;
                            wk2 = wk2.offset(1);
                        }
                        wk1 = wk1.offset(1);

                        while *wk1 == '.' as WCHAR {
                            if *wk1.offset(1 as isize) == '/' as WCHAR
                                || *wk1.offset(1 as isize) == '\\' as WCHAR
                            {
                                /* case of /./ -> skip the ./ */
                                wk1 = wk1.offset(2 as isize);
                            } else if *wk1.offset(1 as isize) == '.' as WCHAR
                                && (*wk1.offset(2 as isize) == '/' as WCHAR
                                    || *wk1.offset(2 as isize) == '\\' as WCHAR
                                    || *wk1.offset(2 as isize) == '?' as WCHAR
                                    || *wk1.offset(2 as isize) == '#' as WCHAR
                                    || *wk1.offset(2 as isize) == 0)
                            {
                                /* case /../ -> need to backup wk2 */
                                *wk2.offset(-(1 as isize)) = '\0' as WCHAR; /* set end of string */
                                mp = strrchrW!(root, '/' as WCHAR) as LPWSTR;
                                mp2 = strrchrW!(root, '\\' as WCHAR) as LPWSTR;
                                if !mp2.is_null() && (mp.is_null() || mp2 < mp) {
                                    mp = mp2;
                                }
                                if !mp.is_null() && mp >= root as LPWSTR {
                                    /* found valid backup point */
                                    wk2 = mp.offset(1 as isize);
                                    if *wk1.offset(2 as isize) != '/' as WCHAR
                                        && *wk1.offset(2 as isize) != '\\' as WCHAR
                                    {
                                        wk1 = wk1.offset(2 as isize);
                                    } else {
                                        wk1 = wk1.offset(3 as isize);
                                    }
                                } else {
                                    /* did not find point, restore '/' */
                                    *wk2.offset(-(1 as isize)) = slash;
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    *wk2 = '\0' as WCHAR;
                    break 'sw;
                }
                _ => {
                    FIXME!("how did we get here - state=%d\n", state);
                    HeapFree(GetProcessHeap(), 0 as DWORD, lpszUrlCpy as LPVOID);
                    HeapFree(GetProcessHeap(), 0 as DWORD, url as LPVOID);
                    return E_INVALIDARG as HRESULT;
                }
            }
        }
        *wk2 = '\0' as WCHAR;
    }

    nLen = lstrlenW(lpszUrlCpy as LPCWSTR) as DWORD;
    while nLen > 0 as DWORD
        && *lpszUrlCpy.offset((nLen - 1 as DWORD) as isize) <= ' ' as WCHAR
    {
        nLen -= 1;
        *lpszUrlCpy.offset(nLen as isize) = 0 as WCHAR;
    }

    if (dwFlags & (URL_UNESCAPE as DWORD)) != 0
        || ((dwFlags & (URL_FILE_USE_PATHURL as DWORD)) != 0
            && nByteLen >= core::mem::size_of::<[u16; 5]>() as DWORD
            && memcmp(
                core::ptr::addr_of!(wszFile) as *const core::ffi::c_void,
                url as *const core::ffi::c_void,
                core::mem::size_of::<[u16; 5]>() as usize,
            ) == 0)
    {
        UrlUnescapeW(
            lpszUrlCpy,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(nLen),
            URL_UNESCAPE_INPLACE as DWORD,
        );
    }

    EscapeFlags = dwFlags
        & ((URL_ESCAPE_UNSAFE as DWORD)
            | (URL_ESCAPE_SPACES_ONLY as DWORD)
            | (URL_ESCAPE_PERCENT as DWORD)
            | (URL_DONT_ESCAPE_EXTRA_INFO as DWORD)
            | (URL_ESCAPE_SEGMENT_ONLY as DWORD));
    if EscapeFlags != 0 {
        EscapeFlags &= !(URL_ESCAPE_UNSAFE as DWORD);
        hr = UrlEscapeW(lpszUrlCpy as LPCWSTR, pszCanonicalized, pcchCanonicalized, EscapeFlags);
    } else {
        /* No escaping needed, just copy the string */
        nLen = lstrlenW(lpszUrlCpy as LPCWSTR) as DWORD;
        if nLen < *pcchCanonicalized {
            memcpy(
                pszCanonicalized as *mut core::ffi::c_void,
                lpszUrlCpy as *const core::ffi::c_void,
                ((nLen + 1 as DWORD) as usize) * core::mem::size_of::<WCHAR>(),
            );
        } else {
            hr = E_POINTER as HRESULT;
            nLen += 1;
        }
        *pcchCanonicalized = nLen;
    }

    HeapFree(GetProcessHeap(), 0 as DWORD, lpszUrlCpy as LPVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, url as LPVOID);

    if hr == S_OK as HRESULT {
        TRACE!("result %s\n", debugstr_w(pszCanonicalized as *const WCHAR));
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn UrlCombineA(
    pszBase: LPCSTR,
    pszRelative: LPCSTR,
    pszCombined: LPSTR,
    pcchCombined: LPDWORD,
    dwFlags: DWORD,
) -> HRESULT {
    let mut base: LPWSTR = core::ptr::null_mut();
    let mut relative: LPWSTR = core::ptr::null_mut();
    let mut combined: LPWSTR = core::ptr::null_mut();
    let mut ret: DWORD = 0;
    let mut len: DWORD = 0;
    let mut len2: DWORD = 0;

    TRACE!(
        "(base %s, Relative %s, Combine size %d, flags %08x) using W version\n",
        debugstr_a(pszBase as *const i8),
        debugstr_a(pszRelative as *const i8),
        if !pcchCombined.is_null() { *pcchCombined } else { 0 as DWORD },
        dwFlags
    );

    if pszBase.is_null() || pszRelative.is_null() || pcchCombined.is_null() {
        return E_INVALIDARG as HRESULT;
    }

    base = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((3 * INTERNET_MAX_URL_LENGTH) as SIZE_T)
            * (core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    relative = base.offset(INTERNET_MAX_URL_LENGTH as isize);
    combined = relative.offset(INTERNET_MAX_URL_LENGTH as isize);

    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        pszBase,
        -(1 as core::ffi::c_int),
        base,
        INTERNET_MAX_URL_LENGTH as core::ffi::c_int,
    );
    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        pszRelative,
        -(1 as core::ffi::c_int),
        relative,
        INTERNET_MAX_URL_LENGTH as core::ffi::c_int,
    );
    len = *pcchCombined;

    ret = UrlCombineW(
        base as LPCWSTR,
        relative as LPCWSTR,
        if !pszCombined.is_null() {
            combined
        } else {
            core::ptr::null_mut()
        },
        core::ptr::addr_of_mut!(len),
        dwFlags,
    ) as DWORD;
    if ret != S_OK as DWORD {
        *pcchCombined = len;
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            base as PVOID,
        );
        return ret as HRESULT;
    }

    len2 = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        combined as LPCWSTR,
        len as core::ffi::c_int,
        core::ptr::null_mut(),
        0 as core::ffi::c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) as DWORD;
    if len2 > *pcchCombined {
        *pcchCombined = len2;
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            base as PVOID,
        );
        return E_POINTER as HRESULT;
    }
    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        combined as LPCWSTR,
        (len + 1) as core::ffi::c_int,
        pszCombined,
        ((*pcchCombined) + 1) as core::ffi::c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    *pcchCombined = len2;
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        base as PVOID,
    );
    return S_OK as HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn UrlCombineW(
    pszBase: LPCWSTR,
    pszRelative: LPCWSTR,
    pszCombined: LPWSTR,
    pcchCombined: LPDWORD,
    dwFlags: DWORD,
) -> HRESULT {
    let mut base: PARSEDURLW = core::mem::zeroed();
    let mut relative: PARSEDURLW = core::mem::zeroed();
    let mut myflags: DWORD = 0;
    let mut sizeloc: DWORD = 0;
    let mut i: DWORD = 0;
    let mut len: DWORD = 0;
    let mut res1: DWORD = 0;
    let mut res2: DWORD = 0;
    let mut process_case: DWORD = 0;
    let mut work: LPWSTR = core::ptr::null_mut();
    let mut preliminary: LPWSTR = core::ptr::null_mut();
    let mut mbase: LPWSTR = core::ptr::null_mut();
    let mut mrelative: LPWSTR = core::ptr::null_mut();
    static myfilestr: [WCHAR; 9] = [0x0066u16, 0x0069, 0x006C, 0x0065, 0x003A, 0x002F, 0x002F, 0x002F, 0x0000];
    static fragquerystr: [WCHAR; 3] = [0x0023u16, 0x003F, 0x0000];
    let mut ret: HRESULT = 0;

    if pszBase.is_null() || pszRelative.is_null() || pcchCombined.is_null() {
        return E_INVALIDARG;
    }

    base.cbSize = core::mem::size_of::<PARSEDURLW>() as DWORD;
    relative.cbSize = core::mem::size_of::<PARSEDURLW>() as DWORD;

    /* Get space for duplicates of the input and the output */
    preliminary = HeapAlloc(
        GetProcessHeap(),
        0,
        ((3 * INTERNET_MAX_URL_LENGTH) as SIZE_T)
            * (core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    mbase = preliminary.add(INTERNET_MAX_URL_LENGTH as usize);
    mrelative = mbase.add(INTERNET_MAX_URL_LENGTH as usize);
    *preliminary = '\0' as WCHAR;

    /* Canonicalize the base input prior to looking for the scheme */
    myflags = dwFlags & (URL_DONT_SIMPLIFY | URL_UNESCAPE);
    len = (INTERNET_MAX_URL_LENGTH) as _;
    UrlCanonicalizeW(pszBase, mbase, core::ptr::addr_of_mut!(len), myflags);

    /* Canonicalize the relative input prior to looking for the scheme */
    len = (INTERNET_MAX_URL_LENGTH) as _;
    UrlCanonicalizeW(pszRelative, mrelative, core::ptr::addr_of_mut!(len), myflags);

    /* See if the base has a scheme */
    res1 = ParseURLW(mbase as LPCWSTR, core::ptr::addr_of_mut!(base)) as DWORD;
    if res1 != 0 {
        /* if pszBase has no scheme, then return pszRelative */
        process_case = 1;
    } else {
        loop {
            let mut manual_search: BOOL = (FALSE as i32);

            work = base.pszProtocol as LPWSTR;
            i = 0;
            while i < base.cchProtocol {
                *work.add(i as usize) = tolowerW(*work.add(i as usize));
                i += 1;
            }

            /* mk is a special case */
            if base.nScheme == (URL_SCHEME_MK as u32) {
                static wsz: [WCHAR; 3] = [0x003Au16, 0x003A, 0x0000];

                let mut ptr: LPWSTR =
                    strstrW!(base.pszSuffix, core::ptr::addr_of!(wsz) as LPCWSTR) as LPWSTR;
                if !ptr.is_null() {
                    let mut delta: i32 = 0;

                    ptr = ptr.add(2 as usize);
                    delta = ptr.offset_from(base.pszSuffix) as i32;
                    base.cchProtocol = base.cchProtocol.wrapping_add(delta as DWORD);
                    base.pszSuffix = base.pszSuffix.offset(delta as isize);
                    base.cchSuffix = base.cchSuffix.wrapping_sub(delta as DWORD);
                }
            } else {
                /* get size of location field (if it exists) */
                work = base.pszSuffix as LPWSTR;
                sizeloc = 0;
                let fresh43 = work;
                work = work.add(1);
                if *fresh43 == '/' as WCHAR {
                    let fresh44 = work;
                    work = work.add(1);
                    if *fresh44 == '/' as WCHAR {
                        /* At this point have start of location and
                         * it ends at next '/' or end of string.
                         */
                        while *work != 0 && *work != '/' as WCHAR {
                            work = work.add(1);
                        }
                        sizeloc = work.offset_from(base.pszSuffix) as DWORD;
                    }
                }
            }

            /* If there is a '?', then the remaining part can only contain a
             * query string or fragment, so start looking for the last leaf
             * from the '?'. Otherwise, if there is a '#' and the characters
             * immediately preceding it are ".htm[l]", then begin looking for
             * the last leaf starting from the '#'. Otherwise the '#' is not
             * meaningful and just start looking from the end. */
            work = strpbrkW!(
                base.pszSuffix.add(sizeloc as usize),
                core::ptr::addr_of!(fragquerystr) as LPCWSTR,
            ) as LPWSTR;
            if !work.is_null() {
                static htmlW: [WCHAR; 6] = [0x002Eu16, 0x0068, 0x0074, 0x006D, 0x006C, 0x0000];
                let len_htmlW: i32 = 5;
                static htmW: [WCHAR; 5] = [0x002Eu16, 0x0068, 0x0074, 0x006D, 0x0000];
                let len_htmW: i32 = 4;

                if *work == '?' as WCHAR
                    || base.nScheme == URL_SCHEME_HTTP as UINT
                    || base.nScheme == URL_SCHEME_HTTPS as UINT
                {
                    manual_search = (TRUE as i32);
                } else if work.offset_from(base.pszSuffix) > len_htmW as isize {
                    work = work.offset(-(len_htmW as isize));
                    if strncmpiW!(
                        work as LPCWSTR,
                        core::ptr::addr_of!(htmW) as LPCWSTR,
                        len_htmW,
                    ) == 0
                    {
                        manual_search = (TRUE as i32);
                    }
                    work = work.offset(len_htmW as isize);
                }

                if manual_search == 0 && work.offset_from(base.pszSuffix) > len_htmlW as isize {
                    work = work.offset(-(len_htmlW as isize));
                    if strncmpiW!(
                        work as LPCWSTR,
                        core::ptr::addr_of!(htmlW) as LPCWSTR,
                        len_htmlW,
                    ) == 0
                    {
                        manual_search = (TRUE as i32);
                    }
                    work = work.offset(len_htmlW as isize);
                }
            }

            if manual_search != 0 {
                /* search backwards starting from the current position */
                while *work != '/' as WCHAR
                    && (work as *const WCHAR) > base.pszSuffix.add(sizeloc as usize)
                {
                    work = work.offset(-1);
                }
                base.cchSuffix =
                    (work.offset_from(base.pszSuffix) + 1 as isize) as DWORD;
            } else {
                /* search backwards starting from the end of the string */
                work = strrchrW!(base.pszSuffix.add(sizeloc as usize), '/' as WCHAR) as LPWSTR;
                if !work.is_null() {
                    len = (work.offset_from(base.pszSuffix) + 1 as isize)
                        as DWORD;
                    base.cchSuffix = len;
                } else {
                    base.cchSuffix = sizeloc;
                }
            }

            /*
             * At this point:
             *    .pszSuffix   points to location (starting with '//')
             *    .cchSuffix   length of location (above) and rest less the last
             *                 leaf (if any)
             *    sizeloc   length of location (above) up to but not including
             *              the last '/'
             */

            res2 = ParseURLW(mrelative as LPCWSTR, core::ptr::addr_of_mut!(relative))
                as DWORD;
            if res2 != 0 {
                /* no scheme in pszRelative */
                relative.pszSuffix = mrelative as LPCWSTR; /* case 3,4,5 depends on this */
                relative.cchSuffix = strlenW(mrelative as LPCWSTR) as DWORD;
                if *pszRelative == ':' as WCHAR {
                    /* case that is either left alone or uses pszBase */
                    if dwFlags & URL_PLUGGABLE_PROTOCOL != 0 {
                        process_case = 5;
                        break;
                    }
                    process_case = 1;
                    break;
                }
                if isalnumW!(*mrelative) != 0
                    && *mrelative.add(1 as usize) == ':' as WCHAR
                {
                    /* case that becomes "file:///" */
                    strcpyW!(preliminary, core::ptr::addr_of!(myfilestr) as LPCWSTR);
                    process_case = 1;
                    break;
                }
                if *mrelative == '/' as WCHAR
                    && *mrelative.add(1 as usize) == '/' as WCHAR
                {
                    /* pszRelative has location and rest */
                    process_case = 3;
                    break;
                }
                if *mrelative == '/' as WCHAR {
                    /* case where pszRelative is root to location */
                    process_case = 4;
                    break;
                }
                if *mrelative == '#' as WCHAR {
                    work = strchrW!(
                        base.pszSuffix.add(base.cchSuffix as usize),
                        '#' as WCHAR,
                    ) as LPWSTR;
                    if work.is_null() {
                        work = (base.pszSuffix as LPWSTR)
                            .add(strlenW(base.pszSuffix) as usize);
                    }

                    memcpy(
                        preliminary as *mut core::ffi::c_void,
                        base.pszProtocol as *const core::ffi::c_void,
                        (work.offset_from(base.pszProtocol) as usize)
                            * core::mem::size_of::<WCHAR>(),
                    );
                    *preliminary.offset(work.offset_from(base.pszProtocol)) = '\0' as WCHAR;
                    process_case = 1;
                    break;
                }
                process_case = if *base.pszSuffix == '/' as WCHAR
                    || base.nScheme == (URL_SCHEME_MK as u32)
                {
                    5
                } else {
                    3
                };
                break;
            } else {
                work = relative.pszProtocol as LPWSTR;
                i = 0;
                while i < relative.cchProtocol {
                    *work.add(i as usize) = tolowerW(*work.add(i as usize));
                    i += 1;
                }
            }

            /* handle cases where pszRelative has scheme */
            if base.cchProtocol == relative.cchProtocol
                && strncmpW!(
                    base.pszProtocol as LPCWSTR,
                    relative.pszProtocol as LPCWSTR,
                    base.cchProtocol as i32,
                ) == 0
            {
                /* since the schemes are the same */
                if *relative.pszSuffix == '/' as WCHAR
                    && *relative.pszSuffix.add(1 as usize) == '/' as WCHAR
                {
                    /* case where pszRelative replaces location and following */
                    process_case = 3;
                    break;
                }
                if *relative.pszSuffix == '/' as WCHAR {
                    /* case where pszRelative is root to location */
                    process_case = 4;
                    break;
                }
                /* replace either just location if base's location starts with a
                 * slash or otherwise everything */
                process_case = if *base.pszSuffix == '/' as WCHAR {
                    5
                } else {
                    1
                };
                break;
            }
            if *relative.pszSuffix == '/' as WCHAR
                && *relative.pszSuffix.add(1 as usize) == '/' as WCHAR
            {
                /* case where pszRelative replaces scheme, location,
                 * and following and handles PLUGGABLE
                 */
                process_case = 2;
                break;
            }
            process_case = 1;
            break;

            #[allow(unreachable_code)]
            if FALSE == 0 {
                break;
            }
        }
    }

    ret = S_OK;
    match process_case {
        1 => {
            /*
             * Return pszRelative appended to what ever is in pszCombined,
             * (which may the string "file:///"
             */
            strcatW!(preliminary, mrelative as LPCWSTR);
        }
        2 => {
            /* case where pszRelative replaces scheme, and location */
            strcpyW!(preliminary, mrelative as LPCWSTR);
        }
        3 => {
            /*
             * Return the pszBase scheme with pszRelative. Basically
             * keeps the scheme and replaces the domain and following.
             */
            memcpy(
                preliminary as *mut core::ffi::c_void,
                base.pszProtocol as *const core::ffi::c_void,
                ((base.cchProtocol + 1) as usize)
                    * core::mem::size_of::<WCHAR>(),
            );
            work = preliminary.add((base.cchProtocol + 1) as usize);
            strcpyW!(work, relative.pszSuffix);
        }
        4 => {
            /*
             * Return the pszBase scheme and location but everything
             * after the location is pszRelative. (Replace document
             * from root on.)
             */
            memcpy(
                preliminary as *mut core::ffi::c_void,
                base.pszProtocol as *const core::ffi::c_void,
                ((base.cchProtocol + 1 + sizeloc) as usize)
                    * core::mem::size_of::<WCHAR>(),
            );
            work = preliminary.add((base.cchProtocol + 1 + sizeloc) as usize);
            if dwFlags & URL_PLUGGABLE_PROTOCOL != 0 {
                let fresh45 = work;
                work = work.add(1);
                *fresh45 = '/' as WCHAR;
            }
            strcpyW!(work, relative.pszSuffix);
        }
        5 => {
            /*
             * Return the pszBase without its document (if any) and
             * append pszRelative after its scheme.
             */
            memcpy(
                preliminary as *mut core::ffi::c_void,
                base.pszProtocol as *const core::ffi::c_void,
                ((base.cchProtocol + 1 + base.cchSuffix) as usize)
                    * core::mem::size_of::<WCHAR>(),
            );
            work = preliminary.offset(
                (base.cchProtocol + 1 + base.cchSuffix) as isize
                    - 1 as isize,
            );
            let fresh46 = work;
            work = work.add(1);
            if *fresh46 != '/' as WCHAR {
                let fresh47 = work;
                work = work.add(1);
                *fresh47 = '/' as WCHAR;
            }
            if *relative.pszSuffix.add(0 as usize) == '.' as WCHAR
                && *relative.pszSuffix.add(1 as usize)
                    == 0 as WCHAR
            {
                *work = 0 as WCHAR;
            } else {
                strcpyW!(work, relative.pszSuffix);
            }
        }
        _ => {
            FIXME!(
                "How did we get here????? process_case=%d\n",
                process_case
            );
            ret = E_INVALIDARG;
        }
    }

    if ret == S_OK {
        /* Reuse mrelative as temp storage as it's already allocated and not needed anymore */
        if *pcchCombined == 0 {
            *pcchCombined = 1;
        }
        ret = UrlCanonicalizeW(
            preliminary as LPCWSTR,
            mrelative,
            pcchCombined,
            dwFlags & !(URL_FILE_USE_PATHURL),
        );
        if ret >= 0 && !pszCombined.is_null() {
            lstrcpyW(pszCombined, mrelative as LPCWSTR);
        }
    }
    HeapFree(
        GetProcessHeap(),
        0,
        preliminary as PVOID,
    );
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn UrlEscapeA(
    pszUrl: LPCSTR,
    pszEscaped: LPSTR,
    pcchEscaped: LPDWORD,
    dwFlags: DWORD,
) -> HRESULT {
    let mut bufW: [WCHAR; INTERNET_MAX_URL_LENGTH] = [0; INTERNET_MAX_URL_LENGTH];
    let mut escapedW: *mut WCHAR = core::ptr::addr_of_mut!(bufW) as *mut WCHAR;
    let mut urlW: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: HRESULT = 0;
    let mut lenW: DWORD = (core::mem::size_of::<[WCHAR; INTERNET_MAX_URL_LENGTH]>()
        / core::mem::size_of::<WCHAR>()) as DWORD;
    let mut lenA: DWORD = 0;

    if pszEscaped.is_null() || pcchEscaped.is_null() || *pcchEscaped == 0 {
        return E_INVALIDARG;
    }

    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(urlW), pszUrl) == 0 {
        return E_INVALIDARG;
    }
    if dwFlags & (URL_ESCAPE_AS_UTF8 as DWORD) != 0 {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(urlW));
        return E_NOTIMPL;
    }
    ret = UrlEscapeW(
        urlW.Buffer as LPCWSTR,
        escapedW as LPWSTR,
        core::ptr::addr_of_mut!(lenW),
        dwFlags,
    );
    if ret == E_POINTER {
        escapedW = HeapAlloc(
            GetProcessHeap(),
            0,
            ((lenW as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as SIZE_T,
        ) as *mut WCHAR;
        ret = UrlEscapeW(
            urlW.Buffer as LPCWSTR,
            escapedW as LPWSTR,
            core::ptr::addr_of_mut!(lenW),
            dwFlags,
        );
    }
    if ret == S_OK {
        RtlUnicodeToMultiByteSize(
            core::ptr::addr_of_mut!(lenA),
            escapedW as PCWSTR,
            (lenW as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
        );
        if *pcchEscaped > lenA {
            RtlUnicodeToMultiByteN(
                pszEscaped,
                (*pcchEscaped).wrapping_sub(1 as DWORD),
                core::ptr::addr_of_mut!(lenA),
                escapedW as LPCWSTR,
                (lenW as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            );
            *pszEscaped.add(lenA as usize) = 0 as CHAR;
            *pcchEscaped = lenA;
        } else {
            *pcchEscaped = lenA.wrapping_add(1 as DWORD);
            ret = E_POINTER;
        }
    }
    if escapedW != core::ptr::addr_of_mut!(bufW) as *mut WCHAR {
        HeapFree(
            GetProcessHeap(),
            0,
            escapedW as PVOID,
        );
    }
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(urlW));
    return ret;
}

unsafe extern "C" fn URL_NeedEscapeW(mut ch: WCHAR, mut flags: DWORD, mut int_flags: DWORD) -> BOOL {
    if (flags & URL_ESCAPE_SPACES_ONLY) != 0 {
        return ((ch as ::core::ffi::c_int) == (' ' as ::core::ffi::c_int)) as BOOL;
    }

    if ((flags & URL_ESCAPE_PERCENT) != 0) && ((ch as ::core::ffi::c_int) == ('%' as ::core::ffi::c_int)) {
        return (TRUE) as BOOL;
    }

    if ((flags & URL_ESCAPE_AS_UTF8) != 0) && ((ch as ::core::ffi::c_int) >= 0x80) {
        return (TRUE) as BOOL;
    }

    if ((ch as ::core::ffi::c_int) <= 31)
        || (((ch as ::core::ffi::c_int) >= 127) && ((ch as ::core::ffi::c_int) <= 255))
    {
        return (TRUE) as BOOL;
    }

    if isalnumW!(ch) != 0 {
        return (FALSE) as BOOL;
    }

    match ch as ::core::ffi::c_int {
        32 | 60 | 62 | 34 | 123 | 125 | 124 | 92 | 94 | 93 | 91 | 96 | 38 => {
            return (TRUE) as BOOL;
        }
        47 => {
            return (((int_flags & (WINE_URL_ESCAPE_SLASH as DWORD)) != 0) as ::core::ffi::c_int) as BOOL;
        }
        63 => {
            return (((int_flags & (WINE_URL_ESCAPE_QUESTION as DWORD)) != 0) as ::core::ffi::c_int) as BOOL;
        }
        35 => {
            return (((int_flags & (WINE_URL_ESCAPE_HASH as DWORD)) != 0) as ::core::ffi::c_int) as BOOL;
        }
        _ => {
            return (FALSE) as BOOL;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn UrlEscapeW(
    pszUrl: LPCWSTR,
    pszEscaped: LPWSTR,
    pcchEscaped: LPDWORD,
    mut dwFlags: DWORD,
) -> HRESULT {
    let mut src: LPCWSTR = core::ptr::null();
    let mut needed: DWORD = 0;
    let mut ret: DWORD = 0;
    let mut stop_escaping: BOOL = (FALSE as i32);
    let mut next: [WCHAR; 12] = [0; 12];
    let mut dst: *mut WCHAR = core::ptr::null_mut();
    let mut dst_ptr: *mut WCHAR = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut len: INT = 0;
    let mut parsed_url: PARSEDURLW = core::mem::zeroed();
    let mut int_flags: DWORD = 0;
    let mut slashes: DWORD = 0;
    static localhost: [WCHAR; 10] = [0x006Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0068, 0x006F, 0x0073, 0x0074, 0x0000];

    TRACE!(
        "(%p(%s) %p %p 0x%08x)\n",
        pszUrl,
        debugstr_w(pszUrl),
        pszEscaped,
        pcchEscaped,
        dwFlags
    );

    if pszUrl.is_null() || pcchEscaped.is_null() || pszEscaped.is_null() || *pcchEscaped == 0 {
        return E_INVALIDARG;
    }

    if (dwFlags & !(URL_ESCAPE_SPACES_ONLY | URL_ESCAPE_SEGMENT_ONLY | URL_DONT_ESCAPE_EXTRA_INFO | URL_ESCAPE_PERCENT | URL_ESCAPE_AS_UTF8)) != 0 {
        FIXME!("Unimplemented flags: %08x\n", dwFlags);
    }

    dst = HeapAlloc(
        GetProcessHeap(),
        0,
        (*pcchEscaped as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;
    dst_ptr = dst;
    if dst_ptr.is_null() {
        return E_OUTOFMEMORY;
    }

    /* fix up flags */
    if (dwFlags & URL_ESCAPE_SPACES_ONLY) != 0 {
        /* if SPACES_ONLY specified, reset the other controls */
        dwFlags &= !(URL_DONT_ESCAPE_EXTRA_INFO | URL_ESCAPE_PERCENT | URL_ESCAPE_SEGMENT_ONLY);
    } else {
        /* if SPACES_ONLY *not* specified the assume DONT_ESCAPE_EXTRA_INFO */
        dwFlags |= URL_DONT_ESCAPE_EXTRA_INFO;
    }

    int_flags = 0;
    if (dwFlags & URL_ESCAPE_SEGMENT_ONLY) != 0 {
        int_flags = (WINE_URL_ESCAPE_QUESTION | WINE_URL_ESCAPE_HASH | WINE_URL_ESCAPE_SLASH) as u32;
    } else {
        parsed_url.cbSize = core::mem::size_of::<PARSEDURLW>() as DWORD;
        if ParseURLW(pszUrl, core::ptr::addr_of_mut!(parsed_url)) != S_OK {
            parsed_url.nScheme = (URL_SCHEME_INVALID) as u32;
        }

        TRACE!(
            "scheme = %d (%s)\n",
            parsed_url.nScheme,
            debugstr_wn(parsed_url.pszProtocol, parsed_url.cchProtocol as INT)
        );

        if (dwFlags & URL_DONT_ESCAPE_EXTRA_INFO) != 0 {
            int_flags = (WINE_URL_STOP_ON_HASH | WINE_URL_STOP_ON_QUESTION) as u32;
        }

        match (parsed_url.nScheme) as i32 {
            URL_SCHEME_FILE => {
                int_flags |= (WINE_URL_BASH_AS_SLASH | WINE_URL_COLLAPSE_SLASHES | WINE_URL_ESCAPE_HASH) as u32;
                int_flags &= (!WINE_URL_STOP_ON_HASH) as u32;
            }
            URL_SCHEME_HTTP | URL_SCHEME_HTTPS => {
                int_flags |= (WINE_URL_BASH_AS_SLASH as u32);
                if *parsed_url.pszSuffix.offset(0 as isize) != '/' as WCHAR
                    && *parsed_url.pszSuffix.offset(0 as isize) != '\\' as WCHAR
                {
                    int_flags |= (WINE_URL_ESCAPE_SLASH as u32);
                }
            }
            URL_SCHEME_MAILTO => {
                int_flags |= (WINE_URL_ESCAPE_SLASH | WINE_URL_ESCAPE_QUESTION | WINE_URL_ESCAPE_HASH) as u32;
                int_flags &= (!(WINE_URL_STOP_ON_QUESTION | WINE_URL_STOP_ON_HASH)) as u32;
            }
            URL_SCHEME_INVALID => {}
            URL_SCHEME_FTP | _ => {
                if *parsed_url.pszSuffix.offset(0 as isize) != '/' as WCHAR {
                    int_flags |= (WINE_URL_ESCAPE_SLASH as u32);
                }
            }
        }
    }

    src = pszUrl;
    while *src != 0 {
        let mut cur: WCHAR = *src;
        len = 0;

        if (int_flags & (WINE_URL_COLLAPSE_SLASHES as DWORD)) != 0
            && src == pszUrl.offset(parsed_url.cchProtocol as isize + 1 as isize)
        {
            let mut localhost_len: INT = (core::mem::size_of::<[WCHAR; 10]>() / core::mem::size_of::<WCHAR>()) as INT
                - 1;
            while cur == '/' as WCHAR || cur == '\\' as WCHAR {
                slashes = slashes.wrapping_add(1);
                src = src.offset(1);
                cur = *src;
            }
            if slashes == 2
                && strncmpiW!(src, core::ptr::addr_of!(localhost) as *const WCHAR, localhost_len) == 0
            {
                /* file://localhost/ -> file:/// */
                if *src.offset(localhost_len as isize) == '/' as WCHAR
                    || *src.offset(localhost_len as isize) == '\\' as WCHAR
                {
                    src = src.offset((localhost_len + 1) as isize);
                }
                slashes = 3;
            }

            match slashes {
                1 | 3 => {
                    next[2 as usize] = '/' as WCHAR;
                    next[1 as usize] = next[2 as usize];
                    next[0 as usize] = next[1 as usize];
                    len = 3;
                }
                0 => {
                    len = 0;
                }
                _ => {
                    next[1 as usize] = '/' as WCHAR;
                    next[0 as usize] = next[1 as usize];
                    len = 2;
                }
            }
        }
        if len == 0 {
            if cur == '#' as WCHAR && (int_flags & (WINE_URL_STOP_ON_HASH as DWORD)) != 0 {
                stop_escaping = (TRUE as i32);
            }

            if cur == '?' as WCHAR && (int_flags & (WINE_URL_STOP_ON_QUESTION as DWORD)) != 0 {
                stop_escaping = (TRUE as i32);
            }

            if cur == '\\' as WCHAR && (int_flags & (WINE_URL_BASH_AS_SLASH as DWORD)) != 0 && stop_escaping == 0 {
                cur = '/' as WCHAR;
            }

            if URL_NeedEscapeW(cur, dwFlags, int_flags) != 0 && stop_escaping == (FALSE as BOOL) {
                if (dwFlags & URL_ESCAPE_AS_UTF8) != 0 {
                    let mut utf: [core::ffi::c_char; 16] = [0; 16];

                    if (cur >= 0xd800 && cur <= 0xdfff)
                        && (*src.offset(1 as isize) >= 0xdc00
                            && *src.offset(1 as isize) <= 0xdfff)
                    {
                        len = WideCharToMultiByte(
                            CP_UTF8,
                            0,
                            src,
                            2,
                            core::ptr::addr_of_mut!(utf) as LPSTR,
                            core::mem::size_of::<[core::ffi::c_char; 16]>() as INT,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                        ) as INT;
                        src = src.offset(1);
                    } else {
                        len = WideCharToMultiByte(
                            CP_UTF8,
                            0,
                            core::ptr::addr_of_mut!(cur) as LPCWSTR,
                            1,
                            core::ptr::addr_of_mut!(utf) as LPSTR,
                            core::mem::size_of::<[core::ffi::c_char; 16]>() as INT,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                        ) as INT;
                    }

                    if len == 0 {
                        utf[0 as usize] = 0xefu8 as core::ffi::c_char;
                        utf[1 as usize] = 0xbfu8 as core::ffi::c_char;
                        utf[2 as usize] = 0xbdu8 as core::ffi::c_char;
                        len = 3;
                    }

                    i = 0;
                    while i < len {
                        next[(i * 3 + 0) as usize] = '%' as WCHAR;
                        next[(i * 3 + 1) as usize] = hexDigits
                            [(((utf[i as usize] as INT) >> 4) & 0xf) as usize]
                            as WCHAR;
                        next[(i * 3 + 2) as usize] = hexDigits
                            [((utf[i as usize] as INT) & 0xf) as usize]
                            as WCHAR;
                        i += 1;
                    }
                    len *= 3;
                } else {
                    next[0 as usize] = '%' as WCHAR;
                    next[1 as usize] = hexDigits
                        [(((cur as INT) >> 4) & 0xf) as usize]
                        as WCHAR;
                    next[2 as usize] = hexDigits
                        [((cur as INT) & 0xf) as usize]
                        as WCHAR;
                    len = 3;
                }
            } else {
                next[0 as usize] = cur;
                len = 1;
            }
            src = src.offset(1);
        }

        if needed.wrapping_add(len as DWORD) <= *pcchEscaped {
            memcpy(
                dst as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(next) as *const core::ffi::c_void,
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
            );
            dst = dst.offset(len as isize);
        }
        needed = needed.wrapping_add(len as DWORD);
    }

    if needed < *pcchEscaped {
        *dst = '\0' as WCHAR;
        memcpy(
            pszEscaped as *mut core::ffi::c_void,
            dst_ptr as *const core::ffi::c_void,
            (needed.wrapping_add(1 as DWORD) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );

        ret = (S_OK) as u32;
    } else {
        needed = needed.wrapping_add(1); /* add one for the '\0' */
        ret = (E_POINTER) as u32;
    }
    *pcchEscaped = needed;

    HeapFree(GetProcessHeap(), 0, dst_ptr as LPVOID);
    return ret as HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn UrlUnescapeA(
    pszUrl: LPSTR,
    pszUnescaped: LPSTR,
    pcchUnescaped: LPDWORD,
    dwFlags: DWORD,
) -> HRESULT {
    let mut dst: *mut ::core::ffi::c_char = core::ptr::null_mut();
    let mut next: ::core::ffi::c_char = 0;
    let mut src: LPCSTR = core::ptr::null();
    let mut ret: HRESULT = 0;
    let mut needed: DWORD = 0;
    let mut stop_unescaping: BOOL = (FALSE as i32);

    WINE_TRACE!("(%s, %p, %p, 0x%08x)\n", debugstr_a(pszUrl as *const ::core::ffi::c_char), pszUnescaped, pcchUnescaped, dwFlags);

    if !pszUrl.is_null() {
    } else {
        return E_INVALIDARG;
    }

    if (dwFlags & URL_UNESCAPE_INPLACE) != 0 {
        dst = pszUrl as *mut ::core::ffi::c_char;
    } else {
        if pszUnescaped.is_null() || pcchUnescaped.is_null() {
            return E_INVALIDARG;
        }
        dst = pszUnescaped as *mut ::core::ffi::c_char;
    }

    src = pszUrl as LPCSTR;
    needed = 0;
    while *src != 0 {
        if (dwFlags & URL_DONT_UNESCAPE_EXTRA_INFO) != 0
            && (*src as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                || *src as ::core::ffi::c_int == '?' as ::core::ffi::c_int)
        {
            stop_unescaping = (TRUE as i32);
            next = *src;
        } else if *src as ::core::ffi::c_int == '%' as ::core::ffi::c_int
            && isxdigit(
                *src.offset(1 as isize) as ::core::ffi::c_int,
            ) != 0
            && isxdigit(
                *src.offset(2 as isize) as ::core::ffi::c_int,
            ) != 0
            && stop_unescaping == (FALSE as BOOL)
        {
            let mut ih: INT = 0;
            let mut buf: [::core::ffi::c_char; 3 as usize] =
                [0 as ::core::ffi::c_char; 3 as usize];
            memcpy(
                buf.as_mut_ptr() as *mut ::core::ffi::c_void,
                src.offset(1 as isize) as *const ::core::ffi::c_void,
                2 as usize,
            );
            buf[2 as usize] = '\0' as ::core::ffi::c_char;
            ih = strtol(
                buf.as_mut_ptr(),
                core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                16,
            ) as INT;
            next = ih as CHAR as ::core::ffi::c_char;
            src = src.offset(2 as isize); /* Advance to end of escape */
        } else {
            next = *src;
        }

        if (dwFlags & URL_UNESCAPE_INPLACE) != 0 || needed < *pcchUnescaped {
            let fresh48 = dst;
            dst = dst.offset(1);
            *fresh48 = next;
        }
        src = src.offset(1);
        needed = needed.wrapping_add(1);
    }

    if (dwFlags & URL_UNESCAPE_INPLACE) != 0 || needed < *pcchUnescaped {
        *dst = '\0' as ::core::ffi::c_char;
        ret = S_OK;
    } else {
        needed = needed.wrapping_add(1); /* add one for the '\0' */
        ret = E_POINTER;
    }

    if (dwFlags & URL_UNESCAPE_INPLACE) == 0 {
        *pcchUnescaped = needed;
    }

    if ret == S_OK {
        WINE_TRACE!("result %s\n", if (dwFlags & 0x100000) != 0 { debugstr_a(pszUrl as *const ::core::ffi::c_char) } else { debugstr_a(pszUnescaped as *const ::core::ffi::c_char) });
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn UrlUnescapeW(
    pszUrl: LPWSTR,
    pszUnescaped: LPWSTR,
    pcchUnescaped: LPDWORD,
    dwFlags: DWORD,
) -> HRESULT {
    let mut dst: *mut WCHAR = core::ptr::null_mut();
    let mut next: WCHAR = 0;
    let mut src: LPCWSTR = core::ptr::null();
    let mut ret: HRESULT = 0;
    let mut needed: DWORD = 0;
    let mut stop_unescaping: BOOL = (FALSE as i32);

    if pszUrl.is_null() {
        return E_INVALIDARG;
    }

    if (dwFlags & URL_UNESCAPE_INPLACE) != 0 {
        dst = pszUrl;
    } else {
        if pszUnescaped.is_null() || pcchUnescaped.is_null() {
            return E_INVALIDARG;
        }
        dst = pszUnescaped;
    }

    src = pszUrl as LPCWSTR;
    needed = 0;
    while *src != 0 {
        if (dwFlags & URL_DONT_UNESCAPE_EXTRA_INFO) != 0
            && (*src == '#' as WCHAR || *src == '?' as WCHAR)
        {
            stop_unescaping = (TRUE as i32);
            next = *src;
        } else if *src == '%' as WCHAR
            && isxdigitW!(*src.offset(1 as isize)) != 0
            && isxdigitW!(*src.offset(2 as isize)) != 0
            && stop_unescaping == (FALSE as BOOL)
        {
            let mut ih: INT = 0;
            let mut buf: [WCHAR; 5] = [0x0030u16, 0x0078, 0x0000, 0x0000, 0x0000];
            memcpy(
                buf.as_mut_ptr().offset(2 as isize) as *mut core::ffi::c_void,
                src.offset(1 as isize) as *const core::ffi::c_void,
                (2 as usize) * core::mem::size_of::<WCHAR>(),
            );
            buf[4 as usize] = 0 as WCHAR;
            StrToIntExW(
                buf.as_mut_ptr() as LPCWSTR,
                STIF_SUPPORT_HEX as DWORD,
                core::ptr::addr_of_mut!(ih),
            );
            next = ih as WCHAR;
            src = src.offset(2 as isize); /* Advance to end of escape */
        } else {
            next = *src;
        }

        if (dwFlags & URL_UNESCAPE_INPLACE) != 0 || needed < *pcchUnescaped {
            let fresh = dst;
            dst = dst.offset(1);
            *fresh = next;
        }
        src = src.offset(1);
        needed = needed.wrapping_add(1);
    }

    if (dwFlags & URL_UNESCAPE_INPLACE) != 0 || needed < *pcchUnescaped {
        *dst = '\0' as WCHAR;
        ret = S_OK;
    } else {
        needed = needed.wrapping_add(1); /* add one for the '\0' */
        ret = E_POINTER;
    }
    if (dwFlags & URL_UNESCAPE_INPLACE) == 0 {
        *pcchUnescaped = needed;
    }

    if ret == S_OK {
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn UrlGetLocationA(pszUrl: LPCSTR) -> LPCSTR {
    let mut base: PARSEDURLA = core::mem::zeroed();
    let mut res1: DWORD = 0;

    base.cbSize = core::mem::size_of::<PARSEDURLA>() as DWORD;
    res1 = ParseURLA(pszUrl, core::ptr::addr_of_mut!(base)) as DWORD;
    if res1 != 0 {
        return core::ptr::null_mut();
    }

    /* if scheme is file: then never return pointer */
    if strncmp(
        base.pszProtocol as *const i8,
        b"file\0".as_ptr() as *const i8,
        min!(4, base.cchProtocol) as usize,
    ) == 0
    {
        return core::ptr::null_mut();
    }

    /* Look for '#' and return its addr */
    return strchr(base.pszSuffix as *const i8, '#' as i32) as LPCSTR;
}

#[no_mangle]
pub unsafe extern "system" fn UrlGetLocationW(pszUrl: LPCWSTR) -> LPCWSTR {
    let mut base: PARSEDURLW = core::mem::zeroed();
    let mut res1: DWORD;

    base.cbSize = core::mem::size_of::<PARSEDURLW>() as DWORD;
    res1 = (ParseURLW(pszUrl, core::ptr::addr_of_mut!(base))) as u32;
    if res1 != 0 {
        return core::ptr::null();
    } /* invalid scheme */

    /* if scheme is file: then never return pointer */
    if strncmpW!(
        base.pszProtocol,
        core::ptr::addr_of!(fileW) as LPCWSTR,
        min!(4, base.cchProtocol),
    ) == 0
    {
        return core::ptr::null();
    }

    /* Look for '#' and return its addr */
    return strchrW!(base.pszSuffix, '#' as WCHAR);
}

#[no_mangle]
pub unsafe extern "system" fn UrlCompareA(
    pszUrl1: LPCSTR,
    pszUrl2: LPCSTR,
    fIgnoreSlash: BOOL,
) -> INT {
    let mut ret: INT;
    let mut len: INT;
    let mut len1: INT;
    let mut len2: INT;

    if fIgnoreSlash == 0 {
        return strcmp(pszUrl1 as *const i8, pszUrl2 as *const i8) as INT;
    }
    len1 = strlen(pszUrl1 as *const i8) as INT;
    if *pszUrl1.offset((len1 - 1) as isize) as INT == '/' as INT {
        len1 -= 1;
    }
    len2 = strlen(pszUrl2 as *const i8) as INT;
    if *pszUrl2.offset((len2 - 1) as isize) as INT == '/' as INT {
        len2 -= 1;
    }
    if len1 == len2 {
        return strncmp(
            pszUrl1 as *const i8,
            pszUrl2 as *const i8,
            len1 as size_t,
        ) as INT;
    }
    len = min!(len1, len2);
    ret = strncmp(
        pszUrl1 as *const i8,
        pszUrl2 as *const i8,
        len as size_t,
    ) as INT;
    if ret != 0 {
        return ret;
    }
    if len1 > len2 {
        return 1;
    }
    return -1;
}

#[no_mangle]
pub unsafe extern "system" fn UrlCompareW(
    pszUrl1: LPCWSTR,
    pszUrl2: LPCWSTR,
    fIgnoreSlash: BOOL,
) -> INT {
    let mut ret: INT;
    let mut len: size_t;
    let mut len1: size_t;
    let mut len2: size_t;

    if fIgnoreSlash == 0 {
        return strcmpW!(pszUrl1, pszUrl2);
    }
    len1 = strlenW(pszUrl1);
    if *pszUrl1.offset(len1.wrapping_sub(1 as size_t) as isize) == '/' as u16 {
        len1 = len1.wrapping_sub(1);
    }
    len2 = strlenW(pszUrl2);
    if *pszUrl2.offset(len2.wrapping_sub(1 as size_t) as isize) == '/' as u16 {
        len2 = len2.wrapping_sub(1);
    }
    if len1 == len2 {
        return strncmpW!(pszUrl1, pszUrl2, len1);
    }
    len = min!(len1, len2);
    ret = strncmpW!(pszUrl1, pszUrl2, len);
    if ret != 0 {
        return ret;
    }
    if len1 > len2 {
        return 1 as INT;
    }
    return -(1 as INT);
}

#[no_mangle]
pub unsafe extern "system" fn HashData(
    lpSrc: *const u8,
    nSrcLen: DWORD,
    lpDest: *mut u8,
    nDestLen: DWORD,
) -> HRESULT {
    let mut srcCount: INT = nSrcLen.wrapping_sub(1 as DWORD) as INT;
    let mut destCount: INT = nDestLen.wrapping_sub(1 as DWORD) as INT;

    if lpSrc.is_null() || lpDest.is_null() {
        return E_INVALIDARG;
    }

    while destCount >= 0 {
        *lpDest.offset(destCount as isize) =
            (destCount & 0xff) as u8;
        destCount -= 1;
    }

    while srcCount >= 0 {
        destCount = nDestLen.wrapping_sub(1 as DWORD) as INT;
        while destCount >= 0 {
            *lpDest.offset(destCount as isize) = HashDataLookup[(*lpSrc.offset(srcCount as isize)
                as INT
                ^ *lpDest.offset(destCount as isize) as INT) as usize];
            destCount -= 1;
        }
        srcCount -= 1;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn UrlHashA(pszUrl: LPCSTR, lpDest: *mut u8, nDestLen: DWORD) -> HRESULT {
    if IsBadStringPtrA(pszUrl, (-(1 as i32)) as UINT_PTR) != 0
        || IsBadWritePtr(lpDest as PVOID, nDestLen as UINT_PTR) != 0
    {
        return E_INVALIDARG;
    }

    HashData(
        pszUrl as *const u8,
        strlen(pszUrl as *const i8) as i32 as DWORD,
        lpDest,
        nDestLen,
    );
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn UrlHashW(pszUrl: LPCWSTR, lpDest: *mut u8, nDestLen: DWORD) -> HRESULT {
    let mut szUrl: [core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if IsBadStringPtrW(pszUrl, (-(1 as isize)) as UINT_PTR) != 0
        || IsBadWritePtr(lpDest as PVOID, (nDestLen) as UINT_PTR) != 0
    {
        return E_INVALIDARG;
    }

    /* Win32 hashes the data as an ASCII string, presumably so that both A+W
     * return the same digests for the same URL.
     */
    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        pszUrl,
        -(1 as i32),
        core::ptr::addr_of_mut!(szUrl) as LPSTR,
        MAX_PATH as i32,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    );
    HashData(
        core::ptr::addr_of_mut!(szUrl) as *mut core::ffi::c_char as *const u8,
        (strlen(core::ptr::addr_of_mut!(szUrl) as *mut core::ffi::c_char) as i32) as DWORD,
        lpDest,
        nDestLen,
    );
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn UrlApplySchemeA(
    pszIn: LPCSTR,
    pszOut: LPSTR,
    pcchOut: LPDWORD,
    dwFlags: DWORD,
) -> HRESULT {
    let in_: LPWSTR;
    let out: LPWSTR;
    let mut ret: HRESULT;
    let mut len: DWORD;

    if pszIn.is_null() || pszOut.is_null() || pcchOut.is_null() {
        return E_INVALIDARG;
    }

    in_ = HeapAlloc(
        GetProcessHeap(),
        0,
        ((2 as SIZE_T) * (INTERNET_MAX_URL_LENGTH as SIZE_T))
            * (core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    out = in_.offset(INTERNET_MAX_URL_LENGTH as isize);

    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        pszIn,
        -(1 as i32),
        in_,
        INTERNET_MAX_URL_LENGTH as i32,
    );
    len = INTERNET_MAX_URL_LENGTH as DWORD;

    ret = UrlApplySchemeW(in_ as LPCWSTR, out, core::ptr::addr_of_mut!(len), dwFlags);
    if ret != S_OK {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            in_ as LPVOID,
        );
        return ret;
    }

    len = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        out as LPCWSTR,
        -(1 as i32),
        core::ptr::null_mut(),
        0 as i32,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) as DWORD;

    'cleanup: {
        if len > *pcchOut {
            ret = E_POINTER;
            break 'cleanup;
        }

        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            out as LPCWSTR,
            -(1 as i32),
            pszOut,
            *pcchOut as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        len = len.wrapping_sub(1);
    }

    *pcchOut = len;
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        in_ as LPVOID,
    );
    return ret;
}

unsafe extern "C" fn URL_GuessScheme(
    mut pszIn: LPCWSTR,
    mut pszOut: LPWSTR,
    mut pcchOut: LPDWORD,
) -> HRESULT {
    let mut newkey: HKEY = core::ptr::null_mut();
    let mut j: BOOL = 0;
    let mut index: INT = 0;
    let mut value_len: DWORD = 0;
    let mut data_len: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut i: DWORD = 0;
    let mut reg_path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut value: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut data: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut Wxx: WCHAR = 0;
    let mut Wyy: WCHAR = 0;

    MultiByteToWideChar(
        CP_ACP,
        0,
        b"Software\\Microsoft\\Windows\\CurrentVersion\\URL\\Prefixes\0" as *const u8 as LPCSTR,
        -(1),
        &raw mut reg_path as LPWSTR,
        (MAX_PATH as i32),
    );
    RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        &raw mut reg_path as LPCWSTR,
        0,
        1,
        &raw mut newkey,
    );
    index = 0;
    loop {
        data_len = MAX_PATH;
        value_len = data_len;
        if !(RegEnumValueW(
            newkey,
            index as DWORD,
            &raw mut value as LPWSTR,
            &raw mut value_len,
            0 as LPDWORD,
            &raw mut dwType,
            &raw mut data as *mut WCHAR as LPVOID as LPBYTE,
            &raw mut data_len,
        ) == 0)
        {
            break;
        }

        j = FALSE as BOOL;
        i = 0 as DWORD;
        while i < value_len {
            Wxx = *pszIn.offset(i as isize);
            Wyy = value[i as usize];
            /* remember that TRUE is not-equal */
            j = ChrCmpIW(Wxx, Wyy);
            if j != 0 {
                break;
            }
            i += 1;
        }
        if i == value_len && j == 0 {
            if strlenW(&raw mut data as *mut WCHAR) + strlenW(pszIn) + 1
                > (*pcchOut) as _
            {
                *pcchOut = (strlenW(&raw mut data as *mut WCHAR)
                    + strlenW(pszIn)
                    + 1) as _;
                RegCloseKey(newkey);
                return E_POINTER;
            }
            strcpyW!(pszOut, &raw mut data as *mut WCHAR);
            strcatW!(pszOut, pszIn);
            *pcchOut = (strlenW(pszOut)) as _;
            RegCloseKey(newkey);
            return S_OK;
        }
        index += 1;
    }
    RegCloseKey(newkey);
    return E_FAIL;
}

unsafe extern "C" fn URL_CreateFromPath(pszPath: LPCWSTR, pszUrl: LPWSTR, pcchUrl: LPDWORD) -> HRESULT {
    let mut needed: DWORD;
    let mut ret: HRESULT = S_OK;
    let mut pszNewUrl: *mut WCHAR;
    let mut file_colonW: [WCHAR; 6] = [0x0066u16, 0x0069, 0x006C, 0x0065, 0x003A, 0x0000];
    let mut three_slashesW: [WCHAR; 4] = [0x002Fu16, 0x002F, 0x002F, 0x0000];
    let mut parsed_url: PARSEDURLW = core::mem::zeroed::<PARSEDURLW>();

    parsed_url.cbSize = core::mem::size_of::<PARSEDURLW>() as DWORD;
    if ParseURLW(pszPath, core::ptr::addr_of_mut!(parsed_url)) == S_OK {
        if parsed_url.nScheme != (URL_SCHEME_INVALID) as u32 && parsed_url.cchProtocol > 1 {
            needed = strlenW(pszPath) as DWORD;
            if needed >= *pcchUrl {
                *pcchUrl = needed + 1;
                return E_POINTER;
            } else {
                *pcchUrl = needed;
                return S_FALSE;
            }
        }
    }

    pszNewUrl = HeapAlloc(
        GetProcessHeap(),
        0,
        (strlenW(pszPath) as SIZE_T + 9 as SIZE_T)
            * core::mem::size_of::<WCHAR>() as SIZE_T,
    ) as *mut WCHAR;
    strcpyW!(pszNewUrl, core::ptr::addr_of_mut!(file_colonW) as *mut WCHAR);
    if isalphaW!(*pszPath.add(0 as usize)) != 0
        && *pszPath.add(1 as usize) == ':' as WCHAR
    {
        strcatW!(pszNewUrl, core::ptr::addr_of_mut!(three_slashesW) as *mut WCHAR);
    }
    strcatW!(pszNewUrl, pszPath);
    ret = UrlEscapeW(pszNewUrl, pszUrl, pcchUrl, URL_ESCAPE_PERCENT);
    HeapFree(GetProcessHeap(), 0, pszNewUrl as LPVOID);
    return ret;
}

unsafe extern "C" fn URL_ApplyDefault(pszIn: LPCWSTR, pszOut: LPWSTR, pcchOut: LPDWORD) -> HRESULT {
    let mut newkey: HKEY = core::ptr::null_mut();
    let mut data_len: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut data: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    static prefix_keyW: [u16; 60] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x005C, 0x0055, 0x0052, 0x004C, 0x005C, 0x0044, 0x0065, 0x0066, 0x0061,
    0x0075, 0x006C, 0x0074, 0x0050, 0x0072, 0x0065, 0x0066, 0x0069, 0x0078, 0x0000,
];

    /* get and prepend default */
    RegOpenKeyExW(HKEY_LOCAL_MACHINE, core::ptr::addr_of!(prefix_keyW) as LPCWSTR, 0, 1, &raw mut newkey);
    data_len = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
    RegQueryValueExW(newkey, core::ptr::null::<WCHAR>(), core::ptr::null_mut(), &raw mut dwType, &raw mut data as *mut WCHAR as LPBYTE, &raw mut data_len);
    RegCloseKey(newkey);
    if (strlenW(&raw mut data as *mut WCHAR) as DWORD)
        .wrapping_add(strlenW(pszIn) as DWORD)
        .wrapping_add(1) > *pcchOut
    {
        *pcchOut = (strlenW(&raw mut data as *mut WCHAR) as DWORD)
            .wrapping_add(strlenW(pszIn) as DWORD)
            .wrapping_add(1);
        return E_POINTER;
    }
    strcpyW!(pszOut, &raw mut data as *mut WCHAR);
    strcatW!(pszOut, pszIn);
    *pcchOut = strlenW(pszOut) as DWORD;

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn UrlApplySchemeW(
    pszIn: LPCWSTR,
    pszOut: LPWSTR,
    pcchOut: LPDWORD,
    dwFlags: DWORD,
) -> HRESULT {
    let mut in_scheme: PARSEDURLW = core::mem::zeroed();
    let mut res1: DWORD = 0;
    let mut ret: HRESULT = 0;

    TRACE!(
        "(%s, %p, %p:out size %d, 0x%08x)\n",
        debugstr_w(pszIn),
        pszOut,
        pcchOut,
        if !pcchOut.is_null() { *pcchOut } else { 0 as DWORD },
        dwFlags
    );

    if pszIn.is_null() || pszOut.is_null() || pcchOut.is_null() {
        return E_INVALIDARG;
    }

    if (dwFlags & URL_APPLY_GUESSFILE as DWORD) != 0 {
        if *pcchOut > 1 as DWORD
            && ':' as i32 == *pszIn.offset(1 as isize) as i32
        {
            res1 = *pcchOut;
            ret = URL_CreateFromPath(pszIn, pszOut, &raw mut res1);
            if ret == S_OK || ret == E_POINTER {
                *pcchOut = res1;
                return ret;
            } else if ret == S_FALSE {
                return ret;
            }
        }
    }

    in_scheme.cbSize = core::mem::size_of::<PARSEDURLW>() as DWORD;
    /* See if the base has a scheme */
    res1 = ParseURLW(pszIn, &raw mut in_scheme) as DWORD;
    if res1 != 0 {
        /* no scheme in input, need to see if we need to guess */
        if (dwFlags & URL_APPLY_GUESSSCHEME as DWORD) != 0 {
            ret = URL_GuessScheme(pszIn, pszOut, pcchOut);
            if ret != E_FAIL {
                return ret;
            }
        }
    }

    /* If we are here, then either invalid scheme,
     * or no scheme and can't/failed guess.
     */
    if ((res1 == 0 as DWORD
        && (dwFlags & URL_APPLY_FORCEAPPLY as DWORD) != 0)
        || (res1 != 0 as DWORD))
        && (dwFlags & URL_APPLY_DEFAULT as DWORD) != 0
    {
        /* find and apply default scheme */
        return URL_ApplyDefault(pszIn, pszOut, pcchOut);
    }

    return S_FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn UrlIsA(pszUrl: LPCSTR, Urlis: URLIS) -> BOOL {
    let mut base: PARSEDURLA = core::mem::zeroed();
    let mut res1: DWORD = 0;
    let mut last: LPCSTR = core::ptr::null();

    if pszUrl.is_null() {
        return (FALSE) as BOOL;
    }

    match (Urlis as u32) as i32 {
        __c if __c == URLIS_OPAQUE as i32 => {
            base.cbSize = core::mem::size_of::<PARSEDURLA>() as DWORD;
            res1 = ParseURLA(pszUrl, &raw mut base) as DWORD;
            if res1 != 0 {
                return (FALSE) as BOOL;
            }
            match (base.nScheme as u32) as i32 {
                URL_SCHEME_MAILTO | URL_SCHEME_SHELL | URL_SCHEME_JAVASCRIPT | URL_SCHEME_VBSCRIPT | URL_SCHEME_ABOUT => {
                    return (TRUE) as BOOL;
                }
                _ => {}
            }
            return (FALSE) as BOOL;
        }
        __c if __c == URLIS_FILEURL as i32 => {
            return (CompareStringA(
                LOCALE_INVARIANT,
                NORM_IGNORECASE,
                pszUrl,
                5,
                b"file:\0".as_ptr() as LPCSTR,
                5,
            ) == (CSTR_EQUAL as i32)) as BOOL;
        }
        __c if __c == URLIS_DIRECTORY as i32 => {
            last = pszUrl
                .offset(strlen(pszUrl as *const i8) as isize)
                .offset(-(1 as isize));
            return (last >= pszUrl && ((*last) as i32 == (b'/') as i32 || (*last) as i32 == (b'\\') as i32)) as BOOL;
        }
        __c if __c == URLIS_URL as i32 => {
            return PathIsURLA(pszUrl);
        }
        _ => {
            FIXME!("(%s %d): stub\n", debugstr_a(pszUrl), Urlis);
        }
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UrlIsW(pszUrl: LPCWSTR, Urlis: URLIS) -> BOOL {
    static mut file_colon: [WCHAR; 6] = [0x0066u16, 0x0069, 0x006C, 0x0065, 0x003A, 0x0000];
    let mut base: PARSEDURLW = core::mem::zeroed();
    let mut res1: DWORD = 0;
    let mut last: LPCWSTR = core::ptr::null_mut();
    TRACE!("(%s %d)\n", debugstr_w(pszUrl), Urlis as ::core::ffi::c_uint);
    if pszUrl.is_null() {
        return (FALSE) as BOOL;
    }
    match ((Urlis as ::core::ffi::c_uint) as i32) as u32 {
        __c if __c == URLIS_OPAQUE as u32 => {
            base.cbSize = core::mem::size_of::<PARSEDURLW>() as DWORD;
            res1 = ParseURLW(pszUrl, &raw mut base) as DWORD;
            if res1 != 0 {
                return (FALSE) as BOOL;
            }
            match (base.nScheme) as i32 {
                URL_SCHEME_MAILTO
                | URL_SCHEME_SHELL
                | URL_SCHEME_JAVASCRIPT
                | URL_SCHEME_VBSCRIPT
                | URL_SCHEME_ABOUT => return (TRUE as i32),
                _ => {}
            }
            return (FALSE) as BOOL;
        }
        __c if __c == URLIS_FILEURL as u32 => {
            return (CompareStringW(
                LOCALE_INVARIANT,
                NORM_IGNORECASE as DWORD,
                pszUrl,
                5 as ::core::ffi::c_int,
                &raw const file_colon as LPCWSTR,
                5 as ::core::ffi::c_int,
            ) == (CSTR_EQUAL as i32)) as BOOL;
        }
        __c if __c == URLIS_DIRECTORY as u32 => {
            last = pszUrl
                .add(strlenW(pszUrl) as usize)
                .offset(-(1 as isize));
            return (last >= pszUrl && (*last == '/' as WCHAR || *last == '\\' as WCHAR)) as BOOL;
        }
        __c if __c == URLIS_URL as u32 => {
            return PathIsURLW(pszUrl);
        }
        _ => {
            FIXME!("(%s %d): stub\n", debugstr_w(pszUrl), Urlis as ::core::ffi::c_uint);
        }
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UrlIsNoHistoryA(pszUrl: LPCSTR) -> i32 {
    return UrlIsA(pszUrl, (URLIS_NOHISTORY) as URLIS);
}

#[no_mangle]
pub unsafe extern "system" fn UrlIsNoHistoryW(pszUrl: LPCWSTR) -> BOOL {
    return UrlIsW(pszUrl, (URLIS_NOHISTORY) as URLIS);
}

#[no_mangle]
pub unsafe extern "system" fn UrlIsOpaqueA(pszUrl: LPCSTR) -> BOOL {
    return UrlIsA(pszUrl, (URLIS_OPAQUE) as URLIS);
}
#[no_mangle]
pub unsafe extern "system" fn UrlIsOpaqueW(pszUrl: LPCWSTR) -> BOOL {
    return UrlIsW(pszUrl, (URLIS_OPAQUE) as URLIS);
}

unsafe extern "C" fn URL_ScanID(start: LPCWSTR, size: LPDWORD, type_: WINE_URL_SCAN_TYPE) -> LPCWSTR {
    static mut alwayszero: DWORD = 0;
    let mut start = start;
    let mut cont: BOOL = (TRUE as i32);

    *size = 0;

    match type_ as i32 {
        SCHEME => {
            while cont != 0 {
                if (islowerW!(*start) != 0 && isalphaW!(*start) != 0)
                    || isdigitW!(*start) != 0
                    || *start == '+' as u16
                    || *start == '-' as u16
                    || *start == '.' as u16
                {
                    start = start.offset(1);
                    *size = (*size).wrapping_add(1);
                } else {
                    cont = (FALSE as i32);
                }
            }

            if *start != ':' as u16 {
                *size = 0;
            }

            // break
        }
        USERPASS => {
            while cont != 0 {
                if isalphaW!(*start) != 0
                    || isdigitW!(*start) != 0
                    || *start == ';' as u16
                    || *start == '?' as u16
                    || *start == '&' as u16
                    || *start == '=' as u16
                    || *start == '!' as u16
                    || *start == '*' as u16
                    || *start == '\'' as u16
                    || *start == '(' as u16
                    || *start == ')' as u16
                    || *start == ',' as u16
                    || *start == '$' as u16
                    || *start == '_' as u16
                    || *start == '+' as u16
                    || *start == '-' as u16
                    || *start == '.' as u16
                    || *start == ' ' as u16
                {
                    start = start.offset(1);
                    *size = (*size).wrapping_add(1);
                } else if *start == '%' as u16 {
                    if isxdigitW!(*start.offset(1)) != 0
                        && isxdigitW!(*start.offset(2)) != 0
                    {
                        start = start.offset(3);
                        *size += 3;
                    } else {
                        cont = (FALSE as i32);
                    }
                } else {
                    cont = (FALSE as i32);
                }
            }
            // break
        }
        PORT => {
            while cont != 0 {
                if isdigitW!(*start) != 0 {
                    start = start.offset(1);
                    *size = (*size).wrapping_add(1);
                } else {
                    cont = (FALSE as i32);
                }
            }
            // break
        }
        HOST => {
            while cont != 0 {
                if isalnumW!(*start) != 0
                    || *start == '-' as u16
                    || *start == '.' as u16
                    || *start == ' ' as u16
                    || *start == '*' as u16
                {
                    start = start.offset(1);
                    *size = (*size).wrapping_add(1);
                } else {
                    cont = (FALSE as i32);
                }
            }
            // break
        }
        _ => {
            FIXME!("unknown type %d\n", type_ as i32);
            return core::ptr::addr_of_mut!(alwayszero) as LPWSTR as LPCWSTR;
        }
    }

    return start;
}

unsafe extern "C" fn URL_ParseUrl(pszUrl: LPCWSTR, pl: *mut WINE_PARSE_URL) -> LONG {
    let mut work: LPCWSTR = core::ptr::null::<WCHAR>();
    memset(
        pl as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<WINE_PARSE_URL>() as size_t,
    );
    (*pl).pScheme = pszUrl;
    work = URL_ScanID(
        (*pl).pScheme,
        core::ptr::addr_of_mut!((*pl).szScheme),
        SCHEME,
    );
    'error: {
        'success: {
            if *work == 0 || *work != ':' as WCHAR {
                break 'error;
            }
            work = work.offset(1);
            if *work != '/' as WCHAR
                || *work.offset(1 as isize) != '/' as WCHAR
            {
                break 'success;
            }
            (*pl).pUserName = work.offset(2 as isize);
            work = URL_ScanID(
                (*pl).pUserName,
                core::ptr::addr_of_mut!((*pl).szUserName),
                USERPASS,
            );
            if *work == ':' as WCHAR {
                work = work.offset(1);
                (*pl).pPassword = work;
                work = URL_ScanID(
                    (*pl).pPassword,
                    core::ptr::addr_of_mut!((*pl).szPassword),
                    USERPASS,
                );
                if *work != '@' as WCHAR {
                    (*pl).szPassword = 0;
                    (*pl).szUserName = (*pl).szPassword;
                    work = (*pl).pUserName.offset(-(1 as isize));
                    (*pl).pPassword = core::ptr::null();
                    (*pl).pUserName = (*pl).pPassword;
                }
            } else if *work == '@' as WCHAR {
                (*pl).szPassword = 0;
                (*pl).pPassword = core::ptr::null();
            } else if *work == 0 || *work == '/' as WCHAR || *work == '.' as WCHAR {
                (*pl).szPassword = 0;
                (*pl).szUserName = (*pl).szPassword;
                work = (*pl).pUserName.offset(-(1 as isize));
                (*pl).pPassword = core::ptr::null();
                (*pl).pUserName = (*pl).pPassword;
            } else {
                break 'error;
            }

            work = work.offset(1);
            (*pl).pHostName = work;
            work = URL_ScanID(
                (*pl).pHostName,
                core::ptr::addr_of_mut!((*pl).szHostName),
                HOST,
            );
            if *work == ':' as WCHAR {
                work = work.offset(1);
                (*pl).pPort = work;
                work = URL_ScanID(
                    (*pl).pPort,
                    core::ptr::addr_of_mut!((*pl).szPort),
                    PORT,
                );
            }
            if *work == '/' as WCHAR {
                (*pl).pQuery = strchrW!(work, '?' as WCHAR);
                if !(*pl).pQuery.is_null() {
                    (*pl).szQuery = strlenW((*pl).pQuery) as DWORD;
                }
            }
        }
        // SuccessExit:
        WINE_TRACE!("parse successful: scheme=%p(%d), user=%p(%d), pass=%p(%d), host=%p(%d), port=%p(%d), query=%p(%d)\n", (*pl).pScheme, (*pl).szScheme, (*pl).pUserName, (*pl).szUserName, (*pl).pPassword, (*pl).szPassword, (*pl).pHostName, (*pl).szHostName, (*pl).pPort, (*pl).szPort, (*pl).pQuery, (*pl).szQuery);
        return S_OK;
    }
    // ErrorExit:
    WINE_FIXME!("failed to parse %s\n", debugstr_w(pszUrl as *const WCHAR));
    return E_INVALIDARG;
}

#[no_mangle]
pub unsafe extern "system" fn UrlGetPartA(
    pszIn: LPCSTR,
    pszOut: LPSTR,
    pcchOut: LPDWORD,
    dwPart: DWORD,
    dwFlags: DWORD,
) -> HRESULT {
    let mut in_0: LPWSTR = core::ptr::null_mut();
    let mut out: LPWSTR = core::ptr::null_mut();
    let mut ret: DWORD = 0;
    let mut len: DWORD = 0;
    let mut len2: DWORD = 0;

    if pszIn.is_null() || pszOut.is_null() || pcchOut.is_null() || *pcchOut <= 0 as DWORD {
        return E_INVALIDARG as HRESULT;
    }

    in_0 = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (2 as SIZE_T)
            .wrapping_mul(INTERNET_MAX_URL_LENGTH as SIZE_T)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    out = in_0.offset(INTERNET_MAX_URL_LENGTH as isize);

    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        pszIn,
        -(1 as ::core::ffi::c_int),
        in_0,
        INTERNET_MAX_URL_LENGTH as ::core::ffi::c_int,
    );

    len = INTERNET_MAX_URL_LENGTH as DWORD;
    ret = UrlGetPartW(in_0 as LPCWSTR, out, core::ptr::addr_of_mut!(len), dwPart, dwFlags) as DWORD;

    // C: FAILED(ret), i.e. ((HRESULT)ret) < 0. `ret` is a DWORD in the C too; comparing it
    // unsigned is never true, and a failed UrlGetPartW fell through to write a size.
    if (ret as HRESULT) < 0 {
        HeapFree(GetProcessHeap(), 0 as DWORD, in_0 as PVOID);
        return ret as HRESULT;
    }

    len2 = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        out as LPCWSTR,
        len as ::core::ffi::c_int,
        core::ptr::null_mut::<CHAR>(),
        0 as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) as DWORD;
    if len2 > *pcchOut {
        *pcchOut = len2.wrapping_add(1 as DWORD);
        HeapFree(GetProcessHeap(), 0 as DWORD, in_0 as PVOID);
        return E_POINTER as HRESULT;
    }
    len2 = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        out as LPCWSTR,
        len.wrapping_add(1 as DWORD) as ::core::ffi::c_int,
        pszOut,
        *pcchOut as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) as DWORD;
    *pcchOut = len2.wrapping_sub(1 as DWORD);
    HeapFree(GetProcessHeap(), 0 as DWORD, in_0 as PVOID);
    return ret as HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn UrlGetPartW(
    pszIn: LPCWSTR,
    pszOut: LPWSTR,
    pcchOut: LPDWORD,
    dwPart: DWORD,
    dwFlags: DWORD,
) -> HRESULT {
    let mut pl: WINE_PARSE_URL = core::mem::zeroed();
    let mut ret: HRESULT;
    let mut scheme: DWORD;
    let mut size: DWORD;
    let mut schsize: DWORD;
    let mut addr: LPCWSTR;
    let mut schaddr: LPCWSTR;

    if pszIn.is_null() || pszOut.is_null() || pcchOut.is_null() || *pcchOut <= 0 {
        return E_INVALIDARG;
    }

    *pszOut = '\0' as WCHAR;

    addr = strchrW!(pszIn, ':' as WCHAR) as LPCWSTR;
    if addr.is_null() {
        scheme = (URL_SCHEME_UNKNOWN as u32);
    } else {
        scheme = get_scheme_code(pszIn, addr.offset_from(pszIn) as DWORD);
    }

    ret = URL_ParseUrl(pszIn, core::ptr::addr_of_mut!(pl));

    match ((dwPart) as i32) as u32 {
        URL_PART_SCHEME => {
            if pl.szScheme == 0 {
                *pcchOut = 0;
                return S_FALSE;
            }
            addr = pl.pScheme;
            size = pl.szScheme;
        }
        URL_PART_HOSTNAME => {
            match ((scheme) as i32) as u32 {
                __c if __c == URL_SCHEME_FTP as u32 || __c == URL_SCHEME_HTTP as u32 || __c == URL_SCHEME_GOPHER as u32 || __c == URL_SCHEME_TELNET as u32 || __c == URL_SCHEME_FILE as u32 || __c == URL_SCHEME_HTTPS as u32 => {}
                _ => {
                    *pcchOut = 0;
                    return E_FAIL;
                }
            }

            if scheme == URL_SCHEME_FILE as u32
                && (pl.szHostName == 0
                    || (pl.szHostName == 1
                        && *pl.pHostName.offset(1 as isize) == ':' as WCHAR))
            {
                *pcchOut = 0;
                return S_FALSE;
            }

            if pl.szHostName == 0 {
                *pcchOut = 0;
                return S_FALSE;
            }
            addr = pl.pHostName;
            size = pl.szHostName;
        }
        URL_PART_USERNAME => {
            if pl.szUserName == 0 {
                *pcchOut = 0;
                return S_FALSE;
            }
            addr = pl.pUserName;
            size = pl.szUserName;
        }
        URL_PART_PASSWORD => {
            if pl.szPassword == 0 {
                *pcchOut = 0;
                return S_FALSE;
            }
            addr = pl.pPassword;
            size = pl.szPassword;
        }
        URL_PART_PORT => {
            if pl.szPort == 0 {
                *pcchOut = 0;
                return S_FALSE;
            }
            addr = pl.pPort;
            size = pl.szPort;
        }
        URL_PART_QUERY => {
            if pl.szQuery == 0 {
                *pcchOut = 0;
                return S_FALSE;
            }
            addr = pl.pQuery;
            size = pl.szQuery;
        }
        _ => {
            *pcchOut = 0;
            return E_INVALIDARG;
        }
    }

    if dwFlags == URL_PARTFLAG_KEEPSCHEME {
        if pl.pScheme.is_null() || pl.szScheme == 0 {
            *pcchOut = 0;
            return E_FAIL;
        }
        schaddr = pl.pScheme;
        schsize = pl.szScheme;
        if *pcchOut < schsize.wrapping_add(size).wrapping_add(2) {
            *pcchOut = schsize.wrapping_add(size).wrapping_add(2);
            return E_POINTER;
        }
        memcpy(
            pszOut as *mut core::ffi::c_void,
            schaddr as *const core::ffi::c_void,
            (schsize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        *pszOut.offset(schsize as isize) = ':' as WCHAR;
        memcpy(
            pszOut
                .offset(schsize as isize)
                .offset(1 as isize) as *mut core::ffi::c_void,
            addr as *const core::ffi::c_void,
            (size as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        *pszOut.offset(
            schsize
                .wrapping_add(1)
                .wrapping_add(size) as isize,
        ) = 0 as WCHAR;
        *pcchOut = schsize.wrapping_add(1).wrapping_add(size);
    } else {
        if *pcchOut < size.wrapping_add(1) {
            *pcchOut = size.wrapping_add(1);
            return E_POINTER;
        }
        memcpy(
            pszOut as *mut core::ffi::c_void,
            addr as *const core::ffi::c_void,
            (size as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        *pszOut.offset(size as isize) = 0 as WCHAR;
        *pcchOut = size;
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsURLA(lpstrPath: LPCSTR) -> BOOL {
    let mut base: PARSEDURLA = core::mem::zeroed();
    let mut hres: HRESULT;

    if lpstrPath.is_null() || *lpstrPath == 0 {
        return (FALSE) as BOOL;
    }

    /* get protocol        */
    base.cbSize = core::mem::size_of::<PARSEDURLA>() as DWORD;
    hres = ParseURLA(lpstrPath, core::ptr::addr_of_mut!(base));
    return (hres == S_OK && (base.nScheme != (URL_SCHEME_INVALID) as u32)) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsURLW(lpstrPath: LPCWSTR) -> BOOL {
    let mut base: PARSEDURLW = PARSEDURLW {
        cbSize: 0,
        pszProtocol: core::ptr::null::<WCHAR>(),
        cchProtocol: 0,
        pszSuffix: core::ptr::null::<WCHAR>(),
        cchSuffix: 0,
        nScheme: 0,
    };
    let mut hres: HRESULT;

    TRACE!("%s\n", debugstr_w(lpstrPath));

    if lpstrPath.is_null() || *lpstrPath == 0 {
        return (FALSE) as BOOL;
    }

    /* get protocol        */
    base.cbSize = core::mem::size_of::<PARSEDURLW>() as DWORD;
    hres = ParseURLW(lpstrPath, &mut base as *mut PARSEDURLW);
    return (hres == S_OK && (base.nScheme != (URL_SCHEME_INVALID) as u32)) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UrlCreateFromPathA(
    pszPath: LPCSTR,
    pszUrl: LPSTR,
    pcchUrl: LPDWORD,
    dwReserved: DWORD,
) -> HRESULT {
    let mut bufW: [WCHAR; INTERNET_MAX_URL_LENGTH] = [0; INTERNET_MAX_URL_LENGTH];
    let mut urlW: *mut WCHAR = core::ptr::addr_of_mut!(bufW) as *mut WCHAR;
    let mut pathW: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: HRESULT;
    let mut lenW: DWORD = (core::mem::size_of::<[WCHAR; INTERNET_MAX_URL_LENGTH]>()
        / core::mem::size_of::<WCHAR>()) as DWORD;
    let mut lenA: DWORD = core::mem::zeroed();

    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(pathW), pszPath) == 0 {
        return E_INVALIDARG;
    }
    ret = UrlCreateFromPathW(
        pathW.Buffer as LPCWSTR,
        urlW as LPWSTR,
        core::ptr::addr_of_mut!(lenW),
        dwReserved,
    );
    if ret == E_POINTER {
        urlW = HeapAlloc(
            GetProcessHeap(),
            0,
            (lenW as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        ret = UrlCreateFromPathW(
            pathW.Buffer as LPCWSTR,
            urlW as LPWSTR,
            core::ptr::addr_of_mut!(lenW),
            dwReserved,
        );
    }
    if ret == S_OK || ret == S_FALSE {
        RtlUnicodeToMultiByteSize(
            core::ptr::addr_of_mut!(lenA),
            urlW as PCWSTR,
            (lenW as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as ULONG,
        );
        if *pcchUrl > lenA {
            RtlUnicodeToMultiByteN(
                pszUrl,
                (*pcchUrl).wrapping_sub(1),
                core::ptr::addr_of_mut!(lenA),
                urlW as LPCWSTR,
                (lenW as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
            );
            *pszUrl.add(lenA as usize) = 0 as CHAR;
            *pcchUrl = lenA;
        } else {
            *pcchUrl = lenA + 1;
            ret = E_POINTER;
        }
    }
    if urlW != core::ptr::addr_of_mut!(bufW) as *mut WCHAR {
        HeapFree(GetProcessHeap(), 0, urlW as PVOID);
    }
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(pathW));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn UrlCreateFromPathW(
    pszPath: LPCWSTR,
    pszUrl: LPWSTR,
    pcchUrl: LPDWORD,
    dwReserved: DWORD,
) -> HRESULT {
    let ret: HRESULT;

    /* Validate arguments */
    if dwReserved != 0 {
        return E_INVALIDARG;
    }
    if pszUrl.is_null() || pcchUrl.is_null() {
        return E_INVALIDARG;
    }

    ret = URL_CreateFromPath(pszPath, pszUrl, pcchUrl);

    if S_FALSE == ret {
        strcpyW!(pszUrl, pszPath);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn MLBuildResURLA(
    mut lpszLibName: LPCSTR,
    mut hMod: HMODULE,
    mut dwFlags: DWORD,
    mut lpszRes: LPCSTR,
    mut lpszDest: LPSTR,
    mut dwDestLen: DWORD,
) -> HRESULT {
    let mut szLibName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szRes: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szDest: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hRet: HRESULT;

    if !lpszLibName.is_null() {
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszLibName,
            (-1) as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(szLibName) as LPWSTR,
            ((core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>())
                / (core::mem::size_of::<WCHAR>())) as ::core::ffi::c_int,
        );
    }

    if !lpszRes.is_null() {
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszRes,
            (-1) as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(szRes) as LPWSTR,
            ((core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>())
                / (core::mem::size_of::<WCHAR>())) as ::core::ffi::c_int,
        );
    }

    if dwDestLen as usize
        > (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>())
            / (core::mem::size_of::<WCHAR>())
    {
        dwDestLen = ((core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>())
            / (core::mem::size_of::<WCHAR>())) as DWORD;
    }

    hRet = MLBuildResURLW(
        (if !lpszLibName.is_null() {
            core::ptr::addr_of_mut!(szLibName) as *mut WCHAR
        } else {
            core::ptr::null_mut::<WCHAR>()
        }) as LPCWSTR,
        hMod,
        dwFlags,
        (if !lpszRes.is_null() {
            core::ptr::addr_of_mut!(szRes) as *mut WCHAR
        } else {
            core::ptr::null_mut::<WCHAR>()
        }) as LPCWSTR,
        if !lpszDest.is_null() {
            core::ptr::addr_of_mut!(szDest) as LPWSTR
        } else {
            core::ptr::null_mut::<WCHAR>()
        },
        dwDestLen,
    );

    if hRet >= 0 as HRESULT && !lpszDest.is_null() {
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            core::ptr::addr_of_mut!(szDest) as *mut WCHAR as LPCWSTR,
            (-1) as ::core::ffi::c_int,
            lpszDest,
            dwDestLen as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
    }

    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn MLBuildResURLW(
    lpszLibName: LPCWSTR,
    mut hMod: HMODULE,
    mut dwFlags: DWORD,
    lpszRes: LPCWSTR,
    lpszDest: LPWSTR,
    mut dwDestLen: DWORD,
) -> HRESULT {
    static szRes: [WCHAR; 7] = [0x0072u16, 0x0065, 0x0073, 0x003A, 0x002F, 0x002F, 0x0000];
    let mut hRet: HRESULT = E_FAIL;

    if lpszLibName.is_null()
        || hMod.is_null()
        || hMod == INVALID_HANDLE_VALUE as HMODULE
        || lpszRes.is_null()
        || lpszDest.is_null()
        || (dwFlags != 0 && dwFlags != 2)
    {
        return E_INVALIDARG;
    }

    if (dwDestLen as usize) >= (szResLen + 1) {
        dwDestLen = dwDestLen.wrapping_sub((szResLen + 1) as DWORD);
        memcpy(
            lpszDest as *mut core::ffi::c_void,
            szRes.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of::<[WCHAR; 7]>() as usize,
        );

        hMod = MLLoadLibraryW(lpszLibName, hMod, dwFlags);

        if !hMod.is_null() {
            let mut szBuff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut len: DWORD;

            len = GetModuleFileNameW(
                hMod as HINSTANCE,
                szBuff.as_mut_ptr(),
                (szBuff.len() * core::mem::size_of::<WCHAR>() / core::mem::size_of::<WCHAR>())
                    as DWORD,
            );
            if len != 0 && (len as usize) < (szBuff.len() * core::mem::size_of::<WCHAR>() / core::mem::size_of::<WCHAR>()) {
                let dwPathLen: DWORD =
                    (strlenW(szBuff.as_ptr()) as DWORD).wrapping_add(1);

                if dwDestLen >= dwPathLen {
                    let mut dwResLen: DWORD;

                    dwDestLen = dwDestLen.wrapping_sub(dwPathLen);
                    memcpy(
                        lpszDest.offset(szResLen as isize) as *mut core::ffi::c_void,
                        szBuff.as_ptr() as *const core::ffi::c_void,
                        (dwPathLen as usize) * core::mem::size_of::<WCHAR>(),
                    );

                    dwResLen = (strlenW(lpszRes) as DWORD).wrapping_add(1);
                    if dwDestLen >= dwResLen.wrapping_add(1) {
                        *lpszDest.offset(
                            (szResLen + dwPathLen as usize - 1) as isize,
                        ) = '/' as u16;
                        memcpy(
                            lpszDest.offset(szResLen as isize).offset(dwPathLen as isize)
                                as *mut core::ffi::c_void,
                            lpszRes as *const core::ffi::c_void,
                            (dwResLen as usize) * core::mem::size_of::<WCHAR>(),
                        );
                        hRet = S_OK;
                    }
                }
            }
            MLFreeLibrary(hMod);
        }
    }
    return hRet;
}
#[no_mangle]
pub unsafe extern "system" fn UrlFixupW(
    mut url: LPCWSTR,
    mut translatedUrl: LPWSTR,
    mut maxChars: DWORD,
) -> HRESULT {
    let mut srcLen: DWORD = 0;
    WINE_FIXME!("(%s,%p,%d) STUB\n", debugstr_w(url as *const WCHAR), translatedUrl, maxChars);
    if url.is_null() {
        return E_FAIL;
    }
    srcLen = (lstrlenW(url) + 1) as DWORD;
    lstrcpynW(
        translatedUrl,
        url,
        (if maxChars < srcLen { maxChars } else { srcLen }) as ::core::ffi::c_int,
    );
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IsInternetESCEnabled() -> BOOL {
    // FIXME(": stub\n");
    (FALSE as i32)
}
