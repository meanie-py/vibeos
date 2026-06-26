//! samsrv-rust — the samsrv.dll C-to-Rust body translation.
//!
//! Cohab with the still-C samsrv .c files (commented out of
//! dll/win32/samsrv/CMakeLists.txt as each flips). The `#[no_mangle]
//! extern "system"` server methods + helpers here are the sole providers of
//! those symbols once their C TU is commented out; the RPC dispatch tables
//! (sam-stub Rust + the generated sam_s glue) bind them by decorated name, and
//! the not-yet-flipped C bodies (Samr*/Samp*/Sam*) resolve at link the other
//! way. FFI substrate = ntdll-sys (nt base) + samsrv-sys (SAMPR_* marshalled) +
//! lsasrv-sys (shared SD/SID/policy types).
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused_unsafe, dead_code)]

// Wine debug macros — no-ops in the NDEBUG build (TRACE/WARN/ERR/FIXME compile
// out in the C too). Defined at crate root before the body modules so textual
// macro scoping makes them visible to every translated file.
macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
macro_rules! WARN { ($($arg:tt)*) => {{}}; }
macro_rules! ERR { ($($arg:tt)*) => {{}}; }
macro_rules! FIXME { ($($arg:tt)*) => {{}}; }
// Lowercase aliases — some translated files emit `trace!`/`warn!`/etc.
macro_rules! trace { ($($arg:tt)*) => {{}}; }
macro_rules! warn { ($($arg:tt)*) => {{}}; }
macro_rules! err { ($($arg:tt)*) => {{}}; }
macro_rules! fixme { ($($arg:tt)*) => {{}}; }

// Compile-time UTF-16 NUL-terminated &'static [u16; N] literal macros (crate is
// #![no_std], no alloc). For ASCII string literals (registry key/attr names).
macro_rules! wide {
    ($s:literal) => {{
        const B: &[u8] = $s.as_bytes();
        const N: usize = B.len() + 1;
        const W: [u16; N] = { let mut a = [0u16; N]; let mut i = 0; while i < B.len() { a[i] = B[i] as u16; i += 1; } a };
        &W
    }};
}
macro_rules! wstr { ($s:literal) => { wide!($s) }; }

mod consts;
mod prelude;
mod samsrv;

// USER-MODE no_std staticlib → own panic handler + eh_personality (same role as
// sam-stub; deduped at link by --allow-multiple-definition).
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
