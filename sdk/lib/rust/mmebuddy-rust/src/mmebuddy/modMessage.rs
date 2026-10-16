use crate::mmebuddy::reentrancy::AcquireEntrypointMutex;
use crate::mmebuddy::devicelist::GetSoundDeviceCount;
use crate::mmebuddy::mmewrap::MmeCloseDevice;
use crate::mmebuddy::mmewrap::MmeGetDeviceInterfaceString;
use crate::mmebuddy::mmewrap::MmeGetSoundDeviceCapabilities;
use crate::mmebuddy::mmewrap::MmeGetVolume;
use crate::mmebuddy::mmewrap::MmeOpenDevice;
use crate::mmebuddy::mmewrap::MmeSetVolume;
use crate::mmebuddy::reentrancy::ReleaseEntrypointMutex;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_772770C7EE36: [u16; 30] = [
    0x006Du16, 0x006F, 0x0064, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x002D, 0x0020, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x0074, 0x0079, 0x0070, 0x0065, 0x0020, 0x0025, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_DB2CAF79C27F: [u16; 34] = [
    0x006Du16, 0x006F, 0x0064, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x0020, 0x0072, 0x0065, 0x0074, 0x0075, 0x0072, 0x006E, 0x0069, 0x006E, 0x0067,
    0x0020, 0x004D, 0x004D, 0x0052, 0x0045, 0x0053, 0x0055, 0x004C, 0x0054, 0x0020,
    0x0025, 0x0064, 0x000A, 0x0000,
];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn modMessage(
    DeviceId: UINT,
    Message: UINT,
    PrivateHandle: DWORD_PTR,
    Parameter1: DWORD_PTR,
    Parameter2: DWORD_PTR,
) -> DWORD {
    let mut Result: MMRESULT = MMSYSERR_NOTSUPPORTED as MMRESULT;

    AcquireEntrypointMutex(MIDI_OUT_DEVICE_TYPE as MMDEVICE_TYPE);

    SND_TRACE!(RUSTCONV_WSTR_772770C7EE36.as_ptr(), Message);

    match Message {
        MODM_GETNUMDEVS => {
            Result = GetSoundDeviceCount(MIDI_OUT_DEVICE_TYPE as MMDEVICE_TYPE) as MMRESULT;
        }
        MODM_GETDEVCAPS => {
            Result = MmeGetSoundDeviceCapabilities(
                MIDI_OUT_DEVICE_TYPE as MMDEVICE_TYPE,
                DeviceId as DWORD,
                Parameter1 as PVOID,
                Parameter2 as DWORD,
            );
        }
        DRV_QUERYDEVICEINTERFACESIZE => {
            Result = MmeGetDeviceInterfaceString(
                MIDI_OUT_DEVICE_TYPE as MMDEVICE_TYPE,
                DeviceId as DWORD,
                ::core::ptr::null_mut::<WCHAR>(),
                0 as DWORD,
                Parameter1 as *mut DWORD,
            );
        }
        DRV_QUERYDEVICEINTERFACE => {
            Result = MmeGetDeviceInterfaceString(
                MIDI_OUT_DEVICE_TYPE as MMDEVICE_TYPE,
                DeviceId as DWORD,
                Parameter1 as LPWSTR,
                Parameter2 as DWORD,
                ::core::ptr::null_mut::<DWORD>(),
            );
        }
        MODM_OPEN => {
            Result = MmeOpenDevice(
                MIDI_OUT_DEVICE_TYPE as MMDEVICE_TYPE,
                DeviceId,
                Parameter1 as LPWAVEOPENDESC,
                Parameter2 as DWORD,
                PrivateHandle as *mut DWORD_PTR,
            );
        }
        MODM_CLOSE => {
            Result = MmeCloseDevice(PrivateHandle);
        }
        MODM_GETVOLUME => {
            Result = MmeGetVolume(
                MIDI_OUT_DEVICE_TYPE as MMDEVICE_TYPE,
                DeviceId as DWORD,
                PrivateHandle,
                Parameter1,
            );
        }
        MODM_SETVOLUME => {
            Result = MmeSetVolume(
                MIDI_OUT_DEVICE_TYPE as MMDEVICE_TYPE,
                DeviceId as DWORD,
                PrivateHandle,
                Parameter1,
            );
        }
        _ => {}
    }

    SND_TRACE!(RUSTCONV_WSTR_DB2CAF79C27F.as_ptr(), Result);

    ReleaseEntrypointMutex(MIDI_OUT_DEVICE_TYPE as MMDEVICE_TYPE);

    return Result as DWORD;
}
