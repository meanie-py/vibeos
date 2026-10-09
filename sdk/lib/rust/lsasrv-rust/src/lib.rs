//! lsasrv-rust — the lsasrv.dll C-to-Rust body translation.
//!
//! Cohab with the still-C lsasrv .c files (commented out of
//! dll/win32/lsasrv/CMakeLists.txt as each flips). The `#[no_mangle]
//! extern "system"` server methods + helpers here are the sole providers of
//! those symbols once their C TU is commented out; the RPC dispatch tables
//! (lsa-stub Rust + the generated dssetup_s.c) bind them by decorated name, and
//! the not-yet-flipped C bodies (Lsar*/Lsap*/Lsa*) resolve at link the other
//! way. FFI substrate = ntdll-sys (nt base) + lsasrv-sys (LSAPR_* marshalled).
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused_unsafe, dead_code)]

// Wine debug macros — no-ops in the NDEBUG build (TRACE/WARN/ERR/FIXME compile
// out in the C too). Defined at crate root before the body modules so textual
// macro scoping makes them visible to every translated file.
// Lowercase aliases — some translated files (lookup.rs) emit `trace!`/`warn!`/etc.
macro_rules! trace { ($($arg:tt)*) => {{}}; }
macro_rules! warn { ($($arg:tt)*) => {{}}; }
macro_rules! err { ($($arg:tt)*) => {{}}; }
macro_rules! fixme { ($($arg:tt)*) => {{}}; }

// Compile-time UTF-16 NUL-terminated literal: `wide!("Accounts")` → &'static [u16; N]
// (no alloc; the #![no_std] crate has no Vec). Replaces the c2rust
// `"X\0".encode_utf16().collect::<Vec<u16>>().as_ptr()` idiom (which both needed
// alloc AND was a use-after-free of the dropped Vec). ASCII literals only.
#[macro_export]
macro_rules! wide {
    ($s:literal) => {{
        const B: &[u8] = $s.as_bytes();
        const N: usize = B.len() + 1;
        const W: [u16; N] = {
            let mut a = [0u16; N];
            let mut i = 0;
            while i < B.len() {
                a[i] = B[i] as u16;
                i += 1;
            }
            a
        };
        &W
    }};
}
#[macro_export]
macro_rules! wstr { ($s:literal) => { $crate::wide!($s) }; }

// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate.
#[macro_use]
extern crate winedebug_rust;

mod consts;
mod prelude;
mod lsasrv;

// USER-MODE no_std staticlib → own panic handler + eh_personality (same role as
// lsa-stub; deduped at link by --allow-multiple-definition).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
