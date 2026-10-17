use crate::mmebuddy::reentrancy::AcquireEntrypointMutex;
use crate::mmebuddy::devicelist::GetSoundDeviceCount;
use crate::mmebuddy::mmewrap::MmeGetSoundDeviceCapabilities;
use crate::mmebuddy::mmewrap::MmeGetVolume;
use crate::mmebuddy::mmewrap::MmeSetVolume;
use crate::mmebuddy::reentrancy::ReleaseEntrypointMutex;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_824F538E84C7: [u16; 34] = [
    0x0061u16, 0x0075, 0x0078, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x0072, 0x0065, 0x0074, 0x0075, 0x0072, 0x006E, 0x0069, 0x006E, 0x0067,
    0x0020, 0x004D, 0x004D, 0x0052, 0x0045, 0x0053, 0x0055, 0x004C, 0x0054, 0x0020,
    0x0025, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_EA59DBC0BC5B: [u16; 30] = [
    0x0061u16, 0x0075, 0x0078, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x002D, 0x0020, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x0074, 0x0079, 0x0070, 0x0065, 0x0020, 0x0025, 0x0064, 0x000A, 0x0000,
];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn auxMessage(
    DeviceId: UINT,
    Message: UINT,
    PrivateHandle: DWORD_PTR,
    Parameter1: DWORD_PTR,
    Parameter2: DWORD_PTR,
) -> DWORD {
    let mut Result: MMRESULT = MMSYSERR_NOTSUPPORTED as MMRESULT;

    AcquireEntrypointMutex((AUX_DEVICE_TYPE) as MMDEVICE_TYPE);

    SND_TRACE!(RUSTCONV_WSTR_EA59DBC0BC5B.as_ptr(), Message);

    match Message {
        AUXDM_GETNUMDEVS => {
            Result = GetSoundDeviceCount((AUX_DEVICE_TYPE) as MMDEVICE_TYPE) as MMRESULT;
        }

        AUXDM_GETDEVCAPS => {
            Result = MmeGetSoundDeviceCapabilities(
                (AUX_DEVICE_TYPE) as MMDEVICE_TYPE,
                DeviceId as DWORD,
                Parameter1 as PVOID,
                Parameter2 as DWORD,
            );
        }

        AUXDM_GETVOLUME => {
            Result = MmeGetVolume(
                (AUX_DEVICE_TYPE) as MMDEVICE_TYPE,
                DeviceId as DWORD,
                PrivateHandle,
                Parameter1,
            );
        }

        AUXDM_SETVOLUME => {
            Result = MmeSetVolume(
                (AUX_DEVICE_TYPE) as MMDEVICE_TYPE,
                DeviceId as DWORD,
                PrivateHandle,
                Parameter1,
            );
        }

        _ => {}
    }

    SND_TRACE!(RUSTCONV_WSTR_824F538E84C7.as_ptr(), Result);

    ReleaseEntrypointMutex((AUX_DEVICE_TYPE) as MMDEVICE_TYPE);

    return Result as DWORD;
}
