use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut RosSymCallbacks: ROSSYM_CALLBACKS = unsafe { core::mem::zeroed() };
#[no_mangle]
pub unsafe extern "C" fn RosSymInit(Callbacks: *mut ROSSYM_CALLBACKS) {
    RosSymCallbacks = core::ptr::read(Callbacks);
}
