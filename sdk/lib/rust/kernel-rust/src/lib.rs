//! Umbrella staticlib that combines all Rust kernel components into a single
//! archive for linking into ntoskrnl. Each component is an rlib dependency;
//! this crate provides the single panic_handler and pulls in all symbols.

#![no_std]

// Re-export pool so its #[no_mangle] symbols are linked in.
extern crate reactos_pool;
// Re-export ntoskrnl-rust (ob subsystem) so its #[no_mangle] symbols are linked in.
extern crate ntoskrnl_rust;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
