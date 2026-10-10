use crate::prelude::*;
use core::ffi::c_void;














impl audio_format {
    pub unsafe fn __ctor_0(&mut self, samples_per_second: u32, bits_per_sample: u16, channels: u16) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).samples_psec = samples_per_second;
        (*this_).bits_psample = bits_per_sample;
        (*this_).chan = channels;
        {
        }
    }
    pub unsafe fn op_eq(&mut self, eq: &mut audio_format) -> bool {
        let this_: *mut Self = self;
        return ((*this_).samples_psec == eq.samples_psec) &&
               ((*this_).bits_psample == eq.bits_psample) &&
               ((*this_).chan == eq.chan);
    }
    pub unsafe fn sample_rate(&mut self) -> u32 {
        let this_: *mut Self = self;
        (*this_).samples_psec
    }
    pub unsafe fn bits(&mut self) -> u16 {
        let this_: *mut Self = self;
        (*this_).bits_psample
    }
    pub unsafe fn channels(&mut self) -> u16 {
        let this_: *mut Self = self;
        (*this_).chan
    }
    pub unsafe fn byte_rate(&mut self) -> u32 {
        let this_: *mut Self = self;
        return (*this_).samples_psec * ((*this_).chan as u32) * (((*this_).bits_psample as u32) / 8);
    }
    pub unsafe fn block_align(&mut self) -> u32 {
        let this_: *mut Self = self;
        return ((*this_).chan as u32 * ((*this_).bits_psample as u32 / 8));
    }
    pub unsafe fn samples_in_seconds(&mut self, seconds: f32) -> u32 {
        let this_: *mut Self = self;
        return (((*this_).samples_psec as f32) * ((*this_).chan as f32) * seconds) as u32;
    }
    pub unsafe fn samples_in_bytes(&mut self, bytes: u32) -> u32 {
        let this_: *mut Self = self;
                return (bytes / (((*this_).bits_psample as u32 / 8 as u32) * (*this_).chan as u32));
            }
    pub unsafe fn bytes_in_samples(&mut self, samples: u32) -> u32 {
        let this_: *mut Self = self;
        return samples.wrapping_mul(((*this_).bits_psample as u32 / 8).wrapping_mul((*this_).chan as u32));
    }
}
