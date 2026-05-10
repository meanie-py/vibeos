//! Rust impls for the `miarm.h` PTE-classification FORCEINLINEs (Phase 1
//! batch M1). The C-side trampolines in `ntoskrnl/mm/mm_rust_shims.c` keep
//! their existing `Mi*` symbols and collapse to one-line delegations
//! (`return _rs_Mi*(...)`) — same augment+isolated-rlib pattern as
//! `cm-inlines` / `ke-inlines` / `ob-inlines` / `ps-inlines` / `se-inlines`
//! / `common-inlines`.
//!
//! Each wrapper is `#[no_mangle] #[inline(never)] extern "C" fn _rs_Mi*` —
//! launders pointer arguments through `opaque_ptr` and delegates to the
//! corresponding `ntkrnl_sys::inline::mm::*` body. Behaviour is identical
//! to the existing C-macro / `_orig_*` paths.
//!
//! 7 functions total — the read-only PTE classifications listed in the
//! Phase 1 plan (`docs/engineering/shim-elimination.md` § M1):
//!
//! * `MiIsMappedPte`            — all arches
//! * `MiIsPageWriteable`        — all arches
//! * `MiIsPageDirty`            — all arches
//! * `MiIsUserPde`              — all arches
//! * `MiIsUserPte`              — all arches
//! * `MiDetermineUserGlobalPteMask` — all arches
//! * `MiIsPdeForAddressValid`   — amd64/arm64 only (i386 paging hierarchy
//!   doesn't have PXE/PPE)

#![no_std]

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::opaque_ptr;

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiIsMappedPte(pte: PMMPTE) -> BOOLEAN {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiIsMappedPte(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiIsPageWriteable(pte: PMMPTE) -> BOOLEAN {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiIsPageWriteable(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiIsPageDirty(pte: PMMPTE) -> BOOLEAN {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiIsPageDirty(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiIsUserPde(address: PVOID) -> BOOLEAN {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::mm::MiIsUserPde(address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiIsUserPte(address: PVOID) -> BOOLEAN {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::mm::MiIsUserPte(address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiDetermineUserGlobalPteMask(pte: PMMPTE) -> ULONG_PTR {
    let pte = opaque_ptr(pte);
    // `inline::mm::MiDetermineUserGlobalPteMask` returns the hand-written
    // `usize`-typed `ULONG_PTR` from `ntkrnl_sys::base`; bindgen's
    // `ddk::ULONG_PTR` is `u32` on i386. Same value either way — just need
    // to satisfy the type checker.
    ntkrnl_sys::inline::mm::MiDetermineUserGlobalPteMask(pte) as ULONG_PTR
}

// MiIsPdeForAddressValid: amd64/arm64 only (relies on PXE/PPE which i386
// doesn't have in its 2-level paging hierarchy). The C trampoline in
// `mm_rust_shims.c` is also gated on `_M_AMD64 || _M_ARM64`, so the symbol
// only needs to exist on those arches.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiIsPdeForAddressValid(address: PVOID) -> BOOLEAN {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::mm::MiIsPdeForAddressValid(address)
}
