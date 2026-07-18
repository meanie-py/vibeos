//! ws2help.dll — Rust translation of dll/win32/ws2help (the WinSock 2 Helper DLL).
//! staticlib → libws2help_rust.a, cohab-linked into the cmake-built ws2help.dll with the
//! translated .c commented out (per-file flip, like ws2_32-rust/wshtcpip-rust/mswsock-rust).
//!
//! ws2help provides the Wah* handle-context table / APC-helper / notification-handle helpers
//! ws2_32/msafd use to map SOCKET handles to per-provider context (WahInsert/Enumerate/
//! CloseHandleContext, WahQueueUserApc, WahNotifyAllProcesses, ...). The WAH handle-context +
//! WS2IFSL EA struct layouts are bindgen-generated per arch into `types` (profile: ws2help).
//!
//! Per-file cohab flip: each translated .c becomes a module here and its #[no_mangle]
//! symbols take over from the commented-out C.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // WAH handle-context / WS2IFSL EA layouts (bindgen, per arch) + re-exported nt types
pub mod prelude; // the ONE import surface: types glob + enum aliases + skipped consts + bridges

// The 5 translated translation units. `ws2help` mirrors the C layout (dll/win32/ws2help/).
#[allow(clippy::module_inception)]
pub mod ws2help {
    pub mod apc; // WahOpenApcHelper / WahQueueUserApc — the APC-helper pseudo-device glue
    pub mod context; // the WAH handle-context hash table (create/destroy/insert/enumerate)
    pub mod dllmain; // Ws2HelpDllMain + Ws2helpInitialize (global heap / SD / spinlock setup)
    pub mod handle; // WahCreate/CloseSocketHandle + the WS2IFSL non-IFS handle support
    pub mod notify; // WahCreateNotificationHandle / WahNotifyAllProcesses
}

// staticlib (libws2help_rust.a) needs a panic handler + eh personality (no_std cohab),
// gated off for `cargo test` where std provides them. Mirrors ws2_32-rust/wshtcpip-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
