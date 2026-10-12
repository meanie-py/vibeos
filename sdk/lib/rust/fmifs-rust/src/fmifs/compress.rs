use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn EnableVolumeCompression(
    mut DriveRoot: PWCHAR,
    mut Compression: USHORT,
) -> BOOLEAN {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut RetBytes: DWORD = 0;
    let mut Ret: BOOL = 0;
    hFile = CreateFileW(
        DriveRoot as LPCWSTR,
        (FILE_READ_DATA | FILE_WRITE_DATA) as DWORD,
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as DWORD,
        core::ptr::null_mut(),
        OPEN_EXISTING as DWORD,
        FILE_FLAG_BACKUP_SEMANTICS as DWORD,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        return FALSE as BOOLEAN;
    }

    Ret = DeviceIoControl(
        hFile,
        FSCTL_SET_COMPRESSION as DWORD,
        core::ptr::addr_of_mut!(Compression) as LPVOID,
        core::mem::size_of::<USHORT>() as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::addr_of_mut!(RetBytes),
        core::ptr::null_mut(),
    );

    CloseHandle(hFile);

    return (Ret != 0) as BOOLEAN;
}
