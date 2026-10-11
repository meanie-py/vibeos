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
// Lowercase aliases — some translated files emit `trace!`/`warn!`/etc.
macro_rules! trace { ($($arg:tt)*) => {{}}; }
macro_rules! warn { ($($arg:tt)*) => {{}}; }
macro_rules! err { ($($arg:tt)*) => {{}}; }
macro_rules! fixme { ($($arg:tt)*) => {{}}; }

// Compile-time UTF-16 NUL-terminated &'static [u16; N] literal macros (crate is
// #![no_std], no alloc). For ASCII string literals (registry key/attr names).
// `#[macro_export]` so the `crate::wstr!` / `crate::wchar_literal!` path form the
// c2rust output uses also resolves (plus the plain textual-scope form). All the
// spellings the per-file translations emit (w!/widestring!/wchar_lit!/
// wchar_literal!/wstr!) alias the one `wide!` implementation.
#[macro_export]
macro_rules! wide {
    ($s:literal) => {{
        const B: &[u8] = $s.as_bytes();
        const N: usize = B.len() + 1;
        const W: [u16; N] = { let mut a = [0u16; N]; let mut i = 0; while i < B.len() { a[i] = B[i] as u16; i += 1; } a };
        &W
    }};
}
// `w!` mirrors `wide!` (array ref; call sites add `.as_ptr()`).
#[macro_export]
macro_rules! w { ($s:literal) => { $crate::wide!($s) }; }
// The c2rust `L"..."`-decay spellings are passed *directly* where a `*const u16`
// is expected (the C array-to-pointer decay), so these resolve to a pointer.
#[macro_export]
macro_rules! wstr { ($s:literal) => { $crate::wide!($s).as_ptr() as *mut u16 }; }
#[macro_export]
macro_rules! widestring { ($s:literal) => { $crate::wide!($s).as_ptr() as *mut u16 }; }
#[macro_export]
macro_rules! wchar_lit { ($s:literal) => { $crate::wide!($s).as_ptr() as *mut u16 }; }
#[macro_export]
macro_rules! wchar_literal { ($s:literal) => { $crate::wide!($s).as_ptr() as *mut u16 }; }
// samrpc.c spellings (all the L"..."-decay forms used as a `*const/*mut u16`).
#[macro_export]
macro_rules! wsz { ($s:literal) => { $crate::wide!($s).as_ptr() as *mut u16 }; }
#[macro_export]
macro_rules! wchar { ($s:literal) => { $crate::wide!($s).as_ptr() as *mut u16 }; }
#[macro_export]
macro_rules! widestr { ($s:literal) => { $crate::wide!($s).as_ptr() as *mut u16 }; }
#[macro_export]
macro_rules! wchar_str { ($s:literal) => { $crate::wide!($s).as_ptr() as *mut u16 }; }
#[macro_export]
macro_rules! L { ($s:literal) => { $crate::wide!($s).as_ptr() as *mut u16 }; }
#[macro_export]
macro_rules! windows_wide { ($s:literal) => { $crate::wide!($s).as_ptr() as *mut u16 }; }
#[macro_export]
macro_rules! bwstr { ($s:literal) => { $crate::wide!($s).as_ptr() as *mut u16 }; }

// FIELD_OFFSET(type, field) — winnt.h macro → core::mem::offset_of! (returns usize).
#[macro_export]
macro_rules! FIELD_OFFSET { ($t:ty, $f:ident) => { core::mem::offset_of!($t, $f) }; }

// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate.
#[macro_use]
extern crate winedebug_rust;

mod consts;
mod prelude;
mod samsrv;

// USER-MODE no_std staticlib → own panic handler + eh_personality (same role as
// sam-stub; deduped at link by --allow-multiple-definition).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
