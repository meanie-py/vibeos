//! wpprecorderum-rust — wpprecorderum.dll, the user-mode WPP in-flight recorder.
//!
//! Not in the 2003 tree; the user-mode half of wpprecorder.sys (wpprecorder-rust), for
//! WPP-instrumented UMDF drivers and services. They call WppAutoLogStart/Stop around
//! their life and WppAutoLogTrace per message. As in the kernel half, there is no trace
//! session and no triage reader, so every message is recorded to nowhere -- what a
//! caller sees on Windows when nothing reads its log -- and the default log handle is
//! non-null, because callers test it before using it.

#![no_std]
#![allow(non_snake_case)]

use core::ffi::c_void;

static LOG: u8 = 0;

#[no_mangle]
pub unsafe extern "system" fn DllMain(_instance: *mut c_void, _reason: u32, _reserved: *mut c_void) -> i32 {
    1
}

#[no_mangle]
pub unsafe extern "system" fn WppAutoLogGetDefaultHandle() -> *mut c_void {
    &LOG as *const u8 as *mut c_void
}

#[no_mangle]
pub unsafe extern "system" fn WppAutoLogStart(_wpp_cb: *mut c_void, _name: *const u16) {}

#[no_mangle]
pub unsafe extern "system" fn WppAutoLogStop(_wpp_cb: *mut c_void) {}

/// Variadic in the WDK (the message's arguments follow MessageNumber), hence cdecl.
#[no_mangle]
pub unsafe extern "C" fn WppAutoLogTrace(
    _auto_log_context: *mut c_void,
    _message_level: u32,
    _message_flags: u32,
    _message_guid: *mut c_void,
    _message_number: u32,
) -> i32 {
    0
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
