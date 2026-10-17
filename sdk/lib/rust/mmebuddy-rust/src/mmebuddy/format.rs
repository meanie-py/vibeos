use crate::mmebuddy::deviceinstance::GetSoundDeviceFromInstance;
use crate::mmebuddy::functiontable::GetSoundDeviceFunctionTable;
use crate::mmebuddy::devicelist::GetSoundDeviceType;
use crate::mmebuddy::devicelist::IsValidSoundDevice;
use crate::mmebuddy::deviceinstance::IsValidSoundDeviceInstance;
use crate::mmebuddy::utility::TranslateInternalMmResult;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_27ADBB33A7C5: [u16; 21] = [
    0x0053u16, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0077, 0x0061,
    0x0076, 0x0065, 0x0020, 0x0066, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_814E4B293E8B: [u16; 30] = [
    0x0051u16, 0x0075, 0x0065, 0x0072, 0x0079, 0x0069, 0x006E, 0x0067, 0x0020, 0x0077,
    0x0061, 0x0076, 0x0065, 0x0020, 0x0066, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074,
    0x0020, 0x0073, 0x0075, 0x0070, 0x0070, 0x006F, 0x0072, 0x0074, 0x000A, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn QueryWaveDeviceFormatSupport(
    SoundDevice: PSOUND_DEVICE,
    Format: LPWAVEFORMATEX,
    FormatSize: DWORD,
) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut DeviceType: MMDEVICE_TYPE = core::mem::zeroed();
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut();

    SND_TRACE!(RUSTCONV_WSTR_814E4B293E8B.as_ptr());

    VALIDATE_MMSYS_PARAMETER!(IsValidSoundDevice(SoundDevice));
    VALIDATE_MMSYS_PARAMETER!(!Format.is_null());
    VALIDATE_MMSYS_PARAMETER!(FormatSize as usize >= (core::mem::size_of::<WAVEFORMATEX>() as usize));

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR as MMRESULT);

    /* Ensure we have a wave device (TODO: check if this applies to wavein as well) */
    VALIDATE_MMSYS_PARAMETER!(IS_WAVE_DEVICE_TYPE!(DeviceType));

    /* Obtain the function table */
    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    SND_ASSERT!(Result == MMSYSERR_NOERROR as MMRESULT);

    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    if (*FunctionTable).QueryWaveFormatSupport.is_none() {
        return MMSYSERR_NOTSUPPORTED as MMRESULT;
    }

    return (*FunctionTable)
        .QueryWaveFormatSupport
        .unwrap_unchecked()(SoundDevice, Format, FormatSize);
}
#[no_mangle]
pub unsafe extern "C" fn SetWaveDeviceFormat(
    mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
    mut DeviceId: DWORD,
    mut Format: LPWAVEFORMATEX,
    mut FormatSize: DWORD,
) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut();
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();

    SND_TRACE!(RUSTCONV_WSTR_27ADBB33A7C5.as_ptr());

    if IsValidSoundDeviceInstance(SoundDeviceInstance) == 0 {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));
    if !(Result == MMSYSERR_NOERROR as MMRESULT) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);
    if (DeviceType) as u32 == WAVE_IN_DEVICE_TYPE || (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE {
        if Format.is_null() {
            return MMSYSERR_INVALPARAM as MMRESULT;
        }
        if !(FormatSize as usize >= core::mem::size_of::<WAVEFORMATEX>() as usize) {
            return MMSYSERR_INVALPARAM as MMRESULT;
        }
    }

    /* Ensure we have a wave device (TODO: check if this applies to wavein as well) */
    if !(((IS_WAVE_DEVICE_TYPE!(DeviceType)) != 0) || ((IS_MIDI_DEVICE_TYPE!(DeviceType)) != 0) || ((IS_MIXER_DEVICE_TYPE!(DeviceType)) != 0)) {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }

    /* Obtain the function table */
    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    if !(Result == MMSYSERR_NOERROR as MMRESULT) {
        return TranslateInternalMmResult(Result);
    }

    if (*FunctionTable).SetWaveFormat.is_none() {
        return MMSYSERR_NOTSUPPORTED;
    }

    return (*FunctionTable)
        .SetWaveFormat
        .unwrap_unchecked()(
        SoundDeviceInstance as *mut _SOUND_DEVICE_INSTANCE,
        DeviceId,
        Format as PWAVEFORMATEX,
        FormatSize,
    );
}
