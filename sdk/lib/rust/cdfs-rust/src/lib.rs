//! cdfs-rust — drivers/filesystems/cdfs → Rust (DeepSeek+deepdrive).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

// UNREFERENCED_PARAMETER is a macro in the DDK headers; an out-of-line no-op fn is an
// optimization barrier that can uncloak an ABI mismatch.
#[allow(unused_macros)]
macro_rules! UNREFERENCED_PARAMETER {
    ($($p:expr),* $(,)?) => {{ $( let _ = $p; )* }};
}

// PAGED_CODE() is the DDK's "pageable code" IRQL assertion (xdk/kefuncs.h): NOP_FUNCTION in
// a free build, and a no-op here. It is a macro, never a fn -- an out-of-line no-op call is an
// optimization barrier, and an `extern` one resolved by whichever crate happened to export it.
macro_rules! PAGED_CODE {
    () => {};
}

// cdprocs.h `AssertVerifyDevice` / `AssertVerifyDeviceIrp` (the NT_ASSERT forms are under
// `#if 0`) and cddata.h `ASSERT_OPTIONAL_IRP` (`{ NOTHING; }` without CD_SANITY): empty,
// operands never evaluated -- so token-discarding macros, not empty fns.
macro_rules! AssertVerifyDevice { ($($t:tt)*) => { () }; }
macro_rules! AssertVerifyDeviceIrp { ($($t:tt)*) => { () }; }
macro_rules! ASSERT_OPTIONAL_IRP { ($($t:tt)*) => { () }; }

pub mod types;
pub mod glue;       // harvested #define consts + grind-noise aliases
pub mod ddk_shims;  // FsRtl*/macro FORCEINLINE shims (shared w/ fastfat)
pub mod cdmacros;   // cdprocs.h/cdstruc.h/fsrtlfuncs.h MACRO shims (grind emitted as extern calls)

pub mod allocsup;
pub mod cachesup;
pub mod cddata;
pub mod cdinit;
pub mod cleanup;
pub mod close;
pub mod create;
pub mod devctrl;
pub mod deviosup;
pub mod dirctrl;
pub mod dirsup;
pub mod fileinfo;
pub mod fsctrl;
pub mod fspdisp;
pub mod lockctrl;
pub mod namesup;
pub mod pathsup;
pub mod pnp;
pub mod prefxsup;
pub mod read;
pub mod resrcsup;
pub mod shutdown;
pub mod strucsup;
pub mod verfysup;
pub mod volinfo;
pub mod workque;
pub mod write;

// staticlib (libcdfs_rust.a) needs a panic handler + eh personality (no_std cohab),
// gated off for `cargo test` where std provides them. Mirrors fs_rec-rust/disk-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
