use crate::prelude::*;
use core::ffi::c_void;











impl audio_waveout {
    pub unsafe fn __ctor_0(&mut self, aud_fmt: &audio_format, a_buf: &mut audio_producer) {
        let this_: *mut Self = self;
        (*this_).wave_headers = core::ptr::null_mut();
        (*this_).aud_info = aud_fmt as *const audio_format;
        (*this_).audio_buf = a_buf as *mut audio_producer;
        (*this_).status = WAVEOUT_NOTREADY;
        (*this_).main_buffer = core::ptr::null_mut();
        (*this_).mb_size = 0;
        (*this_).buffers = (_AUDIO_DEFAULT_WAVEOUTBUFFERS as u32);
        {
            /* Initializing internal wavein data */
            (*this_).init_();
        }
    }
    pub unsafe fn current_status(&mut self) -> audio_waveout_status {
        let this_: *mut Self = self;
        (*this_).status
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
        let fmt = &mut *((*this_).aud_info as *mut audio_format);
        if fmt.bits() == 16 {
            return 65535 as u32;
        } else if fmt.bits() == 8 {
            return 255 as u32;
        } else {
            return 0 as u32;
        }
    }
    pub unsafe fn tot_samples_buf(&mut self) -> u32 {
        let this_: *mut Self = self;
        let fmt = &mut *((*this_).aud_info as *mut audio_format);
        return fmt.samples_in_bytes((*this_).mb_size);
    }
    pub unsafe fn nsample(&mut self, nsamp: u32) -> u32 {
        let this_: *mut Self = self;
        let fmt = &mut *((*this_).aud_info as *mut audio_format);
        let mut svalue: u32;

        if fmt.bits() == 16 {
            svalue = (abs(
                (*((*this_).main_buffer.add(fmt.bytes_in_samples(nsamp) as usize) as *const i16)) as i32,
            )) as u32;
        } else if fmt.bits() == 8 {
            svalue = ((*((*this_).main_buffer.add(fmt.bytes_in_samples(nsamp) as usize))) as isize) as u32;
        } else {
            svalue = 0;
        }

        return svalue;
    }
}
