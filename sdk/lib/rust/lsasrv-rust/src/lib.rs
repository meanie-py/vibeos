//! lsasrv-rust — the lsasrv.dll C-to-Rust body translation.
//!
//! Cohab with the still-C lsasrv .c files (commented out of
//! dll/win32/lsasrv/CMakeLists.txt as each flips). The `#[no_mangle]
//! extern "system"` server methods + helpers here are the sole providers of
//! those symbols once their C TU is commented out; the RPC dispatch tables
//! (lsa-stub Rust + the generated dssetup_s.c) bind them by decorated name, and
//! the not-yet-flipped C bodies (Lsar*/Lsap*/Lsa*) resolve at link the other
//! way. FFI substrate = lsasrv-sys.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused_unsafe)]

mod dssetup;

// USER-MODE no_std staticlib → own panic handler + eh_personality (same role as
// lsa-stub; deduped at link by --allow-multiple-definition).
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
