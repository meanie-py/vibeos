use crate::usetup::progress::CreateProgressBar;
use crate::usetup::progress::DestroyProgressBar;
use crate::usetup::mui::MUIGetString;
use crate::usetup::progress::ProgressSetStep;
use crate::usetup::progress::ProgressSetStepCount;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ProgressBar: PPROGRESSBAR = ::core::ptr::null::<_PROGRESSBAR>() as *mut _PROGRESSBAR;

unsafe extern "system" fn FormatCallback(
    Command: CALLBACKCOMMAND,
    Modifier: ULONG,
    Argument: PVOID,
) -> BOOLEAN {
    match Command as u32 {
        PROGRESS => {
            let Percent: PULONG = Argument as PULONG;
            DPRINT!(b"%lu percent completed\n\0".as_ptr() as *const i8, *Percent);
            ProgressSetStep(ProgressBar, *Percent);
        }
        DONE => {
            DPRINT!(b"Done\n\0".as_ptr() as *const i8);
        }
        _ => {
            DPRINT!(b"Unknown callback %lu\n\0".as_ptr() as *const i8, Command as ULONG);
        }
    }

    return (TRUE) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn StartFormat(
    FmtInfo: *mut FORMAT_VOLUME_INFO,
    SelectedFileSystem: *mut FILE_SYSTEM_ITEM,
) {
    (*FmtInfo).FileSystemName = (*SelectedFileSystem).FileSystem;
    (*FmtInfo).MediaFlag = FMIFS_HARDDISK;
    (*FmtInfo).Label = core::ptr::null_mut();
    (*FmtInfo).QuickFormat = (*SelectedFileSystem).QuickFormat;
    (*FmtInfo).ClusterSize = 0;
    (*FmtInfo).Callback = Some(FormatCallback);

    ProgressBar = CreateProgressBar(
        6,
        yScreen - 14,
        xScreen - 7,
        yScreen - 10,
        10,
        24,
        (TRUE) as BOOLEAN,
        MUIGetString(STRING_FORMATTINGPART),
    );

    ProgressSetStepCount(ProgressBar, 100);
}
#[no_mangle]
pub unsafe extern "C" fn EndFormat(Status: NTSTATUS) {
    DestroyProgressBar(ProgressBar);
    ProgressBar = core::ptr::null_mut();

    DPRINT!(
        b"FormatPartition() finished with status 0x%08lx\n\0".as_ptr() as *const i8,
        Status,
    );
}

unsafe extern "system" fn ChkdskCallback(
    Command: CALLBACKCOMMAND,
    Modifier: ULONG,
    Argument: PVOID,
) -> BOOLEAN {
    match Command as u32 {
        _ => {
            DPRINT!(b"Unknown callback %lu\n\0".as_ptr() as *const i8, Command as u32);
        }
    }

    return (TRUE) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn StartCheck(ChkInfo: PCHECK_VOLUME_INFO) {
    (*ChkInfo).FixErrors = (TRUE) as u8;
    (*ChkInfo).Verbose = (FALSE) as u8;
    (*ChkInfo).CheckOnlyIfDirty = (TRUE) as u8;
    (*ChkInfo).ScanDrive = (FALSE) as u8;
    (*ChkInfo).Callback = Some(ChkdskCallback);

    ProgressBar = CreateProgressBar(
        6,
        yScreen - 14,
        xScreen - 7,
        yScreen - 10,
        10,
        24,
        (TRUE) as BOOLEAN,
        MUIGetString(STRING_CHECKINGDISK),
    );

    ProgressSetStepCount(ProgressBar, 100);
}
#[no_mangle]
pub unsafe extern "C" fn EndCheck(Status: NTSTATUS) {
    DestroyProgressBar(ProgressBar);
    ProgressBar = core::ptr::null_mut();

    DPRINT!(
        b"ChkdskPartition() finished with status 0x%08lx\n\0".as_ptr() as *const i8,
        Status,
    );
}
