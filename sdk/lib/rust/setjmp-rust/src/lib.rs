//! The standalone staticlib over `setjmp-rt` (the code; see its Cargo.toml): `setjmp-rt`'s
//! `#[no_mangle]` exports plus this crate's panic runtime. A CRT DLL takes `setjmp-rt` into its
//! own Rust graph instead, which already has one.
#![no_std]

extern crate setjmp_rt;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

/// core's i386 `.eh_frame` names a personality routine; nothing here unwinds, so it is
/// never called.
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
