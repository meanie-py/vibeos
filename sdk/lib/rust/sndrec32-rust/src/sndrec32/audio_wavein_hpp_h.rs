use crate::prelude::*;
use core::ffi::c_void;



















impl audio_wavein {
    pub unsafe fn __ctor_0(&mut self, a_info: &audio_format, a_receiver: &mut audio_receiver) {
        let this_: *mut Self = self;
        (*this_).wave_headers = core::ptr::null_mut();
        (*this_).aud_info = core::ptr::read(a_info);
        (*this_).audio_rcvd = a_receiver as *mut audio_receiver;
        (*this_).status = WAVEIN_NOTREADY;
        (*this_).main_buffer = core::ptr::null_mut();
        (*this_).mb_size = 0;
        (*this_).buffers = (_AUDIO_DEFAULT_WAVEINBUFFERS as u32);
        {
            /* Initializing internal wavein data */
            (*this_).init_();
            (*this_).aud_info = core::ptr::read(a_info);
        }
    }
    pub unsafe fn current_status(&mut self) -> audio_wavein_status {
        let this_: *mut Self = self;
        (*this_).status
    }
    pub unsafe fn buffer_secs__0(&mut self) -> f32 {
        let this_: *mut Self = self;
        (*this_).buf_secs
    }
    pub unsafe fn buffer_secs__1(&mut self, bsecs: f32) {
        let this_: *mut Self = self;
        /* Some checking */
        if bsecs <= 0.0 {
            return;
        }

        /* Set seconds length for each buffer */
        (*this_).buf_secs = bsecs;
    }
    pub unsafe fn total_buffers__0(&mut self) -> u32 {
        let this_: *mut Self = self;
        (*this_).buffers
    }
    pub unsafe fn total_buffers__1(&mut self, tot_bufs: u32) {
        let this_: *mut Self = self;
        if tot_bufs == 0 {
            return;
        }
        (*this_).buffers = tot_bufs;
    }
    pub unsafe fn format(&mut self) -> audio_format {
        let this_: *mut Self = self;
        (*this_).aud_info
    }
    pub unsafe fn buf(&mut self) -> *mut BYTE {
        let this_: *mut Self = self;
        (*this_).main_buffer
    }
    pub unsafe fn bufsz(&mut self) -> u32 {
        let this_: *mut Self = self;
        (*this_).mb_size
    }
    pub unsafe fn samplevalue_max(&mut self) -> u32 {
        let this_: *mut Self = self;
        if (*this_).aud_info.bits() == 16 {
            return 65535 as u32;
        } else if (*this_).aud_info.bits() == 8 {
            return 255 as u32;
        } else {
            return 0;
        }
    }
    pub unsafe fn tot_samples_buf(&mut self) -> u32 {
        let this_: *mut Self = self;
        return (*this_).aud_info.samples_in_bytes((*this_).mb_size);
    }
    pub unsafe fn nsample(&mut self, nsamp: u32) -> u32 {
        let this_: *mut Self = self;
        let mut svalue: u32;

        if (*this_).aud_info.bits() == 16 {
            svalue = abs(
                *(((*this_).main_buffer.offset((*this_).aud_info.bytes_in_samples(nsamp) as isize)) as *const i16) as i32,
            ) as u32;
        } else if (*this_).aud_info.bits() == 8 {
            svalue = (*(*this_).main_buffer.offset((*this_).aud_info.bytes_in_samples(nsamp) as isize)) as isize as u32;
        } else {
            svalue = 0;
        }

        return svalue;
    }
}
