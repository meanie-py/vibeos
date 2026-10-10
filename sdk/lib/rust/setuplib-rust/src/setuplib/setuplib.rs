use crate::setuplib::mui::AddCodePage;
use crate::setuplib::mui::AddKeyboardLayouts;
use crate::setuplib::filesup::CombinePaths;
use crate::setuplib::partlist::CreatePartition;
use crate::setuplib::genlist::DestroyGenericList;
use crate::setuplib::mui::DoRegistryFontFixup;
use crate::setuplib::partlist::FindSupportedSystemPartition;
use crate::setuplib::fsutil::FormatPartition;
use crate::setuplib::genlist::GetCurrentListEntry;
use crate::setuplib::genlist::GetListEntryData;
use crate::setuplib::filesup::GetNtDevicePathOfDriveNumber;
use crate::setuplib::infsupp::INF_GetData;
use crate::setuplib::infsupp::INF_GetDataField;
use crate::setuplib::registry::ImportRegistryFile;
use crate::setuplib::inicache::IniAddKey;
use crate::setuplib::inicache::IniAddSection;
use crate::setuplib::inicache::IniCacheCreate;
use crate::setuplib::inicache::IniCacheDestroy;
use crate::setuplib::inicache::IniCacheSave;
use crate::setuplib::mui::MUIGetGeoID;
use crate::setuplib::filesup::OpenAndMapFile;
use crate::setuplib::settings::ProcessDisplayRegistry;
use crate::setuplib::settings::ProcessKeyboardLayoutRegistry;
use crate::setuplib::settings::ProcessLocaleRegistry;
use crate::setuplib::registry::RegCleanupRegistry;
use crate::setuplib::registry::RegInitializeRegistry;
use crate::setuplib::partlist::SetActivePartition;
use crate::setuplib::settings::SetDefaultPagefile;
use crate::setuplib::settings::SetGeoID;
use crate::setuplib::partlist::SetMountedDeviceValues;
use crate::setuplib::registry::VerifyRegistryHives;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_00FE7BCF4C97: [u16; 21] = [
    0x0046u16, 0x0072, 0x0065, 0x0065, 0x0053, 0x0079, 0x0073, 0x0050, 0x0061, 0x0072,
    0x0074, 0x0044, 0x0069, 0x0073, 0x006B, 0x0053, 0x0070, 0x0061, 0x0063, 0x0065,
    0x0000,
];
pub static RUSTCONV_WSTR_05F3C233D307: [u16; 5] = [0x0044u16, 0x0061, 0x0074, 0x0061, 0x0000];
pub static RUSTCONV_WSTR_0F0B1FBB5B3E: [u16; 11] = [
    0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0050, 0x0061, 0x0074, 0x0068,
    0x0000,
];
pub static RUSTCONV_WSTR_181895D51D74: [u16; 6] = [0x0025u16, 0x0030, 0x0038, 0x006C, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_1CA9A6B1F115: [u16; 7] = [0x0041u16, 0x0064, 0x0064, 0x0052, 0x0065, 0x0067, 0x0000];
pub static RUSTCONV_WSTR_1F7A38F3D6FA: [u16; 8] = [0x0056u16, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_28FB1027CC87: [u16; 41] = [
    0x006Du16, 0x0075, 0x006C, 0x0074, 0x0069, 0x0028, 0x0030, 0x0029, 0x0064, 0x0069,
    0x0073, 0x006B, 0x0028, 0x0030, 0x0029, 0x0072, 0x0064, 0x0069, 0x0073, 0x006B,
    0x0028, 0x0025, 0x006C, 0x0075, 0x0029, 0x0070, 0x0061, 0x0072, 0x0074, 0x0069,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0028, 0x0025, 0x006C, 0x0075, 0x0029, 0x005C,
    0x0000,
];
pub static RUSTCONV_WSTR_2AD388D1B224: [u16; 41] = [
    0x0073u16, 0x0063, 0x0073, 0x0069, 0x0028, 0x0025, 0x0075, 0x0029, 0x0064, 0x0069,
    0x0073, 0x006B, 0x0028, 0x0025, 0x0075, 0x0029, 0x0072, 0x0064, 0x0069, 0x0073,
    0x006B, 0x0028, 0x0025, 0x0075, 0x0029, 0x0070, 0x0061, 0x0072, 0x0074, 0x0069,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0028, 0x0025, 0x006C, 0x0075, 0x0029, 0x005C,
    0x0000,
];
pub static RUSTCONV_WSTR_2AFA0E90CE75: [u16; 21] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C, 0x0024,
    0x0077, 0x0069, 0x006E, 0x006E, 0x0074, 0x0024, 0x002E, 0x0069, 0x006E, 0x0066,
    0x0000,
];
pub static RUSTCONV_WSTR_3756A572850B: [u16; 15] = [
    0x0048u16, 0x0069, 0x0076, 0x0065, 0x0049, 0x006E, 0x0066, 0x0073, 0x002E, 0x0046,
    0x0072, 0x0065, 0x0073, 0x0068, 0x0000,
];
pub static RUSTCONV_WSTR_3EC29A2723BE: [u16; 10] = [0x0024u16, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0024, 0x0000];
pub static RUSTCONV_WSTR_3F80C3C80BCE: [u16; 13] = [
    0x0075u16, 0x006E, 0x0061, 0x0074, 0x0074, 0x0065, 0x006E, 0x0064, 0x002E, 0x0069,
    0x006E, 0x0066, 0x0000,
];
pub static RUSTCONV_WSTR_43552FB35D21: [u16; 9] = [0x0055u16, 0x006E, 0x0061, 0x0074, 0x0074, 0x0065, 0x006E, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_45F0FA7E4B83: [u16; 20] = [
    0x0055u16, 0x006E, 0x0061, 0x0074, 0x0074, 0x0065, 0x006E, 0x0064, 0x0046, 0x006F,
    0x0072, 0x0044, 0x0065, 0x0074, 0x0061, 0x0063, 0x0068, 0x0065, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_4B9A840DB974: [u16; 10] = [0x0053u16, 0x0065, 0x0074, 0x0075, 0x0070, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000];
pub static RUSTCONV_WSTR_4C853821A99F: [u16; 17] = [
    0x0048u16, 0x0069, 0x0076, 0x0065, 0x0049, 0x006E, 0x0066, 0x0073, 0x002E, 0x0049,
    0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_5D19A68CF474: [u16; 21] = [
    0x0055u16, 0x006E, 0x0061, 0x0074, 0x0074, 0x0065, 0x006E, 0x0064, 0x0053, 0x0065,
    0x0074, 0x0075, 0x0070, 0x0045, 0x006E, 0x0061, 0x0062, 0x006C, 0x0065, 0x0064,
    0x0000,
];
pub static RUSTCONV_WSTR_62CF23C278F1: [u16; 10] = [0x0053u16, 0x0069, 0x0067, 0x006E, 0x0061, 0x0074, 0x0075, 0x0072, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_6900BCBBE1A9: [u16; 12] = [
    0x0050u16, 0x0072, 0x006F, 0x0064, 0x0075, 0x0063, 0x0074, 0x0054, 0x0079, 0x0070,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_6A0CE9AC469F: [u16; 22] = [
    0x0044u16, 0x0065, 0x0073, 0x0074, 0x0069, 0x006E, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0044, 0x0069, 0x0073, 0x006B, 0x004E, 0x0075, 0x006D, 0x0062, 0x0065,
    0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_6D1618CC9381: [u16; 4] = [0x0079u16, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_6DDAE518D89A: [u16; 27] = [
    0x0044u16, 0x0065, 0x0073, 0x0074, 0x0069, 0x006E, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E,
    0x004E, 0x0075, 0x006D, 0x0062, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_81E16440571D: [u16; 6] = [0x0022u16, 0x0025, 0x0073, 0x005C, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_833AD7EF6992: [u16; 3] = [0x006Eu16, 0x006F, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9896841D0A6A: [u16; 5] = [0x0022u16, 0x0025, 0x0073, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_A358635A5742: [u16; 22] = [
    0x0044u16, 0x0069, 0x0073, 0x006B, 0x0053, 0x0070, 0x0061, 0x0063, 0x0065, 0x0052,
    0x0065, 0x0071, 0x0075, 0x0069, 0x0072, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074,
    0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_B472F3142B6E: [u16; 19] = [
    0x0042u16, 0x006F, 0x006F, 0x0074, 0x004C, 0x006F, 0x0061, 0x0064, 0x0065, 0x0072,
    0x004C, 0x006F, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_B57D8EC57C10: [u16; 14] = [
    0x0041u16, 0x0075, 0x0074, 0x006F, 0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074,
    0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_B756C84E9F61: [u16; 7] = [0x0046u16, 0x0073, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_BCF3913447D5: [u16; 4] = [0x0025u16, 0x0073, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_C3BF39D970E8: [u16; 16] = [
    0x0053u16, 0x0065, 0x0074, 0x0075, 0x0070, 0x0053, 0x006F, 0x0075, 0x0072, 0x0063,
    0x0065, 0x0050, 0x0061, 0x0074, 0x0068, 0x0000,
];
pub static RUSTCONV_WSTR_CED1B9EC271C: [u16; 9] = [0x004Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0065, 0x0049, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_CF78061217B0: [u16; 18] = [
    0x0053u16, 0x0065, 0x0074, 0x0075, 0x0070, 0x0053, 0x006F, 0x0075, 0x0072, 0x0063,
    0x0065, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_D50DAFA02F06: [u16; 12] = [
    0x005Cu16, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F,
    0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_D5E4D86E3074: [u16; 17] = [
    0x0048u16, 0x0069, 0x0076, 0x0065, 0x0049, 0x006E, 0x0066, 0x0073, 0x002E, 0x0055,
    0x0070, 0x0067, 0x0072, 0x0061, 0x0064, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_D92823782CE4: [u16; 18] = [
    0x0055u16, 0x006E, 0x0061, 0x0074, 0x0074, 0x0065, 0x006E, 0x0064, 0x0065, 0x0064,
    0x0049, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_DF8C4EF9F756: [u16; 16] = [
    0x0046u16, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0050, 0x0061, 0x0072, 0x0074,
    0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_E4E924642399: [u16; 13] = [
    0x0074u16, 0x0078, 0x0074, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x002E, 0x0073,
    0x0069, 0x0066, 0x0000,
];
pub static RUSTCONV_WSTR_E71C047C4F02: [u16; 6] = [0x0077u16, 0x0069, 0x006E, 0x006E, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_ECD9281DF6C3: [u16; 13] = [
    0x0024u16, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E,
    0x0054, 0x0024, 0x0000,
];
pub static RUSTCONV_WSTR_EF6224203743: [u16; 7] = [0x0044u16, 0x0065, 0x006C, 0x0052, 0x0065, 0x0067, 0x0000];
pub static RUSTCONV_WSTR_F024DF41CF6D: [u16; 22] = [
    0x0049u16, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0061, 0x0074, 0x0069,
    0x006F, 0x006E, 0x0044, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072,
    0x0079, 0x0000,
];
pub static RUSTCONV_WSTR_F2981BFF2146: [u16; 12] = [
    0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0050, 0x0061, 0x0074,
    0x0068, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_3EC29A2723BE: [u16; 10] = [0x0024u16, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0024, 0x0000];

#[no_mangle]
pub static mut ProcessHeap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub static mut IsUnattendedSetup: BOOLEAN = FALSE as BOOLEAN;

static mut InfUnattendSignature: PCWSTR = unsafe {
    RUSTCONV_C2R_WSTR_3EC29A2723BE.as_ptr() as PCWSTR
};

unsafe extern "C" fn CompareInfFirstLineString(
    mut hInf: HINF,
    mut Section: PCWSTR,
    mut Name: PCWSTR,
    mut String: PCWSTR,
) -> ::core::ffi::c_int {
    let mut Context: INFCONTEXT = INFCONTEXT {
        Inf: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        CurrentInf: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        Section: 0,
        Line: 0,
    };
    let mut Value: PCWSTR = ::core::ptr::null::<u16>();
    let mut Result: ::core::ffi::c_int = STATUS_UNSUCCESSFUL;
    if SpInfFindFirstLine!(hInf, Section, Name, &raw mut Context) == 0 {
        return Result;
    }
    if INF_GetData(
        &raw mut Context,
        ::core::ptr::null_mut::<PCWSTR>(),
        &raw mut Value,
    ) == 0
    {
        return Result;
    }
    Result = _wcsicmp(Value as *const u16, String as *const u16);
    INF_FreeData(Value);
    return Result;
}

unsafe extern "C" fn IsInfUnattendSetupEnabled(
    mut pSetupData: PUSETUP_DATA,
    mut InfPath: PCWSTR,
    mut CheckDetached: BOOLEAN,
) -> BOOLEAN {
    let mut hInf: HINF;
    let mut ErrorLine: UINT = core::mem::zeroed();
    let mut Result: BOOLEAN = (FALSE) as u8;

    hInf = SpInfOpenInfFile!(
        InfPath,
        core::ptr::null_mut(),
        INF_STYLE_OLDNT,
        ((*pSetupData).LanguageId) as _,
        &mut ErrorLine,
    );
    if hInf == INVALID_HANDLE_VALUE {
        return Result;
    }

    'quit: {
        if CompareInfFirstLineString(
            hInf,
            RUSTCONV_WSTR_43552FB35D21.as_ptr(),
            RUSTCONV_WSTR_62CF23C278F1.as_ptr(),
            InfUnattendSignature,
        ) != 0
        {
            DPRINT!(
                b"Signature not %ls\n\0".as_ptr() as *const i8,
                InfUnattendSignature,
            );
            break 'quit;
        }

        if CompareInfFirstLineString(
            hInf,
            RUSTCONV_WSTR_43552FB35D21.as_ptr(),
            RUSTCONV_WSTR_5D19A68CF474.as_ptr(),
            RUSTCONV_WSTR_6D1618CC9381.as_ptr(),
        ) != 0
        {
            break 'quit;
        }

        if CheckDetached != 0
            && CompareInfFirstLineString(
                hInf,
                RUSTCONV_WSTR_43552FB35D21.as_ptr(),
                RUSTCONV_WSTR_45F0FA7E4B83.as_ptr(),
                RUSTCONV_WSTR_6D1618CC9381.as_ptr(),
            ) != 0
        {
            break 'quit;
        }

        Result = (TRUE) as u8;
    }

    SpInfCloseInfFile!(hInf);
    return Result;
}

unsafe extern "C" fn GetUnattendInfPath(
    mut pSetupData: PUSETUP_DATA,
    mut InfPath: PWSTR,
    mut cch: SIZE_T,
) -> BOOLEAN {
    static mut UnattendInfPath: [u16; (MAX_PATH) as _] = [UNICODE_NULL; (MAX_PATH) as _];
    static mut UnattendInfLookedUp: BOOLEAN = (FALSE) as u8;
    let mut Status: NTSTATUS;
    let mut Types: USHORT;

    if UnattendInfLookedUp != 0 {
        // goto Quit
    } else {
        UnattendInfLookedUp = (TRUE) as u8;

        Status = CombinePaths(
            UnattendInfPath.as_mut_ptr(),
            (_countof!(UnattendInfPath)) as _,
            2,
            (*pSetupData).SourcePath.Buffer,
            RUSTCONV_WSTR_3F80C3C80BCE.as_ptr(),
        );
        if NT_SUCCESS!(Status) != 0
            && DoesFileExist!(core::ptr::null_mut(), UnattendInfPath.as_mut_ptr()) != 0
            && ((IsInfUnattendSetupEnabled(pSetupData, UnattendInfPath.as_mut_ptr(), (FALSE) as BOOLEAN)) != 0)
        {
            return NT_SUCCESS!(RtlStringCchCopyW(InfPath, (cch) as _, UnattendInfPath.as_mut_ptr()));
        }

        Types = 0;
        while Types <= 1 {
            let mut Drive: CHAR = 'A' as i8;
            while Drive <= 'Z' as i8 {
                let mut DriveType: UCHAR;
                let mut szInf: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
                let mut NtDrive: _UNICODE_STRING = _UNICODE_STRING {
                    Length: 0,
                    MaximumLength: core::mem::size_of_val(&szInf) as u16,
                    Buffer: szInf.as_mut_ptr(),
                };

                DriveType = GetNtDevicePathOfDriveNumber((Drive as i32 - 'A' as i32) as USHORT, &mut NtDrive);
                if (DriveType) as u32 <= DRIVE_NO_ROOT_DIR {
                    Drive += 1;
                    continue;
                }
                if (Types == 0)
                    != ((DriveType) as u32 == DRIVE_REMOVABLE || (DriveType) as u32 == DRIVE_CDROM)
                {
                    Drive += 1;
                    continue;
                }

                Status = ConcatPaths(
                    szInf.as_mut_ptr(),
                    (_countof!(szInf)) as _,
                    1,
                    RUSTCONV_WSTR_3F80C3C80BCE.as_ptr(),
                );
                if NT_SUCCESS!(Status) != 0
                    && DoesFileExist!(core::ptr::null_mut(), szInf.as_mut_ptr()) != 0
                    && ((IsInfUnattendSetupEnabled(pSetupData, szInf.as_mut_ptr(), (TRUE) as BOOLEAN)) != 0)
                {
                    let _ = RtlStringCchCopyW(
                        UnattendInfPath.as_mut_ptr(),
                        (_countof!(UnattendInfPath)) as _,
                        szInf.as_mut_ptr(),
                    );
                    return NT_SUCCESS!(RtlStringCchCopyW(InfPath, (cch) as _, szInf.as_mut_ptr()));
                }

                Drive += 1;
            }
            Types += 1;
        }
    }

    // Quit:
    if *UnattendInfPath.as_ptr() != 0 {
        return NT_SUCCESS!(RtlStringCchCopyW(InfPath, (cch) as _, UnattendInfPath.as_mut_ptr()));
    }
    return (FALSE) as u8;
}

#[no_mangle]
pub unsafe extern "system" fn CheckUnattendedSetup(
    pSetupData: PUSETUP_DATA,
) -> BOOLEAN {
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut UnattendInf: HINF = core::ptr::null_mut();
    let mut ErrorLine: UINT = 0;
    let mut IntValue: INT = 0;
    let mut Value: PCWSTR = core::ptr::null();
    let mut UnattendInfPath: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    'Quit: {
        if GetUnattendInfPath(
            pSetupData,
            UnattendInfPath.as_mut_ptr() as PWSTR,
            (core::mem::size_of_val(&UnattendInfPath) / core::mem::size_of::<u16>()) as SIZE_T,
        ) == 0
        {
            return IsUnattendedSetup;
        }

        if DbgPrint(
            (b"(%s:%d) UnattendInf path: '%S'\n\0".as_ptr()).cast(),
            b"setuplib.c\0".as_ptr(),
            180 as i32,
            UnattendInfPath.as_mut_ptr() as PWSTR,
        ) != 0
        {
            DbgPrint(
                (b"(%s:%d) DbgPrint() failed!\n\0".as_ptr()).cast(),
                b"setuplib.c\0".as_ptr(),
                180 as i32,
            );
        }

        UnattendInf = SpInfOpenInfFile!(
            UnattendInfPath.as_mut_ptr() as PWSTR,
            core::ptr::null(),
            INF_STYLE_OLDNT,
            ((*pSetupData).LanguageId) as _,
            core::ptr::addr_of_mut!(ErrorLine),
        );
        if UnattendInf == INVALID_HANDLE_VALUE {
            return IsUnattendedSetup;
        }

        if SpInfFindFirstLine!(
            UnattendInf,
            RUSTCONV_WSTR_43552FB35D21.as_ptr(),
            RUSTCONV_WSTR_62CF23C278F1.as_ptr(),
            core::ptr::addr_of_mut!(Context),
        ) == 0
        {
            break 'Quit;
        }

        if INF_GetData(
            core::ptr::addr_of_mut!(Context),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Value),
        ) == 0
        {
            break 'Quit;
        }

        if _wcsicmp(Value, InfUnattendSignature) != 0 {
            INF_FreeData(Value);
            break 'Quit;
        }

        INF_FreeData(Value);

        if SpInfFindFirstLine!(
            UnattendInf,
            RUSTCONV_WSTR_43552FB35D21.as_ptr(),
            RUSTCONV_WSTR_5D19A68CF474.as_ptr(),
            core::ptr::addr_of_mut!(Context),
        ) == 0
        {
            break 'Quit;
        }

        if INF_GetData(
            core::ptr::addr_of_mut!(Context),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Value),
        ) == 0
        {
            break 'Quit;
        }

        if _wcsicmp(Value, RUSTCONV_WSTR_6D1618CC9381.as_ptr()) != 0 {
            INF_FreeData(Value);
            break 'Quit;
        }

        INF_FreeData(Value);

        if SpInfFindFirstLine!(
            UnattendInf,
            RUSTCONV_WSTR_43552FB35D21.as_ptr(),
            RUSTCONV_WSTR_6A0CE9AC469F.as_ptr(),
            core::ptr::addr_of_mut!(Context),
        ) == 0
        {
            break 'Quit;
        }

        if SpInfGetIntField!(
            core::ptr::addr_of_mut!(Context),
            1,
            core::ptr::addr_of_mut!(IntValue),
        ) == 0
        {
            break 'Quit;
        }

        (*pSetupData).DestinationDiskNumber = IntValue as i32;

        if SpInfFindFirstLine!(
            UnattendInf,
            RUSTCONV_WSTR_43552FB35D21.as_ptr(),
            RUSTCONV_WSTR_6DDAE518D89A.as_ptr(),
            core::ptr::addr_of_mut!(Context),
        ) == 0
        {
            break 'Quit;
        }

        if SpInfGetIntField!(
            core::ptr::addr_of_mut!(Context),
            1,
            core::ptr::addr_of_mut!(IntValue),
        ) == 0
        {
            break 'Quit;
        }

        (*pSetupData).DestinationPartitionNumber = IntValue as i32;

        if SpInfFindFirstLine!(
            UnattendInf,
            RUSTCONV_WSTR_43552FB35D21.as_ptr(),
            RUSTCONV_WSTR_F024DF41CF6D.as_ptr(),
            core::ptr::addr_of_mut!(Context),
        ) != 0
        {
            if INF_GetData(
                core::ptr::addr_of_mut!(Context),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Value),
            ) != 0
            {
                RtlStringCchCopyW(
                    (*pSetupData).InstallationDirectory.as_mut_ptr() as PWSTR,
                    core::mem::size_of_val(&(*pSetupData).InstallationDirectory)
                        / core::mem::size_of::<u16>(),
                    Value,
                );
                INF_FreeData(Value);
            }
        }

        IsUnattendedSetup = (TRUE) as u8;

        if SpInfFindFirstLine!(
            UnattendInf,
            RUSTCONV_WSTR_43552FB35D21.as_ptr(),
            RUSTCONV_WSTR_B472F3142B6E.as_ptr(),
            core::ptr::addr_of_mut!(Context),
        ) != 0
        {
            if SpInfGetIntField!(
                core::ptr::addr_of_mut!(Context),
                1,
                core::ptr::addr_of_mut!(IntValue),
            ) != 0
            {
                (*pSetupData).BootLoaderLocation = IntValue as i32;
            }
        }

        if SpInfFindFirstLine!(
            UnattendInf,
            RUSTCONV_WSTR_43552FB35D21.as_ptr(),
            RUSTCONV_WSTR_DF8C4EF9F756.as_ptr(),
            core::ptr::addr_of_mut!(Context),
        ) != 0
        {
            if SpInfGetIntField!(
                core::ptr::addr_of_mut!(Context),
                1,
                core::ptr::addr_of_mut!(IntValue),
            ) != 0
            {
                (*pSetupData).FormatPartition = IntValue as i32;
            }
        }

        if SpInfFindFirstLine!(
            UnattendInf,
            RUSTCONV_WSTR_43552FB35D21.as_ptr(),
            RUSTCONV_WSTR_B57D8EC57C10.as_ptr(),
            core::ptr::addr_of_mut!(Context),
        ) != 0
        {
            if SpInfGetIntField!(
                core::ptr::addr_of_mut!(Context),
                1,
                core::ptr::addr_of_mut!(IntValue),
            ) != 0
            {
                (*pSetupData).AutoPartition = IntValue as i32;
            }
        }

        if SpInfFindFirstLine!(
            UnattendInf,
            RUSTCONV_WSTR_43552FB35D21.as_ptr(),
            RUSTCONV_WSTR_CED1B9EC271C.as_ptr(),
            core::ptr::addr_of_mut!(Context),
        ) != 0
        {
            if INF_GetData(
                core::ptr::addr_of_mut!(Context),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Value),
            ) != 0
            {
                let Id: i32 = wcstol(
                    Value,
                    core::ptr::null_mut(),
                    16,
                ) as i32;
                RtlStringCchPrintfW!(
                    (*pSetupData).LocaleID.as_mut_ptr() as PWSTR,
                    core::mem::size_of_val(&(*pSetupData).LocaleID)
                        / core::mem::size_of::<u16>(),
                    RUSTCONV_WSTR_181895D51D74.as_ptr(),
                    Id,
                );
                INF_FreeData(Value);
            }
        }

        if SpInfFindFirstLine!(
            UnattendInf,
            RUSTCONV_WSTR_43552FB35D21.as_ptr(),
            RUSTCONV_WSTR_B756C84E9F61.as_ptr(),
            core::ptr::addr_of_mut!(Context),
        ) != 0
        {
            if SpInfGetIntField!(
                core::ptr::addr_of_mut!(Context),
                1,
                core::ptr::addr_of_mut!(IntValue),
            ) != 0
            {
                (*pSetupData).FsType = IntValue as i32;
            }
        }
    }

    SpInfCloseInfFile!(UnattendInf);
    return IsUnattendedSetup;
}

#[no_mangle]
pub unsafe extern "system" fn InstallSetupInfFile(pSetupData: PUSETUP_DATA) {
    let mut Status: NTSTATUS;
    let mut IniCache: PINICACHE;
    let mut CrLf: [CHAR; 2] = ['\r' as i32 as CHAR, '\n' as i32 as CHAR];
    let mut FileHandle: HANDLE = core::mem::zeroed();
    let mut UnattendFileHandle: HANDLE = core::mem::zeroed();
    let mut SectionHandle: HANDLE = core::mem::zeroed();
    let mut FileInfo: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut FileSize: ULONG = core::mem::zeroed();
    let mut ViewBase: PVOID = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut IniSection: PINI_SECTION;
    let mut PathBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut UnattendInfPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    IniCache = IniCacheCreate();
    if IniCache.is_null() {
        return;
    }

    IniSection = IniAddSection(IniCache, RUSTCONV_WSTR_05F3C233D307.as_ptr());
    if !IniSection.is_null() {
        RtlStringCchPrintfW!(
            PathBuffer.as_mut_ptr(),
            ARRAYSIZE!(PathBuffer),
            RUSTCONV_WSTR_9896841D0A6A.as_ptr(),
            if IsUnattendedSetup != 0 { RUSTCONV_WSTR_6D1618CC9381.as_ptr() } else { RUSTCONV_WSTR_833AD7EF6992.as_ptr() },
        );
        IniAddKey(IniSection, RUSTCONV_WSTR_D92823782CE4.as_ptr(), PathBuffer.as_mut_ptr());

        RtlStringCchPrintfW!(
            PathBuffer.as_mut_ptr(),
            ARRAYSIZE!(PathBuffer),
            RUSTCONV_WSTR_9896841D0A6A.as_ptr(),
            RUSTCONV_WSTR_E71C047C4F02.as_ptr(),
        );
        IniAddKey(IniSection, RUSTCONV_WSTR_6900BCBBE1A9.as_ptr(), PathBuffer.as_mut_ptr());

        RtlStringCchPrintfW!(
            PathBuffer.as_mut_ptr(),
            ARRAYSIZE!(PathBuffer),
            RUSTCONV_WSTR_81E16440571D.as_ptr(),
            (*pSetupData).SourceRootPath.Buffer,
        );
        IniAddKey(IniSection, RUSTCONV_WSTR_0F0B1FBB5B3E.as_ptr(), PathBuffer.as_mut_ptr());
    }

    CombinePaths(
        PathBuffer.as_mut_ptr(),
        (ARRAYSIZE!(PathBuffer)) as _,
        2,
        (*pSetupData).DestinationPath.Buffer,
        RUSTCONV_WSTR_2AFA0E90CE75.as_ptr(),
    );
    IniCacheSave(IniCache, PathBuffer.as_mut_ptr());
    IniCacheDestroy(IniCache);

    if GetUnattendInfPath(pSetupData, UnattendInfPath.as_mut_ptr(), (ARRAYSIZE!(UnattendInfPath)) as SIZE_T) == 0 {
        DPRINT!("Does not exist: %S\n", UnattendInfPath.as_mut_ptr());
        return;
    }

    RtlInitUnicodeString(&mut FileName, PathBuffer.as_mut_ptr());
    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut FileName,
        OBJ_CASE_INSENSITIVE | OBJ_OPENIF,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenFile(
        &mut FileHandle,
        FILE_APPEND_DATA | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("Cannot load %S as an INI file!\n", PathBuffer.as_mut_ptr());
        return;
    }

    Status = NtQueryInformationFile(
        FileHandle,
        &mut IoStatusBlock,
        &mut FileInfo as *mut FILE_STANDARD_INFORMATION as PVOID,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
        FileStandardInformation,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT!("NtQueryInformationFile() failed (Status %lx)\n", Status);
        FileInfo.EndOfFile.QuadPart = 0;
    }

    Status = OpenAndMapFile(
        core::ptr::null_mut(),
        UnattendInfPath.as_mut_ptr(),
        &mut UnattendFileHandle,
        &mut FileSize,
        &mut SectionHandle,
        &mut ViewBase,
        (FALSE) as BOOLEAN,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("Cannot load %S !\n", UnattendInfPath.as_mut_ptr());
        NtClose(FileHandle);
        return;
    }

    Status = NtWriteFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        CrLf.as_mut_ptr() as PVOID,
        core::mem::size_of::<[CHAR; 2]>() as ULONG,
        &mut FileInfo.EndOfFile,
        core::ptr::null_mut(),
    );

    Status = NtWriteFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        ViewBase,
        FileSize,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
    }

    UnMapAndCloseFile!(UnattendFileHandle, SectionHandle, ViewBase);

    NtClose(FileHandle);
}
#[no_mangle]
pub unsafe extern "C" fn GetSourcePaths(
    SourcePath: PUNICODE_STRING,
    SourceRootPath: PUNICODE_STRING,
    SourceRootDir: PUNICODE_STRING,
) -> NTSTATUS {
    #[repr(C)]
    struct ImageFileBuffer {
        Name: OBJECT_NAME_INFORMATION,
        Buffer: [u16; (MAX_PATH) as _],
    }
    #[repr(C)]
    struct SystemRootBuffer {
        Name: OBJECT_NAME_INFORMATION,
        Buffer: [u16; (MAX_PATH) as _],
    }

    let mut Status: NTSTATUS = 0;
    let mut BufferSize: ULONG = 0;
    let mut Ptr: PWCHAR = core::ptr::null_mut();
    let mut LinkHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    let mut ImageFileBuffer: ImageFileBuffer = core::mem::zeroed();
    let mut InstallSourcePath: PUNICODE_STRING = (&raw mut ImageFileBuffer.Name).cast();
    let mut SystemRootBuffer: SystemRootBuffer = core::mem::zeroed();
    let mut SystemRootPath: PUNICODE_STRING = (&raw mut SystemRootBuffer.Name).cast();
    let SystemRoot: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_D50DAFA02F06.as_ptr());

    RtlInitEmptyUnicodeString(
        InstallSourcePath,
        ImageFileBuffer.Buffer.as_mut_ptr() as PWCHAR,
        core::mem::size_of_val(&ImageFileBuffer.Buffer) as u16,
    );
    BufferSize = core::mem::size_of::<ImageFileBuffer>() as ULONG;
    Status = NtQueryInformationProcess(
        NtCurrentProcess(),
        ProcessImageFileName,
        InstallSourcePath as *mut core::ffi::c_void,
        BufferSize,
        core::ptr::null_mut(),
    );
    if !(Status >= 0) {
        return Status;
    }

    let ImageFileBuffer_Buffer = ImageFileBuffer.Buffer.as_mut_ptr() as PWCHAR;
    Ptr = ImageFileBuffer_Buffer.offset(
        ((*InstallSourcePath).Length as usize / core::mem::size_of::<u16>()) as isize,
    );
    while Ptr > ImageFileBuffer_Buffer && *Ptr != (OBJ_NAME_PATH_SEPARATOR) as _ {
        Ptr = Ptr.offset(-1);
    }
    (*InstallSourcePath).Length = (Ptr as usize).wrapping_sub(ImageFileBuffer_Buffer as usize) as u16;
    *(*InstallSourcePath).Buffer.offset(
        ((*InstallSourcePath).Length as usize / core::mem::size_of::<u16>()) as isize,
    ) = UNICODE_NULL;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &raw const SystemRoot as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    RtlInitEmptyUnicodeString(
        SystemRootPath,
        SystemRootBuffer.Buffer.as_mut_ptr() as PWCHAR,
        core::mem::size_of_val(&SystemRootBuffer.Buffer) as u16,
    );

    Status = NtOpenFile(
        &raw mut LinkHandle,
        SYNCHRONIZE,
        &raw mut ObjectAttributes,
        &raw mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if Status >= 0 {
        Status = NtQueryObject(
            LinkHandle,
            _OBJECT_INFORMATION_CLASS_ObjectNameInformation,
            &raw mut SystemRootBuffer as *mut core::ffi::c_void,
            core::mem::size_of::<SystemRootBuffer>() as ULONG,
            &raw mut BufferSize,
        );
        NtClose(LinkHandle);
    }
    if !(Status >= 0) {
        RtlInitEmptyUnicodeString(
            SystemRootPath,
            SystemRootBuffer.Buffer.as_mut_ptr() as PWCHAR,
            core::mem::size_of_val(&SystemRootBuffer.Buffer) as u16,
        );

        Status = NtOpenSymbolicLinkObject(
            &raw mut LinkHandle,
            SYMBOLIC_LINK_QUERY,
            &raw mut ObjectAttributes,
        );
        if Status >= 0 {
            Status = NtQuerySymbolicLinkObject(
                LinkHandle,
                SystemRootPath,
                &raw mut BufferSize,
            );
            NtClose(LinkHandle);
        }
    }

    if Status >= 0 && RtlPrefixUnicodeString(
        SystemRootPath as *const _UNICODE_STRING,
        InstallSourcePath as *const _UNICODE_STRING,
        (TRUE) as BOOLEAN,
    ) != 0 {
        InstallSourcePath = SystemRootPath;
    }

    RtlCreateUnicodeString(SourcePath, (*InstallSourcePath).Buffer as *const u16);

    Ptr = wcsrchr(
        (*InstallSourcePath).Buffer as *const u16,
        (OBJ_NAME_PATH_SEPARATOR) as _,
    );
    if !Ptr.is_null() {
        RtlCreateUnicodeString(SourceRootDir, Ptr as *const u16);
        *Ptr = UNICODE_NULL;
    } else {
        RtlCreateUnicodeString(SourceRootDir, RUSTCONV_WSTR_96A296D224F2.as_ptr());
    }

    RtlCreateUnicodeString(SourceRootPath, (*InstallSourcePath).Buffer as *const u16);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn LoadSetupInf(
    pSetupData: *mut USETUP_DATA,
) -> ERROR_NUMBER {
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut ErrorLine: UINT = 0;
    let mut IntValue: INT = 0;
    let mut Value: PCWSTR = core::ptr::null();
    let mut FileNameBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    CombinePaths(
        FileNameBuffer.as_mut_ptr(),
        (ARRAYSIZE!(FileNameBuffer)) as _,
        2,
        (*pSetupData).SourcePath.Buffer,
        RUSTCONV_WSTR_E4E924642399.as_ptr(),
    );

    DPRINT!("SetupInf path: '%S'\n", FileNameBuffer.as_ptr());

    (*pSetupData).SetupInf = SpInfOpenInfFile!(
        FileNameBuffer.as_ptr(),
        core::ptr::null(),
        INF_STYLE_WIN4,
        ((*pSetupData).LanguageId) as _,
        &mut ErrorLine,
    );
    if (*pSetupData).SetupInf == INVALID_HANDLE_VALUE {
        return ERROR_LOAD_TXTSETUPSIF;
    }

    if SpInfFindFirstLine!(
        (*pSetupData).SetupInf,
        RUSTCONV_WSTR_1F7A38F3D6FA.as_ptr(),
        RUSTCONV_WSTR_62CF23C278F1.as_ptr(),
        &mut Context,
    ) == 0
    {
        return ERROR_CORRUPT_TXTSETUPSIF;
    }

    if INF_GetData(
        &mut Context,
        core::ptr::null_mut(),
        &mut Value,
    ) == 0
    {
        return ERROR_CORRUPT_TXTSETUPSIF;
    }

    if _wcsicmp(Value, RUSTCONV_WSTR_3EC29A2723BE.as_ptr()) != 0
        && _wcsicmp(Value, RUSTCONV_WSTR_ECD9281DF6C3.as_ptr()) != 0
    {
        INF_FreeData(Value);
        return ERROR_SIGNATURE_TXTSETUPSIF;
    }

    INF_FreeData(Value);

    if SpInfFindFirstLine!(
        (*pSetupData).SetupInf,
        RUSTCONV_WSTR_A358635A5742.as_ptr(),
        RUSTCONV_WSTR_00FE7BCF4C97.as_ptr(),
        &mut Context,
    ) == 0
    {
        return ERROR_CORRUPT_TXTSETUPSIF;
    }

    (*pSetupData).RequiredPartitionDiskSpace = !0;

    if SpInfGetIntField!(
        &mut Context,
        1,
        &mut IntValue,
    ) == 0
    {
        return ERROR_CORRUPT_TXTSETUPSIF;
    }

    (*pSetupData).RequiredPartitionDiskSpace = IntValue as ULONG;

    if SpInfFindFirstLine!(
        (*pSetupData).SetupInf,
        RUSTCONV_WSTR_4B9A840DB974.as_ptr(),
        RUSTCONV_WSTR_CF78061217B0.as_ptr(),
        &mut Context,
    ) != 0
    {
        if INF_GetData(
            &mut Context,
            core::ptr::null_mut(),
            &mut Value,
        ) != 0
        {
            RtlFreeUnicodeString(&mut (*pSetupData).SourceRootPath);
            RtlCreateUnicodeString(&mut (*pSetupData).SourceRootPath, Value);
            INF_FreeData(Value);

            if SpInfFindFirstLine!(
                (*pSetupData).SetupInf,
                RUSTCONV_WSTR_4B9A840DB974.as_ptr(),
                RUSTCONV_WSTR_C3BF39D970E8.as_ptr(),
                &mut Context,
            ) == 0
            {
                return ERROR_CORRUPT_TXTSETUPSIF;
            }

            if INF_GetData(
                &mut Context,
                core::ptr::null_mut(),
                &mut Value,
            ) == 0
            {
                return ERROR_CORRUPT_TXTSETUPSIF;
            }

            RtlFreeUnicodeString(&mut (*pSetupData).SourceRootDir);
            RtlCreateUnicodeString(&mut (*pSetupData).SourceRootDir, Value);
            INF_FreeData(Value);
        }
    }

    (*pSetupData).InstallationDirectory[0] = 0;
    if SpInfFindFirstLine!(
        (*pSetupData).SetupInf,
        RUSTCONV_WSTR_4B9A840DB974.as_ptr(),
        RUSTCONV_WSTR_F2981BFF2146.as_ptr(),
        &mut Context,
    ) != 0
    {
        if INF_GetData(
            &mut Context,
            core::ptr::null_mut(),
            &mut Value,
        ) == 0
        {
            return ERROR_CORRUPT_TXTSETUPSIF;
        }

        RtlStringCchCopyW(
            (*pSetupData).InstallationDirectory.as_mut_ptr(),
            (ARRAYSIZE!((*pSetupData).InstallationDirectory)) as _,
            Value,
        );

        INF_FreeData(Value);
    }

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn InitSystemPartition(
    PartitionList: PPARTLIST,
    InstallPartition: PPARTENTRY,
    pSystemPartition: *mut PPARTENTRY,
    FsVolCallback: PFSVOL_CALLBACK,
    Context: PVOID,
) -> BOOLEAN {
    let mut Result: FSVOL_OP;
    let mut SystemPartition: PPARTENTRY;
    let mut OldActivePart: PPARTENTRY;

    if (*(*InstallPartition).DiskEntry).MediaType == FixedMedia {
        SystemPartition = FindSupportedSystemPartition(
            PartitionList,
            (FALSE) as BOOLEAN,
            (*InstallPartition).DiskEntry,
            InstallPartition,
        );
        OldActivePart = (*PartitionList).SystemPartition;

        if !SystemPartition.is_null()
            && !(*PartitionList).SystemPartition.is_null()
            && SystemPartition != (*PartitionList).SystemPartition
        {
            DPRINT1!("We are using a different system partition!!\n");

            Result = (FsVolCallback.unwrap_unchecked())(
                Context,
                ChangeSystemPartition,
                SystemPartition as ULONG_PTR,
                0,
            );
            if Result != FSVOL_DOIT {
                return (FALSE) as u8;
            }
        }
    } else {
        SystemPartition = InstallPartition;
        OldActivePart = core::ptr::null_mut();
    }

    if SystemPartition.is_null() {
        (FsVolCallback.unwrap_unchecked())(
            Context,
            FSVOLNOTIFY_PARTITIONERROR,
            ERROR_SYSTEM_PARTITION_NOT_FOUND as ULONG_PTR,
            0,
        );
        return (FALSE) as u8;
    }

    *pSystemPartition = SystemPartition;

    if (*SystemPartition).IsPartitioned == 0 {
        CreatePartition(
            PartitionList,
            SystemPartition,
            0,
            0,
        );
    }

    if SetActivePartition(PartitionList, SystemPartition, OldActivePart) == 0 {
        DPRINT1!("SetActivePartition(0x%p) failed?!\n", SystemPartition);
    }

    if !(*SystemPartition).Volume.is_null() {
        (*(*SystemPartition).Volume).NeedsCheck = (TRUE) as u8;
    }

    return (TRUE) as u8;
}

#[no_mangle]
pub unsafe extern "system" fn IsValidInstallDirectory(InstallDir: PCWSTR) -> i32 {
    let mut p: PCWCH = core::ptr::null();
    p = InstallDir as PCWCH;
    if *p == 0
        || (IS_PATH_SEPARATOR!(*p) && *p.offset(1 as isize) == 0)
    {
        return (FALSE) as i32;
    }
    p = InstallDir as PCWCH;
    while *p != 0 {
        if !IS_VALID_INSTALL_PATH_CHAR!(*p) {
            return (FALSE) as i32;
        }
        p = p.offset(1);
    }
    p = InstallDir as PCWCH;
    while *p != 0 {
        let mut Path: PCWSTR = core::ptr::null();
        let mut Length: usize = 0;
        let mut Name: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };
        let mut IsNameLegal: BOOLEAN = 0;
        let mut SpacesInName: BOOLEAN = 0;
        if IS_PATH_SEPARATOR!(*p) {
            p = p.offset(1);
        }
        Path = p as PCWSTR;
        while *p != 0 && !IS_PATH_SEPARATOR!(*p) {
            p = p.offset(1);
        }
        if p == Path {
            return (*p == 0) as i32;
        }
        Length = p.offset_from(Path) as usize;
        if (Length == 1 && *Path == '.' as u16)
            || (Length == 2
                && *Path == '.' as u16
                && *Path.offset(1 as isize) == '.' as u16)
        {
            return (FALSE) as i32;
        }
        if Length > 1
            && *p.offset(-(2 as isize)) != 0x002Eu16
            && *p.offset(-(1 as isize)) == 0x002Eu16
        {
            Length = Length.wrapping_sub(1);
        }
        if Length == 0 {
            return (FALSE) as i32;
        }
        Name.MaximumLength = (Length * core::mem::size_of::<u16>()) as u16;
        Name.Length = Name.MaximumLength;
        Name.Buffer = Path as *mut u16;
        SpacesInName = (FALSE) as BOOLEAN;
        IsNameLegal = (RtlIsNameLegalDOS8Dot3(
            &mut Name as *mut _UNICODE_STRING,
            core::ptr::null_mut(),
            &mut SpacesInName,
        )) as _;
        if IsNameLegal == 0 || SpacesInName != 0 {
            return (FALSE) as i32;
        }
    }
    return (TRUE) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn InitDestinationPaths(
    pSetupData: *mut USETUP_DATA,
    mut InstallationDir: PCWSTR,
    Volume: *mut VOLENTRY,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let PartEntry: *mut PARTENTRY = (*Volume).PartEntry;
    let DiskEntry: *mut DISKENTRY = (*PartEntry).DiskEntry;
    let mut PathBuffer: [u16; RTL_NUMBER_OF_FIELD!(VOLINFO, DeviceName) + 1] = [0; RTL_NUMBER_OF_FIELD!(VOLINFO, DeviceName) + 1];

    RtlFreeUnicodeString(&mut (*pSetupData).DestinationRootPath);
    Status = RtlStringCchPrintfW!(
        PathBuffer.as_mut_ptr(),
        _countof!(PathBuffer),
        RUSTCONV_WSTR_BCF3913447D5.as_ptr(),
        (*Volume).Info.DeviceName.as_ptr(),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("RtlStringCchPrintfW() failed with status 0x%08lx\n", Status);
        return Status;
    }

    Status = if ((RtlCreateUnicodeString(&mut (*pSetupData).DestinationRootPath, PathBuffer.as_ptr())) != 0) {
        STATUS_SUCCESS
    } else {
        STATUS_NO_MEMORY
    };

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("RtlCreateUnicodeString() failed with status 0x%08lx\n", Status);
        return Status;
    }

    DPRINT!("DestinationRootPath: %wZ\n", &(*pSetupData).DestinationRootPath);

    if InstallationDir.is_null() {
        InstallationDir = (*pSetupData).InstallationDirectory.as_ptr();
    }

    RtlFreeUnicodeString(&mut (*pSetupData).DestinationArcPath);

    if (*DiskEntry).MediaType == FixedMedia {
        if (*DiskEntry).BiosFound != 0 {
            Status = RtlStringCchPrintfW!(
                PathBuffer.as_mut_ptr(),
                ARRAYSIZE!(PathBuffer),
                RUSTCONV_WSTR_28FB1027CC87.as_ptr(),
                (*DiskEntry).HwFixedDiskNumber,
                (*PartEntry).OnDiskPartitionNumber,
            );
            DPRINT1!("Fixed disk found by BIOS, using MULTI ARC path '%S'\n", PathBuffer.as_ptr());
        } else {
            Status = RtlStringCchPrintfW!(
                PathBuffer.as_mut_ptr(),
                ARRAYSIZE!(PathBuffer),
                RUSTCONV_WSTR_2AD388D1B224.as_ptr(),
                (*DiskEntry).Port as core::ffi::c_uint,
                (*DiskEntry).Bus as core::ffi::c_uint,
                (*DiskEntry).Id as core::ffi::c_uint,
                (*PartEntry).OnDiskPartitionNumber,
            );
            DPRINT1!("Fixed disk not found by BIOS, using SCSI ARC path '%S'\n", PathBuffer.as_ptr());
        }
    } else {
        Status = RtlStringCchPrintfW!(
            PathBuffer.as_mut_ptr(),
            ARRAYSIZE!(PathBuffer),
            RUSTCONV_WSTR_28FB1027CC87.as_ptr(),
            0,
            1,
        );
        DPRINT1!("Removable disk, using MULTI ARC path '%S'\n", PathBuffer.as_ptr());
    }

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("RtlStringCchPrintfW() failed with status 0x%08lx\n", Status);
        RtlFreeUnicodeString(&mut (*pSetupData).DestinationRootPath);
        return Status;
    }

    Status = ConcatPaths(
        PathBuffer.as_mut_ptr(),
        (ARRAYSIZE!(PathBuffer)) as _,
        1,
        InstallationDir,
    );

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("ConcatPaths() failed with status 0x%08lx\n", Status);
        RtlFreeUnicodeString(&mut (*pSetupData).DestinationRootPath);
        return Status;
    }

    Status = if ((RtlCreateUnicodeString(&mut (*pSetupData).DestinationArcPath, PathBuffer.as_ptr())) != 0) {
        STATUS_SUCCESS
    } else {
        STATUS_NO_MEMORY
    };

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("RtlCreateUnicodeString() failed with status 0x%08lx\n", Status);
        RtlFreeUnicodeString(&mut (*pSetupData).DestinationRootPath);
        return Status;
    }

    RtlFreeUnicodeString(&mut (*pSetupData).DestinationPath);
    Status = CombinePaths(
        PathBuffer.as_mut_ptr(),
        (ARRAYSIZE!(PathBuffer)) as _,
        2,
        (*pSetupData).DestinationRootPath.Buffer,
        InstallationDir,
    );

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("CombinePaths() failed with status 0x%08lx\n", Status);
        RtlFreeUnicodeString(&mut (*pSetupData).DestinationArcPath);
        RtlFreeUnicodeString(&mut (*pSetupData).DestinationRootPath);
        return Status;
    }

    Status = if ((RtlCreateUnicodeString(&mut (*pSetupData).DestinationPath, PathBuffer.as_ptr())) != 0) {
        STATUS_SUCCESS
    } else {
        STATUS_NO_MEMORY
    };

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("RtlCreateUnicodeString() failed with status 0x%08lx\n", Status);
        RtlFreeUnicodeString(&mut (*pSetupData).DestinationArcPath);
        RtlFreeUnicodeString(&mut (*pSetupData).DestinationRootPath);
        return Status;
    }

    Status = if ((RtlCreateUnicodeString(&mut (*pSetupData).InstallPath, InstallationDir)) != 0) {
        STATUS_SUCCESS
    } else {
        STATUS_NO_MEMORY
    };

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("RtlCreateUnicodeString() failed with status 0x%08lx\n", Status);
        RtlFreeUnicodeString(&mut (*pSetupData).DestinationPath);
        RtlFreeUnicodeString(&mut (*pSetupData).DestinationArcPath);
        RtlFreeUnicodeString(&mut (*pSetupData).DestinationRootPath);
        return Status;
    }

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn InitializeSetup(
    pSetupData: PUSETUP_DATA,
    ErrorRoutine: PSETUP_ERROR_ROUTINE,
    pSpFileExports: PSPFILE_EXPORTS,
    pSpInfExports: PSPINF_EXPORTS,
) -> ERROR_NUMBER {
    let mut Error: ERROR_NUMBER;
    let mut Status: NTSTATUS;

    IsUnattendedSetup = (FALSE) as u8;
    RtlZeroMemory!(pSetupData as *mut core::ffi::c_void, core::mem::size_of::<USETUP_DATA>());

    (*pSetupData).LastErrorNumber = ERROR_SUCCESS;
    (*pSetupData).ErrorRoutine = ErrorRoutine;

    RtlInitUnicodeString(&raw mut (*pSetupData).SourcePath, core::ptr::null_mut());
    RtlInitUnicodeString(&raw mut (*pSetupData).SourceRootPath, core::ptr::null_mut());
    RtlInitUnicodeString(&raw mut (*pSetupData).SourceRootDir, core::ptr::null_mut());
    RtlInitUnicodeString(&raw mut (*pSetupData).DestinationArcPath, core::ptr::null_mut());
    RtlInitUnicodeString(&raw mut (*pSetupData).DestinationPath, core::ptr::null_mut());
    RtlInitUnicodeString(&raw mut (*pSetupData).DestinationRootPath, core::ptr::null_mut());
    RtlInitUnicodeString(&raw mut (*pSetupData).SystemRootPath, core::ptr::null_mut());

    RtlInitUnicodeString(&raw mut (*pSetupData).InstallPath, core::ptr::null_mut());

    RtlCopyMemory!(
        &raw mut SpFileExports as *mut core::ffi::c_void,
        pSpFileExports as *const core::ffi::c_void,
        core::mem::size_of::<SPFILE_EXPORTS>(),
    );
    RtlCopyMemory!(
        &raw mut SpInfExports as *mut core::ffi::c_void,
        pSpInfExports as *const core::ffi::c_void,
        core::mem::size_of::<SPINF_EXPORTS>(),
    );

    Status = GetSourcePaths(
        &raw mut (*pSetupData).SourcePath,
        &raw mut (*pSetupData).SourceRootPath,
        &raw mut (*pSetupData).SourceRootDir,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("GetSourcePaths() failed (Status 0x%08lx)\n", Status);
        return ERROR_NO_SOURCE_DRIVE;
    }
    DPRINT1!("SourcePath (1): '%wZ'\n", &raw mut (*pSetupData).SourcePath);
    DPRINT1!("SourceRootPath (1): '%wZ'\n", &raw mut (*pSetupData).SourceRootPath);
    DPRINT1!("SourceRootDir (1): '%wZ'\n", &raw mut (*pSetupData).SourceRootDir);

    (*pSetupData).DestinationDiskNumber = 0;
    (*pSetupData).DestinationPartitionNumber = 1;
    (*pSetupData).BootLoaderLocation = 2;
    (*pSetupData).FormatPartition = 0;
    (*pSetupData).AutoPartition = 0;
    (*pSetupData).FsType = 0;

    Error = LoadSetupInf(pSetupData);
    if Error != ERROR_SUCCESS {
        DPRINT1!("LoadSetupInf() failed (Error 0x%lx)\n", Error);
        return Error;
    }
    DPRINT1!("SourcePath (2): '%wZ'\n", &raw mut (*pSetupData).SourcePath);
    DPRINT1!("SourceRootPath (2): '%wZ'\n", &raw mut (*pSetupData).SourceRootPath);
    DPRINT1!("SourceRootDir (2): '%wZ'\n", &raw mut (*pSetupData).SourceRootDir);

    // ndk/ketypes.h: on _X86_, IsNEC_98 is the run-time check
    // `(SharedUserData->AlternativeArchitecture == NEC98x86)`, SharedUserData being
    // `((KUSER_SHARED_DATA *)USER_SHARED_DATA)`; on every other arch the island's IsNEC_98 (FALSE).
    #[cfg(target_arch = "x86")]
    let IsNEC_98: u32 = ((*(USER_SHARED_DATA as usize as *const KUSER_SHARED_DATA)).AlternativeArchitecture
        == NEC98x86) as u32;
    (*pSetupData).ArchType = if IsNEC_98 != 0 {
        ARCH_NEC98x86
    } else if BootedFromUefi() {
        ARCH_Efi
    } else {
        ARCH_PcAT
    };

    return ERROR_SUCCESS;
}

/// Whether the firmware that started this boot is UEFI, from `SystemBootEnvironmentInformation`
/// (the loader block's `BootViaEFI`). Upstream hard-wires `ARCH_PcAT` here, which made every
/// UEFI install write a BIOS boot sector the firmware never runs.
pub(crate) unsafe fn BootedFromUefi() -> bool {
    extern "system" {
        fn NtQuerySystemInformation(Class: u32, Info: *mut core::ffi::c_void, Length: u32, ReturnLength: *mut u32) -> NTSTATUS;
    }
    const SYSTEM_BOOT_ENVIRONMENT_INFORMATION_CLASS: u32 = 90;
    const FIRMWARE_TYPE_UEFI: u32 = 2;
    // GUID BootIdentifier; FIRMWARE_TYPE FirmwareType (the Vista layout).
    let mut info = [0u32; 5];
    let status = NtQuerySystemInformation(
        SYSTEM_BOOT_ENVIRONMENT_INFORMATION_CLASS,
        info.as_mut_ptr().cast(),
        core::mem::size_of_val(&info) as u32,
        core::ptr::null_mut(),
    );
    DPRINT1!("Boot firmware: Status 0x%08lx, FirmwareType %lu\n", status, info[4]);
    status >= 0 && info[4] == FIRMWARE_TYPE_UEFI
}

#[no_mangle]
pub unsafe extern "system" fn FinishSetup(pSetupData: PUSETUP_DATA) {
    if !(*pSetupData).ComputerList.is_null() {
        DestroyGenericList((*pSetupData).ComputerList, (TRUE) as BOOLEAN);
        (*pSetupData).ComputerList = core::ptr::null_mut();
    }
    if !(*pSetupData).DisplayList.is_null() {
        DestroyGenericList((*pSetupData).DisplayList, (TRUE) as BOOLEAN);
        (*pSetupData).DisplayList = core::ptr::null_mut();
    }
    if !(*pSetupData).KeyboardList.is_null() {
        DestroyGenericList((*pSetupData).KeyboardList, (TRUE) as BOOLEAN);
        (*pSetupData).KeyboardList = core::ptr::null_mut();
    }
    if !(*pSetupData).LayoutList.is_null() {
        DestroyGenericList((*pSetupData).LayoutList, (TRUE) as BOOLEAN);
        (*pSetupData).LayoutList = core::ptr::null_mut();
    }
    if !(*pSetupData).LanguageList.is_null() {
        DestroyGenericList((*pSetupData).LanguageList, (FALSE) as BOOLEAN);
        (*pSetupData).LanguageList = core::ptr::null_mut();
    }
    SpInfCloseInfFile!((*pSetupData).SetupInf);
}

#[no_mangle]
pub unsafe extern "system" fn UpdateRegistry(
    pSetupData: *mut USETUP_DATA,
    RepairUpdateFlag: BOOLEAN,
    PartitionList: *mut PARTLIST,
    DestinationDriveLetter: WCHAR,
    SelectedLanguageId: PCWSTR,
    StatusRoutine: PREGISTRY_STATUS_ROUTINE,
    SubstSettings: *mut FONTSUBSTSETTINGS,
) -> ERROR_NUMBER {
    let mut ErrorNumber: ERROR_NUMBER;
    let mut Status: NTSTATUS;
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut Action: PCWSTR = core::ptr::null();
    let mut File: PCWSTR = core::ptr::null();
    let mut Section: PCWSTR = core::ptr::null();
    let mut Success: BOOLEAN;
    let mut ShouldRepairRegistry: BOOLEAN = (FALSE) as u8;
    let mut Delete: BOOLEAN;

    if RepairUpdateFlag != 0 {
        DPRINT1!("TODO: Updating / repairing the registry is not completely implemented yet!\n");

        Status = VerifyRegistryHives(
            &mut (*pSetupData).DestinationPath,
            &mut ShouldRepairRegistry,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!("VerifyRegistryHives failed, Status 0x%08lx\n", Status);
            ShouldRepairRegistry = (FALSE) as u8;
        }
        if ShouldRepairRegistry == 0 {
            DPRINT1!("No need to repair the registry\n");
        }
    }

    // `DoUpdate:` -- every `goto Cleanup` that retries is `continue 'DoUpdate`.
    'DoUpdate: loop {
        ErrorNumber = ERROR_SUCCESS;

        if !StatusRoutine.is_none() {
            (StatusRoutine.unwrap_unchecked())(RegHiveUpdate);
        }

        Status = RegInitializeRegistry(&mut (*pSetupData).DestinationPath);
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!("RegInitializeRegistry() failed\n");
            if Status == STATUS_NOT_IMPLEMENTED {
                return ERROR_INITIALIZE_REGISTRY;
            } else {
                return ERROR_CREATE_HIVE;
            }
        }

        if RepairUpdateFlag == 0 || ShouldRepairRegistry != 0 {
            Success = SpInfFindFirstLine!(
                (*pSetupData).SetupInf,
                RUSTCONV_WSTR_3756A572850B.as_ptr(),
                core::ptr::null_mut(),
                &mut InfContext,
            ) as BOOLEAN;
            if Success == 0 {
                Success = SpInfFindFirstLine!(
                    (*pSetupData).SetupInf,
                    RUSTCONV_WSTR_4C853821A99F.as_ptr(),
                    core::ptr::null_mut(),
                    &mut InfContext,
                ) as BOOLEAN;
            }

            if Success == 0 {
                DPRINT1!("SpInfFindFirstLine!() failed\n");
                ErrorNumber = ERROR_FIND_REGISTRY;
                RegCleanupRegistry(&mut (*pSetupData).DestinationPath);
                if RepairUpdateFlag != 0 && ShouldRepairRegistry != 0 {
                    ShouldRepairRegistry = (FALSE) as u8;
                    continue;
                }
                return ErrorNumber;
            }
        } else {
            Success = SpInfFindFirstLine!(
                (*pSetupData).SetupInf,
                RUSTCONV_WSTR_D5E4D86E3074.as_ptr(),
                core::ptr::null_mut(),
                &mut InfContext,
            ) as BOOLEAN;
            if Success == 0 {
                DPRINT1!("No update needed for the registry!\n");
                RegCleanupRegistry(&mut (*pSetupData).DestinationPath);
                if RepairUpdateFlag != 0 && ShouldRepairRegistry != 0 {
                    ShouldRepairRegistry = (FALSE) as u8;
                    continue;
                }
                return ErrorNumber;
            }
        }

        // C's `do { ... } while (SpInfFindNextLine(...))`: a `continue` in the body goes to the
        // loop condition, so it is `break 'Body`, never a bare `continue` (that would skip
        // SpInfFindNextLine and spin on the same line forever).
        'Records: loop {
            'Body: {
                INF_GetDataField(&mut InfContext, 0, &mut Action);
                INF_GetDataField(&mut InfContext, 1, &mut File);
                INF_GetDataField(&mut InfContext, 2, &mut Section);

                DPRINT!("Action: %S  File: %S  Section %S\n", Action, File, Section);

                if Action.is_null() {
                    INF_FreeData(Action);
                    INF_FreeData(File);
                    INF_FreeData(Section);
                    break 'Records;
                }

                if _wcsicmp(Action, RUSTCONV_WSTR_1CA9A6B1F115.as_ptr()) == 0 {
                    Delete = (FALSE) as u8;
                } else if _wcsicmp(Action, RUSTCONV_WSTR_EF6224203743.as_ptr()) == 0 {
                    Delete = (TRUE) as u8;
                } else {
                    DPRINT1!("Unrecognized registry INF action '%S'\n", Action);
                    INF_FreeData(Action);
                    INF_FreeData(File);
                    INF_FreeData(Section);
                    break 'Body;
                }

                INF_FreeData(Action);

                if !StatusRoutine.is_none() {
                    (StatusRoutine.unwrap_unchecked())(ImportRegHive, File);
                }

                if ImportRegistryFile(
                    (*pSetupData).SourcePath.Buffer,
                    File,
                    Section,
                    ((*pSetupData).LanguageId) as LCID,
                    Delete,
                ) == 0
                {
                    DPRINT1!("Importing %S failed\n", File);
                    INF_FreeData(File);
                    INF_FreeData(Section);
                    ErrorNumber = ERROR_IMPORT_HIVE;
                    RegCleanupRegistry(&mut (*pSetupData).DestinationPath);
                    if RepairUpdateFlag != 0 && ShouldRepairRegistry != 0 {
                        ShouldRepairRegistry = (FALSE) as u8;
                        continue 'DoUpdate;
                    }
                    return ErrorNumber;
                }

            }

            if SpInfFindNextLine!(&mut InfContext, &mut InfContext) == 0 {
                break;
            }
        }

        if RepairUpdateFlag == 0 || ShouldRepairRegistry != 0 {
            let mut Entry: *mut GENERIC_LIST_ENTRY;
            let mut LanguageId: PCWSTR;

            Entry = GetCurrentListEntry((*pSetupData).DisplayList);

            (*pSetupData).DisplayType = (*(GetListEntryData(Entry) as *mut GENENTRY)).Id;

            if !StatusRoutine.is_none() {
                (StatusRoutine.unwrap_unchecked())(DisplaySettingsUpdate);
            }
            if ProcessDisplayRegistry((*pSetupData).SetupInf, (*pSetupData).DisplayType) == 0 {
                ErrorNumber = ERROR_UPDATE_DISPLAY_SETTINGS;
                RegCleanupRegistry(&mut (*pSetupData).DestinationPath);
                if RepairUpdateFlag != 0 && ShouldRepairRegistry != 0 {
                    ShouldRepairRegistry = (FALSE) as u8;
                    continue;
                }
                return ErrorNumber;
            }

            Entry = GetCurrentListEntry((*pSetupData).LanguageList);

            LanguageId = (*(GetListEntryData(Entry) as *mut GENENTRY)).Id;

            if !StatusRoutine.is_none() {
                (StatusRoutine.unwrap_unchecked())(LocaleSettingsUpdate);
            }
            if ProcessLocaleRegistry(LanguageId) == 0 {
                ErrorNumber = ERROR_UPDATE_LOCALESETTINGS;
                RegCleanupRegistry(&mut (*pSetupData).DestinationPath);
                if RepairUpdateFlag != 0 && ShouldRepairRegistry != 0 {
                    ShouldRepairRegistry = (FALSE) as u8;
                    continue;
                }
                return ErrorNumber;
            }

            if !StatusRoutine.is_none() {
                (StatusRoutine.unwrap_unchecked())(KeybLayouts);
            }
            if AddKeyboardLayouts(SelectedLanguageId) == 0 {
                ErrorNumber = ERROR_ADDING_KBLAYOUTS;
                RegCleanupRegistry(&mut (*pSetupData).DestinationPath);
                if RepairUpdateFlag != 0 && ShouldRepairRegistry != 0 {
                    ShouldRepairRegistry = (FALSE) as u8;
                    continue;
                }
                return ErrorNumber;
            }

            Entry = GetCurrentListEntry((*pSetupData).LayoutList);

            (*pSetupData).LayoutId = (*(GetListEntryData(Entry) as *mut GENENTRY)).Id;

            if !StatusRoutine.is_none() {
                (StatusRoutine.unwrap_unchecked())(KeybSettingsUpdate);
            }
            if ProcessKeyboardLayoutRegistry((*pSetupData).LayoutId, SelectedLanguageId) == 0 {
                ErrorNumber = ERROR_UPDATE_KBSETTINGS;
                RegCleanupRegistry(&mut (*pSetupData).DestinationPath);
                if RepairUpdateFlag != 0 && ShouldRepairRegistry != 0 {
                    ShouldRepairRegistry = (FALSE) as u8;
                    continue;
                }
                return ErrorNumber;
            }

            if SetGeoID(MUIGetGeoID(SelectedLanguageId)) == 0 {
                ErrorNumber = ERROR_UPDATE_GEOID;
                RegCleanupRegistry(&mut (*pSetupData).DestinationPath);
                if RepairUpdateFlag != 0 && ShouldRepairRegistry != 0 {
                    ShouldRepairRegistry = (FALSE) as u8;
                    continue;
                }
                return ErrorNumber;
            }

            if !StatusRoutine.is_none() {
                (StatusRoutine.unwrap_unchecked())(CodePageInfoUpdate);
            }
            if AddCodePage(SelectedLanguageId) == 0 {
                ErrorNumber = ERROR_ADDING_CODEPAGE;
                RegCleanupRegistry(&mut (*pSetupData).DestinationPath);
                if RepairUpdateFlag != 0 && ShouldRepairRegistry != 0 {
                    ShouldRepairRegistry = (FALSE) as u8;
                    continue;
                }
                return ErrorNumber;
            }

            SetDefaultPagefile(DestinationDriveLetter);

            SetMountedDeviceValues(PartitionList);
        }

        if !SubstSettings.is_null() {
            DoRegistryFontFixup(
                (SubstSettings).cast(),
                (wcstoul(
                    SelectedLanguageId,
                    core::ptr::null_mut(),
                    16,
                )) as LANGID,
            );
        }

        RegCleanupRegistry(&mut (*pSetupData).DestinationPath);

        if RepairUpdateFlag != 0 && ShouldRepairRegistry != 0 {
            ShouldRepairRegistry = (FALSE) as u8;
            continue;
        }

        return ErrorNumber;
    }
}
