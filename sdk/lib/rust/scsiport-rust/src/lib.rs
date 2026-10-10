#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(clippy::all)]

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

// scsiport.sys — Rust translation of the SCSI port driver (drivers/storage/port/
// scsiport). staticlib -> libscsiport_rust.a, cohab-linked into the cmake scsiport.sys
// with the 8 .c commented out + spec2def + sptilib kept (guid.c stays C). Big EXPORT
// driver: the ~45 ScsiPort* exports come from the #[no_mangle] symbols here. Boot-loaded
// + on the LiveCD IDE read-stack via uniata/atapi ScsiPort* imports.

// Compile-time UTF-16 NUL-terminated literal: `wide!("ResourceList")` -> &'static [u16; N].
// The registry value names SpiParseDeviceInfo matches with _wcsnicmp are C `L"..."`
// literals; building them here rather than by hand is what keeps the terminating NUL
// from going missing (the whole reason rustconv grew --seal-wide). ASCII literals only.
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

pub mod types;
pub mod glue;
pub mod inlines;
pub mod ddk_shims;

pub mod fdo;
pub mod ioctl;
pub mod pdo;
pub mod power;
pub mod registry;
pub mod scsi;
pub mod scsiport;
pub mod stubs;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
