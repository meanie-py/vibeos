use crate::mmebuddy::reentrancy::AcquireEntrypointMutex;
use crate::mmebuddy::devicelist::GetSoundDeviceCount;
use crate::mmebuddy::mmewrap::MmeCloseDevice;
use crate::mmebuddy::mmewrap::MmeGetDeviceInterfaceString;
use crate::mmebuddy::mmewrap::MmeGetSoundDeviceCapabilities;
use crate::mmebuddy::mmewrap::MmeOpenDevice;
use crate::mmebuddy::mmewrap::MmeSetState;
use crate::mmebuddy::reentrancy::ReleaseEntrypointMutex;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_BE66D8220DC1: [u16; 30] = [
    0x006Du16, 0x0069, 0x0064, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x002D, 0x0020, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x0074, 0x0079, 0x0070, 0x0065, 0x0020, 0x0025, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_C546EFCDF7E0: [u16; 34] = [
    0x006Du16, 0x0069, 0x0064, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x0072, 0x0065, 0x0074, 0x0075, 0x0072, 0x006E, 0x0069, 0x006E, 0x0067,
    0x0020, 0x004D, 0x004D, 0x0052, 0x0045, 0x0053, 0x0055, 0x004C, 0x0054, 0x0020,
    0x0025, 0x0064, 0x000A, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn midMessage(
    DeviceId: UINT,
    Message: UINT,
    PrivateHandle: DWORD_PTR,
    Parameter1: DWORD_PTR,
    Parameter2: DWORD_PTR,
) -> DWORD {
    let mut Result: MMRESULT = MMSYSERR_NOTSUPPORTED as MMRESULT;

    AcquireEntrypointMutex(MIDI_IN_DEVICE_TYPE as MMDEVICE_TYPE);

    SND_TRACE!(RUSTCONV_WSTR_BE66D8220DC1.as_ptr(), Message);

    if Message == MIDM_GETNUMDEVS {
        Result = GetSoundDeviceCount(MIDI_IN_DEVICE_TYPE as MMDEVICE_TYPE) as MMRESULT;
    } else if Message == MIDM_GETDEVCAPS {
        Result = MmeGetSoundDeviceCapabilities(
            MIDI_IN_DEVICE_TYPE as MMDEVICE_TYPE,
            DeviceId as DWORD,
            Parameter1 as PVOID,
            Parameter2 as DWORD,
        );
    } else if Message == DRV_QUERYDEVICEINTERFACESIZE {
        Result = MmeGetDeviceInterfaceString(
            MIDI_IN_DEVICE_TYPE as MMDEVICE_TYPE,
            DeviceId as DWORD,
            ::core::ptr::null_mut::<WCHAR>(),
            0 as DWORD,
            Parameter1 as *mut DWORD,
        );
    } else if Message == DRV_QUERYDEVICEINTERFACE {
        Result = MmeGetDeviceInterfaceString(
            MIDI_IN_DEVICE_TYPE as MMDEVICE_TYPE,
            DeviceId as DWORD,
            Parameter1 as LPWSTR,
            Parameter2 as DWORD,
            ::core::ptr::null_mut::<DWORD>(),
        );
    } else if Message == MIDM_OPEN {
        Result = MmeOpenDevice(
            MIDI_IN_DEVICE_TYPE as MMDEVICE_TYPE,
            DeviceId,
            Parameter1 as LPWAVEOPENDESC,
            Parameter2 as DWORD,
            PrivateHandle as *mut DWORD_PTR,
        );
    } else if Message == MIDM_CLOSE {
        Result = MmeCloseDevice(PrivateHandle);
    } else if Message == MIDM_START {
        Result = MmeSetState(PrivateHandle, (TRUE as DWORD_PTR) as BOOL);
    } else if Message == MIDM_STOP {
        Result = MmeSetState(PrivateHandle, (FALSE as DWORD_PTR) as BOOL);
    }

    SND_TRACE!(RUSTCONV_WSTR_C546EFCDF7E0.as_ptr(), Result);

    ReleaseEntrypointMutex(MIDI_IN_DEVICE_TYPE as MMDEVICE_TYPE);

    return Result as DWORD;
}
