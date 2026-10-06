use crate::setuplib::mui::AddKbLayoutsToRegistry;
use crate::setuplib::genlist::AppendGenericListEntry;
use crate::setuplib::genlist::CreateGenericList;
use crate::setuplib::genlist::DestroyGenericList;
use crate::setuplib::genlist::GetFirstListEntry;
use crate::setuplib::genlist::GetListEntryData;
use crate::setuplib::genlist::GetNumberOfListEntries;
use crate::setuplib::registry::GetRootKeyByPredefKey;
use crate::setuplib::infsupp::INF_GetData;
use crate::setuplib::infsupp::INF_GetDataField;
use crate::setuplib::mui::IsLanguageAvailable;
use crate::setuplib::mui::MUIGetLayoutsList;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_01F99237FAD1: [u16; 15] = [
    0x004Bu16, 0x0065, 0x0079, 0x0062, 0x006F, 0x0061, 0x0072, 0x0064, 0x004C, 0x0061,
    0x0079, 0x006F, 0x0075, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_09A7C6FF8E65: [u16; 8] = [0x0044u16, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079, 0x0000];
pub static RUSTCONV_WSTR_0F99D99140B6: [u16; 10] = [0x0041u16, 0x0043, 0x0050, 0x0049, 0x0020, 0x0042, 0x0049, 0x004F, 0x0053, 0x0000];
#[cfg(target_arch = "x86_64")]
pub static RUSTCONV_WSTR_1C7B8DF6585E: [u16; 21] = [
    0x004Du16, 0x0061, 0x0070, 0x002E, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074,
    0x0065, 0x0072, 0x002E, 0x004E, 0x0054, 0x0061, 0x006D, 0x0064, 0x0036, 0x0034,
    0x0000,
];
#[cfg(target_arch = "aarch64")]
pub static RUSTCONV_WSTR_1C7B8DF6585E: [u16; 21] = [
    0x004Du16, 0x0061, 0x0070, 0x002E, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074,
    0x0065, 0x0072, 0x002E, 0x004E, 0x0054, 0x0061, 0x0072, 0x006D, 0x0036, 0x0034,
    0x0000,
];
#[cfg(target_arch = "x86")]
pub static RUSTCONV_WSTR_1C7B8DF6585E: [u16; 19] = [
    0x004Du16, 0x0061, 0x0070, 0x002E, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074,
    0x0065, 0x0072, 0x002E, 0x004E, 0x0054, 0x0078, 0x0038, 0x0036, 0x0000,
];
pub static RUSTCONV_WSTR_23C470F1D8A6: [u16; 28] = [
    0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0053, 0x0065, 0x0074,
    0x0074, 0x0069, 0x006E, 0x0067, 0x0073, 0x002E, 0x0058, 0x0052, 0x0065, 0x0073,
    0x006F, 0x006C, 0x0075, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_252BD2199A94: [u16; 8] = [0x0041u16, 0x0043, 0x0050, 0x0049, 0x0020, 0x0055, 0x0050, 0x0000];
pub static RUSTCONV_WSTR_272522956B9D: [u16; 9] = [0x0046u16, 0x0069, 0x006C, 0x0065, 0x0073, 0x002E, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_2C284597401B: [u16; 67] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0048, 0x0041,
    0x0052, 0x0044, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x0044, 0x0065, 0x0073,
    0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x005C, 0x0053,
    0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x004D, 0x0075, 0x006C, 0x0074,
    0x0069, 0x0066, 0x0075, 0x006E, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0041,
    0x0064, 0x0061, 0x0070, 0x0074, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_344F2CEBFA6E: [u16; 11] = [
    0x0049u16, 0x0064, 0x0065, 0x006E, 0x0074, 0x0069, 0x0066, 0x0069, 0x0065, 0x0072,
    0x0000,
];
pub static RUSTCONV_WSTR_39CDE5D97C33: [u16; 8] = [0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_3F9A22015E8C: [u16; 6] = [0x0050u16, 0x0043, 0x0020, 0x004D, 0x0050, 0x0000];
pub static RUSTCONV_WSTR_400952F905DF: [u16; 18] = [
    0x0044u16, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079, 0x0043, 0x006F, 0x006E,
    0x0074, 0x0072, 0x006F, 0x006C, 0x006C, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_4220FDAA5966: [u16; 14] = [
    0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x004C, 0x0061, 0x0079,
    0x006F, 0x0075, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_4BF3B3F38290: [u16; 45] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0044, 0x0065, 0x0076,
    0x0069, 0x0063, 0x0065, 0x0030, 0x0000,
];
pub static RUSTCONV_WSTR_4C2425E1D768: [u16; 4] = [0x004Eu16, 0x004C, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_4CFDAEB88300: [u16; 28] = [
    0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0053, 0x0065, 0x0074,
    0x0074, 0x0069, 0x006E, 0x0067, 0x0073, 0x002E, 0x0059, 0x0052, 0x0065, 0x0073,
    0x006F, 0x006C, 0x0075, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_5A6B5AE7F088: [u16; 12] = [
    0x004Du16, 0x0061, 0x0070, 0x002E, 0x0044, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061,
    0x0079, 0x0000,
];
pub static RUSTCONV_WSTR_62A77C7DF8CF: [u16; 67] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0048, 0x0041,
    0x0052, 0x0044, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x0044, 0x0045, 0x0053,
    0x0043, 0x0052, 0x0049, 0x0050, 0x0054, 0x0049, 0x004F, 0x004E, 0x005C, 0x0053,
    0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x004D, 0x0075, 0x006C, 0x0074,
    0x0069, 0x0066, 0x0075, 0x006E, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0041,
    0x0064, 0x0061, 0x0070, 0x0074, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_64E8694B8B78: [u16; 41] = [
    0x002Eu16, 0x0044, 0x0045, 0x0046, 0x0041, 0x0055, 0x004C, 0x0054, 0x005C, 0x0043,
    0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0020, 0x0050, 0x0061, 0x006E,
    0x0065, 0x006C, 0x005C, 0x0049, 0x006E, 0x0074, 0x0065, 0x0072, 0x006E, 0x0061,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0061, 0x006C, 0x005C, 0x0047, 0x0065, 0x006F,
    0x0000,
];
pub static RUSTCONV_WSTR_86E5823B1278: [u16; 12] = [
    0x0050u16, 0x0061, 0x0067, 0x0069, 0x006E, 0x0067, 0x0046, 0x0069, 0x006C, 0x0065,
    0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_8EA4B3C5C5D4: [u16; 16] = [
    0x0049u16, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x004C, 0x0061, 0x006E,
    0x0067, 0x0075, 0x0061, 0x0067, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_92C451776EF8: [u16; 7] = [0x004Eu16, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_9A24410C9730: [u16; 27] = [
    0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0053, 0x0065, 0x0074,
    0x0074, 0x0069, 0x006E, 0x0067, 0x0073, 0x002E, 0x0042, 0x0069, 0x0074, 0x0073,
    0x0050, 0x0065, 0x0072, 0x0050, 0x0065, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_9C784B4E5298: [u16; 9] = [0x0043u16, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_A37A558548DD: [u16; 37] = [
    0x002Eu16, 0x0044, 0x0045, 0x0046, 0x0041, 0x0055, 0x004C, 0x0054, 0x005C, 0x0043,
    0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0020, 0x0050, 0x0061, 0x006E,
    0x0065, 0x006C, 0x005C, 0x0049, 0x006E, 0x0074, 0x0065, 0x0072, 0x006E, 0x0061,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0061, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_A62088E6F2CF: [u16; 9] = [0x004Cu16, 0x0061, 0x006E, 0x0067, 0x0075, 0x0061, 0x0067, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_C5FEC8B2F1E4: [u16; 67] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0053, 0x0065, 0x0073, 0x0073, 0x0069, 0x006F, 0x006E,
    0x0020, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x005C, 0x004D,
    0x0065, 0x006D, 0x006F, 0x0072, 0x0079, 0x0020, 0x004D, 0x0061, 0x006E, 0x0061,
    0x0067, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0000,
];
#[cfg(target_arch = "x86_64")]
pub static RUSTCONV_WSTR_CA81E426C5B3: [u16; 17] = [
    0x0043u16, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x002E, 0x004E,
    0x0054, 0x0061, 0x006D, 0x0064, 0x0036, 0x0034, 0x0000,
];
#[cfg(target_arch = "aarch64")]
pub static RUSTCONV_WSTR_CA81E426C5B3: [u16; 17] = [
    0x0043u16, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x002E, 0x004E,
    0x0054, 0x0061, 0x0072, 0x006D, 0x0036, 0x0034, 0x0000,
];
#[cfg(target_arch = "x86")]
pub static RUSTCONV_WSTR_CA81E426C5B3: [u16; 15] = [
    0x0043u16, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x002E, 0x004E,
    0x0054, 0x0078, 0x0038, 0x0036, 0x0000,
];
pub static RUSTCONV_WSTR_CD5D42B78159: [u16; 8] = [0x0041u16, 0x0043, 0x0050, 0x0049, 0x0020, 0x004D, 0x0050, 0x0000];
pub static RUSTCONV_WSTR_D440D5415561: [u16; 63] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0048, 0x0041,
    0x0052, 0x0044, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x0044, 0x0045, 0x0053,
    0x0043, 0x0052, 0x0049, 0x0050, 0x0054, 0x0049, 0x004F, 0x004E, 0x005C, 0x0053,
    0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0065, 0x006E, 0x0074,
    0x0072, 0x0061, 0x006C, 0x0050, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073, 0x0073,
    0x006F, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_D700A80DF51A: [u16; 6] = [0x0050u16, 0x0043, 0x0020, 0x0055, 0x0050, 0x0000];
pub static RUSTCONV_WSTR_E3329FFFD7A6: [u16; 96] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0048, 0x0061, 0x0072, 0x0064, 0x0077,
    0x0061, 0x0072, 0x0065, 0x0020, 0x0050, 0x0072, 0x006F, 0x0066, 0x0069, 0x006C,
    0x0065, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074,
    0x005C, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075,
    0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076,
    0x0069, 0x0063, 0x0065, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0044, 0x0065,
    0x0076, 0x0069, 0x0063, 0x0065, 0x0030, 0x0000,
];
pub static RUSTCONV_WSTR_EAE8EEC66C20: [u16; 46] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x004E, 0x004C, 0x0053, 0x005C, 0x004C, 0x0061, 0x006E,
    0x0067, 0x0075, 0x0061, 0x0067, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_ED2F5E1AB2BB: [u16; 16] = [
    0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x004C, 0x0061, 0x006E,
    0x0067, 0x0075, 0x0061, 0x0067, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_EF20E0741A09: [u16; 9] = [0x004Bu16, 0x0065, 0x0079, 0x0062, 0x006F, 0x0061, 0x0072, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_EF3C7FD20784: [u16; 13] = [
    0x004Du16, 0x0061, 0x0070, 0x002E, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074,
    0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_F625A5873448: [u16; 37] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_FA8A27D23A1A: [u16; 6] = [0x0053u16, 0x0074, 0x0061, 0x0072, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_FF3CF31B44D6: [u16; 7] = [0x004Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0065, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut DefaultLanguageIndex: ULONG = 0 as ULONG;

pub const NDEBUG: () = ();
pub type PPROCESS_ENTRY_ROUTINE = Option<unsafe extern "system" fn(PCWSTR, PCWSTR, *mut PVOID, PBOOLEAN, PVOID) -> UCHAR>;
#[repr(C)]
pub struct _LANG_ENTRY_PARAM {
    pub uIndex: ULONG,
    pub DefaultLanguage: PWCHAR,
}
pub type LANG_ENTRY_PARAM = _LANG_ENTRY_PARAM;
pub type PLANG_ENTRY_PARAM = *mut _LANG_ENTRY_PARAM;
unsafe extern "C" fn IsAcpiComputer() -> BOOLEAN {
    let mut MultiKeyPathU: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_62A77C7DF8CF.as_ptr());
    let mut IdentifierU: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_344F2CEBFA6E.as_ptr());
    let mut AcpiBiosIdentifier: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_0F99D99140B6.as_ptr());
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut pDeviceInformation: PKEY_BASIC_INFORMATION = core::ptr::null_mut();
    let mut DeviceInfoLength: ULONG = core::mem::size_of::<KEY_BASIC_INFORMATION>() as ULONG
        + 50 * core::mem::size_of::<WCHAR>() as ULONG;
    let mut pValueInformation: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut ValueInfoLength: ULONG = core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG
        + 50 * core::mem::size_of::<WCHAR>() as ULONG;
    let mut RequiredSize: ULONG = 0;
    let mut IndexDevice: ULONG = 0;
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut hDevicesKey: HANDLE = core::ptr::null_mut();
    let mut hDeviceKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut ret: BOOLEAN = (FALSE) as u8;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut::<core::ffi::c_void>() as HANDLE;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(MultiKeyPathU) as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>() as PVOID;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>() as PVOID;

    Status = NtOpenKey(
        core::ptr::addr_of_mut!(hDevicesKey),
        KEY_ENUMERATE_SUB_KEYS,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT!(
            b"NtOpenKey() failed with status 0x%08lx\n\0".as_ptr() as *const i8,
            Status,
        );
    } else {
        pDeviceInformation = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            0,
            DeviceInfoLength as SIZE_T,
        ) as PKEY_BASIC_INFORMATION;
        if pDeviceInformation.is_null() {
            DPRINT!(b"RtlAllocateHeap() failed\n\0".as_ptr() as *const i8);
            Status = STATUS_NO_MEMORY;
        } else {
            pValueInformation = RtlAllocateHeap(
                RtlGetProcessHeap!(),
                0,
                ValueInfoLength as SIZE_T,
            ) as PKEY_VALUE_PARTIAL_INFORMATION;
            if pValueInformation.is_null() {
                DPRINT!(b"RtlAllocateHeap() failed\n\0".as_ptr() as *const i8);
                Status = STATUS_NO_MEMORY;
            } else {
                loop {
                    Status = NtEnumerateKey(
                        hDevicesKey,
                        IndexDevice,
                        KeyBasicInformation,
                        pDeviceInformation as PVOID,
                        DeviceInfoLength,
                        core::ptr::addr_of_mut!(RequiredSize),
                    );

                    if Status == STATUS_NO_MORE_ENTRIES {
                        break;
                    } else if Status == STATUS_BUFFER_OVERFLOW
                        || Status == STATUS_BUFFER_TOO_SMALL
                    {
                        RtlFreeHeap(
                            RtlGetProcessHeap!(),
                            0,
                            pDeviceInformation as PVOID,
                        );
                        DeviceInfoLength = RequiredSize;
                        pDeviceInformation = RtlAllocateHeap(
                            RtlGetProcessHeap!(),
                            0,
                            DeviceInfoLength as SIZE_T,
                        ) as PKEY_BASIC_INFORMATION;
                        if pDeviceInformation.is_null() {
                            DPRINT!(b"RtlAllocateHeap() failed\n\0".as_ptr() as *const i8);
                            Status = STATUS_NO_MEMORY;
                            break;
                        } else {
                            Status = NtEnumerateKey(
                                hDevicesKey,
                                IndexDevice,
                                KeyBasicInformation,
                                pDeviceInformation as PVOID,
                                DeviceInfoLength,
                                core::ptr::addr_of_mut!(RequiredSize),
                            );
                        }
                    }

                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT!(
                            b"NtEnumerateKey() failed with status 0x%08lx\n\0".as_ptr()
                                as *const i8,
                            Status,
                        );
                        break;
                    }

                    IndexDevice = IndexDevice.wrapping_add(1);

                    DeviceName.Length = (*pDeviceInformation).NameLength as USHORT;
                    DeviceName.MaximumLength = (*pDeviceInformation).NameLength as USHORT;
                    DeviceName.Buffer = core::ptr::addr_of_mut!((*pDeviceInformation).Name)
                        as *mut WCHAR
                        as PWSTR;

                    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
                    ObjectAttributes.RootDirectory = hDevicesKey;
                    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
                    ObjectAttributes.ObjectName =
                        core::ptr::addr_of_mut!(DeviceName) as PUNICODE_STRING;
                    ObjectAttributes.SecurityDescriptor =
                        core::ptr::null_mut::<core::ffi::c_void>() as PVOID;
                    ObjectAttributes.SecurityQualityOfService =
                        core::ptr::null_mut::<core::ffi::c_void>() as PVOID;

                    Status = NtOpenKey(
                        core::ptr::addr_of_mut!(hDeviceKey),
                        KEY_QUERY_VALUE,
                        core::ptr::addr_of_mut!(ObjectAttributes),
                    );

                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT!(
                            b"NtOpenKey() failed with status 0x%08lx\n\0".as_ptr()
                                as *const i8,
                            Status,
                        );
                        break;
                    }

                    Status = NtQueryValueKey(
                        hDeviceKey,
                        core::ptr::addr_of_mut!(IdentifierU),
                        KeyValuePartialInformation,
                        pValueInformation as PVOID,
                        ValueInfoLength,
                        core::ptr::addr_of_mut!(RequiredSize),
                    );

                    if Status == STATUS_BUFFER_OVERFLOW
                        || Status == STATUS_BUFFER_TOO_SMALL
                    {
                        RtlFreeHeap(
                            RtlGetProcessHeap!(),
                            0,
                            pValueInformation as PVOID,
                        );
                        ValueInfoLength = RequiredSize;
                        pValueInformation = RtlAllocateHeap(
                            RtlGetProcessHeap!(),
                            0,
                            ValueInfoLength as SIZE_T,
                        ) as PKEY_VALUE_PARTIAL_INFORMATION;
                        if pValueInformation.is_null() {
                            DPRINT!(b"RtlAllocateHeap() failed\n\0".as_ptr() as *const i8);
                            Status = STATUS_NO_MEMORY;
                            break;
                        } else {
                            Status = NtQueryValueKey(
                                hDeviceKey,
                                core::ptr::addr_of_mut!(IdentifierU),
                                KeyValuePartialInformation,
                                pValueInformation as PVOID,
                                ValueInfoLength,
                                core::ptr::addr_of_mut!(RequiredSize),
                            );
                        }
                    }

                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT!(
                            b"NtQueryValueKey() failed with status 0x%08lx\n\0".as_ptr()
                                as *const i8,
                            Status,
                        );
                    } else if (*pValueInformation).Type != REG_SZ {
                        DPRINT!(
                            b"Wrong registry type: got 0x%lx, expected 0x%lx\n\0".as_ptr()
                                as *const i8,
                            (*pValueInformation).Type,
                            REG_SZ,
                        );
                    } else {
                        ValueName.Length = (*pValueInformation).DataLength as USHORT;
                        ValueName.MaximumLength = (*pValueInformation).DataLength as USHORT;
                        ValueName.Buffer = core::ptr::addr_of_mut!((*pValueInformation).Data)
                            as *mut UCHAR
                            as PWCHAR
                            as PWSTR;

                        if ValueName.Length as usize >= core::mem::size_of::<WCHAR>() as usize
                            && *ValueName.Buffer.offset(
                                (ValueName.Length as usize)
                                    .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                                    .wrapping_sub(1 as usize)
                                    as isize,
                            ) as core::ffi::c_int
                                == UNICODE_NULL as core::ffi::c_int
                        {
                            ValueName.Length = (ValueName.Length as usize
                                - core::mem::size_of::<WCHAR>() as usize)
                                as USHORT;
                        }

                        if RtlEqualUnicodeString(
                            core::ptr::addr_of!(ValueName) as PCUNICODE_STRING,
                            core::ptr::addr_of!(AcpiBiosIdentifier) as PCUNICODE_STRING,
                            (FALSE) as BOOLEAN,
                        ) != 0
                        {
                            DPRINT!(b"Found ACPI BIOS\n\0".as_ptr() as *const i8);
                            ret = (TRUE) as u8;
                            break;
                        }
                    }

                    NtClose(hDeviceKey);
                    hDeviceKey = core::ptr::null_mut();
                }
            }
        }
    }

    if !pDeviceInformation.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap!(),
            0,
            pDeviceInformation as PVOID,
        );
    }
    if !pValueInformation.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap!(),
            0,
            pValueInformation as PVOID,
        );
    }
    if !hDevicesKey.is_null() {
        NtClose(hDevicesKey);
    }
    if !hDeviceKey.is_null() {
        NtClose(hDeviceKey);
    }

    return ret;
}

unsafe extern "C" fn GetComputerIdentifier(
    mut Identifier: PWSTR,
    mut IdentifierLength: ULONG,
) -> BOOLEAN {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ComputerIdentifier: LPCWSTR = core::ptr::null();
    let mut ProcessorsKey: HANDLE = core::ptr::null_mut();
    let mut pFullInfo: PKEY_FULL_INFORMATION = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut SizeNeeded: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    Size = core::mem::size_of::<KEY_FULL_INFORMATION>() as ULONG;
    pFullInfo = RtlAllocateHeap(RtlGetProcessHeap!(), 0, (Size) as SIZE_T) as PKEY_FULL_INFORMATION;
    if pFullInfo.is_null() {
        return (FALSE) as u8;
    }

    RtlInitUnicodeString(&mut KeyName, RUSTCONV_WSTR_D440D5415561.as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(&mut ProcessorsKey, KEY_QUERY_VALUE, &mut ObjectAttributes);
    if ((NT_SUCCESS!(Status)) == 0) {
        RtlFreeHeap(RtlGetProcessHeap!(), 0, (pFullInfo).cast());
        return (FALSE) as u8;
    }

    Status = NtQueryKey(ProcessorsKey, KeyFullInformation, (pFullInfo).cast(), Size, &mut Size);
    NtClose(ProcessorsKey);
    if ((NT_SUCCESS!(Status)) == 0) && Status != STATUS_BUFFER_OVERFLOW {
        RtlFreeHeap(RtlGetProcessHeap!(), 0, (pFullInfo).cast());
        return (FALSE) as u8;
    }

    if (*pFullInfo).SubKeys == 0 {
        RtlFreeHeap(RtlGetProcessHeap!(), 0, (pFullInfo).cast());
        return (FALSE) as u8;
    }

    if IsAcpiComputer() != 0 {
        if (*pFullInfo).SubKeys == 1 {
            ComputerIdentifier = RUSTCONV_WSTR_252BD2199A94.as_ptr();
        } else {
            ComputerIdentifier = RUSTCONV_WSTR_CD5D42B78159.as_ptr();
        }
    } else {
        if (*pFullInfo).SubKeys == 1 {
            ComputerIdentifier = RUSTCONV_WSTR_D700A80DF51A.as_ptr();
        } else {
            ComputerIdentifier = RUSTCONV_WSTR_3F9A22015E8C.as_ptr();
        }
    }

    RtlFreeHeap(RtlGetProcessHeap!(), 0, (pFullInfo).cast());

    SizeNeeded = ((wcslen(ComputerIdentifier) + 1) * core::mem::size_of::<WCHAR>()) as ULONG;
    if SizeNeeded > IdentifierLength {
        return (FALSE) as u8;
    }

    RtlCopyMemory!(Identifier as *mut _, ComputerIdentifier as *const _, SizeNeeded);

    return (TRUE) as u8;
}

unsafe extern "C" fn AddEntriesFromInfSection(
    mut List: PGENERIC_LIST,
    mut InfFile: HINF,
    mut SectionName: PCWSTR,
    mut pContext: PINFCONTEXT,
    mut ProcessEntry: PPROCESS_ENTRY_ROUTINE,
    mut Parameter: PVOID,
) -> LONG {
    let mut TotalCount: LONG = 0;
    let mut KeyName: PCWSTR = core::ptr::null();
    let mut KeyValue: PCWSTR = core::ptr::null();
    let mut UserData: PVOID = core::ptr::null_mut();
    let mut Current: BOOLEAN = 0;
    let mut RetVal: UCHAR = 0;

    if SpInfFindFirstLine!(InfFile, SectionName, core::ptr::null(), pContext) == 0 {
        return -1;
    }

    loop {
        if INF_GetDataField(pContext, 0, &mut KeyName) == 0 {
            DPRINT!("INF_GetDataField() failed\n");
            return -1;
        }

        if INF_GetDataField(pContext, 1, &mut KeyValue) == 0 {
            DPRINT!("INF_GetDataField() failed\n");
            INF_FreeData(KeyName);
            return -1;
        }

        UserData = core::ptr::null_mut();
        Current = (FALSE) as u8;
        RetVal = (ProcessEntry.unwrap_unchecked())(
            KeyName,
            KeyValue,
            &mut UserData,
            &mut Current,
            Parameter,
        );
        INF_FreeData(KeyName);
        INF_FreeData(KeyValue);

        if RetVal == 0 {
            DPRINT!("ProcessEntry() failed\n");
            return -1;
        } else if RetVal == 1 {
            AppendGenericListEntry(List, UserData, Current);
            TotalCount += 1;
        }

        if SpInfFindNextLine!(pContext, pContext) == 0 {
            break;
        }
    }

    TotalCount
}
unsafe extern "system" fn DefaultProcessEntry(
    KeyName: PCWSTR,
    KeyValue: PCWSTR,
    UserData: *mut PVOID,
    Current: PBOOLEAN,
    Parameter: PVOID,
) -> UCHAR {
    let CompareKey: PWSTR = Parameter as PWSTR;

    let mut GenEntry: PGENENTRY;
    let mut IdSize: SIZE_T;
    let mut ValueSize: SIZE_T;

    IdSize = ((wcslen(KeyName) + 1) * core::mem::size_of::<u16>()) as _;
    ValueSize = ((wcslen(KeyValue) + 1) * core::mem::size_of::<u16>()) as _;

    GenEntry = (RtlAllocateHeap(
        ProcessHeap,
        0,
        core::mem::size_of::<GENENTRY>() as SIZE_T + IdSize + ValueSize,
    )).cast();
    if GenEntry.is_null() {
        DPRINT1!("RtlAllocateHeap() failed\n");
        return 0;
    }

    (*GenEntry).Id = (GenEntry as usize + core::mem::size_of::<GENENTRY>()) as PCWSTR;
    (*GenEntry).Value = (GenEntry as ULONG_PTR + core::mem::size_of::<GENENTRY>() as ULONG_PTR + IdSize) as PCWSTR;
    RtlStringCbCopyW((*GenEntry).Id as PWSTR, (IdSize) as _, KeyName);
    RtlStringCbCopyW((*GenEntry).Value as PWSTR, (ValueSize) as _, KeyValue);

    *UserData = GenEntry as PVOID;
    *Current = if !CompareKey.is_null() {
        // C `!_wcsicmp(..)` is a logical not; Rust `!` on an int is bitwise.
        (_wcsicmp(KeyName, CompareKey) == 0) as u8
    } else {
        (FALSE) as u8
    };

    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn AddComputerTypeEntries(
    mut InfFile: HINF,
    mut List: PGENERIC_LIST,
    mut SectionName: PWSTR,
) -> BOOLEAN {
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut KeyName: PCWSTR = core::ptr::null();
    let mut KeyValue: PCWSTR = core::ptr::null();
    let mut ComputerIdentifier: [WCHAR; 128] = [0; 128];
    let mut ComputerKey: [WCHAR; 32] = [0; 32];
    let mut Count1: ULONG = 0;
    let mut Count2: ULONG = 0;

    if GetComputerIdentifier(
        ComputerIdentifier.as_mut_ptr(),
        128,
    ) == 0
    {
        ComputerIdentifier[0] = 0;
    }

    DPRINT!(
        b"Computer identifier: '%S'\n\0".as_ptr() as *const i8,
        ComputerIdentifier.as_ptr(),
    );

    if SpInfFindFirstLine!(
        InfFile,
        SectionName,
        core::ptr::null(),
        &mut Context,
    ) == 0
    {
        return (FALSE) as u8;
    }

    // C do { ... continue; ... } while (SpInfFindNextLine(...)): a `continue`
    // goes to the condition (next line), so it is `break 'body` here.
    loop {
        'body: {
        let mut FoundId: BOOLEAN = 0;

        if INF_GetDataField(
            &mut Context,
            1,
            &mut KeyValue,
        ) == 0
        {
            DPRINT!(b"INF_GetDataField() failed\n\0".as_ptr() as *const i8);
            return (FALSE) as u8;
        }

        DPRINT!(
            b"KeyValue: %S\n\0".as_ptr() as *const i8,
            KeyValue,
        );
        FoundId = !wcsstr(
            ComputerIdentifier.as_ptr(),
            KeyValue,
        ).is_null() as BOOLEAN;
        INF_FreeData(KeyValue);

        if FoundId == 0 {
            break 'body;
        }

        if INF_GetDataField(
            &mut Context,
            0,
            &mut KeyName,
        ) == 0
        {
            DPRINT!(b"INF_GetDataField() failed\n\0".as_ptr() as *const i8);
            return (FALSE) as u8;
        }

        DPRINT!(
            b"Computer key: %S\n\0".as_ptr() as *const i8,
            KeyName,
        );
        RtlStringCchCopyW(
            ComputerKey.as_mut_ptr(),
            (ARRAYSIZE!(ComputerKey)) as _,
            KeyName,
        );
        INF_FreeData(KeyName);
        }

        if SpInfFindNextLine!(&mut Context, &mut Context) == 0 {
            break;
        }
    }

    Count1 = (AddEntriesFromInfSection(
        List,
        InfFile,
        RUSTCONV_WSTR_9C784B4E5298.as_ptr(),
        &mut Context,
        Some(DefaultProcessEntry),
        ComputerKey.as_mut_ptr() as PVOID,
    )) as u32;
    Count2 = (AddEntriesFromInfSection(
        List,
        InfFile,
        RUSTCONV_WSTR_CA81E426C5B3.as_ptr(),
        &mut Context,
        Some(DefaultProcessEntry),
        ComputerKey.as_mut_ptr() as PVOID,
    )) as u32;

    if Count1 == (1 as u32).wrapping_neg() as ULONG
        && Count2 == (1 as u32).wrapping_neg() as ULONG
    {
        return (FALSE) as u8;
    }

    return (TRUE) as u8;
}

#[no_mangle]
pub unsafe extern "C" fn CreateComputerTypeList(InfFile: HINF) -> PGENERIC_LIST {
    let mut List: PGENERIC_LIST = core::ptr::null_mut();
    let mut Success: BOOLEAN = 0;

    List = CreateGenericList();
    if List.is_null() {
        return core::ptr::null_mut();
    }

    Success = AddComputerTypeEntries(InfFile, List, RUSTCONV_WSTR_EF3C7FD20784.as_ptr() as PWSTR);
    Success = (Success as i32 | AddComputerTypeEntries(InfFile, List, RUSTCONV_WSTR_1C7B8DF6585E.as_ptr() as PWSTR) as i32) as BOOLEAN;

    if Success == 0 {
        DestroyGenericList(List, (TRUE) as BOOLEAN);
        return core::ptr::null_mut();
    }

    return List;
}

unsafe extern "C" fn GetDisplayIdentifier(
    mut Identifier: PWSTR,
    mut IdentifierLength: ULONG,
) -> BOOLEAN {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut Buffer: [u16; 32] = [0; 32];
    let mut BusKey: HANDLE = core::ptr::null_mut();
    let mut BusInstanceKey: HANDLE = core::ptr::null_mut();
    let mut ControllerKey: HANDLE = core::ptr::null_mut();
    let mut ControllerInstanceKey: HANDLE = core::ptr::null_mut();
    let mut BusInstance: ULONG = 0;
    let mut ControllerInstance: ULONG = 0;
    let mut BufferLength: ULONG = 0;
    let mut ReturnedLength: ULONG = 0;
    let mut ValueInfo: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    DPRINT!("GetDisplayIdentifier() called\n");

    RtlInitUnicodeString(&mut KeyName, RUSTCONV_WSTR_2C284597401B.as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(
        &mut BusKey,
        KEY_ENUMERATE_SUB_KEYS,
        &mut ObjectAttributes,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT!("NtOpenKey() failed (Status %lx)\n", Status);
        return (FALSE) as u8;
    }

    BusInstance = 0;
    loop {
        RtlStringCchPrintfW!(
            Buffer.as_mut_ptr(),
            ARRAYSIZE!(Buffer),
            RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
            BusInstance,
        );
        RtlInitUnicodeString(&mut KeyName, Buffer.as_mut_ptr());
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = BusKey;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.ObjectName = &mut KeyName;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenKey(
            &mut BusInstanceKey,
            KEY_ENUMERATE_SUB_KEYS,
            &mut ObjectAttributes,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT!("NtOpenKey() failed (Status %lx)\n", Status);
            NtClose(BusKey);
            return (FALSE) as u8;
        }

        RtlInitUnicodeString(&mut KeyName, RUSTCONV_WSTR_400952F905DF.as_ptr());
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = BusInstanceKey;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.ObjectName = &mut KeyName;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenKey(
            &mut ControllerKey,
            KEY_ENUMERATE_SUB_KEYS,
            &mut ObjectAttributes,
        );
        if NT_SUCCESS!(Status) != 0 {
            ControllerInstance = 0;

            loop {
                RtlStringCchPrintfW!(
                    Buffer.as_mut_ptr(),
                    ARRAYSIZE!(Buffer),
                    RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
                    ControllerInstance,
                );
                RtlInitUnicodeString(&mut KeyName, Buffer.as_mut_ptr());
                ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
                ObjectAttributes.RootDirectory = ControllerKey;
                ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
                ObjectAttributes.ObjectName = &mut KeyName;
                ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                Status = NtOpenKey(
                    &mut ControllerInstanceKey,
                    KEY_QUERY_VALUE,
                    &mut ObjectAttributes,
                );
                if ((NT_SUCCESS!(Status)) == 0) {
                    DPRINT!("NtOpenKey() failed (Status %lx)\n", Status);
                    NtClose(ControllerKey);
                    NtClose(BusInstanceKey);
                    NtClose(BusKey);
                    return (FALSE) as u8;
                }

                RtlInitUnicodeString(&mut KeyName, RUSTCONV_WSTR_344F2CEBFA6E.as_ptr());

                BufferLength = core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG
                    + 256 * core::mem::size_of::<u16>() as ULONG;
                ValueInfo = RtlAllocateHeap(
                    RtlGetProcessHeap!(),
                    0,
                    (BufferLength) as SIZE_T,
                ) as PKEY_VALUE_PARTIAL_INFORMATION;
                if ValueInfo.is_null() {
                    DPRINT!("RtlAllocateHeap() failed\n");
                    NtClose(ControllerInstanceKey);
                    NtClose(ControllerKey);
                    NtClose(BusInstanceKey);
                    NtClose(BusKey);
                    return (FALSE) as u8;
                }

                Status = NtQueryValueKey(
                    ControllerInstanceKey,
                    &mut KeyName,
                    KeyValuePartialInformation,
                    ValueInfo as *mut core::ffi::c_void,
                    BufferLength,
                    &mut ReturnedLength,
                );
                if NT_SUCCESS!(Status) != 0 {
                    DPRINT!("Identifier: %S\n", (*ValueInfo).Data.as_ptr() as PWSTR);

                    BufferLength = min(
                        (*ValueInfo).DataLength / core::mem::size_of::<u16>() as ULONG,
                        IdentifierLength,
                    );
                    RtlCopyMemory!(
                        Identifier as *mut core::ffi::c_void,
                        (*ValueInfo).Data.as_ptr() as *const core::ffi::c_void,
                        BufferLength * core::mem::size_of::<u16>() as ULONG,
                    );
                    *Identifier.offset(BufferLength as isize) = 0;

                    RtlFreeHeap(
                        RtlGetProcessHeap!(),
                        0,
                        ValueInfo as *mut core::ffi::c_void,
                    );

                    NtClose(ControllerInstanceKey);
                    NtClose(ControllerKey);
                    NtClose(BusInstanceKey);
                    NtClose(BusKey);
                    return (TRUE) as u8;
                }

                NtClose(ControllerInstanceKey);

                ControllerInstance = ControllerInstance.wrapping_add(1);
            }

            NtClose(ControllerKey);
        }

        NtClose(BusInstanceKey);

        BusInstance = BusInstance.wrapping_add(1);
    }

    NtClose(BusKey);

    return (FALSE) as u8;
}

#[no_mangle]
pub unsafe extern "C" fn CreateDisplayDriverList(InfFile: HINF) -> PGENERIC_LIST {
    let mut List: PGENERIC_LIST = core::ptr::null_mut();
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut KeyName: PCWSTR = core::ptr::null();
    let mut KeyValue: PCWSTR = core::ptr::null();
    let mut DisplayIdentifier: [WCHAR; 128] = [0; 128];
    let mut DisplayKey: [WCHAR; 32] = [0; 32];

    if GetDisplayIdentifier(
        &raw mut DisplayIdentifier as PWSTR,
        128 as ULONG,
    ) == 0
    {
        DisplayIdentifier[0 as usize] = 0 as WCHAR;
    }

    DPRINT!(
        b"Display identifier: '%S'\n\0".as_ptr() as *const i8,
        &raw mut DisplayIdentifier as PWSTR,
    );

    if SpInfFindFirstLine!(
        InfFile,
        RUSTCONV_WSTR_5A6B5AE7F088.as_ptr(),
        core::ptr::null::<WCHAR>(),
        &raw mut Context,
    ) == 0
    {
        return core::ptr::null_mut();
    }

    loop {
        let mut FoundId: BOOLEAN = 0;

        if INF_GetDataField(
            &raw mut Context,
            1 as ULONG,
            &raw mut KeyValue,
        ) == 0
        {
            DPRINT!(b"INF_GetDataField() failed\n\0".as_ptr() as *const i8);
            return core::ptr::null_mut();
        }

        DPRINT!(b"KeyValue: %S\n\0".as_ptr() as *const i8, KeyValue);

        FoundId = !wcsstr(
            &raw mut DisplayIdentifier as *mut WCHAR,
            KeyValue,
        )
        .is_null() as BOOLEAN;

        INF_FreeData(KeyValue);

        if FoundId != 0 {
            if INF_GetDataField(
                &raw mut Context,
                0 as ULONG,
                &raw mut KeyName,
            ) == 0
            {
                DPRINT!(b"INF_GetDataField() failed\n\0".as_ptr() as *const i8);
                return core::ptr::null_mut();
            }

            DPRINT!(b"Display key: %S\n\0".as_ptr() as *const i8, KeyName);

            RtlStringCchCopyW(
                &raw mut DisplayKey as PWSTR,
                (ARRAYSIZE!(DisplayKey)) as _,
                KeyName,
            );

            INF_FreeData(KeyName);
        }

        if !(SpInfFindNextLine!(
            &raw mut Context,
            &raw mut Context,
        ) != 0)
        {
            break;
        }
    }

    List = CreateGenericList();

    if List.is_null() {
        return core::ptr::null_mut();
    }

    if AddEntriesFromInfSection(
        List,
        InfFile,
        RUSTCONV_WSTR_09A7C6FF8E65.as_ptr(),
        &raw mut Context,
        Some(DefaultProcessEntry),
        &raw mut DisplayKey as *mut WCHAR as PVOID,
    ) == -(1 as LONG)
    {
        DestroyGenericList(List, TRUE as BOOLEAN);
        return core::ptr::null_mut();
    }

    return List;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessComputerFiles(
    InfFile: HINF,
    ComputerType: PCWSTR,
    AdditionalSectionName: *mut PWSTR,
) -> BOOLEAN {
    static mut SectionName: [u16; 128] = [0; 128];

    DPRINT!(
        b"ProcessComputerFiles(%S) called\n\0".as_ptr() as *const i8,
        ComputerType,
    );

    RtlStringCchPrintfW!(
        SectionName.as_mut_ptr(),
        _countof!(SectionName),
        RUSTCONV_WSTR_272522956B9D.as_ptr(),
        ComputerType,
    );
    *AdditionalSectionName = SectionName.as_mut_ptr();

    // TODO: More things to do?

    return (TRUE) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessDisplayRegistry(
    InfFile: HINF,
    DisplayType: PCWSTR,
) -> BOOLEAN {
    let mut Status: NTSTATUS = 0;
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut Buffer: PCWSTR = core::ptr::null();
    let mut ServiceName: PCWSTR = core::ptr::null();
    let mut StartValue: ULONG = 0;
    let mut Width: ULONG = 0;
    let mut Height: ULONG = 0;
    let mut Bpp: ULONG = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut RegPath: [u16; 255] = [0; 255];

    DPRINT!("ProcessDisplayRegistry(%S) called\n", DisplayType);

    if ((SpInfFindFirstLine!(InfFile, RUSTCONV_WSTR_09A7C6FF8E65.as_ptr(), DisplayType, &mut Context)) == 0) {
        DPRINT1!("SpInfFindFirstLine!() failed\n");
        return (FALSE) as u8;
    }

    if ((INF_GetDataField(&mut Context, 3, &mut ServiceName)) == 0) {
        DPRINT1!("INF_GetDataField() failed\n");
        return (FALSE) as u8;
    }

    DPRINT!("Service name: '%S'\n", ServiceName);

    RtlStringCchPrintfW!(
        RegPath.as_mut_ptr(),
        ARRAYSIZE!(RegPath),
        RUSTCONV_WSTR_F625A5873448.as_ptr(),
        ServiceName,
    );
    RtlInitUnicodeString(&mut KeyName, RegPath.as_mut_ptr());
    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut KeyName,
        OBJ_CASE_INSENSITIVE,
        GetRootKeyByPredefKey((HKEY_LOCAL_MACHINE).cast(), core::ptr::null_mut()),
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(
        &mut KeyHandle,
        KEY_SET_VALUE,
        &mut ObjectAttributes,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtOpenKey() failed (Status %lx)\n", Status);
        return (FALSE) as u8;
    }

    StartValue = 1;
    Status = RtlWriteRegistryValue(
        RTL_REGISTRY_HANDLE,
        KeyHandle as PCWSTR,
        RUSTCONV_WSTR_FA8A27D23A1A.as_ptr(),
        REG_DWORD,
        &mut StartValue as *mut ULONG as PVOID,
        core::mem::size_of::<ULONG>() as u32,
    );
    NtClose(KeyHandle);
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("RtlWriteRegistryValue() failed (Status %lx)\n", Status);
        return (FALSE) as u8;
    }

    if ((INF_GetDataField(&mut Context, 4, &mut Buffer)) == 0) {
        DPRINT1!("INF_GetDataField() failed\n");
        return (FALSE) as u8;
    }

    RtlStringCchPrintfW!(
        RegPath.as_mut_ptr(),
        ARRAYSIZE!(RegPath),
        RUSTCONV_WSTR_E3329FFFD7A6.as_ptr(),
        ServiceName,
    );
    DPRINT!("RegPath: '%S'\n", RegPath.as_ptr());
    RtlInitUnicodeString(&mut KeyName, RegPath.as_mut_ptr());
    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut KeyName,
        OBJ_CASE_INSENSITIVE,
        GetRootKeyByPredefKey((HKEY_LOCAL_MACHINE).cast(), core::ptr::null_mut()),
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(
        &mut KeyHandle,
        KEY_SET_VALUE,
        &mut ObjectAttributes,
    );
    if Status == STATUS_OBJECT_NAME_NOT_FOUND {
        RtlStringCchPrintfW!(
            RegPath.as_mut_ptr(),
            ARRAYSIZE!(RegPath),
            RUSTCONV_WSTR_4BF3B3F38290.as_ptr(),
            ServiceName,
        );
        RtlInitUnicodeString(&mut KeyName, RegPath.as_mut_ptr());
        Status = NtOpenKey(
            &mut KeyHandle,
            KEY_SET_VALUE,
            &mut ObjectAttributes,
        );
    }
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtOpenKey() failed (Status %lx)\n", Status);
        return (FALSE) as u8;
    }

    Width = wcstoul(Buffer, core::ptr::null_mut(), 10);
    Status = RtlWriteRegistryValue(
        RTL_REGISTRY_HANDLE,
        KeyHandle as PCWSTR,
        RUSTCONV_WSTR_23C470F1D8A6.as_ptr(),
        REG_DWORD,
        &mut Width as *mut ULONG as PVOID,
        core::mem::size_of::<ULONG>() as u32,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("RtlWriteRegistryValue() failed (Status %lx)\n", Status);
        NtClose(KeyHandle);
        return (FALSE) as u8;
    }

    if ((INF_GetDataField(&mut Context, 5, &mut Buffer)) == 0) {
        DPRINT1!("INF_GetDataField() failed\n");
        NtClose(KeyHandle);
        return (FALSE) as u8;
    }

    Height = wcstoul(Buffer, core::ptr::null_mut(), 10);
    Status = RtlWriteRegistryValue(
        RTL_REGISTRY_HANDLE,
        KeyHandle as PCWSTR,
        RUSTCONV_WSTR_4CFDAEB88300.as_ptr(),
        REG_DWORD,
        &mut Height as *mut ULONG as PVOID,
        core::mem::size_of::<ULONG>() as u32,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("RtlWriteRegistryValue() failed (Status %lx)\n", Status);
        NtClose(KeyHandle);
        return (FALSE) as u8;
    }

    if ((INF_GetDataField(&mut Context, 6, &mut Buffer)) == 0) {
        DPRINT1!("INF_GetDataField() failed\n");
        NtClose(KeyHandle);
        return (FALSE) as u8;
    }

    Bpp = wcstoul(Buffer, core::ptr::null_mut(), 10);
    Status = RtlWriteRegistryValue(
        RTL_REGISTRY_HANDLE,
        KeyHandle as PCWSTR,
        RUSTCONV_WSTR_9A24410C9730.as_ptr(),
        REG_DWORD,
        &mut Bpp as *mut ULONG as PVOID,
        core::mem::size_of::<ULONG>() as u32,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("RtlWriteRegistryValue() failed (Status %lx)\n", Status);
        NtClose(KeyHandle);
        return (FALSE) as u8;
    }

    NtClose(KeyHandle);

    DPRINT!("ProcessDisplayRegistry() done\n");

    return (TRUE) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessLocaleRegistry(mut LanguageId: PCWSTR) -> i32 {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Status: i32 = 0;

    RtlInitUnicodeString(&mut KeyName, RUSTCONV_WSTR_A37A558548DD.as_ptr());

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = GetRootKeyByPredefKey((HKEY_USERS).cast(), core::ptr::null_mut());
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(&mut KeyHandle, KEY_SET_VALUE, &mut ObjectAttributes);
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as i32;
    }

    RtlInitUnicodeString(&mut ValueName, RUSTCONV_WSTR_FF3CF31B44D6.as_ptr());
    Status = NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_SZ,
        LanguageId as *mut core::ffi::c_void,
        ((wcslen(LanguageId) + 1) * core::mem::size_of::<u16>()) as ULONG,
    );
    NtClose(KeyHandle);
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as i32;
    }

    if wcslen(LanguageId) >= 4 {
        LanguageId = LanguageId.add(4 as usize);
    }

    RtlInitUnicodeString(&mut KeyName, RUSTCONV_WSTR_EAE8EEC66C20.as_ptr());

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = GetRootKeyByPredefKey((HKEY_LOCAL_MACHINE).cast(), core::ptr::null_mut());
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(&mut KeyHandle, KEY_SET_VALUE, &mut ObjectAttributes);
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as i32;
    }

    RtlInitUnicodeString(&mut ValueName, RUSTCONV_WSTR_39CDE5D97C33.as_ptr());
    Status = NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_SZ,
        LanguageId as *mut core::ffi::c_void,
        ((wcslen(LanguageId) + 1) * core::mem::size_of::<u16>()) as ULONG,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        NtClose(KeyHandle);
        return (FALSE) as i32;
    }

    RtlInitUnicodeString(&mut ValueName, RUSTCONV_WSTR_8EA4B3C5C5D4.as_ptr());
    Status = NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_SZ,
        LanguageId as *mut core::ffi::c_void,
        ((wcslen(LanguageId) + 1) * core::mem::size_of::<u16>()) as ULONG,
    );
    NtClose(KeyHandle);
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as i32;
    }

    return (TRUE) as i32;
}

#[no_mangle]
pub unsafe extern "C" fn CreateKeyboardDriverList(InfFile: HINF) -> PGENERIC_LIST {
    let mut List: PGENERIC_LIST;
    let mut Context: INFCONTEXT = core::mem::zeroed();

    List = CreateGenericList();
    if List.is_null() {
        return core::ptr::null_mut();
    }

    if AddEntriesFromInfSection(
        List,
        InfFile,
        RUSTCONV_WSTR_EF20E0741A09.as_ptr(),
        &mut Context,
        Some(DefaultProcessEntry),
        core::ptr::null_mut(),
    ) == -1
    {
        DestroyGenericList(List, (TRUE) as BOOLEAN);
        return core::ptr::null_mut();
    }

    return List;
}

#[no_mangle]
pub unsafe extern "C" fn GetDefaultLanguageIndex() -> u32 {
    return DefaultLanguageIndex;
}
unsafe extern "system" fn ProcessLangEntry(
    mut KeyName: PCWSTR,
    mut KeyValue: PCWSTR,
    mut UserData: *mut PVOID,
    mut Current: PBOOLEAN,
    mut Parameter: PVOID,
) -> UCHAR {
    let mut LangEntryParam: PLANG_ENTRY_PARAM = Parameter as PLANG_ENTRY_PARAM;
    let mut GenEntry: PGENENTRY = ::core::ptr::null_mut::<_GENENTRY>();
    let mut IdSize: SIZE_T = 0;
    let mut ValueSize: SIZE_T = 0;
    if IsLanguageAvailable(KeyName) == 0 {
        return 2;
    }
    IdSize = (wcslen(KeyName as *const wchar_t) as usize)
        .wrapping_add(1)
        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as SIZE_T;
    ValueSize = (wcslen(KeyValue as *const wchar_t) as usize)
        .wrapping_add(1)
        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as SIZE_T;
    GenEntry = RtlAllocateHeap(
        ProcessHeap as PVOID,
        0,
        (::core::mem::size_of::<_GENENTRY>() as SIZE_T)
            .wrapping_add(IdSize)
            .wrapping_add(ValueSize),
    ) as PGENENTRY;
    if GenEntry.is_null() {
        DPRINT1!(b"RtlAllocateHeap() failed\n\0" as *const u8 as PCSTR);
        return 0;
    }
    (*GenEntry).Id =
        (GenEntry as usize).wrapping_add(::core::mem::size_of::<_GENENTRY>() as usize) as PCWSTR;
    (*GenEntry).Value = (GenEntry as usize)
        .wrapping_add(::core::mem::size_of::<_GENENTRY>() as usize)
        .wrapping_add(IdSize as usize) as PCWSTR;
    RtlStringCbCopyW(
        (*GenEntry).Id as NTSTRSAFE_PWSTR,
        IdSize as size_t,
        KeyName as NTSTRSAFE_PCWSTR,
    );
    RtlStringCbCopyW(
        (*GenEntry).Value as NTSTRSAFE_PWSTR,
        ValueSize as size_t,
        KeyValue as NTSTRSAFE_PCWSTR,
    );
    *UserData = GenEntry as PVOID;
    *Current = (FALSE) as u8;
    if _wcsicmp(
        KeyName as *const wchar_t,
        (*LangEntryParam).DefaultLanguage as *const wchar_t,
    ) == 0
    {
        DefaultLanguageIndex = (*LangEntryParam).uIndex;
    }
    (*LangEntryParam).uIndex = (*LangEntryParam).uIndex.wrapping_add(1);
    return 1;
}

#[no_mangle]
pub unsafe extern "C" fn CreateLanguageList(
    InfFile: HINF,
    DefaultLanguage: PWSTR,
) -> PGENERIC_LIST {
    let mut List: PGENERIC_LIST = core::ptr::null_mut();
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut KeyValue: PCWSTR = core::ptr::null();
    let mut LangEntryParam: LANG_ENTRY_PARAM = core::mem::zeroed();

    LangEntryParam.uIndex = 0;
    LangEntryParam.DefaultLanguage = DefaultLanguage;

    if ((SpInfFindFirstLine!(InfFile, RUSTCONV_WSTR_4C2425E1D768.as_ptr(), RUSTCONV_WSTR_ED2F5E1AB2BB.as_ptr(), &mut Context)) == 0) {
        return core::ptr::null_mut();
    }

    if ((INF_GetData(&mut Context, core::ptr::null_mut(), &mut KeyValue)) == 0) {
        return core::ptr::null_mut();
    }

    wcscpy(DefaultLanguage, KeyValue);

    List = CreateGenericList();
    if List.is_null() {
        return core::ptr::null_mut();
    }

    if AddEntriesFromInfSection(
        List,
        InfFile,
        RUSTCONV_WSTR_A62088E6F2CF.as_ptr(),
        &mut Context,
        Some(ProcessLangEntry),
        &mut LangEntryParam as *mut LANG_ENTRY_PARAM as PVOID,
    ) == -1
    {
        DestroyGenericList(List, (TRUE) as BOOLEAN);
        return core::ptr::null_mut();
    }

    if LangEntryParam.uIndex == 1 {
        DefaultLanguageIndex = 0;
        wcscpy(
            DefaultLanguage,
            (*(GetListEntryData(GetFirstListEntry(List)) as *mut GENENTRY)).Id,
        );
    }

    List
}

#[no_mangle]
pub unsafe extern "C" fn CreateKeyboardLayoutList(
    InfFile: HINF,
    LanguageId: PCWSTR,
    DefaultKBLayout: PWSTR,
) -> PGENERIC_LIST {
    let mut List: PGENERIC_LIST;
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut KeyValue: PCWSTR = core::mem::zeroed();
    let LayoutsList: *const MUI_LAYOUTS;
    let mut uIndex: ULONG = 0;

    if ((SpInfFindFirstLine!(InfFile, RUSTCONV_WSTR_4C2425E1D768.as_ptr(), RUSTCONV_WSTR_4220FDAA5966.as_ptr(), &mut Context)) == 0) {
        return core::ptr::null_mut();
    }

    if ((INF_GetData(&mut Context, core::ptr::null_mut(), &mut KeyValue)) == 0) {
        return core::ptr::null_mut();
    }

    wcscpy(DefaultKBLayout, KeyValue);

    List = CreateGenericList();
    if List.is_null() {
        return core::ptr::null_mut();
    }

    LayoutsList = MUIGetLayoutsList(LanguageId);

    loop {
        if AddEntriesFromInfSection(
            List,
            InfFile,
            RUSTCONV_WSTR_01F99237FAD1.as_ptr(),
            &mut Context,
            Some(DefaultProcessEntry),
            DefaultKBLayout as PVOID,
        ) == -1
        {
            DestroyGenericList(List, (TRUE) as BOOLEAN);
            return core::ptr::null_mut();
        }

        uIndex = uIndex.wrapping_add(1);

        if (*LayoutsList.offset(uIndex as isize)).LangID == 0 {
            break;
        }
    }

    if GetNumberOfListEntries(List) == 0 {
        DPRINT1!("No keyboard layouts have been found\n");
        DestroyGenericList(List, (TRUE) as BOOLEAN);
        return core::ptr::null_mut();
    }

    List
}
#[no_mangle]
pub unsafe extern "C" fn ProcessKeyboardLayoutRegistry(
    pszLayoutId: PCWSTR,
    LanguageId: PCWSTR,
) -> i32 {
    let mut LayoutId: u32;
    let LayoutsList: *const MUI_LAYOUTS;
    let mut NewLayoutsList: [MUI_LAYOUTS; 20] = core::mem::zeroed();
    let mut uIndex: u32;
    let mut uOldPos: u32 = 0;

    LayoutId = if !pszLayoutId.is_null() {
        wcstoul(pszLayoutId, core::ptr::null_mut(), 16)
    } else {
        0
    };
    if LayoutId == 0 {
        return (FALSE) as i32;
    }

    LayoutsList = MUIGetLayoutsList(LanguageId);

    if (*LayoutsList.offset(0 as isize)).LayoutID == LayoutId {
        return (TRUE) as i32;
    }

    uIndex = 1;
    while (*LayoutsList.offset(uIndex as isize)).LangID != 0 {
        if (*LayoutsList.offset(uIndex as isize)).LayoutID == LayoutId {
            uOldPos = uIndex;
        } else {
            NewLayoutsList[uIndex as usize].LangID = (*LayoutsList.offset(uIndex as isize)).LangID;
            NewLayoutsList[uIndex as usize].LayoutID = (*LayoutsList.offset(uIndex as isize)).LayoutID;
        }
        uIndex = uIndex.wrapping_add(1);
    }

    NewLayoutsList[uIndex as usize].LangID = 0;
    NewLayoutsList[uIndex as usize].LayoutID = 0;
    NewLayoutsList[uOldPos as usize].LangID = (*LayoutsList.offset(0 as isize)).LangID;
    NewLayoutsList[uOldPos as usize].LayoutID = (*LayoutsList.offset(0 as isize)).LayoutID;
    NewLayoutsList[0 as usize].LangID = (*LayoutsList.offset(uOldPos as isize)).LangID;
    NewLayoutsList[0 as usize].LayoutID = (*LayoutsList.offset(uOldPos as isize)).LayoutID;

    return (AddKbLayoutsToRegistry(NewLayoutsList.as_mut_ptr())) as _;
}
#[no_mangle]
pub unsafe extern "C" fn SetGeoID(GeoId: GEOID) -> BOOLEAN {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::mem::zeroed();
    let mut Value: [WCHAR; 11] = [0; 11];

    Status = RtlStringCchPrintfW!(
        Value.as_mut_ptr(),
        _countof!(Value),
        RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
        GeoId,
    );

    RtlInitUnicodeString(&mut Name, RUSTCONV_WSTR_64E8694B8B78.as_ptr());
    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut Name,
        OBJ_CASE_INSENSITIVE,
        GetRootKeyByPredefKey((HKEY_USERS).cast(), core::ptr::null_mut()),
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(
        &mut KeyHandle,
        KEY_SET_VALUE,
        &mut ObjectAttributes,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtOpenKey() failed (Status %lx)\n", Status);
        return (FALSE) as u8;
    }

    RtlInitUnicodeString(&mut Name, RUSTCONV_WSTR_92C451776EF8.as_ptr());
    Status = NtSetValueKey(
        KeyHandle,
        &mut Name,
        0,
        REG_SZ,
        Value.as_mut_ptr() as PVOID,
        ((wcslen(Value.as_mut_ptr()) + 1)
            * core::mem::size_of::<WCHAR>()) as ULONG,
    );
    NtClose(KeyHandle);
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtSetValueKey() failed (Status %lx)\n", Status);
        return (FALSE) as u8;
    }

    return (TRUE) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn SetDefaultPagefile(Drive: u16) -> i32 {
    let mut Status: i32;
    let mut KeyHandle: *mut core::ffi::c_void = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 67]>() - core::mem::size_of::<u16>()) as u16,
        MaximumLength: core::mem::size_of::<[u16; 67]>() as u16,
        Buffer: RUSTCONV_WSTR_C5FEC8B2F1E4.as_ptr() as *mut u16,
    };
    let mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 12]>() - core::mem::size_of::<u16>()) as u16,
        MaximumLength: core::mem::size_of::<[u16; 12]>() as u16,
        Buffer: RUSTCONV_WSTR_86E5823B1278.as_ptr() as *mut u16,
    };
    let mut ValueBuffer: [u16; 21] = [
    0x003Fu16, 0x003A, 0x005C, 0x0070, 0x0061, 0x0067, 0x0065, 0x0066, 0x0069, 0x006C,
    0x0065, 0x002E, 0x0073, 0x0079, 0x0073, 0x0020, 0x0030, 0x0020, 0x0030, 0x0000,
    0x0000,
];

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = GetRootKeyByPredefKey((HKEY_LOCAL_MACHINE).cast(), core::ptr::null_mut());
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(
        &mut KeyHandle,
        KEY_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as i32;
    }

    ValueBuffer[0] = Drive;

    NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_MULTI_SZ,
        &mut ValueBuffer as *mut [u16; 21] as *mut core::ffi::c_void,
        core::mem::size_of::<[u16; 21]>() as u32,
    );

    NtClose(KeyHandle);
    return (TRUE) as i32;
}
