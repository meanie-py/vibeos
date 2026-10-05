//! `atan2` for gdi32: `libm::atan2`, the body msvcrt.dll's own `atan2` export uses (cnt-rust
//! `cabi/math.rs`). It replaces the `atan2` C library, which was a different implementation
//! on each arch: AMD's libm_sse2 `atan2.c` (plus its `_handle_error` / `_invoke_matherr`
//! error path) on amd64, x87 `atan2_asm.s` on i386, `math/arm64/atan2.c` on arm64. gdi32
//! only ever asks it for arc angles of finite coordinates, where all of them agree to the
//! last bit or one; no caller in gdi32 installs a `_matherr` that the amd64 error path could
//! reach.
#![no_std]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

/// core's i386 `.eh_frame` names a personality routine; nothing here unwinds, so it is
/// never called (the pseh-rust / delayimp-rust pattern).
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[no_mangle]
pub extern "C" fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}
