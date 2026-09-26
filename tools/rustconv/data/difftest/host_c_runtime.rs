use core::ffi::{c_char, c_void};

#[link(name = "c")]
unsafe extern "C" {
    pub fn malloc(size: usize) -> *mut c_void;
    pub fn strlen(value: *const c_char) -> usize;
    pub fn strcpy(destination: *mut c_char, source: *const c_char) -> *mut c_char;
}
