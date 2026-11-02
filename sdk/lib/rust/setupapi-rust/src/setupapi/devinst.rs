use crate::setupapi::cfgmgr::CM_Connect_MachineW;
use crate::setupapi::cfgmgr::CM_Disable_DevNode_Ex;
use crate::setupapi::cfgmgr::CM_Disconnect_Machine;
use crate::setupapi::cfgmgr::CM_Enable_DevNode_Ex;
use crate::setupapi::cfgmgr::CM_Enumerate_Classes_Ex;
use crate::setupapi::cfgmgr::CM_Get_DevNode_Registry_Property_ExW;
use crate::setupapi::cfgmgr::CM_Get_Parent_Ex;
use crate::setupapi::cfgmgr::CM_Locate_DevNode_ExW;
use crate::setupapi::cfgmgr::CM_Open_Class_Key_ExW;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_00A6C7A05271: [u16; 18] = [
    0x0044u16, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0050, 0x0061, 0x0072,
    0x0061, 0x006D, 0x0065, 0x0074, 0x0065, 0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_0CF505C5BBC3: [u16; 13] = [
    0x0053u16, 0x0079, 0x006D, 0x0062, 0x006F, 0x006C, 0x0069, 0x0063, 0x004C, 0x0069,
    0x006E, 0x006B, 0x0000,
];
pub static RUSTCONV_WSTR_E9A256AB2F78: [u16; 15] = [
    0x0044u16, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0049, 0x006E, 0x0073, 0x0074,
    0x0061, 0x006E, 0x0063, 0x0065, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_2153E0EE784F: [u16; 13] = [
    0x0055u16, 0x0070, 0x0070, 0x0065, 0x0072, 0x0046, 0x0069, 0x006C, 0x0074, 0x0065,
    0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_266DE79877D8: [u16; 13] = [
    0x0043u16, 0x0061, 0x0070, 0x0061, 0x0062, 0x0069, 0x006C, 0x0069, 0x0074, 0x0069,
    0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_3A8293568D79: [u16; 11] = [
    0x0044u16, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0044, 0x0065, 0x0073, 0x0063,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_543AB1792386: [u16; 11] = [
    0x0048u16, 0x0061, 0x0072, 0x0064, 0x0077, 0x0061, 0x0072, 0x0065, 0x0049, 0x0044,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_5AFC259962D3: [u16; 14] = [
    0x0043u16, 0x006F, 0x006D, 0x0070, 0x0061, 0x0074, 0x0069, 0x0062, 0x006C, 0x0065,
    0x0049, 0x0044, 0x0073, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_61E59C1EA3A0: [u16; 10] = [0x0043u16, 0x006C, 0x0061, 0x0073, 0x0073, 0x0047, 0x0055, 0x0049, 0x0044, 0x0000];
pub static RUSTCONV_C2R_WSTR_6373E785AB3F: [u16; 6] = [0x0043u16, 0x006C, 0x0061, 0x0073, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_75AC876AB589: [u16; 20] = [
    0x004Cu16, 0x006F, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0049, 0x006E,
    0x0066, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_91C465DAE898: [u16; 4] = [0x004Du16, 0x0066, 0x0067, 0x0000];
pub static RUSTCONV_C2R_WSTR_B446E67F66EB: [u16; 12] = [
    0x0043u16, 0x006F, 0x006E, 0x0066, 0x0069, 0x0067, 0x0046, 0x006C, 0x0061, 0x0067,
    0x0073, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_B5B0A80B585D: [u16; 13] = [
    0x0046u16, 0x0072, 0x0069, 0x0065, 0x006E, 0x0064, 0x006C, 0x0079, 0x004E, 0x0061,
    0x006D, 0x0065, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_CF50DD39C387: [u16; 8] = [0x0053u16, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_E1359C0EA47C: [u16; 9] = [0x0055u16, 0x0049, 0x004E, 0x0075, 0x006D, 0x0062, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_F3D8D400CF21: [u16; 7] = [0x0044u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_F7E3ACFD800D: [u16; 13] = [
    0x004Cu16, 0x006F, 0x0077, 0x0065, 0x0072, 0x0046, 0x0069, 0x006C, 0x0074, 0x0065,
    0x0072, 0x0073, 0x0000,
];

static mut BackSlash: [WCHAR; 2] = ['\\' as i32 as WCHAR, 0 as ::core::ffi::c_int as WCHAR];

static mut DateFormat: [WCHAR; 9] = [
    '%' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    '-' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    '-' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut DotCoInstallers: [WCHAR; 14] = [
    '.' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut DotHW: [WCHAR; 4] = [
    '.' as i32 as WCHAR,
    'H' as i32 as WCHAR,
    'W' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut DotServices: [WCHAR; 10] = [
    '.' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut InfDirectory: [WCHAR; 5] = [
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut InstanceKeyFormat: [WCHAR; 6] = [
    '%' as i32 as WCHAR,
    '0' as i32 as WCHAR,
    '4' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut Version: [WCHAR; 8] = [
    'V' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut VersionFormat: [WCHAR; 12] = [
    '%' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    '.' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    '.' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    '.' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut REGSTR_DRIVER_DATE: [WCHAR; 11] = [
    'D' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut REGSTR_DRIVER_DATE_DATA: [WCHAR; 15] = [
    'D' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut REGSTR_DRIVER_VERSION: [WCHAR; 14] = [
    'D' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'V' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut REGSTR_SECURITY: [WCHAR; 9] = [
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut REGSTR_UI_NUMBER_DESC_FORMAT: [WCHAR; 19] = [
    'U' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut PropertyMap: [PropertyMapEntry; 35] = unsafe {
    [
        PropertyMapEntry {
            regType: REG_SZ as DWORD,
            nameA: b"DeviceDesc\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_3A8293568D79.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: REG_MULTI_SZ as DWORD,
            nameA: b"HardwareId\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_543AB1792386.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: REG_MULTI_SZ as DWORD,
            nameA: b"CompatibleIDs\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_5AFC259962D3.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: REG_SZ as DWORD,
            nameA: b"Service\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_CF50DD39C387.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: REG_SZ as DWORD,
            nameA: b"Class\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_6373E785AB3F.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: REG_SZ as DWORD,
            nameA: b"ClassGUID\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_61E59C1EA3A0.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: REG_SZ as DWORD,
            nameA: b"Driver\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_F3D8D400CF21.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: REG_DWORD as DWORD,
            nameA: b"ConfigFlags\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_B446E67F66EB.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: REG_SZ as DWORD,
            nameA: b"Mfg\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_91C465DAE898.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: REG_SZ as DWORD,
            nameA: b"FriendlyName\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_B5B0A80B585D.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: REG_SZ as DWORD,
            nameA: b"LocationInformation\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_75AC876AB589.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: REG_DWORD as DWORD,
            nameA: b"Capabilities\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_266DE79877D8.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: REG_DWORD as DWORD,
            nameA: b"UINumber\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_E1359C0EA47C.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: REG_MULTI_SZ as DWORD,
            nameA: b"UpperFilters\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_2153E0EE784F.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: REG_MULTI_SZ as DWORD,
            nameA: b"LowerFilters\0" as *const u8 as LPCSTR,
            nameW: RUSTCONV_C2R_WSTR_F7E3ACFD800D.as_ptr() as LPCWSTR,
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: REG_BINARY as DWORD,
            nameA: b"Security\0" as *const u8 as LPCSTR,
            nameW: &raw const REGSTR_SECURITY as LPCWSTR,
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: REG_SZ as DWORD,
            nameA: b"UINumberDescFormat\0" as *const u8 as LPCSTR,
            nameW: &raw const REGSTR_UI_NUMBER_DESC_FORMAT as LPCWSTR,
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
        PropertyMapEntry {
            regType: 0 as DWORD,
            nameA: ::core::ptr::null::<CHAR>(),
            nameW: ::core::ptr::null::<WCHAR>(),
        },
    ]
};


pub type CLASS_INSTALL_PROC = Option<
    unsafe extern "system" fn(DI_FUNCTION, HDEVINFO, PSP_DEVINFO_DATA) -> DWORD,
>;

pub type DEFAULT_CLASS_INSTALL_PROC =
    Option<unsafe extern "system" fn(HDEVINFO, PSP_DEVINFO_DATA) -> BOOL>;

pub type COINSTALLER_PROC = Option<
    unsafe extern "system" fn(
        DI_FUNCTION,
        HDEVINFO,
        PSP_DEVINFO_DATA,
        PCOINSTALLER_CONTEXT_DATA,
    ) -> DWORD,
>;

#[repr(C)]
pub struct CoInstallerElement {
    pub ListEntry: LIST_ENTRY,
    pub Module: HMODULE,
    pub Function: COINSTALLER_PROC,
    pub DoPostProcessing: BOOL,
    pub PrivateData: PVOID,
}

#[repr(C)]
pub struct GetSectionCallbackInfo {
    pub PlatformInfo: PSP_ALTPLATFORM_INFO,
    pub ProductType: BYTE,
    pub SuiteMask: WORD,
    pub PrefixLength: DWORD,
    pub BestSection: [WCHAR; (LINE_LEN + 1) as _],
    pub BestScore1: DWORD,
    pub BestScore2: DWORD,
    pub BestScore3: DWORD,
    pub BestScore4: DWORD,
    pub BestScore5: DWORD,
}

// static HKEY OpenHardwareProfileKey(HKEY HKLM, DWORD HwProfile, DWORD samDesired);
// (prototype only; defined with its function body elsewhere)

#[repr(C)]
pub struct PropertyMapEntry {
    pub regType: DWORD,
    pub nameA: LPCSTR,
    pub nameW: LPCWSTR,
}

unsafe extern "C" fn SETUPDI_GuidToString(guid: *const GUID, guidStr: LPWSTR) {
    static fmt: [u16; 51] = [
    0x007Bu16, 0x0025, 0x0030, 0x0038, 0x0058, 0x002D, 0x0025, 0x0030, 0x0034, 0x0058,
    0x002D, 0x0025, 0x0030, 0x0034, 0x0058, 0x002D, 0x0025, 0x0030, 0x0032, 0x0058,
    0x0025, 0x0030, 0x0032, 0x0058, 0x002D, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025,
    0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032,
    0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x007D,
    0x0000,
];

    sprintfW(guidStr, fmt.as_ptr(), (*guid).Data1, (*guid).Data2 as core::ffi::c_uint, (*guid).Data3 as core::ffi::c_uint,
        (*guid).Data4[0 as usize] as core::ffi::c_uint, (*guid).Data4[1 as usize] as core::ffi::c_uint, (*guid).Data4[2 as usize] as core::ffi::c_uint, (*guid).Data4[3 as usize] as core::ffi::c_uint,
        (*guid).Data4[4 as usize] as core::ffi::c_uint, (*guid).Data4[5 as usize] as core::ffi::c_uint, (*guid).Data4[6 as usize] as core::ffi::c_uint, (*guid).Data4[7 as usize] as core::ffi::c_uint);
}
#[no_mangle]
pub unsafe extern "C" fn GetErrorCodeFromCrCode(cr: CONFIGRET) -> DWORD {
    match cr {
        CR_ACCESS_DENIED => return ERROR_ACCESS_DENIED as DWORD,
        CR_BUFFER_SMALL => return ERROR_INSUFFICIENT_BUFFER as DWORD,
        CR_CALL_NOT_IMPLEMENTED => return ERROR_CALL_NOT_IMPLEMENTED as DWORD,
        CR_FAILURE => return ERROR_GEN_FAILURE as DWORD,
        CR_INVALID_DATA => return ERROR_INVALID_USER_BUFFER as DWORD,
        CR_INVALID_DEVICE_ID => return ERROR_INVALID_PARAMETER as DWORD,
        CR_INVALID_MACHINENAME => return ERROR_INVALID_COMPUTERNAME as DWORD,
        CR_INVALID_DEVNODE => return ERROR_INVALID_PARAMETER as DWORD,
        CR_INVALID_FLAG => return ERROR_INVALID_FLAGS as DWORD,
        CR_INVALID_POINTER => return ERROR_INVALID_PARAMETER as DWORD,
        CR_INVALID_PROPERTY => return ERROR_INVALID_PARAMETER as DWORD,
        CR_NO_SUCH_DEVNODE => return ERROR_FILE_NOT_FOUND as DWORD,
        CR_NO_SUCH_REGISTRY_KEY => return ERROR_FILE_NOT_FOUND as DWORD,
        CR_NO_SUCH_VALUE => return ERROR_FILE_NOT_FOUND as DWORD,
        CR_OUT_OF_MEMORY => return ERROR_NOT_ENOUGH_MEMORY as DWORD,
        CR_REGISTRY_ERROR => return ERROR_GEN_FAILURE as DWORD,
        CR_ALREADY_SUCH_DEVINST => return ERROR_DEVINST_ALREADY_EXISTS as DWORD,
        CR_SUCCESS => return ERROR_SUCCESS as DWORD,
        _ => return ERROR_GEN_FAILURE as DWORD,
    };
}

unsafe extern "C" fn CheckSectionValid(
    SectionName: LPCWSTR,
    PlatformInfo: PSP_ALTPLATFORM_INFO,
    ProductType: BYTE,
    SuiteMask: WORD,
    ScorePlatform: PDWORD,
    ScoreMajorVersion: PDWORD,
    ScoreMinorVersion: PDWORD,
    ScoreProductType: PDWORD,
    ScoreSuiteMask: PDWORD,
) -> BOOL {
    let mut Section: LPWSTR = core::ptr::null_mut();
    let mut pExtensionArchitecture: LPCWSTR = core::ptr::null();
    let mut Fields: [LPWSTR; 6] =
        [core::ptr::null_mut(); 6];
    let mut i: DWORD = 0;
    let mut ret: BOOL = FALSE as BOOL;

    static ExtensionPlatformNT: [WCHAR; 4] = [0x002Eu16, 0x004E, 0x0054, 0x0000];
    static ExtensionPlatformWindows: [WCHAR; 5] = [0x002Eu16, 0x0057, 0x0069, 0x006E, 0x0000];

    static ExtensionArchitectureNone: [WCHAR; 1] = [0];
    static ExtensionArchitecturealpha: [WCHAR; 6] = [0x0061u16, 0x006C, 0x0070, 0x0068, 0x0061, 0x0000];
    static ExtensionArchitectureamd64: [WCHAR; 6] = [0x0041u16, 0x004D, 0x0044, 0x0036, 0x0034, 0x0000];
    static ExtensionArchitecturearm: [WCHAR; 4] = [0x0041u16, 0x0052, 0x004D, 0x0000];
    static ExtensionArchitecturearm64: [WCHAR; 6] = [0x0041u16, 0x0052, 0x004D, 0x0036, 0x0034, 0x0000];
    static ExtensionArchitectureia64: [WCHAR; 5] = [0x0049u16, 0x0041, 0x0036, 0x0034, 0x0000];
    static ExtensionArchitecturemips: [WCHAR; 5] = [0x006Du16, 0x0069, 0x0070, 0x0073, 0x0000];
    static ExtensionArchitectureppc: [WCHAR; 4] = [0x0070u16, 0x0070, 0x0063, 0x0000];
    static ExtensionArchitecturex86: [WCHAR; 4] = [0x0078u16, 0x0038, 0x0036, 0x0000];

    *ScoreSuiteMask = 0 as DWORD;
    *ScoreProductType = *ScoreSuiteMask;
    *ScoreMinorVersion = *ScoreProductType;
    *ScoreMajorVersion = *ScoreMinorVersion;
    *ScorePlatform = *ScoreMajorVersion;

    'cleanup: {
        Section = pSetupDuplicateString(SectionName) as LPWSTR;
        if Section.is_null() {
            break 'cleanup;
        }

        /* Set various extensions values */
        match (*PlatformInfo).Platform {
            VER_PLATFORM_WIN32_WINDOWS => {
                //pExtensionPlatform = ExtensionPlatformWindows;
            }
            VER_PLATFORM_WIN32_NT => {
                //pExtensionPlatform = ExtensionPlatformNT;
            }
            _ => {
                ERR!("Unknown platform 0x%lx\n", (*PlatformInfo).Platform);
                //pExtensionPlatform = ExtensionPlatformNone;
            }
        }
        match ((*PlatformInfo).ProcessorArchitecture) as u32 {
            PROCESSOR_ARCHITECTURE_ALPHA => {
                pExtensionArchitecture = core::ptr::addr_of!(ExtensionArchitecturealpha) as LPCWSTR;
            }
            PROCESSOR_ARCHITECTURE_AMD64 => {
                pExtensionArchitecture = core::ptr::addr_of!(ExtensionArchitectureamd64) as LPCWSTR;
            }
            PROCESSOR_ARCHITECTURE_ARM => {
                pExtensionArchitecture = core::ptr::addr_of!(ExtensionArchitecturearm) as LPCWSTR;
            }
            PROCESSOR_ARCHITECTURE_ARM64 => {
                pExtensionArchitecture = core::ptr::addr_of!(ExtensionArchitecturearm64) as LPCWSTR;
            }
            PROCESSOR_ARCHITECTURE_IA64 => {
                pExtensionArchitecture = core::ptr::addr_of!(ExtensionArchitectureia64) as LPCWSTR;
            }
            PROCESSOR_ARCHITECTURE_INTEL => {
                pExtensionArchitecture = core::ptr::addr_of!(ExtensionArchitecturex86) as LPCWSTR;
            }
            PROCESSOR_ARCHITECTURE_MIPS => {
                pExtensionArchitecture = core::ptr::addr_of!(ExtensionArchitecturemips) as LPCWSTR;
            }
            PROCESSOR_ARCHITECTURE_PPC => {
                pExtensionArchitecture = core::ptr::addr_of!(ExtensionArchitectureppc) as LPCWSTR;
            }
            PROCESSOR_ARCHITECTURE_UNKNOWN => {
                pExtensionArchitecture = core::ptr::addr_of!(ExtensionArchitectureNone) as LPCWSTR;
            }
            _ => {
                ERR!("Unknown processor architecture 0x%x\n", (*PlatformInfo).ProcessorArchitecture as core::ffi::c_uint);
                pExtensionArchitecture = core::ptr::addr_of!(ExtensionArchitectureNone) as LPCWSTR;
            }
        }

        /*
         * Field[0] Platform
         * Field[1] Architecture
         * Field[2] Major version
         * Field[3] Minor version
         * Field[4] Product type
         * Field[5] Suite mask
         * Remark: these fields may be NULL if the information is not provided
         */
        Fields[0 as usize] = Section;
        if Fields[0 as usize].is_null() {
            *ScorePlatform = ULONG_MAX as DWORD;
            *ScoreMajorVersion = ULONG_MAX as DWORD;
            *ScoreMinorVersion = ULONG_MAX as DWORD;
            *ScoreProductType = ULONG_MAX as DWORD;
            *ScoreSuiteMask = ULONG_MAX as DWORD;
            ret = TRUE as BOOL;
            break 'cleanup;
        }
        Fields[1 as usize] =
            Fields[0 as usize].offset(1 as isize);
        Fields[5 as usize] = core::ptr::null_mut();
        Fields[4 as usize] = core::ptr::null_mut();
        Fields[3 as usize] = core::ptr::null_mut();
        Fields[2 as usize] = core::ptr::null_mut();
        i = 2 as DWORD;
        while !Fields[i.wrapping_sub(1 as DWORD) as usize].is_null()
            && i < 6 as DWORD
        {
            Fields[i as usize] = wcschr(
                Fields[i.wrapping_sub(1 as DWORD) as usize],
                '.' as WCHAR,
            ) as LPWSTR;
            if !Fields[i as usize].is_null() {
                Fields[i as usize] = Fields[i as usize].offset(1);
                *Fields[i as usize].offset(-(1 as isize)) =
                    UNICODE_NULL as WCHAR;
            }
            i = i.wrapping_add(1);
        }
        /* Take care of first 2 fields */
        if strncmpiW(
            Fields[0 as usize] as LPCWSTR,
            core::ptr::addr_of!(ExtensionPlatformWindows) as LPCWSTR,
            strlenW(core::ptr::addr_of!(ExtensionPlatformWindows) as LPCWSTR),
        ) == 0
        {
            if (*PlatformInfo).Platform != VER_PLATFORM_WIN32_WINDOWS {
                break 'cleanup;
            }
            Fields[1 as usize] = Fields[1 as usize]
                .offset(
                    (wcslen(core::ptr::addr_of!(ExtensionPlatformWindows) as LPCWSTR)
                        as isize)
                        - (1 as isize),
                );
        } else if strncmpiW(
            Fields[0 as usize] as LPCWSTR,
            core::ptr::addr_of!(ExtensionPlatformNT) as LPCWSTR,
            strlenW(core::ptr::addr_of!(ExtensionPlatformNT) as LPCWSTR),
        ) == 0
        {
            if (*PlatformInfo).Platform != VER_PLATFORM_WIN32_NT {
                break 'cleanup;
            }
            Fields[1 as usize] = Fields[1 as usize]
                .offset(
                    (wcslen(core::ptr::addr_of!(ExtensionPlatformNT) as LPCWSTR)
                        as isize)
                        - (1 as isize),
                );
        } else {
            /* No platform specified */
            *ScorePlatform |= 0x02 as DWORD;
        }
        if strcmpiW(
            Fields[1 as usize] as LPCWSTR,
            core::ptr::addr_of!(ExtensionArchitectureNone) as LPCWSTR,
        ) == 0
        {
            /* No architecture specified */
            *ScorePlatform |= 0x01 as DWORD;
        } else if strcmpiW(
            Fields[1 as usize] as LPCWSTR,
            pExtensionArchitecture,
        ) != 0
        {
            break 'cleanup;
        }

        /* Check if informations are matching */
        if !Fields[2 as usize].is_null()
            && *Fields[2 as usize] != 0
        {
            let mut MajorVersion: DWORD = 0;
            let mut MinorVersion: DWORD = 0 as DWORD;
            MajorVersion = strtoulW(
                Fields[2 as usize] as LPCWSTR,
                core::ptr::null_mut(),
                0,
            ) as DWORD;
            if (MajorVersion == 0 as DWORD
                || MajorVersion == ULONG_MAX as DWORD)
                && ((*_errno()) == (ERANGE as i32)
                    || (*_errno()) == (EINVAL as i32))
            {
                break 'cleanup;
            }
            if !Fields[3 as usize].is_null()
                && *Fields[3 as usize] != 0
            {
                MinorVersion = strtoulW(
                    Fields[3 as usize] as LPCWSTR,
                    core::ptr::null_mut(),
                    0,
                ) as DWORD;
                if (MinorVersion == 0 as DWORD
                    || MinorVersion == ULONG_MAX as DWORD)
                    && ((*_errno()) == (ERANGE as i32)
                        || (*_errno()) == (EINVAL as i32))
                {
                    break 'cleanup;
                }
            }
            if (*PlatformInfo).MajorVersion < MajorVersion
                || ((*PlatformInfo).MajorVersion == MajorVersion
                    && (*PlatformInfo).MinorVersion < MinorVersion)
            {
                break 'cleanup;
            }
            *ScoreMajorVersion = MajorVersion.wrapping_sub((*PlatformInfo).MajorVersion);
            if MajorVersion == (*PlatformInfo).MajorVersion {
                *ScoreMinorVersion = MinorVersion.wrapping_sub((*PlatformInfo).MinorVersion);
            } else {
                *ScoreMinorVersion = MinorVersion;
            }
        } else if !Fields[3 as usize].is_null()
            && *Fields[3 as usize] != 0
        {
            break 'cleanup;
        } else {
            *ScoreMajorVersion = (*PlatformInfo).MajorVersion;
            *ScoreMinorVersion = (*PlatformInfo).MinorVersion;
        }

        if !Fields[4 as usize].is_null()
            && *Fields[4 as usize] != 0
        {
            let mut CurrentProductType: DWORD = 0;
            CurrentProductType = strtoulW(
                Fields[4 as usize] as LPCWSTR,
                core::ptr::null_mut(),
                0,
            ) as DWORD;
            if (CurrentProductType == 0 as DWORD
                || CurrentProductType == ULONG_MAX as DWORD)
                && ((*_errno()) == (ERANGE as i32)
                    || (*_errno()) == (EINVAL as i32))
            {
                break 'cleanup;
            }
            if CurrentProductType != ProductType as DWORD {
                break 'cleanup;
            }
        } else {
            *ScoreProductType = 1 as DWORD;
        }

        if !Fields[5 as usize].is_null()
            && *Fields[5 as usize] != 0
        {
            let mut CurrentSuiteMask: DWORD = 0;
            CurrentSuiteMask = strtoulW(
                Fields[5 as usize] as LPCWSTR,
                core::ptr::null_mut(),
                0,
            ) as DWORD;
            if (CurrentSuiteMask == 0 as DWORD
                || CurrentSuiteMask == ULONG_MAX as DWORD)
                && ((*_errno()) == (ERANGE as i32)
                    || (*_errno()) == (EINVAL as i32))
            {
                break 'cleanup;
            }
            if (CurrentSuiteMask & !(SuiteMask as DWORD)) != 0 as DWORD {
                break 'cleanup;
            }
            *ScoreSuiteMask = (SuiteMask as DWORD) & !CurrentSuiteMask;
        } else {
            *ScoreSuiteMask = SuiteMask as DWORD;
        }

        ret = TRUE as BOOL;
    }

    MyFree(Section as PVOID);
    return ret;
}

unsafe extern "C" fn GetSectionCallback(SectionName: LPCWSTR, Context: PVOID) -> BOOL {
    let info: *mut GetSectionCallbackInfo = Context as *mut GetSectionCallbackInfo;
    let mut Score1: DWORD;
    let mut Score2: DWORD;
    let mut Score3: DWORD;
    let mut Score4: DWORD;
    let mut Score5: DWORD;
    let mut ret: BOOL;

    if *SectionName.add((*info).PrefixLength as usize) != '.' as u16 {
        return (TRUE) as BOOL;
    }

    Score1 = 0;
    Score2 = 0;
    Score3 = 0;
    Score4 = 0;
    Score5 = 0;

    ret = CheckSectionValid(
        SectionName.add((*info).PrefixLength as usize),
        (*info).PlatformInfo,
        (*info).ProductType,
        (*info).SuiteMask,
        core::ptr::addr_of_mut!(Score1),
        core::ptr::addr_of_mut!(Score2),
        core::ptr::addr_of_mut!(Score3),
        core::ptr::addr_of_mut!(Score4),
        core::ptr::addr_of_mut!(Score5),
    );
    if ret == 0 {
        return (TRUE) as BOOL;
    }

    'done: {
        'bettersection: {
            if Score1 > (*info).BestScore1 {
                break 'done;
            }
            if Score1 < (*info).BestScore1 {
                break 'bettersection;
            }
            if Score2 > (*info).BestScore2 {
                break 'done;
            }
            if Score2 < (*info).BestScore2 {
                break 'bettersection;
            }
            if Score3 > (*info).BestScore3 {
                break 'done;
            }
            if Score3 < (*info).BestScore3 {
                break 'bettersection;
            }
            if Score4 > (*info).BestScore4 {
                break 'done;
            }
            if Score4 < (*info).BestScore4 {
                break 'bettersection;
            }
            if Score5 > (*info).BestScore5 {
                break 'done;
            }
            if Score5 < (*info).BestScore5 {
                break 'bettersection;
            }
            break 'done;
        }

        // bettersection:
        strcpyW(
            core::ptr::addr_of_mut!((*info).BestSection) as *mut WCHAR,
            SectionName,
        );
        (*info).BestScore1 = Score1;
        (*info).BestScore2 = Score2;
        (*info).BestScore3 = Score3;
        (*info).BestScore4 = Score4;
        (*info).BestScore5 = Score5;
    }

    // done:
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetActualSectionToInstallExW(
    InfHandle: HINF,
    InfSectionName: PCWSTR,
    AlternatePlatformInfo: PSP_ALTPLATFORM_INFO,
    InfSectionWithExt: PWSTR,
    InfSectionWithExtSize: DWORD,
    RequiredSize: PDWORD,
    Extension: *mut PWSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;

    TRACE!(
        "%s(%p %s %p %p %lu %p %p %p)\n",
        "SetupDiGetActualSectionToInstallExW",
        InfHandle,
        debugstr_w(InfSectionName as *const WCHAR),
        AlternatePlatformInfo,
        InfSectionWithExt,
        InfSectionWithExtSize,
        RequiredSize,
        Extension,
        Reserved
    );

    'done: {
        if InfHandle.is_null() || InfHandle == (INVALID_HANDLE_VALUE as isize as HINF) {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else if InfSectionName.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else if !AlternatePlatformInfo.is_null()
            && (*AlternatePlatformInfo).cbSize as usize != core::mem::size_of::<SP_ALTPLATFORM_INFO>() as usize
        {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else if !Reserved.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else {
            static mut CurrentPlatform: SP_ALTPLATFORM_INFO = SP_ALTPLATFORM_INFO {
                cbSize: 0 as DWORD,
                ..unsafe { core::mem::zeroed() }
            };
            static mut CurrentProductType: BYTE = 0 as BYTE;
            static mut CurrentSuiteMask: WORD = 0 as WORD;
            let mut pPlatformInfo: PSP_ALTPLATFORM_INFO =
                core::ptr::addr_of_mut!(CurrentPlatform);
            let mut CallbackInfo: GetSectionCallbackInfo = core::mem::zeroed();
            let mut dwFullLength: DWORD = 0;
            let mut ProductType: BYTE = 0;
            let mut SuiteMask: WORD = 0;

            /* Fill platform info if needed */
            if !AlternatePlatformInfo.is_null() {
                pPlatformInfo = AlternatePlatformInfo;
                ProductType = 0 as BYTE;
                SuiteMask = 0 as WORD;
            } else {
                if CurrentPlatform.cbSize as usize != core::mem::size_of::<SP_ALTPLATFORM_INFO>() as usize {
                    /* That's the first time we go here. We need to fill in the structure */
                    let mut SystemInfo: SYSTEM_INFO = core::mem::zeroed();
                    GetSystemInfo(core::ptr::addr_of_mut!(SystemInfo));
                    CurrentPlatform.cbSize = core::mem::size_of::<SP_ALTPLATFORM_INFO>() as DWORD;
                    CurrentPlatform.Platform = OsVersionInfo.dwPlatformId;
                    CurrentPlatform.MajorVersion = OsVersionInfo.dwMajorVersion;
                    CurrentPlatform.MinorVersion = OsVersionInfo.dwMinorVersion;
                    CurrentPlatform.ProcessorArchitecture = SystemInfo


                        .__bindgen_anon_1.__bindgen_anon_1.wProcessorArchitecture;
                    CurrentPlatform.__bindgen_anon_1.Reserved = 0 as WORD;
                    CurrentProductType = OsVersionInfo.wProductType;
                    CurrentSuiteMask = OsVersionInfo.wSuiteMask;
                }
                ProductType = CurrentProductType;
                SuiteMask = CurrentSuiteMask;
            }

            CallbackInfo.PlatformInfo = pPlatformInfo;
            CallbackInfo.ProductType = ProductType;
            CallbackInfo.SuiteMask = SuiteMask;
            CallbackInfo.PrefixLength = strlenW(InfSectionName) as DWORD;
            CallbackInfo.BestScore1 = ULONG_MAX as DWORD;
            CallbackInfo.BestScore2 = ULONG_MAX as DWORD;
            CallbackInfo.BestScore3 = ULONG_MAX as DWORD;
            CallbackInfo.BestScore4 = ULONG_MAX as DWORD;
            CallbackInfo.BestScore5 = ULONG_MAX as DWORD;
            strcpyW(
                core::ptr::addr_of_mut!(CallbackInfo.BestSection) as *mut WCHAR,
                InfSectionName,
            );

            TRACE!(
                "EnumerateSectionsStartingWith(InfSectionName = %S)\n",
                InfSectionName
            );
            if EnumerateSectionsStartingWith(
                InfHandle,
                InfSectionName as LPCWSTR,
                Some(GetSectionCallback as unsafe extern "C" fn(LPCWSTR, PVOID) -> BOOL),
                core::ptr::addr_of_mut!(CallbackInfo) as PVOID,
            ) == 0
            {
                SetLastError(ERROR_GEN_FAILURE as DWORD);
                break 'done;
            }

            TRACE!(
                "CallbackInfo.BestSection = %S\n",
                core::ptr::addr_of_mut!(CallbackInfo.BestSection) as *mut WCHAR
            );
            dwFullLength = lstrlenW(
                core::ptr::addr_of!(CallbackInfo.BestSection) as LPCWSTR,
            ) as DWORD;
            if !RequiredSize.is_null() {
                *RequiredSize = dwFullLength.wrapping_add(1 as DWORD);
            }

            if InfSectionWithExtSize > 0 as DWORD {
                if InfSectionWithExtSize
                    < dwFullLength.wrapping_add(1 as DWORD)
                {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                    break 'done;
                }
                strcpyW(
                    InfSectionWithExt,
                    core::ptr::addr_of_mut!(CallbackInfo.BestSection) as *mut WCHAR,
                );
                if !Extension.is_null() {
                    let mut dwLength: DWORD =
                        lstrlenW(InfSectionName as LPCWSTR) as DWORD;
                    *Extension = if dwLength == dwFullLength {
                        core::ptr::null_mut()
                    } else {
                        InfSectionWithExt.add(dwLength as usize)
                    };
                }
            }

            ret = TRUE as BOOL;
        }
    }

    TRACE!("Returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetActualModelsSectionW(
    Context: PINFCONTEXT,
    AlternatePlatformInfo: PSP_ALTPLATFORM_INFO,
    InfSectionWithExt: PWSTR,
    InfSectionWithExtSize: DWORD,
    RequiredSize: PDWORD,
    Reserved: PVOID,
) -> BOOL {
    static mut CurrentPlatform: SP_ALTPLATFORM_INFO = unsafe { core::mem::zeroed() };
    static mut CurrentProductType: BYTE = 0;
    static mut CurrentSuiteMask: WORD = 0;

    let mut pPlatformInfo: PSP_ALTPLATFORM_INFO = core::ptr::null_mut();
    let mut ProductType: BYTE;
    let mut SuiteMask: WORD;
    let mut ModelsSection: [WCHAR; (LINE_LEN + 1) as _] =
        [0; (LINE_LEN + 1) as _];
    let mut BestSection: [WCHAR; (LINE_LEN + 1) as _] =
        [0; (LINE_LEN + 1) as _];
    let mut BestScore1: DWORD = ULONG_MAX;
    let mut BestScore2: DWORD = ULONG_MAX;
    let mut BestScore3: DWORD = ULONG_MAX;
    let mut BestScore4: DWORD = ULONG_MAX;
    let mut BestScore5: DWORD = ULONG_MAX;
    let mut FieldCount: DWORD;
    let mut i: DWORD;
    let mut dwFullLength: DWORD;

    if Context.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    if !AlternatePlatformInfo.is_null()
        && (*AlternatePlatformInfo).cbSize != (core::mem::size_of::<SP_ALTPLATFORM_INFO>() as DWORD)
    {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return (FALSE) as BOOL;
    }

    /* Resolve platform */
    if !AlternatePlatformInfo.is_null() {
        pPlatformInfo = AlternatePlatformInfo;
        ProductType = 0;
        SuiteMask = 0;
    } else {
        if CurrentPlatform.cbSize != core::mem::size_of::<SP_ALTPLATFORM_INFO>() as DWORD {
            let mut SystemInfo: SYSTEM_INFO = core::mem::zeroed();
            GetSystemInfo(core::ptr::addr_of_mut!(SystemInfo));
            CurrentPlatform.cbSize = core::mem::size_of::<SP_ALTPLATFORM_INFO>() as DWORD;
            CurrentPlatform.Platform = OsVersionInfo.dwPlatformId;
            CurrentPlatform.MajorVersion = OsVersionInfo.dwMajorVersion;
            CurrentPlatform.MinorVersion = OsVersionInfo.dwMinorVersion;
            CurrentPlatform.ProcessorArchitecture = SystemInfo.__bindgen_anon_1.__bindgen_anon_1.wProcessorArchitecture;
            CurrentPlatform.__bindgen_anon_1.Reserved = 0;
            CurrentProductType = OsVersionInfo.wProductType;
            CurrentSuiteMask = OsVersionInfo.wSuiteMask;
        }
        pPlatformInfo = core::ptr::addr_of_mut!(CurrentPlatform);
        ProductType = CurrentProductType;
        SuiteMask = CurrentSuiteMask;
    }

    /* Field 1: models-section-name */
    if SetupGetStringFieldW(
        Context,
        1,
        core::ptr::addr_of_mut!(ModelsSection) as PWSTR,
        ARRAYSIZE!(ModelsSection) as DWORD,
        core::ptr::null_mut(),
    ) == 0
        && SetupGetStringFieldW(
            Context,
            0,
            core::ptr::addr_of_mut!(ModelsSection) as PWSTR,
            ARRAYSIZE!(ModelsSection) as DWORD,
            core::ptr::null_mut(),
        ) == 0
    {
        SetLastError(ERROR_INVALID_DATA);
        return (FALSE) as BOOL;
    }

    strcpyW(
        core::ptr::addr_of_mut!(BestSection) as PWSTR,
        core::ptr::addr_of!(ModelsSection) as LPCWSTR,
    );
    FieldCount = SetupGetFieldCount(Context);

    /* Fields 2..N are the TargetOSVersion decorations */
    i = 2;
    while i <= FieldCount {
        let mut Candidate: [WCHAR; (LINE_LEN + 2) as _] =
            [0; (LINE_LEN + 2) as _]; /* +1 for possible leading '.' and +1 for NUL */
        let mut Score1: DWORD = 0;
        let mut Score2: DWORD = 0;
        let mut Score3: DWORD = 0;
        let mut Score4: DWORD = 0;
        let mut Score5: DWORD = 0;
        let mut DecLen: DWORD = 0;

        'body: {
            if SetupGetStringFieldW(
                Context,
                i,
                core::ptr::addr_of_mut!(Candidate) as PWSTR,
                (ARRAYSIZE!(Candidate) as DWORD) - 1,
                core::ptr::null_mut(),
            ) == 0
            {
                break 'body;
            }

            if Candidate[0] == 0 {
                break 'body;
            }

            /* CheckSectionValid expects a leading dot */
            if Candidate[0] != ('.' as WCHAR) {
                memmove(
                    (core::ptr::addr_of_mut!(Candidate) as PWSTR).offset(1 as isize) as PVOID,
                    core::ptr::addr_of_mut!(Candidate) as PWSTR as PVOID,
                    ((lstrlenW(core::ptr::addr_of!(Candidate) as LPCWSTR) + 1) as usize)
                        * core::mem::size_of::<WCHAR>(),
                );
                Candidate[0] = '.' as WCHAR;
            }

            if CheckSectionValid(
                core::ptr::addr_of!(Candidate) as LPCWSTR,
                pPlatformInfo,
                ProductType,
                SuiteMask,
                core::ptr::addr_of_mut!(Score1),
                core::ptr::addr_of_mut!(Score2),
                core::ptr::addr_of_mut!(Score3),
                core::ptr::addr_of_mut!(Score4),
                core::ptr::addr_of_mut!(Score5),
            ) == 0
            {
                break 'body;
            }

            /* Lower scores are better */
            'scored: {
                if Score1 > BestScore1 {
                    break 'body;
                }
                if Score1 < BestScore1 {
                    break 'scored;
                }
                if Score2 > BestScore2 {
                    break 'body;
                }
                if Score2 < BestScore2 {
                    break 'scored;
                }
                if Score3 > BestScore3 {
                    break 'body;
                }
                if Score3 < BestScore3 {
                    break 'scored;
                }
                if Score4 > BestScore4 {
                    break 'body;
                }
                if Score4 < BestScore4 {
                    break 'scored;
                }
                if Score5 > BestScore5 {
                    break 'body;
                }
                if Score5 < BestScore5 {
                    break 'scored;
                }
                break 'body;
            }

            /* better: */
            DecLen = lstrlenW(core::ptr::addr_of!(Candidate) as LPCWSTR) as DWORD;
            if (lstrlenW(core::ptr::addr_of!(ModelsSection) as LPCWSTR) as DWORD) + DecLen
                >= ARRAYSIZE!(BestSection) as DWORD
            {
                break 'body;
            }

            strcpyW(
                core::ptr::addr_of_mut!(BestSection) as PWSTR,
                core::ptr::addr_of!(ModelsSection) as LPCWSTR,
            );
            strcatW(
                core::ptr::addr_of_mut!(BestSection) as PWSTR,
                core::ptr::addr_of!(Candidate) as LPCWSTR,
            );
            BestScore1 = Score1;
            BestScore2 = Score2;
            BestScore3 = Score3;
            BestScore4 = Score4;
            BestScore5 = Score5;
        }

        i = i.wrapping_add(1);
    }

    dwFullLength = lstrlenW(core::ptr::addr_of!(BestSection) as LPCWSTR) as DWORD;
    if !RequiredSize.is_null() {
        *RequiredSize = dwFullLength + 1;
    }

    if InfSectionWithExtSize > 0 {
        if InfSectionWithExtSize < dwFullLength + 1 {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            return (FALSE) as BOOL;
        }
        strcpyW(
            InfSectionWithExt,
            core::ptr::addr_of!(BestSection) as LPCWSTR,
        );
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetActualModelsSectionA(
    Context: PINFCONTEXT,
    AlternatePlatformInfo: PSP_ALTPLATFORM_INFO,
    InfSectionWithExt: PSTR,
    InfSectionWithExtSize: DWORD,
    RequiredSize: PDWORD,
    Reserved: PVOID,
) -> BOOL {
    let mut Buffer: PWSTR = core::ptr::null_mut();
    let mut Required: DWORD = 0;
    let mut ret: BOOL = 0;

    if InfSectionWithExtSize > 0 {
        Buffer = HeapAlloc(
            GetProcessHeap(),
            0,
            (InfSectionWithExtSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as PWSTR;
        if Buffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return (FALSE) as BOOL;
        }
    }

    ret = SetupDiGetActualModelsSectionW(
        Context,
        AlternatePlatformInfo,
        Buffer,
        InfSectionWithExtSize,
        core::ptr::addr_of_mut!(Required),
        Reserved,
    );

    if ret == 0 && GetLastError() != ERROR_INSUFFICIENT_BUFFER {
        Required = 0;
    }

    if !RequiredSize.is_null() {
        *RequiredSize = Required;
    }

    if ret != 0 && !Buffer.is_null() && !InfSectionWithExt.is_null() {
        WideCharToMultiByte(
            CP_ACP,
            0,
            Buffer as LPCWSTR,
            -(1 as ::core::ffi::c_int),
            InfSectionWithExt as LPSTR,
            InfSectionWithExtSize as ::core::ffi::c_int,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
    }

    if !Buffer.is_null() {
        HeapFree(GetProcessHeap(), 0, Buffer as PVOID);
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CreateDeviceInfo(
    list: *mut DeviceInfoSet,
    InstancePath: LPCWSTR,
    pClassGuid: LPCGUID,
    pDeviceInfo: *mut *mut DeviceInfo,
) -> BOOL {
    let mut size: DWORD;
    let mut cr: CONFIGRET;
    let mut deviceInfo: *mut DeviceInfo;

    *pDeviceInfo = core::ptr::null_mut();

    size = (core::mem::offset_of!(DeviceInfo, Data) as DWORD)
        + ((strlenW(InstancePath) + 1) * ((core::mem::size_of::<WCHAR>() as DWORD)) as usize) as u32;
    deviceInfo = HeapAlloc(
        GetProcessHeap(),
        0,
        size as SIZE_T,
    ) as *mut DeviceInfo;
    if deviceInfo.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return (FALSE) as BOOL;
    }
    ZeroMemory!(deviceInfo as *mut core::ffi::c_void, size as SIZE_T);

    cr = CM_Locate_DevNode_ExW(
        core::ptr::addr_of_mut!((*deviceInfo).dnDevInst),
        InstancePath as DEVINSTID_W,
        CM_LOCATE_DEVNODE_PHANTOM as ULONG,
        (*list).hMachine,
    );
    if cr != CR_SUCCESS {
        SetLastError(GetErrorCodeFromCrCode(cr));
        return (FALSE) as BOOL;
    }

    (*deviceInfo).set = list;
    (*deviceInfo).InstallParams.cbSize =
        core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
    strcpyW(
        core::ptr::addr_of_mut!((*deviceInfo).Data) as *mut WCHAR,
        InstancePath,
    );
    (*deviceInfo).instanceId =
        core::ptr::addr_of_mut!((*deviceInfo).Data) as *mut WCHAR as PCWSTR;
    (*deviceInfo).UniqueId = strrchrW(
        core::ptr::addr_of_mut!((*deviceInfo).Data) as *mut WCHAR,
        '\\' as WCHAR,
    ) as PCWSTR;
    (*deviceInfo).DeviceDescription = core::ptr::null::<WCHAR>();
    memcpy(
        core::ptr::addr_of_mut!((*deviceInfo).ClassGuid) as *mut core::ffi::c_void,
        pClassGuid as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as usize,
    );
    (*deviceInfo).CreationFlags = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*deviceInfo).DriverListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*deviceInfo).InterfaceListHead));

    *pDeviceInfo = deviceInfo;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn DestroyClassInstallParams(installParams: *mut ClassInstallParams) -> BOOL {
    HeapFree(GetProcessHeap(), 0 as DWORD, (*installParams).PropChangeParams as PVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, (*installParams).AddPropertyPageData as PVOID);
    return TRUE as BOOL;
}

unsafe extern "C" fn DestroyDeviceInfo(deviceInfo: *mut DeviceInfo) -> BOOL {
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut driverInfo: *mut DriverInfoElement = core::ptr::null_mut();
    let mut deviceInterface: *mut DeviceInterface = core::ptr::null_mut();
    while IsListEmpty(&raw mut (*deviceInfo).DriverListHead) == false {
        ListEntry = RemoveHeadList(&raw mut (*deviceInfo).DriverListHead);
        driverInfo = (ListEntry as usize)
            .wrapping_sub(core::mem::offset_of!(DriverInfoElement, ListEntry) as usize)
            as *mut DriverInfoElement;
        if DestroyDriverInfoElement(driverInfo) == 0 {
            return (FALSE) as BOOL;
        }
    }
    while IsListEmpty(&raw mut (*deviceInfo).InterfaceListHead) == false {
        ListEntry = RemoveHeadList(&raw mut (*deviceInfo).InterfaceListHead);
        deviceInterface = (ListEntry as usize)
            .wrapping_sub(core::mem::offset_of!(DeviceInterface, ListEntry) as usize)
            as *mut DeviceInterface;
        if DestroyDeviceInterface(deviceInterface) == 0 {
            return (FALSE) as BOOL;
        }
    }
    DestroyClassInstallParams(&raw mut (*deviceInfo).ClassInstallParams);
    if !(*deviceInfo).hmodDevicePropPageProvider.is_null() {
        FreeLibrary((*deviceInfo).hmodDevicePropPageProvider);
    }
    return HeapFree(
        GetProcessHeap(),
        (0) as DWORD,
        deviceInfo as PVOID,
    );
}

unsafe extern "C" fn DestroyDeviceInfoSet(list: *mut DeviceInfoSet) -> BOOL {
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut();

    while IsListEmpty(core::ptr::addr_of_mut!((*list).ListHead)) == false {
        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!((*list).ListHead));
        deviceInfo = CONTAINING_RECORD!(ListEntry, DeviceInfo, ListEntry);
        if DestroyDeviceInfo(deviceInfo) == 0 {
            return (FALSE) as BOOL;
        }
    }
    if (*list).HKLM != HKEY_LOCAL_MACHINE {
        RegCloseKey((*list).HKLM);
    }
    CM_Disconnect_Machine((*list).hMachine);
    DestroyClassInstallParams(core::ptr::addr_of_mut!((*list).ClassInstallParams));
    if !(*list).hmodClassPropPageProvider.is_null() {
        FreeLibrary((*list).hmodClassPropPageProvider);
    }
    return HeapFree(GetProcessHeap(), 0, list as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiBuildClassInfoList(
    Flags: DWORD,
    ClassGuidList: LPGUID,
    ClassGuidListSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    return SetupDiBuildClassInfoListExW(
        Flags,
        ClassGuidList,
        ClassGuidListSize,
        RequiredSize,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiBuildClassInfoListExA(
    Flags: DWORD,
    ClassGuidList: LPGUID,
    ClassGuidListSize: DWORD,
    RequiredSize: PDWORD,
    MachineName: LPCSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut MachineNameW: LPWSTR = core::ptr::null_mut();
    let mut bResult: BOOL = 0;
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const i8,
            b"SetupDiBuildClassInfoListExA\0" as *const u8 as *const i8,
            938 as i32,
            b"%s(0x%lx %p %lu %p %s %p)\n\0" as *const u8 as *const i8,
            b"SetupDiBuildClassInfoListExA\0" as *const u8 as *const i8,
            Flags,
            ClassGuidList,
            ClassGuidListSize,
            RequiredSize,
            debugstr_a(MachineName as *const i8),
            Reserved,
        );
    }
    if !MachineName.is_null() {
        MachineNameW = pSetupMultiByteToUnicode(MachineName as PCSTR, CP_ACP as UINT) as LPWSTR;
        if MachineNameW.is_null() {
            return (FALSE) as BOOL;
        }
    }
    bResult = SetupDiBuildClassInfoListExW(
        Flags,
        ClassGuidList,
        ClassGuidListSize,
        RequiredSize,
        MachineNameW as LPCWSTR,
        Reserved,
    );
    MyFree(MachineNameW as PVOID);
    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiBuildClassInfoListExW(
    Flags: DWORD,
    ClassGuidList: LPGUID,
    ClassGuidListSize: DWORD,
    RequiredSize: PDWORD,
    MachineName: LPCWSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut CurrentClassGuid: GUID = core::mem::zeroed::<GUID>();
    let mut hClassKey: HKEY = core::ptr::null_mut();
    let mut dwIndex: DWORD = 0;
    let mut dwGuidListIndex: DWORD = 0;
    let mut hMachine: HMACHINE = core::ptr::null_mut();
    let mut cr: CONFIGRET = 0;

    if RequiredSize.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    } else if ClassGuidList.is_null() && ClassGuidListSize > 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    if !MachineName.is_null() {
        cr = CM_Connect_MachineW(MachineName, core::ptr::addr_of_mut!(hMachine));
        if cr != CR_SUCCESS {
            SetLastError(GetErrorCodeFromCrCode(cr));
            return (FALSE) as BOOL;
        }
    }

    dwIndex = 0;
    'lp1628: loop {
        'body: {
            cr = CM_Enumerate_Classes_Ex(
                dwIndex,
                core::ptr::addr_of_mut!(CurrentClassGuid),
                0,
                hMachine,
            );
            if cr == CR_SUCCESS {
                if CM_Open_Class_Key_ExW(
                    core::ptr::addr_of_mut!(CurrentClassGuid),
                    core::ptr::null_mut(),
                    KEY_QUERY_VALUE,
                    RegDisposition_OpenExisting,
                    core::ptr::addr_of_mut!(hClassKey),
                    CM_OPEN_CLASS_KEY_INSTALLER,
                    hMachine,
                ) != CR_SUCCESS
                {
                    SetLastError(GetErrorCodeFromCrCode(cr));
                    if !hMachine.is_null() {
                        CM_Disconnect_Machine(hMachine);
                    }
                    return (FALSE) as BOOL;
                }

                if RegQueryValueExW(
                    hClassKey,
                    REGSTR_VAL_NOUSECLASS,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) == 0
                {
                    RegCloseKey(hClassKey);
                    break 'body;
                }

                if (Flags & DIBCI_NOINSTALLCLASS) != 0
                    && RegQueryValueExW(
                        hClassKey,
                        REGSTR_VAL_NOINSTALLCLASS,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    ) == 0
                {
                    RegCloseKey(hClassKey);
                    break 'body;
                }

                if (Flags & DIBCI_NODISPLAYCLASS) != 0
                    && RegQueryValueExW(
                        hClassKey,
                        REGSTR_VAL_NODISPLAYCLASS,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    ) == 0
                {
                    RegCloseKey(hClassKey);
                    break 'body;
                }

                RegCloseKey(hClassKey);

                if dwGuidListIndex < ClassGuidListSize {
                    *ClassGuidList.add(dwGuidListIndex as usize) = CurrentClassGuid;
                }

                dwGuidListIndex = dwGuidListIndex.wrapping_add(1);
            }

            if cr != ERROR_SUCCESS {
                break 'lp1628;
            }
        }
        dwIndex = dwIndex.wrapping_add(1);
    }

    if !hMachine.is_null() {
        CM_Disconnect_Machine(hMachine);
    }

    if !RequiredSize.is_null() {
        *RequiredSize = dwGuidListIndex;
    }

    if ClassGuidListSize < dwGuidListIndex {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiClassGuidsFromNameA(
    ClassName: LPCSTR,
    ClassGuidList: LPGUID,
    ClassGuidListSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    return SetupDiClassGuidsFromNameExA(
        ClassName,
        ClassGuidList,
        ClassGuidListSize,
        RequiredSize,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiClassGuidsFromNameW(
    ClassName: LPCWSTR,
    ClassGuidList: LPGUID,
    ClassGuidListSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    return SetupDiClassGuidsFromNameExW(
        ClassName,
        ClassGuidList,
        ClassGuidListSize,
        RequiredSize,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiClassGuidsFromNameExA(
    ClassName: LPCSTR,
    ClassGuidList: LPGUID,
    ClassGuidListSize: DWORD,
    RequiredSize: PDWORD,
    MachineName: LPCSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut ClassNameW: LPWSTR = core::ptr::null_mut();
    let mut MachineNameW: LPWSTR = core::ptr::null_mut();
    let mut bResult: BOOL;

    if ClassName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    ClassNameW = pSetupMultiByteToUnicode(ClassName as PCSTR, CP_ACP as UINT) as LPWSTR;
    if ClassNameW.is_null() {
        return FALSE as BOOL;
    }

    if !MachineName.is_null() {
        MachineNameW = pSetupMultiByteToUnicode(MachineName as PCSTR, CP_ACP as UINT) as LPWSTR;
        if MachineNameW.is_null() {
            MyFree(ClassNameW as PVOID);
            return FALSE as BOOL;
        }
    }

    bResult = SetupDiClassGuidsFromNameExW(
        ClassNameW as LPCWSTR,
        ClassGuidList,
        ClassGuidListSize,
        RequiredSize,
        MachineNameW as LPCWSTR,
        Reserved,
    );

    MyFree(MachineNameW as PVOID);
    MyFree(ClassNameW as PVOID);

    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiClassGuidsFromNameExW(
    ClassName: LPCWSTR,
    ClassGuidList: LPGUID,
    ClassGuidListSize: DWORD,
    RequiredSize: PDWORD,
    MachineName: LPCWSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut szKeyName: [WCHAR; 40] = [0 as WCHAR; 40];
    let mut szClassName: [WCHAR; (MAX_CLASS_NAME_LEN as _)] = [0 as WCHAR; (MAX_CLASS_NAME_LEN as _)];
    let mut hClassesKey: HKEY = core::ptr::null_mut();
    let mut hClassKey: HKEY = core::ptr::null_mut();
    let mut dwLength: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut lError: LONG = 0;
    let mut dwGuidListIndex: DWORD = 0;

    if ClassName.is_null() || RequiredSize.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    if ClassGuidList.is_null() && ClassGuidListSize > 0 as DWORD {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    *RequiredSize = 0 as DWORD;

    hClassesKey = SetupDiOpenClassRegKeyExW(
        core::ptr::null::<GUID>(),
        KEY_ENUMERATE_SUB_KEYS as REGSAM,
        DIOCR_INSTALLER as DWORD,
        MachineName,
        Reserved,
    );
    if hClassesKey == INVALID_HANDLE_VALUE as HKEY {
        return (FALSE) as BOOL;
    }

    dwIndex = 0 as DWORD;
    loop {
        dwLength = 40 as DWORD;
        lError = RegEnumKeyExW(
            hClassesKey,
            dwIndex,
            szKeyName.as_mut_ptr() as LPWSTR,
            core::ptr::addr_of_mut!(dwLength),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut(),
        ) as LONG;

        if lError == ERROR_SUCCESS as LONG || lError == ERROR_MORE_DATA as LONG {
            if RegOpenKeyExW(
                hClassesKey,
                szKeyName.as_mut_ptr() as LPCWSTR,
                0 as DWORD,
                KEY_QUERY_VALUE as REGSAM,
                core::ptr::addr_of_mut!(hClassKey),
            ) != 0
            {
                RegCloseKey(hClassesKey);
                return (FALSE) as BOOL;
            }

            // MAX_CLASS_NAME_LEN (32) * sizeof(WCHAR)
            dwLength = (32 as DWORD).wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
            if RegQueryValueExW(
                hClassKey,
                REGSTR_VAL_CLASS,
                core::ptr::null_mut::<DWORD>(),
                core::ptr::null_mut::<DWORD>(),
                szClassName.as_mut_ptr() as LPBYTE,
                core::ptr::addr_of_mut!(dwLength),
            ) == 0
            {
                if strcmpiW(szClassName.as_mut_ptr() as LPWSTR, ClassName) == 0 {
                    if dwGuidListIndex < ClassGuidListSize {
                        if szKeyName[0] == '{' as WCHAR
                            && szKeyName[37] == '}' as WCHAR
                        {
                            szKeyName[37] = 0 as WCHAR;
                        }

                        UuidFromStringW(
                            core::ptr::addr_of_mut!(szKeyName[1]) as *mut WCHAR,
                            ClassGuidList.offset(dwGuidListIndex as isize) as *mut GUID,
                        );
                    }

                    dwGuidListIndex = dwGuidListIndex.wrapping_add(1);
                }
            }

            RegCloseKey(hClassKey);
        }

        if lError != ERROR_SUCCESS as LONG {
            break;
        }
        dwIndex = dwIndex.wrapping_add(1);
    }

    RegCloseKey(hClassesKey);

    if !RequiredSize.is_null() {
        *RequiredSize = dwGuidListIndex;
    }

    if ClassGuidListSize < dwGuidListIndex {
        SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiClassNameFromGuidA(
    ClassGuid: *const GUID,
    ClassName: PSTR,
    ClassNameSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    return SetupDiClassNameFromGuidExA(ClassGuid, ClassName,
                                       ClassNameSize, RequiredSize,
                                       core::ptr::null(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiClassNameFromGuidW(
    ClassGuid: *const GUID,
    ClassName: PWSTR,
    ClassNameSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    return SetupDiClassNameFromGuidExW(ClassGuid, ClassName,
                                       ClassNameSize, RequiredSize,
                                       core::ptr::null_mut(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiClassNameFromGuidExA(
    ClassGuid: *const GUID,
    ClassName: PSTR,
    ClassNameSize: DWORD,
    RequiredSize: PDWORD,
    MachineName: PCSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut ClassNameW: [WCHAR; (MAX_CLASS_NAME_LEN as _)] = [0; (MAX_CLASS_NAME_LEN as _)];
    let mut MachineNameW: LPWSTR = core::ptr::null_mut();
    let mut ret: BOOL;

    if !MachineName.is_null() {
        MachineNameW = pSetupMultiByteToUnicode(MachineName, CP_ACP) as LPWSTR;
    }
    ret = SetupDiClassNameFromGuidExW(
        ClassGuid,
        core::ptr::addr_of_mut!(ClassNameW) as PWSTR,
        MAX_CLASS_NAME_LEN,
        RequiredSize,
        MachineNameW as PCWSTR,
        Reserved,
    );
    if ret != 0 {
        let mut len: core::ffi::c_int = WideCharToMultiByte(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!(ClassNameW) as *mut WCHAR as LPCWSTR,
            -1,
            ClassName as LPSTR,
            ClassNameSize as core::ffi::c_int,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if len == 0 || len as DWORD > ClassNameSize {
            SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            ret = (FALSE as i32);
        }
    }
    MyFree(MachineNameW as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiClassNameFromGuidExW(
    ClassGuid: *const GUID,
    ClassName: PWSTR,
    ClassNameSize: DWORD,
    RequiredSize: PDWORD,
    MachineName: PCWSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut hKey: HKEY;
    let mut dwLength: DWORD = core::mem::zeroed();
    let mut dwRegType: DWORD = core::mem::zeroed();
    let mut rc: LONG;
    let mut Buffer: PWSTR = core::mem::zeroed();

    /* Make sure there's a GUID */
    if ClassGuid.is_null() {
        SetLastError(ERROR_INVALID_CLASS);
        return (FALSE) as BOOL;
    }

    /* Make sure there's a real buffer when there's a size */
    if (ClassNameSize > 0) && ClassName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    /* Open the key for the GUID */
    hKey = SetupDiOpenClassRegKeyExW(ClassGuid, KEY_QUERY_VALUE, DIOCR_INSTALLER, MachineName, Reserved);

    if hKey == (INVALID_HANDLE_VALUE).cast() {
        return (FALSE) as BOOL;
    }

    /* Retrieve the class name data and close the key */
    rc = QueryRegistryValue(
        hKey,
        REGSTR_VAL_CLASS,
        core::ptr::addr_of_mut!(Buffer) as *mut LPBYTE,
        core::ptr::addr_of_mut!(dwRegType),
        core::ptr::addr_of_mut!(dwLength),
    );
    RegCloseKey(hKey);

    /* Make sure we got the data */
    if rc != (ERROR_SUCCESS as LONG) {
        SetLastError(rc as DWORD);
        return (FALSE) as BOOL;
    }

    /* Make sure the data is a string */
    if dwRegType != REG_SZ {
        MyFree(Buffer as PVOID);
        SetLastError(ERROR_GEN_FAILURE);
        return (FALSE) as BOOL;
    }

    /* Determine the length of the class name */
    dwLength /= core::mem::size_of::<WCHAR>() as DWORD;

    if (dwLength == 0)
        || (*Buffer.offset((dwLength - 1) as isize) != UNICODE_NULL)
    {
        /* Count the null-terminator */
        dwLength += 1;
    }

    /* Inform the caller about the class name */
    if (!ClassName.is_null()) && (dwLength <= ClassNameSize) {
        memcpy(
            ClassName as *mut core::ffi::c_void,
            Buffer as *const core::ffi::c_void,
            ((dwLength - 1) as usize) * core::mem::size_of::<WCHAR>(),
        );
        *ClassName.offset((dwLength - 1) as isize) = UNICODE_NULL;
    }

    /* Inform the caller about the required size */
    if !RequiredSize.is_null() {
        *RequiredSize = dwLength;
    }

    /* Clean up the buffer */
    MyFree(Buffer as PVOID);

    /* Make sure the buffer was large enough */
    if ClassName.is_null() || (dwLength > ClassNameSize) {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return (FALSE) as BOOL;
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiCreateDeviceInfoList(
    ClassGuid: *const GUID,
    hwndParent: HWND,
) -> HDEVINFO {
    return SetupDiCreateDeviceInfoListExW(ClassGuid, hwndParent, core::ptr::null(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiCreateDeviceInfoListExA(
    ClassGuid: *const GUID,
    hwndParent: HWND,
    MachineName: PCSTR,
    Reserved: PVOID,
) -> HDEVINFO {
    let mut MachineNameW: LPWSTR = core::ptr::null_mut();
    let mut hDevInfo: HDEVINFO = core::ptr::null_mut();

    if !MachineName.is_null() {
        MachineNameW = pSetupMultiByteToUnicode(MachineName, CP_ACP as UINT) as LPWSTR;
        if MachineNameW.is_null() {
            return INVALID_HANDLE_VALUE;
        }
    }

    hDevInfo = SetupDiCreateDeviceInfoListExW(ClassGuid, hwndParent, MachineNameW as PCWSTR, Reserved);

    MyFree(MachineNameW as PVOID);

    return hDevInfo;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiCreateDeviceInfoListExW(
    ClassGuid: *const GUID,
    hwndParent: HWND,
    mut MachineName: PCWSTR,
    Reserved: PVOID,
) -> HDEVINFO {
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut size: DWORD = core::mem::offset_of!(DeviceInfoSet, szData) as DWORD;
    let mut rc: DWORD = 0;
    let mut cr: CONFIGRET = 0;
    let mut ret: HDEVINFO = INVALID_HANDLE_VALUE;

    'cleanup: {
        if MachineName != core::ptr::null() {
            let mut len: SIZE_T = strlenW(MachineName) as SIZE_T;
            if len >= ((SP_MAX_MACHINENAME_LENGTH - 4)) as _ {
                SetLastError(ERROR_INVALID_MACHINENAME);
                break 'cleanup;
            }
            if len > 0 {
                size += ((len as usize + 3) * core::mem::size_of::<WCHAR>()) as DWORD;
            } else {
                MachineName = core::ptr::null_mut();
            }
        }

        if !Reserved.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
            return INVALID_HANDLE_VALUE;
        }

        list = MyMalloc(size) as *mut DeviceInfoSet;
        if list.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return INVALID_HANDLE_VALUE;
        }
        ZeroMemory!(
            list as *mut core::ffi::c_void,
            core::mem::offset_of!(DeviceInfoSet, szData),
        );

        (*list).magic = SETUP_DEVICE_INFO_SET_MAGIC;
        memcpy(
            core::ptr::addr_of_mut!((*list).ClassGuid) as *mut core::ffi::c_void,
            (if !ClassGuid.is_null() {
                ClassGuid
            } else {
                core::ptr::addr_of!(GUID_NULL) as *const GUID
            }) as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        );
        (*list).InstallParams.cbSize =
            core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
        (*list).InstallParams.Flags |= DI_CLASSINSTALLPARAMS;
        (*list).InstallParams.hwndParent = hwndParent;
        if !MachineName.is_null() {
            rc = RegConnectRegistryW(
                MachineName,
                HKEY_LOCAL_MACHINE,
                core::ptr::addr_of_mut!((*list).HKLM),
            ) as DWORD;
            if rc != ERROR_SUCCESS {
                SetLastError(ERROR_INVALID_MACHINENAME);
                break 'cleanup;
            }

            (*list).szData[0] = '\\' as WCHAR;
            (*core::ptr::addr_of_mut!((*list).szData[0]).add((1) as usize)) = (*list).szData[0];
            strcpyW(
                (core::ptr::addr_of_mut!((*list).szData) as *mut WCHAR)
                    .offset(2 as isize),
                MachineName,
            );
            (*list).MachineName = core::ptr::addr_of_mut!((*list).szData) as *mut WCHAR as PCWSTR;
        } else {
            (*list).HKLM = HKEY_LOCAL_MACHINE;
            (*list).MachineName = core::ptr::null();
        }
        cr = CM_Connect_MachineW(
            (*list).MachineName,
            core::ptr::addr_of_mut!((*list).hMachine),
        );
        if cr != CR_SUCCESS {
            SetLastError(GetErrorCodeFromCrCode(cr));
            break 'cleanup;
        }
        InitializeListHead(core::ptr::addr_of_mut!((*list).DriverListHead));
        InitializeListHead(core::ptr::addr_of_mut!((*list).ListHead));

        return list as HDEVINFO;
    }

    if ret == INVALID_HANDLE_VALUE {
        if !list.is_null() {
            if !(*list).HKLM.is_null()
                && (*list).HKLM != HKEY_LOCAL_MACHINE
            {
                RegCloseKey((*list).HKLM);
            }
            MyFree(list as PVOID);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiCreateDevRegKeyA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Scope: DWORD,
    HwProfile: DWORD,
    KeyType: DWORD,
    InfHandle: HINF,
    InfSectionName: PCSTR,
) -> HKEY {
    let mut InfSectionNameW: PWSTR = core::ptr::null_mut();
    let mut key: HKEY;

    if !InfHandle.is_null() {
        if InfSectionName.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return INVALID_HANDLE_VALUE as HKEY;
        } else {
            InfSectionNameW = pSetupMultiByteToUnicode(InfSectionName, CP_ACP as UINT);
            if InfSectionNameW.is_null() {
                return INVALID_HANDLE_VALUE as HKEY;
            }
        }
    }
    key = SetupDiCreateDevRegKeyW(
        DeviceInfoSet,
        DeviceInfoData,
        Scope,
        HwProfile,
        KeyType,
        InfHandle,
        InfSectionNameW as PCWSTR,
    );
    MyFree(InfSectionNameW as PVOID);
    return key;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiCreateDevRegKeyW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Scope: DWORD,
    HwProfile: DWORD,
    KeyType: DWORD,
    InfHandle: HINF,
    InfSectionName: PCWSTR,
) -> HKEY {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut();
    let mut key: HKEY = (INVALID_HANDLE_VALUE).cast();
    let mut rc: DWORD = 0;
    let mut hHWProfileKey: HKEY = (INVALID_HANDLE_VALUE).cast();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut RootKey: HKEY = core::ptr::null_mut();

    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE);
        return (INVALID_HANDLE_VALUE).cast();
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE);
        return (INVALID_HANDLE_VALUE).cast();
    }
    if DeviceInfoData.is_null()
        || (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
        || (*DeviceInfoData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (INVALID_HANDLE_VALUE).cast();
    }
    if Scope != DICS_FLAG_GLOBAL && Scope != DICS_FLAG_CONFIGSPECIFIC {
        SetLastError(ERROR_INVALID_FLAGS);
        return (INVALID_HANDLE_VALUE).cast();
    }
    if KeyType != DIREG_DEV && KeyType != DIREG_DRV {
        SetLastError(ERROR_INVALID_FLAGS);
        return (INVALID_HANDLE_VALUE).cast();
    }
    if !InfHandle.is_null() && InfSectionName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (INVALID_HANDLE_VALUE).cast();
    }
    if InfHandle.is_null() && !InfSectionName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (INVALID_HANDLE_VALUE).cast();
    }

    deviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;

    'cleanup: {
        if Scope == DICS_FLAG_GLOBAL {
            RootKey = (*set).HKLM;
        } else {
            hHWProfileKey = OpenHardwareProfileKey(
                (*set).HKLM,
                HwProfile,
                KEY_CREATE_SUB_KEY,
            );
            if hHWProfileKey == (INVALID_HANDLE_VALUE).cast() {
                break 'cleanup;
            }
            RootKey = hHWProfileKey;
        }

        if KeyType == DIREG_DEV {
            hKey = SETUPDI_CreateDevKey(
                RootKey,
                deviceInfo,
                KEY_READ | KEY_WRITE,
            );
            if hKey == (INVALID_HANDLE_VALUE).cast() {
                break 'cleanup;
            }

            if Scope == DICS_FLAG_GLOBAL {
                let mut hTempKey: HKEY = hKey;

                rc = RegCreateKeyExW(
                    hTempKey,
                    RUSTCONV_WSTR_00A6C7A05271.as_ptr(),
                    0,
                    core::ptr::null_mut(),
                    REG_OPTION_NON_VOLATILE,
                    KEY_READ | KEY_WRITE,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(hKey),
                    core::ptr::null_mut(),
                ) as DWORD;
                if rc == ERROR_SUCCESS {
                    RegCloseKey(hTempKey);
                }
            }
        } else {
            hKey = SETUPDI_CreateDrvKey(
                RootKey,
                deviceInfo,
                core::ptr::addr_of_mut!((*DeviceInfoData).ClassGuid) as *mut UUID,
                KEY_READ | KEY_WRITE,
            );
            if hKey == (INVALID_HANDLE_VALUE).cast() {
                break 'cleanup;
            }
        }

        /* Do installation of the specified section */
        if !InfHandle.is_null() {
        }
        key = hKey;
    }

    if hHWProfileKey != (INVALID_HANDLE_VALUE).cast() {
        RegCloseKey(hHWProfileKey);
    }
    if !hKey.is_null() && hKey != key {
        RegCloseKey(hKey);
    }

    return key;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiCreateDeviceInfoA(
    DeviceInfoSet: HDEVINFO,
    DeviceName: PCSTR,
    ClassGuid: *const GUID,
    DeviceDescription: PCSTR,
    hwndParent: HWND,
    CreationFlags: DWORD,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut DeviceNameW: LPWSTR = core::ptr::null_mut();
    let mut DeviceDescriptionW: LPWSTR = core::ptr::null_mut();

    if !DeviceName.is_null() {
        DeviceNameW = pSetupMultiByteToUnicode(DeviceName, CP_ACP as UINT) as LPWSTR;
        if DeviceNameW.is_null() {
            return (FALSE) as BOOL;
        }
    }
    if !DeviceDescription.is_null() {
        DeviceDescriptionW = pSetupMultiByteToUnicode(DeviceDescription, CP_ACP as UINT) as LPWSTR;
        if DeviceDescriptionW.is_null() {
            MyFree(DeviceNameW as PVOID);
            return (FALSE) as BOOL;
        }
    }

    ret = SetupDiCreateDeviceInfoW(
        DeviceInfoSet,
        DeviceNameW as PCWSTR,
        ClassGuid,
        DeviceDescriptionW as PCWSTR,
        hwndParent,
        CreationFlags,
        DeviceInfoData,
    );

    MyFree(DeviceNameW as PVOID);
    MyFree(DeviceDescriptionW as PVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiCreateDeviceInfoW(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceName: PCWSTR,
    mut ClassGuid: *const GUID,
    mut DeviceDescription: PCWSTR,
    mut hwndParent: HWND,
    mut CreationFlags: DWORD,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut::<DeviceInfo>();
    let mut ret: BOOL = (FALSE as i32);
    let mut cr: CONFIGRET = 0;
    let mut RootDevInst: DEVINST = 0;
    let mut DevInst: DEVINST = 0;
    let mut GenInstanceId: [WCHAR; (MAX_DEVICE_ID_LEN as _)] = [0; (MAX_DEVICE_ID_LEN as _)];
    let mut dwFlags: DWORD = 0;

    if DeviceName.is_null() {
        SetLastError(ERROR_INVALID_DEVINST_NAME as DWORD);
        return (FALSE) as BOOL;
    }
    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    if ClassGuid.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    if IsEqualGUID!(
        core::ptr::addr_of!((*set).ClassGuid),
        core::ptr::addr_of!(GUID_NULL),
    ) == false
        && IsEqualGUID!(ClassGuid, core::ptr::addr_of!((*set).ClassGuid)) == false
    {
        SetLastError(ERROR_CLASS_MISMATCH as DWORD);
        return (FALSE) as BOOL;
    }
    if (CreationFlags & !(DICD_GENERATE_ID | DICD_INHERIT_CLASSDRVS)) != 0 {
        SetLastError(ERROR_INVALID_FLAGS as DWORD);
        return (FALSE) as BOOL;
    }

    /* Get the root device instance */
    cr = CM_Locate_DevInst_ExW(
        core::ptr::addr_of_mut!(RootDevInst),
        core::ptr::null_mut::<WCHAR>(),
        CM_LOCATE_DEVINST_NORMAL as ULONG,
        (*set).hMachine,
    );
    if cr != CR_SUCCESS as CONFIGRET {
        SetLastError(ERROR_INVALID_DATA as DWORD);
        return FALSE as BOOL;
    }

    dwFlags = CM_CREATE_DEVINST_PHANTOM as DWORD;
    if (CreationFlags & DICD_GENERATE_ID) != 0 {
        dwFlags |= CM_CREATE_DEVINST_GENERATE_ID as DWORD;
    }

    /* Create the new device instance */
    cr = CM_Create_DevInst_ExW(
        core::ptr::addr_of_mut!(DevInst),
        DeviceName as DEVINSTID,
        RootDevInst,
        dwFlags as ULONG,
        (*set).hMachine as HANDLE,
    );
    if cr != CR_SUCCESS as CONFIGRET {
        SetLastError(GetErrorCodeFromCrCode(cr));
        return FALSE as BOOL;
    }

    if (CreationFlags & DICD_GENERATE_ID) != 0 {
        /* Grab the actual instance ID that was created */
        cr = CM_Get_Device_ID_Ex(
            DevInst,
            GenInstanceId.as_mut_ptr() as PWCHAR,
            MAX_DEVICE_ID_LEN as ULONG,
            0 as ULONG,
            (*set).hMachine,
        );
        if cr != CR_SUCCESS as CONFIGRET {
            SetLastError(GetErrorCodeFromCrCode(cr));
            return FALSE as BOOL;
        }

        DeviceName = GenInstanceId.as_mut_ptr() as PCWSTR;
    }

    if CreateDeviceInfo(
        set,
        DeviceName as LPCWSTR,
        ClassGuid as LPCGUID,
        core::ptr::addr_of_mut!(deviceInfo),
    ) != 0
    {
        InsertTailList(
            core::ptr::addr_of_mut!((*set).ListHead),
            core::ptr::addr_of_mut!((*deviceInfo).ListEntry),
        );

        if DeviceInfoData.is_null() {
            ret = (TRUE as i32);
        } else {
            if (*DeviceInfoData).cbSize as usize
                != core::mem::size_of::<SP_DEVINFO_DATA>()
            {
                SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
            } else {
                memcpy(
                    core::ptr::addr_of_mut!((*DeviceInfoData).ClassGuid) as *mut core::ffi::c_void,
                    ClassGuid as *const core::ffi::c_void,
                    core::mem::size_of::<GUID>(),
                );
                (*DeviceInfoData).DevInst = (*deviceInfo).dnDevInst as DWORD;
                (*DeviceInfoData).Reserved = deviceInfo as ULONG_PTR;
                ret = (TRUE as i32);
            }
        }
    }

    if ret == (FALSE as BOOL) {
        if !deviceInfo.is_null() {
            /* Remove deviceInfo from List */
            RemoveEntryList(core::ptr::addr_of_mut!((*deviceInfo).ListEntry));

            /* Destroy deviceInfo */
            DestroyDeviceInfo(deviceInfo);
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiRegisterDeviceInfo(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
    mut Flags: DWORD,
    mut CompareProc: PSP_DETSIG_CMPPROC,
    mut CompareContext: PVOID,
    mut DupDeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut DevInstId: [WCHAR; (MAX_DEVICE_ID_LEN as _)] = [0; (MAX_DEVICE_ID_LEN as _)];
    let mut ParentDevInst: DEVINST = 0;
    let mut cr: CONFIGRET = 0;
    let mut dwError: DWORD = ERROR_SUCCESS as DWORD;

    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return FALSE as BOOL;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return FALSE as BOOL;
    }
    if DeviceInfoData.is_null()
        || (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
        || (*DeviceInfoData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    if Flags & !(SPRDI_FIND_DUPS as DWORD) != 0 {
        SetLastError(ERROR_INVALID_FLAGS as DWORD);
        return FALSE as BOOL;
    }

    if Flags & (SPRDI_FIND_DUPS as DWORD) != 0 {
    }

    CM_Get_Device_ID_Ex(
        (*DeviceInfoData).DevInst,
        core::ptr::addr_of_mut!(DevInstId) as PWCHAR,
        MAX_DEVICE_ID_LEN as ULONG,
        0 as ULONG,
        (*set).hMachine,
    );

    CM_Get_Parent_Ex(
        core::ptr::addr_of_mut!(ParentDevInst),
        (*DeviceInfoData).DevInst,
        0 as ULONG,
        (*set).hMachine,
    );

    cr = CM_Create_DevInst_Ex(
        core::ptr::addr_of_mut!((*DeviceInfoData).DevInst),
        core::ptr::addr_of_mut!(DevInstId) as PWCHAR,
        ParentDevInst,
        (CM_CREATE_DEVINST_NORMAL | CM_CREATE_DEVINST_DO_NOT_INSTALL) as ULONG,
        (*set).hMachine,
    );
    if cr != CR_SUCCESS as CONFIGRET
        && cr != CR_ALREADY_SUCH_DEVINST as CONFIGRET
    {
        dwError = ERROR_NO_SUCH_DEVINST as DWORD;
    }

    SetLastError(dwError);

    return (dwError == ERROR_SUCCESS as DWORD) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiEnumDeviceInfo(
    mut devinfo: HDEVINFO,
    mut index: DWORD,
    mut info: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiEnumDeviceInfo\0" as *const u8 as *const ::core::ffi::c_char,
            2045 as ::core::ffi::c_int,
            b"%s(%p %d %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiEnumDeviceInfo\0" as *const u8 as *const ::core::ffi::c_char,
            devinfo,
            index,
            info,
        );
    }
    if info.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    if !devinfo.is_null() && devinfo != INVALID_HANDLE_VALUE {
        let mut list: *mut DeviceInfoSet = devinfo as *mut DeviceInfoSet;
        if (*list).magic == SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
            if (*info).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() {
                SetLastError(ERROR_INVALID_USER_BUFFER);
            } else {
                let mut ItemList: PLIST_ENTRY = (*list).ListHead.Flink as PLIST_ENTRY;
                while ItemList != core::ptr::addr_of_mut!((*list).ListHead) && {
                    let fresh18 = index;
                    index = index.wrapping_sub(1);
                    fresh18 > 0 as DWORD
                } {
                    ItemList = (*ItemList).Flink as PLIST_ENTRY;
                }
                if ItemList == core::ptr::addr_of_mut!((*list).ListHead) {
                    SetLastError(ERROR_NO_MORE_ITEMS);
                } else {
                    // CONTAINING_RECORD(ItemList, struct DeviceInfo, ListEntry)
                    let mut DevInfo: *mut DeviceInfo = (ItemList as ULONG_PTR).wrapping_sub(
                        core::mem::offset_of!(DeviceInfo, ListEntry)
                            as ULONG_PTR,
                    ) as *mut DeviceInfo;
                    memcpy(
                        core::ptr::addr_of_mut!((*info).ClassGuid) as *mut ::core::ffi::c_void,
                        core::ptr::addr_of_mut!((*DevInfo).ClassGuid) as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<GUID>() as size_t,
                    );
                    (*info).DevInst = (*DevInfo).dnDevInst as DWORD;
                    (*info).Reserved = DevInfo as ULONG_PTR;
                    ret = (TRUE as i32);
                }
            }
        } else {
            SetLastError(ERROR_INVALID_HANDLE);
        }
    } else {
        SetLastError(ERROR_INVALID_HANDLE);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDeviceInstanceIdA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DeviceInstanceId: PSTR,
    DeviceInstanceIdSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    let mut size: DWORD = 0;
    let mut instanceId: PWSTR;

    TRACE!(
        "%s(%p %p %p %d %p)\n",
        b"SetupDiGetDeviceInstanceIdA\0".as_ptr() as *const i8,
        DeviceInfoSet,
        DeviceInfoData,
        DeviceInstanceId,
        DeviceInstanceIdSize,
        RequiredSize
    );

    if DeviceInstanceId.is_null() && DeviceInstanceIdSize > 0 as DWORD {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    ret = SetupDiGetDeviceInstanceIdW(
        DeviceInfoSet,
        DeviceInfoData,
        core::ptr::null_mut::<WCHAR>(),
        0 as DWORD,
        core::ptr::addr_of_mut!(size),
    );
    if ret == 0 && GetLastError() != ERROR_INSUFFICIENT_BUFFER as DWORD {
        return FALSE as BOOL;
    }
    instanceId = MyMalloc(size.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD))
        as PWSTR;
    if !instanceId.is_null() {
        ret = SetupDiGetDeviceInstanceIdW(
            DeviceInfoSet,
            DeviceInfoData,
            instanceId,
            size,
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            let mut len: core::ffi::c_int = WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                instanceId as LPCWSTR,
                -(1 as core::ffi::c_int),
                DeviceInstanceId as LPSTR,
                DeviceInstanceIdSize as core::ffi::c_int,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            );

            if len == 0 {
                ret = FALSE as BOOL;
            } else {
                if len as DWORD > DeviceInstanceIdSize {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                    ret = FALSE as BOOL;
                }
                if !RequiredSize.is_null() {
                    *RequiredSize = len as DWORD;
                }
            }
        }
        MyFree(instanceId as PVOID);
    } else {
        if !RequiredSize.is_null() {
            *RequiredSize = size;
        }
        SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDeviceInstanceIdW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DeviceInstanceId: PWSTR,
    DeviceInstanceIdSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut devInfo: *mut DeviceInfo = core::ptr::null_mut();

    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    if DeviceInfoData.is_null()
        || (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
        || (*DeviceInfoData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    devInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
    if DeviceInstanceId.is_null() && DeviceInstanceIdSize > 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    if !DeviceInstanceId.is_null() && DeviceInstanceIdSize == 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    if DeviceInstanceIdSize
        < (lstrlenW((*devInfo).instanceId as LPCWSTR) + 1) as DWORD
    {
        SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
        if !RequiredSize.is_null() {
            *RequiredSize =
                (lstrlenW((*devInfo).instanceId as LPCWSTR) + 1) as DWORD;
        }
        return (FALSE) as BOOL;
    }
    lstrcpyW(DeviceInstanceId as LPWSTR, (*devInfo).instanceId as LPCWSTR);
    if !RequiredSize.is_null() {
        *RequiredSize =
            (lstrlenW((*devInfo).instanceId as LPCWSTR) + 1) as DWORD;
    }
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetActualSectionToInstallA(
    InfHandle: HINF,
    InfSectionName: PCSTR,
    InfSectionWithExt: PSTR,
    InfSectionWithExtSize: DWORD,
    RequiredSize: PDWORD,
    Extension: *mut PSTR,
) -> BOOL {
    return SetupDiGetActualSectionToInstallExA(
        InfHandle,
        InfSectionName,
        core::ptr::null_mut(),
        InfSectionWithExt,
        InfSectionWithExtSize,
        RequiredSize,
        Extension,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetActualSectionToInstallW(
    InfHandle: HINF,
    InfSectionName: PCWSTR,
    InfSectionWithExt: PWSTR,
    InfSectionWithExtSize: DWORD,
    RequiredSize: PDWORD,
    Extension: *mut PWSTR,
) -> BOOL {
    return SetupDiGetActualSectionToInstallExW(
        InfHandle,
        InfSectionName,
        core::ptr::null_mut(),
        InfSectionWithExt,
        InfSectionWithExtSize,
        RequiredSize,
        Extension,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetActualSectionToInstallExA(
    InfHandle: HINF,
    InfSectionName: PCSTR,
    AlternatePlatformInfo: PSP_ALTPLATFORM_INFO,
    InfSectionWithExt: PSTR,
    InfSectionWithExtSize: DWORD,
    RequiredSize: PDWORD,
    Extension: *mut PSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut InfSectionNameW: LPWSTR = core::ptr::null_mut();
    let mut InfSectionWithExtW: LPWSTR = core::ptr::null_mut();
    let mut ExtensionW: PWSTR = core::ptr::null_mut();
    let mut bResult: BOOL = (FALSE as i32);

    'cleanup: {
        if !InfSectionName.is_null() {
            InfSectionNameW = pSetupMultiByteToUnicode(InfSectionName, CP_ACP as UINT) as LPWSTR;
            if InfSectionNameW.is_null() {
                break 'cleanup;
            }
        }
        if !InfSectionWithExt.is_null() {
            InfSectionWithExtW = MyMalloc(
                (InfSectionWithExtSize as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD
            ) as LPWSTR;
            if InfSectionWithExtW.is_null() {
                break 'cleanup;
            }
        }

        bResult = SetupDiGetActualSectionToInstallExW(
            InfHandle,
            InfSectionNameW as PCWSTR,
            AlternatePlatformInfo,
            if !InfSectionWithExt.is_null() {
                InfSectionWithExtW as PWSTR
            } else {
                core::ptr::null_mut::<WCHAR>()
            },
            InfSectionWithExtSize,
            RequiredSize,
            if !Extension.is_null() {
                core::ptr::addr_of_mut!(ExtensionW)
            } else {
                core::ptr::null_mut::<PWSTR>()
            },
            Reserved,
        );

        if bResult != 0 && !InfSectionWithExt.is_null() {
            bResult = (WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                InfSectionWithExtW as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                InfSectionWithExt as LPSTR,
                InfSectionWithExtSize as ::core::ffi::c_int,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            ) != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as BOOL;
        }

        if bResult != 0 && !Extension.is_null() {
            if ExtensionW.is_null() {
                *Extension = core::ptr::null_mut::<CHAR>();
            } else {
                *Extension = InfSectionWithExt
                    .offset(ExtensionW.offset_from(InfSectionWithExtW) as ::core::ffi::c_longlong as isize)
                    as PSTR;
            }
        }
    }

    MyFree(InfSectionNameW as PVOID);
    MyFree(InfSectionWithExtW as PVOID);

    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassDescriptionA(
    ClassGuid: *const GUID,
    ClassDescription: PSTR,
    ClassDescriptionSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    return SetupDiGetClassDescriptionExA(
        ClassGuid,
        ClassDescription,
        ClassDescriptionSize,
        RequiredSize,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassDescriptionW(
    ClassGuid: *const GUID,
    ClassDescription: PWSTR,
    ClassDescriptionSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    return SetupDiGetClassDescriptionExW(
        ClassGuid,
        ClassDescription,
        ClassDescriptionSize,
        RequiredSize,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassDescriptionExA(
    ClassGuid: *const GUID,
    ClassDescription: PSTR,
    ClassDescriptionSize: DWORD,
    RequiredSize: PDWORD,
    MachineName: PCSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut ClassDescriptionW: PWCHAR = core::ptr::null_mut::<WCHAR>();
    let mut MachineNameW: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ret: BOOL = FALSE as BOOL;

    'cleanup: {
        if ClassDescriptionSize > (0 as DWORD) {
            ClassDescriptionW = MyMalloc(
                (ClassDescriptionSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            ) as PWCHAR;
            if ClassDescriptionW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                break 'cleanup;
            }
        }

        if !MachineName.is_null() {
            MachineNameW = pSetupMultiByteToUnicode(MachineName, CP_ACP as UINT) as LPWSTR;
            if MachineNameW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                break 'cleanup;
            }
        }

        ret = SetupDiGetClassDescriptionExW(
            ClassGuid,
            ClassDescriptionW as PWSTR,
            (ClassDescriptionSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            RequiredSize,
            MachineNameW as PCWSTR,
            Reserved,
        );
        if ret != 0 {
            let mut len: DWORD = WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                ClassDescriptionW as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                ClassDescription as LPSTR,
                ClassDescriptionSize as ::core::ffi::c_int,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            ) as DWORD;
            if len == (0 as DWORD) || len > ClassDescriptionSize {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                ret = FALSE as BOOL;
            }
        }
    }

    MyFree(ClassDescriptionW as PVOID);
    MyFree(MachineNameW as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassDescriptionExW(
    ClassGuid: *const GUID,
    ClassDescription: PWSTR,
    ClassDescriptionSize: DWORD,
    RequiredSize: PDWORD,
    MachineName: PCWSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut hKey: HKEY;
    let mut dwLength: DWORD;
    let mut dwRegType: DWORD;
    let mut rc: LONG;
    let mut Buffer: PWSTR;

    /* Make sure there's a GUID */
    if ClassGuid.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    /* Make sure there's a real buffer when there's a size */
    if ClassDescription.is_null() && ClassDescriptionSize > 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    /* Open the key for the GUID */
    hKey = SetupDiOpenClassRegKeyExW(
        ClassGuid,
        KEY_QUERY_VALUE,
        DIOCR_INSTALLER,
        MachineName,
        Reserved,
    );
    if hKey == (INVALID_HANDLE_VALUE).cast() {
        return (FALSE) as BOOL;
    }

    /* Retrieve the class description data and close the key */
    Buffer = core::ptr::null_mut();
    dwRegType = 0;
    dwLength = 0;
    rc = QueryRegistryValue(
        hKey,
        core::ptr::null(),
        core::ptr::addr_of_mut!(Buffer) as *mut LPBYTE,
        core::ptr::addr_of_mut!(dwRegType),
        core::ptr::addr_of_mut!(dwLength),
    );
    RegCloseKey(hKey);

    /* Make sure we got the data */
    if rc != (ERROR_SUCCESS as LONG) {
        SetLastError((rc) as DWORD);
        return (FALSE) as BOOL;
    }

    /* Make sure the data is a string */
    if dwRegType != REG_SZ {
        MyFree((Buffer).cast());
        SetLastError(ERROR_GEN_FAILURE);
        return (FALSE) as BOOL;
    }

    /* Determine the length of the class description */
    dwLength /= core::mem::size_of::<WCHAR>() as DWORD;

    /* Count the null-terminator if none is present */
    if (dwLength == 0)
        || (*Buffer.offset(dwLength.wrapping_sub(1) as isize)
            != UNICODE_NULL)
    {
        dwLength = dwLength.wrapping_add(1);
    }

    /* Inform the caller about the class description */
    if (!ClassDescription.is_null()) && (dwLength <= ClassDescriptionSize) {
        memcpy(
            ClassDescription as *mut core::ffi::c_void,
            Buffer as *const core::ffi::c_void,
            (dwLength.wrapping_sub(1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        *ClassDescription.offset(dwLength.wrapping_sub(1) as isize) =
            UNICODE_NULL;
    }

    /* Inform the caller about the required size */
    if !RequiredSize.is_null() {
        *RequiredSize = dwLength;
    }

    /* Clean up the buffer */
    MyFree((Buffer).cast());

    /* Make sure the buffer was large enough */
    if ClassDescription.is_null() || (dwLength > ClassDescriptionSize) {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return (FALSE) as BOOL;
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassDevsA(
    class: *const GUID,
    enumstr: LPCSTR,
    parent: HWND,
    flags: DWORD,
) -> HDEVINFO {
    return SetupDiGetClassDevsExA(
        class,
        enumstr as PCSTR,
        parent,
        flags,
        core::ptr::null_mut(),
        core::ptr::null(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassDevsExA(
    class: *const GUID,
    enumstr: PCSTR,
    parent: HWND,
    flags: DWORD,
    deviceset: HDEVINFO,
    machine: PCSTR,
    reserved: PVOID,
) -> HDEVINFO {
    let mut ret: HDEVINFO = core::ptr::null_mut();
    let mut enumstrW: LPWSTR = core::ptr::null_mut();
    let mut machineW: LPWSTR = core::ptr::null_mut();

    'end: {
        if !enumstr.is_null() {
            enumstrW = pSetupMultiByteToUnicode(enumstr, CP_ACP as UINT) as LPWSTR;
            if enumstrW.is_null() {
                ret = INVALID_HANDLE_VALUE as HDEVINFO;
                break 'end;
            }
        }
        if !machine.is_null() {
            machineW = pSetupMultiByteToUnicode(machine, CP_ACP as UINT) as LPWSTR;
            if machineW.is_null() {
                MyFree(enumstrW as PVOID);
                ret = INVALID_HANDLE_VALUE as HDEVINFO;
                break 'end;
            }
        }
        ret = SetupDiGetClassDevsExW(
            class,
            enumstrW as PCWSTR,
            parent,
            flags,
            deviceset,
            machineW as PCWSTR,
            reserved,
        );
        MyFree(enumstrW as PVOID);
        MyFree(machineW as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassDevsW(
    class: *const GUID,
    enumstr: LPCWSTR,
    parent: HWND,
    flags: DWORD,
) -> HDEVINFO {
    return SetupDiGetClassDevsExW(
        class,
        enumstr,
        parent,
        flags,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassDevsExW(
    class: *const GUID,
    enumstr: PCWSTR,
    parent: HWND,
    flags: DWORD,
    deviceset: HDEVINFO,
    machine: PCWSTR,
    reserved: PVOID,
) -> HDEVINFO {
    let mut hDeviceInfo: HDEVINFO = INVALID_HANDLE_VALUE;
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut pClassGuid: *const GUID = core::ptr::null();
    let mut rc: LONG = 0;
    let mut set: HDEVINFO = INVALID_HANDLE_VALUE;

    TRACE!(
        b"%s(%s %s %p 0x%08x %p %s %p)\n\0".as_ptr() as *const i8,
        b"SetupDiGetClassDevsExW\0".as_ptr() as *const i8,
        debugstr_guid(class),
        debugstr_w(enumstr),
        parent,
        flags,
        deviceset,
        debugstr_w(machine),
        reserved,
    );

    if (flags & DIGCF_ALLCLASSES) == 0 && class.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }

    'cleanup: {
        /* Create the deviceset if not set */
        if !deviceset.is_null() {
            list = deviceset as *mut DeviceInfoSet;
            if (*list).magic != SETUP_DEVICE_INFO_SET_MAGIC {
                SetLastError(ERROR_INVALID_HANDLE);
                break 'cleanup;
            }
            hDeviceInfo = deviceset;
        } else {
            hDeviceInfo = SetupDiCreateDeviceInfoListExW(
                if (flags & (DIGCF_DEVICEINTERFACE | DIGCF_ALLCLASSES)) != 0 {
                    core::ptr::null()
                } else {
                    class
                },
                core::ptr::null_mut(),
                machine,
                core::ptr::null_mut(),
            );
            if hDeviceInfo == INVALID_HANDLE_VALUE {
                break 'cleanup;
            }
            list = hDeviceInfo as *mut DeviceInfoSet;
        }

        if (flags & DIGCF_PROFILE) != 0 {
            FIXME!(b": flag DIGCF_PROFILE ignored\n\0".as_ptr() as *const i8);
        }

        if (flags & DIGCF_DEVICEINTERFACE) != 0 {
            if class.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER);
                break 'cleanup;
            }
            rc = SETUP_CreateInterfaceList(
                list,
                machine,
                class,
                enumstr,
                (flags & DIGCF_PRESENT) as BOOL,
            );
        } else {
            /* Determine which class(es) should be included in the deviceset */
            if (flags & DIGCF_ALLCLASSES) != 0 {
                /* The caller wants all classes. Check if
                 * the deviceset limits us to one class */
                if IsEqualIID(
                    core::ptr::addr_of_mut!((*list).ClassGuid),
                    core::ptr::addr_of!(GUID_NULL),
                )
                {
                    pClassGuid = core::ptr::null();
                } else {
                    pClassGuid = core::ptr::addr_of_mut!((*list).ClassGuid);
                }
            } else if !class.is_null() {
                /* The caller wants one class. Check if it matches deviceset class */
                if IsEqualIID(core::ptr::addr_of_mut!((*list).ClassGuid), class)
                    || IsEqualIID(
                        core::ptr::addr_of_mut!((*list).ClassGuid),
                        core::ptr::addr_of!(GUID_NULL),
                    )
                {
                    pClassGuid = class;
                } else {
                    SetLastError(ERROR_INVALID_PARAMETER);
                    break 'cleanup;
                }
            } else if !IsEqualIID(
                core::ptr::addr_of_mut!((*list).ClassGuid),
                core::ptr::addr_of!(GUID_NULL),
            )
            {
                /* No class specified. Try to use the one of the deviceset */
                if IsEqualIID(
                    core::ptr::addr_of_mut!((*list).ClassGuid),
                    core::ptr::addr_of!(GUID_NULL),
                )
                {
                    pClassGuid = core::ptr::addr_of_mut!((*list).ClassGuid);
                } else {
                    SetLastError(ERROR_INVALID_PARAMETER);
                    break 'cleanup;
                }
            } else {
                SetLastError(ERROR_INVALID_PARAMETER);
                break 'cleanup;
            }
            rc = SETUP_CreateDevicesList(list, machine, pClassGuid, enumstr);
        }
        if rc != (ERROR_SUCCESS as LONG) {
            SetLastError((rc) as DWORD);
            break 'cleanup;
        }
        set = hDeviceInfo;
    }

    // cleanup:
    if deviceset.is_null() && hDeviceInfo != INVALID_HANDLE_VALUE && hDeviceInfo != set {
        SetupDiDestroyDeviceInfoList(hDeviceInfo);
    }
    return set;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDeviceInfoListDetailA(
    DeviceInfoSet: HDEVINFO,
    DevInfoData: PSP_DEVINFO_LIST_DETAIL_DATA_A,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    TRACE!("%s(%p %p)\n", "SetupDiGetDeviceInfoListDetailA", DeviceInfoSet, DevInfoData);
    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    if DevInfoData.is_null()
        || (*DevInfoData).cbSize as usize
            != ::core::mem::size_of::<SP_DEVINFO_LIST_DETAIL_DATA_A>() as usize
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    memcpy(
        &raw mut (*DevInfoData).ClassGuid as *mut ::core::ffi::c_void,
        &raw mut (*set).ClassGuid as *const ::core::ffi::c_void,
        ::core::mem::size_of::<GUID>() as size_t,
    );
    (*DevInfoData).RemoteMachineHandle = (*set).hMachine as HANDLE;
    if !(*set).MachineName.is_null() {
        FIXME!("Stub\n");
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
        return (FALSE) as BOOL;
    } else {
        (*DevInfoData).RemoteMachineName[0 as usize] = 0 as CHAR;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDeviceInfoListDetailW(
    mut DeviceInfoSet: HDEVINFO,
    mut DevInfoData: PSP_DEVINFO_LIST_DETAIL_DATA_W,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }
    if DevInfoData.is_null()
        || (*DevInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_LIST_DETAIL_DATA_W>() as usize
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    memcpy(
        core::ptr::addr_of_mut!((*DevInfoData).ClassGuid) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*set).ClassGuid) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as size_t,
    );
    (*DevInfoData).RemoteMachineHandle = (*set).hMachine;
    if !(*set).MachineName.is_null() {
        strcpyW(
            core::ptr::addr_of_mut!((*DevInfoData).RemoteMachineName).cast(),
            (*set).MachineName.add(2 as usize),
        );
    } else {
        (*DevInfoData).RemoteMachineName[0 as usize] = 0;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiCreateDeviceInterfaceA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    InterfaceClassGuid: *const GUID,
    ReferenceString: PCSTR,
    CreationFlags: DWORD,
    DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut ReferenceStringW: LPWSTR = core::ptr::null_mut();

    if !ReferenceString.is_null() {
        ReferenceStringW = pSetupMultiByteToUnicode(ReferenceString, CP_ACP) as LPWSTR;
        if ReferenceStringW.is_null() {
            return (FALSE) as BOOL;
        }
    }

    ret = SetupDiCreateDeviceInterfaceW(
        DeviceInfoSet,
        DeviceInfoData,
        InterfaceClassGuid,
        ReferenceStringW as PCWSTR,
        CreationFlags,
        DeviceInterfaceData,
    );

    MyFree(ReferenceStringW as PVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiCreateDeviceInterfaceW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    InterfaceClassGuid: *const GUID,
    ReferenceString: PCWSTR,
    CreationFlags: DWORD,
    DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
) -> BOOL {
    let set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiCreateDeviceInterfaceW\0" as *const u8 as *const ::core::ffi::c_char,
            2796 as ::core::ffi::c_int,
            b"%s(%p %p %s %s %08x %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiCreateDeviceInterfaceW\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            DeviceInfoData,
            debugstr_guid(InterfaceClassGuid as *const _GUID),
            debugstr_w(ReferenceString as *const WCHAR),
            CreationFlags,
            DeviceInterfaceData,
        );
    }
    if !DeviceInfoSet.is_null() == false || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    if DeviceInfoData.is_null()
        || (*DeviceInfoData).cbSize as usize != ::core::mem::size_of::<SP_DEVINFO_DATA>() as usize
        || (*DeviceInfoData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    if InterfaceClassGuid.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        return (FALSE) as BOOL;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiCreateDeviceInterfaceW\0" as *const u8 as *const ::core::ffi::c_char,
            2822 as ::core::ffi::c_int,
            b"%p %p %s %s %08x %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            DeviceInfoData,
            debugstr_guid(InterfaceClassGuid as *const _GUID),
            debugstr_w(ReferenceString as *const WCHAR),
            CreationFlags,
            DeviceInterfaceData,
        );
    }
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiCreateDeviceInterfaceRegKeyA(
    DeviceInfoSet: HDEVINFO,
    DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
    Reserved: DWORD,
    samDesired: REGSAM,
    InfHandle: HINF,
    InfSectionName: PCSTR,
) -> HKEY {
    let mut key: HKEY;
    let mut InfSectionNameW: PWSTR = core::ptr::null_mut();

    if !InfHandle.is_null() {
        if InfSectionName.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return INVALID_HANDLE_VALUE as HKEY;
        }
        InfSectionNameW = pSetupMultiByteToUnicode(InfSectionName, CP_ACP as UINT);
        if InfSectionNameW.is_null() {
            return INVALID_HANDLE_VALUE as HKEY;
        }
    }
    key = SetupDiCreateDeviceInterfaceRegKeyW(
        DeviceInfoSet,
        DeviceInterfaceData,
        Reserved,
        samDesired,
        InfHandle,
        InfSectionNameW as PCWSTR,
    );
    MyFree(InfSectionNameW as PVOID);
    return key;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiCreateDeviceInterfaceRegKeyW(
    DeviceInfoSet: HDEVINFO,
    DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
    Reserved: DWORD,
    samDesired: REGSAM,
    InfHandle: HINF,
    InfSectionName: PCWSTR,
) -> HKEY {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hDevKey: HKEY = core::ptr::null_mut();
    let mut hRefKey: HKEY = core::ptr::null_mut();
    let mut hDevParamKey: HKEY = core::ptr::null_mut();
    let mut SymbolicLink: LPWSTR = core::ptr::null_mut();
    let mut ReferenceString: LPWSTR = core::ptr::null_mut();
    let mut Length: DWORD = 0;
    let mut RefLength: DWORD = 0;
    let mut Index: DWORD = 0;
    let mut rc: LONG = 0;
    let mut bracedGuidString: [WCHAR; 39] = [0 as WCHAR; 39];
    let mut DevItf: *mut DeviceInterface = core::ptr::null_mut();
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;

    if DeviceInfoSet.is_null()
        || DeviceInfoSet == INVALID_HANDLE_VALUE as HDEVINFO
        || (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD
    {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return INVALID_HANDLE_VALUE as HKEY;
    }
    if DeviceInterfaceData.is_null()
        || (*DeviceInterfaceData).cbSize as usize
            != core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as usize
        || (*DeviceInterfaceData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return INVALID_HANDLE_VALUE as HKEY;
    }
    if !InfHandle.is_null() && InfSectionName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return INVALID_HANDLE_VALUE as HKEY;
    }

    hKey = SetupDiOpenClassRegKeyExW(
        core::ptr::addr_of_mut!((*DeviceInterfaceData).InterfaceClassGuid),
        samDesired,
        DIOCR_INTERFACE as DWORD,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    if hKey == INVALID_HANDLE_VALUE as HKEY {
        hKey = SetupDiOpenClassRegKeyExW(
            core::ptr::null(),
            samDesired,
            DIOCR_INTERFACE as DWORD,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if hKey == INVALID_HANDLE_VALUE as HKEY {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return INVALID_HANDLE_VALUE as HKEY;
        }
        SETUPDI_GuidToString(
            core::ptr::addr_of_mut!((*DeviceInterfaceData).InterfaceClassGuid),
            bracedGuidString.as_mut_ptr(),
        );

        if RegCreateKeyExW(
            hKey,
            bracedGuidString.as_mut_ptr() as LPCWSTR,
            0 as DWORD,
            core::ptr::null_mut(),
            0 as DWORD,
            samDesired,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hDevKey),
            core::ptr::null_mut(),
        ) != ERROR_SUCCESS as LONG
        {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return INVALID_HANDLE_VALUE as HKEY;
        }
        RegCloseKey(hKey);
        hKey = hDevKey;
    }

    DevItf = (*DeviceInterfaceData).Reserved as *mut DeviceInterface;

    Length = ((wcslen(core::ptr::addr_of_mut!((*DevItf).SymbolicLink) as *mut WCHAR) as DWORD)
        + 1 as DWORD)
        * core::mem::size_of::<WCHAR>() as DWORD;
    SymbolicLink = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        Length as SIZE_T,
    ) as LPWSTR;
    if SymbolicLink.is_null() {
        RegCloseKey(hKey);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return INVALID_HANDLE_VALUE as HKEY;
    }

    wcscpy(
        SymbolicLink as *mut WCHAR,
        core::ptr::addr_of_mut!((*DevItf).SymbolicLink) as *mut WCHAR,
    );

    /* Enumerate all characters in symbolic link */
    Index = 0 as DWORD;
    while *SymbolicLink.offset(Index as isize) != 0 {
        /* Check for a start position of reference string */
        if *SymbolicLink.offset(Index as isize) == 0x007Du16
            && *SymbolicLink.offset(Index.wrapping_add(1 as DWORD) as isize)
                == 0x005Cu16
        {
            /* Found it */
            *SymbolicLink.offset(Index.wrapping_add(1 as DWORD) as isize) =
                0x0023u16;
            break;
        } else {
            /* Replace all '\' backslashes by '#' pounds in symbolic link */
            if *SymbolicLink.offset(Index as isize) == 0x005Cu16 {
                *SymbolicLink.offset(Index as isize) = 0x0023u16;
            }
            Index = Index.wrapping_add(1);
        }
    }

    /* Create reference string */
    RefLength = (Length as usize)
        .wrapping_sub((Index as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize))
        as DWORD;
    ReferenceString = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        RefLength as SIZE_T,
    ) as LPWSTR;
    if ReferenceString.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            SymbolicLink as PVOID,
        );
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return INVALID_HANDLE_VALUE as HKEY;
    }

    wcscpy(
        ReferenceString as *mut WCHAR,
        SymbolicLink.offset(Index.wrapping_add(1 as DWORD) as isize) as *mut WCHAR,
    );

    /* Null-terminate symbolic link at the beginning of the reference part,
     * as we don't need a ref part in key name. */
    *SymbolicLink.offset(Index.wrapping_add(1 as DWORD) as isize) =
        UNICODE_NULL as WCHAR;

    /* Create device instance key */
    rc = RegCreateKeyExW(
        hKey,
        SymbolicLink as LPCWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        samDesired,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hDevKey),
        core::ptr::null_mut(),
    ) as LONG;
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        SymbolicLink as PVOID,
    );
    RegCloseKey(hKey);
    if rc != ERROR_SUCCESS as LONG {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            ReferenceString as PVOID,
        );
        SetLastError(rc as DWORD);
        return INVALID_HANDLE_VALUE as HKEY;
    }

    /* Create reference key */
    rc = RegCreateKeyExW(
        hDevKey,
        ReferenceString as LPCWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        samDesired,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hRefKey),
        core::ptr::null_mut(),
    ) as LONG;
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        ReferenceString as PVOID,
    );
    RegCloseKey(hDevKey);
    if rc != ERROR_SUCCESS as LONG {
        SetLastError(rc as DWORD);
        return INVALID_HANDLE_VALUE as HKEY;
    }

    /* Create/open "Device Parameters" subkey */
    rc = RegCreateKeyExW(
        hRefKey,
        RUSTCONV_WSTR_00A6C7A05271.as_ptr(),
        0 as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        samDesired,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hDevParamKey),
        core::ptr::null_mut(),
    ) as LONG;
    RegCloseKey(hRefKey);
    if rc == ERROR_SUCCESS as LONG {
        if !InfHandle.is_null() && !InfSectionName.is_null() {
            if SetupInstallFromInfSectionW(
                core::ptr::null_mut(), /*FIXME */
                InfHandle,
                InfSectionName,
                (SPINST_INIFILES
                    | SPINST_REGISTRY
                    | SPINST_INI2REG
                    | SPINST_FILES
                    | SPINST_BITREG
                    | SPINST_REGSVR
                    | SPINST_UNREGSVR
                    | SPINST_PROFILEITEMS
                    | (SPINST_COPYINF as u32)) as UINT,
                hDevParamKey,
                core::ptr::null(),
                0 as UINT,
                (*(*set).SelectedDevice).InstallParams.InstallMsgHandler,
                (*(*set).SelectedDevice).InstallParams.InstallMsgHandlerContext,
                INVALID_HANDLE_VALUE as HINF,
                core::ptr::null_mut(),
            ) == 0
            {
                RegCloseKey(hDevParamKey);
                return INVALID_HANDLE_VALUE as HKEY;
            }
        }
    }

    SetLastError(rc as DWORD);
    return hDevParamKey;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiDeleteDeviceInterfaceRegKey(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
    mut Reserved: DWORD,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut ret: BOOL = (FALSE as i32);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiDeleteDeviceInterfaceRegKey\0" as *const u8 as *const ::core::ffi::c_char,
            3033 as ::core::ffi::c_int,
            b"%s(%p %p %d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiDeleteDeviceInterfaceRegKey\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            DeviceInterfaceData,
            Reserved,
        );
    }
    if DeviceInfoSet.is_null()
        || DeviceInfoSet == INVALID_HANDLE_VALUE
        || (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD
    {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    if DeviceInterfaceData.is_null()
        || (*DeviceInterfaceData).cbSize as usize
            != ::core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as usize
        || (*DeviceInterfaceData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiDeleteDeviceInterfaceRegKey\0" as *const u8 as *const ::core::ffi::c_char,
            3049 as ::core::ffi::c_int,
            b"%p %p %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            DeviceInterfaceData,
            Reserved,
        );
    }
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiEnumDeviceInterfaces(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
    mut InterfaceClassGuid: *const GUID,
    mut MemberIndex: DWORD,
    mut DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut ret: BOOL = (FALSE as i32);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiEnumDeviceInterfaces\0" as *const u8 as *const ::core::ffi::c_char,
            3088 as ::core::ffi::c_int,
            b"%s(%p, %p, %s, %d, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiEnumDeviceInterfaces\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            DeviceInfoData,
            debugstr_guid(InterfaceClassGuid as *const _GUID),
            MemberIndex,
            DeviceInterfaceData,
        );
    }
    if DeviceInfoSet.is_null()
        || DeviceInfoSet == INVALID_HANDLE_VALUE
        || (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC
    {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    if !DeviceInfoData.is_null()
        && ((*DeviceInfoData).cbSize as usize != ::core::mem::size_of::<SP_DEVINFO_DATA>() as usize
            || (*DeviceInfoData).Reserved == 0)
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    if DeviceInterfaceData.is_null()
        || (*DeviceInterfaceData).cbSize as usize
            != core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as usize
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    if !DeviceInfoData.is_null() {
        let mut devInfo: *mut DeviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
        let mut found: BOOL = (FALSE as i32);
        let mut InterfaceListEntry: PLIST_ENTRY = (*devInfo).InterfaceListHead.Flink as PLIST_ENTRY;
        while InterfaceListEntry != &raw mut (*devInfo).InterfaceListHead && found == 0 {
            let mut DevItf: *mut DeviceInterface = (InterfaceListEntry as ULONG_PTR).wrapping_sub(
                core::mem::offset_of!(DeviceInterface, ListEntry) as ULONG_PTR,
            ) as *mut DeviceInterface;
            if !IsEqualIID(
                &raw mut (*DevItf).InterfaceClassGuid,
                InterfaceClassGuid,
            )
            {
                InterfaceListEntry = (*InterfaceListEntry).Flink as PLIST_ENTRY;
                continue;
            }
            let fresh19 = MemberIndex;
            MemberIndex = MemberIndex.wrapping_sub(1);
            if fresh19 == 0 as DWORD {
                memcpy(
                    &raw mut (*DeviceInterfaceData).InterfaceClassGuid as *mut ::core::ffi::c_void,
                    &raw mut (*DevItf).InterfaceClassGuid as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<GUID>() as size_t,
                );
                (*DeviceInterfaceData).Flags = (*DevItf).Flags;
                (*DeviceInterfaceData).Reserved = DevItf as ULONG_PTR;
                found = (TRUE as i32);
                ret = (TRUE as i32);
            }
            InterfaceListEntry = (*InterfaceListEntry).Flink as PLIST_ENTRY;
        }
        if found == 0 {
            SetLastError(ERROR_NO_MORE_ITEMS as DWORD);
        }
    } else {
        let mut found_0: BOOL = (FALSE as i32);
        let mut ItemList: PLIST_ENTRY = (*set).ListHead.Flink as PLIST_ENTRY;
        while ItemList != &raw mut (*set).ListHead && found_0 == 0 {
            let mut InterfaceListEntry_0: PLIST_ENTRY = ::core::ptr::null_mut::<_LIST_ENTRY>();
            let mut devInfo_0: *mut DeviceInfo = (ItemList as ULONG_PTR).wrapping_sub(
                core::mem::offset_of!(DeviceInfo, ListEntry) as ULONG_PTR,
            ) as *mut DeviceInfo;
            InterfaceListEntry_0 = (*devInfo_0).InterfaceListHead.Flink as PLIST_ENTRY;
            while InterfaceListEntry_0 != &raw mut (*devInfo_0).InterfaceListHead && found_0 == 0 {
                let mut DevItf_0: *mut DeviceInterface =
                    (InterfaceListEntry_0 as ULONG_PTR).wrapping_sub(
                        core::mem::offset_of!(DeviceInterface, ListEntry)
                            as ULONG_PTR,
                    ) as *mut DeviceInterface;
                if !IsEqualIID(
                    &raw mut (*DevItf_0).InterfaceClassGuid,
                    InterfaceClassGuid,
                )
                {
                    InterfaceListEntry_0 = (*InterfaceListEntry_0).Flink as PLIST_ENTRY;
                    continue;
                }
                let fresh20 = MemberIndex;
                MemberIndex = MemberIndex.wrapping_sub(1);
                if fresh20 == 0 as DWORD {
                    memcpy(
                        &raw mut (*DeviceInterfaceData).InterfaceClassGuid
                            as *mut ::core::ffi::c_void,
                        &raw mut (*DevItf_0).InterfaceClassGuid as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<GUID>() as size_t,
                    );
                    (*DeviceInterfaceData).Flags = (*DevItf_0).Flags;
                    (*DeviceInterfaceData).Reserved = DevItf_0 as ULONG_PTR;
                    found_0 = (TRUE as i32);
                    ret = (TRUE as i32);
                }
                InterfaceListEntry_0 = (*InterfaceListEntry_0).Flink as PLIST_ENTRY;
            }
            ItemList = (*ItemList).Flink as PLIST_ENTRY;
        }
        if found_0 == 0 {
            SetLastError(ERROR_NO_MORE_ITEMS as DWORD);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiDestroyDeviceInfoList(devinfo: HDEVINFO) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiDestroyDeviceInfoList\0" as *const u8 as *const ::core::ffi::c_char,
            3194 as ::core::ffi::c_int,
            b"%s(%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiDestroyDeviceInfoList\0" as *const u8 as *const ::core::ffi::c_char,
            devinfo,
        );
    }
    if !devinfo.is_null() && devinfo != INVALID_HANDLE_VALUE {
        let mut list: *mut DeviceInfoSet = devinfo as *mut DeviceInfoSet;
        if (*list).magic == SETUP_DEVICE_INFO_SET_MAGIC {
            ret = DestroyDeviceInfoSet(list);
        }
    }
    if ret == (FALSE as BOOL) {
        SetLastError(ERROR_INVALID_HANDLE);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDeviceInterfaceDetailA(
    DeviceInfoSet: HDEVINFO,
    DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
    DeviceInterfaceDetailData: PSP_DEVICE_INTERFACE_DETAIL_DATA_A,
    DeviceInterfaceDetailDataSize: DWORD,
    RequiredSize: PDWORD,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let set = DeviceInfoSet as *mut DeviceInfoSet;
    let mut DeviceInterfaceDetailDataW: PSP_DEVICE_INTERFACE_DETAIL_DATA_W = core::ptr::null_mut();
    let mut sizeW: DWORD = 0;
    let mut bytesNeeded: DWORD = 0;
    let mut ret: BOOL = (FALSE as i32);

    if DeviceInfoSet.is_null()
        || DeviceInfoSet == INVALID_HANDLE_VALUE
        || (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD
    {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }
    if DeviceInterfaceData.is_null()
        || (*DeviceInterfaceData).cbSize as usize
            != core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as usize
        || (*DeviceInterfaceData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    if !DeviceInterfaceDetailData.is_null()
        && (*DeviceInterfaceDetailData).cbSize as usize
            != core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_A>() as usize
    {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return (FALSE) as BOOL;
    }

    if DeviceInterfaceDetailDataSize != 0
        && (DeviceInterfaceDetailDataSize as usize)
            < (FIELD_OFFSET!(SP_DEVICE_INTERFACE_DETAIL_DATA_A, DevicePath) as usize)
                .wrapping_add(core::mem::size_of::<CHAR>() as usize)
    {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return (FALSE) as BOOL;
    }

    if DeviceInterfaceDetailData.is_null() && DeviceInterfaceDetailDataSize != 0 {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return (FALSE) as BOOL;
    }

    if !DeviceInterfaceDetailData.is_null() {
        sizeW = (FIELD_OFFSET!(SP_DEVICE_INTERFACE_DETAIL_DATA_W, DevicePath) as DWORD).wrapping_add(
            DeviceInterfaceDetailDataSize
                .wrapping_sub(FIELD_OFFSET!(SP_DEVICE_INTERFACE_DETAIL_DATA_A, DevicePath) as DWORD)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
        );
        DeviceInterfaceDetailDataW = MyMalloc(sizeW) as PSP_DEVICE_INTERFACE_DETAIL_DATA_W;
        if DeviceInterfaceDetailDataW.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        }
        (*DeviceInterfaceDetailDataW).cbSize =
            core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as DWORD;
    }
    if DeviceInterfaceDetailData.is_null()
        || (!DeviceInterfaceDetailData.is_null() && !DeviceInterfaceDetailDataW.is_null())
    {
        ret = SetupDiGetDeviceInterfaceDetailW(
            DeviceInfoSet,
            DeviceInterfaceData,
            DeviceInterfaceDetailDataW,
            sizeW,
            core::ptr::addr_of_mut!(sizeW),
            DeviceInfoData,
        );
        bytesNeeded = (sizeW.wrapping_sub(FIELD_OFFSET!(SP_DEVICE_INTERFACE_DETAIL_DATA_W, DevicePath) as DWORD))
            / (core::mem::size_of::<WCHAR>() as DWORD)
            + FIELD_OFFSET!(SP_DEVICE_INTERFACE_DETAIL_DATA_A, DevicePath) as DWORD;
        if !RequiredSize.is_null() {
            *RequiredSize = bytesNeeded;
        }
        if ret != 0
            && !DeviceInterfaceDetailData.is_null()
            && DeviceInterfaceDetailDataSize >= bytesNeeded
        {
            if WideCharToMultiByte(
                CP_ACP,
                0,
                core::ptr::addr_of_mut!((*DeviceInterfaceDetailDataW).DevicePath) as *mut WCHAR as LPCWSTR,
                -1,
                core::ptr::addr_of_mut!((*DeviceInterfaceDetailData).DevicePath) as *mut CHAR as LPSTR,
                (DeviceInterfaceDetailDataSize
                    .wrapping_sub(FIELD_OFFSET!(SP_DEVICE_INTERFACE_DETAIL_DATA_A, DevicePath) as DWORD)) as i32,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            ) == 0
            {
                ret = (FALSE as i32);
            }
        }
    }
    MyFree(DeviceInterfaceDetailDataW as PVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDeviceInterfaceDetailW(
    DeviceInfoSet: HDEVINFO,
    DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
    DeviceInterfaceDetailData: PSP_DEVICE_INTERFACE_DETAIL_DATA_W,
    DeviceInterfaceDetailDataSize: DWORD,
    RequiredSize: PDWORD,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut ret: BOOL = (FALSE as i32);

    if DeviceInfoSet.is_null()
        || DeviceInfoSet == INVALID_HANDLE_VALUE
        || (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD
    {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    if DeviceInterfaceData.is_null()
        || (*DeviceInterfaceData).cbSize as usize
            != core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>()
        || (*DeviceInterfaceData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    if !DeviceInterfaceDetailData.is_null()
        && (*DeviceInterfaceDetailData).cbSize as usize
            != core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>()
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        return (FALSE) as BOOL;
    }
    if DeviceInterfaceDetailData.is_null() && DeviceInterfaceDetailDataSize != 0 {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        return (FALSE) as BOOL;
    }
    if !DeviceInfoData.is_null()
        && (*DeviceInfoData).cbSize as usize
            != core::mem::size_of::<SP_DEVINFO_DATA>()
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    if !DeviceInterfaceDetailData.is_null()
        && (DeviceInterfaceDetailDataSize as usize)
            < core::mem::offset_of!(SP_DEVICE_INTERFACE_DETAIL_DATA_W, DevicePath)
                .wrapping_add(core::mem::size_of::<WCHAR>())
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    } else {
        let deviceInterface: *mut DeviceInterface =
            (*DeviceInterfaceData).Reserved as *mut DeviceInterface;
        let devName: LPCWSTR =
            core::ptr::addr_of_mut!((*deviceInterface).SymbolicLink) as LPCWSTR;
        let sizeRequired: DWORD = (core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>()
            as DWORD)
            .wrapping_add(
                ((lstrlenW(devName) + 1) as DWORD)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
            );

        if sizeRequired > DeviceInterfaceDetailDataSize {
            SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            if !RequiredSize.is_null() {
                *RequiredSize = sizeRequired;
            }
        } else {
            strcpyW(
                core::ptr::addr_of_mut!((*DeviceInterfaceDetailData).DevicePath) as LPWSTR,
                devName,
            );
            if !DeviceInfoData.is_null() {
                memcpy(
                    core::ptr::addr_of_mut!((*DeviceInfoData).ClassGuid) as *mut core::ffi::c_void,
                    core::ptr::addr_of!((*(*deviceInterface).DeviceInfo).ClassGuid)
                        as *const core::ffi::c_void,
                    core::mem::size_of::<GUID>(),
                );
                (*DeviceInfoData).DevInst = (*(*deviceInterface).DeviceInfo).dnDevInst as DWORD;
                (*DeviceInfoData).Reserved = (*deviceInterface).DeviceInfo as ULONG_PTR;
            }
            ret = (TRUE as i32);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDeviceRegistryPropertyA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Property: DWORD,
    PropertyRegDataType: PDWORD,
    PropertyBuffer: PBYTE,
    PropertyBufferSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut bIsStringProperty: BOOL = 0;
    let mut RegType: DWORD = 0;
    let mut RequiredSizeA: DWORD = 0;
    let mut RequiredSizeW: DWORD = 0;
    let mut PropertyBufferSizeW: DWORD = 0;
    let mut PropertyBufferW: PBYTE = core::ptr::null_mut();

    if PropertyBufferSize != 0 {
        PropertyBufferSizeW = PropertyBufferSize * 2;
        PropertyBufferW = HeapAlloc(
            GetProcessHeap(),
            0,
            PropertyBufferSizeW as SIZE_T,
        ) as PBYTE;
        if PropertyBufferW.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return (FALSE) as BOOL;
        }
    }

    ret = SetupDiGetDeviceRegistryPropertyW(
        DeviceInfoSet,
        DeviceInfoData,
        Property,
        core::ptr::addr_of_mut!(RegType),
        PropertyBufferW,
        PropertyBufferSizeW,
        core::ptr::addr_of_mut!(RequiredSizeW),
    );

    if ret != 0 || GetLastError() == ERROR_INSUFFICIENT_BUFFER {
        bIsStringProperty = (RegType == REG_SZ
            || RegType == REG_MULTI_SZ
            || RegType == REG_EXPAND_SZ) as BOOL;

        if bIsStringProperty != 0 {
            RequiredSizeA = RequiredSizeW / (core::mem::size_of::<WCHAR>() as DWORD);
        } else {
            RequiredSizeA = RequiredSizeW;
        }
        if !RequiredSize.is_null() {
            *RequiredSize = RequiredSizeA;
        }
        if !PropertyRegDataType.is_null() {
            *PropertyRegDataType = RegType;
        }
    }

    if ret == 0 {
        HeapFree(GetProcessHeap(), 0, PropertyBufferW as PVOID);
        return ret;
    }

    if RequiredSizeA <= PropertyBufferSize {
        if bIsStringProperty != 0 && PropertyBufferSize > 0 {
            if WideCharToMultiByte(
                CP_ACP,
                0,
                PropertyBufferW as LPWSTR as LPCWSTR,
                (RequiredSizeW / (core::mem::size_of::<WCHAR>() as DWORD)) as core::ffi::c_int,
                PropertyBuffer as LPSTR,
                PropertyBufferSize as core::ffi::c_int,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            ) == 0
            {
                /* Last error is already set by WideCharToMultiByte */
                ret = (FALSE as i32);
            }
        } else {
            memcpy(
                PropertyBuffer as *mut core::ffi::c_void,
                PropertyBufferW as *const core::ffi::c_void,
                RequiredSizeA as usize,
            );
        }
    } else {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        ret = (FALSE as i32);
    }

    HeapFree(GetProcessHeap(), 0, PropertyBufferW as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDeviceRegistryPropertyW(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
    mut Property: DWORD,
    mut PropertyRegDataType: PDWORD,
    mut PropertyBuffer: PBYTE,
    mut PropertyBufferSize: DWORD,
    mut RequiredSize: PDWORD,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut devInfo: *mut DeviceInfo = core::ptr::null_mut::<DeviceInfo>();
    let mut cr: CONFIGRET = 0;
    let mut lError: LONG = ERROR_SUCCESS as LONG;
    let mut size: DWORD = 0;

    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return FALSE as BOOL;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return FALSE as BOOL;
    }
    if DeviceInfoData.is_null()
        || (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
        || (*DeviceInfoData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    if Property >= SPDRP_MAXIMUM_PROPERTY as DWORD {
        SetLastError(ERROR_INVALID_REG_PROPERTY as DWORD);
        return FALSE as BOOL;
    }

    devInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;

    if (Property as usize)
        < (core::mem::size_of_val(&PropertyMap) / core::mem::size_of_val(&PropertyMap[0]))
        && !PropertyMap[Property as usize].nameW.is_null()
    {
        let mut hKey: HKEY = core::ptr::null_mut::<HKEY__>();
        size = PropertyBufferSize;
        hKey = SETUPDI_OpenDevKey((*set).HKLM, devInfo, KEY_QUERY_VALUE as REGSAM);
        if hKey == (INVALID_HANDLE_VALUE).cast() {
            return FALSE as BOOL;
        }
        lError = RegQueryValueExW(
            hKey,
            PropertyMap[Property as usize].nameW,
            core::ptr::null_mut::<DWORD>(),
            PropertyRegDataType as LPDWORD,
            PropertyBuffer as LPBYTE,
            &raw mut size,
        ) as LONG;
        RegCloseKey(hKey);

        if !RequiredSize.is_null() {
            *RequiredSize = size;
        }

        match (lError) as u32 {
            ERROR_SUCCESS => {
                if PropertyBuffer.is_null() && size != 0 as DWORD {
                    lError = ERROR_INSUFFICIENT_BUFFER as LONG;
                }
            }
            ERROR_MORE_DATA => {
                lError = ERROR_INSUFFICIENT_BUFFER as LONG;
            }
            _ => {}
        }
    } else if Property == SPDRP_PHYSICAL_DEVICE_OBJECT_NAME as DWORD {
        size = (strlenW((&raw mut (*devInfo).Data) as *mut WCHAR) as usize)
            .wrapping_add(1 as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD;

        if !PropertyRegDataType.is_null() {
            *PropertyRegDataType = REG_SZ as DWORD;
        }
        if !RequiredSize.is_null() {
            *RequiredSize = size;
        }
        if PropertyBufferSize >= size {
            strcpyW(PropertyBuffer as LPWSTR, (&raw mut (*devInfo).Data) as *mut WCHAR);
        } else {
            lError = ERROR_INSUFFICIENT_BUFFER as LONG;
        }
    } else {
        size = PropertyBufferSize;

        cr = CM_Get_DevNode_Registry_Property_ExW(
            (*devInfo).dnDevInst,
            Property + ((CM_DRP_DEVICEDESC - SPDRP_DEVICEDESC) as DWORD),
            PropertyRegDataType as PULONG,
            PropertyBuffer as PVOID,
            &raw mut size,
            0 as ULONG,
            (*set).hMachine,
        );
        if cr == CR_SUCCESS as CONFIGRET || cr == CR_BUFFER_SMALL as CONFIGRET {
            if !RequiredSize.is_null() {
                *RequiredSize = size;
            }
        }

        if cr != CR_SUCCESS as CONFIGRET {
            match cr {
                CR_INVALID_DEVINST => {
                    lError = ERROR_NO_SUCH_DEVINST as LONG;
                }
                CR_INVALID_PROPERTY => {
                    lError = ERROR_INVALID_REG_PROPERTY as LONG;
                }
                CR_BUFFER_SMALL => {
                    lError = ERROR_INSUFFICIENT_BUFFER as LONG;
                }
                _ => {
                    lError = ERROR_INVALID_DATA as LONG;
                }
            }
        }
    }

    SetLastError(lError as DWORD);
    return (lError == ERROR_SUCCESS as LONG) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn IntSetupDiSetDeviceRegistryPropertyAW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Property: DWORD,
    PropertyBuffer: *const BYTE,
    PropertyBufferSize: DWORD,
    isAnsi: BOOL,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut();

    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }
    if DeviceInfoData.is_null()
        || (*DeviceInfoData).cbSize != core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD
        || (*DeviceInfoData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    deviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;

    if (Property as usize)
        < (core::mem::size_of_val(&PropertyMap)
            / core::mem::size_of_val(&PropertyMap[0]))
        && !PropertyMap[Property as usize].nameW.is_null()
        && !PropertyMap[Property as usize].nameA.is_null()
    {
        let mut hKey: HKEY = core::ptr::null_mut();
        let mut l: LONG = 0;
        hKey = SETUPDI_OpenDevKey((*set).HKLM, deviceInfo, KEY_SET_VALUE);
        if hKey == (INVALID_HANDLE_VALUE).cast() {
            return (FALSE) as BOOL;
        }
        /* Write new data */
        if isAnsi != 0 {
            l = RegSetValueExA(
                hKey,
                PropertyMap[Property as usize].nameA,
                0,
                PropertyMap[Property as usize].regType,
                PropertyBuffer,
                PropertyBufferSize,
            ) as LONG;
        } else {
            l = RegSetValueExW(
                hKey,
                PropertyMap[Property as usize].nameW,
                0,
                PropertyMap[Property as usize].regType,
                PropertyBuffer,
                PropertyBufferSize,
            ) as LONG;
        }
        if l == 0 {
            ret = (TRUE as i32);
        } else {
            SetLastError(l as DWORD);
        }
        RegCloseKey(hKey);
    } else {
        ERR!("Property 0x%lx not implemented\n", Property);
        SetLastError(ERROR_NOT_SUPPORTED);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetDeviceRegistryPropertyA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Property: DWORD,
    PropertyBuffer: *const BYTE,
    PropertyBufferSize: DWORD,
) -> BOOL {
    return IntSetupDiSetDeviceRegistryPropertyAW(
        DeviceInfoSet,
        DeviceInfoData,
        Property,
        PropertyBuffer,
        PropertyBufferSize,
        (TRUE) as BOOL,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetDeviceRegistryPropertyW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Property: DWORD,
    PropertyBuffer: *const BYTE,
    PropertyBufferSize: DWORD,
) -> BOOL {
    return IntSetupDiSetDeviceRegistryPropertyAW(
        DeviceInfoSet,
        DeviceInfoData,
        Property,
        PropertyBuffer,
        PropertyBufferSize,
        (FALSE) as BOOL,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiInstallClassA(
    hwndParent: HWND,
    InfFileName: PCSTR,
    Flags: DWORD,
    FileQueue: HSPFILEQ,
) -> BOOL {
    return SetupDiInstallClassExA(
        hwndParent,
        InfFileName,
        Flags,
        FileQueue,
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiInstallClassExA(
    hwndParent: HWND,
    InfFileName: PCSTR,
    Flags: DWORD,
    FileQueue: HSPFILEQ,
    InterfaceClassGuid: *const GUID,
    Reserved1: PVOID,
    Reserved2: PVOID,
) -> BOOL {
    let mut InfFileNameW: PWSTR = core::ptr::null_mut();
    let mut Result: BOOL;

    if InfFileName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    } else {
        InfFileNameW = pSetupMultiByteToUnicode(InfFileName, CP_ACP as UINT);
        if InfFileNameW.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            return (FALSE) as BOOL;
        }
    }

    Result = SetupDiInstallClassExW(
        hwndParent,
        InfFileNameW as PCWSTR,
        Flags,
        FileQueue,
        InterfaceClassGuid,
        Reserved1,
        Reserved2,
    );

    MyFree(InfFileNameW as PVOID);

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn SETUP_CreateClassKey(hInf: HINF) -> HKEY {
    let mut FullBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut Buffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut RequiredSize: DWORD = 0;
    let mut hClassKey: HKEY = core::ptr::null_mut();
    let mut Disposition: DWORD = 0;

    /* Obtain the Class GUID for this class */
    if SetupGetLineTextW(
        core::ptr::null_mut(),
        hInf,
        core::ptr::addr_of!(Version) as PCWSTR,
        REGSTR_VAL_CLASSGUID,
        core::ptr::addr_of_mut!(Buffer) as PWSTR,
        (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as DWORD,
        core::ptr::addr_of_mut!(RequiredSize),
    ) == 0
    {
        return (INVALID_HANDLE_VALUE).cast();
    }

    /* Build the corresponding registry key name */
    lstrcpyW(core::ptr::addr_of_mut!(FullBuffer) as LPWSTR, REGSTR_PATH_CLASS_NT);
    lstrcatW(core::ptr::addr_of_mut!(FullBuffer) as LPWSTR, core::ptr::addr_of!(BackSlash) as LPCWSTR);
    lstrcatW(core::ptr::addr_of_mut!(FullBuffer) as LPWSTR, core::ptr::addr_of_mut!(Buffer) as LPCWSTR);

    /* Obtain the Class name for this class */
    if SetupGetLineTextW(
        core::ptr::null_mut(),
        hInf,
        core::ptr::addr_of!(Version) as PCWSTR,
        REGSTR_VAL_CLASS,
        core::ptr::addr_of_mut!(Buffer) as PWSTR,
        (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as DWORD,
        core::ptr::addr_of_mut!(RequiredSize),
    ) == 0
    {
        return (INVALID_HANDLE_VALUE).cast();
    }

    /* Try to open or create the registry key */
    if RegCreateKeyExW(
        HKEY_LOCAL_MACHINE,
        core::ptr::addr_of_mut!(FullBuffer) as LPCWSTR,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_SET_VALUE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hClassKey),
        core::ptr::addr_of_mut!(Disposition),
    ) != 0
    {
        ERR!(
            "RegCreateKeyExW(%s) failed\n",
            debugstr_w(core::ptr::addr_of_mut!(FullBuffer) as *mut WCHAR)
        );
        return (INVALID_HANDLE_VALUE).cast();
    }
    if Disposition == REG_CREATED_NEW_KEY {
        TRACE!(
            "The class key %s was successfully created\n",
            debugstr_w(core::ptr::addr_of_mut!(FullBuffer) as *mut WCHAR)
        );
    } else {
        TRACE!(
            "The class key %s was successfully opened\n",
            debugstr_w(core::ptr::addr_of_mut!(FullBuffer) as *mut WCHAR)
        );
    }

    if RegSetValueExW(
        hClassKey,
        REGSTR_VAL_CLASS,
        0,
        REG_SZ,
        core::ptr::addr_of_mut!(Buffer) as LPBYTE,
        RequiredSize * (core::mem::size_of::<WCHAR>() as DWORD),
    ) != 0
    {
        RegCloseKey(hClassKey);
        RegDeleteKeyW(
            HKEY_LOCAL_MACHINE,
            core::ptr::addr_of_mut!(FullBuffer) as LPCWSTR,
        );
        return (INVALID_HANDLE_VALUE).cast();
    }

    return hClassKey;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiInstallClassW(
    hwndParent: HWND,
    InfFileName: PCWSTR,
    Flags: DWORD,
    FileQueue: HSPFILEQ,
) -> BOOL {
    return SetupDiInstallClassExW(hwndParent, InfFileName, Flags, FileQueue, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiOpenClassRegKey(
    ClassGuid: *const GUID,
    samDesired: REGSAM,
) -> HKEY {
    return SetupDiOpenClassRegKeyExW(ClassGuid, samDesired,
                                     DIOCR_INSTALLER as DWORD, core::ptr::null::<u16>(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiOpenClassRegKeyExA(
    ClassGuid: *const GUID,
    samDesired: REGSAM,
    Flags: DWORD,
    MachineName: PCSTR,
    Reserved: PVOID,
) -> HKEY {
    let mut MachineNameW: PWSTR = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();

    if !MachineName.is_null() {
        MachineNameW = pSetupMultiByteToUnicode(MachineName, CP_ACP);
        if MachineNameW.is_null() {
            return (INVALID_HANDLE_VALUE).cast();
        }
    }

    hKey = SetupDiOpenClassRegKeyExW(
        ClassGuid,
        samDesired,
        Flags,
        MachineNameW as PCWSTR,
        Reserved,
    );

    MyFree(MachineNameW as PVOID);

    return hKey;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiOpenClassRegKeyExW(
    ClassGuid: *const GUID,
    samDesired: REGSAM,
    Flags: DWORD,
    MachineName: PCWSTR,
    Reserved: PVOID,
) -> HKEY {
    let mut HKLM: HKEY = core::ptr::null_mut();
    let mut hClassesKey: HKEY = core::ptr::null_mut();
    let mut key: HKEY = core::ptr::null_mut();
    let mut lpKeyName: LPCWSTR = core::ptr::null();
    let mut l: LONG = 0;

    if !MachineName.is_null() {
        l = RegConnectRegistryW(MachineName, HKEY_LOCAL_MACHINE, core::ptr::addr_of_mut!(HKLM)) as LONG;
        if l != ERROR_SUCCESS as LONG {
            SetLastError(l as DWORD);
            return INVALID_HANDLE_VALUE as HKEY;
        }
    } else {
        HKLM = HKEY_LOCAL_MACHINE;
    }

    if Flags == DIOCR_INSTALLER as DWORD {
        lpKeyName = REGSTR_PATH_CLASS_NT;
    } else if Flags == DIOCR_INTERFACE as DWORD {
        lpKeyName = REGSTR_PATH_DEVICE_CLASSES;
    } else {
        ERR!("Invalid Flags parameter!\n");
        SetLastError(ERROR_INVALID_FLAGS as DWORD);
        if !MachineName.is_null() {
            RegCloseKey(HKLM);
        }
        return INVALID_HANDLE_VALUE as HKEY;
    }

    if ClassGuid.is_null() {
        l = RegOpenKeyExW(
            HKLM,
            lpKeyName,
            0 as DWORD,
            samDesired,
            core::ptr::addr_of_mut!(hClassesKey),
        ) as LONG;
        if l != 0 {
            SetLastError(ERROR_INVALID_CLASS as DWORD);
            hClassesKey = INVALID_HANDLE_VALUE as HKEY;
        }
        if !MachineName.is_null() {
            RegCloseKey(HKLM);
        }
        key = hClassesKey;
    } else {
        let mut bracedGuidString: [WCHAR; 39] = [0; 39];

        SETUPDI_GuidToString(ClassGuid, core::ptr::addr_of_mut!(bracedGuidString) as *mut WCHAR);

        l = RegOpenKeyExW(
            HKLM,
            lpKeyName,
            0 as DWORD,
            samDesired,
            core::ptr::addr_of_mut!(hClassesKey),
        ) as LONG;
        if l == 0 {
            if !MachineName.is_null() {
                RegCloseKey(HKLM);
            }

            l = RegOpenKeyExW(
                hClassesKey,
                core::ptr::addr_of_mut!(bracedGuidString) as *mut WCHAR as LPCWSTR,
                0 as DWORD,
                samDesired,
                core::ptr::addr_of_mut!(key),
            ) as LONG;
            if l != 0 {
                SetLastError(l as DWORD);
                key = INVALID_HANDLE_VALUE as HKEY;
            }
            RegCloseKey(hClassesKey);
        } else {
            if !MachineName.is_null() {
                RegCloseKey(HKLM);
            }
            SetLastError(l as DWORD);
            key = INVALID_HANDLE_VALUE as HKEY;
        }
    }

    return key;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiOpenDeviceInterfaceW(
    DeviceInfoSet: HDEVINFO,
    DevicePath: PCWSTR,
    OpenFlags: DWORD,
    DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
) -> BOOL {
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut pEnd: PCWSTR = core::ptr::null();
    let mut dwLength: DWORD = 0;
    let mut dwError: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut dwKeyName: DWORD = 0;
    let mut dwSubIndex: DWORD = 0;
    let mut ClassId: CLSID = core::mem::zeroed();
    let mut Buffer: [WCHAR; (MAX_PATH + 1) as usize] =
        [0; (MAX_PATH + 1) as usize];
    let mut SymBuffer: [WCHAR; (MAX_PATH + 1) as usize] =
        [0; (MAX_PATH + 1) as usize];
    let mut InstancePath: [WCHAR; (MAX_PATH + 1) as usize] =
        [0; (MAX_PATH + 1) as usize];
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hDevKey: HKEY = core::ptr::null_mut();
    let mut hSymKey: HKEY = core::ptr::null_mut();
    let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut();
    let mut deviceInterface: *mut DeviceInterface = core::ptr::null_mut();
    let mut Ret: BOOL = 0;
    let mut ItemList: PLIST_ENTRY = core::ptr::null_mut();
    let mut InterfaceListEntry: PLIST_ENTRY = core::ptr::null_mut();

    if !DeviceInterfaceData.is_null()
        && (*DeviceInterfaceData).cbSize as usize != core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as usize
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    if (*(DeviceInfoSet as *mut DeviceInfoSet)).magic != SETUP_DEVICE_INFO_SET_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }

    list = DeviceInfoSet as *mut DeviceInfoSet;

    dwLength = wcslen(DevicePath) as DWORD;
    if dwLength < 39 {
        /* path must be at least a guid length + L'\0' */
        SetLastError(ERROR_BAD_PATHNAME);
        return (FALSE) as BOOL;
    }

    if *DevicePath.offset(0 as isize) != 0x005Cu16
        || *DevicePath.offset(1 as isize) != 0x005Cu16
        || (*DevicePath.offset(2 as isize) != 0x003Fu16
            && *DevicePath.offset(2 as isize) != 0x002Eu16)
        || *DevicePath.offset(3 as isize) != 0x005Cu16
    {
        /* invalid formatted path */
        SetLastError(ERROR_BAD_PATHNAME);
        return (FALSE) as BOOL;
    }

    /* check for reference strings */
    pEnd = wcschr(
        DevicePath.offset(4 as isize),
        0x005Cu16,
    ) as PCWSTR;
    if pEnd.is_null() {
        /* no reference string */
        pEnd = DevicePath.offset(dwLength as isize);
    }

    /* copy guid */
    wcscpy(
        Buffer.as_mut_ptr(),
        pEnd.offset(-(37 as isize)),
    );
    Buffer[36 as usize] = 0x0000u16;

    dwError = UuidFromStringW(Buffer.as_mut_ptr(), core::ptr::addr_of_mut!(ClassId))
        as DWORD;
    if dwError != NOERROR as DWORD {
        /* invalid formatted path */
        SetLastError(ERROR_BAD_PATHNAME);
        return (FALSE) as BOOL;
    }

    hKey = SetupDiOpenClassRegKeyExW(
        core::ptr::addr_of!(ClassId),
        KEY_READ,
        DIOCR_INTERFACE,
        (*list).MachineName,
        core::ptr::null_mut(),
    );

    if hKey == (INVALID_HANDLE_VALUE).cast() {
        /* invalid device class */
        return (FALSE) as BOOL;
    }

    ItemList = (*list).ListHead.Flink as PLIST_ENTRY;
    while ItemList != core::ptr::addr_of_mut!((*list).ListHead) {
        deviceInfo = CONTAINING_RECORD!(ItemList, DeviceInfo, ListEntry);
        InterfaceListEntry = (*deviceInfo).InterfaceListHead.Flink as PLIST_ENTRY;
        while InterfaceListEntry != core::ptr::addr_of_mut!((*deviceInfo).InterfaceListHead) {
            deviceInterface = CONTAINING_RECORD!(InterfaceListEntry, DeviceInterface, ListEntry);
            if !IsEqualIID(
                core::ptr::addr_of_mut!((*deviceInterface).InterfaceClassGuid),
                core::ptr::addr_of_mut!(ClassId),
            )
            {
                InterfaceListEntry = (*InterfaceListEntry).Flink as PLIST_ENTRY;
                continue;
            }

            if wcsicmp(
                core::ptr::addr_of_mut!((*deviceInterface).SymbolicLink) as *mut WCHAR,
                DevicePath,
            ) == 0
            {
                if !DeviceInterfaceData.is_null() {
                    (*DeviceInterfaceData).Reserved = deviceInterface as ULONG_PTR;
                    (*DeviceInterfaceData).Flags = (*deviceInterface).Flags;
                    CopyMemory!(
                        core::ptr::addr_of_mut!((*DeviceInterfaceData).InterfaceClassGuid)
                            as *mut core::ffi::c_void,
                        core::ptr::addr_of_mut!(ClassId) as *const core::ffi::c_void,
                        core::mem::size_of::<GUID>(),
                    );
                }

                return TRUE as BOOL;
            }
        }
    }

    dwIndex = 0 as DWORD;
    loop {
        Buffer[0 as usize] = 0;
        dwKeyName = (core::mem::size_of::<[WCHAR; (MAX_PATH + 1) as usize]>()
            / core::mem::size_of::<WCHAR>()) as DWORD;
        dwError = RegEnumKeyExW(
            hKey,
            dwIndex,
            Buffer.as_mut_ptr(),
            core::ptr::addr_of_mut!(dwKeyName),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD;

        if dwError != ERROR_SUCCESS {
            break;
        }

        if RegOpenKeyExW(
            hKey,
            Buffer.as_mut_ptr(),
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(hDevKey),
        ) != (ERROR_SUCCESS as i32)
        {
            break;
        }

        dwSubIndex = 0 as DWORD;
        InstancePath[0 as usize] = 0;
        dwKeyName = core::mem::size_of::<[WCHAR; (MAX_PATH + 1) as usize]>()
            as DWORD;

        dwError = RegQueryValueExW(
            hDevKey,
            RUSTCONV_WSTR_E9A256AB2F78.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            InstancePath.as_mut_ptr() as LPBYTE,
            core::ptr::addr_of_mut!(dwKeyName),
        ) as DWORD;

        loop {
            Buffer[0 as usize] = 0;
            dwKeyName = (core::mem::size_of::<[WCHAR; (MAX_PATH + 1) as usize]>()
                / core::mem::size_of::<WCHAR>()) as DWORD;
            dwError = RegEnumKeyExW(
                hDevKey,
                dwSubIndex,
                Buffer.as_mut_ptr(),
                core::ptr::addr_of_mut!(dwKeyName),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as DWORD;

            if dwError != ERROR_SUCCESS {
                break;
            }

            dwError = RegOpenKeyExW(
                hDevKey,
                Buffer.as_mut_ptr(),
                0,
                KEY_READ,
                core::ptr::addr_of_mut!(hSymKey),
            ) as DWORD;
            if dwError != ERROR_SUCCESS {
                break;
            }

            /* query for symbolic link */
            dwKeyName = core::mem::size_of::<[WCHAR; (MAX_PATH + 1) as usize]>()
                as DWORD;
            SymBuffer[0 as usize] = 0x0000u16;
            dwError = RegQueryValueExW(
                hSymKey,
                RUSTCONV_WSTR_0CF505C5BBC3.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                SymBuffer.as_mut_ptr() as LPBYTE,
                core::ptr::addr_of_mut!(dwKeyName),
            ) as DWORD;

            if dwError != ERROR_SUCCESS {
                RegCloseKey(hSymKey);
                break;
            }

            if wcsicmp(SymBuffer.as_mut_ptr(), DevicePath) == 0 {
                Ret = CreateDeviceInfo(
                    list,
                    InstancePath.as_mut_ptr(),
                    core::ptr::addr_of!(ClassId),
                    core::ptr::addr_of_mut!(deviceInfo),
                );
                RegCloseKey(hSymKey);
                RegCloseKey(hDevKey);
                RegCloseKey(hKey);

                if Ret != 0 {
                    deviceInterface = HeapAlloc(
                        GetProcessHeap(),
                        0 as DWORD,
                        ((core::mem::size_of::<DeviceInterface>()
                            + (wcslen(SymBuffer.as_mut_ptr()) as usize
                                + 1 as usize)
                                * core::mem::size_of::<WCHAR>())) as SIZE_T,
                    ) as *mut DeviceInterface;
                    if !deviceInterface.is_null() {
                        CopyMemory!(
                            core::ptr::addr_of_mut!((*deviceInterface).InterfaceClassGuid)
                                as *mut core::ffi::c_void,
                            core::ptr::addr_of!(ClassId) as *const core::ffi::c_void,
                            core::mem::size_of::<GUID>(),
                        );
                        (*deviceInterface).DeviceInfo = deviceInfo;
                        (*deviceInterface).Flags = SPINT_ACTIVE; //FIXME

                        wcscpy(
                            core::ptr::addr_of_mut!((*deviceInterface).SymbolicLink) as *mut WCHAR,
                            SymBuffer.as_mut_ptr(),
                        );

                        InsertTailList(
                            core::ptr::addr_of_mut!((*deviceInfo).InterfaceListHead),
                            core::ptr::addr_of_mut!((*deviceInterface).ListEntry),
                        );
                        InsertTailList(
                            core::ptr::addr_of_mut!((*list).ListHead),
                            core::ptr::addr_of_mut!((*deviceInfo).ListEntry),
                        );

                        if !DeviceInterfaceData.is_null() {
                            (*DeviceInterfaceData).Reserved = deviceInterface as ULONG_PTR;
                            (*DeviceInterfaceData).Flags = (*deviceInterface).Flags;
                            CopyMemory!(
                                core::ptr::addr_of_mut!((*DeviceInterfaceData).InterfaceClassGuid)
                                    as *mut core::ffi::c_void,
                                core::ptr::addr_of!(ClassId) as *const core::ffi::c_void,
                                core::mem::size_of::<GUID>(),
                            );
                        } else {
                            Ret = (FALSE as i32);
                            SetLastError(ERROR_INVALID_USER_BUFFER);
                        }
                    }
                } else {
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        deviceInfo as PVOID,
                    );
                    Ret = (FALSE as i32);
                }
                return Ret;
            }
            RegCloseKey(hSymKey);
            dwSubIndex = dwSubIndex.wrapping_add(1);
        }

        RegCloseKey(hDevKey);
        dwIndex = dwIndex.wrapping_add(1);

        if (TRUE) == 0 {
            break;
        }
    }

    RegCloseKey(hKey);
    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiOpenDeviceInterfaceA(
    DeviceInfoSet: HDEVINFO,
    DevicePath: PCSTR,
    OpenFlags: DWORD,
    DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
) -> BOOL {
    let mut DevicePathW: LPWSTR = core::ptr::null_mut();
    let mut bResult: BOOL = 0;
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const i8,
            b"SetupDiOpenDeviceInterfaceA\0" as *const u8 as *const i8,
            4282i32,
            b"%s(%p %s %08lx %p)\n\0" as *const u8 as *const i8,
            b"SetupDiOpenDeviceInterfaceA\0" as *const u8 as *const i8,
            DeviceInfoSet,
            debugstr_a(DevicePath as *const i8),
            OpenFlags,
            DeviceInterfaceData,
        );
    }

    DevicePathW = pSetupMultiByteToUnicode(DevicePath, CP_ACP) as LPWSTR;
    if DevicePathW.is_null() {
        return (FALSE) as BOOL;
    }

    bResult = SetupDiOpenDeviceInterfaceW(
        DeviceInfoSet,
        DevicePathW as PCWSTR,
        OpenFlags,
        DeviceInterfaceData,
    );

    MyFree(DevicePathW as PVOID);

    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetClassInstallParamsA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    ClassInstallParams: PSP_CLASSINSTALL_HEADER,
    ClassInstallParamsSize: DWORD,
) -> BOOL {
    // FIXME("%p %p %x %u\n",DeviceInfoSet, DeviceInfoData, ClassInstallParams->InstallFunction, ClassInstallParamsSize);
    (FALSE as i32)
}
unsafe extern "system" fn IntSetupDiRegisterDeviceInfo(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    return SetupDiRegisterDeviceInfo(DeviceInfoSet, DeviceInfoData, 0, None, core::ptr::null_mut(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiCallClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else if DeviceInfoSet == (INVALID_HANDLE_VALUE as HDEVINFO) {
        SetLastError(ERROR_INVALID_HANDLE);
    } else if (*(DeviceInfoSet as *mut DeviceInfoSet)).magic != SETUP_DEVICE_INFO_SET_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE);
    } else if (*(DeviceInfoSet as *mut DeviceInfoSet)).HKLM != HKEY_LOCAL_MACHINE {
        SetLastError(ERROR_INVALID_HANDLE);
    } else if !DeviceInfoData.is_null()
        && (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>()
    {
        SetLastError(ERROR_INVALID_USER_BUFFER);
    } else {
        let mut InstallParams: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
        // CLASS_COINSTALLER  0x1
        // DEVICE_COINSTALLER 0x2
        // CLASS_INSTALLER    0x4
        let mut CanHandle: UCHAR = 0;
        let mut DefaultHandler: DEFAULT_CLASS_INSTALL_PROC = None;

        match InstallFunction {
            DIF_ADDPROPERTYPAGE_ADVANCED => {
                CanHandle = (CLASS_COINSTALLER | DEVICE_COINSTALLER | CLASS_INSTALLER) as UCHAR;
            }
            DIF_ADDREMOTEPROPERTYPAGE_ADVANCED => {
                CanHandle = (CLASS_COINSTALLER | DEVICE_COINSTALLER | CLASS_INSTALLER) as UCHAR;
            }
            DIF_ALLOW_INSTALL => {
                CanHandle = (CLASS_COINSTALLER | CLASS_INSTALLER) as UCHAR;
            }
            DIF_DETECT => {
                CanHandle = (CLASS_COINSTALLER | CLASS_INSTALLER) as UCHAR;
            }
            DIF_DESTROYPRIVATEDATA => {
                CanHandle = CLASS_INSTALLER as UCHAR;
            }
            DIF_INSTALLDEVICE => {
                CanHandle = (CLASS_COINSTALLER | DEVICE_COINSTALLER | CLASS_INSTALLER) as UCHAR;
                DefaultHandler = Some(SetupDiInstallDevice);
            }
            DIF_INSTALLDEVICEFILES => {
                CanHandle = (CLASS_COINSTALLER | CLASS_INSTALLER) as UCHAR;
                DefaultHandler = Some(SetupDiInstallDriverFiles);
            }
            DIF_INSTALLINTERFACES => {
                CanHandle = (CLASS_COINSTALLER | DEVICE_COINSTALLER | CLASS_INSTALLER) as UCHAR;
                DefaultHandler = Some(SetupDiInstallDeviceInterfaces);
            }
            DIF_NEWDEVICEWIZARD_FINISHINSTALL => {
                CanHandle = (CLASS_COINSTALLER | DEVICE_COINSTALLER | CLASS_INSTALLER) as UCHAR;
            }
            DIF_NEWDEVICEWIZARD_POSTANALYZE => {
                CanHandle = (CLASS_COINSTALLER | CLASS_INSTALLER) as UCHAR;
            }
            DIF_NEWDEVICEWIZARD_PREANALYZE => {
                CanHandle = (CLASS_COINSTALLER | CLASS_INSTALLER) as UCHAR;
            }
            DIF_NEWDEVICEWIZARD_PRESELECT => {
                CanHandle = (CLASS_COINSTALLER | CLASS_INSTALLER) as UCHAR;
            }
            DIF_NEWDEVICEWIZARD_SELECT => {
                CanHandle = (CLASS_COINSTALLER | CLASS_INSTALLER) as UCHAR;
            }
            DIF_POWERMESSAGEWAKE => {
                CanHandle = (CLASS_COINSTALLER | DEVICE_COINSTALLER | CLASS_INSTALLER) as UCHAR;
            }
            DIF_PROPERTYCHANGE => {
                CanHandle = (CLASS_COINSTALLER | DEVICE_COINSTALLER | CLASS_INSTALLER) as UCHAR;
                DefaultHandler = Some(SetupDiChangeState);
            }
            DIF_REGISTER_COINSTALLERS => {
                CanHandle = (CLASS_COINSTALLER | CLASS_INSTALLER) as UCHAR;
                DefaultHandler = Some(SetupDiRegisterCoDeviceInstallers);
            }
            DIF_REGISTERDEVICE => {
                CanHandle = (CLASS_COINSTALLER | CLASS_INSTALLER) as UCHAR;
                DefaultHandler = Some(IntSetupDiRegisterDeviceInfo);
            }
            DIF_REMOVE => {
                CanHandle = (CLASS_COINSTALLER | DEVICE_COINSTALLER | CLASS_INSTALLER) as UCHAR;
                DefaultHandler = Some(SetupDiRemoveDevice);
            }
            DIF_SELECTBESTCOMPATDRV => {
                CanHandle = (CLASS_COINSTALLER | CLASS_INSTALLER) as UCHAR;
                DefaultHandler = Some(SetupDiSelectBestCompatDrv);
            }
            DIF_SELECTDEVICE => {
                CanHandle = (CLASS_COINSTALLER | CLASS_INSTALLER) as UCHAR;
                DefaultHandler = Some(SetupDiSelectDevice);
            }
            DIF_TROUBLESHOOTER => {
                CanHandle = (CLASS_COINSTALLER | DEVICE_COINSTALLER | CLASS_INSTALLER) as UCHAR;
            }
            DIF_UNREMOVE => {
                CanHandle = (CLASS_COINSTALLER | DEVICE_COINSTALLER | CLASS_INSTALLER) as UCHAR;
                DefaultHandler = Some(SetupDiUnremoveDevice);
            }
            _ => {
                ERR!("Install function %u not supported\n", InstallFunction);
                SetLastError(ERROR_NOT_SUPPORTED);
            }
        }

        InstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
        if SetupDiGetDeviceInstallParamsW(DeviceInfoSet, DeviceInfoData, core::ptr::addr_of_mut!(InstallParams)) == 0 {
            /* Don't process this call, as a parameter is invalid */
            CanHandle = 0;
        }

        if CanHandle != 0 {
            let mut ClassCoInstallersListHead: LIST_ENTRY = core::mem::zeroed();
            let mut DeviceCoInstallersListHead: LIST_ENTRY = core::mem::zeroed();
            let mut ClassInstallerLibrary: HMODULE = core::ptr::null_mut();
            let mut ClassInstaller: CLASS_INSTALL_PROC = None;
            let mut Context: COINSTALLER_CONTEXT_DATA = core::mem::zeroed();
            let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
            let mut hKey: HKEY = core::ptr::null_mut();
            let mut dwRegType: DWORD = 0;
            let mut dwLength: DWORD = 0;
            let mut rc: DWORD = NO_ERROR;

            InitializeListHead(core::ptr::addr_of_mut!(ClassCoInstallersListHead));
            InitializeListHead(core::ptr::addr_of_mut!(DeviceCoInstallersListHead));

            if (CanHandle as u32 & (DEVICE_COINSTALLER as u32)) != 0 {
                hKey = SETUPDI_OpenDrvKey(
                    (*(DeviceInfoSet as *mut DeviceInfoSet)).HKLM,
                    (*DeviceInfoData).Reserved as *mut DeviceInfo,
                    KEY_QUERY_VALUE,
                );
                if hKey != (INVALID_HANDLE_VALUE).cast() {
                    rc = RegQueryValueExW(
                        hKey,
                        REGSTR_VAL_COINSTALLERS_32,
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(dwRegType),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(dwLength),
                    ) as DWORD;
                    if rc == ERROR_SUCCESS && dwRegType == REG_MULTI_SZ {
                        let KeyBuffer: LPWSTR = HeapAlloc(
                            GetProcessHeap(),
                            0,
                            dwLength as SIZE_T,
                        ) as LPWSTR;
                        if KeyBuffer != core::ptr::null_mut() {
                            rc = RegQueryValueExW(
                                hKey,
                                REGSTR_VAL_COINSTALLERS_32,
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                KeyBuffer as LPBYTE,
                                core::ptr::addr_of_mut!(dwLength),
                            ) as DWORD;
                            if rc == ERROR_SUCCESS {
                                let mut ptr: LPWSTR = KeyBuffer;
                                while *ptr != 0 {
                                    'body: {
                                        /* Add coinstaller to DeviceCoInstallersListHead list */
                                        let mut coinstaller: *mut CoInstallerElement = core::ptr::null_mut();

                                        coinstaller = HeapAlloc(
                                            GetProcessHeap(),
                                            0,
                                            core::mem::size_of::<CoInstallerElement>() as SIZE_T,
                                        ) as *mut CoInstallerElement;
                                        if coinstaller.is_null() {
                                            break 'body;
                                        }
                                        ZeroMemory!(coinstaller as PVOID, core::mem::size_of::<CoInstallerElement>());
                                        if GetFunctionPointer(
                                            ptr as PWSTR,
                                            core::ptr::addr_of_mut!((*coinstaller).Module),
                                            core::ptr::addr_of_mut!((*coinstaller).Function) as *mut PVOID,
                                        ) == ERROR_SUCCESS
                                        {
                                            InsertTailList(
                                                core::ptr::addr_of_mut!(DeviceCoInstallersListHead),
                                                core::ptr::addr_of_mut!((*coinstaller).ListEntry),
                                            );
                                        } else {
                                            HeapFree(
                                                GetProcessHeap(),
                                                0,
                                                coinstaller as PVOID,
                                            );
                                        }
                                    }
                                    ptr = ptr.offset((strlenW(ptr) + 1) as isize);
                                }
                            }
                            HeapFree(GetProcessHeap(), 0, KeyBuffer as PVOID);
                        }
                    }
                    RegCloseKey(hKey);
                }
            }
            if (CanHandle as u32 & (CLASS_COINSTALLER as u32)) != 0 {
                rc = RegOpenKeyExW(
                    HKEY_LOCAL_MACHINE,
                    REGSTR_PATH_CODEVICEINSTALLERS,
                    0, /* Options */
                    KEY_QUERY_VALUE,
                    core::ptr::addr_of_mut!(hKey),
                ) as DWORD;
                if rc == ERROR_SUCCESS {
                    let mut szGuidString: [WCHAR; 40] = [0; 40];
                    if pSetupStringFromGuid(
                        core::ptr::addr_of_mut!((*DeviceInfoData).ClassGuid),
                        szGuidString.as_mut_ptr(),
                        ARRAYSIZE!(szGuidString),
                    ) == ERROR_SUCCESS
                    {
                        rc = RegQueryValueExW(
                            hKey,
                            szGuidString.as_mut_ptr() as LPCWSTR,
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!(dwRegType),
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!(dwLength),
                        ) as DWORD;
                        if rc == ERROR_SUCCESS && dwRegType == REG_MULTI_SZ {
                            let KeyBuffer: LPWSTR = HeapAlloc(
                                GetProcessHeap(),
                                0,
                                dwLength as SIZE_T,
                            ) as LPWSTR;
                            if KeyBuffer != core::ptr::null_mut() {
                                rc = RegQueryValueExW(
                                    hKey,
                                    szGuidString.as_mut_ptr() as LPCWSTR,
                                    core::ptr::null_mut(),
                                    core::ptr::null_mut(),
                                    KeyBuffer as LPBYTE,
                                    core::ptr::addr_of_mut!(dwLength),
                                ) as DWORD;
                                if rc == ERROR_SUCCESS {
                                    let mut ptr: LPWSTR = KeyBuffer;
                                    while *ptr != 0 {
                                        'body: {
                                            /* Add coinstaller to ClassCoInstallersListHead list */
                                            let mut coinstaller: *mut CoInstallerElement = core::ptr::null_mut();

                                            coinstaller = HeapAlloc(
                                                GetProcessHeap(),
                                                0,
                                                core::mem::size_of::<CoInstallerElement>() as SIZE_T,
                                            ) as *mut CoInstallerElement;
                                            if coinstaller.is_null() {
                                                break 'body;
                                            }
                                            ZeroMemory!(coinstaller as PVOID, core::mem::size_of::<CoInstallerElement>());
                                            if GetFunctionPointer(
                                                ptr as PWSTR,
                                                core::ptr::addr_of_mut!((*coinstaller).Module),
                                                core::ptr::addr_of_mut!((*coinstaller).Function) as *mut PVOID,
                                            ) == ERROR_SUCCESS
                                            {
                                                InsertTailList(
                                                    core::ptr::addr_of_mut!(ClassCoInstallersListHead),
                                                    core::ptr::addr_of_mut!((*coinstaller).ListEntry),
                                                );
                                            } else {
                                                HeapFree(
                                                    GetProcessHeap(),
                                                    0,
                                                    coinstaller as PVOID,
                                                );
                                            }
                                        }
                                        ptr = ptr.offset((strlenW(ptr) + 1) as isize);
                                    }
                                }
                                HeapFree(GetProcessHeap(), 0, KeyBuffer as PVOID);
                            }
                        }
                    }
                    RegCloseKey(hKey);
                }
            }
            if (CanHandle as u32 & (CLASS_INSTALLER as u32)) != 0
                && (InstallParams.FlagsEx & DI_FLAGSEX_CI_FAILED) == 0
            {
                hKey = SetupDiOpenClassRegKey(
                    core::ptr::addr_of_mut!((*DeviceInfoData).ClassGuid),
                    KEY_QUERY_VALUE,
                );
                if hKey != (INVALID_HANDLE_VALUE).cast() {
                    rc = RegQueryValueExW(
                        hKey,
                        REGSTR_VAL_INSTALLER_32,
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(dwRegType),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(dwLength),
                    ) as DWORD;
                    if rc == ERROR_SUCCESS && dwRegType == REG_SZ {
                        let KeyBuffer: LPWSTR = HeapAlloc(
                            GetProcessHeap(),
                            0,
                            dwLength as SIZE_T,
                        ) as LPWSTR;
                        if KeyBuffer != core::ptr::null_mut() {
                            rc = RegQueryValueExW(
                                hKey,
                                REGSTR_VAL_INSTALLER_32,
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                KeyBuffer as LPBYTE,
                                core::ptr::addr_of_mut!(dwLength),
                            ) as DWORD;
                            if rc == ERROR_SUCCESS {
                                /* Get ClassInstaller function pointer */
                                if GetFunctionPointer(
                                    KeyBuffer,
                                    core::ptr::addr_of_mut!(ClassInstallerLibrary),
                                    core::ptr::addr_of_mut!(ClassInstaller) as *mut PVOID,
                                ) != ERROR_SUCCESS
                                {
                                    InstallParams.FlagsEx |= DI_FLAGSEX_CI_FAILED;
                                    SetupDiSetDeviceInstallParamsW(
                                        DeviceInfoSet,
                                        DeviceInfoData,
                                        core::ptr::addr_of_mut!(InstallParams),
                                    );
                                }
                            }
                            HeapFree(GetProcessHeap(), 0, KeyBuffer as PVOID);
                        }
                    }
                    RegCloseKey(hKey);
                }
            }

            /* Call Class co-installers */
            Context.PostProcessing = (FALSE as i32);
            rc = NO_ERROR;
            ListEntry = ClassCoInstallersListHead.Flink;
            while rc == NO_ERROR && ListEntry != core::ptr::addr_of_mut!(ClassCoInstallersListHead) {
                let coinstaller: *mut CoInstallerElement;
                coinstaller = CONTAINING_RECORD!(ListEntry, CoInstallerElement, ListEntry);
                rc = (*coinstaller).Function.unwrap_unchecked()(
                    InstallFunction,
                    DeviceInfoSet,
                    DeviceInfoData,
                    core::ptr::addr_of_mut!(Context),
                );
                (*coinstaller).PrivateData = Context.PrivateData;
                if rc == ERROR_DI_POSTPROCESSING_REQUIRED {
                    (*coinstaller).DoPostProcessing = (TRUE as i32);
                    rc = NO_ERROR;
                }
                ListEntry = (*ListEntry).Flink;
            }

            /* Call Device co-installers */
            ListEntry = DeviceCoInstallersListHead.Flink;
            while rc == NO_ERROR && ListEntry != core::ptr::addr_of_mut!(DeviceCoInstallersListHead) {
                let coinstaller: *mut CoInstallerElement;
                coinstaller = CONTAINING_RECORD!(ListEntry, CoInstallerElement, ListEntry);
                rc = (*coinstaller).Function.unwrap_unchecked()(
                    InstallFunction,
                    DeviceInfoSet,
                    DeviceInfoData,
                    core::ptr::addr_of_mut!(Context),
                );
                (*coinstaller).PrivateData = Context.PrivateData;
                if rc == ERROR_DI_POSTPROCESSING_REQUIRED {
                    (*coinstaller).DoPostProcessing = (TRUE as i32);
                    rc = NO_ERROR;
                }
                ListEntry = (*ListEntry).Flink;
            }

            /* Call Class installer */
            if ClassInstaller.is_some() {
                rc = ClassInstaller.unwrap_unchecked()(
                    InstallFunction,
                    DeviceInfoSet,
                    DeviceInfoData,
                );
                FreeFunctionPointer(
                    ClassInstallerLibrary,
                    core::mem::transmute::<CLASS_INSTALL_PROC, PVOID>(ClassInstaller),
                );
            } else {
                rc = ERROR_DI_DO_DEFAULT;
            }

            /* Call default handler */
            if rc == ERROR_DI_DO_DEFAULT {
                if DefaultHandler.is_some() && (InstallParams.Flags & DI_NODI_DEFAULTACTION) == 0 {
                    if DefaultHandler.unwrap_unchecked()(DeviceInfoSet, DeviceInfoData) != 0 {
                        rc = NO_ERROR;
                    } else {
                        rc = GetLastError();
                    }
                } else {
                    rc = NO_ERROR;
                }
            }

            /* Call Class co-installers that required postprocessing */
            Context.PostProcessing = (TRUE as i32);
            ListEntry = ClassCoInstallersListHead.Flink;
            while ListEntry != core::ptr::addr_of_mut!(ClassCoInstallersListHead) {
                let coinstaller: *mut CoInstallerElement;
                coinstaller = CONTAINING_RECORD!(ListEntry, CoInstallerElement, ListEntry);
                if (*coinstaller).DoPostProcessing != 0 {
                    Context.InstallResult = rc;
                    Context.PrivateData = (*coinstaller).PrivateData;
                    rc = (*coinstaller).Function.unwrap_unchecked()(
                        InstallFunction,
                        DeviceInfoSet,
                        DeviceInfoData,
                        core::ptr::addr_of_mut!(Context),
                    );
                }
                FreeFunctionPointer(
                    (*coinstaller).Module,
                    core::mem::transmute::<COINSTALLER_PROC, PVOID>((*coinstaller).Function),
                );
                ListEntry = (*ListEntry).Flink;
            }

            /* Call Device co-installers that required postprocessing */
            ListEntry = DeviceCoInstallersListHead.Flink;
            while ListEntry != core::ptr::addr_of_mut!(DeviceCoInstallersListHead) {
                let coinstaller: *mut CoInstallerElement;
                coinstaller = CONTAINING_RECORD!(ListEntry, CoInstallerElement, ListEntry);
                if (*coinstaller).DoPostProcessing != 0 {
                    Context.InstallResult = rc;
                    Context.PrivateData = (*coinstaller).PrivateData;
                    rc = (*coinstaller).Function.unwrap_unchecked()(
                        InstallFunction,
                        DeviceInfoSet,
                        DeviceInfoData,
                        core::ptr::addr_of_mut!(Context),
                    );
                }
                FreeFunctionPointer(
                    (*coinstaller).Module,
                    core::mem::transmute::<COINSTALLER_PROC, PVOID>((*coinstaller).Function),
                );
                ListEntry = (*ListEntry).Flink;
            }

            /* Free allocated memory */
            while IsListEmpty(core::ptr::addr_of_mut!(ClassCoInstallersListHead)) == false {
                ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(ClassCoInstallersListHead));
                HeapFree(
                    GetProcessHeap(),
                    0,
                    CONTAINING_RECORD!(ListEntry, CoInstallerElement, ListEntry) as PVOID,
                );
            }
            while IsListEmpty(core::ptr::addr_of_mut!(DeviceCoInstallersListHead)) == false {
                ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(DeviceCoInstallersListHead));
                HeapFree(
                    GetProcessHeap(),
                    0,
                    CONTAINING_RECORD!(ListEntry, CoInstallerElement, ListEntry) as PVOID,
                );
            }

            ret = (rc == NO_ERROR) as BOOL;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDeviceInstallParamsA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DeviceInstallParams: PSP_DEVINSTALL_PARAMS_A,
) -> BOOL {
    let mut deviceInstallParamsW: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
    let mut ret: BOOL = (FALSE as i32);

    if DeviceInstallParams.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else if (*DeviceInstallParams).cbSize as usize != core::mem::size_of::<SP_DEVINSTALL_PARAMS_A>() as usize {
        SetLastError(ERROR_INVALID_USER_BUFFER);
    } else {
        deviceInstallParamsW.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
        ret = SetupDiGetDeviceInstallParamsW(
            DeviceInfoSet,
            DeviceInfoData,
            core::ptr::addr_of_mut!(deviceInstallParamsW),
        );

        if ret != 0 {
            /* Do W->A conversion */
            memcpy(
                DeviceInstallParams as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(deviceInstallParamsW) as *const core::ffi::c_void,
                core::mem::offset_of!(SP_DEVINSTALL_PARAMS_W, DriverPath) as usize,
            );
            if WideCharToMultiByte(
                CP_ACP,
                0,
                core::ptr::addr_of_mut!(deviceInstallParamsW.DriverPath) as *mut WCHAR as LPCWSTR,
                -1,
                core::ptr::addr_of_mut!((*DeviceInstallParams).DriverPath) as LPSTR,
                (MAX_PATH as i32),
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            ) == 0
            {
                (*DeviceInstallParams).DriverPath[0 as usize] = 0 as CHAR;
                ret = (FALSE as i32);
            }
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDeviceInfoListClass(
    DeviceInfoSet: HDEVINFO,
    ClassGuid: LPGUID,
) -> BOOL {
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut::<DeviceInfoSet>();
    let mut ret: BOOL = FALSE as BOOL;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiGetDeviceInfoListClass\0" as *const u8 as *const ::core::ffi::c_char,
            4725 as ::core::ffi::c_int,
            b"%s(%p %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiGetDeviceInfoListClass\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            ClassGuid,
        );
    }
    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else {
        list = DeviceInfoSet as *mut DeviceInfoSet;
        if (*list).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else if IsEqualIID(
            core::ptr::addr_of_mut!((*list).ClassGuid),
            core::ptr::addr_of!(GUID_NULL),
        ) {
            SetLastError(ERROR_NO_ASSOCIATED_CLASS as DWORD);
        } else {
            *ClassGuid = (*list).ClassGuid;
            ret = TRUE as BOOL;
        }
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiGetDeviceInfoListClass\0" as *const u8 as *const ::core::ffi::c_char,
            4740 as ::core::ffi::c_int,
            b"Returning %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            ret,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDeviceInstallParamsW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DeviceInstallParams: PSP_DEVINSTALL_PARAMS_W,
) -> BOOL {
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut ret: BOOL = FALSE as BOOL;

    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else {
        list = DeviceInfoSet as *mut DeviceInfoSet;
        if (*list).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else if !DeviceInfoData.is_null()
            && ((*DeviceInfoData).cbSize as usize)
                != core::mem::size_of::<SP_DEVINFO_DATA>()
        {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else if DeviceInstallParams.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else if ((*DeviceInstallParams).cbSize as usize)
            != core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>()
        {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else {
            let mut Source: PSP_DEVINSTALL_PARAMS_W = core::ptr::null_mut();

            if !DeviceInfoData.is_null() {
                Source = core::ptr::addr_of_mut!(
                    (*((*DeviceInfoData).Reserved as *mut DeviceInfo)).InstallParams
                ) as PSP_DEVINSTALL_PARAMS_W;
            } else {
                Source = core::ptr::addr_of_mut!((*list).InstallParams) as PSP_DEVINSTALL_PARAMS_W;
            }

            ret = TRUE as BOOL;

            // _SEH2_TRY
            let mut _SEH2_Volatile0_local: i32 = 0;
            {
                memcpy(
                    DeviceInstallParams as *mut core::ffi::c_void,
                    Source as *const core::ffi::c_void,
                    (*Source).cbSize as usize,
                );
            }
            // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
            if _SEH2_Volatile0_local != 0 {
                SetLastError(
                    RtlNtStatusToDosError(_SEH2_GetExceptionCode() as NTSTATUS) as DWORD,
                );
                ret = FALSE as BOOL;
            }
            // _SEH2_END
        }
    }

    return ret;
}

unsafe extern "C" fn CheckDeviceInstallParameters(
    mut DeviceInstallParams: PSP_DEVINSTALL_PARAMS_W,
) -> BOOL {
    let mut SupportedFlags: DWORD = (DI_NOVCP
        | DI_DIDCOMPAT
        | DI_DIDCLASS
        | DI_NEEDRESTART
        | DI_NEEDREBOOT
        | DI_RESOURCEPAGE_ADDED
        | DI_PROPERTIES_CHANGE
        | DI_ENUMSINGLEINF
        | DI_DONOTCALLCONFIGMG
        | DI_CLASSINSTALLPARAMS
        | DI_NODI_DEFAULTACTION
        | DI_QUIETINSTALL
        | DI_NOFILECOPY
        | DI_DRIVERPAGE_ADDED) as DWORD;
    let mut SupportedFlagsEx: DWORD = (DI_FLAGSEX_CI_FAILED
        | DI_FLAGSEX_DIDINFOLIST
        | DI_FLAGSEX_DIDCOMPATINFO
        | DI_FLAGSEX_ALLOWEXCLUDEDDRVS
        | DI_FLAGSEX_NO_DRVREG_MODIFY
        | (DI_FLAGSEX_INSTALLEDDRIVER as u32)) as DWORD;
    let mut ret: BOOL = FALSE as BOOL;
    if (*DeviceInstallParams).Flags & !SupportedFlags != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/setupapi/devinst.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"CheckDeviceInstallParameters\0" as *const u8
                    as *const ::core::ffi::c_char,
                4831 as ::core::ffi::c_int,
                b"Unknown Flags: 0x%08lx\n\0" as *const u8 as *const ::core::ffi::c_char,
                (*DeviceInstallParams).Flags & !SupportedFlags,
            );
        }
        SetLastError(ERROR_INVALID_FLAGS as DWORD);
    } else if (*DeviceInstallParams).FlagsEx & !SupportedFlagsEx != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/setupapi/devinst.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"CheckDeviceInstallParameters\0" as *const u8
                    as *const ::core::ffi::c_char,
                4836 as ::core::ffi::c_int,
                b"Unknown FlagsEx: 0x%08lx\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*DeviceInstallParams).FlagsEx & !SupportedFlagsEx,
            );
        }
        SetLastError(ERROR_INVALID_FLAGS as DWORD);
    } else if (*DeviceInstallParams).Flags & (DI_NOVCP as DWORD) != 0
        && ((*DeviceInstallParams).FileQueue.is_null()
            || (*DeviceInstallParams).FileQueue == (INVALID_HANDLE_VALUE as HSPFILEQ))
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else {
        ret = TRUE as BOOL;
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetDeviceInstallParamsW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DeviceInstallParams: PSP_DEVINSTALL_PARAMS_W,
) -> BOOL {
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiSetDeviceInstallParamsW\0" as *const u8 as *const ::core::ffi::c_char,
            4863 as ::core::ffi::c_int,
            b"%s(%p %p %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiSetDeviceInstallParamsW\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            DeviceInfoData,
            DeviceInstallParams,
        );
    }
    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else {
        list = DeviceInfoSet as *mut DeviceInfoSet;
        if (*list).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else if !DeviceInfoData.is_null()
            && (*DeviceInfoData).cbSize as usize
                != ::core::mem::size_of::<SP_DEVINFO_DATA>() as usize
        {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else if DeviceInstallParams.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else if (*DeviceInstallParams).cbSize as usize
            != ::core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as usize
        {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else if CheckDeviceInstallParameters(DeviceInstallParams) != 0 {
            let mut Destination: PSP_DEVINSTALL_PARAMS_W = core::ptr::null_mut();

            if !DeviceInfoData.is_null() {
                Destination = core::ptr::addr_of_mut!(
                    (*((*DeviceInfoData).Reserved as *mut DeviceInfo)).InstallParams
                ) as PSP_DEVINSTALL_PARAMS_W;
            } else {
                Destination = core::ptr::addr_of_mut!((*list).InstallParams) as PSP_DEVINSTALL_PARAMS_W;
            }
            memcpy(
                Destination as *mut core::ffi::c_void,
                DeviceInstallParams as *const core::ffi::c_void,
                (*DeviceInstallParams).cbSize as usize,
            );
            ret = (TRUE as i32);
        }
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiSetDeviceInstallParamsW\0" as *const u8 as *const ::core::ffi::c_char,
            4887 as ::core::ffi::c_int,
            b"Returning %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            ret,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetDeviceInstallParamsA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DeviceInstallParams: PSP_DEVINSTALL_PARAMS_A,
) -> BOOL {
    let mut deviceInstallParamsW: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
    let mut len: i32 = 0;
    let mut ret: BOOL = (FALSE as i32);

    if DeviceInstallParams.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else if ((*DeviceInstallParams).cbSize as usize)
        < (core::mem::size_of::<SP_DEVINSTALL_PARAMS_A>() as usize)
    {
        SetLastError(ERROR_INVALID_USER_BUFFER);
    } else {
        memcpy(
            core::ptr::addr_of_mut!(deviceInstallParamsW) as *mut core::ffi::c_void,
            DeviceInstallParams as *const core::ffi::c_void,
            core::mem::offset_of!(SP_DEVINSTALL_PARAMS_A, DriverPath) as usize,
        );
        deviceInstallParamsW.cbSize =
            core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!((*DeviceInstallParams).DriverPath) as LPCSTR,
            -(1),
            core::ptr::null_mut(),
            0,
        );
        if len == 0 {
            ERR!("DrivePath is NULL\n");
            ret = (FALSE as i32);
        } else {
            MultiByteToWideChar(
                CP_ACP,
                0,
                core::ptr::addr_of_mut!((*DeviceInstallParams).DriverPath) as LPCSTR,
                -(1),
                core::ptr::addr_of_mut!(deviceInstallParamsW.DriverPath) as LPWSTR,
                len,
            );
            ret = SetupDiSetDeviceInstallParamsW(
                DeviceInfoSet,
                DeviceInfoData,
                core::ptr::addr_of_mut!(deviceInstallParamsW),
            );
        }
    }

    return ret;
}

unsafe extern "C" fn OpenHardwareProfileKey(
    HKLM: HKEY,
    HwProfile: DWORD,
    samDesired: DWORD,
) -> HKEY {
    let mut hHWProfilesKey: HKEY = core::ptr::null_mut();
    let mut hHWProfileKey: HKEY = core::ptr::null_mut();
    let mut ret: HKEY = INVALID_HANDLE_VALUE as HKEY;
    let mut rc: LONG = 0;

    'quit: {
        rc = RegOpenKeyExW(
            HKLM,
            REGSTR_PATH_HWPROFILES,
            0 as DWORD,
            READ_CONTROL as REGSAM,
            &raw mut hHWProfilesKey,
        ) as LONG;
        if rc != ERROR_SUCCESS as LONG {
            SetLastError(rc as DWORD);
            break 'quit;
        }
        if HwProfile == 0 as DWORD {
            rc = RegOpenKeyExW(
                hHWProfilesKey,
                REGSTR_KEY_CURRENT,
                0 as DWORD,
                KEY_CREATE_SUB_KEY as REGSAM,
                &raw mut hHWProfileKey,
            ) as LONG;
        } else {
            let mut subKey: [WCHAR; 5] = [0; 5];
            snprintfW(
                &raw mut subKey as *mut WCHAR,
                4,
                &raw const InstanceKeyFormat as *const WCHAR,
                HwProfile,
            );
            subKey[4 as usize] = 0 as WCHAR;
            rc = RegOpenKeyExW(
                hHWProfilesKey,
                &raw mut subKey as *mut WCHAR as LPCWSTR,
                0 as DWORD,
                KEY_CREATE_SUB_KEY as REGSAM,
                &raw mut hHWProfileKey,
            ) as LONG;
        }
        if rc != ERROR_SUCCESS as LONG {
            SetLastError(rc as DWORD);
            break 'quit;
        }
        ret = hHWProfileKey;
    }

    // cleanup:
    if !hHWProfilesKey.is_null() {
        RegCloseKey(hHWProfilesKey);
    }
    if !hHWProfileKey.is_null() && hHWProfileKey != ret {
        RegCloseKey(hHWProfileKey);
    }
    return ret;
}

unsafe extern "C" fn IsDeviceInfoInDeviceInfoSet(
    deviceInfoSet: *mut DeviceInfoSet,
    deviceInfo: *mut DeviceInfo,
) -> BOOL {
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();

    ListEntry = (*deviceInfoSet).ListHead.Flink as PLIST_ENTRY;
    while ListEntry != core::ptr::addr_of_mut!((*deviceInfoSet).ListHead) {
        if deviceInfo
            == ((ListEntry as usize).wrapping_sub(
                core::mem::offset_of!(DeviceInfo, ListEntry),
            )) as *mut DeviceInfo
        {
            return (TRUE) as BOOL;
        }

        ListEntry = (*ListEntry).Flink as PLIST_ENTRY;
    }

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiDeleteDeviceInfo(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut deviceInfoSet: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut ret: BOOL = FALSE as BOOL;

    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else {
        deviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
        if (*deviceInfoSet).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else if DeviceInfoData.is_null()
            || (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
            || (*DeviceInfoData).Reserved == 0
        {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else if IsDeviceInfoInDeviceInfoSet(
            deviceInfoSet,
            (*DeviceInfoData).Reserved as *mut DeviceInfo,
        ) == 0
        {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else {
            let mut deviceInfo: *mut DeviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
            RemoveEntryList(&raw mut (*deviceInfo).ListEntry);
            DestroyDeviceInfo(deviceInfo);
            ret = TRUE as BOOL;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiOpenDeviceInfoA(
    DeviceInfoSet: HDEVINFO,
    DeviceInstanceId: PCSTR,
    hwndParent: HWND,
    OpenFlags: DWORD,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut DeviceInstanceIdW: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut bResult: BOOL = 0;
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"SetupDiOpenDeviceInfoA\0" as *const u8 as *const core::ffi::c_char,
            5052 as i32,
            b"%s(%p %s %p %lx %p)\n\0" as *const u8 as *const core::ffi::c_char,
            b"SetupDiOpenDeviceInfoA\0" as *const u8 as *const core::ffi::c_char,
            DeviceInfoSet,
            DeviceInstanceId,
            hwndParent,
            OpenFlags,
            DeviceInfoData,
        );
    }
    DeviceInstanceIdW = pSetupMultiByteToUnicode(DeviceInstanceId, CP_ACP as UINT) as LPWSTR;
    if DeviceInstanceIdW.is_null() {
        return (FALSE) as BOOL;
    }
    bResult = SetupDiOpenDeviceInfoW(
        DeviceInfoSet,
        DeviceInstanceIdW as PCWSTR,
        hwndParent,
        OpenFlags,
        DeviceInfoData,
    );
    MyFree(DeviceInstanceIdW as PVOID);
    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiOpenDeviceInfoW(
    DeviceInfoSet: HDEVINFO,
    DeviceInstanceId: PCWSTR,
    hwndParent: HWND,
    OpenFlags: DWORD,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut hEnumKey: HKEY = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut rc: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut ret: BOOL = (FALSE as i32);

    TRACE!(
        "%s(%p %s %p %lx %p)\n",
        "SetupDiOpenDeviceInfoW",
        DeviceInfoSet,
        debugstr_w(DeviceInstanceId as *const WCHAR),
        hwndParent,
        OpenFlags,
        DeviceInfoData
    );

    if (OpenFlags & DIOD_CANCEL_REMOVE) != 0 {
        FIXME!("DIOD_CANCEL_REMOVE flag not implemented\n");
    }

    'cleanup: {
        if DeviceInfoSet.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
        } else {
            list = DeviceInfoSet as *mut DeviceInfoSet;
            if (*list).magic != SETUP_DEVICE_INFO_SET_MAGIC {
                SetLastError(ERROR_INVALID_HANDLE);
            } else if DeviceInstanceId.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER);
            } else if (OpenFlags & !(DIOD_CANCEL_REMOVE | DIOD_INHERIT_CLASSDRVS)) != 0 {
                TRACE!(
                    "Unknown flags: 0x%08lx\n",
                    OpenFlags & !(DIOD_CANCEL_REMOVE | DIOD_INHERIT_CLASSDRVS)
                );
                SetLastError(ERROR_INVALID_FLAGS);
            } else if !DeviceInfoData.is_null()
                && (*DeviceInfoData).cbSize as usize
                    != core::mem::size_of::<SP_DEVINFO_DATA>()
            {
                SetLastError(ERROR_INVALID_USER_BUFFER);
            } else {
                let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut();
                /* Search if device already exists in DeviceInfoSet.
                 *    If yes, return the existing element
                 *    If no, create a new element using information in registry
                 */
                let mut ItemList: PLIST_ENTRY = (*list).ListHead.Flink as PLIST_ENTRY;
                while ItemList != core::ptr::addr_of_mut!((*list).ListHead) {
                    deviceInfo = (ItemList as ULONG_PTR)
                        .wrapping_sub(core::mem::offset_of!(DeviceInfo, ListEntry) as ULONG_PTR)
                        as *mut DeviceInfo;
                    if wcscmp(
                        (*deviceInfo).instanceId as LPCWSTR,
                        DeviceInstanceId as LPCWSTR,
                    ) == 0
                    {
                        break;
                    }
                    deviceInfo = core::ptr::null_mut();
                    ItemList = (*ItemList).Flink as PLIST_ENTRY;
                }

                if !deviceInfo.is_null() {
                    /* good one found */
                    ret = (TRUE as i32);
                } else {
                    let mut ClassGUID: GUID = GUID {
                        Data1: 0,
                        Data2: 0,
                        Data3: 0,
                        Data4: [0; 8],
                    };
                    let mut szClassGuid: [WCHAR; (MAX_GUID_STRING_LEN as _)] =
                        [0; (MAX_GUID_STRING_LEN as _)];

                    /* Open supposed registry key */
                    rc = RegOpenKeyExW(
                        (*list).HKLM,
                        REGSTR_PATH_SYSTEMENUM,
                        0, /* Options */
                        READ_CONTROL,
                        core::ptr::addr_of_mut!(hEnumKey),
                    ) as DWORD;
                    if rc != ERROR_SUCCESS {
                        SetLastError(rc);
                        break 'cleanup;
                    }
                    rc = RegOpenKeyExW(
                        hEnumKey,
                        DeviceInstanceId,
                        0, /* Options */
                        KEY_QUERY_VALUE,
                        core::ptr::addr_of_mut!(hKey),
                    ) as DWORD;
                    RegCloseKey(hEnumKey);
                    if rc != ERROR_SUCCESS {
                        if rc == ERROR_FILE_NOT_FOUND {
                            rc = ERROR_NO_SUCH_DEVINST;
                        }
                        SetLastError(rc);
                        break 'cleanup;
                    }

                    ClassGUID = GUID_NULL;
                    dwSize = ((MAX_GUID_STRING_LEN as usize)
                        * core::mem::size_of::<WCHAR>()) as DWORD;

                    if RegQueryValueExW(
                        hKey,
                        REGSTR_VAL_CLASSGUID,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(szClassGuid) as *mut WCHAR as LPBYTE,
                        core::ptr::addr_of_mut!(dwSize),
                    ) == (ERROR_SUCCESS as i32)
                    {
                        szClassGuid[(MAX_GUID_STRING_LEN - 2) as usize] =
                            UNICODE_NULL;

                        /* Convert a string to a ClassGuid */
                        UuidFromStringW(
                            (core::ptr::addr_of_mut!(szClassGuid) as *mut WCHAR)
                                .offset(1 as isize),
                            core::ptr::addr_of_mut!(ClassGUID),
                        );
                    }

                    if CreateDeviceInfo(
                        list,
                        DeviceInstanceId,
                        core::ptr::addr_of_mut!(ClassGUID) as LPCGUID,
                        core::ptr::addr_of_mut!(deviceInfo),
                    ) == 0
                    {
                        break 'cleanup;
                    }

                    InsertTailList(
                        core::ptr::addr_of_mut!((*list).ListHead),
                        core::ptr::addr_of_mut!((*deviceInfo).ListEntry),
                    );

                    ret = (TRUE as i32);
                }

                if ret != 0 && !deviceInfo.is_null() && !DeviceInfoData.is_null() {
                    memcpy(
                        core::ptr::addr_of_mut!((*DeviceInfoData).ClassGuid) as *mut core::ffi::c_void,
                        core::ptr::addr_of_mut!((*deviceInfo).ClassGuid) as *const core::ffi::c_void,
                        core::mem::size_of::<GUID>() as usize,
                    );
                    (*DeviceInfoData).DevInst = (*deviceInfo).dnDevInst;
                    (*DeviceInfoData).Reserved = deviceInfo as ULONG_PTR;
                }
            }
        }
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetSelectedDevice(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut ret: BOOL = FALSE as BOOL;

    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else {
        list = DeviceInfoSet as *mut DeviceInfoSet;
        if (*list).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else if (*list).SelectedDevice.is_null() {
            SetLastError(ERROR_NO_DEVICE_SELECTED as DWORD);
        } else if DeviceInfoData.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else if (*DeviceInfoData).cbSize as usize
            != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
        {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else {
            memcpy(
                core::ptr::addr_of_mut!((*DeviceInfoData).ClassGuid) as *mut core::ffi::c_void,
                core::ptr::addr_of!((*(*list).SelectedDevice).ClassGuid) as *const core::ffi::c_void,
                core::mem::size_of::<GUID>() as size_t,
            );
            (*DeviceInfoData).DevInst = (*(*list).SelectedDevice).dnDevInst as DWORD;
            (*DeviceInfoData).Reserved = (*list).SelectedDevice as ULONG_PTR;
            ret = TRUE as BOOL;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetSelectedDevice(DeviceInfoSet: HDEVINFO, DeviceInfoData: PSP_DEVINFO_DATA) -> BOOL {
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else {
        list = DeviceInfoSet as *mut DeviceInfoSet;
        if (*list).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else if DeviceInfoData.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else if (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else if (*DeviceInfoData).Reserved == 0 {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else {
            (*list).SelectedDevice = (*DeviceInfoData).Reserved as *mut DeviceInfo;
            ret = (TRUE as i32);
        }
    }

    return ret;
}

unsafe extern "C" fn SETUPAPI_GetCurrentHwProfile(mut DeviceInfoSet: HDEVINFO) -> DWORD {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwRegType: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut hwProfile: DWORD = 0;
    let mut rc: LONG = 0;
    let mut ret: DWORD = (-(1 as ::core::ffi::c_int)) as DWORD;

    'cleanup: {
        rc = RegOpenKeyExW(
            (*(DeviceInfoSet as *mut DeviceInfoSet)).HKLM,
            REGSTR_PATH_IDCONFIGDB as LPCWSTR,
            0 as DWORD, /* Options */
            KEY_QUERY_VALUE as REGSAM,
            &raw mut hKey,
        ) as LONG;
        if rc != ERROR_SUCCESS as LONG {
            SetLastError(rc as DWORD);
            break 'cleanup;
        }

        dwLength = ::core::mem::size_of::<DWORD>() as DWORD;
        rc = RegQueryValueExW(
            hKey,
            REGSTR_VAL_CURRENTCONFIG as LPCWSTR,
            ::core::ptr::null_mut::<DWORD>(),
            &raw mut dwRegType,
            &raw mut hwProfile as LPBYTE,
            &raw mut dwLength,
        ) as LONG;
        if rc != ERROR_SUCCESS as LONG {
            SetLastError(rc as DWORD);
            break 'cleanup;
        } else if dwRegType != REG_DWORD as DWORD
            || dwLength as usize != ::core::mem::size_of::<DWORD>() as usize
        {
            SetLastError(ERROR_GEN_FAILURE as DWORD);
            break 'cleanup;
        }

        ret = hwProfile;
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    return ret;
}

unsafe extern "C" fn ResetDevice(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let deviceInfo: *mut DeviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
    let mut cr: CONFIGRET = 0;
    cr = CM_Enable_DevNode_Ex((*deviceInfo).dnDevInst, 0, (*set).hMachine);
    if cr != CR_SUCCESS {
        SetLastError(GetErrorCodeFromCrCode(cr));
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn StopDevice(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut deviceInfo: *mut DeviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
    let mut cr: CONFIGRET = 0;
    cr = CM_Disable_DevNode_Ex((*deviceInfo).dnDevInst, 0 as ULONG, (*set).hMachine);
    if cr != CR_SUCCESS as CONFIGRET {
        SetLastError(GetErrorCodeFromCrCode(cr));
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiChangeState(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let deviceInfo: *mut DeviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
    let mut PropChange: PSP_PROPCHANGE_PARAMS = core::ptr::null_mut();
    let mut hRootKey: HKEY = (INVALID_HANDLE_VALUE).cast();
    let mut hKey: HKEY = (INVALID_HANDLE_VALUE).cast();
    let mut RegistryValueName: LPCWSTR = core::ptr::null();
    let mut dwConfigFlags: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut dwRegType: DWORD = 0;
    let mut rc: LONG = 0;
    let mut ret: BOOL = (FALSE as i32);

    'quit: {
        if DeviceInfoData.is_null() {
            PropChange = (*(DeviceInfoSet as *mut DeviceInfoSet)).ClassInstallParams.PropChangeParams;
        } else {
            PropChange = (*((*DeviceInfoData).Reserved as *mut DeviceInfo)).ClassInstallParams.PropChangeParams;
        }
        if PropChange.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            break 'quit;
        }

        if (*PropChange).Scope == DICS_FLAG_GLOBAL as DWORD {
            RegistryValueName = REGSTR_VAL_CONFIGFLAGS;
        } else {
            RegistryValueName = REGSTR_VAL_CSCONFIGFLAGS;
        }

        'sw: {
            match (*PropChange).StateChange {
                DICS_ENABLE | DICS_DISABLE => {
                    if (*PropChange).Scope == DICS_FLAG_GLOBAL as DWORD {
                        hRootKey = (*set).HKLM;
                    } else {
                        hRootKey = OpenHardwareProfileKey(
                            (*set).HKLM,
                            (*PropChange).HwProfile,
                            KEY_CREATE_SUB_KEY as DWORD,
                        );
                        if hRootKey == (INVALID_HANDLE_VALUE).cast() {
                            break 'quit;
                        }
                    }

                    /* Enable/disable device in registry */
                    hKey = SETUPDI_OpenDrvKey(
                        hRootKey,
                        deviceInfo,
                        (KEY_QUERY_VALUE | KEY_SET_VALUE) as REGSAM,
                    );
                    if hKey == (INVALID_HANDLE_VALUE).cast()
                        && GetLastError() == ERROR_FILE_NOT_FOUND as DWORD
                    {
                        hKey = SETUPDI_CreateDevKey(
                            hRootKey,
                            deviceInfo,
                            (KEY_QUERY_VALUE | KEY_SET_VALUE) as REGSAM,
                        );
                    }
                    if hKey == (INVALID_HANDLE_VALUE).cast() {
                        break 'sw;
                    }
                    dwLength = core::mem::size_of::<DWORD>() as DWORD;
                    rc = RegQueryValueExW(
                        hKey,
                        RegistryValueName,
                        core::ptr::null_mut::<DWORD>(),
                        &raw mut dwRegType,
                        &raw mut dwConfigFlags as LPBYTE,
                        &raw mut dwLength,
                    ) as LONG;
                    if rc == ERROR_FILE_NOT_FOUND as LONG {
                        dwConfigFlags = 0 as DWORD;
                    } else if rc != ERROR_SUCCESS as LONG {
                        SetLastError(rc as DWORD);
                        break 'quit;
                    } else if dwRegType != REG_DWORD as DWORD
                        || dwLength != core::mem::size_of::<DWORD>() as DWORD
                    {
                        SetLastError(ERROR_GEN_FAILURE as DWORD);
                        break 'quit;
                    }
                    if (*PropChange).StateChange == DICS_ENABLE as DWORD {
                        dwConfigFlags &= !((if (*PropChange).Scope == DICS_FLAG_GLOBAL as DWORD {
                            CONFIGFLAG_DISABLED
                        } else {
                            CSCONFIGFLAG_DISABLED
                        }) as DWORD);
                    } else {
                        dwConfigFlags |= (if (*PropChange).Scope == DICS_FLAG_GLOBAL as DWORD {
                            CONFIGFLAG_DISABLED
                        } else {
                            CSCONFIGFLAG_DISABLED
                        }) as DWORD;
                    }
                    rc = RegSetValueExW(
                        hKey,
                        RegistryValueName,
                        0 as DWORD,
                        REG_DWORD as DWORD,
                        &raw mut dwConfigFlags as LPBYTE as *const BYTE,
                        core::mem::size_of::<DWORD>() as DWORD,
                    ) as LONG;
                    if rc != ERROR_SUCCESS as LONG {
                        SetLastError(rc as DWORD);
                        break 'quit;
                    }

                    /* Enable/disable device if needed */
                    if (*PropChange).Scope == DICS_FLAG_GLOBAL as DWORD
                        || (*PropChange).HwProfile == 0 as DWORD
                        || (*PropChange).HwProfile == SETUPAPI_GetCurrentHwProfile(DeviceInfoSet)
                    {
                        if (*PropChange).StateChange == DICS_ENABLE as DWORD {
                            ret = ResetDevice(DeviceInfoSet, DeviceInfoData);
                        } else {
                            ret = StopDevice(DeviceInfoSet, DeviceInfoData);
                        }
                    } else {
                        ret = TRUE as BOOL;
                    }
                    break 'sw;
                }
                DICS_PROPCHANGE => {
                    ret = ResetDevice(DeviceInfoSet, DeviceInfoData);
                    break 'sw;
                }
                _ => {
                    ERR!("Unknown StateChange 0x%lx\n", (*PropChange).StateChange);
                    SetLastError(ERROR_NOT_SUPPORTED as DWORD);
                }
            }
        }
    }

    // cleanup:
    if hRootKey != (INVALID_HANDLE_VALUE).cast() && hRootKey != (*set).HKLM {
        RegCloseKey(hRootKey);
    }

    if hKey != (INVALID_HANDLE_VALUE).cast() {
        RegCloseKey(hKey);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSelectDevice(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiRegisterCoDeviceInstallers(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else if DeviceInfoSet == (INVALID_HANDLE_VALUE as HDEVINFO) {
        SetLastError(ERROR_INVALID_HANDLE);
    } else if (*(DeviceInfoSet as *mut DeviceInfoSet)).magic != SETUP_DEVICE_INFO_SET_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE);
    } else if DeviceInfoData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else if (*DeviceInfoData).cbSize != core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD {
        SetLastError(ERROR_INVALID_USER_BUFFER);
    } else {
        let mut InstallParams: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
        let mut SelectedDriver: *mut DriverInfoElement = core::ptr::null_mut();
        let mut Result: BOOL = 0;
        let mut DoAction: DWORD = 0;
        let mut SectionName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut SectionNameLength: DWORD = 0;
        let mut hKey: HKEY = (INVALID_HANDLE_VALUE).cast();
        let mut Context: PVOID = core::ptr::null_mut();

        InstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
        Result = SetupDiGetDeviceInstallParamsW(
            DeviceInfoSet,
            DeviceInfoData,
            core::ptr::addr_of_mut!(InstallParams),
        );

        'cleanup: {
            if Result == 0 {
                break 'cleanup;
            }

            SelectedDriver = InstallParams.ClassInstallReserved as *mut DriverInfoElement;
            if SelectedDriver.is_null() {
                SetLastError(ERROR_NO_DRIVER_SELECTED);
                break 'cleanup;
            }

            /* Get .CoInstallers section name */
            Result = SetupDiGetActualSectionToInstallW(
                (*(*SelectedDriver).InfFileDetails).hInf,
                core::ptr::addr_of_mut!((*SelectedDriver).Details.SectionName) as *mut WCHAR
                    as PCWSTR,
                core::ptr::addr_of_mut!(SectionName) as PWSTR,
                MAX_PATH,
                core::ptr::addr_of_mut!(SectionNameLength),
                core::ptr::null_mut::<PWSTR>(),
            );
            if Result == 0
                || (SectionNameLength as usize)
                    > (MAX_PATH as usize)
                        .wrapping_sub(
                            strlenW(core::ptr::addr_of!(DotCoInstallers) as LPCWSTR) as usize,
                        )
                        .wrapping_sub(1 as usize)
            {
                break 'cleanup;
            }
            lstrcatW(
                core::ptr::addr_of_mut!(SectionName) as LPWSTR,
                core::ptr::addr_of!(DotCoInstallers) as LPCWSTR,
            );

            deviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;

            /* Open/Create driver key information */
            hKey = SETUPDI_OpenDrvKey(
                (*set).HKLM,
                deviceInfo,
                (KEY_READ | KEY_WRITE) as REGSAM,
            );
            if hKey == (INVALID_HANDLE_VALUE).cast()
                && GetLastError() == ERROR_FILE_NOT_FOUND
            {
                hKey = SETUPDI_CreateDrvKey(
                    (*set).HKLM,
                    deviceInfo,
                    core::ptr::addr_of_mut!((*DeviceInfoData).ClassGuid) as *mut UUID,
                    (KEY_READ | KEY_WRITE) as REGSAM,
                );
            }
            if hKey == (INVALID_HANDLE_VALUE).cast() {
                break 'cleanup;
            }

            /* Install .CoInstallers section */
            DoAction = SPINST_REGISTRY;
            if (InstallParams.Flags & DI_NOFILECOPY) == 0 {
                DoAction |= SPINST_FILES;
                Context = SetupInitDefaultQueueCallback(InstallParams.hwndParent);
                if Context.is_null() {
                    break 'cleanup;
                }
            }
            Result = SetupInstallFromInfSectionW(
                InstallParams.hwndParent,
                (*(*SelectedDriver).InfFileDetails).hInf,
                core::ptr::addr_of_mut!(SectionName) as PCWSTR,
                DoAction as UINT,
                hKey,
                (*(*SelectedDriver).InfFileDetails).DirectoryName,
                SP_COPY_NEWER as UINT,
                Some(
                    SetupDefaultQueueCallbackW,
                ),
                Context,
                DeviceInfoSet,
                DeviceInfoData,
            );
            if Result == 0 {
                break 'cleanup;
            }

            ret = (TRUE as i32);
        }

        /* cleanup: */
        if !Context.is_null() {
            SetupTermDefaultQueueCallback(Context);
        }
        if hKey != (INVALID_HANDLE_VALUE).cast() {
            RegCloseKey(hKey);
        }
    }

    ret
}

unsafe extern "C" fn InfIsFromOEMLocation(FullName: PCWSTR, IsOEMLocation: LPBOOL) -> BOOL {
    let mut last: PWCHAR;

    last = strrchrW(FullName, '\\' as WCHAR) as PWCHAR;
    if last.is_null() {
        /* No directory specified */
        *IsOEMLocation = (FALSE as i32);
    } else {
        let mut Windir: LPWSTR;
        let mut ret: UINT;

        Windir = MyMalloc(
            ((MAX_PATH + 1
                + (strlenW(core::ptr::addr_of!(InfDirectory) as PCWSTR)) as u32)
                * (core::mem::size_of::<WCHAR>()) as u32) as DWORD,
        ) as LPWSTR;
        if Windir.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return (FALSE) as BOOL;
        }

        ret = GetSystemWindowsDirectoryW(Windir, MAX_PATH);
        if ret == 0 || ret > MAX_PATH {
            MyFree(Windir as PVOID);
            SetLastError(ERROR_GEN_FAILURE);
            return (FALSE) as BOOL;
        }
        if *Windir != 0
            && *Windir.offset(
                (strlenW(Windir as PCWSTR) - 1) as isize,
            ) != '\\' as WCHAR
        {
            strcatW(Windir, core::ptr::addr_of!(BackSlash) as PCWSTR);
        }
        strcatW(Windir, core::ptr::addr_of!(InfDirectory) as PCWSTR);

        if strncmpiW(
            FullName,
            Windir as PCWSTR,
            last.offset_from(FullName) as usize,
        ) == 0
        {
            /* The path is %SYSTEMROOT%\Inf */
            *IsOEMLocation = (FALSE as i32);
        } else {
            /* The file is in another place */
            *IsOEMLocation = (TRUE as i32);
        }
        MyFree(Windir as PVOID);
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiInstallDevice(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut();
    let mut InstallParams: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
    let mut SelectedDriver: *mut DriverInfoElement = core::ptr::null_mut();
    let mut DriverDate: SYSTEMTIME = core::mem::zeroed();
    let mut SectionName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut Buffer: [WCHAR; 32] = [0; 32];
    let mut SectionNameLength: DWORD = 0;
    let mut Result: BOOL = (FALSE as i32);
    let mut DoAction: ULONG = 0;
    let mut RequiredSize: DWORD = 0;
    let mut pSectionName: LPWSTR = core::ptr::null_mut();
    let mut ClassName: [WCHAR; (MAX_CLASS_NAME_LEN as _)] = [0; (MAX_CLASS_NAME_LEN as _)];
    let mut ClassGuid: GUID = core::mem::zeroed();
    let mut lpGuidString: LPWSTR = core::ptr::null_mut();
    let mut lpFullGuidString: LPWSTR = core::ptr::null_mut();
    let mut RebootRequired: BOOL = (FALSE as i32);
    let mut hKey: HKEY = (INVALID_HANDLE_VALUE).cast();
    let mut NeedtoCopyFile: BOOL = 0;
    let mut fullVersion: LARGE_INTEGER = core::mem::zeroed();
    let mut rc: LONG = 0;
    let mut Context: PVOID = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    'cleanup: {
        if DeviceInfoSet.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
        } else if DeviceInfoSet == INVALID_HANDLE_VALUE as HDEVINFO {
            SetLastError(ERROR_INVALID_HANDLE);
        } else if (*(DeviceInfoSet as *mut DeviceInfoSet)).magic != SETUP_DEVICE_INFO_SET_MAGIC {
            SetLastError(ERROR_INVALID_HANDLE);
        } else if !DeviceInfoData.is_null()
            && (*DeviceInfoData).cbSize as usize
                != core::mem::size_of::<SP_DEVINFO_DATA>()
        {
            SetLastError(ERROR_INVALID_USER_BUFFER);
        } else {
            Result = (TRUE as i32);
        }

        if Result == 0 {
            /* One parameter is bad */
            break 'cleanup;
        }

        InstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
        Result = SetupDiGetDeviceInstallParamsW(DeviceInfoSet, DeviceInfoData, &raw mut InstallParams);
        if Result == 0 {
            break 'cleanup;
        }

        if (InstallParams.FlagsEx & DI_FLAGSEX_SETFAILEDINSTALL) != 0 {
            /* Set FAILEDINSTALL in ConfigFlags registry value */
            let mut ConfigFlags: DWORD = 0;
            let mut regType: DWORD = 0;
            Result = SetupDiGetDeviceRegistryPropertyW(
                DeviceInfoSet,
                DeviceInfoData,
                SPDRP_CONFIGFLAGS,
                &raw mut regType,
                &raw mut ConfigFlags as PBYTE,
                core::mem::size_of::<DWORD>() as DWORD,
                core::ptr::null_mut(),
            );
            if Result == 0 || regType != REG_DWORD {
                SetLastError(ERROR_GEN_FAILURE);
                break 'cleanup;
            }
            ConfigFlags |= CONFIGFLAG_FAILEDINSTALL;
            Result = SetupDiSetDeviceRegistryPropertyW(
                DeviceInfoSet,
                DeviceInfoData,
                SPDRP_CONFIGFLAGS,
                &raw mut ConfigFlags as PBYTE as *const BYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            );
            if Result == 0 {
                SetLastError(ERROR_GEN_FAILURE);
                break 'cleanup;
            }

            ret = (TRUE as i32);
            break 'cleanup;
        }

        SelectedDriver = InstallParams.ClassInstallReserved as *mut DriverInfoElement;
        if SelectedDriver.is_null() {
            SetLastError(ERROR_NO_DRIVER_SELECTED);
            break 'cleanup;
        }

        FileTimeToSystemTime(&raw mut (*SelectedDriver).Info.DriverDate, &raw mut DriverDate);

        Result = SetupDiGetActualSectionToInstallW(
            (*(*SelectedDriver).InfFileDetails).hInf,
            &raw mut (*SelectedDriver).Details.SectionName as *mut WCHAR as PCWSTR,
            &raw mut SectionName as *mut WCHAR as PWSTR,
            MAX_PATH,
            &raw mut SectionNameLength,
            core::ptr::null_mut(),
        );
        if Result == 0
            || (SectionNameLength as usize)
                > ((MAX_PATH as usize)
                    - (strlenW(&raw const DotServices as *const WCHAR) as usize))
        {
            break 'cleanup;
        }
        pSectionName = (&raw mut SectionName as *mut WCHAR)
            .offset(strlenW(&raw const SectionName as *const WCHAR) as isize)
            as *mut WCHAR as LPWSTR;

        /* Get information from [Version] section */
        if SetupDiGetINFClassW(
            &raw mut (*SelectedDriver).Details.InfFileName as *mut WCHAR as PCWSTR,
            &raw mut ClassGuid,
            &raw mut ClassName as *mut WCHAR as PWSTR,
            MAX_CLASS_NAME_LEN,
            &raw mut RequiredSize,
        ) == 0
        {
            break 'cleanup;
        }
        /* Format ClassGuid to a string */
        if UuidToStringW(&raw mut ClassGuid as *mut UUID, &raw mut lpGuidString)
            != RPC_S_OK
        {
            break 'cleanup;
        }
        RequiredSize = lstrlenW(lpGuidString as LPCWSTR) as DWORD;
        lpFullGuidString = HeapAlloc(
            GetProcessHeap(),
            0,
            (((RequiredSize + 3) as usize)
                * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as LPWSTR;
        if lpFullGuidString.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        }
        *lpFullGuidString.offset(0 as isize) = '{' as WCHAR;
        memcpy(
            lpFullGuidString.offset(1 as isize) as *mut core::ffi::c_void,
            lpGuidString as *const core::ffi::c_void,
            (RequiredSize as usize) * core::mem::size_of::<WCHAR>(),
        );
        *lpFullGuidString.offset((RequiredSize + 1) as isize) = '}' as WCHAR;
        *lpFullGuidString.offset((RequiredSize + 2) as isize) = '\0' as WCHAR;

        /* Copy .inf file to Inf\ directory (if needed) */
        Result = InfIsFromOEMLocation(
            &raw mut (*SelectedDriver).Details.InfFileName as *mut WCHAR as PCWSTR,
            &raw mut NeedtoCopyFile,
        );
        if Result == 0 {
            break 'cleanup;
        }
        if NeedtoCopyFile != 0 {
            let mut NewFileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut newInfFileDetails: *mut InfFileDetails = core::ptr::null_mut();
            Result = SetupCopyOEMInfW(
                &raw mut (*SelectedDriver).Details.InfFileName as *mut WCHAR as PCWSTR,
                core::ptr::null(),
                SPOST_NONE,
                SP_COPY_NOOVERWRITE,
                &raw mut NewFileName as *mut WCHAR as PWSTR,
                MAX_PATH,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if Result == 0 && GetLastError() != ERROR_FILE_EXISTS {
                break 'cleanup;
            }
            /* Create a new struct InfFileDetails, and set it to
             * SelectedDriver->InfFileDetails, to release use of
             * current InfFile */
            newInfFileDetails =
                CreateInfFileDetails(&raw mut NewFileName as *mut WCHAR as LPCWSTR);
            if newInfFileDetails.is_null() {
                break 'cleanup;
            }
            DereferenceInfFile((*SelectedDriver).InfFileDetails);
            (*SelectedDriver).InfFileDetails = newInfFileDetails;
            strcpyW(
                &raw mut (*SelectedDriver).Details.InfFileName as *mut WCHAR,
                &raw mut NewFileName as *mut WCHAR,
            );
        }

        deviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;

        /* Open/Create driver key information */
        hKey = SETUPDI_OpenDrvKey(
            (*set).HKLM,
            deviceInfo,
            KEY_READ | KEY_WRITE,
        );
        if hKey == (INVALID_HANDLE_VALUE).cast() && GetLastError() == ERROR_FILE_NOT_FOUND {
            hKey = SETUPDI_CreateDrvKey(
                (*set).HKLM,
                deviceInfo,
                &raw mut (*DeviceInfoData).ClassGuid as *mut UUID,
                KEY_READ | KEY_WRITE,
            );
        }
        if hKey == (INVALID_HANDLE_VALUE).cast() {
            break 'cleanup;
        }

        /* Install main section */
        DoAction = 0;
        if (InstallParams.FlagsEx & DI_FLAGSEX_NO_DRVREG_MODIFY) == 0 {
            DoAction |= SPINST_REGISTRY;
        }
        if (InstallParams.Flags & DI_NOFILECOPY) == 0 {
            DoAction |= SPINST_FILES;
            Context = SetupInitDefaultQueueCallback(InstallParams.hwndParent);
            if Context.is_null() {
                break 'cleanup;
            }
        }
        *pSectionName = '\0' as WCHAR;
        Result = SetupInstallFromInfSectionW(
            InstallParams.hwndParent,
            (*(*SelectedDriver).InfFileDetails).hInf,
            &raw mut SectionName as *mut WCHAR as PCWSTR,
            DoAction,
            hKey,
            (*(*SelectedDriver).InfFileDetails).DirectoryName,
            SP_COPY_NEWER,
            Some(SetupDefaultQueueCallbackW),
            Context,
            DeviceInfoSet,
            DeviceInfoData,
        );
        if Result == 0 {
            break 'cleanup;
        }
        InstallParams.Flags |= DI_NOFILECOPY;
        SetupDiSetDeviceInstallParamsW(DeviceInfoSet, DeviceInfoData, &raw mut InstallParams);

        /* Write information to driver key */
        *pSectionName = UNICODE_NULL;
        memcpy(
            &raw mut fullVersion as *mut core::ffi::c_void,
            &raw mut (*SelectedDriver).Info.DriverVersion as *const core::ffi::c_void,
            core::mem::size_of::<LARGE_INTEGER>(),
        );
        sprintfW(
            &raw mut Buffer as *mut WCHAR,
            &raw const DateFormat as *const WCHAR,
            DriverDate.wMonth as core::ffi::c_int,
            DriverDate.wDay as core::ffi::c_int,
            DriverDate.wYear as core::ffi::c_int,
        );
        rc = RegSetValueExW(
            hKey,
            &raw const REGSTR_DRIVER_DATE as LPCWSTR,
            0,
            REG_SZ,
            &raw mut Buffer as *mut WCHAR as *const BYTE,
            ((strlenW(&raw const Buffer as *const WCHAR) as usize + 1 as usize)
                * core::mem::size_of::<WCHAR>()) as DWORD,
        );
        if rc == (ERROR_SUCCESS as LONG) {
            rc = RegSetValueExW(
                hKey,
                &raw const REGSTR_DRIVER_DATE_DATA as LPCWSTR,
                0,
                REG_BINARY,
                &raw mut (*SelectedDriver).Info.DriverDate as *const BYTE,
                core::mem::size_of::<FILETIME>() as DWORD,
            );
        }
        if rc == (ERROR_SUCCESS as LONG) {
            rc = RegSetValueExW(
                hKey,
                REGSTR_VAL_DRVDESC,
                0,
                REG_SZ,
                &raw mut (*SelectedDriver).Info.Description as *mut WCHAR as *const BYTE,
                ((strlenW(&raw const (*SelectedDriver).Info.Description as *const WCHAR) as usize
                    + 1 as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            );
        }
        if rc == (ERROR_SUCCESS as LONG) {
            sprintfW(
                &raw mut Buffer as *mut WCHAR,
                &raw const VersionFormat as *const WCHAR,
                (fullVersion.__bindgen_anon_1.HighPart >> 16) as core::ffi::c_int,
                (fullVersion.__bindgen_anon_1.HighPart & 0xffff) as core::ffi::c_int,
                (fullVersion.__bindgen_anon_1.LowPart >> 16) as core::ffi::c_int,
                (fullVersion.__bindgen_anon_1.LowPart & 0xffff) as core::ffi::c_int,
            );
            rc = RegSetValueExW(
                hKey,
                &raw const REGSTR_DRIVER_VERSION as LPCWSTR,
                0,
                REG_SZ,
                &raw mut Buffer as *mut WCHAR as *const BYTE,
                ((strlenW(&raw const Buffer as *const WCHAR) as usize + 1 as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            );
        }
        if rc == (ERROR_SUCCESS as LONG) {
            rc = RegSetValueExW(
                hKey,
                REGSTR_VAL_INFPATH,
                0,
                REG_SZ,
                (*(*SelectedDriver).InfFileDetails).FileName as *const BYTE,
                ((strlenW((*(*SelectedDriver).InfFileDetails).FileName as *const WCHAR) as usize
                    + 1 as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            );
        }
        if rc == (ERROR_SUCCESS as LONG) {
            rc = RegSetValueExW(
                hKey,
                REGSTR_VAL_INFSECTION,
                0,
                REG_SZ,
                &raw mut (*SelectedDriver).Details.SectionName as *mut WCHAR as *const BYTE,
                ((strlenW(&raw const (*SelectedDriver).Details.SectionName as *const WCHAR)
                    as usize
                    + 1 as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            );
        }
        if rc == (ERROR_SUCCESS as LONG) {
            rc = RegSetValueExW(
                hKey,
                REGSTR_VAL_INFSECTIONEXT,
                0,
                REG_SZ,
                (&raw mut SectionName as *mut WCHAR).offset(strlenW(
                    &raw const (*SelectedDriver).Details.SectionName as *const WCHAR,
                ) as isize) as *const BYTE,
                ((strlenW(&raw const SectionName as *const WCHAR) as usize
                    - strlenW(&raw const (*SelectedDriver).Details.SectionName as *const WCHAR)
                        as usize
                    + 1 as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            );
        }
        if rc == (ERROR_SUCCESS as LONG) {
            rc = RegSetValueExW(
                hKey,
                REGSTR_VAL_MATCHINGDEVID,
                0,
                REG_SZ,
                (*SelectedDriver).MatchingId as *const BYTE,
                ((strlenW((*SelectedDriver).MatchingId as *const WCHAR) as usize
                    + 1 as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            );
        }
        if rc == (ERROR_SUCCESS as LONG) {
            rc = RegSetValueExW(
                hKey,
                REGSTR_VAL_PROVIDER_NAME,
                0,
                REG_SZ,
                &raw mut (*SelectedDriver).Info.ProviderName as *mut WCHAR as *const BYTE,
                ((strlenW(&raw const (*SelectedDriver).Info.ProviderName as *const WCHAR) as usize
                    + 1 as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            );
        }
        if rc != (ERROR_SUCCESS as LONG) {
            SetLastError((rc) as DWORD);
            break 'cleanup;
        }
        RegCloseKey(hKey);
        hKey = (INVALID_HANDLE_VALUE).cast();

        /* FIXME: Process .LogConfigOverride section */

        /* Install .Services section */
        strcpyW(pSectionName, &raw const DotServices as *const WCHAR);
        Result = SetupInstallServicesFromInfSectionExW(
            (*(*SelectedDriver).InfFileDetails).hInf,
            &raw mut SectionName as *mut WCHAR as PCWSTR,
            0,
            DeviceInfoSet,
            DeviceInfoData,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if Result == 0 {
            if GetLastError() != ERROR_SECTION_NOT_FOUND {
                break 'cleanup;
            }
            SetLastError(ERROR_SUCCESS);
        }
        if GetLastError() == ERROR_SUCCESS_REBOOT_REQUIRED {
            RebootRequired = (TRUE as i32);
        }

        /* Open device registry key */
        hKey = SETUPDI_OpenDevKey(
            (*(DeviceInfoSet as *mut DeviceInfoSet)).HKLM,
            (*DeviceInfoData).Reserved as *mut DeviceInfo,
            KEY_SET_VALUE,
        );
        if hKey == (INVALID_HANDLE_VALUE).cast() {
            break 'cleanup;
        }

        /* Install .HW section */
        DoAction = 0;
        if (InstallParams.FlagsEx & DI_FLAGSEX_NO_DRVREG_MODIFY) == 0 {
            DoAction |= SPINST_REGISTRY;
        }
        strcpyW(pSectionName, &raw const DotHW as *const WCHAR);
        Result = SetupInstallFromInfSectionW(
            InstallParams.hwndParent,
            (*(*SelectedDriver).InfFileDetails).hInf,
            &raw mut SectionName as *mut WCHAR as PCWSTR,
            DoAction,
            hKey,
            core::ptr::null(),
            0,
            None,
            core::ptr::null_mut(),
            DeviceInfoSet,
            DeviceInfoData,
        );
        if Result == 0 {
            break 'cleanup;
        }

        /* Write information to enum key */
        rc = RegSetValueExW(
            hKey,
            REGSTR_VAL_CLASS,
            0,
            REG_SZ,
            &raw mut ClassName as *mut WCHAR as *const BYTE,
            ((strlenW(&raw const ClassName as *const WCHAR) as usize + 1 as usize)
                * core::mem::size_of::<WCHAR>()) as DWORD,
        );
        if rc == (ERROR_SUCCESS as LONG) {
            rc = RegSetValueExW(
                hKey,
                REGSTR_VAL_CLASSGUID,
                0,
                REG_SZ,
                lpFullGuidString as *const BYTE,
                ((strlenW(lpFullGuidString as *const WCHAR) as usize + 1 as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            );
        }
        if rc == (ERROR_SUCCESS as LONG) {
            rc = RegSetValueExW(
                hKey,
                REGSTR_VAL_DEVDESC,
                0,
                REG_SZ,
                &raw mut (*SelectedDriver).Info.Description as *mut WCHAR as *const BYTE,
                ((strlenW(&raw const (*SelectedDriver).Info.Description as *const WCHAR) as usize
                    + 1 as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            );
        }
        if rc == (ERROR_SUCCESS as LONG) {
            rc = RegSetValueExW(
                hKey,
                REGSTR_VAL_MFG,
                0,
                REG_SZ,
                &raw mut (*SelectedDriver).Info.MfgName as *mut WCHAR as *const BYTE,
                ((strlenW(&raw const (*SelectedDriver).Info.MfgName as *const WCHAR) as usize
                    + 1 as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            );
        }
        if rc != (ERROR_SUCCESS as LONG) {
            SetLastError((rc) as DWORD);
            break 'cleanup;
        }

        /* Start the device */
        if RebootRequired == 0
            && (InstallParams.Flags
                & (DI_NEEDRESTART | DI_NEEDREBOOT | DI_DONOTCALLCONFIGMG))
                == 0
        {
            ret = ResetDevice(DeviceInfoSet, DeviceInfoData);
        } else {
            ret = (TRUE as i32);
        }
    }

    /* End of installation */
    if hKey != (INVALID_HANDLE_VALUE).cast() {
        RegCloseKey(hKey);
    }
    if !lpGuidString.is_null() {
        RpcStringFreeW(&raw mut lpGuidString);
    }
    HeapFree(
        GetProcessHeap(),
        0,
        lpFullGuidString as PVOID,
    );
    if !Context.is_null() {
        SetupTermDefaultQueueCallback(Context);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn SETUPDI_CreateDevKey(
    mut RootKey: HKEY,
    mut devInfo: *mut DeviceInfo,
    mut samDesired: REGSAM,
) -> HKEY {
    let mut enumKey: HKEY = core::ptr::null_mut();
    let mut key: HKEY = (INVALID_HANDLE_VALUE).cast();
    let mut l: LONG;

    l = RegCreateKeyExW(
        RootKey,
        REGSTR_PATH_SYSTEMENUM,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_CREATE_SUB_KEY,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(enumKey),
        core::ptr::null_mut(),
    );
    if l == 0 {
        l = RegCreateKeyExW(
            enumKey,
            (*devInfo).instanceId as LPCWSTR,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            samDesired,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(key),
            core::ptr::null_mut(),
        );
        RegCloseKey(enumKey);
    }
    if l != 0 {
        SetLastError(l as DWORD);
    }
    return key;
}
#[no_mangle]
pub unsafe extern "C" fn SETUPDI_CreateDrvKey(
    RootKey: HKEY,
    devInfo: *mut DeviceInfo,
    ClassGuid: *mut UUID,
    samDesired: REGSAM,
) -> HKEY {
    let mut key: HKEY = (INVALID_HANDLE_VALUE) as HKEY;
    let mut lpGuidString: LPWSTR = core::ptr::null_mut();
    let mut DriverKey: LPWSTR = core::ptr::null_mut(); /* {GUID}\Index */
    let mut pDeviceInstance: LPWSTR = core::ptr::null_mut(); /* Points into DriverKey, on the Index field */
    let mut Index: DWORD = 0; /* Index used in the DriverKey name */
    let mut dwSize: DWORD = 0;
    let mut Disposition: DWORD = 0;
    let mut rc: DWORD = 0;
    let mut hHWProfileKey: HKEY = (INVALID_HANDLE_VALUE) as HKEY;
    let mut hEnumKey: HKEY = core::ptr::null_mut();
    let mut hClassKey: HKEY = core::ptr::null_mut();
    let mut hDeviceKey: HKEY = (INVALID_HANDLE_VALUE) as HKEY;
    let mut hKey: HKEY = core::ptr::null_mut();

    /* Open device key, to read Driver value */
    'cleanup: {
        hDeviceKey = SETUPDI_OpenDevKey(
            RootKey,
            devInfo,
            (KEY_QUERY_VALUE | KEY_SET_VALUE) as REGSAM,
        );
        if hDeviceKey == (INVALID_HANDLE_VALUE) as HKEY {
            break 'cleanup;
        }

        rc = (RegOpenKeyExW(
            RootKey,
            REGSTR_PATH_CLASS_NT,
            0,
            KEY_CREATE_SUB_KEY,
            &raw mut hClassKey,
        )) as u32;
        if rc != ERROR_SUCCESS {
            SetLastError(rc);
            break 'cleanup;
        }

        rc = (RegQueryValueExW(
            hDeviceKey,
            REGSTR_VAL_DRIVER,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut dwSize,
        )) as u32;
        if rc != ERROR_SUCCESS {
            /* Create a new driver key */

            if UuidToStringW(ClassGuid, &raw mut lpGuidString) != RPC_S_OK {
                break 'cleanup;
            }

            /* The driver key is in \System\CurrentControlSet\Control\Class\{GUID}\Index */
            DriverKey = HeapAlloc(
                GetProcessHeap(),
                0,
                ((strlenW(lpGuidString) as SIZE_T + (7) as SIZE_T)
                    * core::mem::size_of::<WCHAR>() as SIZE_T)
                    + core::mem::size_of_val(&UNICODE_NULL) as SIZE_T,
            ) as LPWSTR;
            if DriverKey.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }

            *DriverKey.add((0) as usize) = '{' as WCHAR;
            strcpyW(
                DriverKey.add((1) as usize),
                lpGuidString,
            );
            pDeviceInstance = DriverKey.add(strlenW(DriverKey) as usize);
            *pDeviceInstance = '}' as WCHAR;
            pDeviceInstance = pDeviceInstance.add(1);
            *pDeviceInstance = '\\' as WCHAR;
            pDeviceInstance = pDeviceInstance.add(1);

            /* Try all values for Index between 0 and 9999 */
            Index = 0;
            while Index <= (9999) {
                sprintfW(
                    pDeviceInstance,
                    core::ptr::addr_of!(InstanceKeyFormat) as *const WCHAR,
                    Index,
                );
                rc = (RegCreateKeyExW(
                    hClassKey,
                    DriverKey,
                    0,
                    core::ptr::null_mut(),
                    REG_OPTION_NON_VOLATILE,
                    (KEY_READ | KEY_WRITE),
                    core::ptr::null_mut(),
                    &raw mut hKey,
                    &raw mut Disposition,
                )) as u32;
                if rc != ERROR_SUCCESS {
                    SetLastError(rc);
                    break 'cleanup;
                }
                if Disposition == REG_CREATED_NEW_KEY {
                    break;
                }
                RegCloseKey(hKey);
                hKey = core::ptr::null_mut();
                Index += 1;
            }

            if Index > (9999) {
                /* Unable to create more than 9999 devices within the same class */
                SetLastError(ERROR_GEN_FAILURE);
                break 'cleanup;
            }

            /* Write the new Driver value */
            rc = (RegSetValueExW(
                hDeviceKey,
                REGSTR_VAL_DRIVER,
                0,
                REG_SZ,
                DriverKey as *const BYTE,
                ((strlenW(DriverKey) as usize + (1) as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            )) as u32;
            if rc != ERROR_SUCCESS {
                SetLastError(rc);
                break 'cleanup;
            }
        } else {
            /* Open the existing driver key */

            DriverKey = HeapAlloc(
                GetProcessHeap(),
                0,
                dwSize as SIZE_T,
            ) as LPWSTR;
            if DriverKey.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }

            rc = (RegQueryValueExW(
                hDeviceKey,
                REGSTR_VAL_DRIVER,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                DriverKey as LPBYTE,
                &raw mut dwSize,
            )) as u32;
            if rc != ERROR_SUCCESS {
                SetLastError(rc);
                break 'cleanup;
            }

            rc = (RegCreateKeyExW(
                hClassKey,
                DriverKey,
                0,
                core::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                (KEY_READ | KEY_WRITE),
                core::ptr::null_mut(),
                &raw mut hKey,
                &raw mut Disposition,
            )) as u32;
            if rc != ERROR_SUCCESS {
                SetLastError(rc);
                break 'cleanup;
            }
        }

        key = hKey;
    }

    /* cleanup: */
    if !lpGuidString.is_null() {
        RpcStringFreeW(&raw mut lpGuidString);
    }
    HeapFree(GetProcessHeap(), 0, DriverKey as PVOID);
    if hHWProfileKey != (INVALID_HANDLE_VALUE) as HKEY {
        RegCloseKey(hHWProfileKey);
    }
    if !hEnumKey.is_null() {
        RegCloseKey(hEnumKey);
    }
    if !hClassKey.is_null() {
        RegCloseKey(hClassKey);
    }
    if hDeviceKey != (INVALID_HANDLE_VALUE) as HKEY {
        RegCloseKey(hDeviceKey);
    }
    if !hKey.is_null() && hKey != key {
        RegCloseKey(hKey);
    }

    return hKey;
}
#[no_mangle]
pub unsafe extern "C" fn SETUPDI_OpenDevKey(RootKey: HKEY, devInfo: *mut DeviceInfo, samDesired: REGSAM) -> HKEY {
    let mut enumKey: HKEY = core::ptr::null_mut();
    let mut key: HKEY = INVALID_HANDLE_VALUE as HKEY;
    let mut l: LONG = 0;

    l = RegOpenKeyExW(RootKey, REGSTR_PATH_SYSTEMENUM as LPCWSTR, 0 as DWORD, READ_CONTROL as REGSAM, core::ptr::addr_of_mut!(enumKey)) as LONG;
    if l == 0 {
        l = RegOpenKeyExW(enumKey, (*devInfo).instanceId as LPCWSTR, 0 as DWORD, samDesired, core::ptr::addr_of_mut!(key)) as LONG;
        RegCloseKey(enumKey);
    }
    if l != 0 {
        SetLastError(l as DWORD);
    }
    return key;
}
#[no_mangle]
pub unsafe extern "C" fn SETUPDI_OpenDrvKey(mut RootKey: HKEY, mut devInfo: *mut DeviceInfo, mut samDesired: REGSAM) -> HKEY {
    let mut DriverKey: LPWSTR = core::ptr::null_mut();
    let mut dwLength: DWORD = 0 as DWORD;
    let mut dwRegType: DWORD = 0;
    let mut rc: DWORD = 0;
    let mut hEnumKey: HKEY = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut key: HKEY = INVALID_HANDLE_VALUE as HKEY;

    'cleanup: {
        hKey = SETUPDI_OpenDevKey(RootKey, devInfo, KEY_QUERY_VALUE as REGSAM);
        if hKey == INVALID_HANDLE_VALUE as HKEY {
            break 'cleanup;
        }
        /* Read the 'Driver' key */
        rc = RegQueryValueExW(
            hKey,
            REGSTR_VAL_DRIVER as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwRegType),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwLength),
        ) as DWORD;
        if rc != ERROR_SUCCESS as DWORD {
            SetLastError(rc);
            break 'cleanup;
        } else if dwRegType != REG_SZ as DWORD {
            SetLastError(ERROR_GEN_FAILURE as DWORD);
            break 'cleanup;
        }
        DriverKey = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            dwLength as SIZE_T,
        ) as LPWSTR;
        if DriverKey.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            break 'cleanup;
        }
        rc = RegQueryValueExW(
            hKey,
            REGSTR_VAL_DRIVER as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwRegType),
            DriverKey as LPBYTE,
            core::ptr::addr_of_mut!(dwLength),
        ) as DWORD;
        if rc != ERROR_SUCCESS as DWORD {
            SetLastError(rc);
            break 'cleanup;
        }
        RegCloseKey(hKey);
        hKey = core::ptr::null_mut();
        /* Need to open the driver key */
        rc = RegOpenKeyExW(
            RootKey,
            REGSTR_PATH_CLASS_NT as LPCWSTR,
            0 as DWORD, /* Options */
            READ_CONTROL as REGSAM,
            core::ptr::addr_of_mut!(hEnumKey),
        ) as DWORD;
        if rc != ERROR_SUCCESS as DWORD {
            SetLastError(rc);
            break 'cleanup;
        }
        rc = RegOpenKeyExW(
            hEnumKey,
            DriverKey as LPCWSTR,
            0 as DWORD, /* Options */
            samDesired,
            core::ptr::addr_of_mut!(hKey),
        ) as DWORD;
        if rc != ERROR_SUCCESS as DWORD {
            SetLastError(rc);
            break 'cleanup;
        }
        key = hKey;
    }

    if !hEnumKey.is_null() {
        RegCloseKey(hEnumKey);
    }
    if !hKey.is_null() && hKey != key {
        RegCloseKey(hKey);
    }
    if !DriverKey.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, DriverKey as PVOID);
    }
    return key;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiOpenDevRegKey(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Scope: DWORD,
    HwProfile: DWORD,
    KeyType: DWORD,
    samDesired: REGSAM,
) -> HKEY {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut devInfo: *mut DeviceInfo = ::core::ptr::null_mut::<DeviceInfo>();
    let mut key: HKEY = INVALID_HANDLE_VALUE as HKEY;
    let mut RootKey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiOpenDevRegKey\0" as *const u8 as *const ::core::ffi::c_char,
            6245 as ::core::ffi::c_int,
            b"%s(%p %p %d %d %d %x)\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiOpenDevRegKey\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            DeviceInfoData,
            Scope,
            HwProfile,
            KeyType,
            samDesired,
        );
    }

    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE);
        return INVALID_HANDLE_VALUE as HKEY;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE);
        return INVALID_HANDLE_VALUE as HKEY;
    }
    if DeviceInfoData.is_null()
        || (*DeviceInfoData).cbSize as usize != ::core::mem::size_of::<SP_DEVINFO_DATA>() as usize
        || (*DeviceInfoData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE as HKEY;
    }
    if Scope != DICS_FLAG_GLOBAL && Scope != DICS_FLAG_CONFIGSPECIFIC {
        SetLastError(ERROR_INVALID_FLAGS);
        return INVALID_HANDLE_VALUE as HKEY;
    }
    if KeyType != DIREG_DEV && KeyType != DIREG_DRV {
        SetLastError(ERROR_INVALID_FLAGS);
        return INVALID_HANDLE_VALUE as HKEY;
    }
    devInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
    if (*devInfo).set != set {
        SetLastError(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE as HKEY;
    }
    if Scope != DICS_FLAG_GLOBAL {
        RootKey = OpenHardwareProfileKey((*set).HKLM, HwProfile, 0);
        if RootKey == INVALID_HANDLE_VALUE as HKEY {
            return INVALID_HANDLE_VALUE as HKEY;
        }
    } else {
        RootKey = (*set).HKLM;
    }
    match KeyType {
        DIREG_DEV => {
            key = SETUPDI_OpenDevKey(RootKey, devInfo, samDesired);
            if Scope == DICS_FLAG_GLOBAL {
                let mut rc: LONG = 0;
                let mut hTempKey: HKEY = key;
                rc = RegOpenKeyExW(
                    hTempKey,
                    RUSTCONV_WSTR_00A6C7A05271.as_ptr(),
                    0,
                    samDesired,
                    core::ptr::addr_of_mut!(key),
                ) as LONG;
                if rc == (ERROR_SUCCESS as LONG) {
                    RegCloseKey(hTempKey);
                }
            }
        }
        DIREG_DRV => {
            key = SETUPDI_OpenDrvKey(RootKey, devInfo, samDesired);
        }
        _ => {
            WARN!("unknown KeyType %d\n", KeyType);
        }
    }
    if RootKey != (*set).HKLM {
        RegCloseKey(RootKey);
    }
    return key;
}

unsafe extern "C" fn SETUPDI_DeleteDevKey(RootKey: HKEY, devInfo: *mut DeviceInfo) -> BOOL {
    // FIXME("\n");
    (FALSE as i32)
}

unsafe extern "C" fn SETUPDI_DeleteDrvKey(RootKey: HKEY, devInfo: *mut DeviceInfo) -> BOOL {
    // FIXME("\n");
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiDeleteDevRegKey(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Scope: DWORD,
    HwProfile: DWORD,
    KeyType: DWORD,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut devInfo: *mut DeviceInfo = core::ptr::null_mut::<DeviceInfo>();
    let mut ret: BOOL = (FALSE as i32);
    let mut RootKey: HKEY = core::ptr::null_mut::<HKEY__>();

    TRACE!("%s(%p %p %d %d %d)\n", "SetupDiDeleteDevRegKey", DeviceInfoSet, DeviceInfoData, Scope, HwProfile, KeyType);

    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }
    if DeviceInfoData.is_null()
        || (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
        || (*DeviceInfoData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    if Scope != DICS_FLAG_GLOBAL && Scope != DICS_FLAG_CONFIGSPECIFIC {
        SetLastError(ERROR_INVALID_FLAGS);
        return (FALSE) as BOOL;
    }
    if KeyType != DIREG_DEV && KeyType != DIREG_DRV && KeyType != DIREG_BOTH {
        SetLastError(ERROR_INVALID_FLAGS);
        return (FALSE) as BOOL;
    }
    devInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
    if (*devInfo).set != set {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    if Scope != DICS_FLAG_GLOBAL {
        RootKey = OpenHardwareProfileKey((*set).HKLM, HwProfile, 0);
        if RootKey == (INVALID_HANDLE_VALUE).cast() {
            return FALSE as BOOL;
        }
    } else {
        RootKey = (*set).HKLM;
    }
    if KeyType == DIREG_DEV {
        ret = SETUPDI_DeleteDevKey(RootKey, devInfo);
    } else if KeyType == DIREG_DRV {
        ret = SETUPDI_DeleteDrvKey(RootKey, devInfo);
    } else if KeyType == DIREG_BOTH {
        ret = SETUPDI_DeleteDevKey(RootKey, devInfo);
        if ret != 0 {
            ret = SETUPDI_DeleteDrvKey(RootKey, devInfo);
        }
    } else {
        WARN!("unknown KeyType %d\n", KeyType);
    }
    if RootKey != (*set).HKLM {
        RegCloseKey(RootKey);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiRestartDevices(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut devInfo: *mut DeviceInfo = core::ptr::null_mut::<DeviceInfo>();
    let mut cr: CONFIGRET = 0;

    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }

    if DeviceInfoData.is_null()
        || (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
        || (*DeviceInfoData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    devInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;

    cr = CM_Enable_DevNode_Ex((*devInfo).dnDevInst, 0, (*set).hMachine);
    if cr != CR_SUCCESS {
        SetLastError(GetErrorCodeFromCrCode(cr));
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetCustomDevicePropertyA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    CustomPropertyName: PCSTR,
    Flags: DWORD,
    PropertyRegDataType: PDWORD,
    PropertyBuffer: PBYTE,
    PropertyBufferSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut();
    let mut ConfigFlags: DWORD = 0;
    let mut PropertySize: DWORD = 0;
    let mut cr: CONFIGRET;

    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }
    if DeviceInfoData.is_null()
        || (*DeviceInfoData).cbSize != core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD
        || (*DeviceInfoData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    if (Flags & !DICUSTOMDEVPROP_MERGE_MULTISZ) != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return (FALSE) as BOOL;
    }

    deviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
    if (*deviceInfo).set != set {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    if (Flags & DICUSTOMDEVPROP_MERGE_MULTISZ) != 0 {
        ConfigFlags |= CM_CUSTOMDEVPROP_MERGE_MULTISZ;
    }

    PropertySize = PropertyBufferSize;
    cr = CM_Get_DevInst_Custom_Property_ExA(
        (*deviceInfo).dnDevInst,
        CustomPropertyName,
        PropertyRegDataType,
        (PropertyBuffer).cast(),
        core::ptr::addr_of_mut!(PropertySize),
        ConfigFlags,
        (*set).hMachine,
    );
    if (cr == CR_SUCCESS) || (cr == CR_BUFFER_SMALL) {
        if !RequiredSize.is_null() {
            *RequiredSize = PropertySize;
        }
    }

    if cr != CR_SUCCESS {
        SetLastError(GetErrorCodeFromCrCode(cr));
        return (FALSE) as BOOL;
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetCustomDevicePropertyW(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
    mut CustomPropertyName: PCWSTR,
    mut Flags: DWORD,
    mut PropertyRegDataType: PDWORD,
    mut PropertyBuffer: PBYTE,
    mut PropertyBufferSize: DWORD,
    mut RequiredSize: PDWORD,
) -> BOOL {
    let mut set: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
    let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut::<DeviceInfo>();
    let mut ConfigFlags: DWORD = 0;
    let mut PropertySize: DWORD;
    let mut cr: CONFIGRET;

    if DeviceInfoSet.is_null() || DeviceInfoSet == INVALID_HANDLE_VALUE
    {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }
    if (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC
    {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }
    if DeviceInfoData.is_null()
        || (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
        || (*DeviceInfoData).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    if (Flags & !DICUSTOMDEVPROP_MERGE_MULTISZ) != 0
    {
        SetLastError(ERROR_INVALID_FLAGS);
        return (FALSE) as BOOL;
    }

    deviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
    if (*deviceInfo).set != set
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    if (Flags & DICUSTOMDEVPROP_MERGE_MULTISZ) != 0
    {
        ConfigFlags |= CM_CUSTOMDEVPROP_MERGE_MULTISZ;
    }

    PropertySize = PropertyBufferSize;
    cr = CM_Get_DevInst_Custom_Property_ExW(
        (*deviceInfo).dnDevInst,
        CustomPropertyName,
        PropertyRegDataType,
        (PropertyBuffer).cast(),
        core::ptr::addr_of_mut!(PropertySize),
        ConfigFlags,
        (*set).hMachine,
    );
    if (cr == CR_SUCCESS) || (cr == CR_BUFFER_SMALL)
    {
        if !RequiredSize.is_null()
        {
            *RequiredSize = PropertySize;
        }
    }

    if cr != CR_SUCCESS
    {
        SetLastError(GetErrorCodeFromCrCode(cr));
        return (FALSE) as BOOL;
    }

    return TRUE as BOOL;
}
