use crate::mmebuddy::reentrancy::AcquireEntrypointMutex;
use crate::mmebuddy::devicelist::GetSoundDeviceCount;
use crate::mmebuddy::mmewrap::MmeCloseDevice;
use crate::mmebuddy::mmewrap::MmeGetDeviceInterfaceString;
use crate::mmebuddy::mmewrap::MmeGetPosition;
use crate::mmebuddy::mmewrap::MmeGetSoundDeviceCapabilities;
use crate::mmebuddy::mmewrap::MmeGetVolume;
use crate::mmebuddy::mmewrap::MmeOpenDevice;
use crate::mmebuddy::mmewrap::MmeResetWavePlayback;
use crate::mmebuddy::mmewrap::MmeSetState;
use crate::mmebuddy::mmewrap::MmeSetVolume;
use crate::mmebuddy::reentrancy::ReleaseEntrypointMutex;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_15997F8BF1BE: [u16; 34] = [
    0x0077u16, 0x006F, 0x0064, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x0072, 0x0065, 0x0074, 0x0075, 0x0072, 0x006E, 0x0069, 0x006E, 0x0067,
    0x0020, 0x004D, 0x004D, 0x0052, 0x0045, 0x0053, 0x0055, 0x004C, 0x0054, 0x0020,
    0x0025, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_969CB4864F52: [u16; 30] = [
    0x0077u16, 0x006F, 0x0064, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x002D, 0x0020, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x0074, 0x0079, 0x0070, 0x0065, 0x0020, 0x0025, 0x0064, 0x000A, 0x0000,
];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn wodMessage(
    DeviceId: UINT,
    Message: UINT,
    PrivateHandle: DWORD_PTR,
    Parameter1: DWORD_PTR,
    Parameter2: DWORD_PTR,
) -> DWORD {
    let mut Result: MMRESULT = MMSYSERR_NOTSUPPORTED as MMRESULT;

    AcquireEntrypointMutex((WAVE_OUT_DEVICE_TYPE) as MMDEVICE_TYPE);

    SND_TRACE!(RUSTCONV_WSTR_969CB4864F52.as_ptr(), Message);

    match Message {
        WODM_GETNUMDEVS => {
            Result = GetSoundDeviceCount((WAVE_OUT_DEVICE_TYPE) as MMDEVICE_TYPE) as MMRESULT;
        }

        WODM_GETDEVCAPS => {
            Result = MmeGetSoundDeviceCapabilities(
                (WAVE_OUT_DEVICE_TYPE) as MMDEVICE_TYPE,
                DeviceId,
                Parameter1 as PVOID,
                (Parameter2) as DWORD,
            );
        }

        WODM_OPEN => {
            Result = MmeOpenDevice(
                (WAVE_OUT_DEVICE_TYPE) as MMDEVICE_TYPE,
                DeviceId,
                Parameter1 as LPWAVEOPENDESC,
                (Parameter2) as DWORD,
                PrivateHandle as *mut DWORD_PTR,
            );
        }

        WODM_CLOSE => {
            Result = MmeCloseDevice(PrivateHandle);
        }

        WODM_PREPARE => {
            /* TODO: Do we need to pass 2nd parameter? */
            Result = MmePrepareWaveHeader!(PrivateHandle, Parameter1);
        }

        WODM_UNPREPARE => {
            Result = MmeUnprepareWaveHeader!(PrivateHandle, Parameter1);
        }

        WODM_WRITE => {
            Result = MmeWriteWaveHeader!(PrivateHandle, Parameter1);
        }

        WODM_RESET => {
            /* Stop playback, reset position to zero */
            Result = MmeResetWavePlayback(PrivateHandle);
        }

        WODM_RESTART => {
            /* Continue playback when paused */
            Result = MmeSetState(PrivateHandle, (TRUE) as BOOL);
        }

        WODM_PAUSE => {
            /* pause playback */
            Result = MmeSetState(PrivateHandle, (FALSE) as BOOL);
        }

        WODM_GETPOS => {
            Result = MmeGetPosition(
                (WAVE_OUT_DEVICE_TYPE) as MMDEVICE_TYPE,
                DeviceId,
                PrivateHandle,
                Parameter1 as *mut MMTIME,
                (Parameter2) as DWORD,
            );
        }

        WODM_GETVOLUME => {
            Result = MmeGetVolume(
                (WAVE_OUT_DEVICE_TYPE) as MMDEVICE_TYPE,
                DeviceId,
                PrivateHandle,
                Parameter1,
            );
        }

        WODM_SETVOLUME => {
            Result = MmeSetVolume(
                (WAVE_OUT_DEVICE_TYPE) as MMDEVICE_TYPE,
                DeviceId,
                PrivateHandle,
                Parameter1,
            );
        }

        DRV_QUERYDEVICEINTERFACESIZE => {
            Result = MmeGetDeviceInterfaceString(
                (WAVE_OUT_DEVICE_TYPE) as MMDEVICE_TYPE,
                DeviceId,
                core::ptr::null_mut(),
                0,
                Parameter1 as *mut DWORD,
            ); //FIXME DWORD_PTR
        }

        DRV_QUERYDEVICEINTERFACE => {
            Result = MmeGetDeviceInterfaceString(
                (WAVE_OUT_DEVICE_TYPE) as MMDEVICE_TYPE,
                DeviceId,
                Parameter1 as LPWSTR,
                (Parameter2) as DWORD,
                core::ptr::null_mut(),
            ); //FIXME DWORD_PTR
        }

        _ => {}
    }

    SND_TRACE!(RUSTCONV_WSTR_15997F8BF1BE.as_ptr(), Result);

    ReleaseEntrypointMutex((WAVE_OUT_DEVICE_TYPE) as MMDEVICE_TYPE);

    return Result as DWORD;
}
