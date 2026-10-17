use crate::mmebuddy::reentrancy::AcquireEntrypointMutex;
use crate::mmebuddy::devicelist::GetSoundDeviceCount;
use crate::mmebuddy::mmewrap::MmeCloseDevice;
use crate::mmebuddy::mmewrap::MmeGetDeviceInterfaceString;
use crate::mmebuddy::mmewrap::MmeGetSoundDeviceCapabilities;
use crate::mmebuddy::mmewrap::MmeOpenDevice;
use crate::mmebuddy::mmewrap::MmeResetWavePlayback;
use crate::mmebuddy::mmewrap::MmeSetState;
use crate::mmebuddy::reentrancy::ReleaseEntrypointMutex;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_6AD3A0D7893E: [u16; 34] = [
    0x0077u16, 0x0069, 0x0064, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x0072, 0x0065, 0x0074, 0x0075, 0x0072, 0x006E, 0x0069, 0x006E, 0x0067,
    0x0020, 0x004D, 0x004D, 0x0052, 0x0045, 0x0053, 0x0055, 0x004C, 0x0054, 0x0020,
    0x0025, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_9E3FEFAE0F7F: [u16; 30] = [
    0x0077u16, 0x0069, 0x0064, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x002D, 0x0020, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x0074, 0x0079, 0x0070, 0x0065, 0x0020, 0x0025, 0x0064, 0x000A, 0x0000,
];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn widMessage(
    DeviceId: UINT,
    Message: UINT,
    PrivateHandle: DWORD_PTR,
    Parameter1: DWORD_PTR,
    Parameter2: DWORD_PTR,
) -> DWORD {
    let mut Result: MMRESULT = MMSYSERR_NOTSUPPORTED as MMRESULT;

    AcquireEntrypointMutex(WAVE_IN_DEVICE_TYPE as MMDEVICE_TYPE);

    SND_TRACE!(RUSTCONV_WSTR_9E3FEFAE0F7F.as_ptr(), Message);

    if Message == WIDM_GETNUMDEVS {
        Result = GetSoundDeviceCount(WAVE_IN_DEVICE_TYPE as MMDEVICE_TYPE) as MMRESULT;
    } else if Message == WIDM_START {
        Result = MmeSetState(PrivateHandle, (TRUE) as BOOL);
    } else if Message == WIDM_STOP {
        Result = MmeSetState(PrivateHandle, (FALSE) as BOOL);
    } else if Message == WIDM_GETDEVCAPS {
        Result = MmeGetSoundDeviceCapabilities(
            WAVE_IN_DEVICE_TYPE as MMDEVICE_TYPE,
            DeviceId as DWORD,
            Parameter1 as PVOID,
            Parameter2 as DWORD,
        );
    } else if Message == WIDM_OPEN {
        /* Do sanity checks for 'recording' SamplesPerSec value */
        let mut OpenParameters: LPWAVEOPENDESC = Parameter1 as LPWAVEOPENDESC;
        if (*(*OpenParameters).lpFormat).nSamplesPerSec > 100000 as DWORD {
            (*(*OpenParameters).lpFormat).nSamplesPerSec = 100000 as DWORD;
        }
        if (*(*OpenParameters).lpFormat).nSamplesPerSec < 5000 as DWORD {
            (*(*OpenParameters).lpFormat).nSamplesPerSec = 5000 as DWORD;
        }

        Result = MmeOpenDevice(
            WAVE_IN_DEVICE_TYPE as MMDEVICE_TYPE,
            DeviceId,
            Parameter1 as LPWAVEOPENDESC,
            Parameter2 as DWORD,
            PrivateHandle as *mut DWORD_PTR,
        );
    } else if Message == WIDM_CLOSE {
        Result = MmeCloseDevice(PrivateHandle);
    } else if Message == WIDM_PREPARE {
        /* TODO: Do we need to pass 2nd parameter? */
        Result = MmePrepareWaveHeader!(PrivateHandle, Parameter1);
    } else if Message == WIDM_UNPREPARE {
        Result = MmeUnprepareWaveHeader!(PrivateHandle, Parameter1);
    } else if Message == WIDM_RESET {
        /* Stop playback, reset position to zero */
        Result = MmeResetWavePlayback(PrivateHandle);
    } else if Message == WIDM_ADDBUFFER {
        Result = MmeWriteWaveHeader!(PrivateHandle, Parameter1);
    } else if Message == DRV_QUERYDEVICEINTERFACESIZE {
        Result = MmeGetDeviceInterfaceString(
            WAVE_IN_DEVICE_TYPE as MMDEVICE_TYPE,
            DeviceId as DWORD,
            core::ptr::null_mut(),
            0 as DWORD,
            Parameter1 as *mut DWORD,
        ); //FIXME DWORD_PTR
    } else if Message == DRV_QUERYDEVICEINTERFACE {
        Result = MmeGetDeviceInterfaceString(
            WAVE_IN_DEVICE_TYPE as MMDEVICE_TYPE,
            DeviceId as DWORD,
            Parameter1 as LPWSTR,
            Parameter2 as DWORD,
            core::ptr::null_mut(),
        ); //FIXME DWORD_PTR
    }

    SND_TRACE!(RUSTCONV_WSTR_6AD3A0D7893E.as_ptr(), Result);

    ReleaseEntrypointMutex(WAVE_IN_DEVICE_TYPE as MMDEVICE_TYPE);

    return Result as DWORD;
}
