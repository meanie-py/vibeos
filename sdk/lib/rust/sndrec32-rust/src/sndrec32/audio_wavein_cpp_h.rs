use crate::prelude::*;
use core::ffi::c_void;













impl audio_wavein {
    pub unsafe fn recording_procedure(&mut self) -> DWORD {
        let this_: *mut Self = self;
        let mut msg: MSG = core::mem::zeroed();
        let mut phdr: *mut WAVEHDR;
        let _this: *mut audio_wavein = this_;

        /* Check the arg pointer */
        if _this == core::ptr::null_mut() {
            return 0;
        }

        /* The thread can go to sleep for now. It will be wake up only when
           there is audio data to be recorded */
        if (*_this).wakeup_recthread as usize != 0 {
            WaitForSingleObject((*_this).wakeup_recthread, INFINITE);
        }

        /* If status of the `audio_wavein' object is not ready or recording the thread can exit */
        if ((*_this).status != WAVEIN_READY) && ((*_this).status != WAVEIN_RECORDING) {
            return 0;
        }

        /* Entering main polling loop */
        while GetMessage(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0) != 0
        {
            /* switch (msg.message) */
            if msg.message == MM_WIM_DATA {
                phdr = msg.lParam as *mut WAVEHDR;

                if ((*_this).status == WAVEIN_RECORDING) ||
                    ((*_this).status == WAVEIN_FLUSHING)
                {
                    if ((*phdr).dwFlags & WHDR_DONE) != 0 {
                        /* Flushes recorded audio data to the `audio_receiver' object */
                        (*(*_this).audio_rcvd).audio_receive(
                            (*phdr).lpData as *mut u8,
                            (*phdr).dwBytesRecorded,
                        );

                        /* Updating `audio_receiver' total bytes received
                           _AFTER_ calling `audio_receive' function */
                        (*(*_this).audio_rcvd).bytes_received += (*phdr).dwBytesRecorded;
                    }

                    /* If status is not flushing data, then we can re-add the buffer
                       for reusing it. Otherwise, if we are flushing pending data,
                       we cannot re-add buffer because we don't need it anymore */
                    if (*_this).status != WAVEIN_FLUSHING {
                        /* Let the audio driver reuse the buffer */
                        waveInAddBuffer((*_this).wavein_handle, phdr, (core::mem::size_of::<WAVEHDR>()) as UINT);
                    } else {
                        /* If we are flushing pending data, we have to prepare
                           to stop recording. Set WAVEHDR flag to 0, and fires
                           the event `data_flushed_event', that will wake up
                           the main thread that is sleeping into wavein_in::stop_recording()
                           member function, waiting the last `MM_WIM_DATA' message
                           that contain pending data */

                        (*phdr).dwFlags = 0;
                        SetEvent((*_this).data_flushed_event);

                        /* The recording is going to stop, so the recording thread can go to sleep! */
                        WaitForSingleObject((*_this).wakeup_recthread, INFINITE);
                    }
                } /* if WAVEIN_RECORDING || WAVEIN_FLUSHING */
            } else if msg.message == MM_WIM_CLOSE {
                /* The thread can exit now */
                return 0;
            }
            /* end switch(msg.message) */
        } /* end while(GetMessage(...)) */

        return 0;
    }
    pub unsafe fn init_(&mut self) {
        let this_: *mut Self = self;
        ZeroMemory!(core::ptr::addr_of_mut!((*this_).wave_format) as LPVOID, core::mem::size_of::<WAVEFORMATEX>());
        (*this_).wave_format.cbSize = core::mem::size_of::<WAVEFORMATEX>() as _;
        (*this_).wavein_handle = core::ptr::null_mut();
        (*this_).recthread_id = 0;
        (*this_).wakeup_recthread = core::ptr::null_mut();
        (*this_).data_flushed_event = core::ptr::null_mut();
        (*this_).buf_secs = _AUDIO_DEFAULT_WAVEINBUFSECS as f32;
        (*this_).status = WAVEIN_NOTREADY;
    }
    pub unsafe fn alloc_buffers_mem_(&mut self, buffs: u32, secs: f32) {
        let this_: *mut Self = self;
        let mut onebuf_size: u32 = 0;
        let mut tot_size: u32 = 0;

        /* Release old memory */
        if !(*this_).main_buffer.is_null() {
            cpp_delete_array((*this_).main_buffer);
        }

        if !(*this_).wave_headers.is_null() {
            cpp_delete_array((*this_).wave_headers);
        }

        /* Calcs size of the buffers */
        onebuf_size = (((*this_).aud_info.byte_rate() as f32) * secs) as u32;
        tot_size = onebuf_size * buffs;

        /* Allocs memory for the audio buffers */
        (*this_).main_buffer = cpp_new_array::<BYTE>(tot_size);
        /* Allocs memory for the `WAVEHDR' structures */
        (*this_).wave_headers = cpp_new_array::<BYTE>(core::mem::size_of::<WAVEHDR>() as u32 * buffs) as *mut WAVEHDR;
        /* Zeros memory */
        ZeroMemory!((*this_).main_buffer as LPVOID, tot_size as usize);
        ZeroMemory!((*this_).wave_headers as LPVOID, core::mem::size_of::<WAVEHDR>() * buffs as usize);
        /* Updates total size of the buffers */
        (*this_).mb_size = tot_size;
    }
    pub unsafe fn free_buffers_mem_(&mut self) {
        let this_: *mut Self = self;
        /* Frees memory */
        if !(*this_).main_buffer.is_null() {
            cpp_delete_array((*this_).main_buffer);
        }

        if !(*this_).wave_headers.is_null() {
            cpp_delete_array((*this_).wave_headers);
        }

        (*this_).main_buffer = core::ptr::null_mut();
        (*this_).wave_headers = core::ptr::null_mut();
    }
    pub unsafe fn init_headers_(&mut self) {
        let this_: *mut Self = self;
        /* If there is no memory for memory or headers, simply return */
        if ((*this_).wave_headers.is_null()) || ((*this_).main_buffer.is_null()) {
            return;
        }

        /* This is the size for one buffer */
        let buf_sz: DWORD = (*this_).mb_size / (*this_).buffers;
        /* This is the base address for one buffer */
        let mut buf_addr: *mut BYTE = (*this_).main_buffer;
        /* Initializes headers */
        let mut i: u32 = 0;
        while i < (*this_).buffers {
            (*(*this_).wave_headers.add(i as usize)).dwBufferLength = (*this_).mb_size / (*this_).buffers;
            (*(*this_).wave_headers.add(i as usize)).lpData = buf_addr as LPSTR;
            buf_addr = buf_addr.add(buf_sz as usize);
            i += 1;
        }
    }
    pub unsafe fn prep_headers_(&mut self) {
        let this_: *mut Self = self;
        let mut err: u32;
        let mut error: bool = false;

        /* If there is no memory for memory or headers, throw error */
        if ((*this_).wave_headers.is_null()) || ((*this_).main_buffer.is_null()) || ((*this_).wavein_handle as usize == 0)
        {
            /* TODO: throw error! */
        }

        let mut i: u32 = 0;
        while i < (*this_).buffers {
            err = waveInPrepareHeader((*this_).wavein_handle, (*this_).wave_headers.offset(i as isize), (core::mem::size_of::<WAVEHDR>()) as UINT);
            if err != MMSYSERR_NOERROR {
                error = true;
            }
            i += 1;
        }

        if error {
            MessageBox(core::ptr::null_mut(), (&[0x0077, 0x0061, 0x0076, 0x0065, 0x0049, 0x006e, 0x0050, 0x0072, 0x0065, 0x0070, 0x0061, 0x0072, 0x0065, 0x0048, 0x0065, 0x0061, 0x0064, 0x0065, 0x0072, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x002e, 0x0000u16]).as_ptr() /* TEXT "waveInPrepareHeader Error." */, core::ptr::null(), 0);
        }
    }
    pub unsafe fn unprep_headers_(&mut self) {
        let this_: *mut Self = self;
        let mut err: u32;
        let mut error: bool = false;

        /* If there is no memory for memory or headers, throw error */
        if ((*this_).wave_headers.is_null()) || ((*this_).main_buffer.is_null()) || ((*this_).wavein_handle.is_null()) {
            /* TODO: throw error! */
        }

        static WAVEIN_UNPREP_ERR: [u16; 29] = [
            'w' as u16, 'a' as u16, 'v' as u16, 'e' as u16, 'I' as u16, 'n' as u16,
            'U' as u16, 'n' as u16, 'P' as u16, 'r' as u16, 'e' as u16, 'p' as u16,
            'a' as u16, 'r' as u16, 'e' as u16, 'H' as u16, 'e' as u16, 'a' as u16,
            'd' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'E' as u16, 'r' as u16,
            'r' as u16, 'o' as u16, 'r' as u16, '.' as u16, 0,
        ];

        let mut i: u32 = 0;
        while i < (*this_).buffers {
            err = waveInUnprepareHeader(
                (*this_).wavein_handle,
                (*this_).wave_headers.add(i as usize),
                core::mem::size_of::<WAVEHDR>() as u32,
            );
            if err != MMSYSERR_NOERROR {
                error = true;
            }
            i += 1;
        }

        if error {
            MessageBox(
                core::ptr::null_mut(),
                (WAVEIN_UNPREP_ERR.as_ptr()),
                core::ptr::null(),
                0,
            );
        }
    }
    pub unsafe fn add_buffers_to_driver_(&mut self) {
        let this_: *mut Self = self;
        let mut err: u32;
        let mut error: bool = false;

        /* If there is no memory for memory or headers, throw error */
        if ((*this_).wave_headers.is_null()) || ((*this_).main_buffer.is_null()) || ((*this_).wavein_handle.is_null()) {
            /* TODO: throw error! */
        }

        let mut i: u32 = 0;
        while i < (*this_).buffers {
            err = waveInAddBuffer(
                (*this_).wavein_handle,
                (*this_).wave_headers.add(i as usize),
                core::mem::size_of::<WAVEHDR>() as u32,
            );
            if err != MMSYSERR_NOERROR {
                error = true;
            }
            i += 1;
        }

        if error {
            let msg: [u16; 23] = [
                'w' as u16, 'a' as u16, 'v' as u16, 'e' as u16, 'I' as u16, 'n' as u16,
                'A' as u16, 'd' as u16, 'd' as u16, 'B' as u16, 'u' as u16, 'f' as u16,
                'f' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'E' as u16, 'r' as u16,
                'r' as u16, 'o' as u16, 'r' as u16, '.' as u16, 0,
            ];
            MessageBox(
                core::ptr::null_mut(),
                (msg.as_ptr()),
                core::ptr::null(),
                0,
            );
        }
    }
    pub unsafe fn open(&mut self) {
        let this_: *mut Self = self;
        let mut err: MMRESULT;
        let mut recthread_handle: HANDLE = core::ptr::null_mut();

        /* Checkin the status of the object */
        if (*this_).status != WAVEIN_NOTREADY {
            /* TODO: throw error */
        }

        /* Creating the EVENT object that will be signaled
           when the recording thread has to wake up */
        (*this_).wakeup_recthread = CreateEvent(core::ptr::null_mut(), (FALSE) as i32, (FALSE) as i32, core::ptr::null());

        (*this_).data_flushed_event = CreateEvent(core::ptr::null_mut(), (FALSE) as i32, (FALSE) as i32, core::ptr::null());

        if ((*this_).wakeup_recthread.is_null()) || ((*this_).data_flushed_event.is_null()) {
            (*this_).status = WAVEIN_ERR;
            MessageBox(core::ptr::null_mut(), (&[0x0054, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x002e, 0x0000u16]).as_ptr() /* TEXT "Thread Error." */, core::ptr::null(), 0);
            /* TODO: throw error */
        }

        /* Inialize buffers for recording audio data from the wavein audio line */
        (*this_).alloc_buffers_mem_((*this_).buffers, (*this_).buf_secs);
        (*this_).init_headers_();

        /* Sound format that will be captured by wavein */
        (*this_).wave_format.wFormatTag = (WAVE_FORMAT_PCM as u16);
        (*this_).wave_format.nChannels = (*this_).aud_info.channels();
        (*this_).wave_format.nSamplesPerSec = (*this_).aud_info.sample_rate();
        (*this_).wave_format.wBitsPerSample = (*this_).aud_info.bits();
        (*this_).wave_format.nBlockAlign = ((*this_).aud_info.block_align()) as u16;
        (*this_).wave_format.nAvgBytesPerSec = (*this_).aud_info.byte_rate();

        /* Creating the recording thread */
        recthread_handle = CreateThread(core::ptr::null_mut(),
                                        0,
                                        Some(audio_wavein_recording_procedure_wrapper),
                                        this_ as PVOID,
                                        0,
                                        core::ptr::addr_of_mut!((*this_).recthread_id));
        /* Checking thread handle */
        if recthread_handle.is_null() {
            /* Updating status */
            (*this_).status = WAVEIN_ERR;
            MessageBox(core::ptr::null_mut(), (&[0x0054, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x002e, 0x0000u16]).as_ptr() /* TEXT "Thread Error." */, core::ptr::null(), 0);
            /* TODO: throw error */
        }

        /* We don't need the thread handle anymore, so we can close it from now.
           (We'll just need the thread ID for the `waveInOpen' API) */
        CloseHandle(recthread_handle);

        /* Opening audio line wavein */
        err = waveInOpen(core::ptr::addr_of_mut!((*this_).wavein_handle),
                         0,
                         core::ptr::addr_of_mut!((*this_).wave_format),
                         ((*this_).recthread_id) as DWORD_PTR,
                         0,
                         CALLBACK_THREAD);

        if err != MMSYSERR_NOERROR {
            /* Updating status */
            (*this_).status = WAVEIN_ERR;

            if err == WAVERR_BADFORMAT {
                MessageBox(core::ptr::null_mut(), (&[0x0077, 0x0061, 0x0076, 0x0065, 0x0049, 0x006e, 0x004f, 0x0070, 0x0065, 0x006e, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x0000u16]).as_ptr() /* TEXT "waveInOpen Error" */, core::ptr::null(), 0);
            }

            /* TODO: throw error */
        }

        /* Update object status */
        (*this_).status = WAVEIN_READY;

        /* Now `audio_wavein' object is ready for audio recording! */
    }
    pub unsafe fn close(&mut self) {
        let this_: *mut Self = self;
        /* If wavein object is already in the status NOTREADY, nothing to do */
        if (*this_).status == WAVEIN_NOTREADY {
            return;
        }

        /* If the wavein is recording, then stop recording and close it */
        if (*this_).status == WAVEIN_RECORDING {
            (*this_).stop_recording();
        }

        /* Updating status */
        (*this_).status = WAVEIN_NOTREADY;

        /* Waking up recording thread, so it can receive
           the `MM_WIM_CLOSE' message then dies */
        if !(*this_).wakeup_recthread.is_null() {
            SetEvent((*this_).wakeup_recthread);
        }

        /* Closing wavein stream */
        while waveInClose((*this_).wavein_handle) != MMSYSERR_NOERROR {
            Sleep(1);
        }

        /* Release buffers memory */
        (*this_).free_buffers_mem_();

        /* Re-initialize variables to the initial state */
        (*this_).init_();
    }
    pub unsafe fn start_recording(&mut self) {
        let this_: *mut Self = self;
        let mut err: MMRESULT;
        let mut ev: BOOL;

        if ((*this_).status != WAVEIN_READY) && ((*this_).status != WAVEIN_STOP) {
            /* TODO: throw error */
        }

        /* Updating to the recording status */
        (*this_).status = WAVEIN_RECORDING;

        /* Let's prepare header of type WAVEHDR that we will pass to the driver
           with our audio informations, and buffer informations */
        (*this_).prep_headers_();

        /* The waveInAddBuffer function sends an input buffer to the given waveform-audio
           input device. When the buffer is filled, the application is notified. */
        (*this_).add_buffers_to_driver_();

        /* Signaling event for waking up the recorder thread */
        ev = SetEvent((*this_).wakeup_recthread);
        if ev == 0 {
            MessageBox(core::ptr::null_mut(), (&[0x0045, 0x0076, 0x0065, 0x006e, 0x0074, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x002e, 0x0000u16]).as_ptr() /* TEXT "Event Error." */, core::ptr::null(), 0);
        }

        /* Start recording */
        err = waveInStart((*this_).wavein_handle);
        if err != MMSYSERR_NOERROR {
            /* Updating status */
            (*this_).status = WAVEIN_ERR;
            MessageBox(core::ptr::null_mut(), (&[0x0077, 0x0061, 0x0076, 0x0065, 0x0049, 0x006e, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x002e, 0x0000u16]).as_ptr() /* TEXT "waveInStart Error." */, core::ptr::null(), 0);
            /* TODO: throw error */
        }
    }
    pub unsafe fn stop_recording(&mut self) {
        let this_: *mut Self = self;
        let mut err: MMRESULT;

        if (*this_).status != WAVEIN_RECORDING {
            return;
        }

        (*this_).status = WAVEIN_FLUSHING;

        /* waveInReset will make all pending buffer as done */
        err = waveInReset((*this_).wavein_handle);
        if err != MMSYSERR_NOERROR {
            /* TODO: throw error */
            MessageBox(
                core::ptr::null_mut(),
                (['w' as u16, 'a' as u16, 'v' as u16, 'e' as u16, 'I' as u16, 'n' as u16, 'R' as u16, 'e' as u16, 's' as u16, 'e' as u16, 't' as u16, ' ' as u16, 'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, '.' as u16, 0u16].as_ptr()),
                core::ptr::null(),
                0,
            );
        }

        if !(*this_).data_flushed_event.is_null() {
            WaitForSingleObject((*this_).data_flushed_event, INFINITE);
        }

        /* Stop recording */
        err = waveInStop((*this_).wavein_handle);
        if err != MMSYSERR_NOERROR {
            /* TODO: throw error */
            MessageBox(
                core::ptr::null_mut(),
                (['w' as u16, 'a' as u16, 'v' as u16, 'e' as u16, 'I' as u16, 'n' as u16, 'S' as u16, 't' as u16, 'o' as u16, 'p' as u16, ' ' as u16, 'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, '.' as u16, 0u16].as_ptr()),
                core::ptr::null(),
                0,
            );
        }

        /* The waveInUnprepareHeader function cleans up the preparation performed
           by the waveInPrepareHeader function */
        (*this_).unprep_headers_();

        (*this_).status = WAVEIN_STOP;
    }
}
