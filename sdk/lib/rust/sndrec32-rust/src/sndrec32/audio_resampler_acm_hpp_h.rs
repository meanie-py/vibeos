use crate::prelude::*;
use core::ffi::c_void;


impl audio_resampler_acm {
    pub unsafe fn __ctor_0(&mut self, fmt_in: audio_format, fmt_out: audio_format) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).acm_stream = core::ptr::null_mut();
        (*this_).src_buflen = 0;
        (*this_).dst_buflen = 0;
        (*this_).stream_opened = false;
        (*this_).audfmt_in = fmt_in;
        (*this_).audfmt_out = fmt_out;
        (*this_).buf_secs = _AUDIO_DEFAULT_BUFSECS as f32;
        {
            (*this_).init_();
        }
    }
}
