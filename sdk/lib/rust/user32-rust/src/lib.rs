#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(static_mut_refs)]
#![allow(clippy::all)]

//! user32.dll — Rust translation of win32ss/user/user32 (the usermode windowing API DLL).
//! staticlib → libuser32_rust.a, cohab-linked into the cmake user32.dll with the
//! translated .c commented out. The usermode *client* half of the windowing system; the
//! kernel half (win32k.sys) is already 100% Rust. Most exports marshal args and call into
//! win32k via NtUser* syscalls (the `ntuser` FFI module). See
//! docs/engineering/user32-translation-audit.md for the per-wave plan.

// Foundation: minimal Win32 type aliases + the NtUser* syscall import surface.
pub mod types;
pub mod ntuser;

// ntwrapper.h port: the 143 NtUserx*/NtUser* inline wrappers (over win32k-sys),
// which the translated windows/ bodies call. Plain #[inline] fns, not exported.
pub mod ntwrapper;

// Shared user32-internal substrate: ntdll Rtl*/Zw* externs, TEB/PEB inlines, the
// ValidateHwnd/UserApiHook cohab externs, and Win32 word macros. Used by the big
// windows/ bodies (class/window/message/defwnd). Typed against win32k-sys.
pub mod internal;
pub mod winbase;

// misc/ — dllmain, timer, object, exit, ... (Wave 0 leaf: timer.rs).
pub mod misc;

// windows/ — the windowing core. Wave-1 integrated leaves: caret, dc.
pub mod windows;

// controls/ — Wave-2 standard control bodies (button/static/edit/combo/listbox/scrollbar).
pub mod controls;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
