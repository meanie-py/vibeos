use crate::setuplib::filesup::CombinePaths;
use crate::setuplib::regutil::ConnectRegistry;
use crate::setuplib::regutil::CreateNestedKey;
use crate::setuplib::regutil::CreateRegistryFile;
use crate::setuplib::regutil::CreateSymLinkKey;
use crate::setuplib::regutil::DeleteSymLinkKey;
use crate::setuplib::regutil::DisconnectRegistry;
use crate::setuplib::filesup::SetupCopyFile;
use crate::setuplib::regutil::VerifyRegistryHive;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0C8C471E3DBF: [u16; 16] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C, 0x0063,
    0x006F, 0x006E, 0x0066, 0x0069, 0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_1820A50AEA7D: [u16; 38] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0059,
    0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0024, 0x0024, 0x0024, 0x0050, 0x0052,
    0x004F, 0x0054, 0x004F, 0x002E, 0x0048, 0x0049, 0x0056, 0x0000,
];
pub static RUSTCONV_WSTR_1CA9A6B1F115: [u16; 7] = [0x0041u16, 0x0064, 0x0064, 0x0052, 0x0065, 0x0067, 0x0000];
#[cfg(target_arch = "x86_64")]
pub static RUSTCONV_WSTR_85E7682516F1: [u16; 15] = [
    0x0041u16, 0x0064, 0x0064, 0x0052, 0x0065, 0x0067, 0x002E, 0x004E, 0x0054, 0x0061,
    0x006D, 0x0064, 0x0036, 0x0034, 0x0000,
];
#[cfg(target_arch = "aarch64")]
pub static RUSTCONV_WSTR_85E7682516F1: [u16; 15] = [
    0x0041u16, 0x0064, 0x0064, 0x0052, 0x0065, 0x0067, 0x002E, 0x004E, 0x0054, 0x0061,
    0x0072, 0x006D, 0x0036, 0x0034, 0x0000,
];
#[cfg(target_arch = "x86")]
pub static RUSTCONV_WSTR_85E7682516F1: [u16; 13] = [
    0x0041u16, 0x0064, 0x0064, 0x0052, 0x0065, 0x0067, 0x002E, 0x004E, 0x0054, 0x0078,
    0x0038, 0x0036, 0x0000,
];
pub static RUSTCONV_WSTR_8D22E72486A8: [u16; 21] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x006F, 0x006E,
    0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065, 0x0074, 0x0030, 0x0030, 0x0031,
    0x0000,
];
pub static RUSTCONV_WSTR_8DA4DAC13B96: [u16; 25] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_A34E7EC9273F: [u16; 61] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0059,
    0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0055, 0x0053, 0x0065, 0x0074, 0x0075,
    0x0070, 0x005F, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C,
    0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x006F, 0x006E,
    0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065, 0x0074, 0x0030, 0x0030, 0x0031,
    0x0000,
];
pub static RUSTCONV_WSTR_D75304D3B217: [u16; 5] = [0x002Eu16, 0x0073, 0x0061, 0x0076, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_091071C6A850: [u16; 32] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0055, 0x0053,
    0x0065, 0x0074, 0x0075, 0x0070, 0x005F, 0x0053, 0x0059, 0x0053, 0x0054, 0x0045,
    0x004D, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_209E1F1B4C03: [u16; 58] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0059,
    0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0055, 0x0053, 0x0065, 0x0074, 0x0075,
    0x0070, 0x005F, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C,
    0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x0043,
    0x006C, 0x0061, 0x0073, 0x0073, 0x0065, 0x0073, 0x005C, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_22FF61FCDE09: [u16; 4] = [0x0048u16, 0x004B, 0x0055, 0x0000];
pub static RUSTCONV_C2R_WSTR_274DBC237377: [u16; 30] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x0055, 0x0073, 0x0065, 0x0072, 0x005C, 0x0055, 0x0053, 0x0065, 0x0074, 0x0075,
    0x0070, 0x005F, 0x0044, 0x0045, 0x0046, 0x0041, 0x0055, 0x004C, 0x0054, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_3D6AA8E05FAE: [u16; 29] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0055, 0x0053,
    0x0065, 0x0074, 0x0075, 0x0070, 0x005F, 0x0053, 0x0041, 0x004D, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_4D8857DD945B: [u16; 8] = [0x0044u16, 0x0045, 0x0046, 0x0041, 0x0055, 0x004C, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_68FC24CACD59: [u16; 9] = [0x002Eu16, 0x0044, 0x0045, 0x0046, 0x0041, 0x0055, 0x004C, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_786BB63C6F29: [u16; 9] = [0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x0000];
pub static RUSTCONV_C2R_WSTR_82C26D835FE5: [u16; 34] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0055, 0x0053,
    0x0065, 0x0074, 0x0075, 0x0070, 0x005F, 0x0053, 0x004F, 0x0046, 0x0054, 0x0057,
    0x0041, 0x0052, 0x0045, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_89F38F0B7FC9: [u16; 5] = [0x0048u16, 0x004B, 0x004C, 0x004D, 0x0000];
pub static RUSTCONV_C2R_WSTR_8DEA0334349B: [u16; 47] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0059,
    0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0055, 0x0053, 0x0065, 0x0074, 0x0075,
    0x0070, 0x005F, 0x0055, 0x0073, 0x0065, 0x0072, 0x005C, 0x002E, 0x0044, 0x0045,
    0x0046, 0x0041, 0x0055, 0x004C, 0x0054, 0x005C, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_8E490F9323D5: [u16; 5] = [0x0048u16, 0x004B, 0x0043, 0x0052, 0x0000];
pub static RUSTCONV_C2R_WSTR_BBB9AF422CDB: [u16; 9] = [0x0053u16, 0x0045, 0x0043, 0x0055, 0x0052, 0x0049, 0x0054, 0x0059, 0x0000];
pub static RUSTCONV_C2R_WSTR_D02D0BCFE5D4: [u16; 5] = [0x0048u16, 0x004B, 0x0043, 0x0055, 0x0000];
pub static RUSTCONV_C2R_WSTR_E1F8BE119079: [u16; 34] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0055, 0x0053,
    0x0065, 0x0074, 0x0075, 0x0070, 0x005F, 0x0053, 0x0045, 0x0043, 0x0055, 0x0052,
    0x0049, 0x0054, 0x0059, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_F404B1D03968: [u16; 4] = [0x0053u16, 0x0041, 0x004D, 0x0000];
pub static RUSTCONV_C2R_WSTR_F9F36001AF87: [u16; 7] = [0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x0000];

pub const REGISTRY_SETUP_MACHINE: [::core::ffi::c_ushort; 41] = unsafe {
    ::core::mem::transmute::<
        [u8; 82],
        [::core::ffi::c_ushort; 41],
    >(
        *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0Y\0S\0T\0E\0M\0\\\0U\0S\0e\0t\0u\0p\0_\0M\0a\0c\0h\0i\0n\0e\0\\\0\0\0",
    )
};

pub const REGISTRY_SETUP_USER: [::core::ffi::c_ushort; 38] = unsafe {
    ::core::mem::transmute::<
        [u8; 76],
        [::core::ffi::c_ushort; 38],
    >(
        *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0Y\0S\0T\0E\0M\0\\\0U\0S\0e\0t\0u\0p\0_\0U\0s\0e\0r\0\\\0\0\0",
    )
};

pub static mut RootKeys: [ROOT_KEY; 4] = unsafe {
    [
        _ROOT_KEY {
            Name: RUSTCONV_C2R_WSTR_8E490F9323D5.as_ptr() as PCWSTR,
            MountPoint: RUSTCONV_C2R_WSTR_209E1F1B4C03.as_ptr() as PCWSTR,
            Handle: NULL,
        },
        _ROOT_KEY {
            Name: RUSTCONV_C2R_WSTR_D02D0BCFE5D4.as_ptr() as PCWSTR,
            MountPoint: RUSTCONV_C2R_WSTR_8DEA0334349B.as_ptr() as PCWSTR,
            Handle: NULL,
        },
        _ROOT_KEY {
            Name: RUSTCONV_C2R_WSTR_89F38F0B7FC9.as_ptr() as PCWSTR,
            MountPoint: REGISTRY_SETUP_MACHINE.as_ptr() as PCWSTR,
            Handle: NULL,
        },
        _ROOT_KEY {
            Name: RUSTCONV_C2R_WSTR_22FF61FCDE09.as_ptr() as PCWSTR,
            MountPoint: REGISTRY_SETUP_USER.as_ptr() as PCWSTR,
            Handle: NULL,
        },
    ]
};

pub static mut RegistryHives: [HIVE_LIST_ENTRY; 3] = unsafe {
    [
        _HIVE_LIST_ENTRY {
            HiveName: RUSTCONV_C2R_WSTR_F9F36001AF87.as_ptr() as PCWSTR,
            HiveRegistryPath: RUSTCONV_C2R_WSTR_091071C6A850.as_ptr() as PCWSTR,
            PredefKeyHandle: HKEY_LOCAL_MACHINE as HANDLE,
            RegSymLink: RUSTCONV_C2R_WSTR_F9F36001AF87.as_ptr() as PCWSTR,
            State: Create,
        },
        _HIVE_LIST_ENTRY {
            HiveName: RUSTCONV_C2R_WSTR_786BB63C6F29.as_ptr() as PCWSTR,
            HiveRegistryPath: RUSTCONV_C2R_WSTR_82C26D835FE5.as_ptr() as PCWSTR,
            PredefKeyHandle: HKEY_LOCAL_MACHINE as HANDLE,
            RegSymLink: RUSTCONV_C2R_WSTR_786BB63C6F29.as_ptr() as PCWSTR,
            State: Create,
        },
        _HIVE_LIST_ENTRY {
            HiveName: RUSTCONV_C2R_WSTR_4D8857DD945B.as_ptr() as PCWSTR,
            HiveRegistryPath: RUSTCONV_C2R_WSTR_274DBC237377.as_ptr() as PCWSTR,
            PredefKeyHandle: HKEY_USERS as HANDLE,
            RegSymLink: RUSTCONV_C2R_WSTR_68FC24CACD59.as_ptr() as PCWSTR,
            State: Create,
        },
    ]
};

pub static mut SecurityRegistryHives: [HIVE_LIST_ENTRY; 2] = unsafe {
    [
        _HIVE_LIST_ENTRY {
            HiveName: RUSTCONV_C2R_WSTR_F404B1D03968.as_ptr() as PCWSTR,
            HiveRegistryPath: RUSTCONV_C2R_WSTR_3D6AA8E05FAE.as_ptr() as PCWSTR,
            PredefKeyHandle: HKEY_LOCAL_MACHINE as HANDLE,
            RegSymLink: RUSTCONV_C2R_WSTR_F404B1D03968.as_ptr() as PCWSTR,
            State: Create,
        },
        _HIVE_LIST_ENTRY {
            HiveName: RUSTCONV_C2R_WSTR_BBB9AF422CDB.as_ptr() as PCWSTR,
            HiveRegistryPath: RUSTCONV_C2R_WSTR_E1F8BE119079.as_ptr() as PCWSTR,
            PredefKeyHandle: HKEY_LOCAL_MACHINE as HANDLE,
            RegSymLink: RUSTCONV_C2R_WSTR_BBB9AF422CDB.as_ptr() as PCWSTR,
            State: Create,
        },
    ]
};

pub const FLG_ADDREG_BINVALUETYPE: u32 = 0x00000001;
pub const FLG_ADDREG_NOCLOBBER: u32 = 0x00000002;
pub const FLG_ADDREG_DELVAL: u32 = 0x00000004;
pub const FLG_ADDREG_APPEND: u32 = 0x00000008;
pub const FLG_ADDREG_KEYONLY: u32 = 0x00000010;
pub const FLG_ADDREG_OVERWRITEONLY: u32 = 0x00000020;
pub const FLG_ADDREG_TYPE_SZ: u32 = 0x00000000;
pub const FLG_ADDREG_TYPE_MULTI_SZ: u32 = 0x00010000;
pub const FLG_ADDREG_TYPE_EXPAND_SZ: u32 = 0x00020000;
pub const FLG_ADDREG_TYPE_BINARY: u32 = (0x00000000 | FLG_ADDREG_BINVALUETYPE);
pub const FLG_ADDREG_TYPE_DWORD: u32 = (0x00010000 | FLG_ADDREG_BINVALUETYPE);
pub const FLG_ADDREG_TYPE_NONE: u32 = (0x00020000 | FLG_ADDREG_BINVALUETYPE);
pub const FLG_ADDREG_TYPE_MASK: u32 = (0xFFFF0000 | FLG_ADDREG_BINVALUETYPE);

pub type PROOT_KEY = *mut ROOT_KEY;

pub const Create: HIVE_UPDATE_STATE = 0;
pub const Repair: HIVE_UPDATE_STATE = 1;
pub const Update: HIVE_UPDATE_STATE = 2;

pub type PHIVE_LIST_ENTRY = *mut HIVE_LIST_ENTRY;

pub const NUMBER_OF_STANDARD_REGISTRY_HIVES: usize = 3;

pub const NUMBER_OF_SECURITY_REGISTRY_HIVES: usize = 2;
#[no_mangle]
pub unsafe extern "C" fn GetRootKeyByPredefKey(
    KeyHandle: HANDLE,
    RootKeyMountPoint: *mut PCWSTR,
) -> HANDLE {
    let Index: ULONG_PTR = GetPredefKeyIndex!(KeyHandle);

    if !IsPredefKey!(KeyHandle) {
        return core::ptr::null_mut();
    }
    if Index >= ARRAYSIZE!(RootKeys) as ULONG_PTR {
        return core::ptr::null_mut();
    }

    if !RootKeyMountPoint.is_null() {
        *RootKeyMountPoint = RootKeys[Index as usize].MountPoint;
    }
    return RootKeys[Index as usize].Handle;
}
#[no_mangle]
pub unsafe extern "C" fn GetRootKeyByName(
    RootKeyName: PCWSTR,
    RootKeyMountPoint: *mut PCWSTR,
) -> HANDLE {
    let mut i: u8;

    i = 0;
    while (i as u32) < ARRAYSIZE!(RootKeys) {
        if ((_wcsicmp(RootKeyName, RootKeys[i as usize].Name)) == 0) {
            if !RootKeyMountPoint.is_null() {
                *RootKeyMountPoint = RootKeys[i as usize].MountPoint;
            }
            return RootKeys[i as usize].Handle;
        }
        i = i.wrapping_add(1);
    }

    return core::ptr::null_mut();
}

unsafe extern "C" fn do_reg_operation(
    mut KeyHandle: HANDLE,
    mut ValueName: PUNICODE_STRING,
    mut Context: PINFCONTEXT,
    mut Flags: ULONG,
) -> BOOLEAN {
    let mut EmptyStr: WCHAR = 0;
    let mut Type: ULONG = 0;
    let mut Size: ULONG = 0;
    if Flags & FLG_ADDREG_DELVAL != 0 {
        return (TRUE) as u8;
    }
    if Flags & FLG_ADDREG_KEYONLY != 0 {
        return (TRUE) as u8;
    }
    match Flags & FLG_ADDREG_TYPE_MASK {
        FLG_ADDREG_TYPE_SZ => {
            Type = REG_SZ;
        }
        FLG_ADDREG_TYPE_MULTI_SZ => {
            Type = REG_MULTI_SZ;
        }
        FLG_ADDREG_TYPE_EXPAND_SZ => {
            Type = REG_EXPAND_SZ;
        }
        FLG_ADDREG_TYPE_BINARY => {
            Type = REG_BINARY;
        }
        FLG_ADDREG_TYPE_DWORD => {
            Type = REG_DWORD;
        }
        FLG_ADDREG_TYPE_NONE => {
            Type = REG_NONE;
        }
        _ => {
            Type = Flags >> 16;
        }
    }
    if Flags & FLG_ADDREG_BINVALUETYPE == 0
        || Type == REG_DWORD
            && SpInfGetFieldCount!(Context) == 5
    {
        let mut Str: PWCHAR = core::ptr::null_mut::<WCHAR>();
        if Type == REG_MULTI_SZ {
            if SpInfGetMultiSzField!(
                Context,
                5,
                core::ptr::null_mut::<WCHAR>(),
                0,
                &raw mut Size,
            ) == 0
            {
                Size = 0;
            }
            if Size != 0 {
                Str = RtlAllocateHeap(
                    ProcessHeap as PVOID,
                    0,
                    (Size as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as *mut WCHAR as PWCHAR;
                if Str.is_null() {
                    return (FALSE) as u8;
                }
                SpInfGetMultiSzField!(
                    Context,
                    5,
                    Str as PWSTR,
                    Size,
                    core::ptr::null_mut::<ULONG>(),
                );
            }
            if Flags & FLG_ADDREG_APPEND != 0 {
                if Str.is_null() {
                    return (TRUE) as u8;
                }
                DPRINT1!("append_multi_sz_value '%S' commented out, WHY??\n", ValueName);
                RtlFreeHeap(ProcessHeap, 0, Str as PVOID);
                return (TRUE) as u8;
            }
        } else {
            if SpInfGetStringField!(
                Context,
                5,
                core::ptr::null_mut::<WCHAR>(),
                0,
                &raw mut Size,
            ) == 0
            {
                Size = 0;
            }
            if Size != 0 {
                Str = RtlAllocateHeap(
                    ProcessHeap as PVOID,
                    0,
                    (Size as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as *mut WCHAR as PWCHAR;
                if Str.is_null() {
                    return (FALSE) as u8;
                }
                SpInfGetStringField!(
                    Context,
                    5,
                    Str as PWSTR,
                    Size,
                    core::ptr::null_mut::<ULONG>(),
                );
            }
        }
        if Type == REG_DWORD {
            let mut dw: ULONG = if !Str.is_null() {
                wcstoul(
                    Str as *const wchar_t,
                    core::ptr::null_mut::<*mut wchar_t>(),
                    0,
                ) as ULONG
            } else {
                0
            };
            DPRINT!("setting dword %wZ to %lx\n", ValueName, dw);
            NtSetValueKey(
                KeyHandle,
                ValueName,
                0,
                Type,
                &raw mut dw as PVOID,
                core::mem::size_of::<ULONG>() as ULONG,
            );
        } else {
            DPRINT!("setting value %wZ to %S\n", ValueName, Str);
            if !Str.is_null() {
                NtSetValueKey(
                    KeyHandle,
                    ValueName,
                    0,
                    Type,
                    Str as PVOID,
                    (Size as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as ULONG,
                );
            } else {
                NtSetValueKey(
                    KeyHandle,
                    ValueName,
                    0,
                    Type,
                    &raw mut EmptyStr as PVOID,
                    core::mem::size_of::<WCHAR>() as ULONG,
                );
            }
        }
        RtlFreeHeap(ProcessHeap, 0, Str as PVOID);
    } else {
        let mut Data: PUCHAR = core::ptr::null_mut::<UCHAR>();
        if SpInfGetBinaryField!(
            Context,
            5,
            core::ptr::null_mut::<UCHAR>(),
            0,
            &raw mut Size,
        ) == 0
        {
            Size = 0;
        }
        if Size != 0 {
            Data = RtlAllocateHeap(
                ProcessHeap as PVOID,
                0,
                Size as SIZE_T,
            ) as *mut core::ffi::c_uchar as PUCHAR;
            if Data.is_null() {
                return (FALSE) as u8;
            }
            DPRINT!("setting binary data %wZ len %lu\n", ValueName, Size);
            SpInfGetBinaryField!(
                Context,
                5,
                Data,
                Size,
                core::ptr::null_mut::<ULONG>(),
            );
        }
        NtSetValueKey(
            KeyHandle,
            ValueName,
            0,
            Type,
            Data as PVOID,
            Size,
        );
        RtlFreeHeap(ProcessHeap, 0, Data as PVOID);
    }
    return (TRUE) as u8;
}

unsafe extern "C" fn registry_callback(
    mut hInf: HINF,
    mut Section: PCWSTR,
    mut Delete: BOOLEAN,
) -> BOOLEAN {
    unsafe {
        let mut Status: NTSTATUS = 0;
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        let mut Name: _UNICODE_STRING = core::mem::zeroed();
        let mut Value: _UNICODE_STRING = core::mem::zeroed();
        let mut ValuePtr: PUNICODE_STRING = core::ptr::null_mut();
        let mut Flags: UINT = 0;
        let mut Buffer = [0u16; (MAX_INF_STRING_LENGTH) as _];
        let mut Context: INFCONTEXT = core::mem::zeroed();
        let mut RootKeyName: PCWSTR = core::ptr::null();
        let mut RootKeyHandle: HANDLE = core::ptr::null_mut();
        let mut KeyHandle: HANDLE = core::ptr::null_mut();
        let mut Ok_: BOOLEAN = 0;

        Ok_ = SpInfFindFirstLine!(
            hInf,
            Section,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Context),
        ) as BOOLEAN;
        if Ok_ == 0 {
            return (TRUE) as u8;
        }

        while Ok_ != 0 {
            if SpInfGetStringField!(
                core::ptr::addr_of_mut!(Context),
                1,
                Buffer.as_mut_ptr(),
                Buffer.len() as u32,
                core::ptr::null_mut(),
            ) != 0
            {
                RootKeyHandle = GetRootKeyByName(
                    Buffer.as_mut_ptr() as PCWSTR,
                    core::ptr::addr_of_mut!(RootKeyName),
                );

                if !RootKeyHandle.is_null() {
                    if SpInfGetStringField!(
                        core::ptr::addr_of_mut!(Context),
                        2,
                        Buffer.as_mut_ptr(),
                        Buffer.len() as u32,
                        core::ptr::null_mut(),
                    ) == 0 {
                        *Buffer.as_mut_ptr() = 0;
                    }

                    if SpInfGetIntField!(
                        core::ptr::addr_of_mut!(Context),
                        4,
                        core::ptr::addr_of_mut!(Flags) as PINT,
                    ) == 0 {
                        Flags = 0;
                    }

                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!(Name),
                        Buffer.as_mut_ptr() as PCWSTR,
                    );

                    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                    ObjectAttributes.RootDirectory = RootKeyHandle;
                    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
                    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(Name);
                    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                    let mut skip_rest = false;

                    if Delete != 0 || (Flags & FLG_ADDREG_OVERWRITEONLY) != 0 {
                        Status = NtOpenKey(
                            core::ptr::addr_of_mut!(KeyHandle),
                            KEY_ALL_ACCESS,
                            core::ptr::addr_of_mut!(ObjectAttributes),
                        );
                        if NT_SUCCESS!(Status) == 0 {
                            if DbgPrint(
                                b"(%s:%d) NtOpenKey(%wZ) failed (Status %lx)\n\0".as_ptr() as *const i8,
                                b"base/setup/lib/registry.c\0".as_ptr() as *const i8,
                                465 as i32,
                                core::ptr::addr_of_mut!(Name),
                                Status,
                            ) != 0 {
                                DbgPrint(
                                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                                    b"base/setup/lib/registry.c\0".as_ptr() as *const i8,
                                    465 as i32,
                                );
                            }
                            skip_rest = true;
                        }
                    } else {
                        Status = CreateNestedKey(
                            core::ptr::addr_of_mut!(KeyHandle),
                            KEY_ALL_ACCESS,
                            core::ptr::addr_of_mut!(ObjectAttributes),
                            REG_OPTION_NON_VOLATILE,
                        );
                        if NT_SUCCESS!(Status) == 0 {
                            if DbgPrint(
                                b"(%s:%d) CreateNestedKey(%wZ) failed (Status %lx)\n\0".as_ptr() as *const i8,
                                b"base/setup/lib/registry.c\0".as_ptr() as *const i8,
                                477 as i32,
                                core::ptr::addr_of_mut!(Name),
                                Status,
                            ) != 0 {
                                DbgPrint(
                                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                                    b"base/setup/lib/registry.c\0".as_ptr() as *const i8,
                                    477 as i32,
                                );
                            }
                            skip_rest = true;
                        }
                    }

                    if !skip_rest {
                        if SpInfGetStringField!(
                            core::ptr::addr_of_mut!(Context),
                            3,
                            Buffer.as_mut_ptr(),
                            Buffer.len() as u32,
                            core::ptr::null_mut(),
                        ) != 0 {
                            RtlInitUnicodeString(
                                core::ptr::addr_of_mut!(Value),
                                Buffer.as_mut_ptr() as PCWSTR,
                            );
                            ValuePtr = core::ptr::addr_of_mut!(Value);
                        } else {
                            ValuePtr = core::ptr::null_mut();
                        }

                        if do_reg_operation(
                            KeyHandle,
                            ValuePtr,
                            core::ptr::addr_of_mut!(Context),
                            Flags,
                        ) == 0 {
                            NtClose(KeyHandle);
                            return (FALSE) as u8;
                        }

                        NtClose(KeyHandle);
                    }
                }
            }

            Ok_ = SpInfFindNextLine!(
                core::ptr::addr_of_mut!(Context),
                core::ptr::addr_of_mut!(Context),
            ) as BOOLEAN;
        }

        return (TRUE) as u8;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ImportRegistryFile(
    mut SourcePath: PCWSTR,
    mut FileName: PCWSTR,
    mut Section: PCWSTR,
    mut LocaleId: LCID,
    mut Delete: BOOLEAN,
) -> BOOLEAN {
    let mut hInf: HINF = core::ptr::null_mut();
    let mut ErrorLine: u32 = 0;
    let mut FileNameBuffer: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    CombinePaths(
        FileNameBuffer.as_mut_ptr(),
        (ARRAYSIZE!(FileNameBuffer)) as _,
        2,
        SourcePath,
        FileName,
    );

    hInf = SpInfOpenInfFile!(
        FileNameBuffer.as_ptr(),
        core::ptr::null(),
        INF_STYLE_WIN4,
        LocaleId,
        &mut ErrorLine,
    );

    if hInf == INVALID_HANDLE_VALUE {
        DPRINT1!("SpInfOpenInfFile() failed\n");
        return (FALSE) as u8;
    }

    if ((registry_callback(hInf, RUSTCONV_WSTR_1CA9A6B1F115.as_ptr(), (FALSE) as BOOLEAN)) == 0) {
        DPRINT1!("registry_callback() failed\n");
        SpInfCloseInfFile!(hInf);
        return (FALSE) as u8;
    }

    if ((registry_callback(hInf, RUSTCONV_WSTR_85E7682516F1.as_ptr(), (FALSE) as BOOLEAN)) == 0) {
        DPRINT1!("registry_callback() failed\n");
        SpInfCloseInfFile!(hInf);
        return (FALSE) as u8;
    }

    SpInfCloseInfFile!(hInf);
    return (TRUE) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn VerifyRegistryHives(
    NtSystemRoot: PUNICODE_STRING,
    ShouldRepairRegistry: PBOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut PrivilegeSet: [BOOLEAN; 2] = [(FALSE) as u8, (FALSE) as u8];
    let mut i: UINT;

    *ShouldRepairRegistry = (FALSE) as u8;

    Status = RtlAdjustPrivilege(
        SE_RESTORE_PRIVILEGE,
        (TRUE) as BOOLEAN,
        (FALSE) as BOOLEAN,
        &mut PrivilegeSet[0],
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"RtlAdjustPrivilege(SE_RESTORE_PRIVILEGE) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        return Status;
    }

    Status = RtlAdjustPrivilege(
        SE_BACKUP_PRIVILEGE,
        (TRUE) as BOOLEAN,
        (FALSE) as BOOLEAN,
        &mut PrivilegeSet[1],
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"RtlAdjustPrivilege(SE_BACKUP_PRIVILEGE) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        RtlAdjustPrivilege(
            SE_RESTORE_PRIVILEGE,
            PrivilegeSet[0],
            (FALSE) as BOOLEAN,
            &mut PrivilegeSet[0],
        );
        return Status;
    }

    i = 0;
    while (i as usize) < (ARRAYSIZE!(RegistryHives) as usize) {
        Status = VerifyRegistryHive(NtSystemRoot, RegistryHives[i as usize].HiveName);
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(
                b"Registry hive '%S' needs repair!\n\0".as_ptr() as *const i8,
                RegistryHives[i as usize].HiveName,
            );
            RegistryHives[i as usize].State = Repair;
            *ShouldRepairRegistry = (TRUE) as u8;
        } else {
            RegistryHives[i as usize].State = Update;
        }
        i = i.wrapping_add(1);
    }

    i = 0;
    while (i as usize) < (ARRAYSIZE!(SecurityRegistryHives) as usize) {
        Status = VerifyRegistryHive(NtSystemRoot, SecurityRegistryHives[i as usize].HiveName);
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(
                b"Registry hive '%S' needs repair!\n\0".as_ptr() as *const i8,
                SecurityRegistryHives[i as usize].HiveName,
            );
            SecurityRegistryHives[i as usize].State = Repair;
        } else {
            SecurityRegistryHives[i as usize].State = Update;
        }
        i = i.wrapping_add(1);
    }

    Status = STATUS_SUCCESS;

    RtlAdjustPrivilege(
        SE_BACKUP_PRIVILEGE,
        PrivilegeSet[1],
        (FALSE) as BOOLEAN,
        &mut PrivilegeSet[1],
    );
    RtlAdjustPrivilege(
        SE_RESTORE_PRIVILEGE,
        PrivilegeSet[0],
        (FALSE) as BOOLEAN,
        &mut PrivilegeSet[0],
    );

    Status
}
#[no_mangle]
pub unsafe extern "C" fn RegInitializeRegistry(NtSystemRoot: PUNICODE_STRING) -> NTSTATUS {
    unsafe {
        let mut Status: NTSTATUS = 0;
        let mut KeyHandle: HANDLE = core::ptr::null_mut();
        let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        let mut PrivilegeSet: [BOOLEAN; 2] = [(FALSE) as u8, (FALSE) as u8];
        let mut Disposition: ULONG = 0;
        let mut i: UINT = 0;

        'quit: {
            Status = RtlAdjustPrivilege(
                SE_RESTORE_PRIVILEGE,
                (TRUE) as BOOLEAN,
                (FALSE) as BOOLEAN,
                &raw mut PrivilegeSet[0],
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(b"RtlAdjustPrivilege(SE_RESTORE_PRIVILEGE) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
                return Status;
            }

            Status = RtlAdjustPrivilege(
                SE_BACKUP_PRIVILEGE,
                (TRUE) as BOOLEAN,
                (FALSE) as BOOLEAN,
                &raw mut PrivilegeSet[1],
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(b"RtlAdjustPrivilege(SE_BACKUP_PRIVILEGE) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
                RtlAdjustPrivilege(
                    SE_RESTORE_PRIVILEGE,
                    PrivilegeSet[0],
                    (FALSE) as BOOLEAN,
                    &raw mut PrivilegeSet[0],
                );
                return Status;
            }

            RtlInitUnicodeString(&raw mut KeyName, RUSTCONV_WSTR_1820A50AEA7D.as_ptr());
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.ObjectName = &raw mut KeyName;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = NtCreateKey(
                &raw mut KeyHandle,
                KEY_ALL_ACCESS,
                &raw mut ObjectAttributes,
                0,
                core::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                core::ptr::null_mut(),
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(b"NtCreateKey() failed to create the proto-hive (Status %lx)\n\0".as_ptr() as *const i8, Status);
                break 'quit;
            }
            NtFlushKey(KeyHandle);

            i = 0;
            while (i as usize) < ARRAYSIZE!(RegistryHives) as usize {
                if RegistryHives[i as usize].State != Create && RegistryHives[i as usize].State != Repair {
                    i = i.wrapping_add(1);
                    continue;
                }

                Status = CreateRegistryFile(
                    NtSystemRoot,
                    RegistryHives[i as usize].HiveName,
                    (RegistryHives[i as usize].State != Repair) as BOOLEAN,
                    KeyHandle,
                );
                if ((NT_SUCCESS!(Status)) == 0) {
                    DPRINT1!(b"CreateRegistryFile(%S) failed, Status 0x%08lx\n\0".as_ptr() as *const i8, RegistryHives[i as usize].HiveName, Status);
                    NtDeleteKey(KeyHandle);
                    NtClose(KeyHandle);
                    break 'quit;
                }

                i = i.wrapping_add(1);
            }

            NtDeleteKey(KeyHandle);
            NtClose(KeyHandle);

            RtlInitUnicodeString(
                &raw mut KeyName,
                RootKeys[GetPredefKeyIndex!(HKEY_LOCAL_MACHINE) as usize].MountPoint,
            );
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.ObjectName = &raw mut KeyName;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            KeyHandle = core::ptr::null_mut();
            Status = NtCreateKey(
                &raw mut KeyHandle,
                KEY_ALL_ACCESS,
                &raw mut ObjectAttributes,
                0,
                core::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                &raw mut Disposition,
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(b"NtCreateKey(%wZ) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, &raw mut KeyName, Status);
            }
            RootKeys[GetPredefKeyIndex!(HKEY_LOCAL_MACHINE) as usize].Handle = KeyHandle;

            RtlInitUnicodeString(
                &raw mut KeyName,
                RootKeys[GetPredefKeyIndex!(HKEY_USERS) as usize].MountPoint,
            );
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.ObjectName = &raw mut KeyName;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            KeyHandle = core::ptr::null_mut();
            Status = NtCreateKey(
                &raw mut KeyHandle,
                KEY_ALL_ACCESS,
                &raw mut ObjectAttributes,
                0,
                core::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                &raw mut Disposition,
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(b"NtCreateKey(%wZ) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, &raw mut KeyName, Status);
            }
            RootKeys[GetPredefKeyIndex!(HKEY_USERS) as usize].Handle = KeyHandle;

            i = 0;
            while (i as usize) < ARRAYSIZE!(RegistryHives) as usize {
                if RegistryHives[i as usize].State == Create || RegistryHives[i as usize].State == Repair {
                    Status = ConnectRegistry(
                        core::ptr::null_mut(),
                        RegistryHives[i as usize].HiveRegistryPath,
                        NtSystemRoot,
                        RegistryHives[i as usize].HiveName,
                    );
                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT1!(b"ConnectRegistry(%S) failed, Status 0x%08lx\n\0".as_ptr() as *const i8, RegistryHives[i as usize].HiveName, Status);
                    }

                    Status = CreateSymLinkKey(
                        RootKeys[GetPredefKeyIndex!(RegistryHives[i as usize].PredefKeyHandle) as usize].Handle,
                        RegistryHives[i as usize].RegSymLink,
                        RegistryHives[i as usize].HiveRegistryPath,
                    );
                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT1!(b"CreateSymLinkKey(%S) failed, Status 0x%08lx\n\0".as_ptr() as *const i8, RegistryHives[i as usize].RegSymLink, Status);
                    }
                } else {
                    RtlInitUnicodeString(
                        &raw mut KeyName,
                        RegistryHives[i as usize].RegSymLink,
                    );
                    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
                    ObjectAttributes.RootDirectory = RootKeys[GetPredefKeyIndex!(RegistryHives[i as usize].PredefKeyHandle) as usize].Handle;
                    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
                    ObjectAttributes.ObjectName = &raw mut KeyName;
                    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                    KeyHandle = core::ptr::null_mut();
                    Status = NtCreateKey(
                        &raw mut KeyHandle,
                        KEY_ALL_ACCESS,
                        &raw mut ObjectAttributes,
                        0,
                        core::ptr::null_mut(),
                        REG_OPTION_NON_VOLATILE,
                        &raw mut Disposition,
                    );
                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT1!(b"NtCreateKey(%wZ) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, &raw mut KeyName, Status);
                    }
                    NtClose(KeyHandle);
                }

                i = i.wrapping_add(1);
            }

            RtlInitUnicodeString(
                &raw mut KeyName,
                RootKeys[GetPredefKeyIndex!(HKEY_CURRENT_USER) as usize].MountPoint,
            );
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.ObjectName = &raw mut KeyName;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            KeyHandle = core::ptr::null_mut();
            Status = NtOpenKey(
                &raw mut KeyHandle,
                KEY_ALL_ACCESS,
                &raw mut ObjectAttributes,
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(b"NtOpenKey(%wZ) failed (Status %lx)\n\0".as_ptr() as *const i8, &raw mut KeyName, Status);
            }
            RootKeys[GetPredefKeyIndex!(HKEY_CURRENT_USER) as usize].Handle = KeyHandle;

            RtlInitUnicodeString(
                &raw mut KeyName,
                RootKeys[GetPredefKeyIndex!(HKEY_CLASSES_ROOT) as usize].MountPoint,
            );
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.ObjectName = &raw mut KeyName;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            KeyHandle = core::ptr::null_mut();
            Status = NtCreateKey(
                &raw mut KeyHandle,
                KEY_ALL_ACCESS,
                &raw mut ObjectAttributes,
                0,
                core::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                &raw mut Disposition,
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(b"NtCreateKey(%wZ) failed (Status %lx)\n\0".as_ptr() as *const i8, &raw mut KeyName, Status);
            } else {
                DPRINT!(b"NtCreateKey() succeeded to %s the %wZ key (Status %lx)\n\0".as_ptr() as *const i8,
                       if Disposition == REG_CREATED_NEW_KEY { b"create\0".as_ptr() as *const i8 } else { b"open\0".as_ptr() as *const i8 },
                       &raw mut KeyName,
                       Status);
            }
            RootKeys[GetPredefKeyIndex!(HKEY_CLASSES_ROOT) as usize].Handle = KeyHandle;

            Status = STATUS_SUCCESS;

            RtlInitUnicodeString(
                &raw mut KeyName,
                RUSTCONV_WSTR_8D22E72486A8.as_ptr(),
            );
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = RootKeys[GetPredefKeyIndex!(HKEY_LOCAL_MACHINE) as usize].Handle;
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.ObjectName = &raw mut KeyName;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = NtCreateKey(
                &raw mut KeyHandle,
                KEY_ALL_ACCESS,
                &raw mut ObjectAttributes,
                0,
                core::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                &raw mut Disposition,
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(b"NtCreateKey() failed to create the ControlSet001 key (Status %lx)\n\0".as_ptr() as *const i8, Status);
            } else {
                DPRINT!(b"NtCreateKey() succeeded to %s the ControlSet001 key (Status %lx)\n\0".as_ptr() as *const i8,
                       if Disposition == REG_CREATED_NEW_KEY { b"create\0".as_ptr() as *const i8 } else { b"open\0".as_ptr() as *const i8 },
                       Status);
            }
            NtClose(KeyHandle);

            Status = CreateSymLinkKey(
                RootKeys[GetPredefKeyIndex!(HKEY_LOCAL_MACHINE) as usize].Handle,
                RUSTCONV_WSTR_8DA4DAC13B96.as_ptr(),
                RUSTCONV_WSTR_A34E7EC9273F.as_ptr(),
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(b"CreateSymLinkKey(CurrentControlSet) failed, Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
            }

            Status = STATUS_SUCCESS;
        }

        RtlAdjustPrivilege(
            SE_BACKUP_PRIVILEGE,
            PrivilegeSet[1],
            (FALSE) as BOOLEAN,
            &raw mut PrivilegeSet[1],
        );
        RtlAdjustPrivilege(
            SE_RESTORE_PRIVILEGE,
            PrivilegeSet[0],
            (FALSE) as BOOLEAN,
            &raw mut PrivilegeSet[0],
        );

        return Status;
    }
}
#[no_mangle]
pub unsafe extern "C" fn RegCleanupRegistry(NtSystemRoot: PUNICODE_STRING) {
    let mut Status: NTSTATUS;
    let mut KeyHandle: *mut core::ffi::c_void;
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PrivilegeSet: [u8; 2] = [(FALSE) as u8, (FALSE) as u8];
    let mut i: u32;
    let mut SrcPath: [u16; (MAX_PATH) as _] = core::mem::zeroed();
    let mut DstPath: [u16; (MAX_PATH) as _] = core::mem::zeroed();

    Status = RtlAdjustPrivilege(
        SE_RESTORE_PRIVILEGE,
        (TRUE) as BOOLEAN,
        (FALSE) as BOOLEAN,
        &mut PrivilegeSet[0] as *mut u8,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"RtlAdjustPrivilege(SE_RESTORE_PRIVILEGE) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        return;
    }

    Status = RtlAdjustPrivilege(
        SE_BACKUP_PRIVILEGE,
        (TRUE) as BOOLEAN,
        (FALSE) as BOOLEAN,
        &mut PrivilegeSet[1] as *mut u8,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"RtlAdjustPrivilege(SE_BACKUP_PRIVILEGE) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        RtlAdjustPrivilege(
            SE_RESTORE_PRIVILEGE,
            PrivilegeSet[0],
            (FALSE) as BOOLEAN,
            &mut PrivilegeSet[0] as *mut u8,
        );
        return;
    }

    i = 0;
    while (i as u64) < (ARRAYSIZE!(RegistryHives) as u64) {
        if RegistryHives[i as usize].State == Create || RegistryHives[i as usize].State == Repair {
            Status = DeleteSymLinkKey(
                RootKeys[GetPredefKeyIndex!(RegistryHives[i as usize].PredefKeyHandle) as usize].Handle,
                RegistryHives[i as usize].RegSymLink,
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(
                    b"DeleteSymLinkKey(%S) failed, Status 0x%08lx\n\0".as_ptr() as *const i8,
                    RegistryHives[i as usize].RegSymLink,
                    Status,
                );
            }

            Status = DisconnectRegistry(
                core::ptr::null_mut(),
                RegistryHives[i as usize].HiveRegistryPath,
                1,
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(
                    b"Unmounting '%S' failed\n\0".as_ptr() as *const i8,
                    RegistryHives[i as usize].HiveRegistryPath,
                );
            }

            RegistryHives[i as usize].State = Update;
        } else {
            RtlInitUnicodeString(&mut KeyName, RegistryHives[i as usize].RegSymLink);
            InitializeObjectAttributes!(
                &mut ObjectAttributes,
                &mut KeyName,
                OBJ_CASE_INSENSITIVE,
                RootKeys[GetPredefKeyIndex!(RegistryHives[i as usize].PredefKeyHandle) as usize].Handle,
                core::ptr::null_mut(),
            );
            KeyHandle = core::ptr::null_mut();
            Status = NtOpenKey(
                &mut KeyHandle,
                DELETE,
                &mut ObjectAttributes,
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(
                    b"NtOpenKey(%wZ) failed, Status 0x%08lx\n\0".as_ptr() as *const i8,
                    &mut KeyName,
                    Status,
                );
            }

            NtDeleteKey(KeyHandle);
            NtClose(KeyHandle);
        }
        i = i.wrapping_add(1);
    }

    i = 0;
    while (i as u64) < (ARRAYSIZE!(RootKeys) as u64) {
        if !RootKeys[i as usize].Handle.is_null() {
            NtFlushKey(RootKeys[i as usize].Handle);
            NtDeleteKey(RootKeys[i as usize].Handle);
            NtClose(RootKeys[i as usize].Handle);
            RootKeys[i as usize].Handle = core::ptr::null_mut();
        }
        i = i.wrapping_add(1);
    }

    i = 0;
    while (i as u64) < (ARRAYSIZE!(RegistryHives) as u64) {
        if RegistryHives[i as usize].State != Create && RegistryHives[i as usize].State != Repair {
            i = i.wrapping_add(1);
            continue;
        }

        CombinePaths(
            SrcPath.as_mut_ptr(),
            (ARRAYSIZE!(SrcPath)) as _,
            3,
            (*NtSystemRoot).Buffer,
            RUSTCONV_WSTR_0C8C471E3DBF.as_ptr(),
            RegistryHives[i as usize].HiveName,
        );
        RtlStringCchCopyW(DstPath.as_mut_ptr(), (ARRAYSIZE!(DstPath)) as _, SrcPath.as_mut_ptr());
        RtlStringCchCatW(DstPath.as_mut_ptr(), (ARRAYSIZE!(DstPath)) as _, RUSTCONV_WSTR_D75304D3B217.as_ptr());

        DPRINT1!(
            b"Copy hive: %S ==> %S\n\0".as_ptr() as *const i8,
            SrcPath.as_mut_ptr(),
            DstPath.as_mut_ptr(),
        );
        Status = SetupCopyFile(SrcPath.as_mut_ptr(), DstPath.as_mut_ptr(), (FALSE) as BOOLEAN);
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(
                b"SetupCopyFile() failed (Status %lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
        i = i.wrapping_add(1);
    }

    RtlAdjustPrivilege(
        SE_BACKUP_PRIVILEGE,
        PrivilegeSet[1],
        (FALSE) as BOOLEAN,
        &mut PrivilegeSet[1] as *mut u8,
    );
    RtlAdjustPrivilege(
        SE_RESTORE_PRIVILEGE,
        PrivilegeSet[0],
        (FALSE) as BOOLEAN,
        &mut PrivilegeSet[0] as *mut u8,
    );
}
