use crate::fmifs::init::LoadProvider;
use crate::fmifs::init::UnloadProvider;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_A953DDDD78B1: [u16; 4] = [0x0046u16, 0x0041, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn Format(
    DriveRoot: PWCHAR,
    MediaFlag: FMIFS_MEDIA_FLAG,
    Format: PWCHAR,
    Label: PWCHAR,
    QuickFormat: BOOLEAN,
    Callback: PFMIFSCALLBACK,
) {
    FormatEx(
        DriveRoot,
        MediaFlag,
        Format,
        Label,
        QuickFormat,
        0,
        Callback,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FormatEx(
    DriveRoot: PWCHAR,
    MediaFlag: FMIFS_MEDIA_FLAG,
    Format: PWCHAR,
    Label: PWCHAR,
    QuickFormat: BOOLEAN,
    ClusterSize: ULONG,
    Callback: PFMIFSCALLBACK,
) {
    let mut Provider: PIFS_PROVIDER = core::ptr::null_mut();
    let mut usDriveRoot: _UNICODE_STRING = core::mem::zeroed();
    let mut usLabel: _UNICODE_STRING = core::mem::zeroed();
    let mut Success: BOOLEAN = (FALSE as u8);
    let mut BackwardCompatible: BOOLEAN = (FALSE as u8); // Default to latest FS versions.
    let mut MediaType: MEDIA_TYPE;
    let mut DriveName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut VolumeName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    Provider = LoadProvider(Format);
    'quit: {
        if Provider.is_null() {
            /* Unknown file system */
            break 'quit;
        }

        if !NT_SUCCESS!(RtlStringCchCopyW(
            &raw mut DriveName as *mut WCHAR,
            (ARRAYSIZE!(DriveName)) as _,
            DriveRoot,
        )) {
            break 'quit;
        }

        if DriveName[(wcslen(&raw mut DriveName as *mut WCHAR) as usize)
            .wrapping_sub(1 as usize)]
            != 0x005Cu16
        {
            /* Append the trailing backslash for GetVolumeNameForVolumeMountPointW */
            if !NT_SUCCESS!(RtlStringCchCatW(
                &raw mut DriveName as *mut WCHAR,
                (ARRAYSIZE!(DriveName)) as _,
                RUSTCONV_WSTR_B7FC02907B1F.as_ptr(),
            )) {
                break 'quit;
            }
        }

        if GetVolumeNameForVolumeMountPointW(
            &raw mut DriveName as *mut WCHAR,
            &raw mut VolumeName as *mut WCHAR,
            ARRAYSIZE!(VolumeName),
        ) == 0
        {
            /* Couldn't get a volume GUID path, try formatting using a parameter provided path */
            DPRINT1!("Couldn't get a volume GUID path for drive %S\n", DriveName);
            wcscpy(
                &raw mut VolumeName as *mut WCHAR,
                &raw mut DriveName as *mut WCHAR,
            );
        }

        if RtlDosPathNameToNtPathName_U(
            &raw mut VolumeName as *mut WCHAR,
            &raw mut usDriveRoot,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            break 'quit;
        }

        /* Trim the trailing backslash since we will work with a device object */
        usDriveRoot.Length -= core::mem::size_of::<WCHAR>() as USHORT;

        RtlInitUnicodeString(&raw mut usLabel, Label);

        /* Set the BackwardCompatible flag in case we format with older FAT12/16 */
        if _wcsicmp(Format, RUSTCONV_WSTR_A953DDDD78B1.as_ptr()) == 0 {
            BackwardCompatible = (TRUE as u8);
        }
        // else if (wcsicmp(Format, L"FAT32") == 0)
        // BackwardCompatible = FALSE;

        /* Convert the FMIFS MediaFlag to a NT MediaType */
        // FIXME: Actually covert all the possible flags.
        match MediaFlag {
            FMIFS_FLOPPY => {
                MediaType = F5_320_1024; // FIXME: This is hardfixed!
            }
            FMIFS_REMOVABLE => {
                MediaType = RemovableMedia;
            }
            FMIFS_HARDDISK => {
                MediaType = FixedMedia;
            }
            _ => {
                DPRINT1!(
                    "Unknown FMIFS MediaFlag %d, converting 1-to-1 to NT MediaType\n",
                    MediaFlag
                );
                MediaType = MediaFlag as MEDIA_TYPE;
            }
        }

        DPRINT!("Format() - %S\n", Format);
        Success = (*Provider).Format.unwrap_unchecked()(
            &raw mut usDriveRoot,
            Callback,
            QuickFormat,
            BackwardCompatible,
            MediaType,
            &raw mut usLabel,
            ClusterSize,
        );
        if Success == 0 {
            DPRINT1!("Format() failed\n");
        }

        RtlFreeUnicodeString(&raw mut usDriveRoot);
    }

    // Quit:
    if !Provider.is_null() {
        UnloadProvider(Provider);
    }

    /* Report result */
    (Callback.unwrap_unchecked())(
        DONE,
        0,
        &raw mut Success as PVOID,
    );
}
