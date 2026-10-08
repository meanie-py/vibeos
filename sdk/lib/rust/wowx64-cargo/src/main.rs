//! Empty bin shell. `wowx64.dll`'s exports + DllMain come from `libwowx64_rust.a`
//! (linked by build.rs via reactos-link, ModuleKind::Win32Dll, entry DllMain).
//! This `main` is never run.

#![no_std]
#![no_main]

// Loud panic handler — a bare `loop {}` here turns any Rust panic into a
// silent wedge. The real one lives in the staticlib; this stub is tolerated
// via --allow-multiple-definition.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
