//! wpprecorder-rust — wpprecorder.sys, the WPP in-flight recorder ("IFR").
//!
//! Not in the 2003 tree; written from the WDK's wpprecorder.h documentation
//! (learn.microsoft.com/windows-hardware/drivers/ddi/wpprecorder/). WPP-instrumented
//! Windows 10 drivers call into it from their DriverEntry and on every trace statement:
//! WppAutoLogStart/Stop around the driver's life, WppAutoLogTrace per message, and
//! imp_WppRecorderReplay when a session enables tracing. Windows keeps those messages in a
//! ring buffer for crash-dump triage; this kernel has no triage reader and no trace
//! session, so every message is recorded to nowhere -- the behaviour a driver sees on
//! Windows when nothing reads its log. What a driver can observe is kept consistent:
//! creating a log succeeds and returns a handle, the default log exists, and a handle
//! handed out is accepted back.

#![no_std]
#![allow(non_snake_case)]

use ntkrnl_sys::base::*;

/// The one log handle: never dereferenced, but distinct and non-null, because drivers
/// test RECORDER_LOG values for NULL before using them.
static LOG: u8 = 0;

fn log_handle() -> PVOID {
    &LOG as *const u8 as PVOID
}

/// Export drivers are never started, but the PE needs an entry point.
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(_driver: PVOID, _registry_path: PVOID) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn DllInitialize(_registry_path: PVOID) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn DllUnload() -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WppAutoLogStart(_wpp_cb: PVOID, _driver: PVOID, _registry_path: PVOID) {}

#[no_mangle]
pub unsafe extern "system" fn WppAutoLogStop(_wpp_cb: PVOID, _driver: PVOID) {}

/// Variadic in the WDK (the message's arguments follow MessageNumber), hence cdecl; the
/// fixed part is all a recorder that keeps nothing needs to accept.
#[no_mangle]
pub unsafe extern "C" fn WppAutoLogTrace(
    _auto_log_context: PVOID,
    _message_level: ULONG,
    _message_flags: ULONG,
    _message_guid: PVOID,
    _message_number: ULONG,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn imp_WppRecorderReplay(
    _wpp_cb: PVOID,
    _trace_handle: u64,
    _enable_flags: ULONG,
    _enable_level: ULONG,
) {
}

#[no_mangle]
pub unsafe extern "system" fn imp_WppRecorderConfigure(_wpp_cb: PVOID, _params: PVOID) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn imp_WppRecorderLogCreate(
    _wpp_cb: PVOID,
    _create_params: PVOID,
    recorder_log: *mut PVOID,
) -> NTSTATUS {
    if !recorder_log.is_null() {
        *recorder_log = log_handle();
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn imp_WppRecorderLogDelete(_wpp_cb: PVOID, _recorder_log: PVOID) {}

#[no_mangle]
pub unsafe extern "system" fn imp_WppRecorderLogGetDefault(_wpp_cb: PVOID) -> PVOID {
    log_handle()
}

#[no_mangle]
pub unsafe extern "system" fn imp_WppRecorderIsDefaultLogAvailable(_wpp_cb: PVOID) -> BOOLEAN {
    1
}

#[no_mangle]
pub unsafe extern "system" fn imp_WppRecorderLogSetIdentifier(
    _wpp_cb: PVOID,
    _recorder_log: PVOID,
    _log_identifier: PVOID,
) -> NTSTATUS {
    STATUS_SUCCESS
}

/// WPP_TRIAGE_INFO is filled for Windows Error Reporting's crash triage; there is none
/// here, and the caller's structure is left as it passed it.
#[no_mangle]
pub unsafe extern "system" fn imp_WppRecorderGetTriageInfo(_wpp_cb: PVOID, _triage_info: PVOID) {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
