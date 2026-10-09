use crate::setuplib::inicache::IniAddKey;
use crate::setuplib::inicache::IniAddSection;
use crate::setuplib::inicache::IniCacheCreate;
use crate::setuplib::inicache::IniCacheDestroy;
use crate::setuplib::inicache::IniCacheLoadByHandle;
use crate::setuplib::inicache::IniCacheLoadFromMemory;
use crate::setuplib::inicache::IniCacheSaveByHandle;
use crate::setuplib::inicache::IniFindClose;
use crate::setuplib::inicache::IniFindFirstValue;
use crate::setuplib::inicache::IniFindNextValue;
use crate::setuplib::inicache::IniGetKey;
use crate::setuplib::inicache::IniGetSection;
use crate::setuplib::inicache::IniInsertKey;
use crate::setuplib::inicache::IniRemoveKeyByName;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_02648A408E10: [u16; 18] = [
    0x006Fu16, 0x0070, 0x0065, 0x0072, 0x0061, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020,
    0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_09A7C6FF8E65: [u16; 8] = [0x0044u16, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079, 0x0000];
pub static RUSTCONV_WSTR_0EB6F1F84A4E: [u16; 9] = [0x0042u16, 0x006F, 0x006F, 0x0074, 0x0050, 0x0061, 0x0074, 0x0068, 0x0000];
pub static RUSTCONV_WSTR_1441070535FF: [u16; 11] = [
    0x0046u16, 0x0052, 0x0045, 0x0045, 0x004C, 0x004F, 0x0041, 0x0044, 0x0045, 0x0052,
    0x0000,
];
pub static RUSTCONV_WSTR_1B9334FEECE6: [u16; 2] = [0x0022u16, 0x0000];
pub static RUSTCONV_WSTR_1BEF59FC85F5: [u16; 12] = [
    0x0057u16, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0032, 0x0030, 0x0030,
    0x0033, 0x0000,
];
pub static RUSTCONV_WSTR_1C8C566347BF: [u16; 12] = [
    0x0022u16, 0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E,
    0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_20FA757B49DF: [u16; 9] = [0x0066u16, 0x006C, 0x0065, 0x0078, 0x0062, 0x006F, 0x006F, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_228EDDCD2F3F: [u16; 14] = [
    0x0042u16, 0x006F, 0x006F, 0x0074, 0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074,
    0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_22FF6DE5922B: [u16; 8] = [0x0022u16, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_2D4564AAFD0B: [u16; 13] = [
    0x0022u16, 0x0042, 0x006F, 0x006F, 0x0074, 0x0053, 0x0065, 0x0063, 0x0074, 0x006F,
    0x0072, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_32434DC5B0F7: [u16; 2] = [0x002Cu16, 0x0000];
pub static RUSTCONV_WSTR_32491EF24F87: [u16; 6] = [0x0044u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_390B38887995: [u16; 10] = [0x0050u16, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_3ADB4FA7D636: [u16; 12] = [
    0x0062u16, 0x006F, 0x006F, 0x0074, 0x0020, 0x006C, 0x006F, 0x0061, 0x0064, 0x0065,
    0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_3EACE4A3EFEE: [u16; 26] = [
    0x006Du16, 0x0075, 0x006C, 0x0074, 0x0069, 0x0028, 0x0030, 0x0029, 0x0064, 0x0069,
    0x0073, 0x006B, 0x0028, 0x0030, 0x0029, 0x0063, 0x0064, 0x0072, 0x006F, 0x006D,
    0x0028, 0x0025, 0x006C, 0x0075, 0x0029, 0x0000,
];
pub static RUSTCONV_WSTR_4D54CCBC2C38: [u16; 10] = [0x0042u16, 0x006F, 0x006F, 0x0074, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_4F8320D91E97: [u16; 2] = [0x0030u16, 0x0000];
pub static RUSTCONV_WSTR_4F964A2F5D70: [u16; 11] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0050, 0x0061, 0x0074, 0x0068,
    0x0000,
];
pub static RUSTCONV_WSTR_604AF35F155D: [u16; 18] = [
    0x004Fu16, 0x0070, 0x0065, 0x0072, 0x0061, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020,
    0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_68F952C868EF: [u16; 8] = [0x004Fu16, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_75E7841C9907: [u16; 4] = [0x0059u16, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_81AF36117C81: [u16; 40] = [
    0x006Du16, 0x0075, 0x006C, 0x0074, 0x0069, 0x0028, 0x0030, 0x0029, 0x0064, 0x0069,
    0x0073, 0x006B, 0x0028, 0x0030, 0x0029, 0x0072, 0x0064, 0x0069, 0x0073, 0x006B,
    0x0028, 0x0025, 0x006C, 0x0075, 0x0029, 0x0070, 0x0061, 0x0072, 0x0074, 0x0069,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0028, 0x0025, 0x006C, 0x0075, 0x0029, 0x0000,
];
pub static RUSTCONV_WSTR_855FBB324137: [u16; 8] = [0x0054u16, 0x0069, 0x006D, 0x0065, 0x004F, 0x0075, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_85631BB3D821: [u16; 10] = [0x004Du16, 0x0069, 0x006E, 0x0069, 0x006D, 0x0061, 0x006C, 0x0055, 0x0049, 0x0000];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_9382A66BEF74: [u16; 10] = [0x0054u16, 0x0069, 0x0074, 0x006C, 0x0065, 0x0054, 0x0065, 0x0078, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_94F106D42113: [u16; 21] = [
    0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0042, 0x006F,
    0x006F, 0x0074, 0x0020, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072,
    0x0000,
];
pub static RUSTCONV_WSTR_9975EA9283F5: [u16; 11] = [
    0x0042u16, 0x006F, 0x006F, 0x0074, 0x0053, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072,
    0x0000,
];
pub static RUSTCONV_WSTR_A9EDBA2FC474: [u16; 3] = [0x0025u16, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_D0DD11036E34: [u16; 10] = [0x006Du16, 0x0075, 0x006C, 0x0074, 0x0069, 0x0062, 0x006F, 0x006F, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_DE61798F28B9: [u16; 15] = [
    0x0042u16, 0x006F, 0x006F, 0x0074, 0x0053, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072,
    0x0046, 0x0069, 0x006C, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_E99788FBE09D: [u16; 14] = [
    0x0022u16, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0032, 0x0030,
    0x0030, 0x0033, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_FACD2F68A4C6: [u16; 9] = [0x0042u16, 0x006F, 0x006F, 0x0074, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_FDD69F3CB5F1: [u16; 26] = [
    0x006Du16, 0x0075, 0x006C, 0x0074, 0x0069, 0x0028, 0x0030, 0x0029, 0x0064, 0x0069,
    0x0073, 0x006B, 0x0028, 0x0030, 0x0029, 0x0066, 0x0064, 0x0069, 0x0073, 0x006B,
    0x0028, 0x0025, 0x006C, 0x0075, 0x0029, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_01AC0DF2395A: [u16; 12] = [
    0x0066u16, 0x0072, 0x0065, 0x0065, 0x006C, 0x0064, 0x0072, 0x002E, 0x0069, 0x006E,
    0x0069, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_0783B6D69C02: [u16; 9] = [0x0062u16, 0x006F, 0x006F, 0x0074, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static RUSTCONV_C2R_WSTR_1BB2219DFCDD: [u16; 20] = [
    0x006Eu16, 0x0074, 0x006C, 0x0064, 0x0072, 0x0000, 0x006F, 0x0073, 0x006C, 0x006F,
    0x0061, 0x0064, 0x0065, 0x0072, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_3F44601EDF78: [u16; 25] = [
    0x0066u16, 0x0072, 0x0065, 0x0065, 0x006C, 0x0064, 0x0072, 0x002E, 0x0073, 0x0079,
    0x0073, 0x0000, 0x0072, 0x006F, 0x0073, 0x006C, 0x006F, 0x0061, 0x0064, 0x002E,
    0x0065, 0x0078, 0x0065, 0x0000, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_6B2401E429DC: [u16; 10] = [0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x004F, 0x0053, 0x0000];
pub static RUSTCONV_C2R_WSTR_72BDF4B795E9: [u16; 8] = [0x0064u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0000];
pub static RUSTCONV_C2R_WSTR_855FBB324137: [u16; 8] = [0x0054u16, 0x0069, 0x006D, 0x0065, 0x004F, 0x0075, 0x0074, 0x0000];
pub static RUSTCONV_C2R_WSTR_FD1F214427BD: [u16; 8] = [0x0074u16, 0x0069, 0x006D, 0x0065, 0x006F, 0x0075, 0x0074, 0x0000];

pub static mut NtosBootLoaders: [NTOS_BOOT_LOADER_FILES; 2] = unsafe {
    [
        _NTOS_BOOT_LOADER_FILES {
            Type: FreeLdr,
            LoaderExecutables: RUSTCONV_C2R_WSTR_3F44601EDF78.as_ptr() as PCZZWSTR,
            LoaderConfigurationFile:
                RUSTCONV_C2R_WSTR_01AC0DF2395A.as_ptr() as PCWSTR,
            OpenBootStore: Some(
                OpenIniBootLoaderStore
                    as unsafe extern "C" fn(
                        *mut PVOID,
                        HANDLE,
                        BOOT_STORE_TYPE,
                        BOOT_STORE_OPENMODE,
                        BOOT_STORE_ACCESS,
                    ) -> NTSTATUS,
            ),
            CloseBootStore: Some(
                CloseIniBootLoaderStore as unsafe extern "C" fn(PVOID) -> NTSTATUS,
            ),
            EnumBootStoreEntries: ::core::mem::transmute::<
                Option<
                    unsafe extern "C" fn(
                        PBOOT_STORE_INI_CONTEXT,
                        PENUM_BOOT_ENTRIES_ROUTINE,
                        PVOID,
                    ) -> NTSTATUS,
                >,
                PENUM_BOOT_STORE_ENTRIES,
            >(Some(
                FreeLdrEnumerateBootEntries
                    as unsafe extern "C" fn(
                        PBOOT_STORE_INI_CONTEXT,
                        PENUM_BOOT_ENTRIES_ROUTINE,
                        PVOID,
                    ) -> NTSTATUS,
            )),
        },
        _NTOS_BOOT_LOADER_FILES {
            Type: NtLdr,
            LoaderExecutables: RUSTCONV_C2R_WSTR_1BB2219DFCDD.as_ptr() as PCZZWSTR,
            LoaderConfigurationFile: RUSTCONV_C2R_WSTR_0783B6D69C02.as_ptr() as PCWSTR,
            OpenBootStore: Some(
                OpenIniBootLoaderStore
                    as unsafe extern "C" fn(
                        *mut PVOID,
                        HANDLE,
                        BOOT_STORE_TYPE,
                        BOOT_STORE_OPENMODE,
                        BOOT_STORE_ACCESS,
                    ) -> NTSTATUS,
            ),
            CloseBootStore: Some(
                CloseIniBootLoaderStore as unsafe extern "C" fn(PVOID) -> NTSTATUS,
            ),
            EnumBootStoreEntries: ::core::mem::transmute::<
                Option<
                    unsafe extern "C" fn(
                        PBOOT_STORE_INI_CONTEXT,
                        PENUM_BOOT_ENTRIES_ROUTINE,
                        PVOID,
                    ) -> NTSTATUS,
                >,
                PENUM_BOOT_STORE_ENTRIES,
            >(Some(
                NtLdrEnumerateBootEntries
                    as unsafe extern "C" fn(
                        PBOOT_STORE_INI_CONTEXT,
                        PENUM_BOOT_ENTRIES_ROUTINE,
                        PVOID,
                    ) -> NTSTATUS,
            )),
        },
    ]
};

static mut BootOptionNames: [[PCWSTR; 2]; 2] = unsafe {
    [
        [
            RUSTCONV_C2R_WSTR_855FBB324137.as_ptr(),
            RUSTCONV_C2R_WSTR_6B2401E429DC.as_ptr(),
        ],
        [
            RUSTCONV_C2R_WSTR_FD1F214427BD.as_ptr(),
            RUSTCONV_C2R_WSTR_72BDF4B795E9.as_ptr(),
        ],
    ]
};

pub const NTOS_OPTIONS_SIGNATURE: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"NTOS_5\0\0\0") };

pub const BOOTSECTOR_OPTIONS_SIGNATURE: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"BootSect\0") };

pub type POPEN_BOOT_STORE = Option<unsafe extern "C" fn(
    Handle: *mut PVOID,
    PartitionDirectoryHandle: HANDLE,
    Type: BOOT_STORE_TYPE,
    OpenMode: BOOT_STORE_OPENMODE,
    Access: BOOT_STORE_ACCESS,
) -> NTSTATUS>;

pub type PCLOSE_BOOT_STORE = Option<unsafe extern "C" fn(
    Handle: PVOID,
) -> NTSTATUS>;

pub type PENUM_BOOT_STORE_ENTRIES = Option<unsafe extern "C" fn(
    Handle: PVOID,
    EnumBootEntriesRoutine: PENUM_BOOT_ENTRIES_ROUTINE,
    Parameter: PVOID,
) -> NTSTATUS>;

#[repr(C)]
pub struct _NTOS_BOOT_LOADER_FILES {
    pub Type: BOOT_STORE_TYPE,
    pub LoaderExecutables: PCZZWSTR,
    pub LoaderConfigurationFile: PCWSTR,
    pub OpenBootStore: POPEN_BOOT_STORE,
    pub CloseBootStore: PCLOSE_BOOT_STORE,
    pub EnumBootStoreEntries: PENUM_BOOT_STORE_ENTRIES,
}
pub type NTOS_BOOT_LOADER_FILES = _NTOS_BOOT_LOADER_FILES;
pub type PNTOS_BOOT_LOADER_FILES = *mut _NTOS_BOOT_LOADER_FILES;

#[repr(C)]
pub struct _BOOT_STORE_CONTEXT {
    pub Type: BOOT_STORE_TYPE,
    pub ReadOnly: BOOLEAN,
}
pub type BOOT_STORE_CONTEXT = _BOOT_STORE_CONTEXT;
pub type PBOOT_STORE_CONTEXT = *mut _BOOT_STORE_CONTEXT;

#[repr(C)]
pub struct _BOOT_STORE_INI_CONTEXT {
    pub Header: BOOT_STORE_CONTEXT,
    pub FileHandle: HANDLE,
    pub SectionHandle: HANDLE,
    pub FileSize: ULONG,
    pub ViewBase: PVOID,
    pub IniCache: PINICACHE,
    pub OptionsIniSection: PINI_SECTION,
    pub OsIniSection: PINI_SECTION,
}
pub type BOOT_STORE_INI_CONTEXT = _BOOT_STORE_INI_CONTEXT;
pub type PBOOT_STORE_INI_CONTEXT = *mut _BOOT_STORE_INI_CONTEXT;

#[repr(C)]
pub struct _BOOT_STORE_BCDREG_CONTEXT {
    pub Header: BOOT_STORE_CONTEXT,
    pub PlaceHolder: ULONG,
}
pub type BOOT_STORE_BCDREG_CONTEXT = _BOOT_STORE_BCDREG_CONTEXT;
pub type PBOOT_STORE_BCDREG_CONTEXT = *mut _BOOT_STORE_BCDREG_CONTEXT;

pub const BO_TimeOut: i32 = 0;
pub const BO_DefaultOS: i32 = 1;
#[no_mangle]
pub unsafe extern "C" fn FindBootStore(
    PartitionDirectoryHandle: HANDLE,
    Type: BOOT_STORE_TYPE,
    VersionNumber: PULONG,
) -> NTSTATUS {
    let mut LoaderExecutable: PCWSTR = core::ptr::null_mut();

    if Type as u32 >= BldrTypeMax as u32 {
        return STATUS_INVALID_PARAMETER;
    }

    if !VersionNumber.is_null() {
        *VersionNumber = 0;
    }

    LoaderExecutable = NtosBootLoaders[Type as usize].LoaderExecutables as PCWSTR;
    while *LoaderExecutable != 0 {
        if DoesPathExist(PartitionDirectoryHandle, LoaderExecutable, FALSE as BOOLEAN) != 0 {
            DPRINT!("Found loader executable '%S'\n", LoaderExecutable);
            break;
        }

        DPRINT!("Loader executable '%S' does not exist, continue with another one...\n", LoaderExecutable);
        LoaderExecutable = LoaderExecutable.offset(
            wcslen(LoaderExecutable).wrapping_add(1) as isize,
        );
    }

    if *LoaderExecutable == 0 {
        DPRINT!("No loader executable was found\n");
        return STATUS_NOT_FOUND;
    }

    if !VersionNumber.is_null() {
        *VersionNumber = 0;
    }

    if DoesPathExist(PartitionDirectoryHandle, NtosBootLoaders[Type as usize].LoaderConfigurationFile, FALSE as BOOLEAN) == 0 {
        DPRINT1!("Loader configuration file '%S' does not exist\n", NtosBootLoaders[Type as usize].LoaderConfigurationFile);
        return STATUS_NOT_FOUND;
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn FreeLdrMigrateBootDrivePartWorker(mut OsIniSection: PINI_SECTION) {
    let mut KeyData: PCWSTR = core::ptr::null();
    let mut OldKey: PINI_KEYWORD = core::ptr::null_mut();

    if !IniGetKey(OsIniSection, RUSTCONV_WSTR_0EB6F1F84A4E.as_ptr(), &mut KeyData).is_null()
        && !KeyData.is_null()
        && *KeyData as i32 != 0
    {
        return;
    }

    OldKey = IniGetKey(OsIniSection, RUSTCONV_WSTR_4D54CCBC2C38.as_ptr(), &mut KeyData);
    if !OldKey.is_null() {
        let OldDrive: PCWSTR = KeyData;
        let mut DriveNumber: ULONG = 0;
        let mut PartitionNumber: ULONG = 0;
        let mut DriveType: UCHAR = 0;
        let mut BufferBootPath: [WCHAR; 80] = [0; 80];

        let mut p: PCWCH = KeyData as PCWCH;
        if *p.offset(0 as isize) as i32 >= 0x0030u16 as i32
            && *p.offset(0 as isize) as i32 <= 0x0039u16 as i32
        {
            DriveNumber = wcstoul(p, &mut p as *mut PCWCH as *mut *mut u16, 0) as ULONG;
            if DriveNumber >= 0x80 {
                DriveNumber = DriveNumber.wrapping_sub(0x80);
                DriveType = 0x0068u16 as UCHAR;
            } else {
                DriveType = 0x0066u16 as UCHAR;
            }
        } else if *p.offset(0 as isize) as i32 != 0
            && towlower(*p.offset(1 as isize)) as i32
                == 0x0064u16 as i32
        {
            DriveType = tolower(*p.offset(0 as isize) as i32) as UCHAR;
            DriveNumber = _wtoi(p.offset(2 as isize)) as ULONG;
        }

        if !IniGetKey(OsIniSection, RUSTCONV_WSTR_228EDDCD2F3F.as_ptr(), &mut KeyData).is_null() {
            PartitionNumber = _wtoi(KeyData) as ULONG;
        }

        if DriveType as i32 == 0x0066u16 as i32 {
            RtlStringCchPrintfW!(
                BufferBootPath.as_mut_ptr(),
                _countof!(BufferBootPath),
                RUSTCONV_WSTR_FDD69F3CB5F1.as_ptr(),
                DriveNumber,
            );
        } else if DriveType as i32 == 0x0068u16 as i32 {
            RtlStringCchPrintfW!(
                BufferBootPath.as_mut_ptr(),
                _countof!(BufferBootPath),
                RUSTCONV_WSTR_81AF36117C81.as_ptr(),
                DriveNumber,
                PartitionNumber,
            );
        } else if DriveType as i32 == 0x0063u16 as i32 {
            RtlStringCchPrintfW!(
                BufferBootPath.as_mut_ptr(),
                _countof!(BufferBootPath),
                RUSTCONV_WSTR_3EACE4A3EFEE.as_ptr(),
                DriveNumber,
            );
        } else {
            DPRINT1!(
                "Unrecognized BootDrive type '%C'\n",
                if DriveType as i32 != 0 { DriveType as i32 } else { 0x003Fu16 as i32 },
            );

            RtlStringCchCopyW(BufferBootPath.as_mut_ptr(), (_countof!(BufferBootPath)) as _, OldDrive);
            if !KeyData.is_null() && *KeyData as i32 != 0 {
                RtlStringCchCatW(BufferBootPath.as_mut_ptr(), (_countof!(BufferBootPath)) as _, RUSTCONV_WSTR_32434DC5B0F7.as_ptr());
                RtlStringCchCatW(BufferBootPath.as_mut_ptr(), (_countof!(BufferBootPath)) as _, KeyData);
            }
        }

        IniInsertKey(
            OsIniSection,
            OldKey,
            INSERT_BEFORE,
            RUSTCONV_WSTR_0EB6F1F84A4E.as_ptr(),
            BufferBootPath.as_mut_ptr(),
        );
    }

    IniRemoveKeyByName(OsIniSection, RUSTCONV_WSTR_4D54CCBC2C38.as_ptr());
    IniRemoveKeyByName(OsIniSection, RUSTCONV_WSTR_228EDDCD2F3F.as_ptr());
}

unsafe extern "C" fn FreeLdrMigrateBootDrivePart(
    BootStore: PBOOT_STORE_INI_CONTEXT,
) {
    let mut Iterator: PINICACHEITERATOR;
    let mut OsIniSection: PINI_SECTION;
    let mut SectionName: PCWSTR = core::ptr::null();
    let mut KeyData: PCWSTR = core::ptr::null();

    Iterator = IniFindFirstValue((*BootStore).OsIniSection, &mut SectionName, &mut KeyData);
    if Iterator.is_null() {
        return;
    }
    loop {
        // C do { ... continue; ... } while (IniFindNextValue(...)): a `continue`
        // goes to the condition, so it is `break 'body` here.
        'body: {
        OsIniSection = IniGetSection((*BootStore).IniCache, SectionName);
        if OsIniSection.is_null() {
            break 'body;
        }

        if IniGetKey(OsIniSection, RUSTCONV_WSTR_FACD2F68A4C6.as_ptr(), &mut KeyData).is_null()
            || KeyData.is_null()
        {
            DPRINT1!("No BootType value present\n");
            break 'body;
        }
        if (_wcsicmp(KeyData, RUSTCONV_WSTR_32491EF24F87.as_ptr()) == 0)
            || (_wcsicmp(KeyData, RUSTCONV_WSTR_22FF6DE5922B.as_ptr()) == 0)
            || (_wcsicmp(KeyData, RUSTCONV_WSTR_390B38887995.as_ptr()) == 0)
            || (_wcsicmp(KeyData, RUSTCONV_WSTR_1C8C566347BF.as_ptr()) == 0)
        {
            IniAddKey(OsIniSection, RUSTCONV_WSTR_FACD2F68A4C6.as_ptr(), RUSTCONV_WSTR_9975EA9283F5.as_ptr());
            DPRINT!("This is a '%S' boot entry\n", KeyData);
            FreeLdrMigrateBootDrivePartWorker(OsIniSection);
        } else if (_wcsicmp(KeyData, RUSTCONV_WSTR_9975EA9283F5.as_ptr()) == 0)
            || (_wcsicmp(KeyData, RUSTCONV_WSTR_2D4564AAFD0B.as_ptr()) == 0)
        {
            DPRINT!("This is a '%S' boot entry\n", KeyData);
            FreeLdrMigrateBootDrivePartWorker(OsIniSection);
        }
        }

        if ((IniFindNextValue(Iterator, &mut SectionName, &mut KeyData)) == 0) {
            break;
        }
    }

    IniFindClose(Iterator);
}

unsafe extern "C" fn CreateCommonFreeLdrSections(mut BootStore: PBOOT_STORE_INI_CONTEXT) {
    let mut IniSection: PINI_SECTION = core::ptr::null_mut();

    IniSection = IniAddSection((*BootStore).IniCache, RUSTCONV_WSTR_1441070535FF.as_ptr());
    if IniSection.is_null() {
        DPRINT1!("CreateCommonFreeLdrSections: Failed to create 'FREELOADER' section!\n");
    }

    (*BootStore).OptionsIniSection = IniSection;

    IniAddKey((*BootStore).OptionsIniSection, RUSTCONV_WSTR_855FBB324137.as_ptr(), RUSTCONV_WSTR_4F8320D91E97.as_ptr());

    IniSection = IniAddSection((*BootStore).IniCache, RUSTCONV_WSTR_09A7C6FF8E65.as_ptr());

    IniAddKey(IniSection, RUSTCONV_WSTR_9382A66BEF74.as_ptr(), RUSTCONV_WSTR_94F106D42113.as_ptr());
    IniAddKey(IniSection, RUSTCONV_WSTR_85631BB3D821.as_ptr(), RUSTCONV_WSTR_75E7841C9907.as_ptr());

    IniSection = IniAddSection((*BootStore).IniCache, RUSTCONV_WSTR_604AF35F155D.as_ptr());
    if IniSection.is_null() {
        DPRINT1!("CreateCommonFreeLdrSections: Failed to create 'Operating Systems' section!\n");
    }

    (*BootStore).OsIniSection = IniSection;
}

unsafe extern "C" fn OpenIniBootLoaderStore(
    Handle: *mut *mut core::ffi::c_void,
    PartitionDirectoryHandle: *mut core::ffi::c_void,
    Type: u32,
    OpenMode: u32,
    Access: u32,
) -> i32 {
    let mut Status: i32;
    let mut BootStore: *mut BOOT_STORE_INI_CONTEXT;
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DesiredAccess: ACCESS_MASK;
    let mut CreateDisposition: u32;

    if Type == NtLdr && (OpenMode == BS_CreateNew || OpenMode == BS_CreateAlways || OpenMode == BS_RecreateExisting) {
        DPRINT1!(b"OpenIniBootLoaderStore() unsupported for NTLDR\n\0".as_ptr() as *const i8);
        return STATUS_NOT_SUPPORTED;
    }

    BootStore = RtlAllocateHeap(
        ProcessHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<BOOT_STORE_INI_CONTEXT>()) as SIZE_T,
    ) as *mut BOOT_STORE_INI_CONTEXT;
    if BootStore.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*BootStore).Header.Type = Type;

    (*BootStore).SectionHandle = core::ptr::null_mut();
    (*BootStore).ViewBase = core::ptr::null_mut();
    (*BootStore).FileSize = 0;

    RtlInitUnicodeString(
        &mut Name,
        NtosBootLoaders[Type as usize].LoaderConfigurationFile,
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = PartitionDirectoryHandle;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut Name;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    DesiredAccess = (if Access & BS_ReadAccess != 0 {
        FILE_GENERIC_READ
    } else {
        0
    }) | (if Access & BS_WriteAccess != 0 {
        FILE_GENERIC_WRITE
    } else {
        0
    });

    CreateDisposition = FILE_OPEN;
    match OpenMode {
        BS_CreateNew => {
            CreateDisposition = FILE_CREATE;
        }
        _BOOT_STORE_OPENMODE_BS_CheckExisting | BS_OpenExisting => {
            CreateDisposition = FILE_OPEN;
        }
        BS_OpenAlways => {
            CreateDisposition = FILE_OPEN_IF;
        }
        BS_RecreateExisting => {
            CreateDisposition = FILE_OVERWRITE;
        }
        BS_CreateAlways => {
            CreateDisposition = FILE_OVERWRITE_IF;
        }
        _ => {
            ASSERT!(FALSE);
        }
    }

    IoStatusBlock.Information = 0;
    Status = NtCreateFile(
        &mut (*BootStore).FileHandle,
        DesiredAccess | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        FILE_SHARE_READ,
        CreateDisposition,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_SEQUENTIAL_ONLY | FILE_NON_DIRECTORY_FILE,
        core::ptr::null_mut(),
        0,
    );

    if OpenMode == _BOOT_STORE_OPENMODE_BS_CheckExisting {
        let Success: u8 = (NT_SUCCESS!(Status) != 0 || Status == STATUS_ACCESS_DENIED) as u8;
        if Success == 0 {
            DPRINT1!(
                b"Couldn't find Loader configuration file '%S'\n\0".as_ptr() as *const i8,
                NtosBootLoaders[Type as usize].LoaderConfigurationFile,
            );
        }
        if !(*BootStore).FileHandle.is_null() {
            NtClose((*BootStore).FileHandle);
        }
        RtlFreeHeap(ProcessHeap, 0, BootStore as *mut core::ffi::c_void);
        return if Success != 0 {
            STATUS_SUCCESS
        } else {
            Status
        };
    }

    if Status == STATUS_ACCESS_DENIED {
        let mut FileInfo: FILE_BASIC_INFORMATION = core::mem::zeroed();

        Status = NtCreateFile(
            &mut (*BootStore).FileHandle,
            FILE_WRITE_ATTRIBUTES | SYNCHRONIZE,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            core::ptr::null_mut(),
            FILE_ATTRIBUTE_NORMAL,
            FILE_SHARE_READ,
            FILE_OPEN,
            FILE_NO_INTERMEDIATE_BUFFERING
                | FILE_SYNCHRONOUS_IO_NONALERT
                | FILE_SEQUENTIAL_ONLY
                | FILE_NON_DIRECTORY_FILE,
            core::ptr::null_mut(),
            0,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            // break out of the do-while
        } else {
            FileInfo.FileAttributes = FILE_ATTRIBUTE_NORMAL;
            NtSetInformationFile(
                (*BootStore).FileHandle,
                &mut IoStatusBlock,
                &mut FileInfo as *mut FILE_BASIC_INFORMATION as *mut core::ffi::c_void,
                core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
                FileBasicInformation,
            );

            NtClose((*BootStore).FileHandle);

            Status = NtCreateFile(
                &mut (*BootStore).FileHandle,
                DesiredAccess | SYNCHRONIZE,
                &mut ObjectAttributes,
                &mut IoStatusBlock,
                core::ptr::null_mut(),
                FILE_ATTRIBUTE_NORMAL,
                FILE_SHARE_READ,
                CreateDisposition,
                FILE_SYNCHRONOUS_IO_NONALERT | FILE_SEQUENTIAL_ONLY | FILE_NON_DIRECTORY_FILE,
                core::ptr::null_mut(),
                0,
            );
        }
    }

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"Couldn't open Loader configuration file '%S' (Status 0x%08lx)\n\0".as_ptr()
                as *const i8,
            NtosBootLoaders[Type as usize].LoaderConfigurationFile,
            Status,
        );
        RtlFreeHeap(ProcessHeap, 0, BootStore as *mut core::ffi::c_void);
        return Status;
    }

    (*BootStore).Header.ReadOnly = (Access & BS_WriteAccess == 0) as u8;

    if IoStatusBlock.Information == (FILE_CREATED) as _
        || IoStatusBlock.Information == (FILE_OVERWRITTEN) as _
        || IoStatusBlock.Information == (FILE_SUPERSEDED) as _
    {
        (*BootStore).IniCache = IniCacheCreate();
        if (*BootStore).IniCache.is_null() {
            DPRINT1!(b"IniCacheCreate() failed\n\0".as_ptr() as *const i8);
            NtClose((*BootStore).FileHandle);
            RtlFreeHeap(ProcessHeap, 0, BootStore as *mut core::ffi::c_void);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        if Type == FreeLdr {
            CreateCommonFreeLdrSections(BootStore);
        }
    } else {
        let mut IniSection: *mut INI_SECTION;

        (*BootStore).SectionHandle = UlongToPtr!(1) as *mut core::ffi::c_void;

        Status = IniCacheLoadByHandle(
            &mut (*BootStore).IniCache,
            (*BootStore).FileHandle,
            (FALSE) as BOOLEAN,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(
                b"IniCacheLoadFromMemory() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
            NtClose((*BootStore).FileHandle);
            RtlFreeHeap(ProcessHeap, 0, BootStore as *mut core::ffi::c_void);
            return Status;
        }

        if Type == FreeLdr {
            IniSection = IniAddSection((*BootStore).IniCache, RUSTCONV_WSTR_1441070535FF.as_ptr());
            if IniSection.is_null() {
                DPRINT1!(
                    b"OpenIniBootLoaderStore: Failed to retrieve 'FREELOADER' section!\n\0"
                        .as_ptr() as *const i8,
                );
            }

            (*BootStore).OptionsIniSection = IniSection;

            IniSection = IniAddSection((*BootStore).IniCache, RUSTCONV_WSTR_604AF35F155D.as_ptr());
            if IniSection.is_null() {
                DPRINT1!(
                    b"OpenIniBootLoaderStore: Failed to retrieve 'Operating Systems' section!\n\0"
                        .as_ptr() as *const i8,
                );
            }

            (*BootStore).OsIniSection = IniSection;

            FreeLdrMigrateBootDrivePart(BootStore);
        } else if Type == NtLdr {
            IniSection = IniGetSection((*BootStore).IniCache, RUSTCONV_WSTR_3ADB4FA7D636.as_ptr());
            if IniSection.is_null() {
                IniSection = IniGetSection((*BootStore).IniCache, RUSTCONV_WSTR_20FA757B49DF.as_ptr());
                if IniSection.is_null() {
                    IniSection = IniGetSection((*BootStore).IniCache, RUSTCONV_WSTR_D0DD11036E34.as_ptr());
                }
            }
            if IniSection.is_null() {
                DPRINT1!(
                    b"OpenIniBootLoaderStore: Failed to retrieve 'boot loader' section!\n\0"
                        .as_ptr() as *const i8,
                );
            }

            (*BootStore).OptionsIniSection = IniSection;

            IniSection = IniAddSection((*BootStore).IniCache, RUSTCONV_WSTR_02648A408E10.as_ptr());
            if IniSection.is_null() {
                DPRINT1!(
                    b"OpenIniBootLoaderStore: Failed to retrieve 'operating systems' section!\n\0"
                        .as_ptr() as *const i8,
                );
            }

            (*BootStore).OsIniSection = IniSection;
        }
    }

    *Handle = BootStore as *mut core::ffi::c_void;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn ProtectFile(
    FileHandle: HANDLE,
    MaskAttributes: ULONG,
    Attributes: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileInfo: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut OldAttributes: ULONG;

    Status = NtQueryInformationFile(
        FileHandle,
        &mut IoStatusBlock,
        &mut FileInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
        FileBasicInformation,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtQueryInformationFile() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        return Status;
    }
    OldAttributes = FileInfo.FileAttributes;

    if MaskAttributes != 0 {
        FileInfo.FileAttributes = (OldAttributes & !MaskAttributes) | (*Attributes & MaskAttributes);
    } else {
        FileInfo.FileAttributes = *Attributes;
    }

    *Attributes = OldAttributes;

    Status = NtSetInformationFile(
        FileHandle,
        &mut IoStatusBlock,
        &mut FileInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
        FileBasicInformation,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtSetInformationFile() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
    }

    Status
}

unsafe extern "C" fn CloseIniBootLoaderStore(Handle: PVOID) -> NTSTATUS {
    static ProtectAttribs: ULONG =
        (FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_READONLY);

    let BootStore: PBOOT_STORE_INI_CONTEXT = Handle as PBOOT_STORE_INI_CONTEXT;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut FileAttribs: ULONG;

    'quit: {
        if BootStore.is_null() {
            RtlAssert(
                b"BootStore\0".as_ptr() as *const i8 as PVOID,
                b"base/setup/lib/utils/bldrsup.c\0"
                    .as_ptr() as *const i8 as PVOID,
                841 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }

        if (*BootStore).Header.ReadOnly != 0 {
            break 'quit;
        }

        if !(*BootStore).SectionHandle.is_null() {
            FileAttribs = 0;
            Status = ProtectFile(
                (*BootStore).FileHandle,
                ProtectAttribs,
                &mut FileAttribs,
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(
                    b"Could not unprotect INI boot store (Status 0x%08lx)\n\0".as_ptr()
                        as *const i8,
                    Status,
                );
                break 'quit;
            }
        }

        IniCacheSaveByHandle((*BootStore).IniCache, (*BootStore).FileHandle);

        FileAttribs = ProtectAttribs;
        if (*BootStore).Header.Type == FreeLdr {
            FileAttribs &= !FILE_ATTRIBUTE_READONLY;
        }
        ProtectFile((*BootStore).FileHandle, FileAttribs, &mut FileAttribs);
        Status = STATUS_SUCCESS;
    }

    IniCacheDestroy((*BootStore).IniCache);

    NtClose((*BootStore).FileHandle);

    RtlFreeHeap(ProcessHeap, 0, BootStore as PVOID);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn OpenBootStoreByHandle(
    Handle: *mut *mut core::ffi::c_void,
    PartitionDirectoryHandle: *mut core::ffi::c_void,
    Type: u32,
    OpenMode: u32,
    Access: u32,
) -> i32 {
    if Type >= BldrTypeMax || NtosBootLoaders[Type as usize].Type >= BldrTypeMax {
        return STATUS_NOT_SUPPORTED;
    }
    if (OpenMode == BS_CreateNew || OpenMode == BS_CreateAlways || OpenMode == BS_RecreateExisting)
        && (Access & BS_WriteAccess) == 0
    {
        return STATUS_ACCESS_DENIED;
    }
    if (OpenMode == BS_OpenExisting || OpenMode == BS_OpenAlways)
        && (Access & BS_ReadWriteAccess) == 0
    {
        return STATUS_ACCESS_DENIED;
    }
    return NtosBootLoaders[Type as usize].OpenBootStore.unwrap_unchecked()(
        Handle,
        PartitionDirectoryHandle,
        Type,
        OpenMode,
        Access,
    );
}
#[no_mangle]
pub unsafe extern "C" fn OpenBootStore_UStr(
    Handle: *mut *mut core::ffi::c_void,
    SystemPartitionPath: PUNICODE_STRING,
    Type: u32,
    OpenMode: u32,
    Access: u32,
) -> i32 {
    let mut Status: i32;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut PartitionDirectoryHandle: *mut core::ffi::c_void = core::mem::zeroed();

    if Type >= 2 || NtosBootLoaders[Type as usize].Type >= 2 {
        DPRINT1!(
            b"Loader type %d is currently unsupported!\n\0".as_ptr() as *const i8,
            NtosBootLoaders[Type as usize].Type,
        );
        return STATUS_NOT_SUPPORTED;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = SystemPartitionPath;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut PartitionDirectoryHandle,
        FILE_LIST_DIRECTORY | FILE_ADD_FILE | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_DIRECTORY_FILE,
    );

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"Failed to open SystemPartition '%wZ' (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            SystemPartitionPath,
            Status,
        );
        return Status;
    }

    Status = OpenBootStoreByHandle(
        Handle,
        PartitionDirectoryHandle,
        Type,
        OpenMode,
        Access,
    );

    NtClose(PartitionDirectoryHandle);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn OpenBootStore(
    // C: `PVOID* Handle` -- the opened store comes back through it. Taken as a bare
    // pointer, CreateFreeLoaderIniForReactOS passed its NULL handle by value and the
    // compiler had no way to object.
    Handle: *mut *mut core::ffi::c_void,
    SystemPartition: PCWSTR,
    Type: BOOT_STORE_TYPE,
    OpenMode: BOOT_STORE_OPENMODE,
    Access: BOOT_STORE_ACCESS,
) -> NTSTATUS {
    let mut SystemPartitionPath: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut SystemPartitionPath, SystemPartition);
    return OpenBootStore_UStr(
        Handle,
        &mut SystemPartitionPath,
        Type,
        OpenMode,
        Access,
    );
}
#[no_mangle]
pub unsafe extern "C" fn CloseBootStore(Handle: *mut core::ffi::c_void) -> i32 {
    let BootStore: *mut BOOT_STORE_CONTEXT = Handle as *mut BOOT_STORE_CONTEXT;

    if BootStore.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*BootStore).Type as u32 >= BldrTypeMax as u32
        || NtosBootLoaders[(*BootStore).Type as usize].Type as u32 >= BldrTypeMax as u32
    {
        DPRINT1!(
            b"Loader type %d is currently unsupported!\n\0".as_ptr() as *const i8,
            NtosBootLoaders[(*BootStore).Type as usize].Type
        );
        return STATUS_NOT_SUPPORTED;
    }

    return (NtosBootLoaders[(*BootStore).Type as usize].CloseBootStore.unwrap_unchecked())(Handle);
}

unsafe extern "C" fn CreateNTOSEntry(
    BootStore: PBOOT_STORE_INI_CONTEXT,
    BootEntryKey: ULONG_PTR,
    BootEntry: PBOOT_STORE_ENTRY,
) -> NTSTATUS {
    let mut IniSection: PINI_SECTION = core::ptr::null_mut();
    let Section: PCWSTR = BootEntryKey as PCWSTR;

    IniAddKey((*BootStore).OsIniSection, Section, (*BootEntry).FriendlyName);

    IniSection = IniAddSection((*BootStore).IniCache, Section);

    if (*BootEntry).OsOptionsLength as usize >= core::mem::size_of::<NTOS_OPTIONS>()
        && RtlCompareMemory(
            &raw mut (*BootEntry).OsOptions as *const core::ffi::c_void,
            NTOS_OPTIONS_SIGNATURE.as_ptr() as *const core::ffi::c_void,
            RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature) as SIZE_T,
        ) == RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature) as SIZE_T
    {
        let Options: PNTOS_OPTIONS = &raw mut (*BootEntry).OsOptions as PNTOS_OPTIONS;

        IniAddKey(IniSection, RUSTCONV_WSTR_FACD2F68A4C6.as_ptr(), RUSTCONV_WSTR_1BEF59FC85F5.as_ptr());
        IniAddKey(IniSection, RUSTCONV_WSTR_4F964A2F5D70.as_ptr(), (*Options).OsLoadPath);
        IniAddKey(IniSection, RUSTCONV_WSTR_68F952C868EF.as_ptr(), (*Options).OsLoadOptions);
    } else if (*BootEntry).OsOptionsLength as usize >= core::mem::size_of::<BOOTSECTOR_OPTIONS>()
        && RtlCompareMemory(
            &raw mut (*BootEntry).OsOptions as *const core::ffi::c_void,
            BOOTSECTOR_OPTIONS_SIGNATURE.as_ptr() as *const core::ffi::c_void,
            RTL_FIELD_SIZE!(BOOTSECTOR_OPTIONS, Signature) as SIZE_T,
        ) == RTL_FIELD_SIZE!(BOOTSECTOR_OPTIONS, Signature) as SIZE_T
    {
        let Options: PBOOTSECTOR_OPTIONS = &raw mut (*BootEntry).OsOptions as PBOOTSECTOR_OPTIONS;

        IniAddKey(IniSection, RUSTCONV_WSTR_FACD2F68A4C6.as_ptr(), RUSTCONV_WSTR_9975EA9283F5.as_ptr());
        IniAddKey(IniSection, RUSTCONV_WSTR_0EB6F1F84A4E.as_ptr(), (*Options).BootPath);
        IniAddKey(IniSection, RUSTCONV_WSTR_DE61798F28B9.as_ptr(), (*Options).FileName);
    } else {
        DPRINT1!("Unsupported BootType %lu\n", (*BootEntry).OsOptionsLength);
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn AddBootStoreEntry(
    Handle: *mut core::ffi::c_void,
    BootEntry: *mut BOOT_STORE_ENTRY,
    BootEntryKey: usize,
) -> i32 {
    let BootStore: *mut BOOT_STORE_CONTEXT = Handle as *mut BOOT_STORE_CONTEXT;

    if BootStore.is_null() || BootEntry.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*BootStore).Type == FreeLdr {
        if (*BootEntry).Version != FreeLdr {
            return STATUS_INVALID_PARAMETER;
        }

        return CreateNTOSEntry(
            BootStore as *mut BOOT_STORE_INI_CONTEXT,
            (BootEntryKey) as ULONG_PTR,
            BootEntry,
        );
    } else if (*BootStore).Type == NtLdr {
        let Options: *mut NTOS_OPTIONS = &raw mut (*BootEntry).OsOptions as *mut NTOS_OPTIONS;
        let mut Buffer: *mut u16 = core::ptr::null_mut();
        let mut BufferLength: u32 = 0;
        let mut InstallName: *const u16 = core::ptr::null();
        let mut OsOptions: *const u16 = core::ptr::null();
        let mut IsNameNotQuoted: u8 = 0;

        if (*BootEntry).Version != NtLdr {
            return STATUS_INVALID_PARAMETER;
        }

        if (*BootEntry).OsOptionsLength < core::mem::size_of::<NTOS_OPTIONS>() as u32
            || RtlCompareMemory(
                &raw mut (*BootEntry).OsOptions as *const core::ffi::c_void,
                NTOS_OPTIONS_SIGNATURE.as_ptr() as *const core::ffi::c_void,
                RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature) as SIZE_T,
            ) != RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature) as SIZE_T
        {
            DPRINT1!("Unsupported BootType %lu\n", (*BootEntry).OsOptionsLength);
            return STATUS_SUCCESS;
        }

        InstallName = (*BootEntry).FriendlyName;
        OsOptions = (*Options).OsLoadOptions;

        IsNameNotQuoted = (*InstallName.offset(0 as isize) != 0x0022u16
            || *InstallName.offset(wcslen(InstallName).wrapping_sub(1) as isize)
                != 0x0022u16) as u8;

        BufferLength = (if IsNameNotQuoted != 0 { 2u32 } else { 0u32 })
            .wrapping_add(wcslen(InstallName) as u32);
        if !OsOptions.is_null() {
            BufferLength = BufferLength
                .wrapping_add(1)
                .wrapping_add((wcslen(OsOptions)) as _);
        }
        BufferLength = BufferLength.wrapping_add(1);

        Buffer = RtlAllocateHeap(
            ProcessHeap,
            HEAP_ZERO_MEMORY,
            ((BufferLength as usize).wrapping_mul(core::mem::size_of::<u16>())) as SIZE_T,
        ) as *mut u16;
        if Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        *Buffer = UNICODE_NULL;
        if IsNameNotQuoted != 0 {
            RtlStringCchCatW(Buffer, (BufferLength) as _, RUSTCONV_WSTR_1B9334FEECE6.as_ptr());
        }
        RtlStringCchCatW(Buffer, (BufferLength) as _, InstallName);
        if IsNameNotQuoted != 0 {
            RtlStringCchCatW(Buffer, (BufferLength) as _, RUSTCONV_WSTR_1B9334FEECE6.as_ptr());
        }
        if !OsOptions.is_null() {
            RtlStringCchCatW(Buffer, (BufferLength) as _, RUSTCONV_WSTR_8D71B3FAAB82.as_ptr());
            RtlStringCchCatW(Buffer, (BufferLength) as _, OsOptions);
        }

        IniAddKey(
            (*(BootStore as *mut BOOT_STORE_INI_CONTEXT)).OsIniSection,
            (*Options).OsLoadPath,
            Buffer,
        );

        RtlFreeHeap(ProcessHeap, 0, (Buffer).cast());
        return STATUS_SUCCESS;
    } else {
        DPRINT1!("Loader type %d is currently unsupported!\n", (*BootStore).Type);
        return STATUS_NOT_SUPPORTED;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DeleteBootStoreEntry(
    Handle: *mut core::ffi::c_void,
    BootEntryKey: usize,
) -> i32 {
    let BootStore: *mut BOOT_STORE_CONTEXT = Handle as *mut BOOT_STORE_CONTEXT;

    if BootStore.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*BootStore).Type != FreeLdr {
        DPRINT1!(
            "Loader type %d is currently unsupported!\n",
            NtosBootLoaders[(*BootStore).Type as usize].Type
        );
        return STATUS_NOT_SUPPORTED;
    }

    UNIMPLEMENTED!();
    return STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn ModifyBootStoreEntry(
    Handle: *mut core::ffi::c_void,
    BootEntry: *mut BOOT_STORE_ENTRY,
) -> NTSTATUS {
    let BootStore: *mut BOOT_STORE_CONTEXT = Handle as *mut BOOT_STORE_CONTEXT;

    if BootStore.is_null() || BootEntry.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*BootStore).Type != FreeLdr {
        DPRINT1!(
            b"Loader type %d is currently unsupported!\n\0".as_ptr() as *const i8,
            NtosBootLoaders[(*BootStore).Type as usize].Type,
        );
        return STATUS_NOT_SUPPORTED;
    }

    UNIMPLEMENTED!();
    return STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn QueryBootStoreEntry(
    Handle: *mut core::ffi::c_void,
    BootEntryKey: usize,
    BootEntry: *mut BOOT_STORE_ENTRY,
) -> i32 {
    let BootStore: *mut BOOT_STORE_CONTEXT = Handle as *mut BOOT_STORE_CONTEXT;

    if BootStore.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*BootStore).Type != FreeLdr {
        DPRINT1!(
            b"Loader type %d is currently unsupported!\n\0".as_ptr() as *const i8,
            NtosBootLoaders[(*BootStore).Type as usize].Type
        );
        return STATUS_NOT_SUPPORTED;
    }

    UNIMPLEMENTED!();
    return STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn QueryBootStoreOptions(
    Handle: *mut core::ffi::c_void,
    BootOptions: *mut BOOT_STORE_OPTIONS,
) -> NTSTATUS {
    let BootStore: *mut BOOT_STORE_CONTEXT = Handle as *mut BOOT_STORE_CONTEXT;
    let mut TimeoutStr: PCWSTR = core::ptr::null();

    if BootStore.is_null() || BootOptions.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*BootStore).Type != FreeLdr && (*BootStore).Type != NtLdr {
        DPRINT1!(
            b"Loader type %d is currently unsupported!\n\0".as_ptr() as *const i8,
            (*BootStore).Type,
        );
        return STATUS_NOT_SUPPORTED;
    }

    (*BootOptions).Timeout = 0;
    (*BootOptions).CurrentBootEntryKey = 0;
    (*BootOptions).NextBootEntryKey = 0;

    if !IniGetKey(
        (*(BootStore as *mut BOOT_STORE_INI_CONTEXT)).OptionsIniSection,
        BootOptionNames[(*BootStore).Type as usize][BO_TimeOut as usize],
        &mut TimeoutStr,
    )
    .is_null()
        && !TimeoutStr.is_null()
    {
        (*BootOptions).Timeout = _wtoi(TimeoutStr) as ULONG;
    }

    IniGetKey(
        (*(BootStore as *mut BOOT_STORE_INI_CONTEXT)).OptionsIniSection,
        BootOptionNames[(*BootStore).Type as usize][BO_DefaultOS as usize],
        core::ptr::addr_of_mut!((*BootOptions).NextBootEntryKey) as *mut PCWSTR,
    );

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn SetBootStoreOptions(
    Handle: *mut core::ffi::c_void,
    BootOptions: *mut BOOT_STORE_OPTIONS,
    FieldsToChange: u32,
) -> i32 {
    let BootStore: *mut BOOT_STORE_CONTEXT = Handle as *mut BOOT_STORE_CONTEXT;

    if BootStore.is_null() || BootOptions.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*BootStore).Type != FreeLdr && (*BootStore).Type != NtLdr {
        DPRINT1!(
            b"Loader type %d is currently unsupported!\n\0".as_ptr() as *const i8,
            NtosBootLoaders[(*BootStore).Type as usize].Type,
        );
        return STATUS_NOT_SUPPORTED;
    }

    if FieldsToChange & BOOT_OPTIONS_TIMEOUT != 0 {
        let mut TimeoutStr: [u16; 15] = [0; 15];
        RtlStringCchPrintfW!(
            TimeoutStr.as_mut_ptr(),
            ARRAYSIZE!(TimeoutStr),
            RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
            (*BootOptions).Timeout,
        );
        IniAddKey(
            (*(BootStore as *mut BOOT_STORE_INI_CONTEXT)).OptionsIniSection,
            BootOptionNames[(*BootStore).Type as usize][BO_TimeOut as usize],
            TimeoutStr.as_ptr(),
        );
    }
    if FieldsToChange & BOOT_OPTIONS_NEXT_BOOTENTRY_KEY != 0 {
        IniAddKey(
            (*(BootStore as *mut BOOT_STORE_INI_CONTEXT)).OptionsIniSection,
            BootOptionNames[(*BootStore).Type as usize][BO_DefaultOS as usize],
            (*BootOptions).NextBootEntryKey as *const u16,
        );
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn FreeLdrEnumerateBootEntries(
    BootStore: PBOOT_STORE_INI_CONTEXT,
    EnumBootEntriesRoutine: PENUM_BOOT_ENTRIES_ROUTINE,
    Parameter: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Iterator: PINICACHEITERATOR;
    let mut OsIniSection: PINI_SECTION;
    let mut SectionName: PCWSTR = core::ptr::null();
    let mut KeyData: PCWSTR = core::ptr::null();
    let mut xxBootEntry: [UCHAR; (FIELD_OFFSET!(BOOT_STORE_ENTRY, OsOptions) +
        (if core::mem::size_of::<NTOS_OPTIONS>() > core::mem::size_of::<BOOTSECTOR_OPTIONS>() { core::mem::size_of::<NTOS_OPTIONS>() } else { core::mem::size_of::<BOOTSECTOR_OPTIONS>() }) as i32) as usize] = [0; (FIELD_OFFSET!(BOOT_STORE_ENTRY, OsOptions) +
        (if core::mem::size_of::<NTOS_OPTIONS>() > core::mem::size_of::<BOOTSECTOR_OPTIONS>() { core::mem::size_of::<NTOS_OPTIONS>() } else { core::mem::size_of::<BOOTSECTOR_OPTIONS>() }) as i32) as usize];
    let BootEntry: PBOOT_STORE_ENTRY = &mut xxBootEntry as *mut UCHAR as PBOOT_STORE_ENTRY;
    let mut Buffer: PWCHAR;

    Iterator = IniFindFirstValue((*BootStore).OsIniSection, &mut SectionName, &mut KeyData);
    if Iterator.is_null() {
        return STATUS_SUCCESS;
    }

    loop {
        let mut InstallName: PCWSTR;
        let mut InstallNameLength: ULONG;

        if *KeyData == 0x0022u16 {
            let End: PWCHAR = wcschr(KeyData.offset(1 as isize), 0x0022u16);
            if !End.is_null() {
                InstallName = KeyData.offset(1 as isize);
                InstallNameLength = End.offset_from(InstallName) as ULONG;
            } else {
                InstallName = KeyData;
                InstallNameLength = (wcslen(InstallName)) as _;
            }
            if InstallNameLength == 0 {
                InstallName = core::ptr::null();
            }
        } else {
            InstallName = KeyData;
            InstallNameLength = (wcslen(InstallName)) as _;
            if InstallNameLength == 0 {
                InstallName = core::ptr::null();
            }
        }

        Buffer = core::ptr::null_mut();
        if InstallNameLength != 0 {
            Buffer = RtlAllocateHeap(ProcessHeap, HEAP_ZERO_MEMORY, ((InstallNameLength + 1) * (core::mem::size_of::<u16>()) as ULONG) as SIZE_T) as PWCHAR;
        }
        if !Buffer.is_null() {
            RtlCopyMemory!(Buffer as *mut _, InstallName as *const _, (InstallNameLength as SIZE_T).wrapping_mul(core::mem::size_of::<u16>() as SIZE_T));
            *Buffer.offset(InstallNameLength as isize) = UNICODE_NULL;
            InstallName = Buffer;
        }

        DPRINT!("Boot entry '%S' in OS section '%S'\n", InstallName, SectionName);

        (*BootEntry).Version = FreeLdr;
        (*BootEntry).BootEntryKey = MAKESTRKEY!(SectionName);
        (*BootEntry).FriendlyName = InstallName;
        (*BootEntry).BootFilePath = core::ptr::null();
        (*BootEntry).OsOptionsLength = 0;

        OsIniSection = IniGetSection((*BootStore).IniCache, SectionName);
        if !OsIniSection.is_null() {
            if IniGetKey(OsIniSection, RUSTCONV_WSTR_FACD2F68A4C6.as_ptr(), &mut KeyData).is_null() || KeyData.is_null() {
                DPRINT1!("No BootType value present\n");
            } else if _wcsicmp(KeyData, RUSTCONV_WSTR_1BEF59FC85F5.as_ptr()) == 0 ||
                      _wcsicmp(KeyData, RUSTCONV_WSTR_E99788FBE09D.as_ptr()) == 0 {
                let Options: PNTOS_OPTIONS = &mut (*BootEntry).OsOptions as *mut _ as PNTOS_OPTIONS;
                DPRINT!("This is a '%S' boot entry\n", KeyData);
                (*BootEntry).OsOptionsLength = core::mem::size_of::<NTOS_OPTIONS>() as ULONG;
                RtlCopyMemory!((*Options).Signature.as_ptr() as *mut u8, NTOS_OPTIONS_SIGNATURE.as_ptr() as *const u8, RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature));
                (*Options).OsLoadPath = core::ptr::null();
                if !IniGetKey(OsIniSection, RUSTCONV_WSTR_4F964A2F5D70.as_ptr(), &mut KeyData).is_null() {
                    (*Options).OsLoadPath = KeyData;
                }
                (*Options).OsLoadOptions = core::ptr::null();
                if !IniGetKey(OsIniSection, RUSTCONV_WSTR_68F952C868EF.as_ptr(), &mut KeyData).is_null() {
                    (*Options).OsLoadOptions = KeyData;
                }
            } else if _wcsicmp(KeyData, RUSTCONV_WSTR_9975EA9283F5.as_ptr()) == 0 ||
                      _wcsicmp(KeyData, RUSTCONV_WSTR_2D4564AAFD0B.as_ptr()) == 0 {
                let Options: PBOOTSECTOR_OPTIONS = &mut (*BootEntry).OsOptions as *mut _ as PBOOTSECTOR_OPTIONS;
                DPRINT!("This is a '%S' boot entry\n", KeyData);
                (*BootEntry).OsOptionsLength = core::mem::size_of::<BOOTSECTOR_OPTIONS>() as ULONG;
                RtlCopyMemory!((*Options).Signature.as_ptr() as *mut u8, BOOTSECTOR_OPTIONS_SIGNATURE.as_ptr() as *const u8, RTL_FIELD_SIZE!(BOOTSECTOR_OPTIONS, Signature));
                (*Options).BootPath = core::ptr::null();
                if !IniGetKey(OsIniSection, RUSTCONV_WSTR_0EB6F1F84A4E.as_ptr(), &mut KeyData).is_null() {
                    (*Options).BootPath = KeyData;
                }
                (*Options).FileName = core::ptr::null();
                if !IniGetKey(OsIniSection, RUSTCONV_WSTR_DE61798F28B9.as_ptr(), &mut KeyData).is_null() {
                    (*Options).FileName = KeyData;
                }
            } else {
                DPRINT1!("Unrecognized BootType value '%S'\n", KeyData);
            }
        }

        Status = (EnumBootEntriesRoutine.unwrap_unchecked())(FreeLdr, BootEntry, Parameter);

        if !Buffer.is_null() {
            RtlFreeHeap(ProcessHeap, 0, (Buffer).cast());
        }

        if ((NT_SUCCESS!(Status)) == 0) {
            break;
        }

        if ((IniFindNextValue(Iterator, &mut SectionName, &mut KeyData)) == 0) {
            break;
        }
    }

    IniFindClose(Iterator);
    Status
}

unsafe extern "C" fn NtLdrEnumerateBootEntries(
    mut BootStore: PBOOT_STORE_INI_CONTEXT,
    mut EnumBootEntriesRoutine: PENUM_BOOT_ENTRIES_ROUTINE,
    mut Parameter: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Iterator: PINICACHEITERATOR;
    let mut SectionName: PCWSTR = core::ptr::null();
    let mut KeyData: PCWSTR = core::ptr::null();
    let mut xxBootEntry: [UCHAR; (FIELD_OFFSET!(BOOT_STORE_ENTRY, OsOptions) + (core::mem::size_of::<NTOS_OPTIONS>()) as i32) as usize] = [0; (FIELD_OFFSET!(BOOT_STORE_ENTRY, OsOptions) + (core::mem::size_of::<NTOS_OPTIONS>()) as i32) as usize];
    let mut BootEntry: PBOOT_STORE_ENTRY = &raw mut xxBootEntry as PBOOT_STORE_ENTRY;
    let mut Options: PNTOS_OPTIONS = &raw mut (*BootEntry).OsOptions as PNTOS_OPTIONS;
    let mut Buffer: PWCHAR;
    let mut BufferLength: ULONG;

    Iterator = IniFindFirstValue((*BootStore).OsIniSection, &raw mut SectionName, &raw mut KeyData);
    if Iterator.is_null() {
        return STATUS_SUCCESS;
    }
    loop {
        let mut InstallName: PCWSTR;
        let mut OsOptions: PCWSTR;
        let mut InstallNameLength: ULONG;
        let mut OsOptionsLength: ULONG;

        if *KeyData as u16 == 0x0022u16 {
            OsOptions = wcschr(KeyData.offset(1 as isize), 0x0022u16);
            if !OsOptions.is_null() {
                InstallName = KeyData.offset(1 as isize);
                InstallNameLength = OsOptions.offset_from(InstallName) as ULONG;
                if InstallNameLength == 0 {
                    InstallName = core::ptr::null();
                }
                OsOptions = OsOptions.offset(1);
                while iswspace(*OsOptions) != 0 {
                    OsOptions = OsOptions.offset(1);
                }
                OsOptionsLength = (wcslen(OsOptions)) as _;
                if OsOptionsLength == 0 {
                    OsOptions = core::ptr::null();
                }
            } else {
                InstallName = KeyData;
                InstallNameLength = (wcslen(InstallName)) as _;
                if InstallNameLength == 0 {
                    InstallName = core::ptr::null();
                }
                OsOptionsLength = 0;
            }
        } else {
            InstallName = KeyData;
            InstallNameLength = (wcslen(InstallName)) as _;
            if InstallNameLength == 0 {
                InstallName = core::ptr::null();
            }
            OsOptions = core::ptr::null();
            OsOptionsLength = 0;
        }

        Buffer = core::ptr::null_mut();
        BufferLength = (InstallNameLength.wrapping_add(OsOptionsLength)).wrapping_mul(core::mem::size_of::<u16>() as ULONG);
        if BufferLength != 0 {
            Buffer = RtlAllocateHeap(ProcessHeap, HEAP_ZERO_MEMORY, (BufferLength as SIZE_T).wrapping_add((2 as SIZE_T).wrapping_mul(core::mem::size_of_val(&UNICODE_NULL) as SIZE_T))) as PWCHAR;
        }
        if !Buffer.is_null() {
            let mut ptr: PWCHAR;
            if !InstallName.is_null() && InstallNameLength != 0 {
                ptr = Buffer;
                RtlCopyMemory!(ptr as *mut core::ffi::c_void, InstallName as *const core::ffi::c_void, (InstallNameLength as usize).wrapping_mul(core::mem::size_of::<u16>()));
                *ptr.offset(InstallNameLength as isize) = UNICODE_NULL;
                InstallName = ptr;
            }
            if !OsOptions.is_null() && OsOptionsLength != 0 {
                ptr = Buffer.offset(InstallNameLength as isize).offset(1 as isize);
                RtlCopyMemory!(ptr as *mut core::ffi::c_void, OsOptions as *const core::ffi::c_void, (OsOptionsLength as usize).wrapping_mul(core::mem::size_of::<u16>()));
                *ptr.offset(OsOptionsLength as isize) = UNICODE_NULL;
                OsOptions = ptr;
            }
        }

        DPRINT1!("Boot entry '%S' in OS section (path) '%S'\n", InstallName, SectionName);

        (*BootEntry).Version = NtLdr;
        (*BootEntry).BootEntryKey = 0;
        (*BootEntry).FriendlyName = InstallName;
        (*BootEntry).BootFilePath = core::ptr::null();

        (*BootEntry).OsOptionsLength = core::mem::size_of::<NTOS_OPTIONS>() as ULONG;
        RtlCopyMemory!(
            &raw mut (*Options).Signature as *mut core::ffi::c_void,
            (NTOS_OPTIONS_SIGNATURE).as_ptr() as *const core::ffi::c_void,
            RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature),
        );

        (*Options).OsLoadPath = SectionName;
        (*Options).OsLoadOptions = OsOptions;

        Status = (EnumBootEntriesRoutine.unwrap_unchecked())(NtLdr, BootEntry, Parameter);

        if !Buffer.is_null() {
            RtlFreeHeap(ProcessHeap, 0, (Buffer).cast());
        }

        if ((NT_SUCCESS!(Status)) == 0) {
            break;
        }
        if ((IniFindNextValue(Iterator, &raw mut SectionName, &raw mut KeyData)) == 0) {
            break;
        }
    }

    IniFindClose(Iterator);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn EnumerateBootStoreEntries(
    Handle: *mut core::ffi::c_void,
    EnumBootEntriesRoutine: PENUM_BOOT_ENTRIES_ROUTINE,
    Parameter: *mut core::ffi::c_void,
) -> NTSTATUS {
    let BootStore: PBOOT_STORE_CONTEXT = Handle as PBOOT_STORE_CONTEXT;

    if BootStore.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*BootStore).Type as u32 >= BldrTypeMax as u32
        || NtosBootLoaders[(*BootStore).Type as usize].Type as u32 >= BldrTypeMax as u32
    {
        DPRINT1!(
            b"Loader type %d is currently unsupported!\n\0".as_ptr() as *const i8,
            NtosBootLoaders[(*BootStore).Type as usize].Type,
        );
        return STATUS_SUCCESS;
    }

    return NtosBootLoaders[(*BootStore).Type as usize]
        .EnumBootStoreEntries
        .unwrap_unchecked()(
        BootStore as PBOOT_STORE_INI_CONTEXT as *mut core::ffi::c_void,
        EnumBootEntriesRoutine,
        Parameter,
    );
}
