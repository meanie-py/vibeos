use crate::mmebuddy::utility::Win32ErrorToMmResult;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_3B8717DB6281: [u16; 34] = [
    0x004Fu16, 0x0070, 0x0065, 0x006E, 0x004B, 0x0065, 0x0072, 0x006E, 0x0065, 0x006C,
    0x0053, 0x006F, 0x0075, 0x006E, 0x0064, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063,
    0x0065, 0x0042, 0x0079, 0x004E, 0x0061, 0x006D, 0x0065, 0x003A, 0x0020, 0x0025,
    0x0077, 0x0053, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_5CF0A8256D62: [u16; 45] = [
    0x0054u16, 0x0072, 0x0061, 0x006E, 0x0073, 0x0066, 0x0065, 0x0072, 0x0072, 0x0065,
    0x0064, 0x0020, 0x0025, 0x0064, 0x0020, 0x0062, 0x0079, 0x0074, 0x0065, 0x0073,
    0x0020, 0x0069, 0x006E, 0x0020, 0x0053, 0x0079, 0x006E, 0x0063, 0x0020, 0x006F,
    0x0076, 0x0065, 0x0072, 0x006C, 0x0061, 0x0070, 0x0070, 0x0065, 0x0064, 0x0020,
    0x0049, 0x002F, 0x004F, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_B1B20D8DF24F: [u16; 32] = [
    0x0043u16, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0046, 0x0069, 0x006C, 0x0065,
    0x0020, 0x0066, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x002D, 0x0020, 0x0077,
    0x0069, 0x006E, 0x0065, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0025, 0x0064,
    0x000A, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn OpenKernelSoundDeviceByName(
    DevicePath: PWSTR,
    ReadOnly: BOOLEAN,
    Handle: PHANDLE,
) -> MMRESULT {
    let mut AccessRights: DWORD;

    // VALIDATE_MMSYS_PARAMETER!( DevicePath );
    if DevicePath.is_null() {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }
    // VALIDATE_MMSYS_PARAMETER!( Handle );
    if Handle.is_null() {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }

    AccessRights = if ReadOnly != 0 {
        GENERIC_READ
    } else {
        GENERIC_READ | GENERIC_WRITE
    };

    SND_TRACE!(RUSTCONV_WSTR_3B8717DB6281.as_ptr(), DevicePath);
    *Handle = CreateFile(
        DevicePath,
        AccessRights,
        FILE_SHARE_WRITE, /* FIXME? Should be read also? */
        core::ptr::null_mut(),
        OPEN_EXISTING,
        if ReadOnly != 0 {
            0
        } else {
            FILE_FLAG_OVERLAPPED
        },
        core::ptr::null_mut(),
    );

    if *Handle == INVALID_HANDLE_VALUE {
        SND_ERR!(RUSTCONV_WSTR_B1B20D8DF24F.as_ptr(), GetLastError());
        return Win32ErrorToMmResult(GetLastError());
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn CloseKernelSoundDevice(mut Handle: HANDLE) -> MMRESULT {
    VALIDATE_MMSYS_PARAMETER!(Handle);
    if Handle.is_null() {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }

    CloseHandle(Handle);

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn SyncOverlappedDeviceIoControl(
    Handle: HANDLE,
    IoControlCode: DWORD,
    InBuffer: LPVOID,
    InBufferSize: DWORD,
    OutBuffer: LPVOID,
    OutBufferSize: DWORD,
    BytesTransferred: LPDWORD,
) -> MMRESULT {
    let mut Overlapped: OVERLAPPED = core::mem::zeroed();
    let mut IoResult: BOOLEAN = 0;
    let mut Transferred: DWORD = 0;

    /* Overlapped I/O is done here - this is used for waiting for completion */
    ZeroMemory!(
        core::ptr::addr_of_mut!(Overlapped) as *mut core::ffi::c_void,
        core::mem::size_of::<OVERLAPPED>(),
    );
    Overlapped.hEvent = CreateEvent(
        core::ptr::null_mut(),
        (FALSE as i32),
        (FALSE as i32),
        core::ptr::null(),
    );

    if Overlapped.hEvent.is_null() {
        return Win32ErrorToMmResult(GetLastError());
    }

    /* Talk to the device */
    IoResult = (DeviceIoControl(
        Handle,
        IoControlCode,
        InBuffer,
        InBufferSize,
        OutBuffer,
        OutBufferSize,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Overlapped),
    )) as u8;

    /* If failure occurs, make sure it's not just due to the overlapped I/O */
    if IoResult == 0 {
        if GetLastError() != ERROR_IO_PENDING {
            CloseHandle(Overlapped.hEvent);
            return Win32ErrorToMmResult(GetLastError());
        }
    }

    /* Wait for the I/O to complete */
    IoResult = (GetOverlappedResult(
        Handle,
        core::ptr::addr_of_mut!(Overlapped),
        core::ptr::addr_of_mut!(Transferred),
        (TRUE as i32),
    )) as u8;

    /* Don't need this any more */
    CloseHandle(Overlapped.hEvent);

    if IoResult == 0 {
        return Win32ErrorToMmResult(GetLastError());
    }

    SND_TRACE!(RUSTCONV_WSTR_5CF0A8256D62.as_ptr(), Transferred);

    if !BytesTransferred.is_null() {
        *BytesTransferred = Transferred;
    }

    return MMSYSERR_NOERROR;
}
