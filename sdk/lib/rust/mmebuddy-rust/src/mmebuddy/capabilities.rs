// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0A5AA9D8995A: [u16; 41] = [
    0x0044u16, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0063, 0x0061, 0x0070,
    0x0061, 0x0062, 0x0069, 0x006C, 0x0069, 0x0074, 0x0069, 0x0065, 0x0073, 0x0020,
    0x0073, 0x0074, 0x0072, 0x0075, 0x0063, 0x0074, 0x0075, 0x0072, 0x0065, 0x0020,
    0x0074, 0x006F, 0x006F, 0x0020, 0x0073, 0x006D, 0x0061, 0x006C, 0x006C, 0x000A,
    0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn GetSoundDeviceCapabilities(SoundDevice: PSOUND_DEVICE, DeviceId: DWORD, Capabilities: PVOID, CapabilitiesSize: DWORD) -> MMRESULT {
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut();
    let mut GoodSize: BOOLEAN = (FALSE as u8);
    let mut Result: MMRESULT = 0;

    VALIDATE_MMSYS_PARAMETER!( IsValidSoundDevice(SoundDevice) );
    VALIDATE_MMSYS_PARAMETER!( !Capabilities.is_null() );
    VALIDATE_MMSYS_PARAMETER!( CapabilitiesSize > 0 );

    /* Obtain the device type */
    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!( Result == MMSYSERR_NOERROR );

    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    /* Obtain the function table */
    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    SND_ASSERT!( Result == MMSYSERR_NOERROR );

    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    SND_ASSERT!( IS_VALID_SOUND_DEVICE_TYPE!(DeviceType) );

    /* Check that the capabilities structure is of a valid size */
    match (DeviceType) as u32 {
        WAVE_OUT_DEVICE_TYPE => {
            GoodSize = (CapabilitiesSize >= core::mem::size_of::<WAVEOUTCAPSW>() as DWORD) as BOOLEAN;
        }
        WAVE_IN_DEVICE_TYPE => {
            GoodSize = (CapabilitiesSize >= core::mem::size_of::<WAVEINCAPSW>() as DWORD) as BOOLEAN;
        }
        MIDI_OUT_DEVICE_TYPE => {
            GoodSize = (CapabilitiesSize >= core::mem::size_of::<MIDIOUTCAPSW>() as DWORD) as BOOLEAN;
        }
        MIDI_IN_DEVICE_TYPE => {
            GoodSize = (CapabilitiesSize >= core::mem::size_of::<MIDIINCAPSW>() as DWORD) as BOOLEAN;
        }
        AUX_DEVICE_TYPE => {
            GoodSize = (CapabilitiesSize >= core::mem::size_of::<AUXCAPSW>() as DWORD) as BOOLEAN;
        }
        MIXER_DEVICE_TYPE => {
            GoodSize = (CapabilitiesSize >= core::mem::size_of::<MIXERCAPSW>() as DWORD) as BOOLEAN;
        }
        _ => {}
    }

    if ((GoodSize) == 0) {
        SND_ERR!(RUSTCONV_WSTR_0A5AA9D8995A.as_ptr());
        return MMSYSERR_INVALPARAM;
    }

    /* Call the "get capabilities" function within the function table */
    SND_ASSERT!( (*FunctionTable).__bindgen_anon_1.GetCapabilities.is_some() );

    if (*FunctionTable).__bindgen_anon_1.GetCapabilities.is_none() {
        return MMSYSERR_NOTSUPPORTED;
    }

    return (*FunctionTable).__bindgen_anon_1.GetCapabilities.unwrap_unchecked()(
        SoundDevice,
        DeviceId,
        Capabilities,
        CapabilitiesSize,
    );
}
