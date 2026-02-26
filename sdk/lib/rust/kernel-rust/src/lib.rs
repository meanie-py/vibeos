//! Umbrella staticlib that combines all Rust kernel components into a single
//! archive for linking into ntoskrnl. Each component is an rlib dependency;
//! this crate provides the single panic_handler and pulls in all symbols.

#![no_std]

// Re-export pool so its #[no_mangle] symbols are linked in.
extern crate reactos_pool;
// Re-export ntoskrnl-rust (ob subsystem) so its #[no_mangle] symbols are linked in.
extern crate ntoskrnl_rust;

// Force inclusion of KDBG extension symbols (called from C kdb_cli.c, not from Rust).
// Without this, rustc dead-code elimination strips them from the staticlib.
mod _force_kdbg_symbols {
    extern "C" {
        fn ExpKdbgExtPool(argc: u32, argv: *mut *mut i8) -> u8;
        fn ExpKdbgExtPoolUsed(argc: u32, argv: *mut *mut i8) -> u8;
        fn ExpKdbgExtPoolFind(argc: u32, argv: *mut *mut i8) -> u8;
        fn ExpKdbgExtIrpFind(argc: u32, argv: *mut *mut i8) -> u8;
    }
    #[used]
    static KDBG_REFS: [unsafe extern "C" fn(u32, *mut *mut i8) -> u8; 4] = [
        ExpKdbgExtPool,
        ExpKdbgExtPoolUsed,
        ExpKdbgExtPoolFind,
        ExpKdbgExtIrpFind,
    ];
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
