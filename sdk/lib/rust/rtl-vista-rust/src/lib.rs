//! rtl-vista-rust — staticlib link surface for the standalone ntdll_vista.dll
//! forwarder.
//!
//! Re-exports the rtl-sync `vista` modules (srw / condvar / runonce / locale)
//! so their `#[no_mangle] extern "system"` exports land in librtl_vista_rust.a,
//! replacing the old condvar/runonce/srw/locale.c that compiled into
//! librtl_vista.a. condvar's RtlEnter/LeaveCriticalSection are *not* defined
//! here — they resolve as ntdll.dll imports (critical.rs is not in the `vista`
//! feature), exactly as the old condvar.c relied on.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// `pub use` makes each `#[no_mangle]` export part of this crate's public API →
// reachable → emitted (with its undecorated/stdcall symbol) into the staticlib.
pub use rtl_sync::condvar::*;
pub use rtl_sync::locale::*;
pub use rtl_sync::runonce::*;
pub use rtl_sync::srw::*;
pub use rtl_sync::waitaddr::*;

// The NT threadpool (TpAllocWork, TpPostWork, ...). Upstream compiles threadpool.c into
// rtl_vista as well as rtl, so ntdll_vista carries its own pool instance, separate from the
// one behind ntdll's RtlQueueWorkItem; this mirrors that. Only the Tp* names are exported
// (ntdll_vista.spec); the crate's other #[no_mangle] bodies are just unexported copies.
pub use rtl_threadpool::threadpool::*;

// Standalone staticlib (no_std) needs its own panic handler; the final
// ntdll_vista.dll link already tolerates the sibling utf8/ntdll-rust handlers.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
