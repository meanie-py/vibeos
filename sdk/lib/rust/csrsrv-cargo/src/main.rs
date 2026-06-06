//! Empty bin shell — csrsrv.dll's real entry is `_DllMain@12` (the native DLL
//! entry, provided by the Rust csrsrv module in libsmss_csrss_rust.a). Cargo
//! requires a `main.rs` to drive the link, but nothing here is executed. See
//! ntdll-cargo for the Win32Dll + spec2def export pattern.

#![no_std]
#![no_main]
#![allow(internal_features)]

// rustc requires a `#[panic_handler]` in main.rs's crate graph (it can't see
// the staticlib at compile time). libsmss_csrss_rust.a defines its own;
// `--allow-multiple-definition` lets lld take the first.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
