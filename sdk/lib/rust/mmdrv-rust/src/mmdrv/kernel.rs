// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_6D619999DFA2: [u16; 9] = [0x005Cu16, 0x005C, 0x002E, 0x0025, 0x006C, 0x0073, 0x0025, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_D217C45C6798: [u16; 4] = [0x0025u16, 0x006C, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WAVE_OUT_DEVICE_NAME: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b"\\\0D\0e\0v\0i\0c\0e\0\\\0W\0a\0v\0e\0O\0u\0t\0\0\0",
    )
};

pub const WAVE_IN_DEVICE_NAME: [::core::ffi::c_ushort; 15] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_ushort; 15]>(
        *b"\\\0D\0e\0v\0i\0c\0e\0\\\0W\0a\0v\0e\0I\0n\0\0\0",
    )
};

pub const MIDI_OUT_DEVICE_NAME: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b"\\\0D\0e\0v\0i\0c\0e\0\\\0M\0i\0d\0i\0O\0u\0t\0\0\0",
    )
};

pub const MIDI_IN_DEVICE_NAME: [::core::ffi::c_ushort; 15] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_ushort; 15]>(
        *b"\\\0D\0e\0v\0i\0c\0e\0\\\0M\0i\0d\0i\0I\0n\0\0\0",
    )
};

pub const AUX_DEVICE_NAME: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"\\\0D\0e\0v\0i\0c\0e\0\\\0M\0M\0A\0u\0x\0\0\0",
    )
};
#[no_mangle]
pub unsafe extern "C" fn CobbleDeviceName(
    mut device_type: DeviceType,
    mut device_id: UINT,
    mut out_device_name: PWCHAR,
) -> MMRESULT {
    let mut base_device_name: [WCHAR; (MAX_DEVICE_NAME_LENGTH) as _] = [0; (MAX_DEVICE_NAME_LENGTH) as _];

    /* Work out the base name from the device type */

    match device_type as ::core::ffi::c_uint {
        0 => {
            /* WaveOutDevice */
            wsprintf(base_device_name.as_mut_ptr(), RUSTCONV_WSTR_D217C45C6798.as_ptr(), WAVE_OUT_DEVICE_NAME);
        }
        1 => {
            /* WaveInDevice */
            wsprintf(base_device_name.as_mut_ptr(), RUSTCONV_WSTR_D217C45C6798.as_ptr(), WAVE_IN_DEVICE_NAME);
        }
        2 => {
            /* MidiOutDevice */
            wsprintf(base_device_name.as_mut_ptr(), RUSTCONV_WSTR_D217C45C6798.as_ptr(), MIDI_OUT_DEVICE_NAME);
        }
        3 => {
            /* MidiInDevice */
            wsprintf(base_device_name.as_mut_ptr(), RUSTCONV_WSTR_D217C45C6798.as_ptr(), MIDI_IN_DEVICE_NAME);
        }
        4 => {
            /* AuxDevice */
            wsprintf(base_device_name.as_mut_ptr(), RUSTCONV_WSTR_D217C45C6798.as_ptr(), AUX_DEVICE_NAME);
        }
        _ => {
            return MMSYSERR_BADDEVICEID as MMRESULT;
        }
    }

    /* Now append the device number, removing the leading \Device */

    wsprintf(
        out_device_name,
        RUSTCONV_WSTR_6D619999DFA2.as_ptr(),
        base_device_name
            .as_mut_ptr()
            .offset(strlen(b"\\Device\0".as_ptr() as *const ::core::ffi::c_char) as isize),
        device_id,
    );

    return MMSYSERR_NOERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn OpenKernelDevice(mut device_type: DeviceType, mut device_id: UINT, mut access: DWORD, mut handle: *mut HANDLE) -> MMRESULT {
    let mut result: MMRESULT;
    let mut device_name: [WCHAR; (MAX_DEVICE_NAME_LENGTH) as _] = [0; (MAX_DEVICE_NAME_LENGTH) as _];
    let mut open_flags: DWORD = 0 as DWORD;

    ASSERT!(!handle.is_null());

    /* Glue the base device name and the ID together */

    result = CobbleDeviceName(device_type, device_id, core::ptr::addr_of_mut!(device_name) as PWCHAR);

    DPRINT!("Opening kernel device %ls\n", device_name);

    if result != MMSYSERR_NOERROR as MMRESULT {
        return result;
    }

    /* We want overlapped I/O when writing */

    if access != GENERIC_READ as DWORD {
        open_flags = FILE_FLAG_OVERLAPPED as DWORD;
    }

    /* Now try opening... */

    *handle = CreateFile(
        core::ptr::addr_of_mut!(device_name) as *mut WCHAR as LPCWSTR,
        access,
        FILE_SHARE_WRITE as DWORD,
        core::ptr::null_mut(),
        OPEN_EXISTING as DWORD,
        open_flags,
        core::ptr::null_mut(),
    );

    if *handle == INVALID_HANDLE_VALUE {
        return ErrorToMmResult(GetLastError() as UINT);
    }

    return MMSYSERR_NOERROR as MMRESULT;
}

#[no_mangle]
pub unsafe extern "C" fn CloseKernelDevice(device_handle: HANDLE) {
    CloseHandle(device_handle);
}

#[no_mangle]
pub unsafe extern "C" fn SetDeviceData(device_handle: HANDLE, ioctl: DWORD, input_buffer: PBYTE, buffer_size: DWORD) -> MMRESULT {
    // DPRINT("SetDeviceData\n");
    0
}
#[no_mangle]
pub unsafe extern "C" fn GetDeviceData(device_handle: HANDLE, ioctl: DWORD, output_buffer: PBYTE, buffer_size: DWORD) -> MMRESULT {
    let mut overlap: OVERLAPPED = ::core::mem::zeroed();
    let mut bytes_returned: DWORD = 0;
    let mut success: BOOL = 0;
    let mut transfer: DWORD = 0;

    DPRINT!("GetDeviceData\n");

    memset(
        core::ptr::addr_of_mut!(overlap) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<OVERLAPPED>(),
    );

    overlap.hEvent = CreateEvent(
        core::ptr::null_mut(),
        (FALSE as i32),
        (FALSE as i32),
        core::ptr::null_mut(),
    );

    if overlap.hEvent.is_null() {
        return MMSYSERR_NOMEM;
    }

    success = DeviceIoControl(
        device_handle,
        ioctl,
        core::ptr::null_mut(),
        0,
        (output_buffer).cast(),
        buffer_size,
        core::ptr::addr_of_mut!(bytes_returned),
        core::ptr::addr_of_mut!(overlap),
    );

    if success == 0 {
        if GetLastError() == ERROR_IO_PENDING {
            if GetOverlappedResult(
                device_handle,
                core::ptr::addr_of_mut!(overlap),
                core::ptr::addr_of_mut!(transfer),
                (TRUE as i32),
            ) == 0
            {
                CloseHandle(overlap.hEvent);
                return ErrorToMmResult(GetLastError());
            }
        } else {
            CloseHandle(overlap.hEvent);
            return ErrorToMmResult(GetLastError());
        }
    }

    while TRUE != 0 {
        SetEvent(overlap.hEvent);

        if WaitForSingleObjectEx(
            overlap.hEvent,
            0,
            (TRUE) as BOOL,
        ) != WAIT_IO_COMPLETION
        {
            break;
        }
    }

    CloseHandle(overlap.hEvent);

    return MMSYSERR_NOERROR;
}
