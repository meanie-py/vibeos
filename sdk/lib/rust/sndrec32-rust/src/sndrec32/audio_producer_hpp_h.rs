use crate::prelude::*;
use core::ffi::c_void;










impl audio_producer {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).bytes_played_ = 0;
        (*this_).play_finished = None;
        {
        }
    }
    pub unsafe fn bytes_played(&mut self) -> u32 {
        let this_: *mut Self = self;
        (*this_).bytes_played_
    }
    pub unsafe fn set_position(&mut self, pos: u32) {
        let this_: *mut Self = self;
        (*this_).bytes_played_ = pos;
    }
    pub unsafe fn set_position_start(&mut self) {
        let this_: *mut Self = self;
        (*this_).bytes_played_ = 0;
    }
    pub unsafe fn forward(&mut self, bytes: u32) {
        let this_: *mut Self = self;
        (*this_).bytes_played_ += bytes;
    }
    pub unsafe fn backward(&mut self, bytes: u32) {
        let this_: *mut Self = self;
        (*this_).bytes_played_ += bytes;
    }
}
