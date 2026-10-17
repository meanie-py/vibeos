use crate::prelude::*;
use ntkrnl_sys::ddk::PMDL;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn TdiMapBuffer(MdlChain: PMDL) {}

#[no_mangle]
pub unsafe extern "system" fn TdiUnmapBuffer(MdlChain: PMDL) {}
