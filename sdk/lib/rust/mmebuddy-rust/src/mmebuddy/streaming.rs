use crate::mmebuddy::thread::CallSoundThread;
use crate::mmebuddy::header::CompleteWaveHeader;
use crate::mmebuddy::utility::FreeMemory;
use crate::mmebuddy::deviceinstance::GetSoundDeviceFromInstance;
use crate::mmebuddy::functiontable::GetSoundDeviceFunctionTable;
use crate::mmebuddy::deviceinstance::GetSoundDeviceInstanceHandle;
use crate::mmebuddy::devicelist::GetSoundDeviceType;
use crate::mmebuddy::deviceinstance::IsValidSoundDeviceInstance;
use crate::mmebuddy::utility::TranslateInternalMmResult;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_067109C93D63: [u16; 64] = [
    0x004Cu16, 0x006F, 0x006F, 0x0070, 0x0069, 0x006E, 0x0067, 0x0020, 0x006D, 0x0075,
    0x006C, 0x0074, 0x0069, 0x0070, 0x006C, 0x0065, 0x0020, 0x0068, 0x0065, 0x0061,
    0x0064, 0x0065, 0x0072, 0x0073, 0x0020, 0x0069, 0x0073, 0x0020, 0x0055, 0x004E,
    0x0049, 0x004D, 0x0050, 0x004C, 0x0045, 0x004D, 0x0045, 0x004E, 0x0054, 0x0045,
    0x0044, 0x002E, 0x0020, 0x0057, 0x0069, 0x006C, 0x006C, 0x0020, 0x0070, 0x006C,
    0x0061, 0x0079, 0x0020, 0x006F, 0x006E, 0x0063, 0x0065, 0x0020, 0x006F, 0x006E,
    0x006C, 0x0079, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_28D447F5BB18: [u16; 40] = [
    0x004Cu16, 0x006F, 0x006F, 0x0070, 0x0069, 0x006E, 0x0067, 0x0020, 0x0074, 0x0068,
    0x0065, 0x0020, 0x0068, 0x0065, 0x0061, 0x0064, 0x0065, 0x0072, 0x002C, 0x0020,
    0x0072, 0x0065, 0x006D, 0x0061, 0x0069, 0x006E, 0x0069, 0x006E, 0x0067, 0x0020,
    0x006C, 0x006F, 0x006F, 0x0070, 0x0073, 0x0020, 0x0025, 0x0075, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_689C7B1F23F4: [u16; 8] = [0x0046u16, 0x0041, 0x0049, 0x004C, 0x0045, 0x0044, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_81177169CA21: [u16; 50] = [
    0x0053u16, 0x0074, 0x006F, 0x0070, 0x0053, 0x0074, 0x0072, 0x0065, 0x0061, 0x006D,
    0x0069, 0x006E, 0x0067, 0x0049, 0x006E, 0x0053, 0x006F, 0x0075, 0x006E, 0x0064,
    0x0054, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x003A, 0x0020, 0x0043, 0x006F,
    0x006D, 0x0070, 0x006C, 0x0065, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0048,
    0x0065, 0x0061, 0x0064, 0x0065, 0x0072, 0x0020, 0x0025, 0x0070, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_95B84553FC47: [u16; 55] = [
    0x0053u16, 0x0074, 0x006F, 0x0070, 0x0053, 0x0074, 0x0072, 0x0065, 0x0061, 0x006D,
    0x0069, 0x006E, 0x0067, 0x0049, 0x006E, 0x0053, 0x006F, 0x0075, 0x006E, 0x0064,
    0x0054, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x004F, 0x0075, 0x0074,
    0x0053, 0x0074, 0x0061, 0x006E, 0x0064, 0x0069, 0x006E, 0x0067, 0x0042, 0x0075,
    0x0066, 0x0066, 0x0065, 0x0072, 0x0043, 0x006F, 0x0075, 0x006E, 0x0074, 0x0020,
    0x0025, 0x006C, 0x0075, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_95EBFAD14685: [u16; 34] = [
    0x0025u16, 0x0075, 0x002F, 0x0025, 0x0075, 0x0020, 0x0062, 0x0079, 0x0074, 0x0065,
    0x0073, 0x0020, 0x006F, 0x0066, 0x0020, 0x0077, 0x0061, 0x0076, 0x0065, 0x0068,
    0x0064, 0x0072, 0x0020, 0x0063, 0x006F, 0x006D, 0x0070, 0x006C, 0x0065, 0x0074,
    0x0065, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_A67604FF0522: [u16; 48] = [
    0x0044u16, 0x006F, 0x0057, 0x0061, 0x0076, 0x0065, 0x0053, 0x0074, 0x0072, 0x0065,
    0x0061, 0x006D, 0x0069, 0x006E, 0x0067, 0x003A, 0x0020, 0x004E, 0x006F, 0x0020,
    0x0077, 0x006F, 0x0072, 0x006B, 0x0020, 0x0074, 0x006F, 0x0020, 0x0064, 0x006F,
    0x0020, 0x002D, 0x0020, 0x0064, 0x006F, 0x0069, 0x006E, 0x0067, 0x0020, 0x006E,
    0x006F, 0x0074, 0x0068, 0x0069, 0x006E, 0x0067, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_CA5DE0C16D36: [u16; 70] = [
    0x0044u16, 0x006F, 0x0057, 0x0061, 0x0076, 0x0065, 0x0053, 0x0074, 0x0072, 0x0065,
    0x0061, 0x006D, 0x0069, 0x006E, 0x0067, 0x003A, 0x0020, 0x004E, 0x006F, 0x0020,
    0x0061, 0x0076, 0x0061, 0x0069, 0x006C, 0x0061, 0x0062, 0x006C, 0x0065, 0x0020,
    0x0062, 0x0075, 0x0066, 0x0066, 0x0065, 0x0072, 0x0073, 0x0020, 0x0074, 0x006F,
    0x0020, 0x0073, 0x0074, 0x0072, 0x0065, 0x0061, 0x006D, 0x0020, 0x0077, 0x0069,
    0x0074, 0x0068, 0x0020, 0x002D, 0x0020, 0x0064, 0x006F, 0x0069, 0x006E, 0x0067,
    0x0020, 0x006E, 0x006F, 0x0074, 0x0068, 0x0069, 0x006E, 0x0067, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_E02460FA282D: [u16; 34] = [
    0x0025u16, 0x0064, 0x002F, 0x0025, 0x0064, 0x0020, 0x0062, 0x0079, 0x0074, 0x0065,
    0x0073, 0x0020, 0x006F, 0x0066, 0x0020, 0x0077, 0x0061, 0x0076, 0x0065, 0x0068,
    0x0064, 0x0072, 0x0020, 0x0063, 0x006F, 0x006D, 0x0070, 0x006C, 0x0065, 0x0074,
    0x0065, 0x0064, 0x000A, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn DoWaveStreaming(SoundDeviceInstance: PSOUND_DEVICE_INSTANCE) {
    let mut Result: MMRESULT = 0;
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut();
    let mut Header: PWAVEHDR = core::ptr::null_mut();
    let mut HeaderExtension: PWAVEHDR_EXTENSION = core::ptr::null_mut();

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));
    SND_ASSERT!(MMSUCCESS!(Result));

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(MMSUCCESS!(Result));

    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    SND_ASSERT!(MMSUCCESS!(Result));
    SND_ASSERT!(!FunctionTable.is_null());
    SND_ASSERT!((*FunctionTable).CommitWaveBuffer.is_some());

    /* No point in doing anything if no resources available to use */
    if (*SoundDeviceInstance).OutstandingBuffers >= (*SoundDeviceInstance).BufferCount {
        SND_TRACE!(RUSTCONV_WSTR_CA5DE0C16D36.as_ptr());
        return;
    }

    /* Is there any work to do? */
    Header = (*SoundDeviceInstance).__bindgen_anon_1.HeadWaveHeader;

    if Header.is_null() {
        SND_TRACE!(RUSTCONV_WSTR_A67604FF0522.as_ptr());
        return;
    }

    // Divergence from streaming.c: the C reloaded `LoopsRemaining = dwLoops` here whenever
    // the HEAD header was BEGINLOOP|ENDLOOP, and CompleteIO calls this after every
    // completion, so a finite loop never ran down. The count is now set once per
    // WODM_WRITE (InitializeWaveHeaderLoops) and kept per header.

    while ((*SoundDeviceInstance).OutstandingBuffers < (*SoundDeviceInstance).BufferCount)
        && (!Header.is_null())
        && (*SoundDeviceInstance).ResetInProgress == (FALSE as i32)
    {
        HeaderExtension = (*Header).reserved as PWAVEHDR_EXTENSION;
        SND_ASSERT!(!HeaderExtension.is_null());

        /* Saniy checks */
        SND_ASSERT!(((*Header).dwFlags & (WHDR_PREPARED as DWORD)) != 0);
        SND_ASSERT!(((*Header).dwFlags & (WHDR_INQUEUE as DWORD)) != 0);

        /* Can never be *above* the length */
        SND_ASSERT!((*HeaderExtension).BytesCommitted <= (*Header).dwBufferLength);

        /* Is this header entirely committed? */
        if (*HeaderExtension).BytesCommitted == (*Header).dwBufferLength {
            if (*HeaderExtension).LoopsToCommit != 0 {
                // Divergence from streaming.c: the C rewound a looped header from
                // CompleteIO, by which time this loop had already moved on and committed
                // the NEXT header, so the repeat played after it. Rewind here, so each
                // pass is committed directly behind the one before.
                (*HeaderExtension).BytesCommitted = 0;
                if (*HeaderExtension).LoopsToCommit != INFINITE {
                    (*HeaderExtension).LoopsToCommit -= 1;
                }
                SND_TRACE!(
                    RUSTCONV_WSTR_28D447F5BB18.as_ptr(),
                    (*HeaderExtension).LoopsToCommit
                );
            } else {
                /* Move on to the next header */
                SND_ASSERT!(Header != (*Header).lpNext);
                Header = (*Header).lpNext;
            }
        } else {
            let mut Overlap: PSOUND_OVERLAPPED = core::ptr::null_mut();
            let mut OffsetPtr: LPVOID = core::ptr::null_mut();
            let mut BytesRemaining: DWORD = 0;
            let mut BytesToCommit: DWORD = 0;
            let mut OK: BOOL = 0;

            /* Where within the header buffer to stream from */
            OffsetPtr = (*Header)
                .lpData
                .offset((*HeaderExtension).BytesCommitted as isize)
                as LPVOID;

            /* How much of this header has not been committed */
            BytesRemaining = (*Header)
                .dwBufferLength
                .wrapping_sub((*HeaderExtension).BytesCommitted);

            /* We can commit anything up to the buffer size limit */
            BytesToCommit = if BytesRemaining > (*SoundDeviceInstance).FrameSize {
                (*SoundDeviceInstance).FrameSize
            } else {
                BytesRemaining
            };

            /* Should always have something to commit by this point */
            SND_ASSERT!(BytesToCommit > 0);

            /* We need a new overlapped info structure for each buffer */
            Overlap = AllocateStruct!(core::mem::size_of::<SOUND_OVERLAPPED>()) as PSOUND_OVERLAPPED;

            if !Overlap.is_null() {
                ZeroMemory!(Overlap as PVOID, core::mem::size_of::<SOUND_OVERLAPPED>() as usize);
                (*Overlap).SoundDeviceInstance = SoundDeviceInstance;
                (*Overlap).Header = Header;

                /* Adjust the commit-related counters */
                (*HeaderExtension).BytesCommitted = (*HeaderExtension)
                    .BytesCommitted
                    .wrapping_add(BytesToCommit);
                (*SoundDeviceInstance).OutstandingBuffers =
                    (*SoundDeviceInstance).OutstandingBuffers.wrapping_add(1);

                OK = MMSUCCESS!((*FunctionTable)
                    .CommitWaveBuffer
                    .unwrap_unchecked()(
                    SoundDeviceInstance,
                    OffsetPtr as PVOID,
                    BytesToCommit,
                    Overlap,
                    Some(CompleteIO),
                ));

                if OK == 0 {
                    /* Clean-up and try again on the next iteration (is this OK?) */
                    SND_WARN!(RUSTCONV_WSTR_689C7B1F23F4.as_ptr());

                    FreeMemory(Overlap as PVOID);
                    (*HeaderExtension).BytesCommitted = (*HeaderExtension)
                        .BytesCommitted
                        .wrapping_sub(BytesToCommit);
                    (*SoundDeviceInstance).OutstandingBuffers =
                        (*SoundDeviceInstance).OutstandingBuffers.wrapping_sub(1);
                }
            }
        }
    }
}
/// Set a header's repeat counts when it is written. Not in streaming.c, whose loop count
/// lived in the device instance and was reloaded on every streaming pass (see
/// DoWaveStreaming). `dwLoops` is the number of times to play the loop (WAVEHDR), so
/// 0 and 1 both play once; the C repeated `dwLoops` extra times, which no client could
/// observe because its count never ran down. INFINITE loops until reset.
pub unsafe fn InitializeWaveHeaderLoops(DeviceType: MMDEVICE_TYPE, Header: PWAVEHDR) {
    let Extension: PWAVEHDR_EXTENSION = (*Header).reserved as PWAVEHDR_EXTENSION;
    SND_ASSERT!(!Extension.is_null());

    let mut Repeats: DWORD = 0;
    if (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE
        && (((*Header).dwFlags & (WHDR_BEGINLOOP as DWORD)) != 0)
    {
        if ((*Header).dwFlags & (WHDR_ENDLOOP as DWORD)) != 0 {
            Repeats = if (*Header).dwLoops == INFINITE {
                INFINITE
            } else {
                (*Header).dwLoops.saturating_sub(1)
            };
        } else {
            /* Report and help notice such a case */
            SND_WARN!(RUSTCONV_WSTR_067109C93D63.as_ptr());
            SND_ASSERT!(
                ((*Header).dwFlags & ((WHDR_BEGINLOOP as DWORD) | (WHDR_ENDLOOP as DWORD)))
                    == ((WHDR_BEGINLOOP as DWORD) | (WHDR_ENDLOOP as DWORD))
            );
        }
    }
    (*Extension).LoopsToCommit = Repeats;
    (*Extension).LoopsToComplete = Repeats;
}
#[no_mangle]
pub unsafe extern "system" fn CompleteIO(
    dwErrorCode: DWORD,
    mut dwNumberOfBytesTransferred: DWORD,
    lpOverlapped: LPOVERLAPPED,
) {
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE = core::ptr::null_mut();
    let SoundOverlapped: PSOUND_OVERLAPPED = lpOverlapped as PSOUND_OVERLAPPED;
    let mut WaveHdr: PWAVEHDR = core::ptr::null_mut();
    let mut HdrExtension: PWAVEHDR_EXTENSION = core::ptr::null_mut();
    let mut Result: MMRESULT = 0;
    let mut Bytes: DWORD = 0;

    WaveHdr = (*SoundOverlapped).Header as PWAVEHDR;
    SND_ASSERT!(!WaveHdr.is_null());

    HdrExtension = (*WaveHdr).reserved as PWAVEHDR_EXTENSION;
    SND_ASSERT!(!HdrExtension.is_null());

    SoundDeviceInstance = (*SoundOverlapped).SoundDeviceInstance as PSOUND_DEVICE_INSTANCE;

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));
    SND_ASSERT!(MMSUCCESS!(Result));

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(MMSUCCESS!(Result));

    // Divergence from streaming.c, which (a) dropped the bytes of the completion that
    // triggered a loop rewind while partial completions kept adding to BytesCompleted
    // across passes, so a later pass could complete the header with its own chunks still
    // in flight, and (b) decremented OutstandingBuffers once per header the completion
    // spilled into, though one completion frees one buffer. Completion now counts each
    // pass to dwBufferLength and wraps while repeats remain, so the header is returned
    // only after its last pass's last byte.

    /* We have an available buffer now */
    (*SoundDeviceInstance).OutstandingBuffers =
        (*SoundDeviceInstance).OutstandingBuffers.wrapping_sub(1);

    loop {
        Bytes = (*WaveHdr)
            .dwBufferLength
            .wrapping_sub((*HdrExtension).BytesCompleted);
        if Bytes > dwNumberOfBytesTransferred {
            Bytes = dwNumberOfBytesTransferred;
        }
        (*HdrExtension).BytesCompleted = (*HdrExtension).BytesCompleted.wrapping_add(Bytes);
        dwNumberOfBytesTransferred = dwNumberOfBytesTransferred.wrapping_sub(Bytes);

        if (*HdrExtension).BytesCompleted < (*WaveHdr).dwBufferLength {
            /* Partially completed */
            SND_TRACE!(
                RUSTCONV_WSTR_95EBFAD14685.as_ptr(),
                (*HdrExtension).BytesCompleted,
                (*WaveHdr).dwBufferLength
            );
            break;
        }

        if (*HdrExtension).LoopsToComplete != 0 {
            /* One pass of a looped header done; the next is already being committed */
            (*HdrExtension).BytesCompleted = 0;
            if (*HdrExtension).LoopsToComplete != INFINITE {
                (*HdrExtension).LoopsToComplete -= 1;
            }
            SND_TRACE!(
                RUSTCONV_WSTR_28D447F5BB18.as_ptr(),
                (*HdrExtension).LoopsToComplete
            );
        } else {
            /* Wave buffer fully completed. Take the link first: the client is
               notified, and may requeue the header, before CompleteWaveHeader returns. */
            let Next: PWAVEHDR = (*WaveHdr).lpNext as PWAVEHDR;
            SND_TRACE!(
                RUSTCONV_WSTR_E02460FA282D.as_ptr(),
                (*HdrExtension).BytesCompleted,
                (*WaveHdr).dwBufferLength
            );
            CompleteWaveHeader(SoundDeviceInstance, WaveHdr);
            WaveHdr = Next;
        }

        if dwNumberOfBytesTransferred == 0 {
            break;
        }

        /* Move to next wave header */
        if WaveHdr.is_null() {
            /* No following WaveHdr */
            SND_ASSERT!(dwNumberOfBytesTransferred == 0);
            break;
        }

        HdrExtension = (*WaveHdr).reserved as PWAVEHDR_EXTENSION;
        SND_ASSERT!(!HdrExtension.is_null());
    }

    // AUDIO-BRANCH DIFF
    // completion callback is performed in a thread
    DoWaveStreaming(SoundDeviceInstance);

    //CompleteWavePortion(SoundDeviceInstance, dwNumberOfBytesTransferred);

    FreeMemory(lpOverlapped as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn WriteFileEx_Committer(
    mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
    mut OffsetPtr: PVOID,
    mut Length: DWORD,
    mut Overlap: PSOUND_OVERLAPPED,
    mut CompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE,
) -> MMRESULT {
    let mut Handle: HANDLE = core::ptr::null_mut();

    // VALIDATE_MMSYS_PARAMETER!( SoundDeviceInstance );
    if SoundDeviceInstance.is_null() {
        return 11 as MMRESULT; // MMSYSERR_INVALPARAM
    }
    // VALIDATE_MMSYS_PARAMETER!( OffsetPtr );
    if OffsetPtr.is_null() {
        return 11 as MMRESULT; // MMSYSERR_INVALPARAM
    }
    // VALIDATE_MMSYS_PARAMETER!( Overlap );
    if Overlap.is_null() {
        return 11 as MMRESULT; // MMSYSERR_INVALPARAM
    }
    // VALIDATE_MMSYS_PARAMETER!( CompletionRoutine );
    if CompletionRoutine.is_none() {
        return 11 as MMRESULT; // MMSYSERR_INVALPARAM
    }

    GetSoundDeviceInstanceHandle(SoundDeviceInstance, core::ptr::addr_of_mut!(Handle));

    if WriteFileEx(Handle, OffsetPtr, Length, Overlap as LPOVERLAPPED, CompletionRoutine) == 0
    {
        // TODO
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn StopStreamingInSoundThread(
    SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
    Parameter: PVOID,
) -> MMRESULT {
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut FunctionTable: PMMFUNCTION_TABLE = core::ptr::null_mut();
    let mut Result: MMRESULT = 0;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();

    /* set state reset in progress */
    (*SoundDeviceInstance).ResetInProgress = (TRUE as i32);

    /* Get sound device */
    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    /* Obtain the function table */
    Result = GetSoundDeviceFunctionTable(SoundDevice, core::ptr::addr_of_mut!(FunctionTable));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    /* Obtain device instance type */
    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    /* Check if reset function is supported */
    if (*FunctionTable).ResetStream.is_some() {
        /* cancel all current audio buffers */
        (*FunctionTable).ResetStream.unwrap_unchecked()(SoundDeviceInstance, DeviceType, (TRUE as u8));
    }
    while (*SoundDeviceInstance).OutstandingBuffers != 0 {
        SND_TRACE!(RUSTCONV_WSTR_95B84553FC47.as_ptr(), (*SoundDeviceInstance).OutstandingBuffers);
        /* wait until pending i/o has completed */
        SleepEx(10, (TRUE) as BOOL);
    }

    /* complete all current headers */
    while !(*SoundDeviceInstance).__bindgen_anon_1.HeadWaveHeader.is_null() {
        SND_TRACE!(RUSTCONV_WSTR_81177169CA21.as_ptr(), (*SoundDeviceInstance).__bindgen_anon_1.HeadWaveHeader);
        CompleteWaveHeader(SoundDeviceInstance, (*SoundDeviceInstance).__bindgen_anon_1.HeadWaveHeader);
    }

    /* there should be no oustanding buffers now */
    SND_ASSERT!((*SoundDeviceInstance).OutstandingBuffers == 0);

    /* Check if reset function is supported */
    if (*FunctionTable).ResetStream.is_some() {
        /* finish the reset */
        (*FunctionTable).ResetStream.unwrap_unchecked()(SoundDeviceInstance, DeviceType, (FALSE as u8));
    }

    /* clear state reset in progress */
    (*SoundDeviceInstance).ResetInProgress = (FALSE as i32);

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn StopStreaming(SoundDeviceInstance: PSOUND_DEVICE_INSTANCE) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut DeviceType: MMDEVICE_TYPE = 0;

    if IsValidSoundDeviceInstance(SoundDeviceInstance) == 0 {
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

    if (DeviceType) as u32 != WAVE_OUT_DEVICE_TYPE && (DeviceType) as u32 != WAVE_IN_DEVICE_TYPE {
        return MMSYSERR_NOTSUPPORTED;
    }

    return CallSoundThread(
        SoundDeviceInstance,
        Some(StopStreamingInSoundThread as unsafe extern "C" fn(PSOUND_DEVICE_INSTANCE, PVOID) -> MMRESULT),
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn PerformWaveStreaming(
    mut SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
    mut Parameter: PVOID,
) -> MMRESULT {
    DoWaveStreaming(SoundDeviceInstance);

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "system" fn WaveActivateSoundStreaming(lpParameter: PVOID) -> DWORD {
    CallSoundThread(
        lpParameter as PSOUND_DEVICE_INSTANCE,
        Some(
            PerformWaveStreaming as unsafe extern "C" fn(PSOUND_DEVICE_INSTANCE, PVOID) -> MMRESULT,
        ),
        core::ptr::null_mut(),
    );

    ExitThread(0 as DWORD);
}
#[no_mangle]
pub unsafe extern "C" fn InitiateSoundStreaming(SoundDeviceInstance: PSOUND_DEVICE_INSTANCE) {
    let mut hThread: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    hThread = CreateThread(
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        0 as DWORD,
        Some(WaveActivateSoundStreaming),
        SoundDeviceInstance as PVOID,
        0 as DWORD,
        ::core::ptr::null_mut::<DWORD>(),
    );
    if !hThread.is_null() {
        CloseHandle(hThread);
    }
}
