use crate::mmebuddy::devicelist::IsValidSoundDevice;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_07BF2245669A: [u16; 36] = [
    0x0044u16, 0x0065, 0x0073, 0x0074, 0x0072, 0x006F, 0x0079, 0x0069, 0x006E, 0x0067,
    0x0020, 0x0061, 0x0020, 0x0073, 0x006F, 0x0075, 0x006E, 0x0064, 0x0020, 0x0064,
    0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0069, 0x006E, 0x0073, 0x0074,
    0x0061, 0x006E, 0x0063, 0x0065, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_70F998CEBFA7: [u16; 71] = [
    0x0053u16, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x004D, 0x004D,
    0x0045, 0x0020, 0x0064, 0x0061, 0x0074, 0x0061, 0x0020, 0x002D, 0x0020, 0x0068,
    0x0061, 0x006E, 0x0064, 0x006C, 0x0065, 0x0020, 0x0025, 0x0078, 0x002C, 0x0020,
    0x0063, 0x0061, 0x006C, 0x006C, 0x0062, 0x0061, 0x0063, 0x006B, 0x0020, 0x0025,
    0x0078, 0x002C, 0x0020, 0x0069, 0x006E, 0x0073, 0x0074, 0x0061, 0x006E, 0x0063,
    0x0065, 0x0020, 0x0064, 0x0061, 0x0074, 0x0061, 0x0020, 0x0025, 0x0078, 0x002C,
    0x0020, 0x0066, 0x006C, 0x0061, 0x0067, 0x0073, 0x0020, 0x0025, 0x0078, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_9551A29671AC: [u16; 31] = [
    0x004Cu16, 0x0069, 0x0073, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0073, 0x006F,
    0x0075, 0x006E, 0x0064, 0x0020, 0x0064, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0020, 0x0069, 0x006E, 0x0073, 0x0074, 0x0061, 0x006E, 0x0063, 0x0065, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_9CD45096E7AD: [u16; 33] = [
    0x0055u16, 0x006E, 0x006C, 0x0069, 0x0073, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020,
    0x0073, 0x006F, 0x0075, 0x006E, 0x0064, 0x0020, 0x0064, 0x0065, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0020, 0x0069, 0x006E, 0x0073, 0x0074, 0x0061, 0x006E, 0x0063,
    0x0065, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_E5B345C50913: [u16; 34] = [
    0x0043u16, 0x0072, 0x0065, 0x0061, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0061,
    0x0020, 0x0073, 0x006F, 0x0075, 0x006E, 0x0064, 0x0020, 0x0064, 0x0065, 0x0076,
    0x0069, 0x0063, 0x0065, 0x0020, 0x0069, 0x006E, 0x0073, 0x0074, 0x0061, 0x006E,
    0x0063, 0x0065, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const SOUND_KERNEL_BUFFER_COUNT: i32 = 10;

pub const SOUND_KERNEL_BUFFER_SIZE: i32 = 16384;

#[no_mangle]
pub unsafe extern "C" fn AllocateSoundDeviceInstance(
    mut SoundDeviceInstance: *mut PSOUND_DEVICE_INSTANCE,
) -> MMRESULT {
    let mut NewInstance: PSOUND_DEVICE_INSTANCE;

    VALIDATE_MMSYS_PARAMETER!(SoundDeviceInstance);

    /* Allocate memory for the new instance */
    NewInstance = AllocateStruct!(core::mem::size_of::<SOUND_DEVICE_INSTANCE>()) as PSOUND_DEVICE_INSTANCE;

    if NewInstance.is_null() {
        return MMSYSERR_NOMEM as MMRESULT;
    }

    /* Use default frame size */
    (*NewInstance).FrameSize = SOUND_KERNEL_BUFFER_SIZE as DWORD;
    /* Use default buffer count */
    (*NewInstance).BufferCount = SOUND_KERNEL_BUFFER_COUNT as DWORD;

    /* Provide the caller with the new instance pointer */
    *SoundDeviceInstance = NewInstance;

    return MMSYSERR_NOERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn FreeSoundDeviceInstance(SoundDeviceInstance: PSOUND_DEVICE_INSTANCE) {
    /*
        Device is marked as invalid by now, but we can still do some sanity
        checking.
    */
    SND_ASSERT!((*SoundDeviceInstance).Thread.is_null());

    ZeroMemory!(SoundDeviceInstance as PVOID, core::mem::size_of::<SOUND_DEVICE_INSTANCE>());
    FreeMemory(SoundDeviceInstance as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn IsValidSoundDeviceInstance(mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE) -> BOOLEAN {
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut CurrentInstance: PSOUND_DEVICE_INSTANCE = core::ptr::null_mut();

    if SoundDeviceInstance.is_null() {
        return (FALSE) as BOOLEAN;
    }

    /* GetSoundDeviceFromInstance would send us into a recursive loop... */
    SoundDevice = (*SoundDeviceInstance).Device as PSOUND_DEVICE;
    SND_ASSERT!(SoundDevice);

    CurrentInstance = (*SoundDevice).HeadInstance as PSOUND_DEVICE_INSTANCE;

    while !CurrentInstance.is_null() {
        if CurrentInstance == SoundDeviceInstance {
            return (TRUE) as BOOLEAN;
        }

        CurrentInstance = (*CurrentInstance).Next as PSOUND_DEVICE_INSTANCE;
    }

    return (FALSE) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn ListSoundDeviceInstance(
    mut SoundDevice: PSOUND_DEVICE,
    mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
) -> MMRESULT {
    VALIDATE_MMSYS_PARAMETER!(IsValidSoundDevice(SoundDevice));
    VALIDATE_MMSYS_PARAMETER!(SoundDeviceInstance);

    SND_TRACE!(RUSTCONV_WSTR_9551A29671AC.as_ptr());

    if (*SoundDevice).HeadInstance.is_null() {
        /* First entry - assign to head and tail */
        (*SoundDevice).HeadInstance = SoundDeviceInstance;
        (*SoundDevice).TailInstance = SoundDeviceInstance;
    } else {
        /* Attach to the end */
        (*(*SoundDevice).TailInstance).Next = SoundDeviceInstance;
        (*SoundDevice).TailInstance = SoundDeviceInstance;
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn UnlistSoundDeviceInstance(
    SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut SoundDevice: PSOUND_DEVICE = core::mem::zeroed();
    let mut CurrentInstance: PSOUND_DEVICE_INSTANCE;
    let mut PreviousInstance: PSOUND_DEVICE_INSTANCE;

    let Valid = IsValidSoundDeviceInstance(SoundDeviceInstance);
    VALIDATE_MMSYS_PARAMETER!(Valid != 0);
    if Valid == 0 {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }

    SND_TRACE!(RUSTCONV_WSTR_9CD45096E7AD.as_ptr());

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, &raw mut SoundDevice);
    SND_ASSERT!(MMSUCCESS!(Result));
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    PreviousInstance = core::ptr::null_mut::<_SOUND_DEVICE_INSTANCE>();
    CurrentInstance = (*SoundDevice).HeadInstance as PSOUND_DEVICE_INSTANCE;

    while !CurrentInstance.is_null() {
        if CurrentInstance == SoundDeviceInstance {
            if PreviousInstance.is_null() {
                /* This is the head node */
                (*SoundDevice).HeadInstance = (*(*SoundDevice).HeadInstance).Next;
            } else {
                /* There are nodes before this one - cut ours out */
                (*PreviousInstance).Next = (*CurrentInstance).Next;
            }

            if (*CurrentInstance).Next.is_null() {
                /* This is the tail node */
                (*SoundDevice).TailInstance = PreviousInstance as *mut _SOUND_DEVICE_INSTANCE;
            }
        }

        PreviousInstance = CurrentInstance;
        CurrentInstance = (*CurrentInstance).Next as PSOUND_DEVICE_INSTANCE;
    }

    return MMSYSERR_NOERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn CreateSoundDeviceInstance(mut SoundDevice: PSOUND_DEVICE, mut SoundDeviceInstance: *mut PSOUND_DEVICE_INSTANCE) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut();

    SND_TRACE!(RUSTCONV_WSTR_E5B345C50913.as_ptr());

    VALIDATE_MMSYS_PARAMETER!(IsValidSoundDevice(SoundDevice));
    VALIDATE_MMSYS_PARAMETER!(!SoundDeviceInstance.is_null());

    Result = AllocateSoundDeviceInstance(SoundDeviceInstance);

    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    /* Get the "open" routine from the function table, and validate it */
    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    if ((MMSUCCESS!(Result)) == 0) {
        FreeSoundDeviceInstance(*SoundDeviceInstance);
        return TranslateInternalMmResult(Result);
    }

    if (*FunctionTable).Open.is_none() {
        FreeSoundDeviceInstance(*SoundDeviceInstance);
        return MMSYSERR_NOTSUPPORTED;
    }

    /* Set up the members of the structure */
    (**SoundDeviceInstance).Next = core::ptr::null_mut();
    (**SoundDeviceInstance).Device = SoundDevice as *mut _SOUND_DEVICE;
    (**SoundDeviceInstance).Handle = core::ptr::null_mut();
    (**SoundDeviceInstance).Thread = core::ptr::null_mut();

    (**SoundDeviceInstance).WinMM.Handle = (INVALID_HANDLE_VALUE).cast();
    (**SoundDeviceInstance).WinMM.ClientCallback = 0;
    (**SoundDeviceInstance).WinMM.ClientCallbackInstanceData = 0;
    (**SoundDeviceInstance).WinMM.Flags = 0;

    /* Initialise the members of the struct used by the sound thread */
    (**SoundDeviceInstance).__bindgen_anon_1.HeadWaveHeader = core::ptr::null_mut();
    (**SoundDeviceInstance).__bindgen_anon_2.TailWaveHeader = core::ptr::null_mut();

    (**SoundDeviceInstance).OutstandingBuffers = 0;

    (**SoundDeviceInstance).LoopsRemaining = 0;

    /* Create the streaming thread (TODO - is this for wave only?) */
    Result = CreateSoundThread(core::ptr::addr_of_mut!((**SoundDeviceInstance).Thread));
    if ((MMSUCCESS!(Result)) == 0) {
        FreeSoundDeviceInstance(*SoundDeviceInstance);
        return TranslateInternalMmResult(Result);
    }

    /* Add the instance to the list */
    Result = ListSoundDeviceInstance(SoundDevice, *SoundDeviceInstance);
    if ((MMSUCCESS!(Result)) == 0) {
        FreeSoundDeviceInstance(*SoundDeviceInstance);
        return TranslateInternalMmResult(Result);
    }

    /* Try and open the device */
    Result = (*FunctionTable).Open.unwrap_unchecked()(
        SoundDevice as *mut _SOUND_DEVICE,
        core::ptr::addr_of_mut!((**SoundDeviceInstance).Handle),
    );
    if ((MMSUCCESS!(Result)) == 0) {
        UnlistSoundDeviceInstance(*SoundDeviceInstance);
        FreeSoundDeviceInstance(*SoundDeviceInstance);
        return TranslateInternalMmResult(Result);
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn DestroySoundDeviceInstance(mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut::<_MMFUNCTION_TABLE>();
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut::<_SOUND_DEVICE>();
    let mut Handle: PVOID = core::ptr::null_mut::<core::ffi::c_void>();

    SND_TRACE!(RUSTCONV_WSTR_07BF2245669A.as_ptr());

    VALIDATE_MMSYS_PARAMETER!(IsValidSoundDeviceInstance(SoundDeviceInstance));

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, &raw mut SoundDevice);
    if (((MMSUCCESS!(Result))) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceInstanceHandle(SoundDeviceInstance, &raw mut Handle);
    if (((MMSUCCESS!(Result))) == 0) {
        return TranslateInternalMmResult(Result);
    }

    /* Get the "close" routine from the function table, and validate it */
    Result = GetSoundDeviceFunctionTable(SoundDevice, &raw mut FunctionTable);
    if (((MMSUCCESS!(Result))) == 0) {
        return TranslateInternalMmResult(Result);
    }

    SND_ASSERT!((*FunctionTable).Close.is_some());
    if (*FunctionTable).Close.is_none() {
        /* This indicates bad practice, really! If you can open, why not close?! */
        return MMSYSERR_NOTSUPPORTED;
    }

    /* Stop the streaming thread */
    if !(*SoundDeviceInstance).Thread.is_null() {
        Result = DestroySoundThread((*SoundDeviceInstance).Thread as PSOUND_THREAD);
        SND_ASSERT!(MMSUCCESS!(Result)); /* It should succeed! */
        if (((MMSUCCESS!(Result))) == 0) {
            return TranslateInternalMmResult(Result);
        }
    }

    /* Blank this out here */
    (*SoundDeviceInstance).Thread = core::ptr::null_mut::<_SOUND_THREAD>();

    /* Try and close the device */
    Result = (*FunctionTable).Close.unwrap_unchecked()(
        SoundDeviceInstance,
        Handle,
    );
    SND_ASSERT!(MMSUCCESS!(Result)); /* It should succeed! */
    if (((MMSUCCESS!(Result))) == 0) {
        return TranslateInternalMmResult(Result);
    }

    /* Drop it from the list */
    Result = UnlistSoundDeviceInstance(SoundDeviceInstance);
    SND_ASSERT!(MMSUCCESS!(Result)); /* It should succeed! */
    if (((MMSUCCESS!(Result))) == 0) {
        return TranslateInternalMmResult(Result);
    }

    FreeSoundDeviceInstance(SoundDeviceInstance);

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn DestroyAllSoundDeviceInstances(SoundDevice: PSOUND_DEVICE) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE;
    let mut NextDeviceInstance: PSOUND_DEVICE_INSTANCE;

    SoundDeviceInstance = (*SoundDevice).HeadInstance;

    while !SoundDeviceInstance.is_null() {
        NextDeviceInstance = (*SoundDeviceInstance).Next;
        Result = DestroySoundDeviceInstance(SoundDeviceInstance);
        SND_ASSERT!(MMSUCCESS!(Result));
        SoundDeviceInstance = NextDeviceInstance;
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn GetSoundDeviceFromInstance(mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE, mut SoundDevice: *mut PSOUND_DEVICE) -> MMRESULT {
    VALIDATE_MMSYS_PARAMETER!(IsValidSoundDeviceInstance(SoundDeviceInstance));
    VALIDATE_MMSYS_PARAMETER!(SoundDevice);
    *SoundDevice = (*SoundDeviceInstance).Device;

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn GetSoundDeviceInstanceHandle(mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE, mut Handle: *mut PVOID) -> MMRESULT {
    if IsValidSoundDeviceInstance(SoundDeviceInstance) == 0 {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }
    if Handle.is_null() {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }
    *Handle = (*SoundDeviceInstance).Handle;
    return MMSYSERR_NOERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn SetSoundDeviceInstanceMmeData(SoundDeviceInstance: PSOUND_DEVICE_INSTANCE, MmeHandle: HDRVR, ClientCallback: DWORD_PTR, ClientCallbackData: DWORD_PTR, Flags: DWORD) -> MMRESULT {
    VALIDATE_MMSYS_PARAMETER!(IsValidSoundDeviceInstance(SoundDeviceInstance) != 0);

    SND_TRACE!(RUSTCONV_WSTR_70F998CEBFA7.as_ptr(), MmeHandle, ClientCallback, ClientCallbackData, Flags);

    (*SoundDeviceInstance).WinMM.Handle = MmeHandle;
    (*SoundDeviceInstance).WinMM.ClientCallback = ClientCallback;
    (*SoundDeviceInstance).WinMM.ClientCallbackInstanceData = ClientCallbackData;
    (*SoundDeviceInstance).WinMM.Flags = Flags;

    return MMSYSERR_NOERROR;
}
