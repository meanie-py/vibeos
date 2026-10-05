//! `__pfnDliNotifyHook2`, the delay-load notify hook (sdk/lib/delayimp/pfnDliNotifyHook2.c), in a
//! crate of its own so that it is its own member of libdelayimp_rust.a. A module may define
//! the hook itself (MSVC's documented contract: dbghelp_apitest, win32u_apitest); its object
//! precedes the archive, so the name is already resolved and this member is never pulled.
//! Had the slot shared an object with `__delayLoadHelper2`, pulling the helper would pull a
//! second definition with it.
#![no_std]
#![allow(non_upper_case_globals)]

/// The `PfnDliHook` slot; null (`None`) until a module installs a hook. delayimp-rust
/// reads it by name as its typed `PfnDliHook`, the same pointer-sized `Option<fn>`.
#[no_mangle]
pub static mut __pfnDliNotifyHook2: Option<unsafe extern "system" fn()> = None;
