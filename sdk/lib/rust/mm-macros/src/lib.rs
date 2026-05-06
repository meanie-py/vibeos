//! Rust impls of `#define` macros from `ntoskrnl/include/internal/mm.h`,
//! `sdk/include/xdk/mmtypes.h`, and `ntoskrnl/mm/ARM3/miarm.h` whose C
//! trampolines live in `ntoskrnl/mm/mm_rust_shims.c`. Phase 2 sibling rlib
//! (mirror of `ke-macros` for ke). Same augment+isolated-rlib pattern as
//! every other `*-inlines` / `*-macros` sibling: workspace `lto = false`
//! plus the crate boundary defuses the cross-CGU captures(none) attribute
//! deduction wire.
//!
//! Pilot batch (4 fns): trivial bit-twiddle macros from `mm.h` /
//! `mmtypes.h` — RMAP_IS_SEGMENT, PAGE_ROUND_DOWN, PFN_FROM_SSE,
//! MM_IS_WAIT_PTE. (`InterlockedExchangeAddUL` and `MmSetDirtyPage`
//! already live in `mm-util-inlines` from Phase 1 batch M3.)

#![no_std]

/// `PAGE_SHIFT` from `xdk/{amd64,arm64,arm,x86}/ke.h` — 12 on every supported
/// arch (4 KiB pages).
const PAGE_SHIFT: u32 = 12;
/// `PAGE_SIZE` = 1 << PAGE_SHIFT.
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

/// `RMAP_SEGMENT_MASK` from `ntoskrnl/include/internal/mm.h`:
/// `~((ULONG_PTR)0xff)` — high bits of an RMAP entry encode the segment;
/// low byte is reserved.
const RMAP_SEGMENT_MASK: usize = !0xffusize;

/// `MM_WAIT_ENTRY` from `ntoskrnl/include/internal/mm.h`. Bit pattern
/// differs per pointer width (low bit is the "wait" flag, the rest is the
/// sentinel value).
#[cfg(target_pointer_width = "64")]
const MM_WAIT_ENTRY: usize = 0x7FFF_FFFF_FFFF_FC00;
#[cfg(target_pointer_width = "32")]
const MM_WAIT_ENTRY: usize = 0x7fff_fc00;

/// RMAP_IS_SEGMENT — `(((ULONG_PTR)Address & RMAP_SEGMENT_MASK) == RMAP_SEGMENT_MASK)`.
#[no_mangle]
pub unsafe extern "C" fn _rs_RMAP_IS_SEGMENT(address: *const core::ffi::c_void) -> u8 {
    if (address as usize) & RMAP_SEGMENT_MASK == RMAP_SEGMENT_MASK {
        1
    } else {
        0
    }
}

/// PAGE_ROUND_DOWN — `((ULONG_PTR)Address & ~(PAGE_SIZE - 1))`.
#[no_mangle]
pub unsafe extern "C" fn _rs_PAGE_ROUND_DOWN(address: *const core::ffi::c_void) -> usize {
    (address as usize) & !(PAGE_SIZE - 1)
}

/// PFN_FROM_SSE — `((PFN_NUMBER)(Entry >> PAGE_SHIFT))`. PFN_NUMBER is
/// pointer-width on Windows (`ULONG_PTR`-typedef under most paths in
/// ntoskrnl); we return `usize` which matches.
#[no_mangle]
pub unsafe extern "C" fn _rs_PFN_FROM_SSE(entry: usize) -> usize {
    entry >> PAGE_SHIFT
}

/// MM_IS_WAIT_PTE — `((Entry & 0x1) && ((Entry >> 1) == MM_WAIT_ENTRY))`.
#[no_mangle]
pub unsafe extern "C" fn _rs_MM_IS_WAIT_PTE(entry: usize) -> u8 {
    if (entry & 0x1) != 0 && (entry >> 1) == MM_WAIT_ENTRY {
        1
    } else {
        0
    }
}
