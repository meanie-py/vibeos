use crate::fmifs::format::Format;
use crate::fmifs::init::LoadProvider;
use crate::fmifs::init::UnloadProvider;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn Chkdsk(
    DriveRoot: PWCHAR,
    Format: PWCHAR,
    CorrectErrors: BOOLEAN,
    Verbose: BOOLEAN,
    CheckOnlyIfDirty: BOOLEAN,
    ScanDrive: BOOLEAN,
    Unused2: PVOID,
    Unused3: PVOID,
    Callback: PFMIFSCALLBACK,
) {
    let mut Provider: PIFS_PROVIDER;
    let mut usDriveRoot: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS;
    let mut Success: BOOLEAN = (FALSE as u8);
    let mut DriveName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut VolumeName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    Status = 0;

    'Quit: {
        Provider = LoadProvider(Format);
        if Provider.is_null() {
            /* Unknown file system */
            break 'Quit;
        }

        if !NT_SUCCESS!(RtlStringCchCopyW(
            DriveName.as_mut_ptr(),
            (ARRAYSIZE!(DriveName)) as _,
            DriveRoot,
        )) {
            break 'Quit;
        }

        if DriveName[(wcslen(DriveName.as_mut_ptr()) as usize)
            .wrapping_sub(1 as usize)]
            != 0x005Cu16
        {
            /* Append the trailing backslash for GetVolumeNameForVolumeMountPointW */
            if !NT_SUCCESS!(RtlStringCchCatW(
                DriveName.as_mut_ptr(),
                (ARRAYSIZE!(DriveName)) as _,
                RUSTCONV_WSTR_B7FC02907B1F.as_ptr(),
            )) {
                break 'Quit;
            }
        }

        // The C's live arm is #ifdef __REACTOS__ (the volume-GUID lookup is the #else).
        /* We really just want to use the disk drive letter */
        wcscpy(VolumeName.as_mut_ptr(), DriveName.as_mut_ptr());

        if RtlDosPathNameToNtPathName_U(
            VolumeName.as_mut_ptr(),
            core::ptr::addr_of_mut!(usDriveRoot),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            break 'Quit;
        }

        /* Trim the trailing backslash since we will work with a device object */
        usDriveRoot.Length = usDriveRoot
            .Length
            .wrapping_sub(core::mem::size_of::<WCHAR>() as USHORT);

        DPRINT!("Chkdsk() - %S\n", Format);
        Status = STATUS_SUCCESS;
        Success = (*Provider).Chkdsk.unwrap_unchecked()(
            core::ptr::addr_of_mut!(usDriveRoot),
            Callback,
            CorrectErrors,
            Verbose,
            CheckOnlyIfDirty,
            ScanDrive,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Status) as PULONG,
        );
        if Success == 0 {
            DPRINT1!("Chkdsk() failed with Status 0x%lx\n", Status);
        }

        RtlFreeUnicodeString(core::ptr::addr_of_mut!(usDriveRoot));
    }

    // Quit:
    if !Provider.is_null() {
        UnloadProvider(Provider);
    }

    /* Report result */
    (Callback.unwrap_unchecked())(
        DONE,
        Status as ULONG,
        core::ptr::addr_of_mut!(Success) as PVOID,
    );
}
