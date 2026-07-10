#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(clippy::all)]

//! npfs.sys — Rust translation of the Named Pipe filesystem driver
//! (drivers/filesystems/npfs). staticlib -> libnpfs_rust.a, cohab-linked into the cmake
//! npfs.sys with the 20 .c bodies commented out. NOT boot-loaded (not in txtsetup) → the
//! bar is link-clean ×3 + no smoke regression. WDM, imports ntoskrnl + hal only.
//!
//! Faithful translation; one module per original .c file. Shared substrate:
//!   types.rs   — the NP_VCB/DCB/FCB/CCB/NONPAGED_CCB node graph (npfs.h)
//!   glue.rs    — pool tags, bugcheck IDs, node-type codes, named-pipe + NTSTATUS constants
//!   globals.rs — NpVcb / NpAliasList / NpAliasListByLength
//!   helpers.rs — VCB lock wrappers, NpCompleteDeferredIrps, CONTAINING_RECORD
//! SEH (PSEH2) is dead under USE_DUMMY_PSEH (TRY inlines, FINALLY always runs, EXCEPT dropped).

pub mod glue;
pub mod globals;
pub mod helpers;
pub mod types;

// --- module bodies (one per .c) ---
pub mod cleanup;
pub mod close;
pub mod create;
pub mod datasup;
pub mod fileinfo;
pub mod fileobsup;
pub mod flushbuf;
pub mod fsctrl;
pub mod main;
pub mod prefxsup;
pub mod read;
pub mod readsup;
pub mod secursup;
pub mod seinfo;
pub mod statesup;
pub mod strucsup;
pub mod volinfo;
pub mod waitsup;
pub mod write;
pub mod writesup;

// arm64: KeGetCurrentPrcb is referenced by the ntkrnl_sys KeGetCurrentIrql inline but is not
// an ntoskrnl export on aarch64. Forward it to the register-reading inline (mirrors mup-rust).
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
