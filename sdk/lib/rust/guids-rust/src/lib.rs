#![no_std]
#![allow(non_snake_case, non_upper_case_globals)]
//! guids-rust — the kernel's GUID data libs translated to Rust.
//!
//! Replaces the three cmake C GUID archives (sdk/lib/wdmguid, poguid, ioevent —
//! each a tiny .c that `#include <initguid.h>` then pulls a header whose
//! `DEFINE_GUID`s emit the GUID storage). Pure data: 217 `#[no_mangle]` GUID
//! constants (the union of the three libs, deduped). crate-type staticlib+rlib
//! so the cargo kernel links it via `.rust_lib("guids-rust")` in place of
//! libwdmguid.a/libpoguid.a/libioevent.a. The C sources stay on disk (never-
//! delete rule) for any remaining cmake consumers.

/// Win32 `GUID` (== `UUID`): 16 bytes, 4-byte aligned. `#[repr(C)]` matches the
/// C layout exactly, so each static is byte-identical to the C `DEFINE_GUID`
/// output the kernel references (Data1 LE u32, Data2/3 LE u16, Data4 raw).
#[repr(C)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

mod defs;

// Standalone staticlib needs its own panic handler + EH personality; the
// kernel link's --allow-multiple-definition absorbs the dup vs kernel-rust.
// (Unreachable — this crate is pure data, no panicking code.)
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
