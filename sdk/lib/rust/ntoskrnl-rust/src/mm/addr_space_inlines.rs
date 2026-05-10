//! Rust impls for the address-space FORCEINLINEs from `mm.h`
//! (Phase 1 batch M5).
//!
//! Five wrappers (no arch divergence at this layer):
//!
//! * `MmGetCurrentAddressSpace` — current process's address space
//! * `MmGetKernelAddressSpace` — pointer to kernel address space
//! * `MmGetAddressSpaceOwner` — EPROCESS that owns a given MMSUPPORT
//! * `MmLockAddressSpace` — acquire AddressCreationLock guarded mutex
//! * `MmUnlockAddressSpace` — release AddressCreationLock guarded mutex
//!
//! Same augment+isolated-rlib pattern as M1/M2/M3/M4. The C trampolines in
//! `mm_rust_shims.c` retain the `Mm*AddressSpace` symbols (they're
//! PCH-covered FORCEINLINEs and can't be `_orig_*`-renamed); the bodies
//! collapse to one-line `_rs_*` calls into Rust.

#![no_std]

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::opaque_ptr;

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MmGetCurrentAddressSpace() -> PMMSUPPORT {
    ntkrnl_sys::inline::mm::MmGetCurrentAddressSpace()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MmGetKernelAddressSpace() -> PMMSUPPORT {
    ntkrnl_sys::inline::mm::MmGetKernelAddressSpace()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MmGetAddressSpaceOwner(address_space: PMMSUPPORT) -> PEPROCESS {
    let address_space = opaque_ptr(address_space);
    ntkrnl_sys::inline::mm::MmGetAddressSpaceOwner(address_space)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MmLockAddressSpace(address_space: PMMSUPPORT) {
    let address_space = opaque_ptr(address_space);
    ntkrnl_sys::inline::mm::MmLockAddressSpace(address_space)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MmUnlockAddressSpace(address_space: PMMSUPPORT) {
    let address_space = opaque_ptr(address_space);
    ntkrnl_sys::inline::mm::MmUnlockAddressSpace(address_space)
}
