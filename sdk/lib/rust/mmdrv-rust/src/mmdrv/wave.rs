use crate::mmdrv::session::CallSessionThread;
use crate::mmdrv::kernel::GetDeviceData;
use crate::mmdrv::mme::NotifyClient;
use crate::mmdrv::kernel::SetDeviceData;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_WAVE_BUFFER_SIZE: u32 = 65536;
#[no_mangle]
pub unsafe extern "C" fn QueueWaveBuffer(
    session_info: *mut SessionInfo,
    wave_header: LPWAVEHDR,
) -> MMRESULT {
    let mut queue_node: PWAVEHDR;
    let mut previous_node: PWAVEHDR;
    DPRINT!("Queueing wave buffer\n");

    if wave_header.is_null() {
        return MMSYSERR_INVALPARAM;
    }

    if (*wave_header).lpData.is_null() {
        return MMSYSERR_INVALPARAM;
    }

    /* Headers must be prepared first */
    if !((*wave_header).dwFlags & WHDR_PREPARED != 0) {
        DPRINT!("I was given a header which hasn't been prepared yet!\n");
        return WAVERR_UNPREPARED;
    }

    /* ...and they must not already be in the playing queue! */
    if (*wave_header).dwFlags & WHDR_INQUEUE != 0 {
        DPRINT!("I was given a header for a buffer which is already playing\n");
        return WAVERR_STILLPLAYING;
    }

    /* Initialize */
    (*wave_header).dwBytesRecorded = 0;

    /* Clear the DONE bit, and mark the buffer as queued */
    (*wave_header).dwFlags &= !WHDR_DONE;
    (*wave_header).dwFlags |= WHDR_INQUEUE;

    /* Save our handle in the header */
    (*wave_header).reserved = session_info as DWORD_PTR;

    /* Locate the end of the queue */
    previous_node = core::ptr::null_mut();
    queue_node = (*session_info).__bindgen_anon_2.wave_queue;

    while !queue_node.is_null() {
        previous_node = queue_node;
        queue_node = (*queue_node).lpNext;
    }

    /* Go back a step to obtain the previous node (non-NULL) */
    queue_node = previous_node;

    /* Append our buffer here, and terminate the queue */
    (*queue_node).lpNext = wave_header;
    (*wave_header).lpNext = core::ptr::null_mut();

    /* When no buffers are playing there's no play queue so we start one */

    /* Pass to the driver - happens automatically during playback */
//    return PerformWaveIO(session_info);
    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn ReturnCompletedBuffers(session_info: *mut SessionInfo) {
    let mut header: PWAVEHDR = core::ptr::null_mut();
    loop {
        header = (*session_info).__bindgen_anon_2.wave_queue;
        if header.is_null() {
            break;
        }
        if ((*header).dwFlags & WHDR_DONE) != 0 {
            let mut message: DWORD = 0;

            /* Mark as done, and unqueued */
            (*header).dwFlags &= !WHDR_INQUEUE;
            (*header).dwFlags |= WHDR_DONE;

            /* Trim it from the start of the queue */
            (*session_info).__bindgen_anon_2.wave_queue = (*header).lpNext as PWAVEHDR;

            /* Choose appropriate notification */
            message = if (*session_info).device_type == WaveOutDevice {
                WOM_DONE
            } else {
                WIM_DATA
            };

            DPRINT!("Notifying client that buffer 0x%p is done\n", header);

            /* Notify the client */
            NotifyClient(session_info, message, header as DWORD_PTR, 0);
        }
    }

    /* TODO: Perform I/O as a new buffer may have arrived */
}
#[no_mangle]
pub unsafe extern "C" fn ProcessSessionThreadRequest(session_info: *mut SessionInfo) -> DWORD {
    let mut result: MMRESULT = MMSYSERR_NOERROR as MMRESULT;

    match (*session_info).thread.function {
        WODM_WRITE => {
            result = QueueWaveBuffer(session_info, (*session_info).thread.parameter as LPWAVEHDR);
        }

        WODM_RESET => {
            /* TODO */
        }

        WODM_PAUSE => {
            /* TODO */
        }

        WODM_RESTART => {
            /* TODO */
        }

        WODM_GETPOS => {
            /* TODO */
        }

        WODM_SETPITCH => {
            result = SetDeviceData(
                (*session_info).kernel_device_handle,
                IOCTL_WAVE_SET_PITCH as DWORD,
                (*session_info).thread.parameter as PBYTE,
                ::core::mem::size_of::<DWORD>() as DWORD,
            );
        }

        WODM_GETPITCH => {
            result = GetDeviceData(
                (*session_info).kernel_device_handle,
                IOCTL_WAVE_GET_PITCH as DWORD,
                (*session_info).thread.parameter as PBYTE,
                ::core::mem::size_of::<DWORD>() as DWORD,
            );
        }

        WODM_SETVOLUME => {}

        WODM_GETVOLUME => {}

        WODM_SETPLAYBACKRATE => {
            result = SetDeviceData(
                (*session_info).kernel_device_handle,
                IOCTL_WAVE_SET_PLAYBACK_RATE as DWORD,
                (*session_info).thread.parameter as PBYTE,
                ::core::mem::size_of::<DWORD>() as DWORD,
            );
        }

        WODM_GETPLAYBACKRATE => {
            result = GetDeviceData(
                (*session_info).kernel_device_handle,
                IOCTL_WAVE_GET_PLAYBACK_RATE as DWORD,
                (*session_info).thread.parameter as PBYTE,
                ::core::mem::size_of::<DWORD>() as DWORD,
            );
        }

        WODM_CLOSE => {
            DPRINT!("Thread was asked if OK to close device\n");

            if !(*session_info).__bindgen_anon_2.wave_queue.is_null() {
                result = WAVERR_STILLPLAYING as MMRESULT;
            } else {
                result = MMSYSERR_NOERROR as MMRESULT;
            }
        }

        DRVM_TERMINATE => {
            DPRINT!("Terminating thread...\n");
            result = MMSYSERR_NOERROR as MMRESULT;
        }

        _ => {
            DPRINT!("INVALID FUNCTION\n");
            result = MMSYSERR_ERROR as MMRESULT;
        }
    }

    /* We're done with the function now */

    return result as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn WaveThread(parameter: LPVOID) -> DWORD {
    let mut result: MMRESULT = MMSYSERR_ERROR;
    let mut session_info: *mut SessionInfo = parameter as *mut SessionInfo;
    let mut terminate: BOOL = (FALSE as i32);

    /* All your CPU time are belong to us */
    SetThreadPriority(GetCurrentThread(), (THREAD_PRIORITY_TIME_CRITICAL as i32));

    DPRINT!("Wave processing thread setting ready state\n");

    SetEvent((*session_info).thread.ready_event);

    while terminate == 0 {
        /* Wait for GO event, or IO completion notification */
        while WaitForSingleObjectEx((*session_info).thread.go_event,
                                     INFINITE,
                                     (TRUE) as BOOL) == WAIT_IO_COMPLETION
        {
            /* A buffer has been finished with - pass back to the client */
            ReturnCompletedBuffers(session_info);
        }

        DPRINT!("Wave processing thread woken up\n");

        /* Set the terminate flag if that's what the caller wants */
        terminate = ((*session_info).thread.function == DRVM_TERMINATE) as BOOL;

        /* Process the request */
        DPRINT!("Processing thread request\n");
        result = ProcessSessionThreadRequest(session_info) as MMRESULT;

        /* Store the result code */
        (*session_info).thread.result = result;

        /* Submit new buffers and continue existing ones */
        DPRINT!("Performing wave I/O\n");
        PerformWaveIO(session_info);

        /* Now we're ready for more action */
        DPRINT!("Wave processing thread sleeping\n");
        SetEvent((*session_info).thread.ready_event);
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn GetWaveFormatExSize(format: PWAVEFORMATEX) -> DWORD {
    if (*format).wFormatTag == WAVE_FORMAT_PCM as WORD {
        return core::mem::size_of::<PCMWAVEFORMAT>() as DWORD;
    } else {
        return core::mem::size_of::<WAVEFORMATEX>() as DWORD + (*format).cbSize as DWORD;
    }
}

#[no_mangle]
pub unsafe extern "C" fn QueryWaveFormat(device_type: DeviceType, lpFormat: PVOID) -> DWORD {
    WAVERR_BADFORMAT
}
#[no_mangle]
pub unsafe extern "C" fn SetWaveFormat(device_handle: HANDLE, format: PWAVEFORMATEX) -> BOOL {
    let mut bytes_returned: DWORD = 0;
    let mut size: DWORD = 0;

    size = GetWaveFormatExSize(format);

    DPRINT!("SetWaveFormat\n");

    return DeviceIoControl(
        device_handle,
        IOCTL_WAVE_SET_FORMAT as DWORD,
        format as PVOID,
        size,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::addr_of_mut!(bytes_returned),
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn WriteWaveBuffer(
    mut private_handle: DWORD_PTR,
    mut wave_header: PWAVEHDR,
    mut wave_header_size: DWORD,
) -> DWORD {
    let mut session_info: *mut SessionInfo = private_handle as *mut SessionInfo;
    ASSERT!(!session_info.is_null());

    /* Let the processing thread know that it has work to do */
    return CallSessionThread(
        session_info,
        WODM_WRITE as ThreadFunction,
        wave_header as PVOID,
    ) as DWORD;
}
