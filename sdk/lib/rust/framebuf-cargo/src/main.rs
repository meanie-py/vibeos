//! Empty bin shell. `framebuf.dll`'s real entry point is `DrvEnableDriver`,
//! provided by `libframebuf_rust.a` (linked by build.rs via reactos-link with
//! `-Wl,-e,DrvEnableDriver`). `ModuleKind::KernelDll` links `-shared` and does
//! NOT `--gc-sections`, so `DrvEnableDriver` is retained. This `main` is never
//! run.

#![no_std]
#![no_main]
#![allow(internal_features)]

// Loud panic handler (2026-07-06 sweep) — a bare `loop {}` here turns any
// Rust panic into a silent wedge (the arm64input avail-ring bug cost a live
// forensics session to even locate). framebuf is a win32k display driver: it
// imports ONLY libwin32k.a (no ntoskrnl), so DbgPrint is unavailable — use the
// DDI channel `EngDebugPrint(str str ptr)` instead. The messages are fully
// pre-formatted (no % specifiers) so the va_list arg is never consumed → null
// is safe. The serial shape is what smoke triage classifies as RUST_PANIC.
extern "system" {
    fn EngDebugPrint(
        StandardPrefix: *const core::ffi::c_char,
        DebugMessage: *const core::ffi::c_char,
        ap: *mut core::ffi::c_void,
    );
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
