//! Rust impls for the `miarm.h` PTE bitfield-WRITE FORCEINLINEs (Phase 1
//! batch M2 — sibling to M1's read-only classifications). Same
//! augment+isolated-rlib pattern as the other Phase 1 inline rlibs.
//!
//! Each wrapper is `#[no_mangle] #[inline(never)] extern "C" fn _rs_Mi*`
//! and delegates to `ntkrnl_sys::inline::mm::*`. The C trampolines in
//! `ntoskrnl/mm/mm_rust_shims.c` collapse to one-line `_rs_Mi*` calls.
//!
//! 8 functions total — all bitfield writes against MMPTE. All arch-aware
//! at the inline:mm layer (e.g. ARM64 inverts Dirty as `NotDirty`,
//! `MiMakeWritePage` uses `Write`/`Writable` per arch).

#![no_std]

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::opaque_ptr;

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiMakeDirtyPage(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiMakeDirtyPage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiMakeCleanPage(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiMakeCleanPage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiMakeAccessedPage(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiMakeAccessedPage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiMakeOwnerPage(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiMakeOwnerPage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiMakeWritePage(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiMakeWritePage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiPageDisableCache(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiPageDisableCache(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiPageWriteThrough(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiPageWriteThrough(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiPageWriteCombined(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiPageWriteCombined(pte)
}
