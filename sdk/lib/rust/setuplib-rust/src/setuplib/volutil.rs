use crate::setuplib::fsrec::InferFileSystem;
use crate::setuplib::devutils::pOpenDevice;
use crate::setuplib::devutils::pOpenDeviceEx;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn MountVolume(
    Volume: PVOLINFO,
    MbrPartitionType: UCHAR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut VolumeHandle: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();

    if (*Volume).FileSystem[0] != 0 {
        return STATUS_SUCCESS;
    }

    VolumeHandle = core::ptr::null_mut::<core::ffi::c_void>();
    Status = pOpenDevice(
        (*Volume).DeviceName.as_ptr(),
        core::ptr::addr_of_mut!(VolumeHandle),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(b"pOpenDevice() failed, Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
        (*Volume).DriveLetter = UNICODE_NULL;
        (*Volume).VolumeLabel[0 as usize] = UNICODE_NULL;
        (*Volume).FileSystem[0 as usize] = UNICODE_NULL;
        return Status;
    }

    Status = InferFileSystem(
        core::ptr::null_mut::<u16>(),
        VolumeHandle,
        (*Volume).FileSystem.as_mut_ptr(),
        (core::mem::size_of_val(&(*Volume).FileSystem)) as SIZE_T,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(b"InferFileSystem() failed, Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
    }

    if (*Volume).FileSystem[0] != 0 {
        if IsUnformatted!(Volume) {
            if (MbrPartitionType as i32) as u32 == PARTITION_FAT_12
                || (MbrPartitionType as i32) as u32 == PARTITION_FAT_16
                || (MbrPartitionType as i32) as u32 == PARTITION_HUGE
                || (MbrPartitionType as i32) as u32 == PARTITION_XINT13
                || (MbrPartitionType as i32) as u32 == PARTITION_FAT32
                || (MbrPartitionType as i32) as u32 == PARTITION_FAT32_XINT13
            {
                // The volume is unformatted
            } else {
                NtClose(VolumeHandle);
                VolumeHandle = core::ptr::null_mut::<core::ffi::c_void>();
                DismountVolume(Volume, (TRUE) as BOOLEAN);
                (*Volume).FileSystem[0 as usize] = UNICODE_NULL;
            }
        }
    }

    if !VolumeHandle.is_null() {
        let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
        let _ = _countof!((*Volume).VolumeLabel);
        #[repr(C)]
        struct LabelInfo {
            VolumeInfo: FILE_FS_VOLUME_INFORMATION,
            Data: [u16; 33],
        }
        let mut LabelInfo = core::mem::zeroed::<LabelInfo>();

        Status = NtQueryVolumeInformationFile(
            VolumeHandle,
            core::ptr::addr_of_mut!(IoStatusBlock),
            core::ptr::addr_of_mut!(LabelInfo) as *mut _,
            core::mem::size_of::<LabelInfo>() as u32,
            FileFsVolumeInformation,
        );
        if NT_SUCCESS!(Status) != 0 {
            RtlStringCbCopyNW(
                (*Volume).VolumeLabel.as_mut_ptr(),
                core::mem::size_of_val(&(*Volume).VolumeLabel),
                LabelInfo.VolumeInfo.VolumeLabel.as_ptr(),
                LabelInfo.VolumeInfo.VolumeLabelLength as usize,
            );
        } else {
            DPRINT1!(b"NtQueryVolumeInformationFile() failed, Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
        }
    }

    if !VolumeHandle.is_null() {
        NtClose(VolumeHandle);
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn DismountVolume(
    Volume: PVOLINFO,
    Force: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut LockStatus: NTSTATUS = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut VolumeHandle: HANDLE = core::ptr::null_mut();

    if *core::ptr::addr_of!((*Volume).FileSystem).cast::<WCHAR>() == 0 {
        return STATUS_SUCCESS;
    }

    Status = pOpenDeviceEx(
        core::ptr::addr_of!((*Volume).DeviceName).cast::<WCHAR>(),
        &mut VolumeHandle,
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"ERROR: Cannot open volume %S for dismounting! (Status 0x%lx)\n\0".as_ptr() as *const i8,
            core::ptr::addr_of!((*Volume).DeviceName).cast::<WCHAR>(),
            Status,
        );
        return Status;
    }

    LockStatus = NtFsControlFile(
        VolumeHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &raw mut IoStatusBlock,
        FSCTL_LOCK_VOLUME,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
    );
    if ((NT_SUCCESS!(LockStatus)) == 0) {
        DPRINT1!(
            b"WARNING: Failed to lock volume (Status 0x%lx)\n\0".as_ptr() as *const i8,
            LockStatus,
        );
    }

    Status = STATUS_ACCESS_DENIED;
    if NT_SUCCESS!(LockStatus) != 0 || Force != 0 {
        Status = NtFsControlFile(
            VolumeHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &raw mut IoStatusBlock,
            FSCTL_DISMOUNT_VOLUME,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(
                b"Failed to unmount volume (Status 0x%lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
    }

    if NT_SUCCESS!(LockStatus) != 0 {
        LockStatus = NtFsControlFile(
            VolumeHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &raw mut IoStatusBlock,
            FSCTL_UNLOCK_VOLUME,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        if ((NT_SUCCESS!(LockStatus)) == 0) {
            DPRINT1!(
                b"Failed to unlock volume (Status 0x%lx)\n\0".as_ptr() as *const i8,
                LockStatus,
            );
        }
    }

    NtClose(VolumeHandle);

    if NT_SUCCESS!(Status) != 0 {
        (*Volume).DriveLetter = UNICODE_NULL;
        (*Volume).VolumeLabel[0 as usize] = UNICODE_NULL;
        (*Volume).FileSystem[0 as usize] = UNICODE_NULL;
    }

    return Status;
}
