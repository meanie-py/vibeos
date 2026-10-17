use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn NotifyClient(session_info: *mut SessionInfo, message: DWORD, parameter1: DWORD_PTR, parameter2: DWORD_PTR) -> BOOL {
    return DriverCallback(
        (*session_info).callback as DWORD,
        (HIWORD!((*session_info).flags as DWORD)) as UINT,
        (*session_info).__bindgen_anon_1.mme_handle,
        message,
        (*session_info).app_user_data as DWORD,
        parameter1 as DWORD,
        parameter2 as DWORD,
    );
}

#[no_mangle]
pub unsafe extern "system" fn wodMessage(
    device_id: UINT,
    message: UINT,
    private_handle: DWORD_PTR,
    parameter1: DWORD_PTR,
    parameter2: DWORD_PTR,
) -> DWORD {
    match message {
        WODM_GETNUMDEVS => {
            DPRINT!("WODM_GETNUMDEVS\n");
            return GetDeviceCount(WaveOutDevice);
        }
        WODM_GETDEVCAPS => {
            DPRINT!("WODM_GETDEVCAPS\n");
            return GetDeviceCapabilities(
                WaveOutDevice,
                device_id,
                parameter1,
                parameter2 as DWORD,
            );
        }
        WODM_OPEN => {
            let mut open_desc: *mut WAVEOPENDESC = parameter1 as *mut WAVEOPENDESC;
            DPRINT!("WODM_OPEN\n");
            if parameter2 != 0 && WAVE_FORMAT_QUERY != 0 {
                return QueryWaveFormat(WaveOutDevice, (*open_desc).lpFormat as PVOID);
            } else {
                return OpenDevice(
                    WaveOutDevice,
                    device_id,
                    open_desc as PVOID,
                    parameter2 as DWORD,
                    private_handle,
                );
            }
        }
        WODM_CLOSE => {
            DPRINT!("WODM_CLOSE\n");
            return CloseDevice(private_handle);
        }
        WODM_WRITE => {
            DPRINT!("WODM_WRITE\n");
            return WriteWaveBuffer(
                private_handle,
                parameter1 as PWAVEHDR,
                parameter2 as DWORD,
            );
        }
        WODM_PAUSE => {
            DPRINT!("WODM_PAUSE\n");
            return HandleBySessionThread(private_handle, message as DWORD_PTR, 0 as DWORD_PTR);
        }
        WODM_RESTART => {
            DPRINT!("WODM_RESTART\n");
            return HandleBySessionThread(private_handle, message as DWORD_PTR, 0 as DWORD_PTR);
        }
        WODM_RESET => {
            DPRINT!("WODM_RESET\n");
            return HandleBySessionThread(private_handle, message as DWORD_PTR, 0 as DWORD_PTR);
        }
        WODM_BREAKLOOP => {
            DPRINT!("WODM_BREAKLOOP\n");
            return HandleBySessionThread(private_handle, message as DWORD_PTR, 0 as DWORD_PTR);
        }
        _ => {}
    }

    DPRINT!("Unsupported message\n");
    return MMSYSERR_NOTSUPPORTED as DWORD;
}
