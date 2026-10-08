//! wowx64.dll — the OS side of BTCpu64: x86-64 guest processes on arm64.
//!
//! Fork B of `docs/engineering/wow64-x64-guest-design.md`. ntdll's process
//! init sees an amd64 main image on an arm64 host and, instead of walking its
//! imports against arm64 DLLs and jumping into it natively, loads this DLL and
//! calls [`WowX64LdrpInitialize`], which never returns. From there:
//!
//!   1. map the GUEST's own amd64 ntdll out of `<SystemRoot>\sysamd64`
//!      (`image.rs`), load the CPU layer `libwowx64fex.dll` (FEXCore in 64-bit
//!      mode), and install a VEH that gives it first refusal on every fault;
//!   2. enter the guest the way an amd64 thread starts: in its ntdll's
//!      `KiUserApcDispatcher`, on an APC frame naming `LdrInitializeThunk`. The
//!      guest's own loader then does its process init -- kernel32, the image's
//!      import closure, DLL init routines -- and NtContinues to the image entry.
//!
//! The guest and the native ntdll share one PEB and one TEB (that is what makes
//! kernel-written fields like `Win32ClientInfo` just work). What each side must
//! keep to itself is called out where it bites: the loader's module list and
//! lock (ntdll-rust `LdrpLdrData`), the TEB stack bounds (`thread.rs`), the
//! `InDbgPrint` flag (`trace!` below), and the heap block layout (rtl-heap's
//! one 64-bit `HEAP_ENTRY`).
//!
//! Guest system calls come back here (`syscall.rs`) and are forwarded by
//! register shuffle: amd64 and arm64 share one service table
//! (`ntoskrnl/sysfuncs.lst` and win32k's `w32ksvc64.h` are arch-independent)
//! and are both LLP64, so the guest's service number IS the native one and
//! every argument already has the right width. That is the whole reason a
//! 64-bit guest needs none of the 13,678-line thunking surface the 32-bit
//! wow64.dll carries. The exceptions -- CONTEXT carriers, the system32 ->
//! sysamd64 redirect, thread creation, callback return, and the address-space
//! changes the CPU layer must hear about -- are listed in `syscall.rs`. Guest
//! exceptions are delivered to the guest's own `KiUserExceptionDispatcher`
//! (`except.rs`), win32k callbacks run in the guest (`callback.rs`), and
//! guest-created threads become guest threads (`thread.rs`), as do threads a
//! native process creates here; user-mode APCs run in the guest (`apc.rs`).
//! What is still owed is listed in docs/engineering/wow64-x64-guest-design.md.

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
mod apc;
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
