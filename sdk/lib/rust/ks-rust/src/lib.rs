//! ks-rust — Rust translation of the Kernel Streaming class library (drivers/ksfilter/ks).
//! EXPORT driver; cohab ks.sys. NOT boot-loaded -> link-clean x3 is the bar.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]

// ASSERT is debug_assert! (ntkrnl_sys::dbgassert): checked in a debug build, never
// evaluated in release -- the kernel's convention. It was an empty fn here, which
// never checked yet still evaluated its argument on every call.
macro_rules! ASSERT { ($($t:tt)*) => { ntkrnl_sys::nt_assert!($($t)*) }; }

// Pull in the slist crate so its #[no_mangle] RtlInterlockedPush/PopEntrySList land in
// libks_rust.a (ntkrnl-sys references them; ntoskrnl doesn't export them under that name).
extern crate slist;

// Coerce a fn item into a bindgen NTAPI callback slot. Uses `$f as *const ()` (thin
// data-pointer cast) rather than `$f as usize`: the integer cast is illegal at const-eval
// time, so this form works in `static`/const vtable initializers too. ABI-agnostic (the
// slot's per-arch ABI — stdcall on i386, C on amd64/arm64 — need not be named).
macro_rules! cb { ($f:expr) => { core::mem::transmute($f as *const ()) }; }
#[allow(unused_imports)] pub(crate) use cb;

// C `L!("...")` wide-string literal -> NUL-terminated UTF-16 array (all ks L-strings are ASCII;
// each byte maps 1:1 to a u16). Const-promoted to 'static so `.as_ptr()` stays valid. The
// DeepSeek translation kept C wide-string syntax; `L!("x")` is rewritten to `L!("x")` at the
// call sites. Trailing NUL is always appended (matches C semantics; a redundant explicit
// `\0` in the source just yields a harmless second terminator).
macro_rules! L {
    ($s:literal) => {{
        const __S: &str = $s;
        const __N: usize = __S.len() + 1;
        const __W: [u16; __N] = {
            let b = __S.as_bytes();
            let mut a = [0u16; __N];
            let mut i = 0;
            while i < __S.len() { a[i] = b[i] as u16; i += 1; }
            a
        };
        &__W
    }};
}
#[allow(unused_imports)] pub(crate) use L;

// C `CONTAINING_RECORD(ptr, Type, field)` -> Rust macro. The translation kept C call
// syntax (no `!`); call sites are rewritten to `CONTAINING_RECORD!(...)`. Accepts a
// dotted field path (e.g. `Tail.Overlay.ListEntry`) via the tt-rest matcher.
macro_rules! CONTAINING_RECORD {
    ($ptr:expr, $type:ty, $($field:tt)+) => {
        ($ptr as *const u8).sub(core::mem::offset_of!($type, $($field)+)) as *mut $type
    };
}
#[allow(unused_imports)] pub(crate) use CONTAINING_RECORD;

pub mod types;
pub mod glue;

// Translated function modules (DeepSeek+c2rust). The #[no_mangle] Ks* exports +
// DriverEntry take over the cohab ks.sys export-driver link.
pub mod allocators;
pub mod api;
pub mod bag;
pub mod clocks;
pub mod connectivity;
pub mod device;
pub mod deviceinterface;
pub mod driver;
pub mod event;
pub mod filter;
pub mod filterfactory;
pub mod image;
pub mod irp;
pub mod kcom;
pub mod methods;
pub mod misc;
pub mod pin;
pub mod property;
pub mod swenum;
pub mod topology;
pub mod unknown;
pub mod worker;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
