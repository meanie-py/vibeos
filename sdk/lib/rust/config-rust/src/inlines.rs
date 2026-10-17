//! Rust impls of the eight KCB-lock FORCEINLINEs from `cm_x.h`.
//!
//! Architecturally this crate exists in **isolation** from `ntoskrnl-rust`
//! to dodge the LLVM captures(none) cross-CGU contagion that killed the
//! first Phase 1 pilot — see `docs/engineering/shim-elimination.md` §
//! "Phase 1 — FORCEINLINE batch — PILOT FAILED, plan blocked". The previous
//! attempt added `cm_inlines.rs` directly to `ntoskrnl-rust`; that triggers
//! rustc-side cross-CGU attribute deduction that lands ud2 in *neighbouring*
//! pre-existing config code (cmparse.rs, cmkcbncb.rs, …). With workspace
//! `lto = false`, a separate rlib compiles in its own rustc invocation, so
//! the cross-CGU inference can never see ntoskrnl-rust's bodies.
//!
//! The C side keeps the `Cmp{Acquire,Release,…}KcbLock*` symbols alive in
//! `cm_rust_shims.c`; their bodies now collapse to a single delegating call
//! into the `_rs_*` impls below (mirrors the `ntprintf` vararg trampoline
//! pattern). That means existing Rust callers (in `ntoskrnl-rust::config::*`)
//! keep using the unchanged `extern "C" { fn CmpAcquireKcbLockExclusive(...) }`
//! decls — no source change in `ntoskrnl-rust` for this round.

#![no_std]

use core::ffi::c_void;

use ntkrnl_sys::ddk::{CM_KEY_CONTROL_BLOCK, CM_KEY_HASH_TABLE_ENTRY, PEX_PUSH_LOCK};
use ntkrnl_sys::opaque_ptr;

/// Read the current thread pointer directly from the KPCR. Mirrors
/// `ntkrnl_sys::mm::KeGetCurrentThread`, but inlined here so the cross-crate
/// `#[inline(always)]` doesn't get dropped by rustc at opt-level=1 (which
/// would emit a `_KeGetCurrentThread` symbol reference for which there is
/// no exported function on i386 — the C-side definition is itself a
/// FORCEINLINE in `ke_x.h`).
#[inline(always)]
unsafe fn ke_get_current_thread() -> *mut c_void {
    let result: *mut c_void;
    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!(
            "mov {}, fs:[0x124]",
            out(reg) result,
            options(nostack, preserves_flags, readonly),
        );
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!(
            "mov {}, gs:[0x188]",
            out(reg) result,
            options(nostack, preserves_flags, readonly),
        );
    }
    #[cfg(target_arch = "aarch64")]
    {
        // Match ntkrnl_sys::mm::KeGetCurrentThread: walk PCR → CurrentPrcb →
        // CurrentThread. Reading TPIDR_EL1 → PKPCR; offset to CurrentPrcb,
        // then CurrentThread. To avoid replicating those layouts, defer to
        // the ntkrnl_sys impl on this arch (it always inlines on aarch64).
        result = ntkrnl_sys::mm::KeGetCurrentThread();
    }
    result
}

/// Hash-table tuning constants from `cm_x.h` / `cmkcbncb.c`. Held local to
/// avoid pulling another module dependency for two `const`s.
const CMP_HASH_IRRATIONAL: u32 = 314159269;
const CMP_HASH_PRIME: u32 = 1000000007;

/// Globals owned by `ntoskrnl-rust::config::cmkcbncb`. Declared `extern` here
/// so this crate links against them at static-archive merge time.
extern "C" {
    static mut CmpCacheTable: *mut CM_KEY_HASH_TABLE_ENTRY;
    static mut CmpHashTableSize: u32;
}

extern "C" {
    fn ExAcquirePushLockExclusive(push_lock: PEX_PUSH_LOCK);
    fn ExAcquirePushLockShared(push_lock: PEX_PUSH_LOCK);
    fn ExReleasePushLock(push_lock: PEX_PUSH_LOCK);
    fn ExConvertPushLockSharedToExclusive(push_lock: PEX_PUSH_LOCK) -> u8;
}

#[inline(always)]
unsafe fn get_hash_index(conv_key: u32) -> u32 {
    conv_key
        .wrapping_mul(CMP_HASH_IRRATIONAL)
        .wrapping_rem(CMP_HASH_PRIME)
        .wrapping_rem(core::ptr::read_volatile(core::ptr::addr_of!(CmpHashTableSize)))
}

/// `&CmpCacheTable[index]`, with `opaque_ptr` laundering on the table base
/// to short-circuit any future LLVM attribute deduction.
#[inline(always)]
unsafe fn entry_ptr(index: u32) -> *mut CM_KEY_HASH_TABLE_ENTRY {
    let base = opaque_ptr(core::ptr::read_volatile(
        core::ptr::addr_of!(CmpCacheTable),
    ));
    base.add(index as usize)
}

#[inline(always)]
unsafe fn lock_ptr(index: u32) -> PEX_PUSH_LOCK {
    core::ptr::addr_of_mut!((*entry_ptr(index)).Lock) as PEX_PUSH_LOCK
}

// ============================================================================
// Exclusive acquire
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn CmpAcquireKcbLockExclusiveByIndex(index: u32) {
    // cm_x.h's ASSERTs, restored (scan_dropped_asserts). ntkrnl_sys::ASSERT! prints
    // and continues, exactly as the C under DBG; this crate's debug_assert! would
    // stop the box. A recursive acquire would self-deadlock right after.
    ntkrnl_sys::ASSERT!(
        core::ptr::read_volatile(core::ptr::addr_of!((*entry_ptr(index)).Owner)) as *mut c_void
            != ke_get_current_thread()
    );
    ExAcquirePushLockExclusive(lock_ptr(index));
    let entry = entry_ptr(index);
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*entry).Owner),
        ke_get_current_thread() as _,
    );
}

#[no_mangle]
pub unsafe extern "C" fn CmpAcquireKcbLockExclusive(kcb: *mut CM_KEY_CONTROL_BLOCK) {
    let kcb = opaque_ptr(kcb);
    let conv_key = core::ptr::read_volatile(core::ptr::addr_of!(
        (*kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey
    ));
    CmpAcquireKcbLockExclusiveByIndex(get_hash_index(conv_key));
}

#[no_mangle]
pub unsafe extern "C" fn CmpAcquireKcbLockExclusiveByKey(conv_key: u32) {
    CmpAcquireKcbLockExclusiveByIndex(get_hash_index(conv_key));
}

// ============================================================================
// Release
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn CmpReleaseKcbLockByIndex(index: u32) {
    let entry = entry_ptr(index);
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*entry).Owner),
        core::ptr::null_mut(),
    );
    ExReleasePushLock(lock_ptr(index));
}

#[no_mangle]
pub unsafe extern "C" fn CmpReleaseKcbLock(kcb: *mut CM_KEY_CONTROL_BLOCK) {
    let kcb = opaque_ptr(kcb);
    let conv_key = core::ptr::read_volatile(core::ptr::addr_of!(
        (*kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey
    ));
    CmpReleaseKcbLockByIndex(get_hash_index(conv_key));
}

#[no_mangle]
pub unsafe extern "C" fn CmpReleaseKcbLockByKey(conv_key: u32) {
    CmpReleaseKcbLockByIndex(get_hash_index(conv_key));
}

// ============================================================================
// Convert (shared → exclusive)
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn CmpTryToConvertKcbSharedToExclusive(
    kcb: *mut CM_KEY_CONTROL_BLOCK,
) -> u8 {
    let kcb = opaque_ptr(kcb);
    ntkrnl_sys::ASSERT!(CmpIsKcbLockedExclusive(kcb) == 0);
    let conv_key = core::ptr::read_volatile(core::ptr::addr_of!(
        (*kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey
    ));
    let index = get_hash_index(conv_key);
    if ExConvertPushLockSharedToExclusive(lock_ptr(index)) != 0 {
        let entry = entry_ptr(index);
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*entry).Owner),
            ke_get_current_thread() as _,
        );
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn CmpConvertKcbSharedToExclusive(
    kcb: *mut CM_KEY_CONTROL_BLOCK,
) {
    ntkrnl_sys::ASSERT!(CmpIsKcbLockedExclusive(kcb) == 0);
    CmpReleaseKcbLock(kcb);
    CmpAcquireKcbLockExclusive(kcb);
}

// ============================================================================
// Shared KCB lock acquire (translated from cm_rust_shims.c on 2026-05-07)
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn CmpAcquireKcbLockSharedByIndex(index: u32) {
    ExAcquirePushLockShared(lock_ptr(index));
}

#[no_mangle]
pub unsafe extern "C" fn CmpAcquireKcbLockShared(kcb: *mut CM_KEY_CONTROL_BLOCK) {
    let kcb = opaque_ptr(kcb);
    let conv_key = core::ptr::read_volatile(core::ptr::addr_of!(
        (*kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey
    ));
    CmpAcquireKcbLockSharedByIndex(get_hash_index(conv_key));
}

// ============================================================================
// NCB locks — name-cache hash table parallel to KCB
// ============================================================================

use ntkrnl_sys::ddk::{CM_NAME_HASH_TABLE_ENTRY, PCM_NAME_HASH_TABLE_ENTRY};

extern "C" {
    static mut CmpNameCacheTable: PCM_NAME_HASH_TABLE_ENTRY;
}

#[inline(always)]
unsafe fn ncb_lock_ptr(index: u32) -> PEX_PUSH_LOCK {
    let base = opaque_ptr(core::ptr::read_volatile(core::ptr::addr_of!(CmpNameCacheTable)));
    core::ptr::addr_of_mut!((*base.add(index as usize)).Lock) as PEX_PUSH_LOCK
}

#[no_mangle]
pub unsafe extern "C" fn CmpAcquireNcbLockExclusiveByKey(conv_key: u32) {
    ExAcquirePushLockExclusive(ncb_lock_ptr(get_hash_index(conv_key)));
}

#[no_mangle]
pub unsafe extern "C" fn CmpReleaseNcbLockByKey(conv_key: u32) {
    ExReleasePushLock(ncb_lock_ptr(get_hash_index(conv_key)));
}

// ============================================================================
// CmpIsKcbLockedExclusive — checks Owner == current thread
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn CmpIsKcbLockedExclusive(kcb: *mut CM_KEY_CONTROL_BLOCK) -> u8 {
    let kcb = opaque_ptr(kcb);
    let conv_key = core::ptr::read_volatile(core::ptr::addr_of!(
        (*kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey
    ));
    let entry = entry_ptr(get_hash_index(conv_key));
    let owner = core::ptr::read_volatile(core::ptr::addr_of!((*entry).Owner));
    if owner as *mut c_void == ke_get_current_thread() { 1 } else { 0 }
}

// ============================================================================
// HHIVE function-pointer dispatch (HvGetCell / HvReleaseCell / HvGetCellType)
// ============================================================================

use ntkrnl_sys::ddk::{_HHIVE, HCELL_INDEX};

#[no_mangle]
pub unsafe extern "C" fn HvGetCell(hive: *mut _HHIVE, cell: HCELL_INDEX) -> *mut c_void {
    if let Some(get) = (*hive).GetCellRoutine {
        get(hive, cell) as *mut c_void
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn HvReleaseCell(hive: *mut _HHIVE, cell: HCELL_INDEX) {
    if let Some(release) = (*hive).ReleaseCellRoutine {
        release(hive, cell);
    }
}

const HCELL_TYPE_MASK: u32 = 0x80000000;
const HCELL_TYPE_SHIFT: u32 = 31;

#[no_mangle]
pub unsafe extern "C" fn HvGetCellType(cell: HCELL_INDEX) -> u32 {
    (cell & HCELL_TYPE_MASK) >> HCELL_TYPE_SHIFT
}

// ============================================================================
// Empty assertion / trace stubs (NDEBUG build) — bodies are deliberately empty
// ============================================================================

#[no_mangle] pub unsafe extern "C" fn CMP_ASSERT_REGISTRY_LOCK() {}
#[no_mangle] pub unsafe extern "C" fn CMP_ASSERT_FLUSH_LOCK(_h: *mut _HHIVE) {}
#[no_mangle] pub unsafe extern "C" fn CMP_ASSERT_KCB_LOCK(_k: *mut CM_KEY_CONTROL_BLOCK) {}
#[no_mangle] pub unsafe extern "C" fn CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK() {}
#[no_mangle] pub unsafe extern "C" fn CMP_ASSERT_REGISTRY_LOCK_OR_LOADING(_h: *mut _HHIVE) {}
#[no_mangle] pub unsafe extern "C" fn ASSERT_VALUE_BIG(_h: *mut _HHIVE, _l: u32) {}

#[no_mangle]
pub unsafe extern "C" fn CMTRACE(
    _filter: u32,
    _format: *const i8,
    _arg1: *const i8,
    _arg2: *mut c_void,
) {}

// ============================================================================
// Trivial returns (always-true / bitfield-extract / pointer cast)
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn IS_NULL_TERMINATED(_s: *mut u16, _l: u32) -> u8 { 1 }

#[no_mangle]
pub unsafe extern "C" fn LANGIDFROMLCID(lcid: u32) -> u32 {
    lcid & 0xFFFF
}

#[no_mangle]
pub unsafe extern "C" fn PtrToUlong(p: *mut c_void) -> u32 {
    p as usize as u32
}

#[no_mangle]
pub unsafe extern "C" fn GET_HASH_INDEX(conv_key: u32) -> u32 {
    get_hash_index(conv_key)
}

// ============================================================================
// CmpIsKeyValueSmall — translated from cmlib.h FORCEINLINE
// ============================================================================

const CM_KEY_VALUE_SPECIAL_SIZE: u32 = 0x80000000;

#[no_mangle]
pub unsafe extern "C" fn CmpIsKeyValueSmall(real_length: *mut u32, length: u32) -> u8 {
    if length >= CM_KEY_VALUE_SPECIAL_SIZE {
        core::ptr::write_volatile(real_length, length - CM_KEY_VALUE_SPECIAL_SIZE);
        1
    } else {
        core::ptr::write_volatile(real_length, length);
        0
    }
}
