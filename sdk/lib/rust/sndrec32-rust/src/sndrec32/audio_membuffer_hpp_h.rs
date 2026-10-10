use crate::prelude::*;
use core::ffi::c_void;




















impl audio_membuffer {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__init_vtables();
        (*this_).audio_data = core::ptr::null_mut();
        (*this_).aud_info = _AUDIO_DEFAULT_FORMAT;
        (*this_).buf_size = 0;
        (*this_).init_size = 0;
        {
            /* Allocs memory for at least 1 or some seconds of recording */
            (*this_).init_size = (((*this_).aud_info.byte_rate() as f32) * (_AUDIO_DEFAULT_BUFSECS as f32)) as u32;
            (*this_).alloc_mem_((*this_).init_size);
        }
    }
    pub unsafe fn __ctor_1(&mut self, aud_fmt: audio_format) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__init_vtables();
        (*this_).audio_data = core::ptr::null_mut();
        (*this_).aud_info = aud_fmt;
        (*this_).buf_size = 0;
        (*this_).init_size = 0;
        {
            /* Allocs memory for at least 1 or some seconds of recording */
            (*this_).init_size = ((*this_).aud_info.byte_rate() as f32 * (_AUDIO_DEFAULT_BUFSECS as f32)) as u32;
            (*this_).alloc_mem_((*this_).init_size);
        }
    }
    pub unsafe fn __ctor_2(&mut self, aud_fmt: audio_format, seconds: u32) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__init_vtables();
        (*this_).audio_data = 0 as *mut BYTE;
        (*this_).aud_info = aud_fmt;
        (*this_).buf_size = 0;
        (*this_).init_size = 0;
        {
            /* Allocs memory for audio recording the specified number of seconds */
            (*this_).init_size = (*this_).aud_info.byte_rate() * seconds;
            (*this_).alloc_mem_((*this_).init_size);
        }
    }
    pub unsafe fn __ctor_3(&mut self, aud_fmt: audio_format, seconds: f32) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__init_vtables();
        (*this_).audio_data = core::ptr::null_mut();
        (*this_).aud_info = aud_fmt;
        (*this_).buf_size = 0;
        (*this_).init_size = 0;
        {
            /* Allocs memory for audio recording the specified number of seconds */
            let t: f32 = if ((*this_).aud_info.byte_rate() as f32) * seconds <= 0 as f32 {
                1 as f32
            } else {
                seconds
            };
            (*this_).init_size = t as u32;
            (*this_).alloc_mem_((*this_).init_size);
        }
    }
    pub unsafe fn __ctor_4(&mut self, bytes: u32) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__init_vtables();
        (*this_).audio_data = core::ptr::null_mut();
        (*this_).aud_info = _AUDIO_DEFAULT_FORMAT;
        (*this_).buf_size = 0;
        (*this_).init_size = 0;
        {
            /* Allocs memory for the specified bytes */
            (*this_).init_size = bytes;
            (*this_).alloc_mem_((*this_).init_size);
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        /* Frees memory and reset values */
        (*this_).clear();
    }
    pub unsafe fn mem_size(&mut self) -> u32 {
        let this_: *mut Self = self;
        (*this_).buf_size
    }
    pub unsafe fn bytes_recorded(&mut self) -> u32 {
        let this_: *mut Self = self;
        return (*this_).__base0.bytes_received;
    }
    pub unsafe fn seconds_total(&mut self) -> u32 {
        let this_: *mut Self = self;
        return (*this_).buf_size / (*this_).aud_info.byte_rate();
    }
    pub unsafe fn seconds_recorded(&mut self) -> u32 {
        let this_: *mut Self = self;
        return (*this_).__base0.bytes_received / (*this_).aud_info.byte_rate();
    }
    pub unsafe fn fseconds_total(&mut self) -> f32 {
        let this_: *mut Self = self;
        return ((*this_).buf_size as f32) / ((*this_).aud_info.byte_rate() as f32);
    }
    pub unsafe fn fseconds_recorded(&mut self) -> f32 {
        let this_: *mut Self = self;
        return ((*this_).__base0.bytes_received as f32) / ((*this_).aud_info.byte_rate() as f32);
    }
    pub unsafe fn total_samples(&mut self) -> u32 {
        let this_: *mut Self = self;
        return (*this_).aud_info.samples_in_seconds((*this_).fseconds_total());
    }
    pub unsafe fn samples_received(&mut self) -> u32 {
        let this_: *mut Self = self;
        return (*this_).aud_info.samples_in_bytes((*this_).__base0.bytes_received);
    }
    pub unsafe fn audio_buffer(&mut self) -> *mut BYTE {
        let this_: *mut Self = self;
        (*this_).audio_data
    }
    pub unsafe fn audinfo(&mut self) -> &'static mut audio_format {
        let this_: *mut Self = self;
        return &mut (*this_).aud_info;
    }
    pub unsafe fn truncate(&mut self) {
        let this_: *mut Self = self;
        (*this_).truncate_();
    }
}
