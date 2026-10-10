use crate::prelude::*;
use core::ffi::c_void;













impl audio_waveout {
    pub unsafe fn playing_procedure(&mut self) -> u32 {
        let this_: *mut Self = self;
        let mut msg: MSG = core::mem::zeroed();
        let mut phdr: *mut WAVEHDR;
        let mut err: u32;
        let _this: *mut audio_waveout = this_;
        let mut read_size: u32;

        /* Check the arg pointer */
        if _this == core::ptr::null_mut() {
            return 0;
        }

        /* The thread can go to sleep for now. It will be wake up only when
           there is audio data to be recorded */
        if !(*_this).wakeup_playthread.is_null() {
            WaitForSingleObject((*_this).wakeup_playthread, INFINITE);
        }

        /* Entering main polling loop */
        while GetMessage(
            core::ptr::addr_of_mut!(msg),
            core::ptr::null_mut(),
            0,
            0,
        ) != 0
        {
            if msg.message == MM_WOM_DONE {
                phdr = msg.lParam as *mut WAVEHDR;

                /* If the status of the `wave_out' object is different
                   than playing, then the thread can go to sleep */
                if ((*_this).status != WAVEOUT_PLAYING)
                    && ((*_this).status != WAVEOUT_FLUSHING)
                    && !(*_this).wakeup_playthread.is_null()
                {
                    WaitForSingleObject((*_this).wakeup_playthread, INFINITE);
                }

                /* The playing thread doesn't have to sleep, so let's checking
                   first if the little buffer has been sent to the audio driver
                   (it has the WHDR_DONE flag). If it is, we can read new audio
                   datas from the `audio_producer' object, refill the little buffer,
                   and resend it to the driver with waveOutWrite( ) API */
                if (*phdr).dwFlags & WHDR_DONE != 0 {
                    if (*_this).status == WAVEOUT_PLAYING {
                        /* Here the thread is still playing a sound, so it can
                           read new audio data from the `audio_producer' object */
                        read_size = (*(*_this).audio_buf).read(
                            (*phdr).lpData as *mut BYTE,
                            (*phdr).dwBufferLength,
                        );
                    } else {
                        read_size = 0;
                    }

                    /* If the `audio_producer' object, has produced some
                       audio data, so `read_size' will be > 0 */
                    if read_size != 0 {
                        /* Adjusts the correct effectively read size */
                        (*phdr).dwBufferLength = read_size;

                        /* Before sending the little buffer to the driver,
                           we have to remove the `WHDR_DONE' flag, because
                           the little buffer now contain new audio data that have to be played */
                        (*phdr).dwFlags &= !WHDR_DONE;

                        /* Plays the sound of the little buffer */
                        err = waveOutWrite(
                            (*_this).waveout_handle,
                            phdr,
                            core::mem::size_of::<WAVEHDR>() as u32,
                        );

                        /* Checking if any error has occured */
                        if err != MMSYSERR_NOERROR {
                            MessageBox(
                                core::ptr::null_mut(),
                                ([
                                    'w' as u16, 'a' as u16, 'v' as u16, 'e' as u16,
                                    'O' as u16, 'u' as u16, 't' as u16, 'W' as u16,
                                    'r' as u16, 'i' as u16, 't' as u16, 'e' as u16,
                                    ' ' as u16, 'E' as u16, 'r' as u16, 'r' as u16,
                                    'o' as u16, 'r' as u16, 0u16,
                                ]
                                .as_ptr()),
                                core::ptr::null(),
                                0,
                            );
                            /* TODO: throw error */
                        }
                    } else {
                        /* Here `read_size' is 0, so the `audio_producer' object,
                           doesn't have any sound data to produce anymore. So,
                           now we have to see the little buffer #ID to establishing what to do */
                        if (*phdr).dwUser == 0 {
                            /* Here `read_size' is 0, and the buffer user data
                               contain 0, so this is the first of N little
                               buffers that came back with `WHDR_DONE' flag;
                               this means that this is the last little buffer
                               in which we have to read data to; so we can
                               _STOP_ reading data from the `audio_producer'
                               object: doing this is accomplished just setting
                               the current status as "WAVEOUT_FLUSHING" */
                            (*_this).status = WAVEOUT_FLUSHING;
                        } else if (*phdr).dwUser == (((*_this).buffers - 1)) as _ {
                            /* Here `read_size' and the buffer user data, that
                               contain a buffer ID#, is equal to the number of
                               the total buffers - 1. This means that this is
                               the _LAST_ little buffer that has been played by
                               the audio driver. We can STOP the `wave_out'
                               object now, or restart the sound playing, if we have a infinite loop */
                            (*_this).stop();

                            /* Let the thread go to sleep */
                            if let Some(f) = (*(*_this).audio_buf).play_finished {
                                f();
                            }

                            if !(*_this).wakeup_playthread.is_null() {
                                WaitForSingleObject(
                                    (*_this).wakeup_playthread,
                                    INFINITE,
                                );
                            }
                        } /* if (phdr->dwUser == (_this->buffers - 1)) */
                    } /* if read_size != 0 */
                } /* (phdr->dwFlags & WHDR_DONE) */
            } else if msg.message == MM_WOM_CLOSE {
                /* The thread can exit now */
                return 0;
            } else if msg.message == MM_WOM_OPEN {
                /* Do nothing */
            }
        } /* end while(GetMessage(...)) */

        return 0;
    }
    pub unsafe fn init_(&mut self) {
        let this_: *mut Self = self;
        ZeroMemory!(
            core::ptr::addr_of_mut!((*this_).wave_format) as LPVOID,
            core::mem::size_of::<WAVEFORMATEX>(),
        );
        (*this_).wave_format.cbSize = core::mem::size_of::<WAVEFORMATEX>() as _;
        (*this_).waveout_handle = core::ptr::null_mut();
        (*this_).playthread_id = 0;
        (*this_).wakeup_playthread = core::ptr::null_mut();
        (*this_).buf_secs = _AUDIO_DEFAULT_WAVEOUTBUFSECS as f32;
        (*this_).status = WAVEOUT_NOTREADY;
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
        let fmt = &mut *((*this_).aud_info as *mut audio_format);
        onebuf_size = (fmt.byte_rate() as f32 * secs) as u32;
        tot_size = onebuf_size * buffs;
        /* Allocs memory for the audio buffers */
        (*this_).main_buffer = cpp_new_array::<BYTE>(tot_size as usize);
        /* Allocs memory for the `WAVEHDR' structures */
        (*this_).wave_headers = cpp_new_array::<BYTE>(core::mem::size_of::<WAVEHDR>() * buffs as usize) as *mut WAVEHDR;
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
        if (*this_).wave_headers.is_null() || (*this_).main_buffer.is_null() {
            return;
        }

        /* This is the size for one buffer */
        let buf_sz: DWORD = (*this_).mb_size / (*this_).buffers;
        /* This is the base address for one buffer */
        let mut buf_addr: *mut BYTE = (*this_).main_buffer;

        ZeroMemory!(
            (*this_).wave_headers as *mut core::ffi::c_void,
            core::mem::size_of::<WAVEHDR>() * ((*this_).buffers as usize),
        );

        /* Initializes headers */
        let mut i: u32 = 0;
        while i < (*this_).buffers {
            /* Sets the correct base address and length for the little buffer */
            (*(*this_).wave_headers.add(i as usize)).dwBufferLength = (*this_).mb_size / (*this_).buffers;
            (*(*this_).wave_headers.add(i as usize)).lpData = buf_addr as LPSTR;

            /* Unsets the WHDR_DONE flag */
            (*(*this_).wave_headers.add(i as usize)).dwFlags &= !WHDR_DONE;

            /* Sets the WAVEHDR user data with an unique little buffer ID# */
            (*(*this_).wave_headers.add(i as usize)).dwUser = ((i as u32) as usize) as _;

            /* Increments little buffer base address */
            buf_addr = buf_addr.add(buf_sz as usize);

            i += 1;
        }
    }
    pub unsafe fn prep_headers_(&mut self) {
        let this_: *mut Self = self;
        let mut err: MMRESULT;
        let mut error: bool = false;

        /* If there is no memory for memory or headers, throw error */
        if ((*this_).wave_headers.is_null()) || ((*this_).main_buffer.is_null()) || ((*this_).waveout_handle as usize == 0) {
            /* TODO: throw error! */
        }

        let mut i: u32 = 0;
        while i < (*this_).buffers {
            err = waveOutPrepareHeader((*this_).waveout_handle, (*this_).wave_headers.add(i as usize), core::mem::size_of::<WAVEHDR>() as u32);
            if err != MMSYSERR_NOERROR {
                error = true;
            }
            i += 1;
        }

        if error {
            /* TODO: throw error indicating which header i-th is errorneous */
        }
    }
    pub unsafe fn unprep_headers_(&mut self) {
        let this_: *mut Self = self;
        let mut err: MMRESULT;
        let mut error: bool = false;

        /* If there is no memory for memory or headers, throw error */
        if ((*this_).wave_headers.is_null()) || ((*this_).main_buffer.is_null()) || ((*this_).waveout_handle.is_null())
        {
            /* TODO: throw error! */
        }

        let mut i: u32 = 0;
        while i < (*this_).buffers
        {
            err = waveOutUnprepareHeader((*this_).waveout_handle, (*this_).wave_headers.add(i as usize), core::mem::size_of::<WAVEHDR>() as u32);
            if err != MMSYSERR_NOERROR
            {
                error = true;
            }
            i += 1;
        }

        if error
        {
            /* TODO: throw error indicating which header i-th is errorneous */
        }
    }
    pub unsafe fn open(&mut self) {
        let this_: *mut Self = self;
        let fmt = &mut *((*this_).aud_info as *mut audio_format);
        let mut err: MMRESULT;
        let mut playthread_handle: HANDLE = core::ptr::null_mut();

        /* Checkin the status of the object */
        if (*this_).status != WAVEOUT_NOTREADY
        {
            /* TODO: throw error */
        }

        /* Creating the EVENT object that will be signaled when
           the playing thread has to wake up */
        (*this_).wakeup_playthread = CreateEvent(core::ptr::null_mut(), (FALSE) as i32, (FALSE) as i32, core::ptr::null());
        if (*this_).wakeup_playthread.is_null()
        {
            (*this_).status = WAVEOUT_ERR;
            /* TODO: throw error */
        }

        /* Inialize buffers for recording audio data from the wavein audio line */
        (*this_).alloc_buffers_mem_((*this_).buffers, (*this_).buf_secs);
        (*this_).init_headers_();

        /* Sound format that will be captured by wavein */
        (*this_).wave_format.wFormatTag = (WAVE_FORMAT_PCM as u16);
        (*this_).wave_format.nChannels = fmt.channels();
        (*this_).wave_format.nSamplesPerSec = fmt.sample_rate();
        (*this_).wave_format.wBitsPerSample = fmt.bits();
        (*this_).wave_format.nBlockAlign = (fmt.block_align()) as u16;
        (*this_).wave_format.nAvgBytesPerSec = fmt.byte_rate();

        /* Creating the recording thread */
        playthread_handle = CreateThread(core::ptr::null_mut(),
                                         0,
                                         Some(audio_waveout_playing_procedure_wrapper),
                                         this_ as PVOID,
                                         0,
                                         core::ptr::addr_of_mut!((*this_).playthread_id));
        /* Checking thread handle */
        if playthread_handle.is_null()
        {
            /* Updating status */
            (*this_).status = WAVEOUT_ERR;
            /* TODO: throw error */
        }

        /* We don't need the thread handle anymore, so we can close it from now.
           (We'll just need the thread ID for the `waveInOpen' API) */
        CloseHandle(playthread_handle);

        /* Reset the `audio_source' to the start position */
        (*(*this_).audio_buf).set_position_start();
        /* Opens the WAVE_OUT device */
        err = waveOutOpen(core::ptr::addr_of_mut!((*this_).waveout_handle),
                          WAVE_MAPPER,
                          core::ptr::addr_of_mut!((*this_).wave_format),
                          ((*this_).playthread_id) as DWORD_PTR,
                          0,
                          CALLBACK_THREAD | WAVE_ALLOWSYNC);
        if err != MMSYSERR_NOERROR
        {
            MessageBox(core::ptr::null_mut(), (&[0x0077, 0x0061, 0x0076, 0x0065, 0x004f, 0x0075, 0x0074, 0x004f, 0x0070, 0x0065, 0x006e, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x0000u16]).as_ptr() /* _T "waveOutOpen Error" */, core::ptr::null(), 0);
            /* TODO: throw error */
        }

        (*this_).status = WAVEOUT_READY;
    }
    pub unsafe fn play(&mut self) {
        let this_: *mut Self = self;
        let mut err: u32;
        let mut i: u32;

        if (*this_).main_buffer.is_null() {
            /* TODO; throw error, or assert */
            return;
        }

        /* If the status is PAUSED, we have to resume the audio playing */
        if (*this_).status == WAVEOUT_PAUSED {
            /* Updates status */
            (*this_).status = WAVEOUT_PLAYING;
            /* Tells to the driver to resume audio playing */
            waveOutRestart((*this_).waveout_handle);
            /* Wakeup playing thread */
            SetEvent((*this_).wakeup_playthread);
            return;
        } /* if status == WAVEOUT_PAUSED */

        if (*this_).status != WAVEOUT_READY {
            return;
        }

        /* Prepares WAVEHDR structures */
        (*this_).prep_headers_();
        /* Sets correct status */
        (*this_).status = WAVEOUT_PLAYING;
        /* Reads the audio from the start */
        //audio_buf.set_position_start();

        /* Reads the first N bytes from the audio buffer, where N = the total
           size of all little buffers */
        (*(*this_).audio_buf).read((*this_).main_buffer, (*this_).mb_size);

        /* Wakeup the playing thread */
        SetEvent((*this_).wakeup_playthread);

        /* Sends all the little buffers to the audio driver, so it can play
           the sound data */
        i = 0 as u32;
        while i < (*this_).buffers {
            err = waveOutWrite(
                (*this_).waveout_handle,
                (*this_).wave_headers.add(i as usize),
                core::mem::size_of::<WAVEHDR>() as u32,
            );
            if err != MMSYSERR_NOERROR as u32 {
                MessageBox(
                    core::ptr::null_mut(),
                    (&[0x0077, 0x0061, 0x0076, 0x0065, 0x004f, 0x0075, 0x0074, 0x0057, 0x0072, 0x0069, 0x0074, 0x0065, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x0000u16]).as_ptr() /* _T "waveOutWrite Error" */,
                    core::ptr::null(),
                    0,
                );
                /* TODO: throw error */
            }
            i += 1;
        }
    }
    pub unsafe fn pause(&mut self) {
        let this_: *mut Self = self;
        let mut err: MMRESULT;

        /* If the waveout object is not playing audio, do nothing */
        if (*this_).status == WAVEOUT_PLAYING {
            /* Updating status */
            (*this_).status = WAVEOUT_PAUSED;
            /* Tells to audio driver to pause audio */
            err = waveOutPause((*this_).waveout_handle);
            if err != MMSYSERR_NOERROR {
                static MSG_TEXT: [u16; 19] = ['w' as u16, 'a' as u16, 'v' as u16, 'e' as u16, 'O' as u16, 'u' as u16, 't' as u16, 'P' as u16, 'a' as u16, 'u' as u16, 's' as u16, 'e' as u16, ' ' as u16, 'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, 0];
                MessageBox(core::ptr::null_mut(), (MSG_TEXT.as_ptr()), core::ptr::null(), 0);
                /* TODO: throw error */
            }
        }
    }
    pub unsafe fn stop(&mut self) {
        let this_: *mut Self = self;
        let mut err: MMRESULT;

        /* Checks the current status */
        if ((*this_).status != WAVEOUT_PLAYING) &&
            ((*this_).status != WAVEOUT_FLUSHING) &&
            ((*this_).status != WAVEOUT_PAUSED)
        {
            /* Do nothing */
            return;
        }

        /* Sets a new status */
        (*this_).status = WAVEOUT_STOP;
        /* Flushes pending audio datas */
        err = waveOutReset((*this_).waveout_handle);
        if err != MMSYSERR_NOERROR
        {
            MessageBox(core::ptr::null_mut(), ([
                'e' as u16, 'r' as u16, 'r' as u16, ' ' as u16, 'W' as u16, 'a' as u16,
                'v' as u16, 'e' as u16, 'O' as u16, 'u' as u16, 't' as u16, 'R' as u16,
                'e' as u16, 's' as u16, 'e' as u16, 't' as u16, '.' as u16, '\n' as u16,
                0u16
            ].as_ptr()), ([
                'E' as u16, 'R' as u16, 'R' as u16, 'O' as u16, 'R' as u16, 0u16
            ].as_ptr()), 0);
            /* TODO: throw error */
        }

        /* Sets the start position of the audio buffer */
        (*(*this_).audio_buf).set_position_start();
        /* Cleans little buffers */
        (*this_).unprep_headers_();
        (*this_).init_headers_();
        /* Refreshes the status */
        (*this_).status = WAVEOUT_READY;
    }
    pub unsafe fn close(&mut self) {
        let this_: *mut Self = self;
        let mut err: MMRESULT;

        /* If the `wave_out' object is playing audio, or it is in paused state,
           we have to call the `stop' member function, to flush pending buffers */
        if ((*this_).status == WAVEOUT_PLAYING) || ((*this_).status == WAVEOUT_PAUSED) {
            (*this_).stop();
        }

        /* When we have flushed all pending buffers, the wave out handle can be successfully closed */
        err = waveOutClose((*this_).waveout_handle);
        if err != MMSYSERR_NOERROR {
            MessageBox(core::ptr::null_mut(), (&[0x0077, 0x0061, 0x0076, 0x0065, 0x004f, 0x0075, 0x0074, 0x0043, 0x006c, 0x006f, 0x0073, 0x0065, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x0000u16]).as_ptr() /* _T "waveOutClose Error" */, core::ptr::null(), 0);
            /* TODO: throw error */
        }

        (*this_).free_buffers_mem_();
    }
}
