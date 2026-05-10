//! Rust impls for the `miarm.h` / `mm.h` / `probe.h` / `exfuncs.h`
//! utility FORCEINLINEs (Phase 1 batch M3 — mm utility helpers).
//!
//! Six wrappers (no arch divergence at this layer):
//!
//! * `InterlockedIncrementUL` — atomic ULONG increment
//! * `InterlockedExchangeAddUL` — atomic ULONG exchange-add
//! * `MmSetDirtyPage` — wrapper for `MmSetDirtyBit(_, _, TRUE)`
//! * `ProbeForWriteSize_t` — probe a SIZE_T for write access
//! * `ProbeForWriteIoStatusBlock` — probe an IO_STATUS_BLOCK for write
//! * `ExAllocatePoolZero` — `ExAllocatePoolWithTag` + zero-fill
//!
//! Same augment+isolated-rlib pattern as M1/M2. The C trampolines in
//! `mm_rust_shims.c` keep their existing NTAPI / cdecl decoration as the
//! ABI anchor; bodies collapse to `_rs_*` calls. The Rust impls are all
//! `extern "C"` (cdecl) — the trampoline calls them with cdecl convention,
//! independent of the trampoline's own NTAPI/cdecl convention.

#![no_std]

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::opaque_ptr;

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_InterlockedIncrementUL(addend: *mut ULONG) -> ULONG {
    let addend = opaque_ptr(addend);
    ntkrnl_sys::inline::mm::InterlockedIncrementUL(addend)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_InterlockedExchangeAddUL(
    addend: *mut ULONG,
    value: ULONG,
) -> ULONG {
    let addend = opaque_ptr(addend);
    ntkrnl_sys::inline::mm::InterlockedExchangeAddUL(addend, value)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MmSetDirtyPage(process: PEPROCESS, address: PVOID) {
    let process = opaque_ptr(process);
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::mm::MmSetDirtyPage(process, address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_ProbeForWriteSize_t(ptr: *mut ULONG_PTR) {
    let ptr = opaque_ptr(ptr);
    // Bindgen `ddk::ULONG_PTR` is `u32` on i386; `inline::mm::ProbeForWriteSize_t`
    // takes the hand-written `base::ULONG_PTR` (= `usize`). Same width on every
    // arch — just satisfy the type checker.
    ntkrnl_sys::inline::mm::ProbeForWriteSize_t(ptr as *mut usize)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_ProbeForWriteIoStatusBlock(ptr: *mut IO_STATUS_BLOCK) {
    let ptr = opaque_ptr(ptr);
    ntkrnl_sys::inline::mm::ProbeForWriteIoStatusBlock(ptr)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_ExAllocatePoolZero(
    pool_type: POOL_TYPE,
    number_of_bytes: ULONG_PTR,
    tag: ULONG,
) -> PVOID {
    ntkrnl_sys::inline::mm::ExAllocatePoolZero(pool_type, number_of_bytes as _, tag)
}

/// `ExAllocatePoolZero` stdcall variant for i386 — bindgen on i386 declares
/// this as `extern "stdcall"` (NTAPI = stdcall there), producing
/// `_ExAllocatePoolZero@12`. Replaces the C trampoline that previously
/// lived in `ntoskrnl/mm/mm_rust_shims.c`. The cdecl variant that used to
/// live in `shim_exports.rs` was dropped 2026-05-09 along with its sole
/// caller (ARM3/virtual.rs flipped to bindgen's stdcall import).
#[no_mangle]
pub unsafe extern "system" fn ExAllocatePoolZero(
    pool_type: POOL_TYPE,
    number_of_bytes: ULONG_PTR,
    tag: ULONG,
) -> PVOID {
    _rs_ExAllocatePoolZero(pool_type, number_of_bytes, tag)
}
