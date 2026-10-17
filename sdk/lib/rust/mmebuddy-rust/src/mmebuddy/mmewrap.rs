use crate::mmebuddy::deviceinstance::CreateSoundDeviceInstance;
use crate::mmebuddy::deviceinstance::DestroySoundDeviceInstance;
use crate::mmebuddy::devicelist::GetSoundDevice;
use crate::mmebuddy::capabilities::GetSoundDeviceCapabilities;
use crate::mmebuddy::deviceinstance::GetSoundDeviceFromInstance;
use crate::mmebuddy::functiontable::GetSoundDeviceFunctionTable;
use crate::mmebuddy::devicelist::GetSoundDeviceType;
use crate::mmebuddy::deviceinstance::IsValidSoundDeviceInstance;
use crate::mmebuddy::deviceinstance::SetSoundDeviceInstanceMmeData;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1B0505F57EA3: [u16; 40] = [
    0x004Du16, 0x004D, 0x0045, 0x0020, 0x002A, 0x005F, 0x0047, 0x0045, 0x0054, 0x0043,
    0x0041, 0x0050, 0x0053, 0x0020, 0x0066, 0x006F, 0x0072, 0x0020, 0x0064, 0x0065,
    0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0025, 0x0064, 0x0020, 0x006F, 0x0066,
    0x0020, 0x0074, 0x0079, 0x0070, 0x0065, 0x0020, 0x0025, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_671FDAA2014F: [u16; 22] = [
    0x0046u16, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0020, 0x006E, 0x006F, 0x0074,
    0x0020, 0x0073, 0x0075, 0x0070, 0x0070, 0x006F, 0x0072, 0x0074, 0x0065, 0x0064,
    0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_84BD46EC5E69: [u16; 48] = [
    0x004Du16, 0x004D, 0x0045, 0x0020, 0x0063, 0x006C, 0x0069, 0x0065, 0x006E, 0x0074,
    0x0020, 0x0063, 0x0061, 0x006C, 0x006C, 0x0062, 0x0061, 0x0063, 0x006B, 0x0020,
    0x002D, 0x0020, 0x006D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065, 0x0020,
    0x0025, 0x0064, 0x002C, 0x0020, 0x0070, 0x0061, 0x0072, 0x0061, 0x006D, 0x0065,
    0x0074, 0x0065, 0x0072, 0x0020, 0x0025, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_AB131BEE91EE: [u16; 36] = [
    0x0052u16, 0x0065, 0x0073, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020,
    0x0077, 0x0061, 0x0076, 0x0065, 0x0020, 0x0064, 0x0065, 0x0076, 0x0069, 0x0063,
    0x0065, 0x0020, 0x0028, 0x0057, 0x004F, 0x0044, 0x004D, 0x005F, 0x0052, 0x0045,
    0x0053, 0x0045, 0x0054, 0x0029, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_AD9EF9580233: [u16; 16] = [
    0x004Fu16, 0x0070, 0x0065, 0x006E, 0x0069, 0x006E, 0x0067, 0x0020, 0x0064, 0x0065,
    0x0076, 0x0069, 0x0063, 0x0065, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_B1881C2EE48B: [u16; 17] = [
    0x0064u16, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x006E, 0x006F, 0x0077,
    0x0020, 0x006F, 0x0070, 0x0065, 0x006E, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_EDD0538B1000: [u16; 47] = [
    0x0043u16, 0x006C, 0x006F, 0x0073, 0x0069, 0x006E, 0x0067, 0x0020, 0x0077, 0x0061,
    0x0076, 0x0065, 0x0020, 0x0064, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020,
    0x0028, 0x0057, 0x0049, 0x0044, 0x004D, 0x005F, 0x0043, 0x004C, 0x004F, 0x0053,
    0x0045, 0x0020, 0x002F, 0x0020, 0x0057, 0x004F, 0x0044, 0x004D, 0x005F, 0x0043,
    0x004C, 0x004F, 0x0053, 0x0045, 0x0029, 0x000A, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn MmeSetState(PrivateHandle: DWORD_PTR, bStart: BOOL) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut FunctionTable: PMMFUNCTION_TABLE = core::mem::zeroed();
    let mut SoundDevice: PSOUND_DEVICE = core::mem::zeroed();
    let mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE;
    let mut OldState: BOOL;

    VALIDATE_MMSYS_PARAMETER!(PrivateHandle);
    SoundDeviceInstance = PrivateHandle as PSOUND_DEVICE_INSTANCE;

    VALIDATE_MMSYS_PARAMETER!(IsValidSoundDeviceInstance(SoundDeviceInstance));

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));
    if MMSUCCESS!(Result) == 0 {
        return TranslateInternalMmResult(Result);
    }

    /* Get the function table, and validate it */
    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    if MMSUCCESS!(Result) == 0 {
        return TranslateInternalMmResult(Result);
    }

    SND_ASSERT!((*FunctionTable).SetState.is_some());
    if (*FunctionTable).SetState.is_none() {
        /* FIXME */
        return MMSYSERR_NOTSUPPORTED;
    }
    /* Try change state */
    Result = (*FunctionTable)
        .SetState
        .unwrap_unchecked()(
        SoundDeviceInstance as *mut _SOUND_DEVICE_INSTANCE,
        bStart,
    );

    if MMSUCCESS!(Result) != 0 {
        /* Get old audio stream state */
        OldState = (*SoundDeviceInstance).bPaused;

        /* Store audio stream pause state */
        (*SoundDeviceInstance).bPaused = (bStart == 0) as i32 as BOOL;

        if (*SoundDeviceInstance).bPaused == (FALSE as i32) && OldState != 0 {
            InitiateSoundStreaming(SoundDeviceInstance);
        }
    }

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn NotifyMmeClient(SoundDeviceInstance: PSOUND_DEVICE_INSTANCE, Message: UINT, Parameter: DWORD_PTR) {
    SND_ASSERT!(SoundDeviceInstance);

    SND_TRACE!(RUSTCONV_WSTR_84BD46EC5E69.as_ptr(),
              Message as i32,
              Parameter as i32);

    if (*SoundDeviceInstance).WinMM.ClientCallback != 0 {
        DriverCallback((*SoundDeviceInstance).WinMM.ClientCallback,
                       (HIWORD!((*SoundDeviceInstance).WinMM.Flags)) as DWORD,
                       (*SoundDeviceInstance).WinMM.Handle,
                       Message,
                       (*SoundDeviceInstance).WinMM.ClientCallbackInstanceData,
                       Parameter,
                       0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MmeGetSoundDeviceCapabilities(mut DeviceType: MMDEVICE_TYPE, mut DeviceId: DWORD, mut Capabilities: PVOID, mut CapabilitiesSize: DWORD) -> MMRESULT {
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut::<_SOUND_DEVICE>();
    let mut Result: MMRESULT = 0;

    SND_TRACE!(RUSTCONV_WSTR_1B0505F57EA3.as_ptr(), DeviceId, DeviceType);

    /* FIXME: Validate device ID */
    if Capabilities.is_null() {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }
    if !(IS_VALID_SOUND_DEVICE_TYPE!(DeviceType) != 0) {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }

    /* Our parameter checks are done elsewhere */

    Result = GetSoundDevice(DeviceType, DeviceId, core::ptr::addr_of_mut!(SoundDevice));

    if ((MMSUCCESS!(Result)) == 0) {
        return Result;
    }

    return GetSoundDeviceCapabilities(SoundDevice, DeviceId, Capabilities, CapabilitiesSize);
}

#[no_mangle]
pub unsafe extern "C" fn MmeOpenDevice(mut DeviceType: MMDEVICE_TYPE, mut DeviceId: UINT, mut OpenParameters: LPWAVEOPENDESC, mut Flags: DWORD, mut PrivateHandle: *mut DWORD_PTR) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut Message: UINT = 0;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE = core::ptr::null_mut();
    let mut Format: LPWAVEFORMATEX = core::ptr::null_mut();

    SND_TRACE!(RUSTCONV_WSTR_AD9EF9580233.as_ptr());

    VALIDATE_MMSYS_PARAMETER!(((IS_WAVE_DEVICE_TYPE!(DeviceType)) != 0) || ((IS_MIXER_DEVICE_TYPE!(DeviceType)) != 0) || ((IS_MIDI_DEVICE_TYPE!(DeviceType)) != 0));
    VALIDATE_MMSYS_PARAMETER!(OpenParameters);

    Result = GetSoundDevice(DeviceType, DeviceId, core::ptr::addr_of_mut!(SoundDevice));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    if (DeviceType) as u32 == WAVE_IN_DEVICE_TYPE || (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE {
        Format = (*OpenParameters).lpFormat;

        /* Does this device support the format? */
        Result = QueryWaveDeviceFormatSupport(SoundDevice, Format, core::mem::size_of::<WAVEFORMATEX>() as DWORD);
        if ((MMSUCCESS!(Result)) == 0) {
            SND_ERR!(RUSTCONV_WSTR_671FDAA2014F.as_ptr());
            return TranslateInternalMmResult(Result);
        }

        /* If the caller just wanted to know if a format is supported, end here */
        if (Flags & WAVE_FORMAT_QUERY) != 0 {
            return MMSYSERR_NOERROR;
        }
    }

    /* Check that winmm gave us a private handle to fill */
    VALIDATE_MMSYS_PARAMETER!(PrivateHandle);

    /* Create a sound device instance and open the sound device */
    Result = CreateSoundDeviceInstance(SoundDevice, core::ptr::addr_of_mut!(SoundDeviceInstance));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = SetWaveDeviceFormat(SoundDeviceInstance, DeviceId, Format, core::mem::size_of::<WAVEFORMATEX>() as DWORD);
    if ((MMSUCCESS!(Result)) == 0) {
        /* TODO: Destroy sound instance */
        return TranslateInternalMmResult(Result);
    }

    /* Store the device instance pointer in the private handle */
    *PrivateHandle = SoundDeviceInstance as DWORD_PTR;

    /* Store the additional information we were given - FIXME: Need flags! */
    SetSoundDeviceInstanceMmeData(
        SoundDeviceInstance,
        (*OpenParameters).hWave as HDRVR, /* works because LPMIXEROPENDESC/etc has also the handle as first member */
        (*OpenParameters).dwCallback,
        (*OpenParameters).dwInstance,
        Flags,
    );

    if (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE || (DeviceType) as u32 == WAVE_IN_DEVICE_TYPE
        || (DeviceType) as u32 == MIDI_OUT_DEVICE_TYPE || (DeviceType) as u32 == MIDI_IN_DEVICE_TYPE
    {
        /* Let the application know the device is open */

        if (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE {
            Message = WOM_OPEN;
        } else if (DeviceType) as u32 == WAVE_IN_DEVICE_TYPE {
            Message = WIM_OPEN;
        } else if (DeviceType) as u32 == MIDI_IN_DEVICE_TYPE {
            Message = MIM_OPEN;
        } else {
            Message = MOM_OPEN;
        }

        ReleaseEntrypointMutex(DeviceType);

        NotifyMmeClient(
            SoundDeviceInstance,
            Message,
            0,
        );

        AcquireEntrypointMutex(DeviceType);
    }

    SND_TRACE!(RUSTCONV_WSTR_B1881C2EE48B.as_ptr());

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn MmeCloseDevice(PrivateHandle: DWORD_PTR) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE;
    let mut SoundDevice: PSOUND_DEVICE = core::mem::zeroed();
    let mut DeviceType: MMDEVICE_TYPE = core::mem::zeroed();
    let mut Message: UINT = 0;

    SND_TRACE!(RUSTCONV_WSTR_EDD0538B1000.as_ptr());

    VALIDATE_MMSYS_PARAMETER!(PrivateHandle);
    SoundDeviceInstance = PrivateHandle as PSOUND_DEVICE_INSTANCE;

    if (((IsValidSoundDeviceInstance(SoundDeviceInstance))) == 0) {
        return MMSYSERR_INVALHANDLE;
    }

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    /* TODO: Check device is stopped! */

    if (DeviceType) as u32 != MIXER_DEVICE_TYPE {
        ReleaseEntrypointMutex(DeviceType);

        if (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE {
            Message = WOM_CLOSE;
        } else if (DeviceType) as u32 == WAVE_IN_DEVICE_TYPE {
            Message = WIM_CLOSE;
        } else if (DeviceType) as u32 == MIDI_IN_DEVICE_TYPE {
            Message = MIM_CLOSE;
        } else if (DeviceType) as u32 == MIDI_OUT_DEVICE_TYPE {
            Message = MOM_CLOSE;
        }

        /* TODO: Work with MIDI devices too */
        NotifyMmeClient(SoundDeviceInstance, Message, 0);
        AcquireEntrypointMutex(DeviceType);
    }

    Result = DestroySoundDeviceInstance(SoundDeviceInstance);

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn MmeResetWavePlayback(PrivateHandle: DWORD_PTR) -> MMRESULT {
    let mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE = core::ptr::null_mut();

    SND_TRACE!(RUSTCONV_WSTR_AB131BEE91EE.as_ptr());

    VALIDATE_MMSYS_PARAMETER!(PrivateHandle);
    if PrivateHandle == 0 {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }
    SoundDeviceInstance = PrivateHandle as PSOUND_DEVICE_INSTANCE;

    return StopStreaming(SoundDeviceInstance);
}
#[no_mangle]
pub unsafe extern "C" fn MmeGetDeviceInterfaceString(DeviceType: MMDEVICE_TYPE, DeviceId: DWORD, Interface: LPWSTR, InterfaceLength: DWORD, InterfaceSize: *mut DWORD) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut();

    Result = GetSoundDevice(DeviceType, DeviceId, core::ptr::addr_of_mut!(SoundDevice));
    if (((MMSUCCESS!(Result))) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    if (((MMSUCCESS!(Result))) == 0) {
        return TranslateInternalMmResult(Result);
    }

    if (*FunctionTable).GetDeviceInterfaceString.is_none() {
        /* querying device interface string / size not supported */
        return MMSYSERR_NOTSUPPORTED;
    }

    /* Call the driver */
    Result = (*FunctionTable)
        .GetDeviceInterfaceString
        .unwrap_unchecked()(
            DeviceType,
            DeviceId,
            Interface,
            InterfaceLength,
            InterfaceSize,
        );

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn MmeGetPosition(mut DeviceType: MMDEVICE_TYPE, mut DeviceId: DWORD, mut PrivateHandle: DWORD_PTR, mut Time: *mut MMTIME, mut Size: DWORD) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE = core::ptr::null_mut();
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut();

    VALIDATE_MMSYS_PARAMETER!(PrivateHandle);
    SoundDeviceInstance = PrivateHandle as PSOUND_DEVICE_INSTANCE;

    if IsValidSoundDeviceInstance(SoundDeviceInstance) == 0 {
        return MMSYSERR_INVALHANDLE;
    }

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    if Size as usize != core::mem::size_of::<MMTIME>() {
        return MMSYSERR_INVALPARAM;
    }

    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    if (*FunctionTable).GetPos.is_none() {
        /* This indicates bad practice, really! If you can open, why not close?! */
        return MMSYSERR_NOTSUPPORTED;
    }

    /* Call the driver */
    Result = (*FunctionTable).GetPos.unwrap_unchecked()(
        SoundDeviceInstance as *mut _SOUND_DEVICE_INSTANCE,
        Time,
    );

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn MmeGetVolume(mut DeviceType: MMDEVICE_TYPE, mut DeviceId: DWORD, mut PrivateHandle: DWORD_PTR, mut pdwVolume: DWORD_PTR) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE = core::ptr::null_mut();
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut();

    /* Sanity check */
    SND_ASSERT!(
        (DeviceType) as u32 == AUX_DEVICE_TYPE
            || (DeviceType) as u32 == MIDI_OUT_DEVICE_TYPE
            || (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE,
    );

    VALIDATE_MMSYS_PARAMETER!(PrivateHandle);
    SoundDeviceInstance = PrivateHandle as PSOUND_DEVICE_INSTANCE;

    if IsValidSoundDeviceInstance(SoundDeviceInstance) == 0 {
        return MMSYSERR_INVALHANDLE as MMRESULT;
    }

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    if (*FunctionTable).GetVolume.is_none() {
        return MMSYSERR_NOTSUPPORTED as MMRESULT;
    }

    /* Call the driver */
    Result = (*FunctionTable).GetVolume.unwrap_unchecked()(
        SoundDeviceInstance,
        DeviceId,
        pdwVolume as PDWORD,
    );

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn MmeSetVolume(DeviceType: MMDEVICE_TYPE, DeviceId: DWORD, PrivateHandle: DWORD_PTR, dwVolume: DWORD_PTR) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE = core::ptr::null_mut();
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut();

    /* Sanity check */
    SND_ASSERT!((DeviceType) as u32 == AUX_DEVICE_TYPE ||
               (DeviceType) as u32 == MIDI_OUT_DEVICE_TYPE ||
               (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE);

    VALIDATE_MMSYS_PARAMETER!(PrivateHandle);
    SoundDeviceInstance = PrivateHandle as PSOUND_DEVICE_INSTANCE;

    if IsValidSoundDeviceInstance(SoundDeviceInstance) == 0 {
        return MMSYSERR_INVALHANDLE;
    }

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    if (*FunctionTable).SetVolume.is_none() {
        return MMSYSERR_NOTSUPPORTED;
    }

    /* Call the driver */
    Result = (*FunctionTable)
        .SetVolume
        .unwrap_unchecked()(
        SoundDeviceInstance,
        DeviceId,
        dwVolume as DWORD,
    );

    return Result;
}
