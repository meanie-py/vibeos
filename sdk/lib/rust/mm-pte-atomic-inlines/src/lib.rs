//! Rust impls for the PTE atomic-update macros from `mm.h`
//! (Phase 1 batch M6).
//!
//! Two wrappers:
//!
//! * `MiInterlockedCompareExchangePte` — atomic CAS on a PTE
//! * `MiInterlockedExchangePte` — atomic swap on a PTE
//!
//! In C these are macros that expand to `InterlockedCompareExchange[64]` /
//! `InterlockedExchange[64]` with the PTE pointer cast to `PLONG[64]`.
//! `inline::mm` already provides matching `core::sync::atomic`-based
//! implementations — the per-arch divergence (32-bit on i386, 64-bit on
//! amd64/arm64) is handled inside that module via `cfg(target_arch)`. No
//! inline asm needed in the rlib.
//!
//! Same augment+isolated-rlib pattern as M1/M2/M3/M4/M5. C trampolines in
//! `mm_rust_shims.c` keep the `Mi*` symbols as the ABI anchor; bodies
//! collapse to one-line `_rs_*` calls.

#![no_std]

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::opaque_ptr;

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiInterlockedCompareExchangePte(
    pte: PMMPTE,
    exchange: LONG_PTR,
    comperand: LONG_PTR,
) -> LONG_PTR {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiInterlockedCompareExchangePte(pte, exchange as _, comperand as _)
        as LONG_PTR
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiInterlockedExchangePte(
    pte: PMMPTE,
    value: LONG_PTR,
) -> LONG_PTR {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiInterlockedExchangePte(pte, value as _) as LONG_PTR
}
