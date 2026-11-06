//! The shipping rtl-sync SRW lock, compiled for the host against `fake-nt` (see Cargo.toml).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#![allow(clippy::all)]

#[path = "../../rtl-sync/src/atomics.rs"]
pub mod atomics;
/// Stands in for rtl-sync's `shared`: the PEB's processor count, set per round.
pub mod shared {
    pub static UNIPROCESSOR: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
    pub unsafe fn RtlpUniprocessor() -> bool {
        UNIPROCESSOR.load(core::sync::atomic::Ordering::Relaxed)
    }
}
#[path = "../../rtl-sync/src/srw.rs"]
pub mod srw;

pub use ntkrnl_sys::ddk::{KEYED_RELEASES, KEYED_WAITS, RTL_SRWLOCK};
