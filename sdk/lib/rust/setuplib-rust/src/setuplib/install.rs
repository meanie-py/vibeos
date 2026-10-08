use crate::setuplib::filesup::CombinePaths;
use crate::setuplib::genlist::GetCurrentListEntry;
use crate::setuplib::genlist::GetListEntryData;
use crate::setuplib::infsupp::INF_GetData;
use crate::setuplib::infsupp::INF_GetDataField;
use crate::setuplib::settings::ProcessComputerFiles;
use crate::setuplib::filesup::SetupCreateDirectory;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_068BF9CAD306: [u16; 5] = [0x002Eu16, 0x0069, 0x006E, 0x0066, 0x0000];
pub static RUSTCONV_WSTR_4DB9C031839F: [u16; 12] = [
    0x0044u16, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072, 0x0069, 0x0065,
    0x0073, 0x0000,
];
#[cfg(target_arch = "x86_64")]
pub static RUSTCONV_WSTR_58A47D42C9D1: [u16; 23] = [
    0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x006B,
    0x0073, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x002E, 0x0061, 0x006D, 0x0064,
    0x0036, 0x0034, 0x0000,
];
#[cfg(target_arch = "aarch64")]
pub static RUSTCONV_WSTR_58A47D42C9D1: [u16; 23] = [
    0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x006B,
    0x0073, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x002E, 0x0061, 0x0072, 0x006D,
    0x0036, 0x0034, 0x0000,
];
#[cfg(target_arch = "x86")]
pub static RUSTCONV_WSTR_58A47D42C9D1: [u16; 21] = [
    0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x006B,
    0x0073, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x002E, 0x0078, 0x0038, 0x0036,
    0x0000,
];
pub static RUSTCONV_WSTR_7BA2F9B4369C: [u16; 17] = [
    0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x006B,
    0x0073, 0x004E, 0x0061, 0x006D, 0x0065, 0x0073, 0x0000,
];
#[cfg(target_arch = "x86_64")]
pub static RUSTCONV_WSTR_861484E4A678: [u16; 23] = [
    0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x006B,
    0x0073, 0x004E, 0x0061, 0x006D, 0x0065, 0x0073, 0x002E, 0x0061, 0x006D, 0x0064,
    0x0036, 0x0034, 0x0000,
];
#[cfg(target_arch = "aarch64")]
pub static RUSTCONV_WSTR_861484E4A678: [u16; 23] = [
    0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x006B,
    0x0073, 0x004E, 0x0061, 0x006D, 0x0065, 0x0073, 0x002E, 0x0061, 0x0072, 0x006D,
    0x0036, 0x0034, 0x0000,
];
#[cfg(target_arch = "x86")]
pub static RUSTCONV_WSTR_861484E4A678: [u16; 21] = [
    0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x006B,
    0x0073, 0x004E, 0x0061, 0x006D, 0x0065, 0x0073, 0x002E, 0x0078, 0x0038, 0x0036,
    0x0000,
];
pub static RUSTCONV_WSTR_8884DC47A3A7: [u16; 9] = [0x0043u16, 0x0061, 0x0062, 0x0069, 0x006E, 0x0065, 0x0074, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_8917519BE0FF: [u16; 17] = [
    0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x006B,
    0x0073, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_97A4EBECDF18: [u16; 17] = [
    0x0057u16, 0x0069, 0x006E, 0x006E, 0x0074, 0x0044, 0x0069, 0x0072, 0x0065, 0x0063,
    0x0074, 0x006F, 0x0072, 0x0069, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_EDFBB205E3F3: [u16; 12] = [
    0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0046, 0x0069, 0x006C, 0x0065,
    0x0073, 0x0000,
];

use core::ffi::c_void;

unsafe extern "C" fn LookupDirectoryById(
    mut InfHandle: HINF,
    mut InfContext: PINFCONTEXT,
    mut DirId: PCWSTR,
    mut pDirectory: *mut PCWSTR,
) -> BOOL {
    let mut Success: BOOL = 0;
    Success = SpInfFindFirstLine!(InfHandle, RUSTCONV_WSTR_4DB9C031839F.as_ptr(), DirId, InfContext);
    if Success == 0 {
        Success = SpInfFindFirstLine!(InfHandle, RUSTCONV_WSTR_97A4EBECDF18.as_ptr(), DirId, InfContext);
        if Success == 0 {
            DPRINT1!(b"SpInfFindFirstLine() failed\n\0".as_ptr() as *const i8);
        }
    }
    if Success != 0 {
        Success = (INF_GetData(InfContext, core::ptr::null_mut::<PCWSTR>(), pDirectory)) as _;
        if Success == 0 {
            DPRINT1!(b"INF_GetData() failed\n\0".as_ptr() as *const i8);
        }
    }
    if Success == 0 {
        DPRINT1!(
            b"LookupDirectoryById(%S) - directory not found!\n\0".as_ptr() as *const i8,
            DirId,
        );
    }
    return Success;
}

unsafe extern "C" fn GetSourceFileAndTargetLocation(
    mut InfHandle: HINF,
    mut InfContext: PINFCONTEXT,
    mut SourceFileName: PCWSTR,
    mut pSourceRootPath: *mut PCWSTR,
    mut pSourcePath: *mut PCWSTR,
    mut pTargetDirectory: *mut PCWSTR,
    mut pTargetFileName: *mut PCWSTR,
) -> NTSTATUS {
    unsafe {
        let mut Success: BOOL = 0;
        let mut FileContext: INFCONTEXT = core::mem::zeroed();
        let mut DirContext: INFCONTEXT = core::mem::zeroed();
        let mut SourceRootDirId: PCWSTR = core::ptr::null::<u16>();
        let mut SourceRootDir: PCWSTR = core::ptr::null::<u16>();
        let mut SourceRelativePath: PCWSTR = core::ptr::null::<u16>();
        let mut TargetDirId: PCWSTR = core::ptr::null::<u16>();
        let mut TargetDir: PCWSTR = core::ptr::null::<u16>();
        let mut TargetFileName: PCWSTR = core::ptr::null::<u16>();

        if InfContext.is_null() && SourceFileName.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

        if InfContext.is_null() {
            Success = SpInfFindFirstLine!(
                InfHandle,
                RUSTCONV_WSTR_58A47D42C9D1.as_ptr(),
                SourceFileName,
                core::ptr::addr_of_mut!(FileContext),
            );
            if Success == 0 {
                Success = SpInfFindFirstLine!(
                    InfHandle,
                    RUSTCONV_WSTR_8917519BE0FF.as_ptr(),
                    SourceFileName,
                    core::ptr::addr_of_mut!(FileContext),
                );
            }
            if Success == 0 {
                return STATUS_NOT_FOUND;
            }
            InfContext = core::ptr::addr_of_mut!(FileContext);
        }

        if INF_GetDataField(
            InfContext,
            1,
            core::ptr::addr_of_mut!(SourceRootDirId),
        ) == 0
        {
            DPRINT1!(b"INF_GetData() failed\n\0".as_ptr() as *const core::ffi::c_char);
            return STATUS_NOT_FOUND;
        }

        Success = SpInfFindFirstLine!(
            InfHandle,
            RUSTCONV_WSTR_861484E4A678.as_ptr(),
            SourceRootDirId,
            core::ptr::addr_of_mut!(DirContext),
        );
        if Success == 0 {
            Success = SpInfFindFirstLine!(
                InfHandle,
                RUSTCONV_WSTR_7BA2F9B4369C.as_ptr(),
                SourceRootDirId,
                core::ptr::addr_of_mut!(DirContext),
            );
            if Success == 0 {
                DPRINT1!(
                    b"SpInfFindFirstLine(\"SourceDisksNames\", \"%S\") failed\n\0".as_ptr()
                        as *const core::ffi::c_char,
                    SourceRootDirId,
                );
            }
        }
        INF_FreeData(SourceRootDirId);
        if Success == 0 {
            return STATUS_NOT_FOUND;
        }
        if INF_GetDataField(
            core::ptr::addr_of_mut!(DirContext),
            4,
            core::ptr::addr_of_mut!(SourceRootDir),
        ) == 0
        {
            DPRINT1!(b"INF_GetData() failed\n\0".as_ptr() as *const core::ffi::c_char);
            return STATUS_NOT_FOUND;
        }

        if INF_GetDataField(
            InfContext,
            2,
            core::ptr::addr_of_mut!(SourceRelativePath),
        ) == 0
        {
            SourceRelativePath = core::ptr::null::<u16>();
        } else if *SourceRelativePath == 0 {
            INF_FreeData(SourceRelativePath);
            SourceRelativePath = core::ptr::null::<u16>();
        }
        if SourceRelativePath.is_null() {
            if INF_GetDataField(
                InfContext,
                13,
                core::ptr::addr_of_mut!(TargetDirId),
            ) != 0
            {
                Success = LookupDirectoryById(
                    InfHandle,
                    core::ptr::addr_of_mut!(DirContext),
                    TargetDirId,
                    core::ptr::addr_of_mut!(SourceRelativePath),
                );
                INF_FreeData(TargetDirId);
                if Success == 0 {
                    SourceRelativePath = core::ptr::null::<u16>();
                } else if *SourceRelativePath == 0 {
                    INF_FreeData(SourceRelativePath);
                    SourceRelativePath = core::ptr::null::<u16>();
                }
            }
        }

        if INF_GetDataField(
            InfContext,
            8,
            core::ptr::addr_of_mut!(TargetDirId),
        ) == 0
        {
            DPRINT1!(b"INF_GetData() failed\n\0".as_ptr() as *const core::ffi::c_char);
            INF_FreeData(SourceRelativePath);
            INF_FreeData(SourceRootDir);
            return STATUS_NOT_FOUND;
        }

        Success = LookupDirectoryById(
            InfHandle,
            core::ptr::addr_of_mut!(DirContext),
            TargetDirId,
            core::ptr::addr_of_mut!(TargetDir),
        );
        INF_FreeData(TargetDirId);
        if Success == 0 {
            INF_FreeData(SourceRelativePath);
            INF_FreeData(SourceRootDir);
            return STATUS_NOT_FOUND;
        }

        if INF_GetDataField(
            InfContext,
            11,
            core::ptr::addr_of_mut!(TargetFileName),
        ) == 0
        {
            TargetFileName = core::ptr::null::<u16>();
        } else if *TargetFileName == 0 {
            TargetFileName = core::ptr::null::<u16>();
        }

        DPRINT!(
            b"GetSourceFileAndTargetLocation(%S) = SrcRootDir: '%S', SrcRelPath: '%S' --> TargetDir: '%S', TargetFileName: '%S'\n\0"
                .as_ptr() as *const core::ffi::c_char,
            SourceFileName,
            SourceRootDir,
            SourceRelativePath,
            TargetDir,
            TargetFileName,
        );

        *pSourceRootPath = SourceRootDir;
        *pSourcePath = SourceRelativePath;
        *pTargetDirectory = TargetDir;
        *pTargetFileName = TargetFileName;

        return STATUS_SUCCESS;
    }
}

unsafe extern "C" fn BuildFullDirectoryPath(
    RootPath: PCWSTR,
    BasePath: PCWSTR,
    RelativePath: PCWSTR,
    FullPath: PWSTR,
    cchFullPathSize: usize,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if (*RelativePath.offset(0 as isize) as u16 == UNICODE_NULL as u16)
        || (*RelativePath.offset(0 as isize) as u16 == 0x005Cu16
            && *RelativePath.offset(1 as isize) as u16 == UNICODE_NULL as u16)
    {
        DPRINT!(b"InstallationPath: '%S'\n\0".as_ptr() as *const i8, RelativePath);

        Status = CombinePaths(
            FullPath,
            cchFullPathSize,
            2,
            RootPath,
            BasePath,
        );

        DPRINT!(b"InstallationPath(2): '%S'\n\0".as_ptr() as *const i8, FullPath);
    } else if *RelativePath.offset(0 as isize) as u16 == 0x005Cu16 {
        DPRINT!(b"AbsolutePath: '%S'\n\0".as_ptr() as *const i8, RelativePath);

        Status = CombinePaths(
            FullPath,
            cchFullPathSize,
            2,
            RootPath,
            RelativePath,
        );

        DPRINT!(b"AbsolutePath(2): '%S'\n\0".as_ptr() as *const i8, FullPath);
    } else {
        DPRINT!(b"RelativePath: '%S'\n\0".as_ptr() as *const i8, RelativePath);

        Status = CombinePaths(
            FullPath,
            cchFullPathSize,
            3,
            RootPath,
            BasePath,
            RelativePath,
        );

        DPRINT!(b"RelativePath(2): '%S'\n\0".as_ptr() as *const i8, FullPath);
    }

    return Status;
}

unsafe extern "C" fn AddSectionToCopyQueueCab(
    mut pSetupData: PUSETUP_DATA,
    mut InfFile: HINF,
    mut SectionName: PCWSTR,
    mut SourceCabinet: PCWSTR,
    mut DestinationPath: PCUNICODE_STRING,
) -> BOOLEAN {
    let mut Success: BOOLEAN = 0 as BOOLEAN;
    let mut Status: NTSTATUS = 0 as NTSTATUS;
    let mut FilesContext: INFCONTEXT = core::mem::zeroed();
    let mut DirContext: INFCONTEXT = core::mem::zeroed();
    let mut SourceFileName: PCWSTR = core::ptr::null();
    let mut TargetDirId: PCWSTR = core::ptr::null();
    let mut TargetDir: PCWSTR = core::ptr::null();
    let mut TargetFileName: PCWSTR = core::ptr::null();
    let mut FileDstPath: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];

    if SpInfFindFirstLine!(
        InfFile,
        SectionName,
        core::ptr::null(),
        &raw mut FilesContext,
    ) == 0
    {
        DPRINT1!(
            b"AddSectionToCopyQueueCab(): Unable to find section '%S' in cabinet file\n\0"
                .as_ptr() as *const i8,
            SectionName,
        );
        return FALSE as BOOLEAN;
    }

    loop {
        if INF_GetData(
            &raw mut FilesContext,
            &raw mut SourceFileName,
            &raw mut TargetDirId,
        ) == 0
        {
            DPRINT1!(
                b"INF_GetData() failed\n\0".as_ptr() as *const i8,
            );
            break;
        }

        if INF_GetDataField(
            &raw mut FilesContext,
            2,
            &raw mut TargetFileName,
        ) == 0
        {
            TargetFileName = core::ptr::null();
        } else if *TargetFileName == 0 {
            INF_FreeData(TargetFileName);
            TargetFileName = core::ptr::null();
        }

        Success = LookupDirectoryById(
            InfFile,
            &raw mut DirContext,
            TargetDirId,
            &raw mut TargetDir,
        ) as BOOLEAN;
        INF_FreeData(TargetDirId);
        if Success == 0 {
            INF_FreeData(TargetFileName);
            INF_FreeData(SourceFileName);
            break;
        }

        DPRINT!(
            b"GetSourceTargetFromCab(%S) = SrcRootDir: '%S', SrcRelPath: '%S' --> TargetDir: '%S', TargetFileName: '%S'\n\0"
                .as_ptr() as *const i8,
            SourceFileName,
            (*pSetupData).SourcePath.Buffer,
            (*pSetupData).SourceRootDir.Buffer,
            TargetDir,
            TargetFileName,
        );

        Status = CombinePaths(
            &raw mut FileDstPath as PWSTR,
            (ARRAYSIZE!(FileDstPath)) as _,
            2,
            (*pSetupData).DestinationPath.Buffer,
            TargetDir,
        );
        UNREFERENCED_PARAMETER!(Status);
        DPRINT!(
            b"  --> FileDstPath = '%S'\n\0".as_ptr() as *const i8,
            FileDstPath.as_ptr(),
        );

        INF_FreeData(TargetDir);

        if SpFileQueueCopy!(
            (*pSetupData).SetupFileQueue as HSPFILEQ,
            (*pSetupData).SourcePath.Buffer,
            core::ptr::null(),
            SourceFileName,
            core::ptr::null(),
            SourceCabinet,
            core::ptr::null(),
            &raw mut FileDstPath as *mut WCHAR as PCWSTR,
            TargetFileName,
            0,
        ) == 0
        {
            DPRINT1!(
                b"SpFileQueueCopy() failed\n\0".as_ptr() as *const i8,
            );
        }

        INF_FreeData(TargetFileName);
        INF_FreeData(SourceFileName);

        if SpInfFindNextLine!(
            &raw mut FilesContext,
            &raw mut FilesContext,
        ) == 0
        {
            break;
        }
    }

    return TRUE as BOOLEAN;
}

unsafe extern "C" fn AddSectionToCopyQueue(
    mut pSetupData: PUSETUP_DATA,
    mut InfFile: HINF,
    mut SectionName: PCWSTR,
    mut DestinationPath: PCUNICODE_STRING,
) -> BOOLEAN {
    let mut Status: NTSTATUS = 0;
    let mut FilesContext: INFCONTEXT = INFCONTEXT {
        Inf: core::ptr::null_mut(),
        CurrentInf: core::ptr::null_mut(),
        Section: 0,
        Line: 0,
    };
    let mut SourceFileName: PCWSTR = core::ptr::null();
    let mut SourceRootPath: PCWSTR = core::ptr::null();
    let mut SourcePath: PCWSTR = core::ptr::null();
    let mut TargetDirectory: PCWSTR = core::ptr::null();
    let mut TargetFileName: PCWSTR = core::ptr::null();
    let mut FileSrcRootPath: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut FileDstPath: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if SpInfFindFirstLine!(InfFile, SectionName, core::ptr::null(), &mut FilesContext) == 0 {
        DPRINT1!("AddSectionToCopyQueue(): Unable to find section '%S' in TXTSETUP.SIF\n", SectionName);
        return (FALSE) as u8;
    }

    loop {
        if INF_GetDataField(&mut FilesContext, 0, &mut SourceFileName) == 0 {
            DPRINT1!("INF_GetData() failed\n");
            break;
        }

        Status = GetSourceFileAndTargetLocation(
            InfFile,
            &mut FilesContext,
            SourceFileName,
            &mut SourceRootPath,
            &mut SourcePath,
            &mut TargetDirectory,
            &mut TargetFileName,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!("Could not find source and target location for file '%S'\n", SourceFileName);
            INF_FreeData(SourceFileName);

            (*pSetupData).LastErrorNumber = ERROR_TXTSETUP_SECTION;
            if !(*pSetupData).ErrorRoutine.is_none() {
                (((*pSetupData).ErrorRoutine).unwrap_unchecked())(pSetupData, SectionName);
            }
            return (FALSE) as u8;
        }

        Status = CombinePaths(
            FileSrcRootPath.as_mut_ptr(),
            (ARRAYSIZE!(FileSrcRootPath)) as _,
            2,
            (*pSetupData).SourceRootPath.Buffer,
            SourceRootPath,
        );
        UNREFERENCED_PARAMETER!(Status);
        DPRINT!("  --> FileSrcRootPath = '%S'\n", FileSrcRootPath.as_ptr());

        INF_FreeData(SourceRootPath);

        Status = CombinePaths(
            FileDstPath.as_mut_ptr(),
            (ARRAYSIZE!(FileDstPath)) as _,
            2,
            (*pSetupData).DestinationPath.Buffer,
            TargetDirectory,
        );
        UNREFERENCED_PARAMETER!(Status);
        DPRINT!("  --> FileDstPath = '%S'\n", FileDstPath.as_ptr());

        INF_FreeData(TargetDirectory);

        if SpFileQueueCopy!(
            (*pSetupData).SetupFileQueue,
            FileSrcRootPath.as_ptr(),
            SourcePath,
            SourceFileName,
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null(),
            FileDstPath.as_ptr(),
            TargetFileName,
            0,
        ) == 0
        {
            DPRINT1!("SpFileQueueCopy!() failed\n");
        }

        INF_FreeData(TargetFileName);
        INF_FreeData(SourcePath);
        INF_FreeData(SourceFileName);

        if SpInfFindNextLine!(&mut FilesContext, &mut FilesContext) == 0 {
            break;
        }
    }

    return (TRUE) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn PrepareCopyInfFile(
    mut pSetupData: PUSETUP_DATA,
    mut InfFile: HINF,
    mut SourceCabinet: PCWSTR,
) -> BOOLEAN {
    let mut Success: BOOLEAN;
    let mut Status: NTSTATUS;
    let mut DirContext: INFCONTEXT = core::mem::zeroed();
    let mut AdditionalSectionName: PWCHAR = core::ptr::null_mut();
    let mut DirKeyValue: PCWSTR = core::mem::zeroed();
    let mut PathBuffer: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if SourceCabinet.is_null() {
        Success = AddSectionToCopyQueue(
            pSetupData,
            InfFile,
            RUSTCONV_WSTR_58A47D42C9D1.as_ptr(),
            &mut (*pSetupData).DestinationPath,
        );
        if Success == 0 {
            DPRINT1!(
                b"AddSectionToCopyQueue(%S) failed!\n\0".as_ptr() as *const i8,
                RUSTCONV_WSTR_58A47D42C9D1.as_ptr()
            );
        }
        Success = AddSectionToCopyQueue(
            pSetupData,
            InfFile,
            RUSTCONV_WSTR_8917519BE0FF.as_ptr(),
            &mut (*pSetupData).DestinationPath,
        );
        if Success == 0 {
            DPRINT1!(
                b"AddSectionToCopyQueue(%S) failed!\n\0".as_ptr() as *const i8,
                RUSTCONV_WSTR_8917519BE0FF.as_ptr()
            );
            (*pSetupData).LastErrorNumber = ERROR_TXTSETUP_SECTION;
            if !(*pSetupData).ErrorRoutine.is_none() {
                (((*pSetupData).ErrorRoutine).unwrap_unchecked())(pSetupData, RUSTCONV_WSTR_8917519BE0FF.as_ptr());
            }
            return (FALSE) as u8;
        }

        {
            let mut Entry: PGENERIC_LIST_ENTRY;
            Entry = GetCurrentListEntry((*pSetupData).ComputerList);

            (*pSetupData).ComputerType = (*(GetListEntryData(Entry) as PGENENTRY)).Id;

            if ProcessComputerFiles(InfFile, (*pSetupData).ComputerType, &mut AdditionalSectionName) == 0 {
                return (FALSE) as u8;
            }
        }

        if !AdditionalSectionName.is_null()
            && AddSectionToCopyQueue(
                pSetupData,
                InfFile,
                AdditionalSectionName,
                &mut (*pSetupData).DestinationPath,
            ) == 0
        {
            (*pSetupData).LastErrorNumber = ERROR_TXTSETUP_SECTION;
            if !(*pSetupData).ErrorRoutine.is_none() {
                (((*pSetupData).ErrorRoutine).unwrap_unchecked())(pSetupData, AdditionalSectionName);
            }
            return (FALSE) as u8;
        }
    } else {
        Success = AddSectionToCopyQueueCab(
            pSetupData,
            InfFile,
            RUSTCONV_WSTR_EDFBB205E3F3.as_ptr(),
            SourceCabinet,
            &mut (*pSetupData).DestinationPath,
        );
        if Success == 0 {
            DPRINT1!(
                b"AddSectionToCopyQueueCab(%S) failed!\n\0".as_ptr() as *const i8,
                SourceCabinet
            );
            (*pSetupData).LastErrorNumber = ERROR_CABINET_SECTION;
            if !(*pSetupData).ErrorRoutine.is_none() {
                (((*pSetupData).ErrorRoutine).unwrap_unchecked())(pSetupData, RUSTCONV_WSTR_EDFBB205E3F3.as_ptr());
            }
            return (FALSE) as u8;
        }
    }

    RtlStringCchCopyW(
        PathBuffer.as_mut_ptr(),
        (ARRAYSIZE!(PathBuffer)) as _,
        (*pSetupData).DestinationPath.Buffer,
    );

    DPRINT!(b"FullPath(1): '%S'\n\0".as_ptr() as *const i8, PathBuffer.as_ptr());

    Status = SetupCreateDirectory(PathBuffer.as_ptr());
    if ((NT_SUCCESS!(Status)) == 0) && Status != STATUS_OBJECT_NAME_COLLISION {
        DPRINT1!(
            b"Creating directory '%S' failed: Status = 0x%08lx\n\0".as_ptr() as *const i8,
            PathBuffer.as_ptr(),
            Status
        );
        (*pSetupData).LastErrorNumber = ERROR_CREATE_INSTALL_DIR;
        if !(*pSetupData).ErrorRoutine.is_none() {
            (((*pSetupData).ErrorRoutine).unwrap_unchecked())(pSetupData, PathBuffer.as_ptr());
        }
        return (FALSE) as u8;
    }

    if SpInfFindFirstLine!(InfFile, RUSTCONV_WSTR_4DB9C031839F.as_ptr(), core::ptr::null(), &mut DirContext) == 0 {
        if SpInfFindFirstLine!(InfFile, RUSTCONV_WSTR_97A4EBECDF18.as_ptr(), core::ptr::null(), &mut DirContext) == 0 {
            if !SourceCabinet.is_null() {
                (*pSetupData).LastErrorNumber = ERROR_CABINET_SECTION;
            } else {
                (*pSetupData).LastErrorNumber = ERROR_TXTSETUP_SECTION;
            }

            if !(*pSetupData).ErrorRoutine.is_none() {
                (((*pSetupData).ErrorRoutine).unwrap_unchecked())(pSetupData, RUSTCONV_WSTR_4DB9C031839F.as_ptr());
            }
            return (FALSE) as u8;
        }
    }

    // C do { ... continue; ... } while (SpInfFindNextLine(...)): a `continue`
    // goes to the condition (next line), so it is `break 'body` here.
    'dirs: loop {
        'body: {
        if INF_GetData(&mut DirContext, core::ptr::null_mut(), &mut DirKeyValue) == 0 {
            DPRINT1!(b"break\n\0".as_ptr() as *const i8);
            break 'dirs;
        }

        Status = BuildFullDirectoryPath(
            (*pSetupData).DestinationRootPath.Buffer,
            (*pSetupData).InstallPath.Buffer,
            DirKeyValue,
            PathBuffer.as_mut_ptr(),
            (ARRAYSIZE!(PathBuffer)) as _,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(
                b"Could not build the full path for '%S', skipping...\n\0".as_ptr() as *const i8,
                DirKeyValue
            );
            INF_FreeData(DirKeyValue);
            break 'body;
        }

        if (*DirKeyValue.offset(0 as isize) == UNICODE_NULL)
            || (*DirKeyValue.offset(0 as isize) == 0x005Cu16
                && *DirKeyValue.offset(1 as isize) == UNICODE_NULL)
        {
        } else {
            Status = SetupCreateDirectory(PathBuffer.as_ptr());
            if ((NT_SUCCESS!(Status)) == 0) && Status != STATUS_OBJECT_NAME_COLLISION {
                INF_FreeData(DirKeyValue);
                DPRINT1!(
                    b"Creating directory '%S' failed: Status = 0x%08lx\n\0".as_ptr() as *const i8,
                    PathBuffer.as_ptr(),
                    Status
                );
                (*pSetupData).LastErrorNumber = ERROR_CREATE_DIR;
                if !(*pSetupData).ErrorRoutine.is_none() {
                    (((*pSetupData).ErrorRoutine).unwrap_unchecked())(pSetupData, PathBuffer.as_ptr());
                }
                return (FALSE) as u8;
            }
        }

        INF_FreeData(DirKeyValue);
        }

        if SpInfFindNextLine!(&mut DirContext, &mut DirContext) == 0 {
            break;
        }
    }

    return (TRUE) as u8;
}

#[no_mangle]
pub unsafe extern "system" fn PrepareFileCopy(
    pSetupData: PUSETUP_DATA,
    StatusRoutine: PFILE_COPY_STATUS_ROUTINE,
) -> BOOLEAN {
    let mut InfHandle: HINF = core::ptr::null_mut();
    let mut CabinetsContext: INFCONTEXT = INFCONTEXT {
        Inf: core::ptr::null_mut(),
        CurrentInf: core::ptr::null_mut(),
        Section: 0,
        Line: 0,
    };
    let mut CabinetName: PCWSTR = core::ptr::null();
    let mut ErrorLine: UINT = 0;
    let mut PathBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    (*pSetupData).SetupFileQueue = SpFileQueueOpen!() as PVOID;
    if (*pSetupData).SetupFileQueue.is_null() {
        (*pSetupData).LastErrorNumber = ERROR_COPY_QUEUE;
        if !(*pSetupData).ErrorRoutine.is_none() {
            (((*pSetupData).ErrorRoutine).unwrap_unchecked())(pSetupData);
        }
        return (FALSE) as u8;
    }

    if PrepareCopyInfFile(pSetupData, (*pSetupData).SetupInf, core::ptr::null()) == 0 {
        return (FALSE) as u8;
    }

    if SpInfFindFirstLine!(
        (*pSetupData).SetupInf,
        RUSTCONV_WSTR_8884DC47A3A7.as_ptr(),
        core::ptr::null(),
        &mut CabinetsContext,
    ) == 0
    {
        return (TRUE) as u8;
    }

    loop {
        if INF_GetData(
            &mut CabinetsContext,
            core::ptr::null_mut(),
            &mut CabinetName,
        ) == 0
        {
            break;
        }

        CombinePaths(
            PathBuffer.as_mut_ptr(),
            (ARRAYSIZE!(PathBuffer)) as _,
            2,
            (*pSetupData).SourcePath.Buffer,
            CabinetName,
        );

        {
            let mut ptr: PWCHAR;

            ptr = wcsrchr(PathBuffer.as_mut_ptr(), 0x005Cu16);
            if ptr.is_null() {
                ptr = PathBuffer.as_mut_ptr();
            }

            ptr = wcsrchr(ptr, 0x002Eu16);
            if ptr.is_null() {
                ptr = PathBuffer.as_mut_ptr().add(wcslen(PathBuffer.as_mut_ptr()));
            }

            wcscpy(ptr, RUSTCONV_WSTR_068BF9CAD306.as_ptr());

            InfHandle = SpInfOpenInfFile!(
                PathBuffer.as_mut_ptr(),
                core::ptr::null(),
                INF_STYLE_WIN4,
                ((*pSetupData).LanguageId) as _,
                &mut ErrorLine,
            );
        }

        if InfHandle == INVALID_HANDLE_VALUE {
            (*pSetupData).LastErrorNumber = ERROR_INVALID_CABINET_INF;
            if !(*pSetupData).ErrorRoutine.is_none() {
                (((*pSetupData).ErrorRoutine).unwrap_unchecked())(pSetupData, PathBuffer.as_mut_ptr());
            }
            return (FALSE) as u8;
        }

        if PrepareCopyInfFile(pSetupData, InfHandle, CabinetName) == 0 {
            SpInfCloseInfFile!(InfHandle);
            return (FALSE) as u8;
        }

        SpInfCloseInfFile!(InfHandle);

        if SpInfFindNextLine!(&mut CabinetsContext, &mut CabinetsContext) == 0 {
            break;
        }
    }

    return (TRUE) as u8;
}

#[no_mangle]
pub unsafe extern "system" fn DoFileCopy(
    pSetupData: *mut USETUP_DATA,
    MsgHandler: PSP_FILE_CALLBACK_W,
    Context: *mut core::ffi::c_void,
) -> i32 {
    let mut Success: i32;

    Success = SpFileQueueCommit!(
        core::ptr::null_mut(),
        (*pSetupData).SetupFileQueue,
        MsgHandler,
        Context,
    );

    SpFileQueueClose!((*pSetupData).SetupFileQueue);
    (*pSetupData).SetupFileQueue = core::ptr::null_mut();

    return Success;
}
