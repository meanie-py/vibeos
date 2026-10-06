//! wowx64.dll — the OS side of BTCpu64: x86-64 guest processes on arm64.
//!
//! Fork B of `docs/engineering/wow64-x64-guest-design.md`. ntdll's process
//! init sees an amd64 main image on an arm64 host and, instead of walking its
//! imports against arm64 DLLs and jumping into it natively, loads this DLL and
//! calls [`WowX64LdrpInitialize`], which never returns. From there:
//!
//!   1. load the CPU layer `libwowx64fex.dll` (FEXCore in 64-bit mode) and
//!      install a VEH that gives it first refusal on every native fault;
//!   2. give the guest a stack and an initial x86-64 context at the image entry;
//!   3. `BTCpu64Simulate` — the guest runs until it terminates the process.
//!
//! Guest system calls come back here through [`WowX64SystemService`], and are
//! forwarded by register shuffle alone: amd64 and arm64 share one service
//! table (`ntoskrnl/sysfuncs.lst` is arch-independent) and are both LLP64, so
//! the guest's service number IS the native one and every argument already has
//! the right width. That is the whole reason a 64-bit guest needs none of the
//! 13,678-line thunking surface the 32-bit wow64.dll carries.
//!
//! Milestone M1 (this file): a no-import guest (`hello64.exe`) enters FEX and
//! round-trips a syscall. NOT yet here, all owed to later milestones and
//! called out where they bite: the guest's own amd64 ntdll + loader, guest
//! exception dispatch (a guest fault is reported and the process terminated),
//! kernel->user callbacks and APCs into the guest, memory-change notifications
//! to the CPU layer (needed for SMC correctness and for images mapped after
//! startup), more than one thread, and TEB stack bounds that describe the guest
//! stack.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
// Off arm64 the crate is only its contract types, with nothing to use them.
#![cfg_attr(not(target_arch = "aarch64"), allow(dead_code))]

/// DbgPrint for the OS side. The TEB is SHARED with the guest, and so is its
/// `InDbgPrint` re-entrancy flag: a guest DbgPrint sets it and then executes
/// `int 2d`, which arrives here as an exception, and a plain DbgPrint would
/// then see the flag and silently print nothing — every line this layer says
/// from its VEH (the guest's own message included) was lost that way. So the
/// flag is the guest's to keep, and ours to step around.
#[cfg(target_arch = "aarch64")]
macro_rules! trace {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {{
        let Teb = ntdll_sys::nt::NtCurrentTeb();
        let Saved = (*Teb).InDbgPrint;
        (*Teb).InDbgPrint = 0;
        ntdll_sys::nt::DbgPrint(concat!("[WOWX64] ", $fmt, "\n\0").as_ptr() as ntdll_sys::nt::PCSTR $(, $arg)*);
        (*Teb).InDbgPrint = Saved;
    }};
}
#[cfg(target_arch = "aarch64")]
pub(crate) use trace;

mod contract;

#[cfg(target_arch = "aarch64")]
mod callback;
#[cfg(target_arch = "aarch64")]
mod except;
#[cfg(target_arch = "aarch64")]
mod host;
#[cfg(target_arch = "aarch64")]
mod image;
#[cfg(target_arch = "aarch64")]
mod syscall;
#[cfg(target_arch = "aarch64")]
mod thread;

// no_std staticlib needs its own panic handler + EH personality.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
