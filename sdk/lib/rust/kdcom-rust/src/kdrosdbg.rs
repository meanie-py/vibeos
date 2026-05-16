//! Rust translation of `drivers/base/kdrosdbg/kdrosdbg.c`.
//!
//! COPYRIGHT:  See COPYING in the top level directory
//! PROJECT:    ReactOS kernel
//! PURPOSE:    Serial i/o functions for the kernel debugger.
//! PROGRAMMER: Alex Ionescu / Hervé Poussineau
//!
//! Faithful 1:1 port of the 116-LOC stub. Under the KDBG build the
//! in-kernel debugger owns the transport, so this DLL only satisfies
//! the `kdrosdbg.spec` ABI: every entry returns a constant; the two
//! packet routines are `@unimplemented` and reproduce the C
//! `UNIMPLEMENTED` macro (a `DbgPrint` warning) verbatim.
//!
//! `NTAPI` is stdcall on i386 → `extern "system"` yields the spec's
//! `_Name@N` decoration (e.g. `_KdSendPacket@16`). Pointer/`long`/
//! `BOOLEAN` args are declared at their 4-byte stdcall stack-slot width
//! so the `@N` byte count matches `kdrosdbg.spec` exactly.

use core::ffi::{c_char, c_void};

/// `LONG` status code. `STATUS_SUCCESS` = 0,
/// `STATUS_NOT_IMPLEMENTED` = 0xC0000002.
type NTSTATUS = i32;
/// KD transport status (`KDSTATUS` is a `ULONG`).
type KDSTATUS = u32;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC000_0002u32 as NTSTATUS;

extern "C" {
    /// `ULONG __cdecl DbgPrint(PCSTR Format, ...)` — imported from
    /// ntoskrnl.exe (the `kdrosdbg` cmake target does
    /// `add_importlibs(kdcom ntoskrnl hal)`; only `DbgPrint` is
    /// actually referenced).
    fn DbgPrint(format: *const c_char, ...) -> u32;
}

/// ReactOS `UNIMPLEMENTED` macro: warn via `DbgPrint`. Kept as the
/// faithful no-op-equivalent it is under the KDBG smoke baseline (the
/// KD packet path is never routed through `kdcom.dll` there).
#[inline]
unsafe fn unimplemented(msg: &'static [u8]) {
    // `msg` is a NUL-terminated byte string; pass it as the lone
    // (non-variadic) DbgPrint argument.
    DbgPrint(msg.as_ptr() as *const c_char);
}

/// @unimplemented
#[no_mangle]
pub unsafe extern "system" fn KdDebuggerInitialize0(_loader_block: *mut c_void) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

/// @unimplemented
#[no_mangle]
pub unsafe extern "system" fn KdDebuggerInitialize1(_loader_block: *mut c_void) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn KdD0Transition() -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdD3Transition() -> NTSTATUS {
    STATUS_SUCCESS
}

/// @implemented — nothing to do on COM ports.
/// `IN BOOLEAN SleepTransition` (`kdrosdbg.spec`: `long` → `@4`).
#[no_mangle]
pub unsafe extern "system" fn KdSave(_sleep_transition: u32) -> NTSTATUS {
    STATUS_SUCCESS
}

/// @implemented — nothing to do on COM ports.
/// `IN BOOLEAN SleepTransition` (`kdrosdbg.spec`: `long` → `@4`).
#[no_mangle]
pub unsafe extern "system" fn KdRestore(_sleep_transition: u32) -> NTSTATUS {
    STATUS_SUCCESS
}

/// @unimplemented
/// `KdSendPacket(IN ULONG, IN PSTRING, IN PSTRING, IN OUT PKD_CONTEXT)`
/// — 4 args → `_KdSendPacket@16`.
#[no_mangle]
pub unsafe extern "system" fn KdSendPacket(
    _packet_type: u32,
    _message_header: *mut c_void,
    _message_data: *mut c_void,
    _context: *mut c_void,
) {
    unimplemented(b"KdSendPacket at drivers/base/kdrosdbg/kdrosdbg.c is UNIMPLEMENTED!\n\0");
}

/// @unimplemented
/// `KdReceivePacket(IN ULONG, OUT PSTRING, OUT PSTRING, OUT PULONG,
/// IN OUT PKD_CONTEXT)` — 5 args → `_KdReceivePacket@20`.
#[no_mangle]
pub unsafe extern "system" fn KdReceivePacket(
    _packet_type: u32,
    _message_header: *mut c_void,
    _message_data: *mut c_void,
    _data_length: *mut u32,
    _context: *mut c_void,
) -> KDSTATUS {
    unimplemented(b"KdReceivePacket at drivers/base/kdrosdbg/kdrosdbg.c is UNIMPLEMENTED!\n\0");
    0
}
