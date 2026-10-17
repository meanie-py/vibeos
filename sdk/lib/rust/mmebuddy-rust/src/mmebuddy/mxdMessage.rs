use crate::mmebuddy::reentrancy::AcquireEntrypointMutex;
use crate::mmebuddy::devicelist::GetSoundDevice;
use crate::mmebuddy::devicelist::GetSoundDeviceCount;
use crate::mmebuddy::deviceinstance::GetSoundDeviceFromInstance;
use crate::mmebuddy::functiontable::GetSoundDeviceFunctionTable;
use crate::mmebuddy::mmewrap::MmeCloseDevice;
use crate::mmebuddy::mmewrap::MmeGetDeviceInterfaceString;
use crate::mmebuddy::mmewrap::MmeGetSoundDeviceCapabilities;
use crate::mmebuddy::mmewrap::MmeOpenDevice;
use crate::mmebuddy::reentrancy::ReleaseEntrypointMutex;
use crate::mmebuddy::utility::TranslateInternalMmResult;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn MmeGetLineInfo(
    DeviceId: UINT,
    Message: UINT,
    PrivateHandle: DWORD_PTR,
    Parameter1: DWORD_PTR,
    Parameter2: DWORD_PTR,
) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE;
    let mut SoundDevice: PSOUND_DEVICE = core::mem::zeroed();
    let mut FunctionTable: PMMFUNCTION_TABLE = core::mem::zeroed();

    //SND_TRACE!(L"Getting mixer info %u\n", Message);

    if PrivateHandle == 0 {
        Result = GetSoundDevice(
            MIXER_DEVICE_TYPE as MMDEVICE_TYPE,
            DeviceId as DWORD,
            core::ptr::addr_of_mut!(SoundDevice),
        );

        if ((MMSUCCESS!(Result)) == 0) {
            return TranslateInternalMmResult(Result);
        }

        Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
        if ((MMSUCCESS!(Result)) == 0) {
            return TranslateInternalMmResult(Result);
        }

        Result = (*FunctionTable).QueryMixerInfo.unwrap_unchecked()(
            core::ptr::null_mut(),
            DeviceId as DWORD,
            Message,
            Parameter1 as LPVOID,
            Parameter2 as DWORD,
        );
        return Result;
    }

    VALIDATE_MMSYS_PARAMETER!(PrivateHandle);
    SoundDeviceInstance = PrivateHandle as PSOUND_DEVICE_INSTANCE;

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    if (*FunctionTable).QueryMixerInfo.is_none() {
        return MMSYSERR_NOTSUPPORTED as MMRESULT;
    }

    Result = (*FunctionTable).QueryMixerInfo.unwrap_unchecked()(
        SoundDeviceInstance as *mut _SOUND_DEVICE_INSTANCE,
        DeviceId as DWORD,
        Message,
        Parameter1 as LPVOID,
        Parameter2 as DWORD,
    );

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn mxdMessage(DeviceId: UINT, Message: UINT, PrivateHandle: DWORD_PTR, Parameter1: DWORD_PTR, Parameter2: DWORD_PTR) -> DWORD {
    let mut Result: MMRESULT = MMSYSERR_NOTSUPPORTED as MMRESULT;

    AcquireEntrypointMutex(MIXER_DEVICE_TYPE as MMDEVICE_TYPE);

    //SND_TRACE!(L"mxdMessage - Message type %d\n", Message);

    match Message {
        MXDM_GETNUMDEVS => {
            Result = GetSoundDeviceCount(MIXER_DEVICE_TYPE as MMDEVICE_TYPE) as MMRESULT;
        }

        MXDM_GETDEVCAPS => {
            Result = MmeGetSoundDeviceCapabilities(
                MIXER_DEVICE_TYPE as MMDEVICE_TYPE,
                DeviceId as DWORD,
                Parameter1 as PVOID,
                Parameter2 as DWORD,
            ) as MMRESULT;
        }

        MXDM_INIT => {
            Result = MMSYSERR_NOERROR as MMRESULT;
        }

        MXDM_OPEN => {
            Result = MmeOpenDevice(
                MIXER_DEVICE_TYPE as MMDEVICE_TYPE,
                DeviceId,
                Parameter1 as LPWAVEOPENDESC,
                Parameter2 as DWORD,
                PrivateHandle as *mut DWORD_PTR,
            ) as MMRESULT;
            VALIDATE_MMSYS_PARAMETER!(*(PrivateHandle as *mut DWORD_PTR));
            if *(PrivateHandle as *mut DWORD_PTR) == 0 {
                return 11 as DWORD; // MMSYSERR_INVALPARAM
            }
        }

        MXDM_CLOSE => {
            Result = MmeCloseDevice(PrivateHandle) as MMRESULT;
        }

        MXDM_GETCONTROLDETAILS => {
            Result = MmeGetLineInfo(
                DeviceId,
                Message,
                PrivateHandle,
                Parameter1,
                Parameter2,
            ) as MMRESULT;
        }

        MXDM_SETCONTROLDETAILS => {
            Result = MmeGetLineInfo(
                DeviceId,
                Message,
                PrivateHandle,
                Parameter1,
                Parameter2,
            ) as MMRESULT;
        }

        MXDM_GETLINECONTROLS => {
            Result = MmeGetLineInfo(
                DeviceId,
                Message,
                PrivateHandle,
                Parameter1,
                Parameter2,
            ) as MMRESULT;
        }

        MXDM_GETLINEINFO => {
            Result = MmeGetLineInfo(
                DeviceId,
                Message,
                PrivateHandle,
                Parameter1,
                Parameter2,
            ) as MMRESULT;
        }

        DRV_QUERYDEVICEINTERFACESIZE => {
            Result = MmeGetDeviceInterfaceString(
                MIXER_DEVICE_TYPE as MMDEVICE_TYPE,
                DeviceId as DWORD,
                ::core::ptr::null_mut::<WCHAR>(),
                0 as DWORD,
                Parameter1 as *mut DWORD,
            ) as MMRESULT; //FIXME DWORD_PTR
        }

        DRV_QUERYDEVICEINTERFACE => {
            Result = MmeGetDeviceInterfaceString(
                MIXER_DEVICE_TYPE as MMDEVICE_TYPE,
                DeviceId as DWORD,
                Parameter1 as LPWSTR,
                Parameter2 as DWORD,
                ::core::ptr::null_mut::<DWORD>(),
            ) as MMRESULT; //FIXME DWORD_PTR
        }

        _ => {}
    }

    //SND_TRACE!(L"mxdMessage returning MMRESULT %d\n", Result);

    ReleaseEntrypointMutex(MIXER_DEVICE_TYPE as MMDEVICE_TYPE);

    return Result as DWORD;
}
