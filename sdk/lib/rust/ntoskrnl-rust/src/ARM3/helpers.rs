//! Shared helpers, constants, type aliases, extern declarations, and inline functions
//! needed by the Tier 3a ARM3 translated files (pfnlist, session, mminit, pagfault,
//! mdlsup, procsup, sysldr).
//!
//! This module bridges gaps between the LLM-translated code and what ntkrnl-sys provides.
//! It does NOT modify ntkrnl-sys itself.
//!
//! IMPORTANT: This module must NOT re-define symbols that are already available via:
//!   - ntkrnl_sys::base::* (_1KB, _1MB, PAGE_SIZE, PVOID, etc.)
//!   - ntkrnl_sys::mm::* (MMPTE, MMPFN, PMMPTE, PMMPFN, PMMPDE, PFN_FROM_PTE, PTE_PER_PAGE, etc.)
//!   - ntkrnl_sys::ke::* (KIRQL, etc.)
//!   - ntkrnl_sys::constants::*

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
pub use crate::ke::processor::KeActiveProcessors;
use ntkrnl_sys::ke::*;

// ============================================================================
// Type aliases NOT already in ntkrnl-sys
// ============================================================================

pub type c_void = core::ffi::c_void;
// _SECURITY_DESCRIPTOR: use ntkrnl_sys::ddk::_SECURITY_DESCRIPTOR (not opaque — needs full struct for stack alloc)
pub type PPFN_NUMBER = *mut PFN_NUMBER;
pub type PSHORT = *mut i16;

// ============================================================================
// Size constant NOT already in base.rs
// ============================================================================

pub const _64K: usize = 64 * 1024;

// ============================================================================
// Architecture-dependent memory layout constants
// ============================================================================

// MI_USER_PROBE_ADDRESS
#[cfg(target_arch = "x86")]
pub const MI_USER_PROBE_ADDRESS: ULONG_PTR = 0x7FFF0000;
#[cfg(target_arch = "x86_64")]
pub const MI_USER_PROBE_ADDRESS: ULONG_PTR = 0x000007FFFFFF0000;
#[cfg(target_arch = "aarch64")]
pub const MI_USER_PROBE_ADDRESS: ULONG_PTR = 0x7FFFFFFEFFFF;

// MI_HIGHEST_USER_ADDRESS
pub const MI_HIGHEST_USER_ADDRESS: ULONG_PTR = MM_HIGHEST_USER_ADDRESS;

// MI_DEFAULT_SYSTEM_RANGE_START
#[cfg(target_arch = "x86")]
pub const MI_DEFAULT_SYSTEM_RANGE_START: ULONG_PTR = 0x80000000;
// NOTE: The C header defines MI_DEFAULT_SYSTEM_RANGE_START as 0xFFFF080000000000 but
// also has #define MmSystemRangeStart ((PVOID)MI_REAL_SYSTEM_RANGE_START) with
// MI_REAL_SYSTEM_RANGE_START = 0xFFFF800000000000. Since Rust has no macro override,
// we use the canonical MI_REAL value here. The MI_DEFAULT value (0xFFFF080000000000)
// is non-canonical on 48-bit VA CPUs and causes #GP when accessed.
#[cfg(target_arch = "x86_64")]
pub const MI_DEFAULT_SYSTEM_RANGE_START: ULONG_PTR = 0xFFFF800000000000;
#[cfg(target_arch = "aarch64")]
pub const MI_DEFAULT_SYSTEM_RANGE_START: ULONG_PTR = 0xFFFF800000000000;

// MI_SYSTEM_CACHE_WS_START
#[cfg(target_arch = "x86")]
pub const MI_SYSTEM_CACHE_WS_START: ULONG_PTR = 0xC0C00000;
#[cfg(target_arch = "x86_64")]
pub const MI_SYSTEM_CACHE_WS_START: ULONG_PTR = 0xFFFFF78000001000;
#[cfg(target_arch = "aarch64")]
pub const MI_SYSTEM_CACHE_WS_START: ULONG_PTR = 0xFFFFF78000000000;

// MI_SYSTEM_CACHE_START
#[cfg(target_arch = "x86")]
pub const MI_SYSTEM_CACHE_START: ULONG_PTR = 0xC1000000;
#[cfg(target_arch = "x86_64")]
pub const MI_SYSTEM_CACHE_START: ULONG_PTR = 0xFFFFF98000000000;
#[cfg(target_arch = "aarch64")]
pub const MI_SYSTEM_CACHE_START: ULONG_PTR = 0xFFFFF98000000000;

// MI_SYSTEM_CACHE_END
#[cfg(target_arch = "x86")]
pub const MI_SYSTEM_CACHE_END: ULONG_PTR = 0xC1000000 + 512 * _1MB;
#[cfg(target_arch = "x86_64")]
pub const MI_SYSTEM_CACHE_END: ULONG_PTR = 0xFFFFFA7FFFFFFFFF;
#[cfg(target_arch = "aarch64")]
pub const MI_SYSTEM_CACHE_END: ULONG_PTR = 0xFFFFFA7FFFFFFFFF;

// MI_PAGED_POOL_START
#[cfg(target_arch = "x86")]
pub const MI_PAGED_POOL_START: ULONG_PTR = 0xE1000000;
#[cfg(target_arch = "x86_64")]
pub const MI_PAGED_POOL_START: ULONG_PTR = 0xFFFFF8A000000000;
#[cfg(target_arch = "aarch64")]
pub const MI_PAGED_POOL_START: ULONG_PTR = 0xFFFFFA8000000000;

// MI_NONPAGED_POOL_END
#[cfg(target_arch = "x86")]
pub const MI_NONPAGED_POOL_END: ULONG_PTR = 0xFFBE0000;
#[cfg(target_arch = "x86_64")]
pub const MI_NONPAGED_POOL_END: ULONG_PTR = 0xFFFFFFFFFFBFFFFF;
#[cfg(target_arch = "aarch64")]
pub const MI_NONPAGED_POOL_END: ULONG_PTR = 0xFFFFFAFFFFFFFFFF;

// PDE_MAPPED_VA (PTE_PER_PAGE is in mm.rs)
pub const PDE_MAPPED_VA: usize = ntkrnl_sys::mm::PTE_PER_PAGE * PAGE_SIZE;

// MM_HAL_VA_START / MM_HAL_VA_END
#[cfg(target_arch = "x86")]
pub const MM_HAL_VA_START: ULONG_PTR = 0xFFC00000;
#[cfg(target_arch = "x86")]
pub const MM_HAL_VA_END: ULONG_PTR = 0xFFFFFFFF;
#[cfg(target_arch = "x86_64")]
pub const MM_HAL_VA_START: ULONG_PTR = 0xFFFFFFFFFFC00000;
#[cfg(target_arch = "x86_64")]
pub const MM_HAL_VA_END: ULONG_PTR = 0xFFFFFFFFFFFFFFFF;
#[cfg(target_arch = "aarch64")]
pub const MM_HAL_VA_START: ULONG_PTR = 0xFFFFFFFFFFC00000;
#[cfg(target_arch = "aarch64")]
pub const MM_HAL_VA_END: ULONG_PTR = 0xFFFFFFFFFFFFFFFF;

// PXE_TOP (64-bit only)
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const PXE_TOP: usize = PXE_BASE + PXE_PER_PAGE * 8 - 1;

// PDE_TOP
#[cfg(target_arch = "x86")]
pub const PDE_TOP: usize = PDE_BASE + ntkrnl_sys::mm::PDE_PER_PAGE * 4 - 1;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const PDE_TOP: usize = PDE_BASE + ntkrnl_sys::mm::PDE_PER_PAGE * ntkrnl_sys::mm::PDE_PER_PAGE * 8 - 1;

// USER_SHARED_DATA
pub const USER_SHARED_DATA: ULONG_PTR = 0x7FFE0000;

// ============================================================================
// Memory tuning constants
// ============================================================================

pub const MI_MIN_PAGES_FOR_SYSPTE_TUNING: usize = (19 * _1MB) >> PAGE_SHIFT;
pub const MI_MIN_PAGES_FOR_SYSPTE_BOOST: usize = (32 * _1MB) >> PAGE_SHIFT;
pub const MI_MIN_PAGES_FOR_SYSPTE_BOOST_BOOST: usize = (256 * _1MB) >> PAGE_SHIFT;
pub const MI_MIN_INIT_PAGED_POOLSIZE: usize = 32 * _1MB;

pub const MI_MIN_ALLOCATION_FRAGMENT: usize = 4 * _1KB;
pub const MI_ALLOCATION_FRAGMENT: usize = 64 * _1KB;
pub const MI_MAX_ALLOCATION_FRAGMENT: usize = 2 * _1MB;

pub const MI_MIN_SECONDARY_COLORS: usize = 8;
pub const MI_SECONDARY_COLORS: usize = 64;
pub const MI_MAX_SECONDARY_COLORS: usize = 1024;

pub const MI_HYPERSPACE_PTES: usize = 255;

// MM_EMPTY_LIST
pub const MM_EMPTY_LIST: ULONG_PTR = ULONG_PTR::MAX;

// MM_PTE_SOFTWARE_PROTECTION_BITS — where a SOFTWARE PTE keeps its Protection.
//
// This is the shift WRITERS use to build the PTE templates below; the READER is
// `ntkrnl_sys::mm::MMPTE::soft_protection()`. They must agree, and on arm64 they
// did not: the reader uses 1 (per ndk/arm64/mmtypes.h) while this said 6, which
// is what ntoskrnl/include/internal/arm64/mm.h still carried.
//
// The header settles it. arm64 MMPTE_SOFTWARE is
// `Valid:1, Protection:5, PageFileLow:4, Prototype:1, Transition:1, ...`, so a
// 5-bit Protection at shift 6 would occupy bits 6..10 — overlapping PageFileLow
// (6..9) and colliding with Prototype (bit 10). Structurally impossible for this
// layout, and every other software variant in that header (TRANSITION,
// PROTOTYPE, SUBSECTION, LIST) puts Protection at bits 1..5 too. Shift 1 is
// correct; 6 was wrong.
//
// What the mismatch cost (task #39): MmDecommittedPte is MM_DECOMMIT << shift,
// and MM_DECOMMIT is 0x10, so at shift 6 it built 0x400 — which IS the Prototype
// bit. Every reader then saw that PTE as "Prototype=1, Protection=0". A fault on
// a decommitted page therefore took neither the decommitted path (access
// violation) nor the prototype path, and fell through to demand-zero with
// Protection 0. That reached MI_MAKE_HARDWARE_PTE_USER, whose three C ASSERTs
// against exactly that had been dropped in translation, and on arm64 the result
// is Valid=1 on a RESERVED level-3 descriptor: the MMU refuses the translation
// forever while MmAccessFault reads Valid and reports success, with DAIF masked.
// A dead machine, no bugcheck, PC parked. Two independent defects had to line
// up; either alone is survivable.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub const MM_PTE_SOFTWARE_PROTECTION_BITS: u32 = 5;
#[cfg(target_arch = "aarch64")]
pub const MM_PTE_SOFTWARE_PROTECTION_BITS: u32 = 1;

// PTE_READWRITE
#[cfg(target_arch = "x86")]
pub const PTE_READWRITE: ULONG_PTR = 0x2;
#[cfg(target_arch = "x86_64")]
pub const PTE_READWRITE: ULONG_PTR = 0x0000000000000002u64 as ULONG_PTR; // R/W without NX — ReactOS has no NonPagedPoolNx, so NonPagedPool must be executable
#[cfg(target_arch = "aarch64")]
pub const PTE_READWRITE: ULONG_PTR = 0x00C0000000000000u64 as ULONG_PTR; // UXN=1, Writable=1

// PTE_READONLY — on x86 0 (no separate read-only bit), on x86_64 NX bit set
#[cfg(target_arch = "x86")]
pub const PTE_READONLY: ULONG_PTR = 0;
#[cfg(target_arch = "x86_64")]
pub const PTE_READONLY: ULONG_PTR = 0x8000000000000000u64 as ULONG_PTR;
#[cfg(target_arch = "aarch64")]
pub const PTE_READONLY: ULONG_PTR = 0x0040000000000000u64 as ULONG_PTR;

// PTE_EXECUTE — on x86/x64 0 (execute is implicit), on arm64 0
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
pub const PTE_EXECUTE: ULONG_PTR = 0x0;

// PTE_EXECUTE_READ — same as execute on x86
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
pub const PTE_EXECUTE_READ: ULONG_PTR = PTE_EXECUTE;

// PTE_WRITECOPY — copy-on-write bit
#[cfg(target_arch = "x86")]
pub const PTE_WRITECOPY: ULONG_PTR = 0x200;
#[cfg(target_arch = "x86_64")]
pub const PTE_WRITECOPY: ULONG_PTR = 0x8000000000000200u64 as ULONG_PTR;
#[cfg(target_arch = "aarch64")]
pub const PTE_WRITECOPY: ULONG_PTR = 0x0140000000000000u64 as ULONG_PTR;

// PTE_EXECUTE_READWRITE — read/write/execute
#[cfg(target_arch = "x86")]
pub const PTE_EXECUTE_READWRITE: ULONG_PTR = 0x2;
#[cfg(target_arch = "x86_64")]
pub const PTE_EXECUTE_READWRITE: ULONG_PTR = 0x2;
#[cfg(target_arch = "aarch64")]
pub const PTE_EXECUTE_READWRITE: ULONG_PTR = 0x0080000000000000u64 as ULONG_PTR;

// PTE_EXECUTE_WRITECOPY — execute + copy-on-write
#[cfg(target_arch = "x86")]
pub const PTE_EXECUTE_WRITECOPY: ULONG_PTR = 0x200;
#[cfg(target_arch = "x86_64")]
pub const PTE_EXECUTE_WRITECOPY: ULONG_PTR = 0x200;
#[cfg(target_arch = "aarch64")]
pub const PTE_EXECUTE_WRITECOPY: ULONG_PTR = 0x0100000000000000u64 as ULONG_PTR;

// PTE_PROTOTYPE — prototype PTE bit
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
pub const PTE_PROTOTYPE: ULONG_PTR = 0x400;

// PTE_TRANSITION — transition PTE bit
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
pub const PTE_TRANSITION: ULONG_PTR = 0x800;

// PTE_ENABLE_CACHE — enable cache (0 on x86/x64; arm64 AttrIndx in bits[3:2]).
// freeldr programs MAIR_EL1=0x004400FF: AttrIndx 0=Normal-WB, 1=Device-nGnRnE,
// 2=Normal-NC (winldr.c WinLdrSetProcessorContext). So cached normal memory is
// AttrIndx 0 → value 0x0. The previous 0x4 selected AttrIndx 1 = Device, which
// mapped every "cached" kernel page (pool, NLS tables) as Device memory: aligned
// accesses worked but any *unaligned* access alignment-faulted (DFSC 0x21, A=0) —
// the cold-build 0x1E cascade (RtlInitCodePageTable, ObpAllocateObject SIMD store).
// stubs.c's comment had the correct mapping; miarm.h/here had it bit-swapped.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub const PTE_ENABLE_CACHE: ULONG_PTR = 0x0;
#[cfg(target_arch = "aarch64")]
pub const PTE_ENABLE_CACHE: ULONG_PTR = 0x0;

// PTE_DISABLE_CACHE — disable cache (PWT+PCD on x86; AttrIndx 1 = Device on arm64).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub const PTE_DISABLE_CACHE: ULONG_PTR = 0x10;
#[cfg(target_arch = "aarch64")]
pub const PTE_DISABLE_CACHE: ULONG_PTR = 0x4;

// PTE_WRITECOMBINED_CACHE — write-combined (same as disable on x86; AttrIndx 2 =
// Normal-NC on arm64).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub const PTE_WRITECOMBINED_CACHE: ULONG_PTR = 0x10;
#[cfg(target_arch = "aarch64")]
pub const PTE_WRITECOMBINED_CACHE: ULONG_PTR = 0x8;

// ============================================================================
// InterlockedExchangePte — atomic swap of PTE value
// ============================================================================

/// InterlockedExchangePte — atomically exchange a PTE with a new value.
/// Returns the old PTE value.
#[inline]
pub unsafe fn InterlockedExchangePte(pte: PMMPTE, new_val: ULONG_PTR) -> ULONG_PTR {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(pte as *const AtomicUsize);
    atomic.swap(new_val, Ordering::SeqCst)
}

// ============================================================================
// MiGetPdeOffset — get PDE index for a virtual address
// ============================================================================

/// MiGetPdeOffset — returns the page directory entry offset for a VA.
/// On x86: VA >> 22 (1024 PTEs per page, each covers 4MB)
/// On x86_64/arm64: (VA >> 21) & 0x1FF (512 PDEs, each covers 2MB)
#[inline]
pub fn MiGetPdeOffset(va: PVOID) -> usize {
    #[cfg(target_arch = "x86")]
    { (va as usize) >> 22 }
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    { ((va as usize) >> 21) & 0x1FF }
}

// ============================================================================
// MiSynchronizeSystemPde — synchronize a system PDE (i386 only)
// Returns 1 if the PDE was already valid, 0 if it was synchronised from kernel
// ============================================================================

/// MiSynchronizeSystemPde_Bool — wrapper that returns BOOLEAN (as from DDK).
/// On i386, the real function is in the DDK. This provides a Rust-friendly wrapper
/// that works with the hand-written PMMPTE type.
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn MiSynchronizeSystemPde_Bool(pde: PMMPTE) -> BOOLEAN {
    use ntkrnl_sys::ddk::MiSynchronizeSystemPde as DdkMiSynchronizeSystemPde;
    DdkMiSynchronizeSystemPde(pde as *mut ntkrnl_sys::ddk::_MMPTE)
}

// ============================================================================
// MMPTE bitfield accessor helpers
// ============================================================================

#[inline]
pub fn mmpte_software_protection(pte: &MMPTE) -> ULONG {
    pte.soft_protection()
}

#[inline]
pub fn mmpte_software_prototype(pte: &MMPTE) -> bool {
    pte.prototype()
}

#[inline]
pub fn mmpte_software_transition(pte: &MMPTE) -> bool {
    pte.transition()
}

// ============================================================================
// NTSTATUS codes NOT already in ntkrnl-sys
// ============================================================================

// STATUS_INVALID_ADDRESS moved to ntkrnl-sys/constants.rs
pub const STATUS_IN_PAGE_ERROR: NTSTATUS = 0xC0000006u32 as NTSTATUS;
pub const STATUS_GUARD_PAGE_VIOLATION: NTSTATUS = 0x80000001u32 as NTSTATUS;
pub const STATUS_STACK_OVERFLOW: NTSTATUS = 0xC00000FDu32 as NTSTATUS;
pub const STATUS_PAGE_FAULT_DEMAND_ZERO: NTSTATUS = 0x00000111;
pub const STATUS_PAGE_FAULT_TRANSITION: NTSTATUS = 0x00000110;
pub const STATUS_PAGE_FAULT_COPY_ON_WRITE: NTSTATUS = 0x00000112;
pub const STATUS_PAGE_FAULT_GUARD_PAGE: NTSTATUS = 0x00000113;
pub const STATUS_RETRY: NTSTATUS = 0xC000022Du32 as NTSTATUS;
// STATUS_UNABLE_TO_FREE_VM moved to ntkrnl-sys/constants.rs
pub const STATUS_INVALID_SYSTEM_SERVICE: NTSTATUS = 0xC000001Cu32 as NTSTATUS;
pub const STATUS_DRIVER_ENTRYPOINT_NOT_FOUND: NTSTATUS = 0xC0000263u32 as NTSTATUS;
pub const STATUS_DRIVER_ORDINAL_NOT_FOUND: NTSTATUS = 0xC0000262u32 as NTSTATUS;
pub const STATUS_ENTRYPOINT_NOT_FOUND: NTSTATUS = 0xC0000139u32 as NTSTATUS;
pub const STATUS_IMAGE_MACHINE_TYPE_MISMATCH: NTSTATUS = 0x4000000Eu32 as NTSTATUS;
pub const STATUS_INVALID_IMAGE_FORMAT: NTSTATUS = 0xC000007Bu32 as NTSTATUS;
pub const STATUS_INVALID_IMAGE_PROTECT: NTSTATUS = 0xC0000130u32 as NTSTATUS;

// ============================================================================
// Bugcheck codes
// ============================================================================

pub const PAGE_FAULT_IN_NONPAGED_AREA: ULONG = 0x50;
pub const ATTEMPTED_WRITE_TO_READONLY_MEMORY: ULONG = 0xBE;
pub const ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY: ULONG = 0xFC;
pub const DRIVER_CAUGHT_MODIFYING_FREED_POOL: ULONG = 0xC6;
pub const INSTALL_MORE_MEMORY: ULONG = 0x7D;

// ============================================================================
// Page protection constants (from winnt.h)
// ============================================================================

// PAGE_READONLY, PAGE_EXECUTE, PAGE_GUARD, PAGE_NOACCESS, PAGE_READWRITE, PAGE_NOCACHE
// moved to ntkrnl-sys (mm.rs / base.rs)

// ============================================================================
// MDL flags
// ============================================================================

pub const MDL_IO_SPACE: ULONG = 0x800;
pub const MDL_MAPPED_TO_SYSTEM_VA: ULONG = 0x0001;
pub const MDL_SOURCE_IS_NONPAGED_POOL: ULONG = 0x0004;
pub const MDL_PARTIAL: ULONG = 0x10;
pub const MDL_WRITE_OPERATION: ULONG = 0x0080;

// ============================================================================
// Miscellaneous constants
// ============================================================================

// Per-arch kernel stack sizes — must match sdk/include/xdk/<arch>/ke.h.
// GUI threads transition from KERNEL_STACK_SIZE to a large stack via
// KeSwitchKernelStack; KERNEL_LARGE_STACK_COMMIT is the *committed* size of
// that large stack, and must be >= KERNEL_STACK_SIZE — otherwise switching to
// GUI shrinks the committed stack and a deep call (e.g. RtlVirtualUnwind chain
// from RtlWalkFrameChain) page-faults on a guard page → DOUBLE FAULT (0x7F/8).
#[cfg(target_arch = "x86_64")]
pub const KERNEL_STACK_SIZE: usize = 0xC000; // 48KB — matches xdk/amd64/ke.h
#[cfg(target_arch = "x86_64")]
pub const KERNEL_LARGE_STACK_SIZE: usize = 0x12000; // 72KB — matches xdk/amd64/ke.h
#[cfg(target_arch = "x86_64")]
pub const KERNEL_LARGE_STACK_COMMIT: usize = KERNEL_STACK_SIZE; // 48KB — matches xdk/amd64/ke.h

#[cfg(target_arch = "x86")]
pub const KERNEL_STACK_SIZE: usize = 12 * 1024; // 12KB — matches xdk/x86/ke.h
#[cfg(target_arch = "x86")]
pub const KERNEL_LARGE_STACK_SIZE: usize = 60 * 1024; // 60KB — matches xdk/x86/ke.h
// i386 LARGE_COMMIT intentionally exceeds the C value (xdk/x86/ke.h sets it to
// KERNEL_STACK_SIZE = 12KB). The Rust port has deeper kernel call stacks than
// the C original (Rust frames + extra inlining), so 12KB GUI commit causes
// taskbar render to wedge in user-mode painting paths. 16KB matches what the
// previous (buggy-on-amd64) Rust constant happened to produce on i386 and
// what shipped working in test_smoke for i386 prior to the amd64 fix.
#[cfg(target_arch = "x86")]
pub const KERNEL_LARGE_STACK_COMMIT: usize = 16 * 1024; // 16KB — Rust-port margin

// arm64 is a 64-bit arch and must use amd64's stack sizing, NOT i386's. The
// original port copied i386's 12KB value into xdk/arm64/ke.h, but the Rust port
// has deeper kernel call stacks than the C original (see the note above), arm64
// has no dedicated exception stack (a guard-page overflow nest-faults into a
// hang, not a clean bugcheck), and PsConvertToGuiThread does NOT switch win32k
// GUI threads to a large stack on arm64 (ps/win32.rs) — so deep win32k/exit-path
// chains run on this base stack. 12KB intermittently overflows on the thread-exit
// teardown path (the historical arm64_lpcexitthread_race / fastfat 0x50). Match
// amd64: 48KB base / 72KB large.
#[cfg(target_arch = "aarch64")]
pub const KERNEL_STACK_SIZE: usize = 0xC000; // 48KB — matches xdk/amd64/ke.h sizing
#[cfg(target_arch = "aarch64")]
pub const KERNEL_LARGE_STACK_SIZE: usize = 0x12000; // 72KB — matches xdk/amd64/ke.h sizing
#[cfg(target_arch = "aarch64")]
pub const KERNEL_LARGE_STACK_COMMIT: usize = KERNEL_STACK_SIZE; // 48KB — matches xdk/amd64/ke.h sizing
pub const EXCEPTION_CHAIN_END: ULONG_PTR = ULONG_PTR::MAX;
pub const EVENT_MODIFY_STATE: ULONG = 0x0002;
pub const IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG: usize = 10;
pub const IMAGE_FILE_UP_SYSTEM_ONLY: ULONG = 0x4000;
pub const VER_PLATFORM_WIN32_NT: ULONG = 2;

pub const ViewShare: ULONG = 1;
pub const ViewUnmap: ULONG = 2;

pub const VadNone: ULONG = 0;

pub const _EVENT_TYPE_NotificationEvent: ULONG = 0;
pub const _KWAIT_REASON_WrVirtualMemory: ULONG = 11;

pub const MiExpansionPoolPagesInitialCharge: PFN_NUMBER = 32;

// ============================================================================
// Extern globals (template PTEs and MM state)
// ============================================================================

extern "C" {
    pub static ValidKernelPteLocal: MMPTE;
    pub static ValidKernelPdeLocal: MMPTE;
    pub static DemandZeroPte: MMPTE;
    pub static DemandZeroPde: MMPTE;
    pub static MmDecommittedPte: MMPTE;
    // ValidKernelPpe = ValidKernelPde on 64-bit
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    #[link_name = "ValidKernelPde"]
    pub static ValidKernelPpe: MMPTE;

    pub static mut MmSharedUserDataPte: PMMPTE;
    pub static mut MmSessionSpace: PVOID;
    pub static mut MiSessionSpaceWs: PVOID;
    pub static mut MiSessionViewStart: PVOID;
    pub static mut MmWorkingSetList: PVOID;
    pub static mut MmWorkingSetExpansionHead: LIST_ENTRY;
    pub static mut MmFreePagesByColor: [PMMCOLOR_TABLES; 2];
    pub static mut MmStandbyPageListByPriority: [MMPFNLIST; 8];
    pub static mut MmMinimumFreePages: PFN_NUMBER;
    pub static mut MmZeroingPageEvent: PVOID;
    pub static mut MiHighMemoryEvent: PVOID;
    pub static mut MiLowMemoryEvent: PVOID;
    pub static mut MmHighMemoryThreshold: PFN_NUMBER;
    pub static mut MmLowMemoryThreshold: PFN_NUMBER;
    pub static mut MmHeapSegmentReserve: SIZE_T;
    pub static mut MmHeapSegmentCommit: SIZE_T;
    pub static mut MmHeapDeCommitTotalFreeThreshold: SIZE_T;
    pub static mut MmHeapDeCommitFreeBlockThreshold: SIZE_T;
    pub static mut MmCriticalSectionTimeout: i64;
    pub static mut MmLargeStackSize: ULONG; // ULONG in mminit — a SIZE_T decl over-reads on 64-bit
    pub static mut MmMinimumStackCommitInBytes: SIZE_T;
    pub static mut MmMaximumDeadKernelStacks: ULONG;
    pub static mut MmDeadStackSListHead: [u8; 16];
    pub static mut MmHighSectionBase: PVOID;
    pub static mut MmProtectFreedNonPagedPool: BOOLEAN;
    pub static mut MmPagedPoolMutex: [u8; 64];
    pub static mut MmSectionBasedMutex: [u8; 64];
    pub static mut MmSectionCommitMutex: [u8; 64];
    pub static mut MmResidentAvailableAtInit: isize;
    pub static mut ExpAnsiCodePageDataOffset: ULONG;
    pub static mut ExpOemCodePageDataOffset: ULONG;
    pub static mut ExpUnicodeCaseTableDataOffset: ULONG;
    pub static mut ExpNlsSectionPointer: PVOID;
    pub static mut NtMajorVersion: ULONG;
    pub static mut NtMinorVersion: ULONG;
    pub static mut CmNtCSDVersion: ULONG;
    pub static mut PsDefaultSystemLocaleId: ULONG;
    pub static mut PsDefaultThreadLocaleId: ULONG;
    // MmSecondaryColorMask moved to ntkrnl-sys/mm.rs
}

// Registry-controlled values owned by ARM3/helpers.
#[no_mangle]
pub static HELPERS_CONTROL_VECTOR: ntkrnl_sys::SubVector<5> =
    ntkrnl_sys::SubVector(unsafe { [
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "LargeStackSize",
            core::ptr::addr_of_mut!(MmLargeStackSize)),
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "HighMemoryThreshold",
            core::ptr::addr_of_mut!(MmHighMemoryThreshold)),
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "ProtectNonPagedPool",
            core::ptr::addr_of_mut!(MmProtectFreedNonPagedPool)),
        ntkrnl_sys::cv!("Session Manager", "HeapSegmentReserve",
            core::ptr::addr_of_mut!(MmHeapSegmentReserve)),
        ntkrnl_sys::cv!("Session Manager", "HeapSegmentCommit",
            core::ptr::addr_of_mut!(MmHeapSegmentCommit)),
    ] });

// ============================================================================
// PFN_FROM_PPE / PFN_FROM_PXE (PFN_FROM_PTE and PFN_FROM_PDE are in mm.rs)
// ============================================================================

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn PFN_FROM_PPE(ppe: PMMPTE) -> PFN_NUMBER {
    ntkrnl_sys::mm::PFN_FROM_PTE(ppe)
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn PFN_FROM_PXE(pxe: PMMPTE) -> PFN_NUMBER {
    ntkrnl_sys::mm::PFN_FROM_PTE(pxe)
}

// ============================================================================
// MI_MAKE_HARDWARE_PTE — create a valid PTE with protection
// ============================================================================

#[inline]
pub unsafe fn MI_MAKE_HARDWARE_PTE(
    new_pte: &mut MMPTE,
    mapping_pte: PMMPTE,
    protection_mask: ULONG_PTR,
    page_frame_number: PFN_NUMBER,
) {
    new_pte.Long = MiDetermineUserGlobalPteMask(mapping_pte);
    new_pte.Long |= MmProtectToPteMask[protection_mask as usize];
    new_pte.set_page_frame_number(page_frame_number);
}

#[inline]
#[track_caller]
pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(
    new_pte: &mut MMPTE,
    _mapping_pte: PMMPTE,
    protection_mask: ULONG_PTR,
    page_frame_number: PFN_NUMBER,
) {
    new_pte.Long = 0;
    // The C original's three ASSERTs, restored: a mask granting no access must
    // never be made valid. See ntkrnl_sys::mm::mi_user_pte_may_be_valid.
    if !ntkrnl_sys::mm::mi_user_pte_may_be_valid(protection_mask as usize) {
        return;
    }
    new_pte.set_valid(true);
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    { new_pte.Long |= 4; }
    #[cfg(target_arch = "aarch64")]
    { new_pte.Long |= 1 << 6; }
    new_pte.set_page_frame_number(page_frame_number);
    new_pte.Long |= MmProtectToPteMask[protection_mask as usize];
}

#[inline]
pub fn MI_MAKE_TRANSITION_PTE(
    new_pte: &mut MMPTE,
    page: PFN_NUMBER,
    protection: ULONG,
) {
    new_pte.Long = 0;
    new_pte.set_transition(true);
    new_pte.set_soft_protection(protection);
    new_pte.set_page_frame_number(page);
}

/// Determine the base PTE mask (Valid + Accessed, plus Owner for user PTEs).
/// Delegates to the C shim in mm_rust_shims.c which calls the original
/// FORCEINLINE from miarm.h — ensures correct bit positions per architecture.
#[inline]
pub unsafe fn MiDetermineUserGlobalPteMask(mapping_pte: PMMPTE) -> ULONG_PTR {
    extern "C" {
        #[link_name = "MiDetermineUserGlobalPteMask"]
        fn c_impl(mapping_pte: PMMPTE) -> ULONG_PTR;
    }
    c_impl(mapping_pte)
}

// ============================================================================
// MI_IS_PAGE_WRITEABLE / COPY_ON_WRITE / EXECUTABLE / LARGE
// ============================================================================

#[inline]
pub fn MI_IS_PAGE_WRITEABLE(pte: *const MMPTE) -> BOOLEAN {
    unsafe {
        let long = (*pte).Long;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        { if (long & (1 << 1)) != 0 { TRUE } else { FALSE } }
        // arm64 port convention (ke/arm64/stubs.rs PTE map): bit 55 = software Writable
        // (PTE_READWRITE = 0x00C0... = bits 55+54). Bit 7 is NotDirty/AP[2] — 1 means
        // hardware READ-ONLY — so the old `bit 7 set → writeable` test was inverted.
        #[cfg(target_arch = "aarch64")]
        { if (long & (1u64 << 55) as ULONG_PTR) != 0 { TRUE } else { FALSE } }
    }
}

#[inline]
pub fn MI_IS_PAGE_COPY_ON_WRITE(pte: *const MMPTE) -> BOOLEAN {
    unsafe {
        let long = (*pte).Long;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        { if (long & (1 << 9)) != 0 { TRUE } else { FALSE } }
        // arm64 port convention: bit 56 = software CopyOnWrite (PTE_WRITECOPY =
        // 0x0140... = bits 56+54(+7)). Bit 8 is SH[0] (shareability) — wrong field.
        #[cfg(target_arch = "aarch64")]
        { if (long & (1u64 << 56) as ULONG_PTR) != 0 { TRUE } else { FALSE } }
    }
}

#[inline]
pub fn MI_IS_PAGE_EXECUTABLE(pte: *const MMPTE) -> BOOLEAN {
    unsafe {
        let long = (*pte).Long;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        { if (long & (1usize.wrapping_shl(63))) == 0 { TRUE } else { FALSE } }
        #[cfg(target_arch = "aarch64")]
        { if (long & (1usize << 54)) == 0 { TRUE } else { FALSE } }
    }
}

#[inline]
pub fn MI_IS_PAGE_LARGE(pte: *const MMPTE) -> BOOLEAN {
    unsafe {
        let long = (*pte).Long;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        { if (long & (1 << 7)) != 0 { TRUE } else { FALSE } }
        #[cfg(target_arch = "aarch64")]
        { if (long & (1 << 1)) == 0 { TRUE } else { FALSE } }
    }
}

// ============================================================================
// MI_MAKE_DIRTY_PAGE / CLEAN / ACCESSED / WRITE
// ============================================================================

#[inline]
pub fn MI_MAKE_DIRTY_PAGE(pte: *mut MMPTE) {
    unsafe {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        { (*pte).Long |= 1 << 6; }
        #[cfg(target_arch = "aarch64")]
        { (*pte).Long &= !(1 << 7); }
    }
}

#[inline]
pub fn MI_MAKE_CLEAN_PAGE(pte: *mut MMPTE) {
    unsafe {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        { (*pte).Long &= !(1 << 6); }
        #[cfg(target_arch = "aarch64")]
        { (*pte).Long = ((*pte).Long | 1 << 7) & !(PTE_SOFTWARE_DIRTY as ULONG_PTR); }
    }
}

/// arm64: a sticky software dirty bit (HARDWARE_PTE.OsAvailable, bit 57, which
/// the MMU ignores). arm64 has no hardware dirty bit: "dirty" is NotDirty (AP[2],
/// bit 7) clear, i.e. hardware-WRITABLE, so a page made read-only after it was
/// written reads clean. x86 keeps its Dirty bit across a protection change.
/// RosMm's `MmSetPageProtect` (arm64/page.rs) sets this when it reprotects a
/// dirty page, and every dirty test (`MI_PTE_IS_DIRTY`) reads it. Without it the
/// loader's bound IAT -- written, then made read-only -- was paged out as
/// clean and read back from the file under memory pressure: native processes
/// called through unbound slots (IMAGE_IMPORT_BY_NAME RVAs, PC 0xAA462) and
/// csrss's death bugchecked 0xF4.
#[cfg(target_arch = "aarch64")]
pub const PTE_SOFTWARE_DIRTY: u64 = 1 << 57;

/// Was the page this valid PTE maps written since it was last cleaned?
#[inline]
pub fn MI_PTE_IS_DIRTY(long: u64) -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    { (long & (1 << 6)) != 0 }
    #[cfg(target_arch = "aarch64")]
    { (long & (1 << 7)) == 0 || (long & PTE_SOFTWARE_DIRTY) != 0 }
}

#[inline]
pub fn MI_MAKE_ACCESSED_PAGE(pte: *mut MMPTE) {
    unsafe {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        { (*pte).Long |= 1 << 5; }
        #[cfg(target_arch = "aarch64")]
        { (*pte).Long |= 1 << 10; }
    }
}

#[inline]
pub fn MI_MAKE_WRITE_PAGE(pte: *mut MMPTE) {
    unsafe {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        { (*pte).Long |= 1 << 1; }
        #[cfg(target_arch = "aarch64")]
        { (*pte).Long |= 1 << 7; }
    }
}

#[inline]
pub fn mi_clear_copy_on_write(pte: *mut MMPTE) {
    unsafe {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        { (*pte).Long &= !(1 << 9); }
        #[cfg(target_arch = "aarch64")]
        { (*pte).Long &= !(1 << 8); }
    }
}

#[inline]
pub fn mi_make_executable_page(pte: *mut MMPTE) {
    unsafe {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        { (*pte).Long &= !(1usize.wrapping_shl(63)); }
        #[cfg(target_arch = "aarch64")]
        { (*pte).Long &= !(1usize << 54); }
    }
}

// ============================================================================
// MI_IS_WRITE_ACCESS / MI_ASSERT_PFN_LOCK_HELD
// ============================================================================

#[inline]
pub fn MI_IS_WRITE_ACCESS(fault_code: ULONG_PTR) -> bool {
    (fault_code & 0x00000002) != 0
}

#[inline]
pub fn MI_ASSERT_PFN_LOCK_HELD() {}

// MI_IS_NOT_PRESENT_FAULT and MI_IS_INSTRUCTION_FETCH are in ntkrnl_sys::mm

#[inline]
pub fn mmpfn_increment_reference_count(pfn: PMMPFN) {
    unsafe {
        let count = (*pfn).reference_count();
        (*pfn).set_reference_count(count.wrapping_add(1));
    }
}

// ============================================================================
// MI_SET_PROCESS / MI_IS_ROS_PFN / MI_IS_MEMORY_AREA_VAD
// ============================================================================

#[inline]
pub fn MI_SET_PROCESS(_process: PVOID) {}

#[inline]
pub fn MI_SET_PROCESS2(_name: PVOID) {}

#[inline]
pub unsafe fn MI_IS_ROS_PFN(pfn: PMMPFN) -> bool {
    (*pfn).awe_allocation()
}

#[inline]
pub unsafe fn MI_IS_MEMORY_AREA_VAD(vad: PVOID) -> bool {
    let vad = vad as *const ntkrnl_sys::ddk::_MMVAD;
    ((*vad).u.VadFlags.Spare() & 1) != 0
}

// ============================================================================
// MI_GET_NEXT_PROCESS_COLOR
// ============================================================================

#[inline]
pub unsafe fn MI_GET_NEXT_PROCESS_COLOR(process: PVOID) -> ULONG {
    extern "C" {
        static mut MmSystemPageColor: ULONG;
    }
    MmSystemPageColor = MmSystemPageColor.wrapping_add(1);
    MmSystemPageColor & MmSecondaryColorMask
}

// ============================================================================
// Address range checks
// ============================================================================

#[inline]
pub unsafe fn MI_IS_SESSION_IMAGE_ADDRESS(address: PVOID) -> bool {
    let addr = address as ULONG_PTR;
    addr >= MiSessionImageStart as ULONG_PTR && addr < MiSessionImageEnd as ULONG_PTR
}

#[inline]
pub unsafe fn MI_IS_SYSTEM_PAGE_TABLE_ADDRESS(address: PVOID) -> bool {
    let addr = address as ULONG_PTR;
    let start = MiAddressToPte(MmSystemRangeStart) as ULONG_PTR;
    addr >= start && addr <= PTE_TOP
}

#[inline]
pub fn MI_IS_PAGE_TABLE_OR_HYPER_ADDRESS(address: PVOID) -> bool {
    let addr = address as ULONG_PTR;
    addr >= PTE_BASE && addr <= HYPER_SPACE_END
}

// ============================================================================
// MI_WRITE_INVALID_PDE / MI_WRITE_VALID_PPE
// ============================================================================

#[inline]
pub unsafe fn MI_WRITE_INVALID_PDE(pointer_pde: PMMPTE, invalid_pde: MMPTE) {
    *pointer_pde = invalid_pde;
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub unsafe fn MI_WRITE_VALID_PPE(pointer_ppe: PMMPTE, valid_ppe: MMPTE) {
    MI_WRITE_VALID_PTE(pointer_ppe, valid_ppe);
}

// ============================================================================
// Additional PTE helpers
// ============================================================================

#[inline]
pub fn pointer_pte_pte(_pte: &MMPTE) -> ULONG_PTR { 0 }

// ============================================================================
// MiProtoPteToPte / MiSynchronizeSystemPde
// ============================================================================

/// Decodes a prototype PTE into the underlying PTE pointer.
/// On amd64/arm64: arithmetic right-shift by 16 sign-extends the 48-bit address.
/// On i386: offset from MmPagedPoolStart using ProtoAddressHigh/ProtoAddressLow fields.
#[inline]
pub unsafe fn MiProtoPteToPte(proto_pte: PMMPTE) -> PMMPTE {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        // C: (PMMPTE)(((LONG64)(x)->u.Long) >> 16)
        (((*proto_pte).Long as i64) >> 16) as PMMPTE
    }
    #[cfg(target_arch = "x86")]
    {
        use crate::ARM3::mminit::MmPagedPoolStart;
        // C: (PMMPTE)((ULONG_PTR)MmPagedPoolStart +
        //     (((x)->u.Proto.ProtoAddressHigh << 9) | (x)->u.Proto.ProtoAddressLow << 2))
        // i386 MMPTE_PROTOTYPE bitfield layout:
        //   Valid:1 [0], ProtoAddressLow:7 [7:1], ReadOnly:1 [8],
        //   WhichPool:1 [9], Prototype:1 [10], ProtoAddressHigh:21 [31:11]
        let long_val = (*proto_pte).Long as u32;
        let proto_addr_low = (long_val >> 1) & 0x7F;   // bits [7:1]
        let proto_addr_high = (long_val >> 11) & 0x1FFFFF; // bits [31:11]
        let offset = (proto_addr_high << 9) | (proto_addr_low << 2);
        (MmPagedPoolStart as ULONG_PTR + offset as ULONG_PTR) as PMMPTE
    }
}

/// Builds a Prototype PTE that encodes the address of a prototype PTE pointer.
/// On amd64/arm64: stores `PointerPte << 16` with Prototype bit 10 set.
/// On i386: stores offset from MmPagedPoolStart in ProtoAddressLow/High fields.
#[inline]
pub unsafe fn MI_MAKE_PROTOTYPE_PTE(new_pte: PMMPTE, pointer_pte: PMMPTE) {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        // C: NewPte->u.Long = (ULONG64)PointerPte << 16;
        //    NewPte->u.Proto.Prototype = 1;  (bit 10)
        (*new_pte).Long = ((pointer_pte as u64) << 16) as ULONG_PTR;
        (*new_pte).Long |= (1 << 10); // Prototype bit
    }
    #[cfg(target_arch = "x86")]
    {
        use crate::ARM3::mminit::MmPagedPoolStart;
        // C: Offset = (ULONG_PTR)PointerPte - (ULONG_PTR)MmPagedPoolStart;
        //    NewPte->u.Proto.Prototype = 1;  (bit 10)
        //    NewPte->u.Proto.ProtoAddressLow = (Offset & 0x1FC) >> 2;   (7 bits → bits [7:1])
        //    NewPte->u.Proto.ProtoAddressHigh = (Offset & 0x3FFFFE00) >> 9;  (21 bits → bits [31:11])
        let offset = pointer_pte as u32 - MmPagedPoolStart as u32;
        let proto_addr_low = (offset & 0x1FC) >> 2;    // 7 bits
        let proto_addr_high = (offset & 0x3FFFFE00) >> 9; // 21 bits
        (*new_pte).Long = 0;
        (*new_pte).Long |= (1 << 10) as ULONG_PTR;     // Prototype bit
        (*new_pte).Long |= (proto_addr_low << 1) as ULONG_PTR;  // bits [7:1]
        (*new_pte).Long |= (proto_addr_high << 11) as ULONG_PTR; // bits [31:11]
    }
}

#[inline]
pub unsafe fn MiSynchronizeSystemPde(_pde: PMMPTE) {}

// ============================================================================
// AFFINITY_MASK
// ============================================================================

#[inline]
// MiIsPteOnPdeBoundary, MiIsPteOnPpeBoundary, MiIsPteOnPxeBoundary are in ntkrnl_sys::mm

pub fn AFFINITY_MASK(processor: ULONG) -> ULONG_PTR {
    1usize << processor
}

// ============================================================================
// Interlocked helpers
// ============================================================================

#[inline]
pub unsafe fn InterlockedIncrement16(addend: *mut i16) -> i16 {
    let atomic = &*(addend as *const core::sync::atomic::AtomicI16);
    atomic.fetch_add(1, core::sync::atomic::Ordering::SeqCst).wrapping_add(1)
}

// ============================================================================
// Misc function stubs
// ============================================================================

#[inline]
pub unsafe fn KeRaiseIrql(new_irql: KIRQL, old_irql: *mut KIRQL) {
    // On amd64: C shim provides cdecl KfRaiseIrql (FORCEINLINE in SDK header).
    // On ARM64: HAL exports cdecl KfRaiseIrql.
    // On i386: HAL exports __fastcall KfRaiseIrql — declare as `extern "fastcall"`
    //   (Rust supports it on i686-pc-windows-gnu).
    #[cfg(not(target_arch = "x86"))]
    {
        extern "C" { fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL; }
        *old_irql = KfRaiseIrql(new_irql);
    }
    #[cfg(target_arch = "x86")]
    {
        extern "fastcall" { fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL; }
        *old_irql = KfRaiseIrql(new_irql);
    }
}

#[inline]
pub unsafe fn KeLowerIrql(new_irql: KIRQL) {
    #[cfg(not(target_arch = "x86"))]
    {
        extern "C" { fn KfLowerIrql(NewIrql: KIRQL); }
        KfLowerIrql(new_irql);
    }
    #[cfg(target_arch = "x86")]
    {
        extern "fastcall" { fn KfLowerIrql(NewIrql: KIRQL); }
        KfLowerIrql(new_irql);
    }
}

#[inline]
pub unsafe fn KeQueryTickCount(tick_count: *mut i64) {
    *tick_count = 0;
}

#[inline]
pub fn NtCurrentProcess() -> PVOID {
    -1isize as PVOID
}

#[inline]
pub fn PAGE_ROUND_UP(addr: ULONG_PTR) -> ULONG_PTR {
    (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

// ============================================================================
// Extern functions needed by Tier 3a files
// ============================================================================

// FORCEINLINE shims — must be extern "C" (cdecl) to match C shim calling convention.
// On arm64/amd64, extern "C" == extern "system". On i386, extern "system" = stdcall which
// would produce @N-decorated symbols that don't match the cdecl C shims.
extern "C" {
    pub fn MmLockAddressSpace(AddressSpace: PVOID);
    pub fn MmUnlockAddressSpace(AddressSpace: PVOID);
}

extern "system" {
    pub fn MiDecrementReferenceCount(Pfn: PMMPFN, PageFrameIndex: PFN_NUMBER);
    pub fn MiInitializePfn(PageFrameIndex: PFN_NUMBER, PointerPte: PMMPTE, Modified: BOOLEAN);
    pub fn MiReleaseProcessReferenceToSessionDataPage(SessionGlobal: PVOID);
    pub fn PsChargeProcessNonPagedPoolQuota(Process: PVOID, Amount: SIZE_T) -> NTSTATUS;
    pub fn PsReturnProcessNonPagedPoolQuota(Process: PVOID, Amount: SIZE_T);
    pub fn RtlImageNtHeader(Base: PVOID) -> PVOID;
    pub fn RtlImageDirectoryEntryToData(
        BaseAddress: PVOID, MappedAsImage: BOOLEAN, Directory: USHORT, Size: *mut ULONG,
    ) -> PVOID;
    pub fn RtlRandom(Seed: *mut ULONG) -> ULONG;
    pub fn RtlSetBits(BitMapHeader: PVOID, StartingIndex: ULONG, NumberToSet: ULONG);
    pub fn RtlSetDaclSecurityDescriptor(
        SecurityDescriptor: PVOID, DaclPresent: BOOLEAN, Dacl: PVOID, DaclDefaulted: BOOLEAN,
    ) -> NTSTATUS;
    pub fn SeInitializeProcessAuditName(
        FileObject: PVOID, DoAudit: BOOLEAN, AuditInfo: *mut PVOID,
    ) -> NTSTATUS;
}
extern "system" {
    // NTAPI (stdcall on i386) per ntoskrnl.spec — extern "system" resolves correctly per arch
    pub fn PsGetCurrentThread() -> PVOID;
}

// ============================================================================
// FORCEINLINE gap functions (miarm.h) — no C symbol, must be implemented in Rust
// ============================================================================

use ntkrnl_sys::ddk::{_MMPFN, _MMPTE, PEPROCESS, MiDeletePte, MiIsUserPde};
use ntkrnl_sys::interlocked::InterlockedDecrementSizeT;

extern "system" {
    static mut MmSystemLockPagesCount: isize;
}

/// InterlockedCompareExchange16 — atomic CAS on a 16-bit value.
/// Returns the old value.
#[inline]
unsafe fn InterlockedCompareExchange16(destination: *mut i16, exchange: i16, comparand: i16) -> i16 {
    use core::sync::atomic::{AtomicI16, Ordering};
    let atomic = &*(destination as *const AtomicI16);
    match atomic.compare_exchange(comparand, exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) => old,
        Err(old) => old,
    }
}

/// MiDereferencePfnAndDropLockCount — drops a locked page reference.
/// Translated from FORCEINLINE in miarm.h.
#[no_mangle]
pub unsafe extern "C" fn MiDereferencePfnAndDropLockCount(Pfn1: *mut _MMPFN) {
    let Pfn1 = opaque_ptr(Pfn1);
    let pfn = Pfn1 as PMMPFN;
    let mut OldRefCount: USHORT;
    let mut RefCount: USHORT;

    loop {
        OldRefCount = (*pfn).reference_count();
        ASSERT!(OldRefCount != 0);

        if OldRefCount == 1 {
            ASSERT!((*pfn).reference_count() == 1);
            ASSERT!((*pfn).page_location() != ActiveAndValid);
            ASSERT!((*pfn).share_count() == 0);

            // Prototype PTE check — FIXME: should return commit
            // (matches C: just a DPRINT1, no action)

            InterlockedDecrementSizeT(core::ptr::addr_of_mut!(MmSystemLockPagesCount));
            let PageFrameIndex = MiGetPfnEntryIndex(pfn);
            MiDecrementReferenceCount(pfn, PageFrameIndex);
            return;
        }

        // Atomic CAS on the reference count (at offset U3, first 2 bytes).
        // Derived from bindgen (offset_check-asserted vs C) — never a numeral.
        const U3_OFFSET: usize = core::mem::offset_of!(ntkrnl_sys::ddk::_MMPFN, u3);
        let refcount_ptr = (pfn as *mut u8).add(U3_OFFSET) as *mut i16;
        RefCount = InterlockedCompareExchange16(
            refcount_ptr,
            (OldRefCount - 1) as i16,
            OldRefCount as i16,
        ) as USHORT;
        ASSERT!(RefCount != 0);

        if OldRefCount == RefCount {
            break;
        }
    }

    ASSERT!(RefCount > 1);
    if RefCount == 2 {
        if (*pfn).share_count() >= 1 {
            ASSERT!((*pfn).page_location() == ActiveAndValid);
            // Prototype PTE check — ASSERT(FALSE) in C, skip for now
            InterlockedDecrementSizeT(core::ptr::addr_of_mut!(MmSystemLockPagesCount));
        }
    }
}

/// MiDecrementPageTableReferences — decrements page table refcount.
/// i386 (2-level): uses MmWorkingSetList->UsedPageTableEntries[]
/// 64-bit (3+level): uses PFN entry UsedPageTableEntries field
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn MiDecrementPageTableReferences(Address: PVOID) -> USHORT {
    let Address = opaque_ptr(Address);
    // On i386: MmWorkingSetList->UsedPageTableEntries[MiGetPdeOffset(Address)]
    // MiGetPdeOffset(Va) = (ULONG_PTR)Va / PDE_MAPPED_VA
    // PDE_MAPPED_VA = 4MB = 0x400000
    const PDE_MAPPED_VA: ULONG_PTR = 0x400000; // nolint: const-contradiction (i386 non-PAE: internal/i386/mm.h:172 PTE_PER_PAGE(1024)*PAGE_SIZE; the 0x200000 island value is amd64's)
    // Offset of UsedPageTableEntries in MMWSL (i386):
    // 4*ULONG + PVOID + 2*ULONG + PVOID + ULONG + ULONG + 2*PVOID + 2*ULONG
    // = 16 + 4 + 8 + 4 + 4 + 4 + 8 + 8 = 56 bytes
    const MMWSL_USED_PTE_OFFSET: usize = 56;
    let pde_offset = (Address as ULONG_PTR) / PDE_MAPPED_VA;
    let wsl_ptr = MmWorkingSetList as *mut u8;
    let ref_count_ptr = wsl_ptr.add(MMWSL_USED_PTE_OFFSET) as *mut USHORT;
    let entry = ref_count_ptr.add(pde_offset);
    *entry -= 1;
    ASSERT!((*entry as ULONG) < PTE_PER_PAGE as ULONG);
    *entry
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MiDecrementPageTableReferences(Address: PVOID) -> USHORT {
    let Address = opaque_ptr(Address);
    let PointerPde = MiAddressToPde(Address);
    let Pfn = MiGetPfnEntry(PFN_FROM_PTE(PointerPde));
    ASSERT!(!Pfn.is_null());
    let val = mmpfn_used_page_table_entries(Pfn);
    ASSERT!(val != 0);
    let new_val = val - 1;
    mmpfn_set_used_page_table_entries(Pfn, new_val);
    ASSERT!(new_val < PTE_PER_PAGE as ULONG);
    new_val as USHORT
}

/// MiDeletePde — deletes a PDE and cascades to higher-level page tables on 64-bit.
/// Translated from FORCEINLINE in miarm.h.
#[no_mangle]
pub unsafe extern "C" fn MiDeletePde(PointerPde: PMMPDE, CurrentProcess: PEPROCESS) {
    let CurrentProcess = opaque_ptr(CurrentProcess);
    ASSERT!(MiIsUserPde(PointerPde as PVOID) != 0);

    // Kill this one as a PTE
    MiDeletePte(
        PointerPde as *mut _MMPTE,
        MiPdeToPte(PointerPde) as PVOID,
        CurrentProcess,
        core::ptr::null_mut(),
    );

    // On 3+ level paging, cascade down
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        if MiDecrementPageTableReferences(MiPdeToPte(PointerPde) as PVOID) == 0 {
            MiDeletePte(
                MiPdeToPpe(PointerPde) as *mut _MMPTE,
                PointerPde as PVOID,
                CurrentProcess,
                core::ptr::null_mut(),
            );
            // On 4-level paging (amd64 + arm64), cascade further
            if MiDecrementPageTableReferences(PointerPde as PVOID) == 0 {
                MiDeletePte(
                    MiPdeToPxe(PointerPde) as *mut _MMPTE,
                    MiPdeToPpe(PointerPde) as PVOID,
                    CurrentProcess,
                    core::ptr::null_mut(),
                );
            }
        }
    }
}

// ============================================================================
// MDL helper functions (C macros, not in ntkrnl-sys)
// ============================================================================

use ntkrnl_sys::ddk::{_MDL, PMDL};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

/// MmGetMdlByteCount — returns the byte count from an MDL.
#[inline]
pub unsafe fn MmGetMdlByteCount(mdl: PMDL) -> ULONG {
    (*mdl).ByteCount
}

/// MmGetMdlByteOffset — returns the byte offset from an MDL.
#[inline]
pub unsafe fn MmGetMdlByteOffset(mdl: PMDL) -> ULONG {
    (*mdl).ByteOffset
}

/// MmGetMdlVirtualAddress — returns the virtual address from an MDL.
#[inline]
pub unsafe fn MmGetMdlVirtualAddress(mdl: PMDL) -> PVOID {
    ((*mdl).StartVa as ULONG_PTR + (*mdl).ByteOffset as ULONG_PTR) as PVOID
}

/// MmGetMdlPfnArray — returns pointer to the PFN array following the MDL header.
#[inline]
pub unsafe fn MmGetMdlPfnArray(mdl: PMDL) -> *mut PFN_NUMBER {
    (mdl as *mut u8).add(core::mem::size_of::<_MDL>()) as *mut PFN_NUMBER
}

/// MmInitializeMdl — initializes an MDL.
#[inline]
pub unsafe fn MmInitializeMdl(mdl: PMDL, base_va: PVOID, length: SIZE_T) {
    (*mdl).Next = core::ptr::null_mut();
    (*mdl).Size = (core::mem::size_of::<_MDL>()
        + core::mem::size_of::<PFN_NUMBER>()
            * ADDRESS_AND_SIZE_TO_SPAN_PAGES(base_va, length)) as i16;
    (*mdl).MdlFlags = 0;
    (*mdl).StartVa = (base_va as ULONG_PTR & !(PAGE_SIZE - 1)) as PVOID;
    (*mdl).ByteOffset = (base_va as ULONG_PTR & (PAGE_SIZE - 1)) as ULONG;
    (*mdl).ByteCount = length as ULONG;
}

/// ProbeForWriteChar — probes for write access on a single byte.
#[inline]
pub unsafe fn ProbeForWriteChar(addr: *mut u8) {
    let _val = core::ptr::read_volatile(addr);
    core::ptr::write_volatile(addr, _val);
}

// ============================================================================
// MDL / IO / VAD constants (C #defines, not in ntkrnl-sys)
// ============================================================================

pub const IoReadAccess: ULONG = 0;
pub const IoWriteAccess: ULONG = 1;
pub const IoModifyAccess: ULONG = 2;

pub const HighPagePriority: ULONG = 2;

pub const TableFoundNode: ULONG = 1;

pub const VadDevicePhysicalMemory: ULONG = 0x01;

// PrototypePte — soft PTE template used in MmArmAccessFault when constructing
// a prototype PTE for an address whose VAD has FirstPrototypePte set
// (e.g. the user-mode mirror of KUSER_SHARED_DATA at 0x7FFE0000).
//
// Layout mirrors the per-arch C globals in ntoskrnl/mm/{i386,amd64}/init.c:
//     PrototypePte = (Protection << SOFT_PROTECTION_SHIFT)
//                  | PTE_PROTOTYPE
//                  | (MI_PTE_LOOKUP_NEEDED << SoftPageFileHighShift)
// The protection bits are overwritten at use time via set_soft_protection,
// so we only need PTE_PROTOTYPE (bit 10) and PageFileHigh = MI_PTE_LOOKUP_NEEDED
// (which lives at bits [63:32] on 64-bit, [31:12] on 32-bit). Bit 0 must NOT
// be set — that's the hardware Valid bit; this is an INVALID (software) PTE.
//
// NOTE: was incorrectly defined as `1` for a long time, which on every arch
// produced a soft PTE with bit 0 (Valid) set and PTE_PROTOTYPE clear. amd64
// happened to dispatch it through MiDispatchFault before re-faulting, but on
// arm64 the next access re-faulted on the same address forever, hanging smss
// during ntdll::LdrpInit's first read of SharedUserData->ProcessorFeatures.
#[cfg(target_pointer_width = "64")]
pub const PrototypePte: ULONG_PTR =
    /* PTE_PROTOTYPE */ 0x400
    | /* MI_PTE_LOOKUP_NEEDED << 32 */ 0xFFFFFFFF_00000000;
#[cfg(target_pointer_width = "32")]
pub const PrototypePte: ULONG_PTR =
    /* PTE_PROTOTYPE */ 0x400
    | /* MI_PTE_LOOKUP_NEEDED << 12 */ 0xFFFFF000;

// ============================================================================
// Security / Event / ACL constants for mminit.rs
// NOTE: SECURITY_DESCRIPTOR_REVISION, READ_CONTROL, SYNCHRONIZE, ACL_REVISION,
// TAG_DACL are already in ntkrnl_sys::constants — do NOT redefine here.
// ============================================================================

pub const EVENT_ALL_ACCESS: ULONG = 0x001F0003;
pub const EVENT_QUERY_STATE: ULONG = 0x0001;

// ============================================================================
// Loader block / memory descriptor types for mminit.rs
// ============================================================================

// LoaderMemoryType enum values (from arc.h)
pub const LoaderExceptionBlock: ULONG = 0;
pub const LoaderSystemBlock: ULONG = 1;
pub const LoaderFree: ULONG = 2;
pub const LoaderBad: ULONG = 3;
pub const LoaderLoadedProgram: ULONG = 4;
pub const LoaderFirmwareTemporary: ULONG = 5;
pub const LoaderFirmwarePermanent: ULONG = 6;
pub const LoaderOsloaderHeap: ULONG = 7;
pub const LoaderOsloaderStack: ULONG = 8;
pub const LoaderSystemCode: ULONG = 9;
pub const LoaderHalCode: ULONG = 10;
pub const LoaderBootDriver: ULONG = 11;
pub const LoaderConsoleInDriver: ULONG = 12;
pub const LoaderConsoleOutDriver: ULONG = 13;
pub const LoaderStartupDpcStack: ULONG = 14;
pub const LoaderStartupKernelStack: ULONG = 15;
pub const LoaderStartupPanicStack: ULONG = 16;
pub const LoaderStartupPcrPage: ULONG = 17;
pub const LoaderStartupPdrPage: ULONG = 18;
pub const LoaderRegistryData: ULONG = 19;
pub const LoaderMemoryData: ULONG = 20;
pub const LoaderNlsData: ULONG = 21;
pub const LoaderSpecialMemory: ULONG = 22;
pub const LoaderBBTMemory: ULONG = 23;
pub const LoaderReserve: ULONG = 24;
pub const LoaderXIPRom: ULONG = 25;
pub const LoaderHALCachedMemory: ULONG = 26;
pub const LoaderLargePageFiller: ULONG = 27;
pub const LoaderErrorLogMemory: ULONG = 28;
pub const LoaderMaximum: ULONG = 29;

// NT Product types
pub const NtProductWinNt: ULONG = 1;
pub const NtProductLanManNt: ULONG = 2;
pub const NtProductServer: ULONG = 3;

// NOTE: KeLoaderBlock, SeLocalSystemSid, SeAliasAdminsSid, SeWorldSid, SharedUserData
// are already in ntkrnl_sys::globals / ntkrnl_sys::macros — do NOT redefine here.
extern "C" {
    pub static mut ExEventObjectType: PVOID;
    pub static mut MmSystemPtesStart: [PVOID; 1];
    pub static mut MmSystemPtesEnd: [PVOID; 1];
}

// InitializePool — extern function
extern "system" {
    pub fn InitializePool(PoolType: ULONG, Threshold: ULONG);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn MiInsertStandbyListAtFront(PageFrameIndex: PFN_NUMBER);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn MiInsertStandbyListAtFront(PageFrameIndex: PFN_NUMBER);
}

// ============================================================================
// MMPFN OriginalPte accessor helpers
// (OriginalPte overlays the __bindgen_anon_1 field in bindgen MMPFN)
// ============================================================================

// OriginalPte offset in MMPFN:
// OriginalPte union ({OriginalPte|AweReferenceCount|RmapListHead}, bindgen
// __bindgen_anon_1). Derived from bindgen (offset_check-asserted vs C).
const MMPFN_ORIGINAL_PTE_OFFSET: usize =
    core::mem::offset_of!(ntkrnl_sys::ddk::_MMPFN, __bindgen_anon_1);

// Note: MMPFN has a dedicated ULONG UsedPageTableEntries at offset 28 on _WIN64,
// but ReactOS C code (miarm.h) uses OriginalPte.u.Soft.UsedPageTableEntries
// (bits 12-21 of OriginalPte at offset 32) instead. Rust accessors match C.

/// Get OriginalPte.Long from an MMPFN
#[inline]
pub unsafe fn mmpfn_original_pte_long(pfn: PMMPFN) -> ULONG_PTR {
    let p = (pfn as *const u8).add(MMPFN_ORIGINAL_PTE_OFFSET);
    *(p as *const ULONG_PTR)
}

/// Set OriginalPte.Long on an MMPFN
#[inline]
pub unsafe fn mmpfn_set_original_pte_long(pfn: PMMPFN, val: ULONG_PTR) {
    let p = (pfn as *mut u8).add(MMPFN_ORIGINAL_PTE_OFFSET);
    *(p as *mut ULONG_PTR) = val;
}

/// Get pointer to OriginalPte in MMPFN (as PMMPTE)
#[inline]
pub unsafe fn mmpfn_original_pte_ptr(pfn: PMMPFN) -> PMMPTE {
    (pfn as *mut u8).add(MMPFN_ORIGINAL_PTE_OFFSET) as PMMPTE
}

/// Get OriginalPte as MMPTE value
#[inline]
pub unsafe fn mmpfn_original_pte(pfn: PMMPFN) -> MMPTE {
    let p = (pfn as *const u8).add(MMPFN_ORIGINAL_PTE_OFFSET);
    *(p as *const MMPTE)
}

/// Set OriginalPte as MMPTE value
#[inline]
pub unsafe fn mmpfn_set_original_pte(pfn: PMMPFN, val: MMPTE) {
    let p = (pfn as *mut u8).add(MMPFN_ORIGINAL_PTE_OFFSET);
    *(p as *mut MMPTE) = val;
}

// MMPFNENTRY bitfield positions (matching mm.rs MMPFNENTRY constants)
// Bit 0: Modified
// Bit 1: ReadInProgress (= StartOfAllocation)
// Bit 2: WriteInProgress (= EndOfAllocation)
// Bit 3: PrototypePte
// Bit 4-7: PageColor
// Bit 8-10: PageLocation
// Bit 11: RemovalRequested (bit 5 from miarm.h) -> actually shifted
// Bit 12-13: CacheAttribute

/// Get u4.InPageError.
#[inline]
pub unsafe fn mmpfn_in_page_error_u4(pfn: PMMPFN) -> bool {
    (*pfn).in_page_error()
}

/// Set u4.InPageError
#[inline]
pub unsafe fn mmpfn_set_in_page_error_u4(pfn: PMMPFN, val: bool) {
    (*pfn).set_in_page_error(val);
}

/// Get page color from MMPFNENTRY (bits 4-7 of the entry u16)
#[inline]
pub unsafe fn mmpfn_page_color(pfn: PMMPFN) -> ULONG {
    let entry = mmpfn_mmpfnentry_raw(pfn);
    ((entry >> 4) & 0xF) as ULONG
}

/// Set page color in MMPFNENTRY
#[inline]
pub unsafe fn mmpfn_set_page_color(pfn: PMMPFN, color: ULONG) {
    let mut entry = mmpfn_mmpfnentry_raw(pfn);
    entry &= !(0xF << 4);
    entry |= ((color as u16) & 0xF) << 4;
    mmpfn_set_mmpfnentry_raw(pfn, entry);
}

/// Get raw MMPFNENTRY u16 (at u3 + 2). Derived from bindgen (offset_check-
/// asserted vs C).
const MMPFN_U3_OFFSET: usize = core::mem::offset_of!(ntkrnl_sys::ddk::_MMPFN, u3);

#[inline]
pub unsafe fn mmpfn_mmpfnentry_raw(pfn: PMMPFN) -> u16 {
    let p = (pfn as *const u8).add(MMPFN_U3_OFFSET + 2);
    *(p as *const u16)
}

/// Set raw MMPFNENTRY u16
#[inline]
pub unsafe fn mmpfn_set_mmpfnentry_raw(pfn: PMMPFN, val: u16) {
    let p = (pfn as *mut u8).add(MMPFN_U3_OFFSET + 2);
    *(p as *mut u16) = val;
}

/// Get u4.Priority.
#[inline]
pub unsafe fn mmpfn_priority(pfn: PMMPFN) -> ULONG {
    (*pfn).priority() as ULONG
}

/// Set u4.Priority.
#[inline]
pub unsafe fn mmpfn_set_priority(pfn: PMMPFN, val: ULONG) {
    (*pfn).set_priority(val as u16);
}

/// Get RomPte flag from MMPFNENTRY
#[inline]
pub unsafe fn mmpfn_rom_pte(pfn: PMMPFN) -> bool {
    (*pfn).rom()
}

/// Set RomPte flag
#[inline]
pub unsafe fn mmpfn_set_rom_pte(pfn: PMMPFN, val: bool) {
    (*pfn).set_rom(val);
}

/// Get OriginalPte.u.Soft.UsedPageTableEntries.
/// NOT the dedicated ULONG at offset 28 on 64-bit MMPFN.
#[inline]
pub unsafe fn mmpfn_used_page_table_entries(pfn: PMMPFN) -> ULONG {
    (*mmpfn_original_pte_ptr(pfn)).soft_used_page_table_entries()
}

/// Set OriginalPte.u.Soft.UsedPageTableEntries.
/// NOT the dedicated ULONG at offset 28 on 64-bit MMPFN.
#[inline]
pub unsafe fn mmpfn_set_used_page_table_entries(pfn: PMMPFN, val: ULONG) {
    (*mmpfn_original_pte_ptr(pfn)).set_soft_used_page_table_entries(val);
}

/// Get MMPTE.u.Soft.PageFileLow.
#[inline]
pub fn mmpte_soft_page_file_low(pte: &MMPTE) -> ULONG {
    #[cfg(target_arch = "aarch64")]
    { ((pte.Long >> 6) & 0xF) as ULONG }
    #[cfg(not(target_arch = "aarch64"))]
    { ((pte.Long >> 1) & 0xF) as ULONG }
}

// NOTE: Cannot add impl MMPFN / impl MMPTE here because they are defined in
// ntkrnl-sys crate (E0116). Instead, use the standalone mmpfn_* / mmpte_* helper
// functions above, which the source files call directly after automated transform.

// ============================================================================
// Additional type aliases for translated code
// ============================================================================

/// MMPDE is just MMPTE (same layout, different semantic meaning)
pub type MMPDE = MMPTE;

/// PEB, TEB — re-export bindgen types (size_of::<PEB/TEB> must be correct for MmCreatePeb/Teb)
pub use ntkrnl_sys::ddk::PEB;
pub use ntkrnl_sys::ddk::TEB;

/// _UNICODE_STRING — simplified for translated code
#[repr(C)]
pub struct _UNICODE_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: *mut u16,
}

// ============================================================================
// KeGetPcr stub (FORCEINLINE, not available in bindgen)
// ============================================================================

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn KeGetPcr() -> PVOID {
    let pcr: PVOID;
    core::arch::asm!("mrs {}, tpidr_el1", out(reg) pcr, options(nostack, readonly));
    pcr
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn KeGetPcr() -> PVOID {
    let pcr: PVOID;
    core::arch::asm!("mov {}, gs:[0x18]", out(reg) pcr, options(nostack, readonly));
    pcr
}

#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn KeGetPcr() -> PVOID {
    let pcr: PVOID;
    core::arch::asm!("mov {}, fs:[0x1C]", out(reg) pcr, options(nostack, readonly));
    pcr
}

// ============================================================================
// MMPPE / PMMPPE type aliases (PPE = PTE on 64-bit)
// ============================================================================

pub type MMPPE = MMPTE;
pub type PMMPPE = PMMPTE;

// ============================================================================
// _A3_TRACE! macro
// ============================================================================

#[macro_export]
macro_rules! _A3_TRACE {
    ($($arg:tt)*) => {};
}
pub struct PfnLockGuard { previous_irql: KIRQL }
impl PfnLockGuard {
    #[inline] pub unsafe fn acquire() -> Self { Self { previous_irql: MiAcquirePfnLock() } }
}
impl Drop for PfnLockGuard {
    #[inline] fn drop(&mut self) { unsafe { MiReleasePfnLock(self.previous_irql) } }
}
