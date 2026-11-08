use crate::shlwapi::path::PathCombineW;
use crate::shlwapi::path::PathQuoteSpacesW;
use crate::shlwapi::reg::SHDeleteKeyW;
use crate::shlwapi::reg::SHSetValueW;
use crate::shlwapi::string::SHStrDupW;
use crate::shlwapi::string::StrCmpIW;
use crate::shlwapi::string::StrStrIW;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0C71B3CB5580: [u16; 8] = [0x0063u16, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_DCD1430B981B: [u16; 2] = [0x0025u16, 0x0000];
pub static RUSTCONV_WSTR_DD0FD6B09C2E: [u16; 5] = [0x0022u16, 0x0025, 0x0031, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_EE89605B31B0: [u16; 6] = [0x0073u16, 0x0068, 0x0065, 0x006C, 0x006C, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut unspecified_exts: [WCHAR; 17] = [
    '.' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    '-' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut image_exts: [WCHAR; 74] = [
    '.' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'j' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'j' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'j' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'j' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut audio_exts: [WCHAR; 68] = [
    '.' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut video_exts: [WCHAR; 91] = [
    '.' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    '-' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'V' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    '1' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut compressed_exts: [WCHAR; 6] = [
    '.' as i32 as WCHAR,
    'z' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut document_exts: [WCHAR; 17] = [
    '.' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    't' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut system_exts: [WCHAR; 6] = [
    '.' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut application_exts: [WCHAR; 55] = [
    '.' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    '.' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[no_mangle]
pub static mut type_text: [WCHAR; 5] = [
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    't' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[no_mangle]
pub static mut type_image: [WCHAR; 6] = [
    'i' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[no_mangle]
pub static mut type_audio: [WCHAR; 6] = [
    'a' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[no_mangle]
pub static mut type_video: [WCHAR; 6] = [
    'v' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[no_mangle]
pub static mut type_compressed: [WCHAR; 11] = [
    'c' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[no_mangle]
pub static mut type_document: [WCHAR; 9] = [
    'd' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[no_mangle]
pub static mut type_system: [WCHAR; 7] = [
    's' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[no_mangle]
pub static mut type_application: [WCHAR; 12] = [
    'a' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut known_types: [AssocPerceivedInfo; 9] = unsafe {
    [
        AssocPerceivedInfo {
            Type: ::core::ptr::null::<WCHAR>(),
            Perceived: PERCEIVED_TYPE_UNSPECIFIED,
            FlagHardcoded: (PERCEIVEDFLAG_HARDCODED as i32),
            FlagSoftcoded: (PERCEIVEDFLAG_SOFTCODED as i32),
            Extensions: &raw const unspecified_exts as PCWSTR,
        },
        AssocPerceivedInfo {
            Type: &raw const type_text as PCWSTR,
            Perceived: PERCEIVED_TYPE_TEXT,
            FlagHardcoded: (PERCEIVEDFLAG_HARDCODED as i32),
            FlagSoftcoded: SOFTCODED_NATIVESUPPORT,
            Extensions: ::core::ptr::null::<WCHAR>(),
        },
        AssocPerceivedInfo {
            Type: &raw const type_image as PCWSTR,
            Perceived: PERCEIVED_TYPE_IMAGE,
            FlagHardcoded: HARDCODED_NATIVE_GDIPLUS,
            FlagSoftcoded: (PERCEIVEDFLAG_SOFTCODED as i32),
            Extensions: &raw const image_exts as PCWSTR,
        },
        AssocPerceivedInfo {
            Type: &raw const type_audio as PCWSTR,
            Perceived: PERCEIVED_TYPE_AUDIO,
            FlagHardcoded: HARDCODED_NATIVE_WMSDK,
            FlagSoftcoded: (PERCEIVEDFLAG_SOFTCODED as i32),
            Extensions: &raw const audio_exts as PCWSTR,
        },
        AssocPerceivedInfo {
            Type: &raw const type_video as PCWSTR,
            Perceived: PERCEIVED_TYPE_VIDEO,
            FlagHardcoded: HARDCODED_NATIVE_WMSDK,
            FlagSoftcoded: (PERCEIVEDFLAG_SOFTCODED as i32),
            Extensions: &raw const video_exts as PCWSTR,
        },
        AssocPerceivedInfo {
            Type: &raw const type_compressed as PCWSTR,
            Perceived: PERCEIVED_TYPE_COMPRESSED,
            FlagHardcoded: HARDCODED_NATIVE_ZIPFLDR,
            FlagSoftcoded: (PERCEIVEDFLAG_SOFTCODED as i32),
            Extensions: &raw const compressed_exts as PCWSTR,
        },
        AssocPerceivedInfo {
            Type: &raw const type_document as PCWSTR,
            Perceived: PERCEIVED_TYPE_DOCUMENT,
            FlagHardcoded: (PERCEIVEDFLAG_HARDCODED as i32),
            FlagSoftcoded: (PERCEIVEDFLAG_SOFTCODED as i32),
            Extensions: &raw const document_exts as PCWSTR,
        },
        AssocPerceivedInfo {
            Type: &raw const type_system as PCWSTR,
            Perceived: PERCEIVED_TYPE_SYSTEM,
            FlagHardcoded: (PERCEIVEDFLAG_HARDCODED as i32),
            FlagSoftcoded: (PERCEIVEDFLAG_SOFTCODED as i32),
            Extensions: &raw const system_exts as PCWSTR,
        },
        AssocPerceivedInfo {
            Type: &raw const type_application as PCWSTR,
            Perceived: PERCEIVED_TYPE_APPLICATION,
            FlagHardcoded: (PERCEIVEDFLAG_HARDCODED as i32),
            FlagSoftcoded: (PERCEIVEDFLAG_SOFTCODED as i32),
            Extensions: &raw const application_exts as PCWSTR,
        },
    ]
};

// #undef _WIN32_WINNT / #define _WIN32_WINNT _WIN32_WINNT_VISTA: preprocessor-only, no Rust emitted.
// WINE_DEFAULT_DEBUG_CHANNEL(shell): debug-channel macro, no Rust emitted.
// extern ULONG WINAPI GetProcessOsVersion(void): extern declaration, provided by the caller's extern block.

#[repr(C)]
pub struct AssocPerceivedInfo {
    pub Type: PCWSTR,
    pub Perceived: PERCEIVED,
    pub FlagHardcoded: i32,
    pub FlagSoftcoded: i32,
    pub Extensions: PCWSTR,
}

unsafe impl Sync for AssocPerceivedInfo {}

pub const HARDCODED_NATIVE_WMSDK: i32 = (PERCEIVEDFLAG_HARDCODED | PERCEIVEDFLAG_NATIVESUPPORT | PERCEIVEDFLAG_WMSDK) as i32;

pub const HARDCODED_NATIVE_GDIPLUS: i32 = (PERCEIVEDFLAG_HARDCODED | PERCEIVEDFLAG_NATIVESUPPORT | PERCEIVEDFLAG_GDIPLUS) as i32;

pub const HARDCODED_NATIVE_ZIPFLDR: i32 = (PERCEIVEDFLAG_HARDCODED | PERCEIVEDFLAG_NATIVESUPPORT | PERCEIVEDFLAG_ZIPFOLDER) as i32;

pub const SOFTCODED_NATIVESUPPORT: i32 = (PERCEIVEDFLAG_SOFTCODED | PERCEIVEDFLAG_NATIVESUPPORT) as i32;

unsafe extern "C" fn SHELL_SetRegString(hKey: HKEY, pszSubKey: LPCWSTR, pszName: LPCWSTR, Type: UINT, pszData: LPCWSTR) -> HRESULT {
    let err: ULONG = SHSetValueW(hKey, pszSubKey, pszName, Type as DWORD, pszData as LPCVOID, ((lstrlenW(pszData) + 1) as usize * core::mem::size_of::<WCHAR>()) as DWORD) as ULONG;
    return HRESULT_FROM_WIN32!(err);
}

unsafe extern "C" fn _AssocOpenRegKey(hKey: HKEY, lpSubKey: PCWSTR, phkResult: PHKEY, bCreate: BOOL) -> HRESULT {
    *phkResult = core::ptr::null_mut();
    if hKey.is_null() {
        return HRESULT_FROM_WIN32!(ERROR_NO_ASSOCIATION);
    }

    let mut error: LSTATUS;
    if bCreate != 0 {
        error = RegCreateKeyExW(
            hKey,
            lpSubKey as LPCWSTR,
            0,
            core::ptr::null_mut(),
            0,
            MAXIMUM_ALLOWED,
            core::ptr::null_mut(),
            phkResult,
            core::ptr::null_mut(),
        );
    } else {
        error = RegOpenKeyExW(
            hKey,
            lpSubKey as LPCWSTR,
            0,
            MAXIMUM_ALLOWED,
            phkResult,
        );
    }

    if error == (ERROR_SUCCESS as LSTATUS) {
        return S_OK;
    }

    return HRESULT_FROM_WIN32!(error);
}

unsafe extern "C" fn AssocDeleteSubkey(hBase: HKEY, pszSubKey: LPCWSTR) {
    let mut hKey: HKEY = core::ptr::null_mut();
    if *pszSubKey != 0 && (_AssocOpenRegKey(hBase, pszSubKey, core::ptr::addr_of_mut!(hKey), (FALSE) as BOOL)) >= 0 {
        RegCloseKey(hKey);
        SHDeleteKeyW(hBase, pszSubKey);
    }
}

unsafe extern "C" fn AssocMakeVerbCommand(hVerb: HKEY, pszExeFallback: LPCWSTR, pVerb: *const ASSOCMAKEVERB) -> HRESULT {
    let mut Type: UINT = REG_SZ as UINT;
    let mut szCmd: [WCHAR; (MAX_PATH * 4) as _] = [0; (MAX_PATH * 4) as _];
    let mut pszExe: LPCWSTR = if !(*pVerb).pszExe.is_null() {
        (*pVerb).pszExe
    } else {
        pszExeFallback
    };
    let mut pszOrgArgs: LPCWSTR = if !(*pVerb).pszArgs.is_null() {
        (*pVerb).pszArgs
    } else {
        RUSTCONV_WSTR_96A296D224F2.as_ptr()
    };
    let mut pszArgs: LPCWSTR = if *pszOrgArgs as ::core::ffi::c_int != 0 {
        pszOrgArgs
    } else {
        RUSTCONV_WSTR_DD0FD6B09C2E.as_ptr()
    };

    if !(*pVerb).pszFriendlyName.is_null() && *(*pVerb).pszFriendlyName as ::core::ffi::c_int != 0 {
        SHELL_SetRegString(
            hVerb,
            ::core::ptr::null::<WCHAR>(),
            ::core::ptr::null::<WCHAR>(),
            REG_SZ as UINT,
            (*pVerb).pszFriendlyName,
        );
    }

    lstrcpyW(core::ptr::addr_of_mut!(szCmd) as LPWSTR, pszExe);
    PathQuoteSpacesW(core::ptr::addr_of_mut!(szCmd) as LPWSTR);
    lstrcatW(core::ptr::addr_of_mut!(szCmd) as LPWSTR, RUSTCONV_WSTR_8D71B3FAAB82.as_ptr());
    lstrcatW(core::ptr::addr_of_mut!(szCmd) as LPWSTR, pszArgs);
    if !StrStrIW(pszExe, RUSTCONV_WSTR_DCD1430B981B.as_ptr()).is_null()
        || !StrStrIW(pszOrgArgs, RUSTCONV_WSTR_DCD1430B981B.as_ptr()).is_null()
    {
        Type = REG_EXPAND_SZ as UINT;
    }
    return SHELL_SetRegString(
        hVerb,
        RUSTCONV_WSTR_0C71B3CB5580.as_ptr(),
        ::core::ptr::null::<WCHAR>(),
        Type,
        core::ptr::addr_of_mut!(szCmd) as *mut WCHAR as LPCWSTR,
    );
}

#[no_mangle]
pub unsafe extern "system" fn AssocMakeShell(mut Unknown_local: SIZE_T, hClass: HKEY, pszExe: LPCWSTR, pShell: *const ASSOCMAKESHELL) -> HRESULT {
    let mut hr: HRESULT = E_INVALIDARG;
    if hClass.is_null() || pszExe.is_null() || pShell.is_null() {
        return E_INVALIDARG;
    }
    let mut i: SIZE_T = 0;
    while i < (*pShell).Count as SIZE_T {
        let mut hVerb: HKEY = core::ptr::null_mut();
        let mut szVerbKey: [WCHAR; (6 + MAX_PATH) as _] = [0 as WCHAR; (6 + MAX_PATH) as _];
        let pVerb: *const ASSOCMAKEVERB = (*pShell).pVerbs.offset(i as isize) as *const ASSOCMAKEVERB;
        if !(*pVerb).pszVerb.is_null() {
            PathCombineW(
                &raw mut szVerbKey as LPWSTR,
                RUSTCONV_WSTR_EE89605B31B0.as_ptr(),
                (*pVerb).pszVerb,
            );
            AssocDeleteSubkey(hClass, &raw mut szVerbKey as *mut WCHAR as LPCWSTR); // Delete possibly existing old DDE/Delegate info
            if !(_AssocOpenRegKey(
                hClass,
                &raw mut szVerbKey as *mut WCHAR as PCWSTR,
                &raw mut hVerb,
                (TRUE) as BOOL,
            ) < 0)
            {
                hr = AssocMakeVerbCommand(hVerb, pszExe, pVerb);
                RegCloseKey(hVerb);
                if i == (*pShell).DefaultIndex as SIZE_T {
                    SHELL_SetRegString(
                        hClass,
                        RUSTCONV_WSTR_EE89605B31B0.as_ptr(),
                        core::ptr::null::<WCHAR>(),
                        REG_SZ,
                        (*pVerb).pszVerb,
                    );
                }
            }
        }
        i = i.wrapping_add(1);
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn AssocMakeProgid(
    Unknown1: SIZE_T,
    Unknown2: SIZE_T,
    Unknown3: SIZE_T,
    Unknown4: SIZE_T,
) -> HRESULT {
    // FIXME("stub\n");
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn AssocMakeApplicationByKeyW(
    Unknown1: SIZE_T,
    Unknown2: SIZE_T,
    Unknown3: SIZE_T,
) -> HRESULT {
    // FIXME("stub\n");
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn AssocMakeApplicationByKeyA(
    Unknown1: SIZE_T,
    Unknown2: SIZE_T,
    Unknown3: SIZE_T,
) -> HRESULT {
    AssocMakeApplicationByKeyW(Unknown1, Unknown2, Unknown3)
}

#[no_mangle]
pub unsafe extern "system" fn AssocCopyVerbs(hSrc: HKEY, hDst: HKEY) -> HRESULT {
    // FIXME("stub\n");
    E_NOTIMPL
}

unsafe extern "C" fn SHLWAPI_ParamAToW(
    mut lpszParam: LPCSTR,
    mut lpszBuff: LPWSTR,
    mut dwLen: DWORD,
    mut lpszOut: *mut LPWSTR,
) -> BOOL {
    if !lpszParam.is_null() {
        let mut dwStrLen: DWORD = MultiByteToWideChar(
            CP_ACP,
            0,
            lpszParam,
            -1,
            core::ptr::null_mut::<WCHAR>(),
            0,
        ) as DWORD;

        if dwStrLen < dwLen {
            *lpszOut = lpszBuff;
        } else {
            *lpszOut = HeapAlloc(
                GetProcessHeap(),
                0,
                dwStrLen.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD) as SIZE_T,
            ) as LPWSTR;
            if (*lpszOut).is_null() {
                return (FALSE) as BOOL;
            }
        }
        MultiByteToWideChar(
            CP_ACP,
            0,
            lpszParam,
            -1,
            *lpszOut,
            dwStrLen as core::ffi::c_int,
        );
    } else {
        *lpszOut = core::ptr::null_mut::<WCHAR>();
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn AssocCreate(
    mut clsid: CLSID,
    refiid: REFIID,
    lpInterface: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if lpInterface.is_null() {
        return E_INVALIDARG;
    }

    *(lpInterface as *mut DWORD) = 0;

    if !IsEqualGUID!(core::ptr::addr_of_mut!(clsid), core::ptr::addr_of!(CLSID_QueryAssociations)) {
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    return SHCoCreateInstance(
        core::ptr::null::<WCHAR>(),
        core::ptr::addr_of_mut!(clsid),
        core::ptr::null_mut::<IUnknown>(),
        refiid,
        lpInterface as *mut LPVOID,
    );
}

unsafe extern "C" fn AssocFindByBuiltinExtension(pszExt: LPCWSTR) -> *const AssocPerceivedInfo {
    let mut n: UINT = 0;
    while (n as usize)
        < (core::mem::size_of::<[AssocPerceivedInfo; 9]>()
            / core::mem::size_of_val(&known_types[0]))
    {
        let mut Ext: LPCWSTR = known_types[n as usize].Extensions;
        while !Ext.is_null() && *Ext != 0 {
            if StrCmpIW(Ext, pszExt) == 0 {
                return &known_types[n as usize] as *const AssocPerceivedInfo;
            }
            Ext = Ext.offset((strlenW(Ext) + 1) as isize);
        }
        n += 1;
    }
    return core::ptr::null();
}

unsafe extern "C" fn AssocFindByType(pszType: LPCWSTR) -> *const AssocPerceivedInfo {
    let mut n: UINT;
    n = 0;
    while (n as usize) < (core::mem::size_of_val(&known_types) / core::mem::size_of_val(&known_types[0])) {
        if !known_types[n as usize].Type.is_null() {
            if StrCmpIW(known_types[n as usize].Type, pszType) == 0 {
                return (core::ptr::addr_of!(known_types) as *const AssocPerceivedInfo).add(n as usize);
            }
        }
        n = n.wrapping_add(1);
    }
    return core::ptr::null::<AssocPerceivedInfo>();
}

#[no_mangle]
pub unsafe extern "system" fn AssocGetPerceivedType(
    lpszExt: LPCWSTR,
    lpType: *mut PERCEIVED,
    lpFlag: *mut INT,
    lppszType: *mut LPWSTR,
) -> HRESULT {
    static mut PerceivedTypeKey: [WCHAR; 14] = [
    0x0050u16, 0x0065, 0x0072, 0x0063, 0x0065, 0x0069, 0x0076, 0x0065, 0x0064, 0x0054,
    0x0079, 0x0070, 0x0065, 0x0000,
];
    static mut SystemFileAssociationsKey: [WCHAR; 26] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0046, 0x0069, 0x006C, 0x0065,
    0x0041, 0x0073, 0x0073, 0x006F, 0x0063, 0x0069, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000,
];
    let mut Info: *const AssocPerceivedInfo = core::ptr::null();

    WINE_TRACE!("(%s,%p,%p,%p)\n", debugstr_w(lpszExt as *const WCHAR), lpType, lpFlag, lppszType);

    Info = AssocFindByBuiltinExtension(lpszExt);
    if !Info.is_null() {
        *lpType = (*Info).Perceived;
        *lpFlag = (*Info).FlagHardcoded;
    } else {
        let mut Buffer: [WCHAR; 100] = [0; 100];
        let mut Size: DWORD = core::mem::size_of::<[WCHAR; 100]>() as DWORD;
        if RegGetValueW(
            HKEY_CLASSES_ROOT as HKEY,
            lpszExt,
            core::ptr::addr_of!(PerceivedTypeKey) as LPCWSTR,
            RRF_RT_REG_SZ as DWORD,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(Buffer) as PVOID,
            core::ptr::addr_of_mut!(Size),
        ) == ERROR_SUCCESS as LSTATUS
        {
            Info = AssocFindByType(core::ptr::addr_of_mut!(Buffer) as *mut WCHAR as LPCWSTR);
        }
        if Info.is_null() {
            let mut KeyName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            snprintfW(
                core::ptr::addr_of_mut!(KeyName) as *mut WCHAR,
                MAX_PATH as size_t,
                core::ptr::addr_of!(SystemFileAssociationsKey) as *const WCHAR,
                lpszExt,
            );
            Size = core::mem::size_of::<[WCHAR; 100]>() as DWORD;
            if RegGetValueW(
                HKEY_CLASSES_ROOT as HKEY,
                core::ptr::addr_of_mut!(KeyName) as *mut WCHAR as LPCWSTR,
                core::ptr::addr_of!(PerceivedTypeKey) as LPCWSTR,
                RRF_RT_REG_SZ as DWORD,
                core::ptr::null_mut::<DWORD>(),
                core::ptr::addr_of_mut!(Buffer) as PVOID,
                core::ptr::addr_of_mut!(Size),
            ) == ERROR_SUCCESS as LSTATUS
            {
                Info = AssocFindByType(core::ptr::addr_of_mut!(Buffer) as *mut WCHAR as LPCWSTR);
            }
        }
        if !Info.is_null() {
            *lpType = (*Info).Perceived;
            *lpFlag = (*Info).FlagSoftcoded;
        }
    }

    if !Info.is_null() {
        if !lppszType.is_null() && !(*Info).Type.is_null() {
            return SHStrDupW((*Info).Type as LPCWSTR, lppszType as *mut *mut WCHAR);
        }
        return if !(*Info).Type.is_null() {
            S_OK
        } else {
            E_FAIL
        };
    } else {
        *lpType = PERCEIVED_TYPE_UNSPECIFIED;
        *lpFlag = 0 as INT;
    }
    return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
}

#[no_mangle]
pub unsafe extern "system" fn AssocQueryKeyW(
    cfFlags: ASSOCF,
    assockey: ASSOCKEY,
    pszAssoc: LPCWSTR,
    pszExtra: LPCWSTR,
    phkeyOut: *mut HKEY,
) -> HRESULT {
    let mut hRet: HRESULT = 0;
    let mut lpAssoc: *mut IQueryAssociations = core::ptr::null_mut();

    hRet = AssocCreate(
        CLSID_QueryAssociations,
        core::ptr::addr_of!(IID_IQueryAssociations),
        core::ptr::addr_of_mut!(lpAssoc) as *mut *mut core::ffi::c_void,
    );
    if hRet < 0 {
        return hRet;
    }

    hRet = IQueryAssociations_Init!(
        lpAssoc,
        cfFlags & (SHLWAPI_DEF_ASSOCF as ASSOCF),
        pszAssoc,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if hRet >= 0 {
        hRet = IQueryAssociations_GetKey!(lpAssoc, cfFlags, assockey, pszExtra, phkeyOut);
    }

    IQueryAssociations_Release!(lpAssoc);
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn AssocQueryKeyA(
    cfFlags: ASSOCF,
    assockey: ASSOCKEY,
    pszAssoc: LPCSTR,
    pszExtra: LPCSTR,
    phkeyOut: *mut HKEY,
) -> HRESULT {
    let mut szAssocW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpszAssocW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut szExtraW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpszExtraW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut hRet: HRESULT = E_OUTOFMEMORY;
    WINE_TRACE!("(0x%x,%d,%s,%s,%p)\n", cfFlags, assockey as core::ffi::c_uint, debugstr_a(pszAssoc as *const core::ffi::c_char), debugstr_a(pszExtra as *const core::ffi::c_char), phkeyOut);
    if SHLWAPI_ParamAToW(
        pszAssoc,
        core::ptr::addr_of_mut!(szAssocW) as LPWSTR,
        MAX_PATH as DWORD,
        core::ptr::addr_of_mut!(lpszAssocW),
    ) != 0
        && SHLWAPI_ParamAToW(
            pszExtra,
            core::ptr::addr_of_mut!(szExtraW) as LPWSTR,
            MAX_PATH as DWORD,
            core::ptr::addr_of_mut!(lpszExtraW),
        ) != 0
    {
        hRet = AssocQueryKeyW(
            cfFlags,
            assockey,
            lpszAssocW as LPCWSTR,
            lpszExtraW as LPCWSTR,
            phkeyOut,
        );
    }
    if lpszAssocW != core::ptr::addr_of_mut!(szAssocW) as *mut WCHAR {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpszAssocW as PVOID,
        );
    }
    if lpszExtraW != core::ptr::addr_of_mut!(szExtraW) as *mut WCHAR {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpszExtraW as PVOID,
        );
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn AssocQueryStringW(
    cfFlags: ASSOCF,
    str: ASSOCSTR,
    pszAssoc: LPCWSTR,
    pszExtra: LPCWSTR,
    pszOut: LPWSTR,
    pcchOut: *mut DWORD,
) -> HRESULT {
    let mut hRet: HRESULT;
    let mut lpAssoc: *mut IQueryAssociations = core::ptr::null_mut();

    TRACE!(
        "(0x%x,%d,%s,%s,%p,%p)\n",
        cfFlags,
        str as u32,
        debugstr_w(pszAssoc),
        debugstr_w(pszExtra),
        pszOut,
        pcchOut
    );

    if pcchOut.is_null() {
        return E_UNEXPECTED;
    }

    hRet = AssocCreate(
        CLSID_QueryAssociations as CLSID,
        core::ptr::addr_of!(IID_IQueryAssociations),
        core::ptr::addr_of_mut!(lpAssoc) as *mut *mut core::ffi::c_void,
    );
    if hRet < 0 {
        return hRet;
    }

    hRet = IQueryAssociations_Init!(
        lpAssoc,
        cfFlags & (SHLWAPI_DEF_ASSOCF as ASSOCF),
        pszAssoc,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if hRet >= 0 {
        hRet = IQueryAssociations_GetString!(
            lpAssoc, cfFlags, str, pszExtra, pszOut, pcchOut,
        );
    }

    IQueryAssociations_Release!(lpAssoc);
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn AssocQueryStringA(
    cfFlags: ASSOCF,
    str: ASSOCSTR,
    pszAssoc: LPCSTR,
    pszExtra: LPCSTR,
    pszOut: LPSTR,
    pcchOut: *mut DWORD,
) -> HRESULT {
    let mut szAssocW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpszAssocW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut szExtraW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpszExtraW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut hRet: HRESULT = E_OUTOFMEMORY;

    WINE_TRACE!("(0x%x,0x%d,%s,%s,%p,%p)\n", cfFlags, str as ::core::ffi::c_uint, debugstr_a(pszAssoc as *const ::core::ffi::c_char), debugstr_a(pszExtra as *const ::core::ffi::c_char), pszOut, pcchOut);

    if pcchOut.is_null() {
        hRet = E_UNEXPECTED;
    } else if SHLWAPI_ParamAToW(
        pszAssoc,
        core::ptr::addr_of_mut!(szAssocW) as LPWSTR,
        MAX_PATH,
        core::ptr::addr_of_mut!(lpszAssocW),
    ) != 0
        && SHLWAPI_ParamAToW(
            pszExtra,
            core::ptr::addr_of_mut!(szExtraW) as LPWSTR,
            MAX_PATH,
            core::ptr::addr_of_mut!(lpszExtraW),
        ) != 0
    {
        let mut szReturnW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut lpszReturnW: *mut WCHAR = core::ptr::addr_of_mut!(szReturnW) as *mut WCHAR;
        let mut dwLenOut: DWORD = *pcchOut;

        if dwLenOut >= MAX_PATH as DWORD {
            lpszReturnW = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                (dwLenOut.wrapping_add(1 as DWORD) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as *mut WCHAR;
        } else {
            dwLenOut = (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                as DWORD;
        }

        if lpszReturnW.is_null() {
            hRet = E_OUTOFMEMORY;
        } else {
            hRet = AssocQueryStringW(
                cfFlags,
                str,
                lpszAssocW as LPCWSTR,
                lpszExtraW as LPCWSTR,
                lpszReturnW as LPWSTR,
                core::ptr::addr_of_mut!(dwLenOut),
            );

            if hRet >= 0 as HRESULT {
                dwLenOut = WideCharToMultiByte(
                    CP_ACP as UINT,
                    0 as DWORD,
                    lpszReturnW as LPCWSTR,
                    -(1 as ::core::ffi::c_int),
                    pszOut,
                    *pcchOut as ::core::ffi::c_int,
                    core::ptr::null::<CHAR>(),
                    core::ptr::null_mut::<BOOL>(),
                ) as DWORD;
            }

            *pcchOut = dwLenOut;
            if lpszReturnW != core::ptr::addr_of_mut!(szReturnW) as *mut WCHAR {
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    lpszReturnW as PVOID,
                );
            }
        }
    }

    if lpszAssocW != core::ptr::addr_of_mut!(szAssocW) as *mut WCHAR {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpszAssocW as PVOID,
        );
    }
    if lpszExtraW != core::ptr::addr_of_mut!(szExtraW) as *mut WCHAR {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpszExtraW as PVOID,
        );
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn AssocQueryStringByKeyW(
    cfFlags: ASSOCF,
    str: ASSOCSTR,
    hkAssoc: HKEY,
    pszExtra: LPCWSTR,
    pszOut: LPWSTR,
    pcchOut: *mut DWORD,
) -> HRESULT {
    let mut hRet: HRESULT = 0;
    let mut lpAssoc: *mut IQueryAssociations = core::ptr::null_mut::<IQueryAssociations>();
    WINE_TRACE!("(0x%x,0x%d,%p,%s,%p,%p)\n", cfFlags, str as ::core::ffi::c_uint, hkAssoc, debugstr_w(pszExtra as *const WCHAR), pszOut, pcchOut);

    hRet = AssocCreate(
        CLSID_QueryAssociations as CLSID,
        core::ptr::addr_of!(IID_IQueryAssociations),
        core::ptr::addr_of_mut!(lpAssoc) as *mut *mut ::core::ffi::c_void,
    );
    if hRet < 0 {
        return hRet;
    }

    hRet = IQueryAssociations_Init!(
        lpAssoc,
        cfFlags & (SHLWAPI_DEF_ASSOCF as ASSOCF),
        0,
        hkAssoc,
        core::ptr::null_mut::<HWND__>(),
    );

    if hRet >= 0 {
        hRet = IQueryAssociations_GetString!(lpAssoc, cfFlags, str, pszExtra, pszOut, pcchOut);
    }

    IQueryAssociations_Release!(lpAssoc);
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn AssocQueryStringByKeyA(
    cfFlags: ASSOCF,
    str: ASSOCSTR,
    hkAssoc: HKEY,
    pszExtra: LPCSTR,
    pszOut: LPSTR,
    pcchOut: *mut DWORD,
) -> HRESULT {
    let mut szExtraW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpszExtraW: *mut WCHAR = core::ptr::addr_of_mut!(szExtraW) as *mut WCHAR;
    let mut szReturnW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpszReturnW: *mut WCHAR = core::ptr::addr_of_mut!(szReturnW) as *mut WCHAR;
    let mut hRet: HRESULT = E_OUTOFMEMORY;

    if pcchOut.is_null() {
        hRet = E_INVALIDARG;
    } else if SHLWAPI_ParamAToW(
        pszExtra,
        core::ptr::addr_of_mut!(szExtraW) as *mut WCHAR as LPWSTR,
        MAX_PATH,
        core::ptr::addr_of_mut!(lpszExtraW),
    ) != 0
    {
        let mut dwLenOut: DWORD = *pcchOut;
        if dwLenOut >= MAX_PATH {
            lpszReturnW = HeapAlloc(
                GetProcessHeap(),
                0,
                (dwLenOut.wrapping_add(1) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as *mut WCHAR;
        }

        if !lpszReturnW.is_null() {
            hRet = AssocQueryStringByKeyW(
                cfFlags,
                str,
                hkAssoc,
                lpszExtraW as LPCWSTR,
                lpszReturnW as LPWSTR,
                core::ptr::addr_of_mut!(dwLenOut),
            );

            if hRet >= 0 {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    core::ptr::addr_of_mut!(szReturnW) as *mut WCHAR as LPCWSTR,
                    -1,
                    pszOut,
                    dwLenOut as core::ffi::c_int,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            }
            *pcchOut = dwLenOut;

            if lpszReturnW != core::ptr::addr_of_mut!(szReturnW) as *mut WCHAR {
                HeapFree(
                    GetProcessHeap(),
                    0,
                    lpszReturnW as PVOID,
                );
            }
        }
    }

    if lpszExtraW != core::ptr::addr_of_mut!(szExtraW) as *mut WCHAR {
        HeapFree(
            GetProcessHeap(),
            0,
            lpszExtraW as PVOID,
        );
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn AssocIsDangerous(lpszAssoc: LPCWSTR) -> BOOL {
    // FIXME("%s\n", debugstr_w(lpszAssoc));
    (FALSE as i32)
}
