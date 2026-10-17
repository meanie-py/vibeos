use crate::mmebuddy::thread::CallSoundThread;
use crate::mmebuddy::utility::FreeMemory;
use crate::mmebuddy::deviceinstance::GetSoundDeviceFromInstance;
use crate::mmebuddy::functiontable::GetSoundDeviceFunctionTable;
use crate::mmebuddy::devicelist::GetSoundDeviceType;
use crate::mmebuddy::deviceinstance::IsValidSoundDeviceInstance;
use crate::mmebuddy::mmewrap::NotifyMmeClient;
use crate::mmebuddy::utility::TranslateInternalMmResult;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_10E2E1251C1E: [u16; 31] = [
    0x0052u16, 0x0065, 0x0074, 0x0075, 0x0072, 0x006E, 0x0069, 0x006E, 0x0067, 0x0020,
    0x0062, 0x0075, 0x0066, 0x0066, 0x0065, 0x0072, 0x0020, 0x0074, 0x006F, 0x0020,
    0x0063, 0x006C, 0x0069, 0x0065, 0x006E, 0x0074, 0x002E, 0x002E, 0x002E, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_1CF0CB4C0A74: [u16; 20] = [
    0x0055u16, 0x0070, 0x0064, 0x0061, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0074,
    0x0061, 0x0069, 0x006C, 0x0020, 0x006E, 0x006F, 0x0064, 0x0065, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_295E13C4F96C: [u16; 27] = [
    0x0045u16, 0x006E, 0x0071, 0x0075, 0x0065, 0x0075, 0x0065, 0x0064, 0x0020, 0x006E,
    0x0065, 0x0078, 0x0074, 0x0020, 0x0077, 0x0061, 0x0076, 0x0065, 0x0020, 0x0068,
    0x0065, 0x0061, 0x0064, 0x0065, 0x0072, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_34410F2C2137: [u16; 23] = [
    0x0050u16, 0x0072, 0x0065, 0x0070, 0x0061, 0x0072, 0x0069, 0x006E, 0x0067, 0x0020,
    0x0077, 0x0061, 0x0076, 0x0065, 0x0020, 0x0068, 0x0065, 0x0061, 0x0064, 0x0065,
    0x0072, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_5609D3276282: [u16; 24] = [
    0x0053u16, 0x0075, 0x0062, 0x006D, 0x0069, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067,
    0x0020, 0x0077, 0x0061, 0x0076, 0x0065, 0x0020, 0x0068, 0x0065, 0x0061, 0x0064,
    0x0065, 0x0072, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_6DB90DCD4031: [u16; 20] = [
    0x0044u16, 0x0072, 0x006F, 0x0070, 0x0070, 0x0069, 0x006E, 0x0067, 0x0020, 0x0074,
    0x0061, 0x0069, 0x006C, 0x0020, 0x006E, 0x006F, 0x0064, 0x0065, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_87DFBB51B94D: [u16; 28] = [
    0x0045u16, 0x006E, 0x0071, 0x0075, 0x0065, 0x0075, 0x0065, 0x0064, 0x0020, 0x0066,
    0x0069, 0x0072, 0x0073, 0x0074, 0x0020, 0x0077, 0x0061, 0x0076, 0x0065, 0x0020,
    0x0068, 0x0065, 0x0061, 0x0064, 0x0065, 0x0072, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_9C3FC76734B9: [u16; 17] = [
    0x0052u16, 0x0065, 0x006C, 0x0069, 0x006E, 0x006B, 0x0069, 0x006E, 0x0067, 0x0020,
    0x006E, 0x006F, 0x0064, 0x0065, 0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_A2485ED22911: [u16; 20] = [
    0x0044u16, 0x0072, 0x006F, 0x0070, 0x0070, 0x0069, 0x006E, 0x0067, 0x0020, 0x0068,
    0x0065, 0x0061, 0x0064, 0x0020, 0x006E, 0x006F, 0x0064, 0x0065, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D58770028F81: [u16; 20] = [
    0x0042u16, 0x0055, 0x0046, 0x0046, 0x0045, 0x0052, 0x0020, 0x0043, 0x004F, 0x004D,
    0x0050, 0x004C, 0x0045, 0x0054, 0x0045, 0x0020, 0x003A, 0x0029, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_F9421C4DE1F9: [u16; 26] = [
    0x0055u16, 0x006E, 0x002D, 0x0070, 0x0072, 0x0065, 0x0070, 0x0061, 0x0072, 0x0069,
    0x006E, 0x0067, 0x0020, 0x0077, 0x0061, 0x0076, 0x0065, 0x0020, 0x0068, 0x0065,
    0x0061, 0x0064, 0x0065, 0x0072, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct THREADED_WAVEHEADER_PARAMETERS {
    pub Function: MMWAVEHEADER_FUNC,
    pub Header: PWAVEHDR,
}

pub type PTHREADED_WAVEHEADER_PARAMETERS = *mut THREADED_WAVEHEADER_PARAMETERS;
#[no_mangle]
pub unsafe extern "C" fn WaveHeaderOperationInSoundThread(mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE, mut Parameter: PVOID) -> MMRESULT {
    let mut Parameters: *mut THREADED_WAVEHEADER_PARAMETERS = Parameter as *mut THREADED_WAVEHEADER_PARAMETERS;
    return (*Parameters).Function.unwrap_unchecked()(
        SoundDeviceInstance as *mut _SOUND_DEVICE_INSTANCE,
        (*Parameters).Header,
    );
}
#[no_mangle]
pub unsafe extern "C" fn WaveHeaderOperation(
    mut Function: MMWAVEHEADER_FUNC,
    mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
    mut Header: PWAVEHDR,
) -> MMRESULT {
    let mut Parameters: THREADED_WAVEHEADER_PARAMETERS = THREADED_WAVEHEADER_PARAMETERS {
        Function: None,
        Header: core::ptr::null_mut::<wavehdr_tag>(),
    };
    Parameters.Function = Function;
    Parameters.Header = Header;
    return CallSoundThread(
        SoundDeviceInstance,
        Some(
            WaveHeaderOperationInSoundThread
                as unsafe extern "C" fn(PSOUND_DEVICE_INSTANCE, PVOID) -> MMRESULT,
        ),
        &raw mut Parameters as PVOID,
    );
}
#[no_mangle]
pub unsafe extern "C" fn SanitizeWaveHeader(Header: PWAVEHDR) {
    let mut Extension: PWAVEHDR_EXTENSION = (*Header).reserved as PWAVEHDR_EXTENSION;
    SND_ASSERT!(Extension);

    (*Header).dwBytesRecorded = 0;

    (*Extension).BytesCommitted = 0;
    (*Extension).BytesCompleted = 0;
}
#[no_mangle]
pub unsafe extern "C" fn PrepareWaveHeader(
    mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
    mut Header: PWAVEHDR,
) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut();
    let mut Extension: PWAVEHDR_EXTENSION = core::ptr::null_mut();

    if IsValidSoundDeviceInstance(SoundDeviceInstance) == 0 {
        return 11 as MMRESULT; // MMSYSERR_INVALPARAM
    }
    if Header.is_null() {
        return 11 as MMRESULT; // MMSYSERR_INVALPARAM
    }

    SND_TRACE!(RUSTCONV_WSTR_34410F2C2137.as_ptr());

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, &raw mut SoundDevice);
    if !(Result == 0 as MMRESULT) { // MMSYSERR_NOERROR
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceFunctionTable(SoundDevice, &raw mut FunctionTable);
    if !(Result == 0 as MMRESULT) { // MMSYSERR_NOERROR
        return TranslateInternalMmResult(Result);
    }

    Extension = AllocateStruct!(core::mem::size_of::<WAVEHDR_EXTENSION>()) as PWAVEHDR_EXTENSION;
    if Extension.is_null() {
        return MMSYSERR_NOMEM;
    }

    (*Header).reserved = Extension as DWORD_PTR;
    (*Extension).BytesCommitted = 0;
    (*Extension).BytesCompleted = 0;

    /* Configure the flags */
    (*Header).dwFlags |= WHDR_PREPARED;

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn UnprepareWaveHeader(
    mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
    mut Header: PWAVEHDR,
) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut();
    let mut Extension: PWAVEHDR_EXTENSION = core::ptr::null_mut();

    if IsValidSoundDeviceInstance(SoundDeviceInstance) == 0 {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }
    if Header.is_null() {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }

    SND_TRACE!(RUSTCONV_WSTR_F9421C4DE1F9.as_ptr());

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, &raw mut SoundDevice);
    if MMSUCCESS!(Result) == 0 {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceFunctionTable(SoundDevice, &raw mut FunctionTable);
    if MMSUCCESS!(Result) == 0 {
        return TranslateInternalMmResult(Result);
    }

    SND_ASSERT!((*Header).reserved as usize != 0);
    Extension = (*Header).reserved as PWAVEHDR_EXTENSION;
    FreeMemory(Extension as PVOID);

    /* Configure the flags */
    (*Header).dwFlags &= !WHDR_PREPARED as DWORD;

    return MMSYSERR_NOERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn WriteWaveHeader(
    SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
    Header: PWAVEHDR,
) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut SoundDevice: PSOUND_DEVICE = ::core::ptr::null_mut::<_SOUND_DEVICE>();
    let mut FunctionTable: PMMFUNCTION_TABLE = ::core::ptr::null_mut::<_MMFUNCTION_TABLE>();

    VALIDATE_MMSYS_PARAMETER!(IsValidSoundDeviceInstance(SoundDeviceInstance) != 0);
    VALIDATE_MMSYS_PARAMETER!(!Header.is_null());

    SND_TRACE!(RUSTCONV_WSTR_5609D3276282.as_ptr());

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    if (*FunctionTable).CommitWaveBuffer.is_none() {
        return MMSYSERR_NOTSUPPORTED;
    }

    /*
        A few minor sanity checks - any custom checks should've been carried
        out during wave header preparation etc.
    */
    VALIDATE_MMSYS_PARAMETER!(!(*Header).lpData.is_null());
    VALIDATE_MMSYS_PARAMETER!((*Header).dwBufferLength > 0);
    VALIDATE_MMSYS_PARAMETER!((*Header).dwFlags & WHDR_PREPARED != 0);
    VALIDATE_MMSYS_PARAMETER!(!((*Header).dwFlags & WHDR_INQUEUE != 0));

    SanitizeWaveHeader(Header);

    /* Clear the "done" flag for the buffer */
    (*Header).dwFlags &= !WHDR_DONE;

    Result = CallSoundThread(
        SoundDeviceInstance,
        Some(EnqueueWaveHeader as unsafe extern "C" fn(PSOUND_DEVICE_INSTANCE, PVOID) -> MMRESULT),
        Header as PVOID,
    );

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn EnqueueWaveHeader(mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE, mut Parameter: PVOID) -> MMRESULT {
    let mut WaveHeader: PWAVEHDR = Parameter as PWAVEHDR;

    VALIDATE_MMSYS_PARAMETER!(SoundDeviceInstance);
    VALIDATE_MMSYS_PARAMETER!(Parameter);

    /* Initialise */
    (*WaveHeader).lpNext = core::ptr::null_mut();

    /* Set the "in queue" flag */
    (*WaveHeader).dwFlags |= WHDR_INQUEUE;

    if !(*SoundDeviceInstance).__bindgen_anon_1.HeadWaveHeader.is_null() {
        /* There are already queued headers - make this one the tail */
        SND_TRACE!(RUSTCONV_WSTR_295E13C4F96C.as_ptr());

        /* FIXME - Make sure that the buffer has not already been added to the list */
        if (*SoundDeviceInstance).__bindgen_anon_2.TailWaveHeader != WaveHeader {
            SND_ASSERT!((*SoundDeviceInstance).__bindgen_anon_2.TailWaveHeader != WaveHeader);

            (*(*SoundDeviceInstance).__bindgen_anon_2.TailWaveHeader).lpNext = WaveHeader;
            (*SoundDeviceInstance).__bindgen_anon_2.TailWaveHeader = WaveHeader;
            DUMP_WAVEHDR_QUEUE!(SoundDeviceInstance);

            /* Only do wave streaming when the stream has not been paused */
            if (*SoundDeviceInstance).bPaused == (FALSE as i32) {
                DoWaveStreaming(SoundDeviceInstance);
            }
        }
    } else {
        /* This is the first header in the queue */
        SND_TRACE!(RUSTCONV_WSTR_87DFBB51B94D.as_ptr());
        (*SoundDeviceInstance).__bindgen_anon_1.HeadWaveHeader = WaveHeader;
        (*SoundDeviceInstance).__bindgen_anon_2.TailWaveHeader = WaveHeader;

        /* Only do wave streaming when the stream has not been paused */
        if (*SoundDeviceInstance).bPaused == (FALSE as i32) {
            DoWaveStreaming(SoundDeviceInstance);
        }
    }

    DUMP_WAVEHDR_QUEUE!(SoundDeviceInstance);

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn CompleteWaveHeader(SoundDeviceInstance: PSOUND_DEVICE_INSTANCE, Header: PWAVEHDR) {
    let mut PrevHdr: PWAVEHDR = core::ptr::null_mut();
    let mut CurrHdr: PWAVEHDR = core::ptr::null_mut();
    let mut Extension: PWAVEHDR_EXTENSION = core::ptr::null_mut();
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut Result: MMRESULT = 0;

    SND_TRACE!(RUSTCONV_WSTR_D58770028F81.as_ptr());

    // TODO: Set header flags?
    // TODO: Call client
    // TODO: Streaming

    //DoWaveStreaming(SoundDeviceInstance);

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, &mut SoundDevice);
    SND_ASSERT!(MMSUCCESS!(Result));
    Result = GetSoundDeviceType(SoundDevice, &mut DeviceType);
    SND_ASSERT!(MMSUCCESS!(Result));

    Extension = (*Header).reserved as PWAVEHDR_EXTENSION;
    SND_ASSERT!(Extension);

    /* Remove the header from the queue, like so */
    if (*SoundDeviceInstance).__bindgen_anon_1.HeadWaveHeader == Header {
        (*SoundDeviceInstance).__bindgen_anon_1.HeadWaveHeader = (*Header).lpNext;

        SND_TRACE!(RUSTCONV_WSTR_A2485ED22911.as_ptr());

        /* If nothing after the head, then there is no tail */
        if (*Header).lpNext.is_null() {
            SND_TRACE!(RUSTCONV_WSTR_6DB90DCD4031.as_ptr());
            (*SoundDeviceInstance).__bindgen_anon_2.TailWaveHeader = core::ptr::null_mut();
        }
    } else {
        PrevHdr = core::ptr::null_mut();
        CurrHdr = (*SoundDeviceInstance).__bindgen_anon_1.HeadWaveHeader;

        SND_TRACE!(RUSTCONV_WSTR_9C3FC76734B9.as_ptr());

        while CurrHdr != Header {
            PrevHdr = CurrHdr;
            CurrHdr = (*CurrHdr).lpNext;
            SND_ASSERT!(CurrHdr);
        }

        SND_ASSERT!(PrevHdr);

        (*PrevHdr).lpNext = (*CurrHdr).lpNext;

        /* If this is the tail node, update the tail */
        if (*Header).lpNext.is_null() {
            SND_TRACE!(RUSTCONV_WSTR_1CF0CB4C0A74.as_ptr());
            (*SoundDeviceInstance).__bindgen_anon_2.TailWaveHeader = PrevHdr;
        }
    }

    /* Make sure we're not using this as the current buffer any more, either! */
    /*
    if ( SoundDeviceInstance->CurrentWaveHeader == Header )
    {
        SoundDeviceInstance->CurrentWaveHeader = Header->lpNext;
    }
    */

    DUMP_WAVEHDR_QUEUE!(SoundDeviceInstance);

    SND_TRACE!(RUSTCONV_WSTR_10E2E1251C1E.as_ptr());

    /* Update the header */
    (*Header).dwFlags &= !WHDR_INQUEUE;
    (*Header).dwFlags |= WHDR_DONE;

    if (DeviceType) as u32 == WAVE_IN_DEVICE_TYPE {
        // FIXME: We won't be called on incomplete buffer!
        (*Header).dwBytesRecorded = (*Extension).BytesCompleted;
    }

    /* Safe to do this without thread protection, as we're done with the header */
    NotifyMmeClient(
        SoundDeviceInstance,
        (if (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE {
            WOM_DONE
        } else {
            WIM_DATA
        }) as UINT,
        Header as DWORD_PTR,
    );
}
