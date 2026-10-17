use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn CTEAllocateString(Unknown0: ULONG, Unknown1: ULONG) -> BOOLEAN {
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn CTEInitString(Unknown0: ULONG, Unknown1: ULONG) -> BOOLEAN {
    FALSE
}
