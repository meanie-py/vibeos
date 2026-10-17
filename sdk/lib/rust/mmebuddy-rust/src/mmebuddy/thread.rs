use crate::mmebuddy::utility::FreeMemory;
use crate::mmebuddy::deviceinstance::IsValidSoundDeviceInstance;
use crate::mmebuddy::utility::TranslateInternalMmResult;
use crate::mmebuddy::utility::Win32ErrorToMmResult;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0248063A37C8: [u16; 40] = [
    0x0053u16, 0x006F, 0x0075, 0x006E, 0x0064, 0x0054, 0x0068, 0x0072, 0x0065, 0x0061,
    0x0064, 0x0020, 0x002D, 0x0020, 0x0050, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073,
    0x0073, 0x0069, 0x006E, 0x0067, 0x0020, 0x0049, 0x004F, 0x0020, 0x0063, 0x006F,
    0x006D, 0x0070, 0x006C, 0x0065, 0x0074, 0x0069, 0x006F, 0x006E, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_0E2D85A38C07: [u16; 34] = [
    0x0053u16, 0x006F, 0x0075, 0x006E, 0x0064, 0x0054, 0x0068, 0x0072, 0x0065, 0x0061,
    0x0064, 0x0020, 0x002D, 0x0020, 0x0050, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073,
    0x0073, 0x0069, 0x006E, 0x0067, 0x0020, 0x0072, 0x0065, 0x0071, 0x0075, 0x0065,
    0x0073, 0x0074, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_3983C89640A6: [u16; 23] = [
    0x0053u16, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0052, 0x0045,
    0x0051, 0x0055, 0x0045, 0x0053, 0x0054, 0x0020, 0x0065, 0x0076, 0x0065, 0x006E,
    0x0074, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_3D55EB6DC9FA: [u16; 25] = [
    0x0057u16, 0x0061, 0x0069, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0066, 0x006F,
    0x0072, 0x0020, 0x0052, 0x0045, 0x0041, 0x0044, 0x0059, 0x0020, 0x0065, 0x0076,
    0x0065, 0x006E, 0x0074, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_4035DDD89DD8: [u16; 31] = [
    0x0053u16, 0x006F, 0x0075, 0x006E, 0x0064, 0x0020, 0x0074, 0x0068, 0x0072, 0x0065,
    0x0061, 0x0064, 0x0020, 0x0063, 0x0072, 0x0065, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0020, 0x0066, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0021, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_4A9FE65071B1: [u16; 25] = [
    0x0053u16, 0x006F, 0x0075, 0x006E, 0x0064, 0x0020, 0x0074, 0x0068, 0x0072, 0x0065,
    0x0061, 0x0064, 0x0020, 0x0074, 0x0065, 0x0072, 0x006D, 0x0069, 0x006E, 0x0061,
    0x0074, 0x0065, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_4D95C6256FB0: [u16; 40] = [
    0x0053u16, 0x006F, 0x0075, 0x006E, 0x0064, 0x0020, 0x0074, 0x0068, 0x0072, 0x0065,
    0x0061, 0x0064, 0x0020, 0x0074, 0x0065, 0x0072, 0x006D, 0x0069, 0x006E, 0x0061,
    0x0074, 0x006F, 0x0072, 0x0020, 0x0072, 0x006F, 0x0075, 0x0074, 0x0069, 0x006E,
    0x0065, 0x0020, 0x0063, 0x0061, 0x006C, 0x006C, 0x0065, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_54E53D507BBF: [u16; 24] = [
    0x0043u16, 0x0072, 0x0065, 0x0061, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0074,
    0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x0065, 0x0076, 0x0065, 0x006E,
    0x0074, 0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_5C1FE0F7D125: [u16; 26] = [
    0x0044u16, 0x0065, 0x0073, 0x0074, 0x0072, 0x006F, 0x0079, 0x0069, 0x006E, 0x0067,
    0x0020, 0x0074, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x0065, 0x0076,
    0x0065, 0x006E, 0x0074, 0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_63CBD0227209: [u16; 25] = [
    0x0046u16, 0x0069, 0x006E, 0x0069, 0x0073, 0x0068, 0x0065, 0x0064, 0x0020, 0x0074,
    0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x0063, 0x006C, 0x0065, 0x0061,
    0x006E, 0x0075, 0x0070, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_B24DD43588F6: [u16; 24] = [
    0x0057u16, 0x0061, 0x0069, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0066, 0x006F,
    0x0072, 0x0020, 0x0044, 0x004F, 0x004E, 0x0045, 0x0020, 0x0065, 0x0076, 0x0065,
    0x006E, 0x0074, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_B256063B1A5F: [u16; 35] = [
    0x0053u16, 0x006F, 0x0075, 0x006E, 0x0064, 0x0054, 0x0068, 0x0072, 0x0065, 0x0061,
    0x0064, 0x0020, 0x002D, 0x0020, 0x0043, 0x0061, 0x006D, 0x0065, 0x0020, 0x006F,
    0x0075, 0x0074, 0x0020, 0x006F, 0x0066, 0x0020, 0x0077, 0x0061, 0x0069, 0x0074,
    0x0069, 0x006E, 0x0067, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_CF15974B4E8F: [u16; 26] = [
    0x0054u16, 0x0065, 0x0072, 0x006D, 0x0069, 0x006E, 0x0061, 0x0074, 0x0069, 0x006E,
    0x0067, 0x0020, 0x0073, 0x006F, 0x0075, 0x006E, 0x0064, 0x0020, 0x0074, 0x0068,
    0x0072, 0x0065, 0x0061, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D133C654F1CF: [u16; 65] = [
    0x0053u16, 0x006F, 0x0075, 0x006E, 0x0064, 0x0020, 0x0074, 0x0068, 0x0072, 0x0065,
    0x0061, 0x0064, 0x0020, 0x0074, 0x0065, 0x0072, 0x006D, 0x0069, 0x006E, 0x0061,
    0x0074, 0x0065, 0x0064, 0x002C, 0x0020, 0x0070, 0x0065, 0x0072, 0x0066, 0x006F,
    0x0072, 0x006D, 0x0069, 0x006E, 0x0067, 0x0020, 0x0063, 0x006C, 0x0065, 0x0061,
    0x006E, 0x0075, 0x0070, 0x0020, 0x006F, 0x0066, 0x0020, 0x0074, 0x0068, 0x0072,
    0x0065, 0x0061, 0x0064, 0x0020, 0x0072, 0x0065, 0x0073, 0x006F, 0x0075, 0x0072,
    0x0063, 0x0065, 0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D62D98D7D7CC: [u16; 24] = [
    0x0053u16, 0x006F, 0x0075, 0x006E, 0x0064, 0x0054, 0x0068, 0x0072, 0x0065, 0x0061,
    0x0064, 0x0020, 0x0072, 0x0075, 0x006E, 0x006E, 0x0069, 0x006E, 0x0067, 0x0020,
    0x003A, 0x0029, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D74410EC9169: [u16; 26] = [
    0x0046u16, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020,
    0x0072, 0x0065, 0x0073, 0x0075, 0x006D, 0x0065, 0x0020, 0x0074, 0x0068, 0x0072,
    0x0065, 0x0061, 0x0064, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_E294E959ACD2: [u16; 25] = [
    0x0043u16, 0x0072, 0x0065, 0x0061, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0061,
    0x0020, 0x0073, 0x006F, 0x0075, 0x006E, 0x0064, 0x0020, 0x0074, 0x0068, 0x0072,
    0x0065, 0x0061, 0x0064, 0x000A, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn SoundThreadMain(lpParameter: LPVOID) -> DWORD {
    let mut Thread: PSOUND_THREAD = lpParameter as PSOUND_THREAD;

    SND_TRACE!(RUSTCONV_WSTR_D62D98D7D7CC.as_ptr());

    /* Callers will wait for us to be ready */
    (*Thread).Running = (TRUE as i32);
    SetEvent((*Thread).Events.Ready);

    while (*Thread).Running != 0 {
        let mut WaitResult: DWORD;

        /* Wait for a request, or an I/O completion */
        WaitResult = WaitForSingleObjectEx((*Thread).Events.Request, INFINITE, (TRUE) as BOOL);
        SND_TRACE!(RUSTCONV_WSTR_B256063B1A5F.as_ptr());

        if WaitResult == WAIT_OBJECT_0 {
            SND_TRACE!(RUSTCONV_WSTR_0E2D85A38C07.as_ptr());

            if (*Thread).Request.Handler.is_some() {
                (*Thread).Request.Result = (*Thread)
                    .Request
                    .Handler
                    .unwrap_unchecked()(
                    (*Thread).Request.SoundDeviceInstance,
                    (*Thread).Request.Parameter,
                );
            } else {
                (*Thread).Request.Result = MMSYSERR_ERROR;
            }

            /* Announce completion of the request */
            SetEvent((*Thread).Events.Done);
            /* Accept new requests */
            SetEvent((*Thread).Events.Ready);
        } else if WaitResult == WAIT_IO_COMPLETION {
            SND_TRACE!(RUSTCONV_WSTR_0248063A37C8.as_ptr());
            /* TODO? What do we do here? Stream stuff? */
        } else {
            /* This should not happen! */
            SND_ASSERT!(FALSE);
        }
    }

    SND_TRACE!(RUSTCONV_WSTR_4A9FE65071B1.as_ptr());

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn CallSoundThread(
        SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
        RequestHandler: SOUND_THREAD_REQUEST_HANDLER,
        Parameter: PVOID,
) -> MMRESULT {
    let mut Thread: PSOUND_THREAD;

    VALIDATE_MMSYS_PARAMETER!(IsValidSoundDeviceInstance(SoundDeviceInstance) != 0);
    VALIDATE_MMSYS_PARAMETER!(!RequestHandler.is_none());

    Thread = (*SoundDeviceInstance).Thread;

    SND_TRACE!(RUSTCONV_WSTR_3D55EB6DC9FA.as_ptr());
    WaitForSingleObject((*Thread).Events.Ready, INFINITE);

    (*Thread).Request.Result = MMSYSERR_NOTSUPPORTED;
    (*Thread).Request.Handler = RequestHandler;
    (*Thread).Request.SoundDeviceInstance = SoundDeviceInstance;
    (*Thread).Request.Parameter = Parameter;

    /* Notify the thread it has work to do */
    SND_TRACE!(RUSTCONV_WSTR_3983C89640A6.as_ptr());
    SetEvent((*Thread).Events.Request);

    /* Wait for the work to be done */
    SND_TRACE!(RUSTCONV_WSTR_B24DD43588F6.as_ptr());
    WaitForSingleObject((*Thread).Events.Done, INFINITE);

    return (*Thread).Request.Result;
}
#[no_mangle]
pub unsafe extern "C" fn SoundThreadTerminator(
    Instance: PSOUND_DEVICE_INSTANCE,
    Parameter: PVOID,
) -> MMRESULT {
    let mut Thread: PSOUND_THREAD = Parameter as PSOUND_THREAD;

    SND_TRACE!(RUSTCONV_WSTR_4D95C6256FB0.as_ptr());
    SND_ASSERT!(Thread);

    (*Thread).Running = (FALSE as i32);

    return MMSYSERR_NOERROR;
}

#[no_mangle]
pub unsafe extern "C" fn TerminateSoundThread(Thread: PSOUND_THREAD) -> MMRESULT {
    let mut WaitResult: DWORD;

    SND_ASSERT!(!Thread.is_null());

    SND_TRACE!(RUSTCONV_WSTR_3D55EB6DC9FA.as_ptr());
    WaitForSingleObject((*Thread).Events.Ready, INFINITE);

    (*Thread).Request.Result = MMSYSERR_NOTSUPPORTED as MMRESULT;
    (*Thread).Request.Handler = Some(
        SoundThreadTerminator as unsafe extern "C" fn(PSOUND_DEVICE_INSTANCE, PVOID) -> MMRESULT,
    ) as SOUND_THREAD_REQUEST_HANDLER;
    (*Thread).Request.SoundDeviceInstance = core::ptr::null_mut();
    (*Thread).Request.Parameter = Thread as PVOID;

    /* Notify the thread it has work to do */
    SND_TRACE!(RUSTCONV_WSTR_3983C89640A6.as_ptr());
    SetEvent((*Thread).Events.Request);

    /* Wait for the work to be done */
    SND_TRACE!(RUSTCONV_WSTR_B24DD43588F6.as_ptr());
    WaitForSingleObject((*Thread).Events.Done, INFINITE);

    /* Wait for the thread to actually end */
    WaitResult = WaitForSingleObject((*Thread).Handle, INFINITE);
    SND_ASSERT!(WaitResult == WAIT_OBJECT_0);

    return MMSYSERR_NOERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn CreateSoundThreadEvents(
    ReadyEvent: *mut HANDLE,
    RequestEvent: *mut HANDLE,
    DoneEvent: *mut HANDLE,
) -> MMRESULT {
    let mut ok: BOOL;

    // VALIDATE_MMSYS_PARAMETER!( ReadyEvent ); (MMSYSERR_INVALPARAM = 11)
    if ReadyEvent.is_null() {
        return 11; // MMSYSERR_INVALPARAM
    }
    // VALIDATE_MMSYS_PARAMETER!( RequestEvent );
    if RequestEvent.is_null() {
        return 11; // MMSYSERR_INVALPARAM
    }
    // VALIDATE_MMSYS_PARAMETER!( DoneEvent );
    if DoneEvent.is_null() {
        return 11; // MMSYSERR_INVALPARAM
    }

    SND_TRACE!(RUSTCONV_WSTR_54E53D507BBF.as_ptr());

    /* Initialise these so we can identify them upon failure */
    *ReadyEvent = INVALID_HANDLE_VALUE;
    *RequestEvent = INVALID_HANDLE_VALUE;
    *DoneEvent = INVALID_HANDLE_VALUE;

    *ReadyEvent = CreateEvent(core::ptr::null_mut(), (FALSE as i32), (FALSE as i32), core::ptr::null());
    ok = (*ReadyEvent != INVALID_HANDLE_VALUE) as BOOL;
    *RequestEvent = CreateEvent(core::ptr::null_mut(), (FALSE as i32), (FALSE as i32), core::ptr::null());
    ok &= (*RequestEvent != INVALID_HANDLE_VALUE) as BOOL;
    *DoneEvent = CreateEvent(core::ptr::null_mut(), (FALSE as i32), (FALSE as i32), core::ptr::null());
    ok &= (*DoneEvent != INVALID_HANDLE_VALUE) as BOOL;

    /* If something went wrong, clean up */
    if ok == 0 {
        if *ReadyEvent != INVALID_HANDLE_VALUE {
            CloseHandle(*ReadyEvent);
        }

        if *RequestEvent != INVALID_HANDLE_VALUE {
            CloseHandle(*RequestEvent);
        }

        if *DoneEvent != INVALID_HANDLE_VALUE {
            CloseHandle(*DoneEvent);
        }

        return MMSYSERR_NOMEM;
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn DestroySoundThreadEvents(
    ReadyEvent: HANDLE,
    RequestEvent: HANDLE,
    DoneEvent: HANDLE,
) -> MMRESULT {
    VALIDATE_MMSYS_PARAMETER!(ReadyEvent != INVALID_HANDLE_VALUE);
    VALIDATE_MMSYS_PARAMETER!(RequestEvent != INVALID_HANDLE_VALUE);
    VALIDATE_MMSYS_PARAMETER!(DoneEvent != INVALID_HANDLE_VALUE);

    SND_TRACE!(RUSTCONV_WSTR_5C1FE0F7D125.as_ptr());

    CloseHandle(ReadyEvent);
    CloseHandle(RequestEvent);
    CloseHandle(DoneEvent);

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn CreateSoundThread(Thread: *mut PSOUND_THREAD) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut NewThread: PSOUND_THREAD;

    VALIDATE_MMSYS_PARAMETER!(Thread);

    NewThread = AllocateStruct!(core::mem::size_of::<SOUND_THREAD>()) as PSOUND_THREAD;
    if NewThread.is_null() {
        return MMSYSERR_NOMEM;
    }

    /* Prepare the events we'll be using to sync. everything */
    Result = CreateSoundThreadEvents(
        core::ptr::addr_of_mut!((*NewThread).Events.Ready),
        core::ptr::addr_of_mut!((*NewThread).Events.Request),
        core::ptr::addr_of_mut!((*NewThread).Events.Done),
    );

    if ((MMSUCCESS!(Result)) == 0) {
        FreeMemory(NewThread as LPVOID);
        return TranslateInternalMmResult(Result);
    }

    SND_TRACE!(RUSTCONV_WSTR_E294E959ACD2.as_ptr());
    (*NewThread).Handle = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(SoundThreadMain),
        NewThread as LPVOID,
        CREATE_SUSPENDED,
        core::ptr::null_mut(),
    );

    /* Something went wrong, bail out! */
    if (*NewThread).Handle == INVALID_HANDLE_VALUE {
        SND_ERR!(RUSTCONV_WSTR_4035DDD89DD8.as_ptr());
        DestroySoundThreadEvents(
            (*NewThread).Events.Ready,
            (*NewThread).Events.Request,
            (*NewThread).Events.Done,
        );

        FreeMemory(NewThread as LPVOID);

        return Win32ErrorToMmResult(GetLastError() as UINT);
    }

    /* Wake the thread up */
    if ResumeThread((*NewThread).Handle) == (-(1 as i32)) as DWORD {
        SND_ERR!(RUSTCONV_WSTR_D74410EC9169.as_ptr());
        CloseHandle((*NewThread).Handle);
        DestroySoundThreadEvents(
            (*NewThread).Events.Ready,
            (*NewThread).Events.Request,
            (*NewThread).Events.Done,
        );

        FreeMemory(NewThread as LPVOID);
        return Win32ErrorToMmResult(GetLastError() as UINT);
    }

    /* If all is well we can now give the thread to the caller */
    *Thread = NewThread;
    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn DestroySoundThread(Thread: PSOUND_THREAD) -> MMRESULT {
    VALIDATE_MMSYS_PARAMETER!(Thread);
    if Thread.is_null() {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }
    SND_ASSERT!((*Thread).Handle != INVALID_HANDLE_VALUE);

    SND_TRACE!(RUSTCONV_WSTR_CF15974B4E8F.as_ptr());

    /* Tell the thread to terminate itself */
    TerminateSoundThread(Thread);

    SND_TRACE!(RUSTCONV_WSTR_D133C654F1CF.as_ptr());

    CloseHandle((*Thread).Handle);    /* Is this needed? */
    (*Thread).Handle = INVALID_HANDLE_VALUE;

    DestroySoundThreadEvents(
        (*Thread).Events.Ready,
        (*Thread).Events.Request,
        (*Thread).Events.Done,
    );

    /* Wipe and free the memory used for the thread */
    ZeroMemory!(Thread as *mut core::ffi::c_void, core::mem::size_of::<SOUND_THREAD>() as usize);
    FreeMemory(Thread as LPVOID);

    SND_TRACE!(RUSTCONV_WSTR_63CBD0227209.as_ptr());

    return MMSYSERR_NOERROR as MMRESULT;
}
