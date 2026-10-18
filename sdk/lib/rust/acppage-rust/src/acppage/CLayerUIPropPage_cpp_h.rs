use crate::prelude::*;
use crate::acppage::CEditCompatModes_cpp_h::ShowEditCompatModes;
use crate::acppage::ACPPage_cpp_h::g_hModule;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_01518F84F715: [WCHAR; 46] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x0050, 0x006F, 0x006C, 0x0069, 0x0063, 0x0069, 0x0065, 0x0073, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0041, 0x0070, 0x0070, 0x0043, 0x006F, 0x006D, 0x0070, 0x0061, 0x0074, 0x0000];
pub static CPP_COM_WSTR_07A29117B682: [WCHAR; 9] = [0x005B, 0x0025, 0x002D, 0x0032, 0x0030, 0x0053, 0x005D, 0x0020, 0x0000];
pub static CPP_COM_WSTR_0EB4CDB484FE: [WCHAR; 22] = [0x0049, 0x0067, 0x006E, 0x006F, 0x0072, 0x0069, 0x006E, 0x0067, 0x0020, 0x0057, 0x0069, 0x006E, 0x0053, 0x0078, 0x0073, 0x003A, 0x0020, 0x0025, 0x0073, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_133F4E666837: [WCHAR; 24] = [0x0045, 0x0078, 0x0070, 0x0061, 0x006E, 0x0064, 0x0065, 0x0064, 0x0020, 0x0027, 0x0025, 0x0073, 0x0027, 0x0020, 0x003D, 0x003E, 0x0020, 0x0027, 0x0025, 0x0073, 0x0027, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_13B80F0791ED: [WCHAR; 24] = [0x0049, 0x0067, 0x006E, 0x006F, 0x0072, 0x0069, 0x006E, 0x0067, 0x0020, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x003A, 0x0020, 0x0025, 0x0073, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_21C321DDA9DE: [WCHAR; 41] = [0x0046, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020, 0x0072, 0x0065, 0x0074, 0x0072, 0x0069, 0x0065, 0x0076, 0x0065, 0x0020, 0x0044, 0x0061, 0x0074, 0x0061, 0x0020, 0x0066, 0x0072, 0x006F, 0x006D, 0x0020, 0x0070, 0x0044, 0x0061, 0x0074, 0x0061, 0x004F, 0x0062, 0x006A, 0x002E, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_2E892533F5BA: [WCHAR; 32] = [0x0041, 0x006C, 0x006C, 0x006F, 0x0077, 0x0050, 0x0065, 0x0072, 0x006D, 0x004C, 0x0061, 0x0079, 0x0065, 0x0072, 0x0020, 0x0072, 0x0065, 0x0074, 0x0075, 0x0072, 0x006E, 0x0065, 0x0064, 0x0020, 0x0046, 0x0041, 0x004C, 0x0053, 0x0045, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_35C1689B190B: [WCHAR; 30] = [0x0049, 0x006E, 0x0076, 0x0061, 0x006C, 0x0069, 0x0064, 0x0020, 0x006E, 0x0075, 0x006D, 0x0062, 0x0065, 0x0072, 0x0020, 0x006F, 0x0066, 0x0020, 0x0066, 0x0069, 0x006C, 0x0065, 0x0073, 0x003A, 0x0020, 0x0025, 0x0064, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_4449408319CF: [WCHAR; 31] = [0x0045, 0x0078, 0x0074, 0x0065, 0x006E, 0x0073, 0x0069, 0x006F, 0x006E, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020, 0x0069, 0x006E, 0x0063, 0x006C, 0x0075, 0x0064, 0x0065, 0x0064, 0x003A, 0x0020, 0x0027, 0x0025, 0x0073, 0x0027, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_4A36961D9B2E: [WCHAR; 29] = [0x0043, 0x006F, 0x0075, 0x006C, 0x0064, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020, 0x006C, 0x006F, 0x0063, 0x006B, 0x0020, 0x0073, 0x0074, 0x0067, 0x002E, 0x0068, 0x0047, 0x006C, 0x006F, 0x0062, 0x0061, 0x006C, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_57B9731B469C: [WCHAR; 8] = [0x005C, 0x0057, 0x0069, 0x006E, 0x0053, 0x0078, 0x0073, 0x0000];
pub static CPP_COM_WSTR_5DBA9F26C9D0: [WCHAR; 24] = [0x0046, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020, 0x0065, 0x0078, 0x0070, 0x0061, 0x006E, 0x0064, 0x0020, 0x0027, 0x0025, 0x0073, 0x0027, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_640DE941CE91: [WCHAR; 44] = [0x0068, 0x0074, 0x0074, 0x0070, 0x0073, 0x003A, 0x002F, 0x002F, 0x0072, 0x0065, 0x0061, 0x0063, 0x0074, 0x006F, 0x0073, 0x002E, 0x006F, 0x0072, 0x0067, 0x002F, 0x0066, 0x006F, 0x0072, 0x0075, 0x006D, 0x002F, 0x0076, 0x0069, 0x0065, 0x0077, 0x0066, 0x006F, 0x0072, 0x0075, 0x006D, 0x002E, 0x0070, 0x0068, 0x0070, 0x003F, 0x0066, 0x003D, 0x0034, 0x0000];
pub static CPP_COM_WSTR_6785322A4096: [WCHAR; 10] = [0x005C, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x0000];
pub static CPP_COM_WSTR_6FECEA5B5508: [WCHAR; 33] = [0x004C, 0x0069, 0x006E, 0x006B, 0x0020, 0x0072, 0x0065, 0x0064, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x0073, 0x0020, 0x0074, 0x006F, 0x0020, 0x0069, 0x0074, 0x0073, 0x0065, 0x006C, 0x0066, 0x003A, 0x0020, 0x0027, 0x0025, 0x0073, 0x0027, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_7721435B36A0: [WCHAR; 20] = [0x0050, 0x0072, 0x006F, 0x0074, 0x0065, 0x0063, 0x0074, 0x0065, 0x0064, 0x0020, 0x004F, 0x0053, 0x0020, 0x0066, 0x0069, 0x006C, 0x0065, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_82294FD79A0D: [WCHAR; 3] = [0x002C, 0x0020, 0x0000];
pub static CPP_COM_WSTR_82DB442E7A60: [WCHAR; 5] = [0x006F, 0x0070, 0x0065, 0x006E, 0x0000];
pub static CPP_COM_WSTR_8D71B3FAAB82: [WCHAR; 2] = [0x0020, 0x0000];
pub static CPP_COM_WSTR_A8879050313A: [WCHAR; 31] = [0x0046, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0064, 0x0075, 0x0072, 0x0069, 0x006E, 0x0067, 0x0020, 0x0065, 0x0078, 0x0070, 0x0061, 0x006E, 0x0073, 0x0069, 0x006F, 0x006E, 0x0020, 0x0027, 0x0025, 0x0073, 0x0027, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_ACCB067678BF: [WCHAR; 40] = [0x0046, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x006C, 0x0069, 0x006E, 0x006B, 0x0020, 0x0074, 0x0061, 0x0072, 0x0067, 0x0065, 0x0074, 0x0020, 0x0066, 0x0072, 0x006F, 0x006D, 0x003A, 0x0020, 0x0027, 0x0025, 0x0073, 0x0027, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_B74DE3D9527D: [WCHAR; 10] = [0x0047, 0x006F, 0x0074, 0x003A, 0x0020, 0x0025, 0x0073, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_DCD52D52D7F4: [WCHAR; 36] = [0x0046, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020, 0x0066, 0x0069, 0x006E, 0x0064, 0x0020, 0x0061, 0x006E, 0x0020, 0x0065, 0x0078, 0x0074, 0x0065, 0x006E, 0x0073, 0x0069, 0x006F, 0x006E, 0x003A, 0x0020, 0x0027, 0x0025, 0x0073, 0x0027, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_E2ED9821845A: [WCHAR; 28] = [0x0046, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020, 0x0071, 0x0075, 0x0065, 0x0072, 0x0079, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0066, 0x0069, 0x006C, 0x0065, 0x002E, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_EE9140933D4C: [WCHAR; 14] = [0x0044, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0045, 0x006E, 0x0067, 0x0069, 0x006E, 0x0065, 0x0000];
pub static CPP_COM_WSTR_EFAFC0F54D9B: [WCHAR; 16] = [0x0044, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0050, 0x0072, 0x006F, 0x0070, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000];
pub static CPP_COM_WSTR_FD1B96BF0940: [WCHAR; 11] = [0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F, 0x0074, 0x0000];
pub static CPP_COM_WSTR_FE706C396A0C: [WCHAR; 5] = [0x002E, 0x006C, 0x006E, 0x006B, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static CLSID_CLayerUIPropPage: GUID = GUID {
    Data1: 0x513D916F,
    Data2: 0x2A8E,
    Data3: 0x4F51,
    Data4: [0xAE, 0xAB, 0x0C, 0xBC, 0x76, 0xFB, 0x1A, 0xF8],
};

#[repr(C)]
// The C++'s anonymous `struct { const PCWSTR Display; const PCWSTR Name; }`, by its field names.
pub struct CompatModeEntry {
    pub Display: PCWSTR,
    pub Name: PCWSTR,
}
pub type PCompatModeEntry = *mut CompatModeEntry;
unsafe impl Sync for CompatModeEntry {}

#[repr(C)]
// `struct { const PCWSTR Name; DWORD Id; }`.
pub struct LayerEntry {
    pub Name: PCWSTR,
    pub Id: DWORD,
}
pub type PLayerEntry = *mut LayerEntry;
unsafe impl Sync for LayerEntry {}

pub static g_CM0_name: [u16; 11] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, '9' as u16, '5' as u16, 0];
pub static g_CM0_layer: [u16; 6] = ['W' as u16, 'I' as u16, 'N' as u16, '9' as u16, '5' as u16, 0];
pub static g_CM1_name: [u16; 14] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, '9' as u16, '8' as u16, '/' as u16, 'M' as u16, 'E' as u16, 0];
pub static g_CM1_layer: [u16; 6] = ['W' as u16, 'I' as u16, 'N' as u16, '9' as u16, '8' as u16, 0];
pub static g_CM2_name: [u16; 21] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, 'N' as u16, 'T' as u16, ' ' as u16, '4' as u16, '.' as u16, '0' as u16, ' ' as u16, '(' as u16, 'S' as u16, 'P' as u16, '5' as u16, ')' as u16, 0];
pub static g_CM2_layer: [u16; 7] = ['N' as u16, 'T' as u16, '4' as u16, 'S' as u16, 'P' as u16, '5' as u16, 0];
pub static g_CM3_name: [u16; 13] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, '2' as u16, '0' as u16, '0' as u16, '0' as u16, 0];
pub static g_CM3_layer: [u16; 8] = ['W' as u16, 'I' as u16, 'N' as u16, '2' as u16, '0' as u16, '0' as u16, '0' as u16, 0];
pub static g_CM4_name: [u16; 17] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, 'X' as u16, 'P' as u16, ' ' as u16, '(' as u16, 'S' as u16, 'P' as u16, '3' as u16, ')' as u16, 0];
pub static g_CM4_layer: [u16; 9] = ['W' as u16, 'I' as u16, 'N' as u16, 'X' as u16, 'P' as u16, 'S' as u16, 'P' as u16, '3' as u16, 0];
pub static g_CM5_name: [u16; 26] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, ' ' as u16, '2' as u16, '0' as u16, '0' as u16, '3' as u16, ' ' as u16, '(' as u16, 'S' as u16, 'P' as u16, '1' as u16, ')' as u16, 0];
pub static g_CM5_layer: [u16; 12] = ['W' as u16, 'I' as u16, 'N' as u16, 'S' as u16, 'R' as u16, 'V' as u16, '0' as u16, '3' as u16, 'S' as u16, 'P' as u16, '1' as u16, 0];
pub static g_CM6_name: [u16; 26] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, ' ' as u16, '2' as u16, '0' as u16, '0' as u16, '8' as u16, ' ' as u16, '(' as u16, 'S' as u16, 'P' as u16, '1' as u16, ')' as u16, 0];
pub static g_CM6_layer: [u16; 12] = ['W' as u16, 'I' as u16, 'N' as u16, 'S' as u16, 'R' as u16, 'V' as u16, '0' as u16, '8' as u16, 'S' as u16, 'P' as u16, '1' as u16, 0];
pub static g_CM7_name: [u16; 20] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, 'V' as u16, 'i' as u16, 's' as u16, 't' as u16, 'a' as u16, ' ' as u16, '(' as u16, 'S' as u16, 'P' as u16, '2' as u16, ')' as u16, 0];
pub static g_CM7_layer: [u16; 9] = ['V' as u16, 'I' as u16, 'S' as u16, 'T' as u16, 'A' as u16, 'S' as u16, 'P' as u16, '2' as u16, 0];
pub static g_CM8_name: [u16; 10] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, '7' as u16, 0];
pub static g_CM8_layer: [u16; 8] = ['W' as u16, 'I' as u16, 'N' as u16, '7' as u16, 'R' as u16, 'T' as u16, 'M' as u16, 0];
pub static g_CM9_name: [u16; 16] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, '7' as u16, ' ' as u16, '(' as u16, 'S' as u16, 'P' as u16, '1' as u16, ')' as u16, 0];
pub static g_CM9_layer: [u16; 8] = ['W' as u16, 'I' as u16, 'N' as u16, '7' as u16, 'S' as u16, 'P' as u16, '1' as u16, 0];
pub static g_CM10_name: [u16; 12] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, '8' as u16, '.' as u16, '1' as u16, 0];
pub static g_CM10_layer: [u16; 9] = ['W' as u16, 'I' as u16, 'N' as u16, '8' as u16, '1' as u16, 'R' as u16, 'T' as u16, 'M' as u16, 0];
pub static g_CM11_name: [u16; 11] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, '1' as u16, '0' as u16, 0];
pub static g_CM11_layer: [u16; 9] = ['W' as u16, 'I' as u16, 'N' as u16, '1' as u16, '0' as u16, 'R' as u16, 'T' as u16, 'M' as u16, 0];
pub static g_CM12_name: [u16; 20] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, ' ' as u16, '2' as u16, '0' as u16, '1' as u16, '6' as u16, 0];
pub static g_CM12_layer: [u16; 12] = ['W' as u16, 'I' as u16, 'N' as u16, 'S' as u16, 'R' as u16, 'V' as u16, '1' as u16, '6' as u16, 'R' as u16, 'T' as u16, 'M' as u16, 0];
pub static g_CM13_name: [u16; 20] = ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, ' ' as u16, '2' as u16, '0' as u16, '1' as u16, '9' as u16, 0];
pub static g_CM13_layer: [u16; 12] = ['W' as u16, 'I' as u16, 'N' as u16, 'S' as u16, 'R' as u16, 'V' as u16, '1' as u16, '9' as u16, 'R' as u16, 'T' as u16, 'M' as u16, 0];

pub static g_CompatModes: [CompatModeEntry; 15] = [
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM0_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM0_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM1_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM1_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM2_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM2_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM3_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM3_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM4_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM4_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM5_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM5_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM6_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM6_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM7_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM7_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM8_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM8_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM9_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM9_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM10_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM10_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM11_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM11_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM12_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM12_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::addr_of!(g_CM13_name) as PCWSTR, Name: core::ptr::addr_of!(g_CM13_layer) as PCWSTR },
    CompatModeEntry { Display: core::ptr::null(), Name: core::ptr::null() },
];

pub static g_LayerName0: [u16; 9] = ['2' as u16, '5' as u16, '6' as u16, 'C' as u16, 'O' as u16, 'L' as u16, 'O' as u16, 'R' as u16, 0];
pub static g_LayerName1: [u16; 8] = ['6' as u16, '4' as u16, '0' as u16, 'X' as u16, '4' as u16, '8' as u16, '0' as u16, 0];
pub static g_LayerName2: [u16; 14] = ['D' as u16, 'I' as u16, 'S' as u16, 'A' as u16, 'B' as u16, 'L' as u16, 'E' as u16, 'T' as u16, 'H' as u16, 'E' as u16, 'M' as u16, 'E' as u16, 'S' as u16, 0];

pub static g_Layers: [LayerEntry; 4] = [
    LayerEntry { Name: core::ptr::addr_of!(g_LayerName0) as PCWSTR, Id: IDC_CHKRUNIN256COLORS as u32 },
    LayerEntry { Name: core::ptr::addr_of!(g_LayerName1) as PCWSTR, Id: IDC_CHKRUNIN640480RES as u32 },
    LayerEntry { Name: core::ptr::addr_of!(g_LayerName2) as PCWSTR, Id: IDC_CHKDISABLEVISUALTHEMES as u32 },
    LayerEntry { Name: core::ptr::null(), Id: 0 },
];

pub static g_ExtExe: [u16; 5] = ['.' as u16, 'e' as u16, 'x' as u16, 'e' as u16, 0];
pub static g_ExtMsi: [u16; 5] = ['.' as u16, 'm' as u16, 's' as u16, 'i' as u16, 0];
pub static g_ExtPif: [u16; 5] = ['.' as u16, 'p' as u16, 'i' as u16, 'f' as u16, 0];
pub static g_ExtBat: [u16; 5] = ['.' as u16, 'b' as u16, 'a' as u16, 't' as u16, 0];
pub static g_ExtCmd: [u16; 5] = ['.' as u16, 'c' as u16, 'm' as u16, 'd' as u16, 0];

// Mutable only so the raw-pointer table can be a plain static without a Sync wrapper.
pub static mut g_AllowedExtensions: [PCWSTR; 6] = [
    core::ptr::addr_of!(g_ExtExe) as PCWSTR,
    core::ptr::addr_of!(g_ExtMsi) as PCWSTR,
    core::ptr::addr_of!(g_ExtPif) as PCWSTR,
    core::ptr::addr_of!(g_ExtBat) as PCWSTR,
    core::ptr::addr_of!(g_ExtCmd) as PCWSTR,
    core::ptr::null(),
];













#[no_mangle]
pub unsafe extern "C" fn IsBuiltinLayer(Name: PCWSTR) -> i32 {
    let mut n: usize;
    n = 0;
    loop {
        if g_Layers[n].Name.is_null() {
            break;
        }
        if _wcsicmp(g_Layers[n].Name, Name) == 0 {
            return (TRUE as i32);
        }
        n += 1;
    }

    n = 0;
    loop {
        if g_CompatModes[n].Name.is_null() {
            break;
        }
        if _wcsicmp(g_CompatModes[n].Name, Name) == 0 {
            return (TRUE as i32);
        }
        n += 1;
    }
    return (FALSE as i32);
}
#[no_mangle]
pub unsafe extern "C" fn ACDBG_FN_v(FunctionName: PCSTR, Format: PCWSTR, ap_list: *mut ::core::ffi::c_void) {
    let mut Buffer: [u16; 512] = [0u16; 512];
    let mut Current: *mut u16 = Buffer.as_mut_ptr();
    let mut Length: usize = (_countof!(Buffer)) as usize;

    StringCchPrintfExW!(Current, Length, core::ptr::addr_of_mut!(Current), core::ptr::addr_of_mut!(Length), STRSAFE_NULL_ON_FAILURE, CPP_COM_WSTR_07A29117B682.as_ptr(), FunctionName);
    let mut ArgList: va_list = *(ap_list as *mut va_list);

    StringCchVPrintfExW(Current, Length, core::ptr::addr_of_mut!(Current), core::ptr::addr_of_mut!(Length), STRSAFE_NULL_ON_FAILURE, Format, ArgList as *mut c_void);

    OutputDebugStringW(Buffer.as_ptr());
}



unsafe extern "C" fn GetLayerInfo(Filename: PCWSTR, QueryFlags: DWORD, OSMode: PDWORD, Enabledlayers: PDWORD, customLayers: &mut CSimpleArray<CStringW>) -> BOOL {
    let mut wszLayers: [WCHAR; (MAX_LAYER_LENGTH) as _] = [0; (MAX_LAYER_LENGTH) as _];
    let mut dwBytes: DWORD = core::mem::size_of::<[WCHAR; (MAX_LAYER_LENGTH) as _]>() as DWORD;

    *Enabledlayers = 0;
    *OSMode = *Enabledlayers;
    customLayers.RemoveAll();
    if SdbGetPermLayerKeys(Filename, wszLayers.as_mut_ptr(), core::ptr::addr_of_mut!(dwBytes), QueryFlags) == 0 {
        return (FALSE) as BOOL;
    }

    let mut Layer: PWCHAR = wcstok(wszLayers.as_mut_ptr(), CPP_COM_WSTR_8D71B3FAAB82.as_ptr() as PWCHAR);
    while !Layer.is_null() {
        'body: {
            let mut n: usize;
            n = 0;
            while !g_Layers[n].Name.is_null() {
                if _wcsicmp(g_Layers[n].Name, Layer) == 0 {
                    *Enabledlayers |= 1 << n;
                    break;
                }
                n += 1;
            }
            /* Did we find it? */
            if !g_Layers[n].Name.is_null() {
                break 'body;
            }

            n = 0;
            while !g_CompatModes[n].Name.is_null() {
                if _wcsicmp(g_CompatModes[n].Name, Layer) == 0 {
                    *OSMode = (n + 1) as DWORD;
                    break;
                }
                n += 1;
            }
            /* Did we find it? */
            if !g_CompatModes[n].Name.is_null() {
                break 'body;
            }

            /* Must be a 'custom' layer */
            customLayers.Add(CStringW::from_ptr(Layer as *const WCHAR));
        }
        Layer = wcstok(core::ptr::null_mut(), CPP_COM_WSTR_8D71B3FAAB82.as_ptr() as PWCHAR);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn ArrayEquals(lhs: *const CSimpleArray<CStringW>, rhs: *const CSimpleArray<CStringW>) -> BOOL {
    if (*lhs).GetSize() != (*rhs).GetSize() {
        return (FALSE) as BOOL;
    }

    let mut n: i32 = 0;
    while n < (*lhs).GetSize() {
        if (*(*lhs).GetAt(n)).Compare((*(*rhs).GetAt(n)).GetString()) != 0 {
            return (FALSE) as BOOL;
        }
        n += 1;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn DisableShellext() -> BOOL {
    let mut hkey: HKEY = core::mem::zeroed();
    let mut ret: LSTATUS = RegOpenKeyExW(HKEY_LOCAL_MACHINE, CPP_COM_WSTR_01518F84F715.as_ptr(), 0, KEY_QUERY_VALUE, core::ptr::addr_of_mut!(hkey));
    let mut Disable: BOOL = (FALSE as i32);
    if ret == (ERROR_SUCCESS as i32)
    {
        let mut dwValue: DWORD = 0;
        let mut r#type: DWORD = 0;
        let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;
        ret = RegQueryValueExW(hkey, CPP_COM_WSTR_EE9140933D4C.as_ptr(), core::ptr::null_mut(), core::ptr::addr_of_mut!(r#type), core::ptr::addr_of_mut!(dwValue) as PBYTE, core::ptr::addr_of_mut!(size));
        if ret == (ERROR_SUCCESS as i32) && r#type == REG_DWORD
        {
            Disable = (dwValue != 0) as BOOL;
        }
        if Disable == 0
        {
            size = core::mem::size_of::<DWORD>() as DWORD;
            ret = RegQueryValueExW(hkey, CPP_COM_WSTR_EFAFC0F54D9B.as_ptr(), core::ptr::null_mut(), core::ptr::addr_of_mut!(r#type), core::ptr::addr_of_mut!(dwValue) as PBYTE, core::ptr::addr_of_mut!(size));
            if ret == (ERROR_SUCCESS as i32) && r#type == REG_DWORD
            {
                Disable = (dwValue != 0) as BOOL;
            }
        }

        RegCloseKey(hkey);
    }
    Disable
}

impl CLayerUIPropPage {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_psp = CLayerUIPropPage::__psp_init(this_);
        (*this_).m_IsSfcProtected = (FALSE as i32);
        (*this_).m_AllowPermLayer = (FALSE as i32);
        (*this_).m_LayerQueryFlags = (GPLK_USER as u32);
        (*this_).m_RegistryOSMode = 0;
        (*this_).m_OSMode = 0;
        (*this_).m_RegistryEnabledLayers = 0;
        (*this_).m_EnabledLayers = 0;
        {
            let mut title = CComBSTR::new();
            title.LoadString(g_hModule as *mut c_void, IDS_COMPAT_TITLE as u32);
            (*this_).m_psp.pszTitle = title.Detach() as *const u16;
            (*this_).m_psp.dwFlags |= PSP_USETITLE;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        let mut title: CComBSTR = CComBSTR::new();
        title.Attach((*this_).m_psp.pszTitle as *mut u16);
    }
    pub unsafe fn Initialize(&mut self, pidlFolder: PCIDLIST_ABSOLUTE, pDataObj: LPDATAOBJECT, hkeyProgID: HKEY) -> HRESULT {
        let this_: *mut Self = self;
        let mut etc = FORMATETC { cfFormat: (CF_HDROP as u16), ptd: core::ptr::null_mut(), dwAspect: DVASPECT_CONTENT, lindex: -1, tymed: TYMED_HGLOBAL };
        let mut stg: STGMEDIUM = core::mem::zeroed();

        if DisableShellext() != 0 {
            return E_ACCESSDENIED;
        }

        let mut hr: HRESULT = (*pDataObj).GetData(core::ptr::addr_of_mut!(etc), core::ptr::addr_of_mut!(stg));
        if hr < 0 {
            ACDBG!(CPP_COM_WSTR_21C321DDA9DE.as_ptr());
            return E_INVALIDARG;
        }
        hr = E_FAIL;
        let hdrop: HDROP = GlobalLock(stg.__bindgen_anon_1.hGlobal) as HDROP;
        if !hdrop.is_null() {
            let uNumFiles: UINT = DragQueryFileW(hdrop, 0xFFFFFFFF, core::ptr::null_mut(), 0);
            if uNumFiles == 1 {
                let mut szFile = [0u16; (MAX_PATH * 2) as _];
                if DragQueryFileW(hdrop, 0, szFile.as_mut_ptr(), _countof!(szFile)) != 0 {
                    (*this_).AddRef();
                    hr = (*this_).InitFile(szFile.as_ptr());
                } else {
                    ACDBG!(CPP_COM_WSTR_E2ED9821845A.as_ptr());
                }
            } else {
                ACDBG!(CPP_COM_WSTR_35C1689B190B.as_ptr(), uNumFiles);
            }
            GlobalUnlock(stg.__bindgen_anon_1.hGlobal);
        } else {
            ACDBG!(CPP_COM_WSTR_4A36961D9B2E.as_ptr());
        }
        ReleaseStgMedium(core::ptr::addr_of_mut!(stg));
        return hr;
    }
    pub unsafe fn InitFile(&mut self, Filename: PCWSTR) -> HRESULT {
        let this_: *mut Self = self;
        let mut ExpandedFilename = CStringW::new();
        let dwRequired: DWORD = ExpandEnvironmentStringsW(Filename, core::ptr::null_mut(), 0);
        if dwRequired > 0 {
            let Buffer: LPWSTR = ExpandedFilename.GetBuffer(dwRequired as i32);
            let dwReturned: DWORD = ExpandEnvironmentStringsW(Filename, Buffer, dwRequired);
            if dwRequired == dwReturned {
                ExpandedFilename.ReleaseBufferSetLength((dwReturned - 1) as i32);
                ACDBG!(CPP_COM_WSTR_133F4E666837.as_ptr(), Filename, ExpandedFilename.GetString());
            } else {
                ExpandedFilename.ReleaseBufferSetLength(0);
                ExpandedFilename = CStringW::from_ptr(Filename);
                ACDBG!(CPP_COM_WSTR_A8879050313A.as_ptr(), Filename);
            }
        } else {
            ACDBG!(CPP_COM_WSTR_5DBA9F26C9D0.as_ptr(), Filename);
            ExpandedFilename = CStringW::from_ptr(Filename);
        }
        let pwszExt: PCWSTR = PathFindExtensionW(ExpandedFilename.GetString());
        if pwszExt.is_null() {
            ACDBG!(CPP_COM_WSTR_DCD52D52D7F4.as_ptr(), ExpandedFilename.GetString());
            return E_FAIL;
        }
        if _wcsicmp(pwszExt, CPP_COM_WSTR_FE706C396A0C.as_ptr()) == 0 {
            let mut Buffer = [0 as WCHAR; (MAX_PATH) as _];
            if GetExeFromLnk(ExpandedFilename.GetString(), Buffer.as_mut_ptr(), (_countof!(Buffer)) as _) == 0 {
                ACDBG!(CPP_COM_WSTR_ACCB067678BF.as_ptr(), ExpandedFilename.GetString());
                return E_FAIL;
            }
            if _wcsicmp(Buffer.as_ptr(), ExpandedFilename.GetString()) == 0 {
                ACDBG!(CPP_COM_WSTR_6FECEA5B5508.as_ptr(), ExpandedFilename.GetString());
                return E_FAIL;
            }
            return (*this_).InitFile(Buffer.as_ptr());
        }

        let mut tmp = CStringW::new();
        if tmp.GetEnvironmentVariable(CPP_COM_WSTR_FD1B96BF0940.as_ptr()) != 0 {
            tmp += CPP_COM_WSTR_6785322A4096.as_ptr();
            if ExpandedFilename.GetLength() >= tmp.GetLength() {
                let mut left = ExpandedFilename.Left(tmp.GetLength());
                left.MakeLower();
                tmp.MakeLower();
                if left.Compare(tmp.GetString()) == 0 {
                    ACDBG!(CPP_COM_WSTR_13B80F0791ED.as_ptr(), ExpandedFilename.GetString());
                    return E_FAIL;
                }
            }
            tmp.GetEnvironmentVariable(CPP_COM_WSTR_FD1B96BF0940.as_ptr());
            tmp += CPP_COM_WSTR_57B9731B469C.as_ptr();
            if ExpandedFilename.GetLength() >= tmp.GetLength() {
                let mut left = ExpandedFilename.Left(tmp.GetLength());
                left.MakeLower();
                tmp.MakeLower();
                if left.Compare(tmp.GetString()) == 0 {
                    ACDBG!(CPP_COM_WSTR_0EB4CDB484FE.as_ptr(), ExpandedFilename.GetString());
                    return E_FAIL;
                }
            }
        }

        let mut n: usize = 0;
        while !g_AllowedExtensions[n].is_null() {
            if _wcsicmp(g_AllowedExtensions[n], pwszExt) == 0 {
                (*this_).m_Filename = CStringW::from_ptr(ExpandedFilename.GetString());
                ACDBG!(CPP_COM_WSTR_B74DE3D9527D.as_ptr(), ExpandedFilename.GetString());
                (*this_).m_IsSfcProtected = SfcIsFileProtected(core::ptr::null_mut(), (*this_).m_Filename.GetString());
                (*this_).m_AllowPermLayer = AllowPermLayer(ExpandedFilename.GetString());
                return S_OK;
            }
            n += 1;
        }
        ACDBG!(CPP_COM_WSTR_4449408319CF.as_ptr(), pwszExt);
        return E_FAIL;
    }
    pub unsafe fn UpdateControls(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_OSMode = 0;
        (*this_).m_EnabledLayers = 0;
        let ModeEnabled: BOOL = ((*this_).__base0.IsDlgButtonChecked((IDC_CHKRUNCOMPATIBILITY as i32))) as i32;
        if ModeEnabled != 0 {
            (*this_).m_OSMode = (ComboBox_GetCurSel!((*this_).__base0.GetDlgItem((IDC_COMPATIBILITYMODE as i32))) + 1) as DWORD;
        }
        EnableWindow((*this_).__base0.GetDlgItem((IDC_COMPATIBILITYMODE as i32)), ModeEnabled);

        let mut n: usize = 0;
        while !g_Layers[n].Name.is_null() {
            (*this_).m_EnabledLayers |= if (*this_).__base0.IsDlgButtonChecked((g_Layers[n].Id) as i32) != 0 {
                (1 << n)
            } else {
                0
            };
            ShowWindow((*this_).__base0.GetDlgItem((g_Layers[n].Id) as i32), (SW_SHOW as i32));
            n += 1;
        }

        let mut customLayers: CStringW = CStringW::new();
        let mut j: i32 = 0;
        while j < (*this_).m_CustomLayers.GetSize() {
            if j > 0 {
                customLayers += CPP_COM_WSTR_82294FD79A0D.as_ptr();
            }
            customLayers += (&(*this_).m_CustomLayers)[j as usize].GetString();
            j += 1;
        }
        (*this_).__base0.SetDlgItemTextW((IDC_ENABLED_LAYERS as i32), customLayers.GetString());

        (*this_).SetModified((*this_).HasChanges());
    }
    pub unsafe fn DisableControls(&mut self) -> INT_PTR {
        let this_: *mut Self = self;
        EnableWindow((*this_).__base0.GetDlgItem((IDC_COMPATIBILITYMODE as i32)), 0);
        EnableWindow((*this_).__base0.GetDlgItem((IDC_CHKRUNCOMPATIBILITY as i32)), 0);
        let mut n: usize = 0;
        while !g_Layers[n].Name.is_null() {
            EnableWindow((*this_).__base0.GetDlgItem((g_Layers[n].Id) as i32), 0);
            n += 1;
        }
        EnableWindow((*this_).__base0.GetDlgItem((IDC_EDITCOMPATIBILITYMODES as i32)), 0);
        return (TRUE) as INT_PTR;
    }
    pub unsafe fn HasChanges(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_RegistryEnabledLayers != (*this_).m_EnabledLayers {
            return (TRUE) as BOOL;
        }

        if (*this_).m_RegistryOSMode != (*this_).m_OSMode {
            return (TRUE) as BOOL;
        }

        if ((ArrayEquals(&(*this_).m_RegistryCustomLayers, &(*this_).m_CustomLayers)) == 0) {
            return (TRUE) as BOOL;
        }

        return (FALSE) as BOOL;
    }
    pub unsafe fn OnSetActive(&mut self) -> i32 {
        let this_: *mut Self = self;
        if GetLayerInfo((*this_).m_Filename.GetString(), (*this_).m_LayerQueryFlags, core::ptr::addr_of_mut!((*this_).m_RegistryOSMode), core::ptr::addr_of_mut!((*this_).m_RegistryEnabledLayers), &mut (*this_).m_RegistryCustomLayers) == 0 {
            (*this_).m_RegistryEnabledLayers = 0;
            (*this_).m_RegistryOSMode = 0;
        }

        let mut n: usize = 0;
        while !g_Layers[n].Name.is_null() {
            (*this_).__base0.CheckDlgButton((g_Layers[n].Id) as i32, if ((*this_).m_RegistryEnabledLayers & (1 << n)) != 0 { BST_CHECKED } else { BST_UNCHECKED });
            n += 1;
        }

        (*this_).__base0.CheckDlgButton((IDC_CHKRUNCOMPATIBILITY as i32), if (*this_).m_RegistryOSMode != 0 { BST_CHECKED } else { BST_UNCHECKED });

        if (*this_).m_RegistryOSMode != 0 {
            ComboBox_SetCurSel!((*this_).__base0.GetDlgItem((IDC_COMPATIBILITYMODE as i32)), (*this_).m_RegistryOSMode - 1);
        }

        (*this_).m_CustomLayers = (*this_).m_RegistryCustomLayers.clone();

        (*this_).UpdateControls();

        return 0;
    }
    pub unsafe fn OnApply(&mut self) -> i32 {
        let this_: *mut Self = self;
        if (*this_).HasChanges() != 0 {
            let bMachine: BOOL = ((*this_).m_LayerQueryFlags == (GPLK_MACHINE as u32)) as BOOL;

            let mut n: usize = 0 as usize;
            while !g_CompatModes[n].Name.is_null() {
                SetPermLayerState((&(*this_).m_Filename).GetString(), g_CompatModes[n].Name, 0, bMachine, ((n + 1 as usize) == (*this_).m_OSMode as usize) as i32);
                n += 1;
            }

            let mut n: usize = 0 as usize;
            while !g_Layers[n].Name.is_null() {
                SetPermLayerState((&(*this_).m_Filename).GetString(), g_Layers[n].Name, 0, bMachine, (((1 << n) & (*this_).m_EnabledLayers) != 0) as BOOL);
                n += 1;
            }

            /* Disable all old values */
            let mut j: i32 = 0;
            while j < (*this_).m_RegistryCustomLayers.GetSize() {
                SetPermLayerState((&(*this_).m_Filename).GetString(), (&(*this_).m_RegistryCustomLayers)[(j) as usize].GetString(), 0, bMachine, (FALSE) as BOOL);
                j += 1;
            }

            /* Enable all new values */
            let mut j: i32 = 0;
            while j < (*this_).m_CustomLayers.GetSize() {
                SetPermLayerState((&(*this_).m_Filename).GetString(), (&(*this_).m_CustomLayers)[(j) as usize].GetString(), 0, bMachine, (TRUE) as BOOL);
                j += 1;
            }

            SHChangeNotify((SHCNE_UPDATEITEM) as LONG, SHCNF_PATHW, ((*this_).m_Filename.GetString() as PCWSTR).cast(), core::ptr::null_mut());
        }

        return (PSNRET_NOERROR as i32);
    }
    pub unsafe fn OnInitDialog(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let cboMode: HWND = (*this_).__base0.GetDlgItem((IDC_COMPATIBILITYMODE as i32));
        let mut n: usize = 0;
        while !g_CompatModes[n].Display.is_null() {
            ComboBox_AddString!(cboMode, g_CompatModes[n].Display);
            n += 1;
        }
        ComboBox_SetCurSel!(cboMode, 4);

        let mut explanation = CStringW::new();
        if (*this_).m_AllowPermLayer == 0 {
            explanation.LoadStringW(IDS_FAILED_NETWORK);
            (*this_).DisableControls();
            ACDBG!(CPP_COM_WSTR_2E892533F5BA.as_ptr());
        } else if (*this_).m_IsSfcProtected != 0 {
            explanation.LoadStringW(IDS_FAILED_PROTECTED);
            (*this_).DisableControls();
            ACDBG!(CPP_COM_WSTR_7721435B36A0.as_ptr());
        } else {
            return (TRUE) as LRESULT;
        }
        (*this_).__base0.SetDlgItemTextW((IDC_EXPLANATION as i32), explanation.GetString());
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnCtrlCommand(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).UpdateControls();
        return 0;
    }
    pub unsafe fn OnEditModes(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if ShowEditCompatModes((*this_).__base0.m_hWnd, this_) != 0 {
            (*this_).UpdateControls();
        }
        return 0;
    }
    pub unsafe fn OnClickNotify(&mut self, uCode: INT, hdr: LPNMHDR, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*hdr).idFrom == (IDC_INFOLINK as _) {
            ShellExecute(core::ptr::null_mut(), CPP_COM_WSTR_82DB442E7A60.as_ptr(), CPP_COM_WSTR_640DE941CE91.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), (SW_SHOW as i32));
        }
        return 0;
    }
}
