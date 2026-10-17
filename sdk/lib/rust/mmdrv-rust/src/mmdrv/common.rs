use crate::mmdrv::session::CallSessionThread;
use crate::mmdrv::kernel::CloseKernelDevice;
use crate::mmdrv::session::CreateSession;
use crate::mmdrv::session::DestroySession;
use crate::mmdrv::mme::NotifyClient;
use crate::mmdrv::kernel::OpenKernelDevice;
use crate::mmdrv::session::StartSessionThread;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn ErrorToMmResult(mut error_code: UINT) -> MMRESULT {
    match error_code {
        NO_ERROR | ERROR_IO_PENDING => return MMSYSERR_NOERROR as MMRESULT,
        ERROR_BUSY => return MMSYSERR_ALLOCATED as MMRESULT,
        ERROR_NOT_SUPPORTED | ERROR_INVALID_FUNCTION => return MMSYSERR_NOTSUPPORTED as MMRESULT,
        ERROR_NOT_ENOUGH_MEMORY => return MMSYSERR_NOMEM as MMRESULT,
        ERROR_ACCESS_DENIED => return MMSYSERR_BADDEVICEID as MMRESULT,
        ERROR_INSUFFICIENT_BUFFER => return MMSYSERR_INVALPARAM as MMRESULT,
        _ => {}
    }

    /* If all else fails, it's just a plain old error */

    return MMSYSERR_ERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn GetDeviceCount(device_type: DeviceType) -> DWORD {
    let mut index: UINT = 0;
    let mut handle: HANDLE = core::ptr::null_mut();

    /* Cycle through devices until an error occurs */

    while OpenKernelDevice(device_type, index, GENERIC_READ, core::ptr::addr_of_mut!(handle)) == MMSYSERR_NOERROR
    {
        CloseHandle(handle);
        index = index.wrapping_add(1);
    }

    DPRINT!("Found %d devices of type %d\n", index, device_type);

    return index as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn GetDeviceCapabilities(
    device_type: DeviceType,
    device_id: UINT,
    capabilities: DWORD_PTR,
    capabilities_size: DWORD,
) -> DWORD {
    let mut result: MMRESULT = 0;
    let mut ioctl: DWORD = 0;
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut bytes_returned: DWORD = 0;
    let mut device_io_result: BOOL = 0;

    ASSERT!(capabilities);

    /* Choose the right IOCTL for the job */

    if IsWaveDevice!(device_type) != false {
        ioctl = (IOCTL_WAVE_GET_CAPABILITIES as u32);
    } else if IsMidiDevice!(device_type) != false {
        ioctl = (IOCTL_MIDI_GET_CAPABILITIES as u32);
    } else if IsAuxDevice!(device_type) != false {
        return MMSYSERR_NOTSUPPORTED; /* TODO */
    } else {
        return MMSYSERR_NOTSUPPORTED;
    }

    result = OpenKernelDevice(
        device_type,
        device_id,
        GENERIC_READ,
        core::ptr::addr_of_mut!(handle),
    );

    if result != MMSYSERR_NOERROR {
        DPRINT!("Failed to open kernel device\n");
        return result;
    }

    device_io_result = DeviceIoControl(
        handle,
        ioctl,
        core::ptr::null_mut(),
        0,
        capabilities as LPVOID,
        capabilities_size,
        core::ptr::addr_of_mut!(bytes_returned),
        core::ptr::null_mut(),
    );

    /* Translate result */

    if device_io_result != 0 {
        result = MMSYSERR_NOERROR;
    } else {
        result = ErrorToMmResult(GetLastError() as UINT);
    }

    /* Clean up and return */

    CloseKernelDevice(handle);

    return result;
}

#[no_mangle]
pub unsafe extern "C" fn OpenDevice(
    device_type: DeviceType,
    device_id: UINT,
    open_descriptor: PVOID,
    flags: DWORD,
    private_handle: DWORD_PTR,
) -> DWORD {
    let mut session_info: *mut SessionInfo = core::ptr::null_mut::<SessionInfo>();
    let mut result: MMRESULT = 0;
    let mut message: DWORD = 0;

    /* This will automatically check for duplicate sessions */
    result = CreateSession(device_type, device_id, core::ptr::addr_of_mut!(session_info));

    if result != MMSYSERR_NOERROR as MMRESULT {
        DPRINT!("Couldn't allocate session info\n");
        return result as DWORD;
    }

    result = OpenKernelDevice(
        device_type,
        device_id,
        GENERIC_READ,
        core::ptr::addr_of_mut!((*session_info).kernel_device_handle),
    );

    if result != MMSYSERR_NOERROR as MMRESULT {
        DPRINT!("Failed to open kernel device\n");
        DestroySession(session_info);
        return result as DWORD;
    }

    /* Set common session data */

    (*session_info).flags = flags;

    /* Set wave/MIDI specific data */

    if IsWaveDevice!(device_type) != false {
        let mut wave_open_desc: LPWAVEOPENDESC = open_descriptor as LPWAVEOPENDESC;
        (*session_info).callback = (*wave_open_desc).dwCallback as DWORD_PTR;
        (*session_info).__bindgen_anon_1.mme_wave_handle = (*wave_open_desc).hWave;
        (*session_info).app_user_data = (*wave_open_desc).dwInstance as DWORD_PTR;
    } else {
        DPRINT!("Only wave devices are supported at present!\n");
        DestroySession(session_info);
        return MMSYSERR_NOTSUPPORTED as DWORD;
    }

    /* Start the processing thread */

    result = StartSessionThread(session_info);

    if result != MMSYSERR_NOERROR as MMRESULT {
        DestroySession(session_info);
        return result as DWORD;
    }

    /* Store the session info */

    *(private_handle as *mut *mut SessionInfo) = session_info;

    /* Send the right message */

    message = (if device_type as ::core::ffi::c_uint
        == WaveOutDevice as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        WOM_OPEN
    } else if device_type as ::core::ffi::c_uint
        == WaveInDevice as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        WIM_OPEN
    } else if device_type as ::core::ffi::c_uint
        == MidiOutDevice as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        MOM_OPEN
    } else if device_type as ::core::ffi::c_uint
        == MidiInDevice as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        MIM_OPEN
    } else {
        0xFFFFFFFF
    }) as DWORD;

    NotifyClient(session_info, message, 0, 0);

    return MMSYSERR_NOERROR as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn CloseDevice(private_handle: DWORD_PTR) -> DWORD {
    let mut result: MMRESULT;
    let session_info: *mut SessionInfo = private_handle as *mut SessionInfo;
    /* TODO: Maybe this is best off inside the playback thread? */

    ASSERT!(session_info);

    result = CallSessionThread(session_info, WODM_CLOSE, core::ptr::null_mut());

    if result == MMSYSERR_NOERROR {
        /* TODO: Wait for it to be safe to terminate */

        CloseKernelDevice((*session_info).kernel_device_handle);

        DestroySession(session_info);
    }

    return result as DWORD;
}
